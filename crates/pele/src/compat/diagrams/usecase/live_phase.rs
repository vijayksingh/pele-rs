//! Own a complete derived graph per draw; only root metadata stays live afterward.
use super::{
    graph,
    json::{JsonAtom, JsonValue},
    model::*,
    render::PhaseInput,
};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
pub(crate) struct UsecaseDetachedInput {
    limit: Option<PeleError>,
}
impl UsecaseDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
fn error(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("usecase".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Usecase public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
struct Host<'a, R> {
    read: &'a mut R,
    units: usize,
    work: usize,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 8_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn keep(&mut self, n: usize) -> Result<(), PublicError> {
        self.units = self.units.saturating_add(n);
        if self.units > 16_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn text(&mut self, v: &R::Value) -> Result<String, PublicError> {
        let s = self.read.require_string(
            v,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.keep(s.0.len())?;
        s.well_formed()
            .ok_or_else(|| error("Usecase rendering requires well-formed UTF16 text").into())
    }
    fn field(&mut self, v: &R::Value, key: &str) -> Result<String, PublicError> {
        let v = self.read.get(v, key)?;
        self.text(&v)
    }
    fn optional(&mut self, v: &R::Value, key: &str) -> Result<Option<String>, PublicError> {
        let v = self.read.get(v, key)?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Null | LiveValueKind::Undefined
        ) {
            Ok(None)
        } else {
            self.text(&v).map(Some)
        }
    }
    fn boolean(&mut self, v: &R::Value, key: &str) -> Result<bool, PublicError> {
        let v = self.read.get(v, key)?;
        Ok(self.read.truthy(&v))
    }
    fn length(&mut self, v: &R::Value) -> Result<usize, PublicError> {
        if !self.read.is_array(v)? {
            return Err(error("Usecase collection must be an Array").into());
        }
        let n = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&n)
            .ok_or_else(|| error("Usecase collection must be an Array"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > u32::MAX as f64 {
            return Err(error("Invalid Usecase Array length").into());
        }
        if n > 1_000_000. {
            return Err(limit().into());
        }
        self.tick(n as usize)?;
        Ok(n as usize)
    }
    fn list_value(&mut self, v: &R::Value) -> Result<Vec<String>, PublicError> {
        let n = self.length(v)?;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            if !self.read.has_index(v, i)? {
                return Err(error("Sparse Usecase render collections are not implemented").into());
            }
            let value = self.read.index(v, i)?;
            out.push(self.text(&value)?);
        }
        Ok(out)
    }
    fn list(&mut self, v: &R::Value, key: &str) -> Result<Vec<String>, PublicError> {
        let value = self.read.get(v, key)?;
        self.list_value(&value)
    }
    fn values(&mut self, root: &R::Value, key: &str) -> Result<Vec<R::Value>, PublicError> {
        let value = self.read.get(root, key)?;
        let out = self.read.snapshot_map_values(&value, 1_000_000)?;
        self.tick(out.len().saturating_mul(2))?;
        Ok(out)
    }
    fn entries(&mut self, value: &R::Value) -> Result<Vec<(R::Value, R::Value)>, PublicError> {
        let pairs = self.read.snapshot_iterable(value, 1_000_000)?;
        self.tick(pairs.len().saturating_mul(2))?;
        let mut out = Vec::with_capacity(pairs.len());
        for pair in pairs {
            out.push((self.read.index(&pair, 0)?, self.read.index(&pair, 1)?));
        }
        Ok(out)
    }
    fn atom(
        &mut self,
        value: R::Value,
        atoms: &mut Vec<JsonAtom>,
        ids: &mut HashMap<u64, usize>,
        pending: &mut Vec<(usize, R::Value)>,
    ) -> Result<usize, PublicError> {
        self.tick(1)?;
        if atoms.len() >= 1_000_000 {
            return Err(limit().into());
        }
        let scalar = match self.read.kind(&value) {
            LiveValueKind::Undefined => JsonAtom::Undefined,
            LiveValueKind::Null => JsonAtom::Null,
            LiveValueKind::Boolean => JsonAtom::Bool(self.read.truthy(&value)),
            LiveValueKind::Number => JsonAtom::Number(
                self.read
                    .number(&value)
                    .ok_or_else(|| error("Invalid Usecase JSON number"))?,
            ),
            LiveValueKind::String => JsonAtom::String(self.text(&value)?),
            LiveValueKind::Object => {
                let id = self
                    .read
                    .object_identity(&value)?
                    .ok_or_else(|| error("Usecase JSON requires object identity"))?;
                if let Some(index) = ids.get(&id) {
                    return Ok(*index);
                }
                let index = atoms.len();
                ids.insert(id, index);
                atoms.push(JsonAtom::Undefined);
                pending.push((index, value));
                return Ok(index);
            }
            _ => {
                return Err(
                    error("Usecase JSON rendering of functions/symbols is not implemented").into(),
                )
            }
        };
        let index = atoms.len();
        atoms.push(scalar);
        Ok(index)
    }
    fn json(&mut self, value: R::Value) -> Result<JsonValue, PublicError> {
        let mut atoms = Vec::new();
        let mut ids = HashMap::new();
        let mut pending = Vec::new();
        let root = self.atom(value, &mut atoms, &mut ids, &mut pending)?;
        while let Some((id, value)) = pending.pop() {
            self.tick(1)?;
            atoms[id] = if self.read.is_array(&value)? {
                let n = self.length(&value)?;
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    if self.read.has_index(&value, i)? {
                        let v = self.read.index(&value, i)?;
                        out.push(self.atom(v, &mut atoms, &mut ids, &mut pending)?);
                    } else {
                        if atoms.len() >= 1_000_000 {
                            return Err(limit().into());
                        }
                        out.push(atoms.len());
                        atoms.push(JsonAtom::Hole);
                    }
                }
                JsonAtom::Array(out)
            } else {
                let keys = self.read.own_keys(&value)?;
                self.tick(keys.len())?;
                let mut out = Vec::with_capacity(keys.len());
                for key in keys {
                    self.keep(key.0.len())?;
                    let key = key.well_formed().ok_or_else(|| {
                        error("Usecase rendering requires well-formed UTF16 keys")
                    })?;
                    let v = self.read.get(&value, &key)?;
                    let index = self.atom(v, &mut atoms, &mut ids, &mut pending)?;
                    out.push((key, index));
                }
                JsonAtom::Object(out)
            };
        }
        // Reject cyclic render projections before the shared flattening walker. DAG
        // references remain shared and its actual expansion receives a second budget.
        let mut seen = HashSet::new();
        let mut active = HashSet::new();
        let mut walk = vec![(root, false)];
        while let Some((id, leave)) = walk.pop() {
            self.tick(1)?;
            if leave {
                active.remove(&id);
                seen.insert(id);
                continue;
            }
            if active.contains(&id) {
                return Err(limit().into());
            }
            if seen.contains(&id) {
                continue;
            }
            active.insert(id);
            walk.push((id, true));
            match &atoms[id] {
                JsonAtom::Array(v) => walk.extend(v.iter().rev().map(|i| (*i, false))),
                JsonAtom::Object(v) => walk.extend(v.iter().rev().map(|(_, i)| (*i, false))),
                _ => {}
            }
        }
        Ok(JsonValue::from_render_atoms(atoms, root))
    }
    fn model(&mut self) -> Result<UsecaseModel, PublicError> {
        let root = self.read.root();
        let mut m = UsecaseModel::default();
        for v in self.values(&root, "actors")? {
            let id = self.field(&v, "id")?;
            m.actors.push((
                id.clone(),
                Actor {
                    id,
                    label: self.field(&v, "label")?,
                    label_type: self.field(&v, "labelType")?,
                    actor_type: self.field(&v, "type")?,
                    icon: self.optional(&v, "icon")?,
                    business: self.boolean(&v, "business")?,
                    stereotype: self.optional(&v, "stereotype")?,
                    parent_id: self.optional(&v, "parentId")?,
                    classes: self.list(&v, "classes")?,
                    styles: self.list(&v, "styles")?,
                },
            ));
        }
        for v in self.values(&root, "useCases")? {
            let id = self.field(&v, "id")?;
            m.use_cases.push((
                id.clone(),
                UseCase {
                    id,
                    label: self.field(&v, "label")?,
                    label_type: self.field(&v, "labelType")?,
                    shape: self.field(&v, "shape")?,
                    business: self.boolean(&v, "business")?,
                    stereotype: self.optional(&v, "stereotype")?,
                    parent_id: self.optional(&v, "parentId")?,
                    classes: self.list(&v, "classes")?,
                    styles: self.list(&v, "styles")?,
                },
            ));
        }
        for v in self.values(&root, "notes")? {
            let id = self.field(&v, "id")?;
            m.notes.push((
                id.clone(),
                UsecaseNote {
                    id,
                    target: self.field(&v, "target")?,
                    label: self.field(&v, "label")?,
                    label_type: self.field(&v, "labelType")?,
                },
            ));
        }
        for v in self.values(&root, "jsonNodes")? {
            let id = self.field(&v, "id")?;
            let json = self.read.get(&v, "value")?;
            let value = self.json(json)?;
            let order = self.read.get(&v, "propertyOrder")?;
            let mut property_order = Vec::new();
            for (key, values) in self.entries(&order)? {
                if self.read.kind(&key) == LiveValueKind::String {
                    property_order.push((self.text(&key)?, self.list_value(&values)?));
                }
            }
            super::json_budget::validate_json_rows(
                &value,
                &property_order,
                &mut self.units,
                &mut self.work,
            )?;
            m.json_nodes.push((
                id.clone(),
                UsecaseJsonNode {
                    id,
                    value,
                    property_order,
                    classes: self.list(&v, "classes")?,
                    styles: self.list(&v, "styles")?,
                },
            ));
        }
        for v in self.values(&root, "systemBoundaries")? {
            let id = self.field(&v, "id")?;
            m.system_boundaries.push((
                id.clone(),
                SystemBoundary {
                    id,
                    label: self.field(&v, "label")?,
                    label_type: self.field(&v, "labelType")?,
                    boundary_type: self.field(&v, "type")?,
                    members: vec![],
                    classes: self.list(&v, "classes")?,
                    styles: self.list(&v, "styles")?,
                },
            ));
        }
        let relations = self.read.get(&root, "relationships")?;
        let count = self.length(&relations)?;
        for i in 0..count {
            let v = self.read.index(&relations, i)?;
            let arrow = self.read.get(&v, "arrowType")?;
            let arrow_type = self
                .read
                .number(&arrow)
                .filter(|n| n.is_finite() && *n >= 0. && *n < 7. && n.fract() == 0.)
                .map_or(usize::MAX, |n| n as usize);
            let minlen = self.read.get(&v, "minlen")?;
            let minlen = self
                .read
                .number(&minlen)
                .filter(|n| n.is_finite() && *n >= 0. && n.fract() == 0. && *n <= usize::MAX as f64)
                .ok_or_else(|| error("Usecase minlen requires a nonnegative integer"))?
                as usize;
            m.relationships.push(Relationship {
                id: self.field(&v, "id")?,
                explicit_id: self.boolean(&v, "explicitId")?,
                source: self.field(&v, "source")?,
                target: self.field(&v, "target")?,
                relationship_type: self.field(&v, "type")?,
                arrow_type,
                label: self.optional(&v, "label")?,
                label_type: self.optional(&v, "labelType")?,
                minlen,
                classes: self.list(&v, "classes")?,
                styles: self.list(&v, "styles")?,
                animate: self.boolean(&v, "animate")?,
                animation: self.optional(&v, "animation")?,
            });
        }
        let definitions = self.read.get(&root, "classDefs")?;
        let mut styles_by_identity = HashMap::<u64, Arc<Vec<String>>>::new();
        for (key, v) in self.entries(&definitions)? {
            if self.read.kind(&key) != LiveValueKind::String {
                continue;
            }
            let id = self.text(&key)?;
            let styles = self.read.get(&v, "styles")?;
            let identity = self
                .read
                .object_identity(&styles)?
                .ok_or_else(|| error("Usecase styles must be an Array"))?;
            let styles = if let Some(styles) = styles_by_identity.get(&identity) {
                Arc::clone(styles)
            } else {
                let value = Arc::new(self.list_value(&styles)?);
                styles_by_identity.insert(identity, Arc::clone(&value));
                value
            };
            m.class_defs.push((id.clone(), ClassDef { id, styles }));
        }
        // Bound copied compiled-style expansion before graph::build retains strings.
        let defs: HashMap<_, _> = m
            .class_defs
            .iter()
            .map(|(key, d)| (key.as_str(), d))
            .collect();
        let groups = m
            .actors
            .iter()
            .map(|(_, n)| n.classes.as_slice())
            .chain(m.use_cases.iter().map(|(_, n)| n.classes.as_slice()))
            .chain(m.json_nodes.iter().map(|(_, n)| n.classes.as_slice()))
            .chain(
                m.system_boundaries
                    .iter()
                    .map(|(_, n)| n.classes.as_slice()),
            )
            .chain(m.relationships.iter().map(|n| n.classes.as_slice()))
            .chain(m.notes.iter().map(|_| &[][..]));
        for classes in groups {
            for name in std::iter::once("default").chain(classes.iter().map(String::as_str)) {
                if let Some(def) = defs.get(name) {
                    self.tick(def.styles.len())?;
                    for style in def.styles.iter() {
                        self.keep(style.encode_utf16().count())?;
                    }
                }
            }
        }
        if m.actors
            .len()
            .saturating_add(m.use_cases.len())
            .saturating_add(m.json_nodes.len())
            .saturating_add(m.system_boundaries.len())
            .saturating_add(m.notes.len())
            > 1_000_000
        {
            return Err(limit().into());
        }
        Ok(m)
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Error = PublicError;
    fn direction(&mut self) -> Result<String, PublicError> {
        let root = self.read.root();
        self.field(&root, "direction")
    }
    fn graph(&mut self) -> Result<(Vec<graph::Node>, Vec<graph::Edge>), PublicError> {
        self.units = 0;
        self.work = 0;
        let model = self.model()?;
        Ok(graph::build(&model))
    }
    fn title(&mut self) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        self.optional(&root, "title")
    }
    fn accessibility(&mut self) -> Result<(String, String), PublicError> {
        let root = self.read.root();
        let title = self.field(&root, "accTitle")?;
        let description = self.field(&root, "accDescr")?;
        Ok((title, description))
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    seed: &UsecaseDetachedInput,
    c: &Object,
    o: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &seed.limit {
        return Err(e.clone().into());
    }
    super::render::render_phase(
        &mut Host {
            read,
            units: 0,
            work: 0,
        },
        c,
        o,
        resources,
    )
}
