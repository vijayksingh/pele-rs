//! Pinned Usecase plain mutable data model. No custom prototype methods exist.
use super::{
    json::{JsonAtom, JsonValue},
    model::*,
    Usecase,
};
use crate::compat::{
    diagrams::common::json_graph, public::*, DiagramModule, ErrorCode, Object, PeleError,
    RenderResources, RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
const ITEMS: usize = 1_000_000;
const WORK: usize = 8_000_000;
const TEXT: usize = 16_000_000;
const fn f(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        optional: false,
        undefined,
        nullable: false,
        flags: 7,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/usecase@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "UsecaseModel",
        kind: RecordKind::Object,
        prototype: None,
        members: &[],
        fields: &[
            f("type", FieldKind::String, false),
            f("actors", FieldKind::Map, false),
            f("useCases", FieldKind::Map, false),
            f("systemBoundaries", FieldKind::Map, false),
            f("relationships", FieldKind::Array, false),
            f("notes", FieldKind::Map, false),
            f("jsonNodes", FieldKind::Map, false),
            f("classDefs", FieldKind::Map, false),
            f("direction", FieldKind::String, false),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, false),
            f("accDescr", FieldKind::String, false),
            f("accDescription", FieldKind::String, false),
            f("source", FieldKind::String, false),
            f("headerSpan", FieldKind::Array, false),
            f("statements", FieldKind::Array, false),
        ],
    }],
    prototypes: &[],
    methods: &[],
};
pub struct UsecasePublicState {
    graph: ModelGraph,
    projected: Result<UsecaseModel, PeleError>,
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("usecase".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Usecase public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
fn get<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(&Atom::Undefined, |p| &p.value)
}
fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(i) = a {
        Ok(*i)
    } else {
        Err(error("Usecase field requires an object reference"))
    }
}
fn truth(a: &Atom, g: &ModelGraph) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(v) => *v,
        Atom::Number(v) => *v != 0. && !v.is_nan(),
        Atom::String(i) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
fn index(s: &JsString) -> Option<u32> {
    let s = s.well_formed()?;
    s.parse::<u32>()
        .ok()
        .filter(|i| *i < u32::MAX && i.to_string() == s)
}
struct Writer {
    g: ModelGraph,
    strings: HashMap<JsString, u32>,
    work: usize,
    units: usize,
}
impl Writer {
    fn new() -> Self {
        Self {
            g: ModelGraph {
                version: 1,
                root: Atom::Undefined,
                strings: vec![],
                nodes: vec![],
            },
            strings: HashMap::new(),
            work: 0,
            units: 0,
        }
    }
    fn tick(&mut self, n: usize) -> Result<(), PeleError> {
        self.work = self.work.checked_add(n).ok_or_else(limit)?;
        if self.work > WORK {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn string(&mut self, s: impl Into<JsString>) -> Result<Atom, PeleError> {
        let s = s.into();
        if let Some(i) = self.strings.get(&s) {
            return Ok(Atom::String(*i));
        }
        self.units = self.units.checked_add(s.0.len()).ok_or_else(limit)?;
        if self.units > TEXT || self.g.strings.len() >= ITEMS {
            return Err(limit());
        }
        let i = self.g.strings.len() as u32;
        self.strings.insert(s.clone(), i);
        self.g.strings.push(s);
        Ok(Atom::String(i))
    }
    fn node(&mut self, n: Node) -> Result<u32, PeleError> {
        if self.g.nodes.len() >= ITEMS {
            return Err(limit());
        }
        let i = self.g.nodes.len() as u32;
        self.g.nodes.push(n);
        Ok(i)
    }
    fn object(&mut self) -> Result<u32, PeleError> {
        self.node(Node::Object { properties: vec![] })
    }
    fn put(&mut self, id: u32, key: &str, value: Atom) -> Result<(), PeleError> {
        self.tick(1)?;
        let Atom::String(key) = self.string(key)? else {
            unreachable!()
        };
        self.g.nodes[id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        });
        Ok(())
    }
    fn record(&mut self, fields: Vec<(&str, Atom)>) -> Result<Atom, PeleError> {
        let id = self.object()?;
        for (k, v) in fields {
            self.put(id, k, v)?
        }
        Ok(Atom::Ref(id))
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<Atom, PeleError> {
        self.tick(values.len())?;
        let mut properties = Vec::with_capacity(values.len());
        for (i, value) in values.into_iter().enumerate() {
            let Atom::String(key) = self.string(i.to_string().as_str())? else {
                unreachable!()
            };
            properties.push(Property {
                key,
                value,
                flags: 7,
            })
        }
        let length = properties.len() as u32;
        Ok(Atom::Ref(self.node(Node::Array { length, properties })?))
    }
    fn strings(&mut self, values: &[String]) -> Result<Atom, PeleError> {
        if self.work.saturating_add(values.len()) > WORK {
            return Err(limit());
        }
        let values = values
            .iter()
            .map(|v| self.string(v.as_str()))
            .collect::<Result<_, _>>()?;
        self.array(values)
    }
    fn span(&mut self, s: Span) -> Result<Atom, PeleError> {
        self.array(s.into_iter().map(|v| Atom::Number(v as f64)).collect())
    }
    fn spans(&mut self, s: &[Span]) -> Result<Atom, PeleError> {
        let values = s.iter().map(|v| self.span(*v)).collect::<Result<_, _>>()?;
        self.array(values)
    }
    fn map(&mut self, entries: Vec<(Atom, Atom)>) -> Result<Atom, PeleError> {
        self.tick(entries.len().saturating_mul(2))?;
        Ok(Atom::Ref(self.node(Node::Map {
            entries,
            properties: vec![],
        })?))
    }
    fn opt(&mut self, s: Option<&String>) -> Result<Atom, PeleError> {
        s.map_or(Ok(Atom::Undefined), |s| self.string(s.as_str()))
    }
    fn metadata(&mut self, values: &[MetadataOccurrence]) -> Result<Atom, PeleError> {
        let mut out = Vec::with_capacity(values.len());
        for v in values {
            let key = self.string(v.key.as_str())?;
            let span = self.span(v.span)?;
            let key_span = self.span(v.key_span)?;
            let value_span = self.span(v.value_span)?;
            out.push(self.record(vec![
                ("key", key),
                ("span", span),
                ("keySpan", key_span),
                ("valueSpan", value_span),
            ])?)
        }
        self.array(out)
    }
    fn occurrence(
        &mut self,
        v: &NodeOccurrence,
        shared_classes: Option<Atom>,
    ) -> Result<Atom, PeleError> {
        let id = self.string(v.id.as_str())?;
        let span = self.span(v.span)?;
        let id_span = if v.label_span.is_none() {
            span.clone()
        } else {
            self.span(v.id_span)?
        };
        let mut out = vec![("id", id), ("span", span), ("idSpan", id_span)];
        if let Some(s) = v.label_span {
            out.push(("labelSpan", self.span(s)?))
        }
        if let Some(d) = v.defines {
            out.push(("defines", Atom::Bool(d)))
        }
        if let Some(s) = v.stereotype_span {
            out.push(("stereotypeSpan", self.span(s)?))
        }
        if let Some(m) = &v.metadata {
            out.push(("metadata", self.metadata(m)?))
        }
        if let Some(s) = &v.class_spans {
            out.push((
                "classSpans",
                if let Some(v) = shared_classes {
                    v
                } else {
                    self.spans(s)?
                },
            ))
        }
        self.record(out)
    }
    fn statements(&mut self, values: &[GraphStatement]) -> Result<Atom, PeleError> {
        let mut pending = Vec::new();
        fn allocate<'a>(
            w: &mut Writer,
            values: &'a [GraphStatement],
            pending: &mut Vec<(u32, &'a GraphStatement)>,
        ) -> Result<Atom, PeleError> {
            let mut out = Vec::with_capacity(values.len());
            for v in values {
                let id = w.object()?;
                pending.push((id, v));
                out.push(Atom::Ref(id))
            }
            w.array(out)
        }
        let result = allocate(self, values, &mut pending)?;
        while let Some((id, v)) = pending.pop() {
            let kind = self.string(v.kind.as_str())?;
            self.put(id, "kind", kind)?;
            let span = self.span(v.span)?;
            self.put(id, "span", span.clone())?;
            let classes = if v.kind == "json" {
                Some(self.spans(v.class_spans.as_deref().unwrap_or(&[]))?)
            } else {
                None
            };
            let metadata = if let Some(m) = &v.metadata {
                Some(self.metadata(m)?)
            } else {
                None
            };
            let nodes = if let Some(nodes) = &v.nodes {
                let nodes = nodes
                    .iter()
                    .map(|n| self.occurrence(n, classes.clone()))
                    .collect::<Result<_, _>>()?;
                Some(self.array(nodes)?)
            } else {
                None
            };
            let edges = if let Some(edges) = &v.edges {
                let mut out = vec![];
                for e in edges {
                    let name = self.string(e.id.as_str())?;
                    let mut fields = vec![("id", name), ("span", span.clone())];
                    if let Some(s) = e.id_span {
                        fields.push(("idSpan", self.span(s)?))
                    }
                    if let Some(s) = e.label_span {
                        fields.push(("labelSpan", self.span(s)?))
                    }
                    if let Some(m) = &e.metadata {
                        fields.push((
                            "metadata",
                            if v.kind == "edgeMetadata" {
                                metadata.clone().unwrap_or(Atom::Undefined)
                            } else {
                                self.metadata(m)?
                            },
                        ))
                    }
                    out.push(self.record(fields)?)
                }
                Some(self.array(out)?)
            } else {
                None
            };
            let mut fields = Vec::new();
            if matches!(v.kind.as_str(), "note" | "classDef" | "classAssign") {
                if let Some(s) = &v.reference {
                    fields.push(("ref", self.string(s.as_str())?))
                }
                if let Some(s) = v.ref_span {
                    fields.push(("refSpan", self.span(s)?))
                }
            }
            if v.kind == "group" {
                if let Some(s) = &v.group {
                    fields.push(("group", self.string(s.as_str())?))
                }
                for (key, s) in [
                    ("idSpan", v.id_span),
                    ("titleSpan", v.title_span),
                    ("endSpan", v.end_span),
                ] {
                    if let Some(s) = s {
                        fields.push((key, self.span(s)?))
                    }
                }
                if let Some(s) = &v.class_spans {
                    fields.push(("classSpans", self.spans(s)?))
                }
                if let Some(children) = &v.children {
                    fields.push(("children", allocate(self, children, &mut pending)?))
                }
            }
            if let Some(nodes) = nodes {
                fields.push(("nodes", nodes))
            }
            if v.kind == "edgeMetadata" {
                if let Some(m) = metadata.clone() {
                    fields.push(("metadata", m))
                }
            }
            if let Some(edges) = edges {
                fields.push(("edges", edges))
            }
            if let Some(classes) = classes {
                fields.push(("classSpans", classes))
            }
            if v.kind != "edgeMetadata" {
                if let Some(m) = metadata {
                    fields.push(("metadata", m))
                }
            }
            for (k, v) in fields {
                self.put(id, k, v)?
            }
        }
        Ok(result)
    }
    fn merge_json(&mut self, g: ModelGraph) -> Result<Atom, PeleError> {
        fn value(
            w: &mut Writer,
            g: &ModelGraph,
            a: &Atom,
            ids: &mut HashMap<u32, u32>,
            pending: &mut Vec<u32>,
        ) -> Result<Atom, PeleError> {
            Ok(match a {
                Atom::String(i) => w.string(g.strings[*i as usize].clone())?,
                Atom::Ref(i) => {
                    let id = if let Some(id) = ids.get(i) {
                        *id
                    } else {
                        let id = w.object()?;
                        ids.insert(*i, id);
                        pending.push(*i);
                        id
                    };
                    Atom::Ref(id)
                }
                v => v.clone(),
            })
        }
        let mut ids = HashMap::new();
        let mut pending = vec![];
        let root = value(self, &g, &g.root, &mut ids, &mut pending)?;
        while let Some(old) = pending.pop() {
            let id = ids[&old];
            let mut props = vec![];
            for p in g.nodes[old as usize].properties() {
                self.tick(1)?;
                let Atom::String(key) = self.string(g.strings[p.key as usize].clone())? else {
                    unreachable!()
                };
                props.push(Property {
                    key,
                    value: value(self, &g, &p.value, &mut ids, &mut pending)?,
                    flags: p.flags,
                })
            }
            self.g.nodes[id as usize] = match &g.nodes[old as usize] {
                Node::Array { length, .. } => Node::Array {
                    length: *length,
                    properties: props,
                },
                _ => Node::Object { properties: props },
            }
        }
        Ok(root)
    }
}
fn estimate(m: &UsecaseModel) -> Result<(), PeleError> {
    let mut work = 0usize;
    let mut add = |n: usize| {
        work = work.saturating_add(n);
        if work > WORK {
            Err(limit())
        } else {
            Ok(())
        }
    };
    for (_, v) in &m.actors {
        add(7 + v.classes.len() + v.styles.len())?
    }
    for (_, v) in &m.use_cases {
        add(7 + v.classes.len() + v.styles.len())?
    }
    for (_, v) in &m.system_boundaries {
        add(7 + v.members.len() + v.classes.len() + v.styles.len())?
    }
    for (_, v) in &m.class_defs {
        add(2 + v.styles.len())?
    }
    Ok(())
}
fn new_graph(m: &UsecaseModel) -> Result<ModelGraph, PeleError> {
    estimate(m)?;
    let mut w = Writer::new();
    let root = w.object()?;
    w.g.root = Atom::Ref(root);
    let kind = w.string("usecase")?;
    w.put(root, "type", kind)?;
    let mut actors = vec![];
    for (key, v) in &m.actors {
        let id = w.string(v.id.as_str())?;
        let label = w.string(v.label.as_str())?;
        let lt = w.string(v.label_type.as_str())?;
        let classes = w.strings(&v.classes)?;
        let kind = w.string(v.actor_type.as_str())?;
        let styles = w.strings(&v.styles)?;
        let mut fields = vec![
            ("id", id),
            ("label", label),
            ("labelType", lt),
            ("business", Atom::Bool(v.business)),
            ("classes", classes),
            ("type", kind),
            ("styles", styles),
        ];
        if let Some(s) = &v.icon {
            fields.push(("icon", w.string(s.as_str())?))
        }
        if let Some(s) = &v.stereotype {
            fields.push(("stereotype", w.string(s.as_str())?))
        }
        if let Some(s) = &v.parent_id {
            fields.push(("parentId", w.string(s.as_str())?))
        }
        let object = w.record(fields)?;
        actors.push((w.string(key.as_str())?, object))
    }
    let actors = w.map(actors)?;
    w.put(root, "actors", actors)?;
    let mut cases = vec![];
    for (key, v) in &m.use_cases {
        let id = w.string(v.id.as_str())?;
        let label = w.string(v.label.as_str())?;
        let lt = w.string(v.label_type.as_str())?;
        let classes = w.strings(&v.classes)?;
        let shape = w.string(v.shape.as_str())?;
        let styles = w.strings(&v.styles)?;
        let mut fields = vec![
            ("id", id),
            ("label", label),
            ("labelType", lt),
            ("business", Atom::Bool(v.business)),
            ("classes", classes),
            ("shape", shape),
            ("styles", styles),
        ];
        if let Some(s) = &v.stereotype {
            fields.push(("stereotype", w.string(s.as_str())?))
        }
        if let Some(s) = &v.parent_id {
            fields.push(("parentId", w.string(s.as_str())?))
        }
        let object = w.record(fields)?;
        cases.push((w.string(key.as_str())?, object))
    }
    let cases = w.map(cases)?;
    w.put(root, "useCases", cases)?;
    let mut groups = vec![];
    for (key, v) in &m.system_boundaries {
        let id = w.string(v.id.as_str())?;
        let label = w.string(v.label.as_str())?;
        let lt = w.string(v.label_type.as_str())?;
        let kind = w.string(v.boundary_type.as_str())?;
        let members = w.strings(&v.members)?;
        let classes = w.strings(&v.classes)?;
        let styles = w.strings(&v.styles)?;
        let object = w.record(vec![
            ("id", id),
            ("label", label),
            ("labelType", lt),
            ("type", kind),
            ("members", members),
            ("classes", classes),
            ("styles", styles),
        ])?;
        groups.push((w.string(key.as_str())?, object))
    }
    let groups = w.map(groups)?;
    w.put(root, "systemBoundaries", groups)?;
    let mut relationships = vec![];
    for v in &m.relationships {
        let id = w.string(v.id.as_str())?;
        let source = w.string(v.source.as_str())?;
        let target = w.string(v.target.as_str())?;
        let kind = w.string(v.relationship_type.as_str())?;
        let classes = w.strings(&v.classes)?;
        let styles = w.strings(&v.styles)?;
        let mut fields = vec![
            ("id", id),
            ("explicitId", Atom::Bool(v.explicit_id)),
            ("source", source),
            ("target", target),
            ("type", kind),
            ("arrowType", Atom::Number(v.arrow_type as f64)),
            ("minlen", Atom::Number(v.minlen as f64)),
            ("classes", classes),
            ("styles", styles),
            ("animate", Atom::Bool(v.animate)),
        ];
        if let Some(s) = &v.label {
            fields.push(("label", w.string(s.as_str())?))
        }
        if let Some(s) = &v.label_type {
            fields.push(("labelType", w.string(s.as_str())?))
        }
        if let Some(s) = &v.animation {
            fields.push(("animation", w.string(s.as_str())?))
        }
        relationships.push(w.record(fields)?)
    }
    let relationships = w.array(relationships)?;
    w.put(root, "relationships", relationships)?;
    let mut notes = vec![];
    for (key, v) in &m.notes {
        let id = w.string(v.id.as_str())?;
        let target = w.string(v.target.as_str())?;
        let label = w.string(v.label.as_str())?;
        let kind = w.string(v.label_type.as_str())?;
        let object = w.record(vec![
            ("id", id),
            ("target", target),
            ("label", label),
            ("labelType", kind),
        ])?;
        notes.push((w.string(key.as_str())?, object))
    }
    let notes = w.map(notes)?;
    w.put(root, "notes", notes)?;
    let mut json = vec![];
    for (key, v) in &m.json_nodes {
        let (value, authored_order) = if let Some(source) = &v.value.authored {
            let (graph, order) =
                json_graph::parse_graph_with_order(&JsString::from(source.as_ref()))
                    .map_err(|_| limit())?;
            (w.merge_json(graph)?, Some(order))
        } else {
            (export_json_atoms(&mut w, &v.value)?, None)
        };
        let mut order = vec![];
        if let Some(authored) = authored_order {
            for (pointer, keys) in authored {
                let pointer = w.string(pointer)?;
                let keys = keys
                    .into_iter()
                    .map(|key| w.string(key))
                    .collect::<Result<_, _>>()?;
                let keys = w.array(keys)?;
                order.push((pointer, keys))
            }
        } else {
            for (k, keys) in &v.property_order {
                let k = w.string(k.as_str())?;
                let keys = w.strings(keys)?;
                order.push((k, keys))
            }
        }
        let order = w.map(order)?;
        let id = w.string(v.id.as_str())?;
        let classes = w.strings(&v.classes)?;
        let styles = w.strings(&v.styles)?;
        let object = w.record(vec![
            ("id", id),
            ("value", value),
            ("propertyOrder", order),
            ("classes", classes),
            ("styles", styles),
        ])?;
        json.push((w.string(key.as_str())?, object))
    }
    let json = w.map(json)?;
    w.put(root, "jsonNodes", json)?;
    let mut defs = vec![];
    for (key, v) in &m.class_defs {
        let id = w.string(v.id.as_str())?;
        let styles = w.strings(&v.styles)?;
        let object = w.record(vec![("id", id), ("styles", styles)])?;
        defs.push((w.string(key.as_str())?, object))
    }
    let defs = w.map(defs)?;
    w.put(root, "classDefs", defs)?;
    for (key, value) in [
        ("direction", w.string(m.direction.as_str())?),
        ("title", w.opt(m.title.as_ref())?),
        ("accTitle", w.string(m.acc_title.as_str())?),
        ("accDescr", w.string(m.acc_descr.as_str())?),
        ("accDescription", w.string(m.acc_description.as_str())?),
        ("source", w.string(m.source.as_str())?),
    ] {
        w.put(root, key, value)?
    }
    let header = w.span(m.header_span)?;
    w.put(root, "headerSpan", header)?;
    let statements = w.statements(&m.statements)?;
    w.put(root, "statements", statements)?;
    w.g.validate()?;
    Ok(w.g)
}
fn export_json_atoms(w: &mut Writer, value: &JsonValue) -> Result<Atom, PeleError> {
    fn atom(
        w: &mut Writer,
        value: &JsonValue,
        i: usize,
        ids: &mut HashMap<usize, u32>,
        pending: &mut Vec<usize>,
    ) -> Result<Atom, PeleError> {
        Ok(match &value.atoms[i] {
            JsonAtom::String(s) => w.string(s.as_str())?,
            JsonAtom::Number(n) => Atom::Number(*n),
            JsonAtom::Bool(v) => Atom::Bool(*v),
            JsonAtom::Null => Atom::Null,
            JsonAtom::Undefined | JsonAtom::Hole => Atom::Undefined,
            _ => {
                let id = if let Some(id) = ids.get(&i) {
                    *id
                } else {
                    let id = w.object()?;
                    ids.insert(i, id);
                    pending.push(i);
                    id
                };
                Atom::Ref(id)
            }
        })
    }
    let mut ids = HashMap::new();
    let mut pending = vec![];
    let root = atom(w, value, value.root, &mut ids, &mut pending)?;
    while let Some(old) = pending.pop() {
        let id = ids[&old];
        let mut fields = vec![];
        let length = match &value.atoms[old] {
            JsonAtom::Object(entries) => {
                for (k, v) in entries {
                    let key = reference_string(w, k)?;
                    fields.push(Property {
                        key,
                        value: atom(w, value, *v, &mut ids, &mut pending)?,
                        flags: 7,
                    })
                }
                fields.sort_by(|a, b| {
                    match (
                        index(&w.g.strings[a.key as usize]),
                        index(&w.g.strings[b.key as usize]),
                    ) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                None
            }
            JsonAtom::Array(values) => {
                for (i, v) in values.iter().enumerate() {
                    if matches!(value.atoms[*v], JsonAtom::Hole) {
                        continue;
                    }
                    let key = reference_string(w, &i.to_string())?;
                    fields.push(Property {
                        key,
                        value: atom(w, value, *v, &mut ids, &mut pending)?,
                        flags: 7,
                    })
                }
                Some(values.len() as u32)
            }
            _ => unreachable!(),
        };
        w.tick(fields.len())?;
        w.g.nodes[id as usize] = if let Some(length) = length {
            Node::Array {
                length,
                properties: fields,
            }
        } else {
            Node::Object { properties: fields }
        }
    }
    Ok(root)
}
fn reference_string(w: &mut Writer, s: &str) -> Result<u32, PeleError> {
    let Atom::String(i) = w.string(s)? else {
        unreachable!()
    };
    Ok(i)
}
struct Import<'a> {
    g: &'a ModelGraph,
    units: usize,
    work: usize,
}
impl Import<'_> {
    fn tick(&mut self, n: usize) -> Result<(), PeleError> {
        self.work = self.work.saturating_add(n);
        if self.work > WORK {
            Err(limit())
        } else {
            Ok(())
        }
    }
    fn text(&mut self, a: &Atom) -> Result<String, PeleError> {
        let s = self.g.text(a)?;
        self.units = self.units.saturating_add(s.0.len());
        if self.units > TEXT {
            return Err(limit());
        }
        s.well_formed()
            .ok_or_else(|| error("Usecase rendering requires well-formed UTF16 text"))
    }
    fn field(&mut self, id: u32, key: &str) -> Result<String, PeleError> {
        self.text(get(self.g, id, key))
    }
    fn opt(&mut self, id: u32, key: &str) -> Result<Option<String>, PeleError> {
        let a = get(self.g, id, key);
        if matches!(a, Atom::Undefined | Atom::Null) {
            Ok(None)
        } else {
            self.text(a).map(Some)
        }
    }
    fn dense(&mut self, a: &Atom) -> Result<Vec<Atom>, PeleError> {
        let id = reference(a)?;
        let Node::Array { length, properties } = &self.g.nodes[id as usize] else {
            return Err(error("Usecase collection must be an Array"));
        };
        if *length as usize > ITEMS {
            return Err(limit());
        }
        self.tick(*length as usize)?;
        let mut out = vec![None; *length as usize];
        for p in properties {
            if let Some(i) = index(&self.g.strings[p.key as usize]) {
                if (i as usize) < out.len() {
                    out[i as usize] = Some(p.value.clone())
                }
            }
        }
        out.into_iter()
            .map(|v| {
                v.ok_or_else(|| error("Sparse Usecase render collections are not implemented"))
            })
            .collect()
    }
    fn strings(&mut self, a: &Atom) -> Result<Vec<String>, PeleError> {
        let values = self.dense(a)?;
        values.iter().map(|v| self.text(v)).collect()
    }
    fn list(&mut self, id: u32, key: &str) -> Result<Vec<String>, PeleError> {
        self.strings(get(self.g, id, key))
    }
    fn entries(&mut self, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        let id = reference(a)?;
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            return Err(error("Usecase collection must be a Map"));
        };
        self.tick(entries.len().saturating_mul(2))?;
        Ok(entries.clone())
    }
    fn count(&self, a: &Atom) -> Result<usize, PeleError> {
        match a {
            Atom::Number(n)
                if n.is_finite() && *n >= 0. && n.fract() == 0. && *n <= (usize::MAX as f64) =>
            {
                Ok(*n as usize)
            }
            _ => Err(error("Usecase minlen requires a nonnegative integer")),
        }
    }
    fn json(&mut self, root: &Atom) -> Result<JsonValue, PeleError> {
        fn atom(
            i: &mut Import<'_>,
            a: &Atom,
            atoms: &mut Vec<JsonAtom>,
            ids: &mut HashMap<u32, usize>,
            pending: &mut Vec<u32>,
        ) -> Result<usize, PeleError> {
            if atoms.len() >= ITEMS {
                return Err(limit());
            }
            if let Atom::Ref(id) = a {
                if let Some(index) = ids.get(id) {
                    return Ok(*index);
                }
                let index = atoms.len();
                atoms.push(JsonAtom::Null);
                ids.insert(*id, index);
                pending.push(*id);
                return Ok(index);
            }
            let v = match a {
                Atom::String(_) => JsonAtom::String(i.text(a)?),
                Atom::Number(n) => JsonAtom::Number(*n),
                Atom::Bool(v) => JsonAtom::Bool(*v),
                Atom::Null => JsonAtom::Null,
                Atom::Undefined => JsonAtom::Undefined,
                _ => {
                    return Err(error(
                        "Usecase JSON callable/host scalar coercion is not implemented",
                    ))
                }
            };
            let index = atoms.len();
            atoms.push(v);
            Ok(index)
        }
        let mut atoms = Vec::new();
        let mut ids = HashMap::new();
        let mut pending = Vec::new();
        let root = atom(self, root, &mut atoms, &mut ids, &mut pending)?;
        while let Some(old) = pending.pop() {
            self.tick(1)?;
            let index = ids[&old];
            let node = &self.g.nodes[old as usize];
            atoms[index] = if let Node::Array { length, properties } = node {
                if *length as usize > ITEMS {
                    return Err(limit());
                }
                self.tick(*length as usize)?;
                let mut values = vec![None; *length as usize];
                for p in properties {
                    if let Some(k) = index_key(self.g, p.key) {
                        if (k as usize) < values.len() {
                            values[k as usize] = Some(p.value.clone())
                        }
                    }
                }
                let mut out = Vec::with_capacity(values.len());
                for v in values {
                    out.push(if let Some(v) = v {
                        atom(self, &v, &mut atoms, &mut ids, &mut pending)?
                    } else {
                        let index = atoms.len();
                        atoms.push(JsonAtom::Hole);
                        index
                    })
                }
                JsonAtom::Array(out)
            } else {
                let mut props = node
                    .properties()
                    .iter()
                    .filter(|p| p.flags & 1 != 0)
                    .collect::<Vec<_>>();
                props.sort_by(
                    |a, b| match (index_key(self.g, a.key), index_key(self.g, b.key)) {
                        (Some(a), Some(b)) => a.cmp(&b),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        _ => std::cmp::Ordering::Equal,
                    },
                );
                self.tick(props.len())?;
                let mut out = vec![];
                for p in props {
                    let key = self.text(&Atom::String(p.key))?;
                    let v = atom(self, &p.value, &mut atoms, &mut ids, &mut pending)?;
                    out.push((key, v))
                }
                JsonAtom::Object(out)
            };
            if atoms.len() > ITEMS {
                return Err(limit());
            }
        }
        // The original row walker has no cycle semantics. Bound cycles before calling
        // it, while retaining arbitrary unused cycles in the authoritative arena.
        enum Visit {
            Node(usize),
            Leave(usize),
        }
        let mut active = HashSet::new();
        let mut seen = HashSet::new();
        let mut walk = vec![Visit::Node(root)];
        while let Some(v) = walk.pop() {
            self.tick(1)?;
            match v {
                Visit::Leave(id) => {
                    active.remove(&id);
                    seen.insert(id);
                }
                Visit::Node(id) => {
                    if active.contains(&id) {
                        return Err(limit());
                    }
                    if seen.contains(&id) {
                        continue;
                    }
                    active.insert(id);
                    walk.push(Visit::Leave(id));
                    match &atoms[id] {
                        JsonAtom::Object(v) => {
                            walk.extend(v.iter().rev().map(|(_, i)| Visit::Node(*i)))
                        }
                        JsonAtom::Array(v) => walk.extend(v.iter().rev().map(|i| Visit::Node(*i))),
                        _ => {}
                    }
                }
            }
        }
        Ok(JsonValue::from_render_atoms(atoms, root))
    }
}
impl Import<'_> {
    fn validate_json_rows(
        &mut self,
        value: &JsonValue,
        order: &[(String, Vec<String>)],
    ) -> Result<(), PeleError> {
        super::json_budget::validate_json_rows(value, order, &mut self.units, &mut self.work)
    }
}

fn index_key(g: &ModelGraph, key: u32) -> Option<u32> {
    index(&g.strings[key as usize])
}
fn project(g: &ModelGraph) -> Result<UsecaseModel, PeleError> {
    let mut i = Import {
        g,
        units: 0,
        work: 0,
    };
    let root = reference(&g.root)?;
    let mut m = UsecaseModel::default();
    for (_, value) in i.entries(get(g, root, "actors"))? {
        let id = reference(&value)?;
        let name = i.field(id, "id")?;
        m.actors.push((
            name.clone(),
            Actor {
                id: name,
                label: i.field(id, "label")?,
                label_type: i.field(id, "labelType")?,
                actor_type: i.field(id, "type")?,
                icon: i.opt(id, "icon")?,
                business: truth(get(g, id, "business"), g),
                stereotype: i.opt(id, "stereotype")?,
                parent_id: i.opt(id, "parentId")?,
                classes: i.list(id, "classes")?,
                styles: i.list(id, "styles")?,
            },
        ))
    }
    for (_, value) in i.entries(get(g, root, "useCases"))? {
        let id = reference(&value)?;
        let name = i.field(id, "id")?;
        m.use_cases.push((
            name.clone(),
            UseCase {
                id: name,
                label: i.field(id, "label")?,
                label_type: i.field(id, "labelType")?,
                shape: i.field(id, "shape")?,
                business: truth(get(g, id, "business"), g),
                stereotype: i.opt(id, "stereotype")?,
                parent_id: i.opt(id, "parentId")?,
                classes: i.list(id, "classes")?,
                styles: i.list(id, "styles")?,
            },
        ))
    }
    for (_, value) in i.entries(get(g, root, "systemBoundaries"))? {
        let id = reference(&value)?;
        let name = i.field(id, "id")?;
        m.system_boundaries.push((
            name.clone(),
            SystemBoundary {
                id: name,
                label: i.field(id, "label")?,
                label_type: i.field(id, "labelType")?,
                boundary_type: i.field(id, "type")?,
                members: vec![],
                classes: i.list(id, "classes")?,
                styles: i.list(id, "styles")?,
            },
        ))
    }
    for value in i.dense(get(g, root, "relationships"))? {
        let id = reference(&value)?;
        let arrow = match get(g, id, "arrowType") {
            Atom::Number(n) if n.is_finite() && *n >= 0. && *n < 7. && n.fract() == 0. => {
                *n as usize
            }
            _ => usize::MAX,
        };
        m.relationships.push(Relationship {
            id: i.field(id, "id")?,
            explicit_id: truth(get(g, id, "explicitId"), g),
            source: i.field(id, "source")?,
            target: i.field(id, "target")?,
            relationship_type: i.field(id, "type")?,
            arrow_type: arrow,
            label: i.opt(id, "label")?,
            label_type: i.opt(id, "labelType")?,
            minlen: i.count(get(g, id, "minlen"))?,
            classes: i.list(id, "classes")?,
            styles: i.list(id, "styles")?,
            animate: truth(get(g, id, "animate"), g),
            animation: i.opt(id, "animation")?,
        })
    }
    for (_, value) in i.entries(get(g, root, "notes"))? {
        let id = reference(&value)?;
        let name = i.field(id, "id")?;
        m.notes.push((
            name.clone(),
            UsecaseNote {
                id: name,
                target: i.field(id, "target")?,
                label: i.field(id, "label")?,
                label_type: i.field(id, "labelType")?,
            },
        ))
    }
    for (_, value) in i.entries(get(g, root, "jsonNodes"))? {
        let id = reference(&value)?;
        let name = i.field(id, "id")?;
        let value = i.json(get(g, id, "value"))?;
        let mut order = vec![];
        for (key, keys) in i.entries(get(g, id, "propertyOrder"))? {
            if matches!(key, Atom::String(_)) {
                order.push((i.text(&key)?, i.strings(&keys)?))
            }
        }
        i.validate_json_rows(&value, &order)?;
        m.json_nodes.push((
            name.clone(),
            UsecaseJsonNode {
                id: name,
                value,
                property_order: order,
                classes: i.list(id, "classes")?,
                styles: i.list(id, "styles")?,
            },
        ))
    }
    // Shared public style arrays stay shared in the typed class definitions.
    // Re-importing the same live array per definition would amplify a small
    // valid alias graph into millions of owned String slots.
    let mut class_styles = HashMap::new();
    for (key, value) in i.entries(get(g, root, "classDefs"))? {
        if !matches!(key, Atom::String(_)) {
            continue;
        }
        let id = reference(&value)?;
        let key = i.text(&key)?;
        let styles_id = reference(get(g, id, "styles"))?;
        let styles = if let Some(styles) = class_styles.get(&styles_id) {
            std::sync::Arc::clone(styles)
        } else {
            let styles = std::sync::Arc::new(i.list(id, "styles")?);
            class_styles.insert(styles_id, styles.clone());
            styles
        };
        m.class_defs
            .push((key.clone(), ClassDef { id: key, styles }))
    }
    m.direction = i.field(root, "direction")?;
    m.title = i.opt(root, "title")?;
    m.acc_title = i.field(root, "accTitle")?;
    m.acc_descr = i.field(root, "accDescr")?;
    Ok(m)
}
impl PublicDiagramModule for Usecase {
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::usecase())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        Some(DetachedPublicRender::usecase(
            super::UsecaseDetachedInput::new(state.projected.as_ref().err()),
        ))
    }
    type PublicState = UsecasePublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let projected = project(&graph);
        Ok(UsecasePublicState { graph, projected })
    }
    fn create_public(&self, m: &UsecaseModel) -> Result<Self::PublicState, PeleError> {
        let graph = new_graph(m)?;
        let projected = project(&graph);
        Ok(UsecasePublicState { graph, projected })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &Self::PublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut Self::PublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.projected = project(&g);
        s.graph = g;
        Ok(())
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.projected.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
