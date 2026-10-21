//! Mutable GitGraph plain objects, Pele 458322a5 (MIT kepano).
//! buildGit's builder is private closure state, not a returned prototype API.
//! Retain all identity graph data; derive only the existing renderer's cache.
use super::{GitCommit, GitGraph, GitModel};
use crate::compat::{
    public::*, DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
const fn field_schema(
    name: &'static str,
    kind: FieldKind,
    undefined: bool,
    optional: bool,
    nullable: bool,
) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        undefined,
        optional,
        nullable,
        flags: 7,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/gitGraph@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "GitGraph",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("type", FieldKind::String, false, false, false),
                field_schema("title", FieldKind::String, true, false, false),
                field_schema("accTitle", FieldKind::String, true, false, false),
                field_schema("accDescr", FieldKind::String, true, false, false),
                field_schema("direction", FieldKind::String, false, false, false),
                field_schema("commits", FieldKind::Map, false, false, false),
                field_schema("branches", FieldKind::Map, false, false, false),
                field_schema("lanes", FieldKind::Array, false, false, false),
                field_schema("currentBranch", FieldKind::String, false, false, false),
                field_schema("head", FieldKind::Record(RecordId(1)), false, false, true),
                field_schema("warnings", FieldKind::Array, false, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "GitCommit",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false, false),
                field_schema("message", FieldKind::String, false, false, false),
                field_schema("seq", FieldKind::Number, false, false, false),
                field_schema("type", FieldKind::Number, false, false, false),
                field_schema("tags", FieldKind::Array, false, false, false),
                field_schema("parents", FieldKind::Array, false, false, false),
                field_schema("branch", FieldKind::String, false, false, false),
                field_schema("customType", FieldKind::Number, true, true, false),
                field_schema("customId", FieldKind::Bool, false, true, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[],
    methods: &[],
};
pub struct GitGraphPublicState {
    graph: ModelGraph,
    model: Result<GitModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("gitGraph".into());
    e
}
fn limit(message: &str) -> PeleError {
    let mut e = invalid(message);
    e.code = ErrorCode::Limit;
    e
}
fn reference(value: Option<&Atom>) -> Result<u32, PeleError> {
    match value {
        Some(Atom::Ref(id)) => Ok(*id),
        _ => Err(invalid(
            "GitGraph active collection or record requires an object",
        )),
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn truthy(g: &ModelGraph, value: Option<&Atom>) -> bool {
    match value {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => false,
        Some(Atom::Number(n)) => *n != 0. && !n.is_nan(),
        Some(Atom::String(id)) => !g.strings[*id as usize].0.is_empty(),
        _ => true,
    }
}
fn array<'a>(g: &'a ModelGraph, id: u32) -> Result<Vec<&'a Atom>, PeleError> {
    let Some(Node::Array { length, properties }) = g.nodes.get(id as usize) else {
        return Err(invalid(
            "GitGraph active indexed collection must be an Array",
        ));
    };
    if *length > 1_000_000 {
        return Err(limit("GitGraph array exceeds native 1000000 work budget"));
    }
    let mut indexed = HashMap::new();
    for p in properties {
        // Array indices are at most ten ASCII digits. Never decode unrelated
        // multi-megabyte extra keys repeatedly through aliased arrays.
        let units = &g.strings[p.key as usize].0;
        if units.is_empty() || units.len() > 10 {
            continue;
        }
        let Some(k) = g.strings[p.key as usize].well_formed() else {
            continue;
        };
        if let Ok(index) = k.parse::<u32>() {
            if index < *length && index.to_string() == k {
                indexed.insert(index, &p.value);
            }
        }
    }
    if indexed.len() != *length as usize {
        return Err(invalid("GitGraph sparse active arrays are not implemented"));
    }
    Ok((0..*length).map(|i| indexed[&i]).collect())
}
struct Budget {
    text: usize,
    values: usize,
    scans: usize,
}
impl Budget {
    fn scan(&mut self, work: usize) -> Result<(), PeleError> {
        self.scans = self.scans.saturating_add(work);
        if self.scans > 32_000_000 {
            Err(limit(
                "GitGraph property reads exceed native 32000000 work budget",
            ))
        } else {
            Ok(())
        }
    }
    fn array<'a>(&mut self, g: &'a ModelGraph, id: u32) -> Result<Vec<&'a Atom>, PeleError> {
        self.scan(g.nodes[id as usize].properties().len())?;
        array(g, id)
    }
    fn values(&mut self, n: usize) -> Result<(), PeleError> {
        self.values = self.values.saturating_add(n);
        if self.values > 1_000_000 {
            Err(limit(
                "GitGraph imported collections exceed native 1000000 work budget",
            ))
        } else {
            Ok(())
        }
    }
    fn text(&mut self, g: &ModelGraph, value: Option<&Atom>) -> Result<String, PeleError> {
        let Some(Atom::String(id)) = value else {
            return Err(invalid(
                "GitGraph active text coercion for this runtime kind is not implemented",
            ));
        };
        let s = &g.strings[*id as usize];
        self.text = self.text.saturating_add(s.0.len());
        if self.text > 16_000_000 {
            return Err(limit(
                "GitGraph imported text exceeds native 16000000 code-unit work budget",
            ));
        }
        s.well_formed().ok_or_else(|| {
            invalid("GitGraph rendering of lone UTF-16 surrogates is not implemented")
        })
    }
    fn metadata(
        &mut self,
        g: &ModelGraph,
        key: &str,
        accessible: bool,
    ) -> Result<Option<String>, PeleError> {
        let a = g.field(key)?;
        if matches!(a, None | Some(Atom::Undefined)) || (accessible && !truthy(g, a)) {
            return Ok(None);
        }
        self.text(g, a).map(Some)
    }
    fn tags(&mut self, g: &ModelGraph, value: Option<&Atom>) -> Result<Vec<String>, PeleError> {
        if let Some(Atom::String(_)) = value {
            // String's default iterator yields code points, unlike indexed parents.
            let s = self.text(g, value)?;
            let mut tags = Vec::new();
            for c in s.chars() {
                self.values(1)?;
                tags.push(c.to_string());
            }
            return Ok(tags);
        }
        let id = reference(value)?;
        if field(g, id, "values").is_some() || field(g, id, "[Symbol.iterator]").is_some() {
            return Err(invalid(
                "GitGraph replaced collection iterators are not implemented",
            ));
        }
        let values = match &g.nodes[id as usize] {
            Node::Set { values, .. } => values.iter().collect(),
            Node::Array { .. } => self.array(g, id)?,
            _ => {
                return Err(invalid(
                    "GitGraph tags require an Array, Set or default String iterator",
                ))
            }
        };
        self.values(values.len())?;
        values.into_iter().map(|a| self.text(g, Some(a))).collect()
    }
}
// JS strict numeric symbol tests plus CLASSES fallback; written-message timing
// depends on original type < 3. These cache codes preserve both observations.
fn type_number(value: Option<&Atom>) -> Result<f64, PeleError> {
    match value {
        Some(Atom::Number(n)) => Ok(*n),
        None | Some(Atom::Undefined) => Ok(f64::NAN),
        Some(Atom::Null | Atom::Bool(false)) => Ok(0.),
        Some(Atom::Bool(true)) => Ok(1.),
        _ => Err(invalid(
            "GitGraph active commit type coercion for this runtime kind is not implemented",
        )),
    }
}
fn cache_type(value: Option<&Atom>) -> Result<u8, PeleError> {
    let n = type_number(value)?;
    if let Some(Atom::Number(_)) = value {
        if n >= 0. && n <= 4. && n.fract() == 0. {
            return Ok(n as u8);
        }
    }
    Ok(if n < 3. { 0 } else { 255 })
}
fn imported(g: &ModelGraph) -> Result<GitModel, PeleError> {
    let mut budget = Budget {
        text: 0,
        values: 0,
        scans: 0,
    };
    budget.scan(
        g.nodes[reference(Some(&g.root))? as usize]
            .properties()
            .len()
            .saturating_mul(20),
    )?;
    let lanes = budget.array(g, reference(g.field("lanes")?)?)?;
    budget.values(lanes.len())?;
    let lanes = lanes
        .into_iter()
        .map(|v| budget.text(g, Some(v)))
        .collect::<Result<Vec<_>, _>>()?;
    let id = reference(g.field("commits")?)?;
    if field(g, id, "values").is_some() {
        return Err(invalid("GitGraph replaced Map.values is not implemented"));
    }
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        return Err(invalid("GitGraph commits must be a Map"));
    };
    budget.values(entries.len())?;
    if lanes.is_empty() && !entries.is_empty() {
        return Err(invalid(
            "GitGraph commits without a lane are not supported by the native renderer",
        ));
    }
    let mut values = Vec::with_capacity(entries.len());
    for (_, value) in entries {
        let id = reference(Some(value))?;
        budget.scan(g.nodes[id as usize].properties().len().saturating_mul(16))?;
        let seq = match field(g, id, "seq") {
            Some(Atom::Number(n)) if !n.is_nan() => *n,
            _ => {
                return Err(invalid(
                    "GitGraph nonnumeric or NaN sequence sorting is not implemented",
                ))
            }
        };
        values.push((seq, id));
    }
    values.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut commit_ids = HashSet::new();
    for (_, id) in &values {
        commit_ids.insert(budget.text(g, field(g, *id, "id"))?);
    }
    let mut sentinel = "\0pele-native-unmatched-parent".to_string();
    while commit_ids.contains(&sentinel) || lanes.contains(&sentinel) {
        sentinel.push('\0');
    }
    let mut commits = Vec::with_capacity(values.len());
    for (seq, (_, id)) in values.into_iter().enumerate() {
        let kind = field(g, id, "type");
        let original = type_number(kind)?;
        let commit_type = cache_type(kind)?;
        let cid = budget.text(g, field(g, id, "id"))?;
        let message = if original < 3. {
            budget.text(g, field(g, id, "message"))?
        } else {
            String::new()
        };
        let tags = budget.tags(g, field(g, id, "tags"))?;
        let parents = budget.array(g, reference(field(g, id, "parents"))?)?;
        budget.values(parents.len())?;
        let parents = parents
            .into_iter()
            .map(|a| match a {
                Atom::String(index) if g.strings[*index as usize].well_formed().is_some() => {
                    budget.text(g, Some(a))
                }
                _ => Ok(sentinel.clone()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let branch = match field(g, id, "branch") {
            a @ Some(Atom::String(index)) if g.strings[*index as usize].well_formed().is_some() => {
                budget.text(g, a)?
            }
            _ => sentinel.clone(),
        };
        let custom_type = match field(g, id, "customType") {
            None | Some(Atom::Undefined | Atom::Null) => None,
            a => Some(cache_type(a)?),
        };
        commits.push((
            cid.clone(),
            GitCommit {
                id: cid,
                message,
                seq,
                commit_type,
                tags,
                parents,
                branch,
                custom_type,
                custom_id: Some(matches!(field(g, id, "customId"), Some(Atom::Bool(true)))),
            },
        ));
    }
    let direction = match g.field("direction")? {
        Some(Atom::String(id)) if g.strings[*id as usize].is("LR") => "LR",
        Some(Atom::String(id)) if g.strings[*id as usize].is("BT") => "BT",
        _ => "TB",
    };
    Ok(GitModel {
        diagram_type: DiagramType::GitGraph,
        title: budget.metadata(g, "title", false)?,
        acc_title: budget.metadata(g, "accTitle", true)?,
        acc_descr: budget.metadata(g, "accDescr", true)?,
        direction: direction.into(),
        commits,
        branches: vec![],
        lanes,
        current_branch: String::new(),
        head: None,
        warnings: vec![],
    })
}
fn array_node(out: &mut GraphWriter, values: impl IntoIterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(i, a)| out.property(&i.to_string(), a))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn strings(out: &mut GraphWriter, values: &[String]) -> u32 {
    let values = values
        .iter()
        .map(|s| Atom::String(out.string(s.as_str())))
        .collect::<Vec<_>>();
    array_node(out, values)
}
fn commit(out: &mut GraphWriter, c: &GitCommit) -> u32 {
    let id = out.string(c.id.as_str());
    let message = out.string(c.message.as_str());
    let tags = strings(out, &c.tags);
    let parents = strings(out, &c.parents);
    let branch = out.string(c.branch.as_str());
    let mut properties = vec![
        out.property("id", Atom::String(id)),
        out.property("message", Atom::String(message)),
        out.property("seq", Atom::Number(c.seq as f64)),
    ];
    if c.commit_type < 3 {
        properties.extend([
            out.property("type", Atom::Number(c.commit_type as f64)),
            out.property("tags", Atom::Ref(tags)),
            out.property("parents", Atom::Ref(parents)),
            out.property("branch", Atom::String(branch)),
        ]);
    } else {
        properties.extend([
            out.property("parents", Atom::Ref(parents)),
            out.property("branch", Atom::String(branch)),
            out.property("type", Atom::Number(c.commit_type as f64)),
        ]);
        if c.commit_type == 3 {
            properties.push(
                out.property(
                    "customType",
                    c.custom_type
                        .map_or(Atom::Undefined, |n| Atom::Number(n as f64)),
                ),
            );
            properties.push(out.property("customId", Atom::Bool(c.custom_id.unwrap_or(false))));
        }
        properties.push(out.property("tags", Atom::Ref(tags)));
    }
    out.node(Node::Object { properties })
}
impl PublicDiagramModule for GitGraph {
    type PublicState = GitGraphPublicState;
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::git_graph())
    }
    fn detached_public_render(
        &self,
        state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::git_graph(
            super::GitGraphDetachedInput::new(state.model.as_ref().err()),
        ))
    }
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(GitGraphPublicState { graph, model })
    }
    fn create_public(&self, m: &GitModel) -> Result<GitGraphPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut records = HashMap::new();
        let mut entries = vec![];
        for (key, c) in &m.commits {
            let id = commit(&mut out, c);
            records.insert(c.id.as_str(), id);
            let key = Atom::String(out.string(key.as_str()));
            entries.push((key, Atom::Ref(id)));
        }
        let commits = out.node(Node::Map {
            entries,
            properties: vec![],
        });
        let branches = m
            .branches
            .iter()
            .map(|(k, v)| {
                let k = Atom::String(out.string(k.as_str()));
                let v = v
                    .as_deref()
                    .map_or(Atom::Null, |v| Atom::String(out.string(v)));
                (k, v)
            })
            .collect();
        let branches = out.node(Node::Map {
            entries: branches,
            properties: vec![],
        });
        let lanes = strings(&mut out, &m.lanes);
        let warnings = strings(&mut out, &m.warnings);
        let head = match &m.head {
            None => Atom::Null,
            Some(h) => Atom::Ref(
                *records
                    .get(h.id.as_str())
                    .ok_or_else(|| invalid("GitGraph parsed head identity is unavailable"))?,
            ),
        };
        let kind = out.string("gitGraph");
        let mut properties = vec![out.property("type", Atom::String(kind))];
        for (k, v) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let a = v
                .as_deref()
                .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
            properties.push(out.property(k, a));
        }
        let direction = out.string(m.direction.as_str());
        let current = out.string(m.current_branch.as_str());
        properties.extend([
            out.property("direction", Atom::String(direction)),
            out.property("commits", Atom::Ref(commits)),
            out.property("branches", Atom::Ref(branches)),
            out.property("lanes", Atom::Ref(lanes)),
            out.property("currentBranch", Atom::String(current)),
            out.property("head", head),
            out.property("warnings", Atom::Ref(warnings)),
        ]);
        let root = out.node(Node::Object { properties });
        let graph = out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(GitGraphPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &GitGraphPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(
        &self,
        s: &mut GitGraphPublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        s.model = imported(&graph);
        s.graph = graph;
        Ok(())
    }
    fn render_public(
        &self,
        s: &mut GitGraphPublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, r)
    }
}
