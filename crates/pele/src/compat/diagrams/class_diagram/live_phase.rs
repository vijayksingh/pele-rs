//! Captured graph rows plus live ClassNode/member reads at shared renderer phases.
use super::{
    model::*,
    render::{self, Line, LiveMembers},
};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
pub(crate) struct ClassDetachedInput {
    limit: Option<PeleError>,
}
impl ClassDetachedInput {
    pub(super) fn new(error: Option<&PeleError>) -> Self {
        Self {
            limit: error.filter(|e| e.code == ErrorCode::Limit).cloned(),
        }
    }
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("class".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Class public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    classes: Vec<R::Value>,
    units: usize,
    work: usize,
    slots: HashMap<String, usize>,
    error: Option<PublicError>,
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
    fn keep(&mut self, slot: &str, n: usize) -> Result<(), PublicError> {
        let prior = self.slots.get(slot).copied().unwrap_or(0);
        self.units = self.units.saturating_add(n.saturating_sub(prior));
        if self.units > 16_000_000 {
            return Err(limit().into());
        }
        self.slots.insert(slot.into(), n.max(prior));
        Ok(())
    }
    fn text(&mut self, v: &R::Value, slot: &str) -> Result<String, PublicError> {
        let s = self.read.require_string(
            v,
            LiveStringUse {
                method: "includes",
                expression_receiver: "t",
            },
        )?;
        self.keep(slot, s.0.len())?;
        s.well_formed()
            .ok_or_else(|| invalid("Class rendering requires valid UTF16 text").into())
    }
    fn field(&mut self, v: &R::Value, key: &str, slot: &str) -> Result<String, PublicError> {
        let v = self.read.get(v, key)?;
        self.text(&v, &format!("{slot}/{key}"))
    }
    fn optional(
        &mut self,
        v: &R::Value,
        key: &str,
        slot: &str,
    ) -> Result<Option<String>, PublicError> {
        let v = self.read.get(v, key)?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Null | LiveValueKind::Undefined
        ) {
            Ok(None)
        } else {
            self.text(&v, &format!("{slot}/{key}")).map(Some)
        }
    }
    fn length(&mut self, v: &R::Value) -> Result<usize, PublicError> {
        if !self.read.is_array(v)? {
            return Err(invalid("Class drawable list must be an Array").into());
        }
        let n = self.read.get(v, "length")?;
        let n = self
            .read
            .number(&n)
            .ok_or_else(|| invalid("Invalid Class list length"))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > 1_000_000. {
            return Err(limit().into());
        }
        self.tick(n as usize)?;
        Ok(n as usize)
    }
    fn list(&mut self, v: &R::Value, key: &str, slot: &str) -> Result<Vec<String>, PublicError> {
        let a = self.read.get(v, key)?;
        let n = self.length(&a)?;
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            if !self.read.has_index(&a, i)? {
                continue;
            }
            let v = self.read.index(&a, i)?;
            out.push(self.text(&v, &format!("{slot}/{key}/{i}"))?)
        }
        Ok(out)
    }
    fn pairs(&mut self, root: &R::Value, key: &str) -> Result<Vec<R::Value>, PublicError> {
        let m = self.read.get(root, key)?;
        let values = self.read.snapshot_iterable(&m, 1_000_000)?;
        self.tick(values.len())?;
        Ok(values)
    }
    fn at(&self, i: usize) -> Result<R::Value, PublicError> {
        self.classes
            .get(i)
            .cloned()
            .ok_or_else(|| invalid("Class drawable identity unavailable").into())
    }
    fn record(&mut self, v: &R::Value, i: usize, phase: u8) -> Result<ClassNode, PublicError> {
        let mut c = ClassNode {
            id: String::new(),
            label: String::new(),
            generic: String::new(),
            css_classes: String::new(),
            members: vec![],
            methods: vec![],
            annotations: vec![],
            styles: vec![],
            parent: None,
            link: None,
            link_target: None,
            have_callback: None,
            tooltip: None,
        };
        let s = format!("class/{i}");
        if phase == 0 {
            c.id = self.field(v, "id", &s)?;
            c.label = self.field(v, "label", &s)?;
            c.parent = self.optional(v, "parent", &s)?;
        }
        if phase == 1 {
            c.css_classes = self.field(v, "cssClasses", &s)?;
            c.styles = self.list(v, "styles", &s)?;
            c.generic = self.field(v, "type", &s)?;
            c.label = self.field(v, "label", &s)?;
        }
        if phase == 2 {
            c.tooltip = self.optional(v, "tooltip", &s)?;
            c.link = self.optional(v, "link", &s)?;
            c.id = self.field(v, "id", &s)?;
            c.link_target = self.optional(v, "linkTarget", &s)?;
            c.css_classes = self.field(v, "cssClasses", &s)?;
        }
        Ok(c)
    }
    fn capture(&mut self) -> Result<ClassModel, PublicError> {
        self.units = 0;
        self.work = 0;
        self.slots.clear();
        self.classes.clear();
        let root = self.read.root();
        let mut m = ClassModel::new(None);
        // Keep original class handles; graph::build owns layout scalars only.
        for (i, pair) in self.pairs(&root, "namespaces")?.into_iter().enumerate() {
            let key = self.read.index(&pair, 0)?;
            let v = self.read.index(&pair, 1)?;
            let slot = format!("namespace/{i}");
            let key = self.text(&key, &format!("{slot}/key"))?;
            let explicit = self.read.get(&v, "explicit")?;
            m.namespaces.push((
                key,
                NamespaceNode {
                    id: self.field(&v, "id", &slot)?,
                    label: self.field(&v, "label", &slot)?,
                    classes: vec![],
                    notes: vec![],
                    children: vec![],
                    parent: self.optional(&v, "parent", &slot)?,
                    explicit: self.read.truthy(&explicit),
                },
            ));
        }
        for (i, pair) in self.pairs(&root, "classes")?.into_iter().enumerate() {
            let key = self.read.index(&pair, 0)?;
            let v = self.read.index(&pair, 1)?;
            let key = self.text(&key, &format!("class/{i}/key"))?;
            let c = self.record(&v, i, 0)?;
            m.classes.push((key, c));
            self.classes.push(v);
        }
        for (i, pair) in self.pairs(&root, "notes")?.into_iter().enumerate() {
            let key = self.read.index(&pair, 0)?;
            let v = self.read.index(&pair, 1)?;
            let slot = format!("note/{i}");
            let key = self.text(&key, &format!("{slot}/key"))?;
            let class_name = self.optional(&v, "class", &slot)?;
            let n = self.read.get(&v, "index")?;
            let n = self.read.number(&n).unwrap_or(0.);
            m.set_public_note_index(key.clone(), n);
            m.notes.push((
                key,
                ClassNote {
                    id: self.field(&v, "id", &slot)?,
                    class_name,
                    text: self.field(&v, "text", &slot)?,
                    index: if n.is_finite() && n >= 0. && n <= usize::MAX as f64 {
                        n as usize
                    } else {
                        0
                    },
                    parent: self.optional(&v, "parent", &slot)?,
                },
            ));
        }
        let interfaces = self.read.get(&root, "interfaces")?;
        let n = self.length(&interfaces)?;
        for i in 0..n {
            let v = self.read.index(&interfaces, i)?;
            let slot = format!("interface/{i}");
            m.interfaces.push(ClassInterface {
                id: self.field(&v, "id", &slot)?,
                label: self.field(&v, "label", &slot)?,
                class_id: self.field(&v, "classId", &slot)?,
            });
        }
        let relations = self.read.get(&root, "relations")?;
        let n = self.length(&relations)?;
        for i in 0..n {
            let v = self.read.index(&relations, i)?;
            let spec = self.read.get(&v, "relation")?;
            let slot = format!("relation/{i}");
            let mut end = |name| -> Result<RelationEnd, PublicError> {
                let value = self.read.get(&spec, name)?;
                if let Some(n) = self.read.number(&value) {
                    Ok(RelationEnd::RawNumber(n))
                } else {
                    self.text(&value, &format!("{slot}/{name}"))
                        .map(RelationEnd::None)
                }
            };
            let type1 = end("type1")?;
            let type2 = end("type2")?;
            let line = self.read.get(&spec, "lineType")?;
            m.relations.push(ClassRelation {
                id1: self.field(&v, "id1", &slot)?,
                id2: self.field(&v, "id2", &slot)?,
                relation_title1: self.field(&v, "relationTitle1", &slot)?,
                relation_title2: self.field(&v, "relationTitle2", &slot)?,
                title: self.optional(&v, "title", &slot)?,
                relation: super::model::RelationType {
                    type1,
                    type2,
                    line_type: u8::from(self.read.number(&line) == Some(1.)),
                },
            });
        }
        // Existing graph ancestry loops require acyclic captured parent links.
        let parents = m
            .namespaces
            .iter()
            .map(|(_, v)| (v.id.as_str(), v.parent.as_deref()))
            .collect::<HashMap<_, _>>();
        for id in parents.keys() {
            let mut path = HashSet::new();
            let mut at = Some(*id);
            while let Some(id) = at {
                self.tick(1)?;
                if !path.insert(id) {
                    return Err(limit().into());
                }
                at = parents.get(id).copied().flatten();
            }
        }
        m.direction = self.field(&root, "direction", "root")?;
        Ok(m)
    }
    fn map_error<T>(&mut self, result: Result<T, PublicError>) -> Result<T, PeleError> {
        result.map_err(|e| {
            let legacy = match &e {
                PublicError::Pele(e) => e.clone(),
                _ => invalid("Class live read failed"),
            };
            if self.error.is_none() {
                self.error = Some(e)
            }
            legacy
        })
    }
}
impl<R: LiveRenderRead> LiveMembers for Host<'_, R> {
    fn begin_draw(&mut self) -> Result<ClassModel, PeleError> {
        let r = self.capture();
        self.map_error(r)
    }
    fn class(&mut self, i: usize) -> Result<ClassNode, PeleError> {
        let r = (|| {
            let v = self.at(i)?;
            self.record(&v, i, 1)
        })();
        self.map_error(r)
    }
    fn paint(&mut self, i: usize) -> Result<ClassNode, PeleError> {
        let r = (|| {
            let v = self.at(i)?;
            self.record(&v, i, 2)
        })();
        self.map_error(r)
    }
    fn annotations(&mut self, i: usize) -> Result<Vec<String>, PeleError> {
        let r = (|| {
            let v = self.at(i)?;
            self.list(&v, "annotations", &format!("class/{i}"))
        })();
        self.map_error(r)
    }
    fn metadata(&mut self, key: &str) -> Result<Option<String>, PeleError> {
        let root = self.read.root();
        let r = self.optional(&root, key, "root");
        self.map_error(r)
    }
    fn definition(&mut self, key: &str) -> Result<Vec<String>, PeleError> {
        let r = (|| {
            let root = self.read.root();
            let map = self.read.get(&root, "styleClasses")?;
            let v = self.read.map_get_text(&map, key)?;
            if self.read.truthy(&v) {
                self.list(&v, "styles", &format!("style/{key}"))
            } else {
                Ok(vec![])
            }
        })();
        self.map_error(r)
    }
    fn lines(
        &mut self,
        i: usize,
        field: &str,
        size: f64,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Vec<Line>, PeleError> {
        let r = (|| {
            let class = self.at(i)?;
            let array = self.read.get(&class, field)?;
            let n = self.length(&array)?;
            let mut lines = Vec::new();
            for at in 0..n {
                if !self.read.has_index(&array, at)? {
                    continue;
                }
                let member = self.read.index(&array, at)?;
                let function = self.read.get(&member, "getDisplayDetails")?;
                let display = self.read.call(
                    &function,
                    &member,
                    &[],
                    host,
                    "member.getDisplayDetails is not a function",
                )?;
                let text = self.read.get(&display, "displayText")?;
                let text = self.text(&text, &format!("class/{i}/{field}/{at}/text"))?;
                let text = crate::compat::source::decode_entities(&text)?
                    .trim_matches(crate::compat::source::space)
                    .to_owned();
                let classifier = self.read.get(&member, "classifier")?;
                let italic = self
                    .read
                    .string_units(&classifier)?
                    .is_some_and(|v| v.is("*"));
                let width = crate::compat::text::measure(
                    host,
                    options,
                    &text,
                    size,
                    if italic { 2 } else { 0 },
                )?;
                let classifier = self.read.get(&member, "classifier")?;
                let classifier = self
                    .read
                    .string_units(&classifier)?
                    .map_or("", |v| {
                        if v.is("*") {
                            "*"
                        } else if v.is("$") {
                            "$"
                        } else {
                            ""
                        }
                    })
                    .to_owned();
                lines.push(Line {
                    text,
                    width,
                    classifier,
                });
            }
            Ok(lines)
        })();
        self.map_error(r)
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &ClassDetachedInput,
    read: &mut R,
    config: &Object,
    options: &RenderSettings,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    if let Some(e) = &input.limit {
        return Err(e.clone().into());
    }
    let mut host = Host {
        read,
        classes: vec![],
        units: 0,
        work: 0,
        slots: HashMap::new(),
        error: None,
    };
    let root = host.read.root();
    let mut policy = ClassModel::new(None);
    policy.direction = host.field(&root, "direction", "root")?;
    let result = render::render_live(&policy, config, options, resources, true, &mut host);
    match (result, host.error) {
        (_, Some(error)) => Err(error),
        (result, None) => result.map_err(Into::into),
    }
}
