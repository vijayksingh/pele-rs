//! Native AgentflowDb mutable records/methods, Pele 458322a5 (MIT, kepano).
use super::model::{AgentDb, Loc};
use super::{AgentFlow, AgentNode, AgentflowModel, DiagnosticId, ElementPosition, LabelType};
use crate::compat::{
    public::*, source::space, DiagramModule, ErrorCode, Object, ParseContext, PeleError,
    RenderResources, RenderSettings, Rendered, Value,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};
#[path = "public_model/live_options.rs"]
mod live_options;
const MAX_ITEMS: usize = 1_000_000;
const MAX_WORK: usize = 8_000_000;
const MAX_TEXT: usize = 16_000_000;
const PROTOTYPE: PrototypeId = PrototypeId(1);
/// Source-only observables omitted from the immutable convenience projection.
#[derive(Clone, Debug, Default)]
pub struct AgentflowPublicOrigin {
    pub nodes: Vec<Vec<&'static str>>,
    pub raw_node_metadata: HashMap<usize, Vec<Value>>,
    pub raw_edge_metadata: HashMap<usize, Vec<Value>>,
    pub raw_group_metadata: HashMap<usize, Vec<Value>>,
    pub edges: Vec<Vec<&'static str>>,
    pub global_nodes: Vec<String>,
    pub members: Vec<String>,
    pub bare_mappings: Vec<usize>,
    pub diagnostic_mappings: Vec<Option<usize>>,
    pub sub_count: usize,
    pub max_edges: usize,
    pub inherit_dir: bool,
    pub direction: Option<String>,
    pub edge_styles: Vec<Option<usize>>,
    pub default_style: Option<usize>,
    pub next_style: usize,
}
impl AgentflowPublicOrigin {
    pub(super) fn metadata(
        raw: &mut HashMap<usize, Vec<Value>>,
        index: usize,
        before: &Option<Object>,
        doc: &Value,
    ) {
        let astral = matches!(doc,Value::String(s)if s.chars().any(|c|c as u32>0xffff));
        if astral || raw.contains_key(&index) {
            let operations = raw
                .entry(index)
                .or_insert_with(|| vec![Value::Object(before.clone().unwrap_or_default())]);
            operations.push(doc.clone());
        }
    }

    pub(super) fn key(keys: &mut Vec<&'static str>, key: &'static str) {
        if !keys.contains(&key) {
            keys.push(key)
        }
    }
}
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
const fn m(id: u32, name: &'static str, arity: u32) -> MethodSchema {
    MethodSchema {
        id: MethodId(id),
        name,
        arity,
    }
}
const fn p(id: u32, name: &'static str) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(id),
        },
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/agentflow@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "AgentflowModel",
        kind: RecordKind::Object,
        prototype: Some(PROTOTYPE),
        fields: &[
            f("type", FieldKind::String, false),
            f("direction", FieldKind::String, true),
            f("nodes", FieldKind::Map, false),
            f("edges", FieldKind::Array, false),
            f("subgraphs", FieldKind::Array, false),
            f("connectors", FieldKind::Map, false),
            f("classes", FieldKind::Map, false),
            f("tooltips", FieldKind::Map, false),
            f("defaultEdgeStyle", FieldKind::Array, true),
            f("defaultInterpolate", FieldKind::Any, true),
            f("mappings", FieldKind::Array, false),
            f("diagnostics", FieldKind::Array, false),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, true),
            f("accDescr", FieldKind::String, true),
            f("lineOffset", FieldKind::Number, false),
            f("subgraphLookup", FieldKind::Map, false),
            f("globalNodes", FieldKind::Set, false),
            f("members", FieldKind::Set, false),
            f("edgeById", FieldKind::Map, false),
            f("pairCount", FieldKind::Map, false),
            f("subCount", FieldKind::Number, false),
            f("bareMappings", FieldKind::Set, false),
            f("firstMapping", FieldKind::Map, false),
            f("reported", FieldKind::Map, false),
            f("options", FieldKind::Object, false),
        ],
        members: &[],
    }],
    prototypes: &[PrototypeSchema {
        id: PROTOTYPE,
        name: "AgentflowDb",
        parent: PrototypeParent::Object,
        members: &[
            p(1, "metadata"),
            p(2, "shapeError"),
            p(3, "addVertex"),
            p(4, "addSingleLink"),
            p(5, "addLink"),
            p(6, "updateLinkInterpolate"),
            p(7, "updateLink"),
            p(8, "edgeAt"),
            p(9, "addClass"),
            p(10, "setDirection"),
            p(11, "setClass"),
            p(12, "setTooltip"),
            p(13, "setLink"),
            p(14, "setClickEvent"),
            p(15, "setAccTitle"),
            p(16, "setAccDescription"),
            p(17, "addSubGraph"),
            p(18, "addGlobal"),
            p(19, "addConnector"),
            p(20, "destructLink"),
            p(21, "subgraph"),
            p(22, "vertexKind"),
            p(23, "semanticModel"),
            p(24, "position"),
            p(25, "pushMapping"),
            p(26, "addVertexMapping"),
            p(27, "extendVertexMapping"),
            p(28, "addEdgeMapping"),
            p(29, "blockMapping"),
            p(30, "addSubgraphMapping"),
            p(31, "addConnectorMapping"),
            p(32, "emit"),
            p(33, "emitWarning"),
            p(34, "emitError"),
        ],
    }],
    methods: &[
        m(1, "metadata", 3),
        m(2, "shapeError", 2),
        m(3, "addVertex", 6),
        m(4, "addSingleLink", 4),
        m(5, "addLink", 3),
        m(6, "updateLinkInterpolate", 2),
        m(7, "updateLink", 2),
        m(8, "edgeAt", 1),
        m(9, "addClass", 2),
        m(10, "setDirection", 1),
        m(11, "setClass", 2),
        m(12, "setTooltip", 2),
        m(13, "setLink", 3),
        m(14, "setClickEvent", 3),
        m(15, "setAccTitle", 1),
        m(16, "setAccDescription", 1),
        m(17, "addSubGraph", 4),
        m(18, "addGlobal", 1),
        m(19, "addConnector", 2),
        m(20, "destructLink", 2),
        m(21, "subgraph", 1),
        m(22, "vertexKind", 1),
        m(23, "semanticModel", 0),
        m(24, "position", 1),
        m(25, "pushMapping", 3),
        m(26, "addVertexMapping", 4),
        m(27, "extendVertexMapping", 2),
        m(28, "addEdgeMapping", 4),
        m(29, "blockMapping", 4),
        m(30, "addSubgraphMapping", 4),
        m(31, "addConnectorMapping", 4),
        m(32, "emit", 4),
        m(33, "emitWarning", 3),
        m(34, "emitError", 3),
    ],
};
pub struct AgentflowPublicState {
    graph: ModelGraph,
    base: Option<AgentflowModel>,
    constructed_cache: super::live_builder::Cache,
    style_bindings: Vec<Option<u32>>,
    projected: Result<AgentflowModel, PeleError>,
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("agentflow".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("AgentFlow public model expansion exceeds the work budget");
    e.code = ErrorCode::Limit;
    e
}
fn ty(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn get<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(&Atom::Undefined, |p| &p.value)
}
fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = a {
        Ok(*id)
    } else {
        Err(error("AgentFlow public model requires an object reference"))
    }
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| error("AgentFlow public text must be a well-formed string"))
}
fn dense(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PeleError> {
    let id = reference(a)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(error("AgentFlow public collection must be an Array"));
    };
    if *length as usize > MAX_ITEMS {
        return Err(limit());
    }
    let mut values = vec![None; *length as usize];
    for p in properties {
        let Some(k) = g.strings[p.key as usize].well_formed() else {
            continue;
        };
        if let Ok(i) = k.parse::<usize>() {
            if i < values.len() && i.to_string() == k {
                values[i] = Some(p.value.clone())
            }
        }
    }
    values
        .into_iter()
        .map(|v| v.ok_or_else(|| error("Sparse AgentFlow public collections are not implemented")))
        .collect()
}
fn entries(g: &ModelGraph, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
    let id = reference(a)?;
    if let Node::Map { entries, .. } = &g.nodes[id as usize] {
        if entries.len() > MAX_ITEMS {
            return Err(limit());
        }
        Ok(entries.clone())
    } else {
        Err(error("AgentFlow public collection must be a Map"))
    }
}
fn label(t: &LabelType) -> &'static str {
    match t {
        LabelType::Text => "text",
        LabelType::String => "string",
        LabelType::Markdown => "markdown",
    }
}
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Key {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    String(u32),
    Ref(u32),
    Builtin(u32),
    Host(u32),
}
struct CachedSet {
    values: Vec<Option<Atom>>,
    index: HashMap<Key, usize>,
}
struct Arena<'a> {
    g: &'a mut ModelGraph,
    units: usize,
    work: usize,
    string_ids: HashMap<JsString, u32>,
    canonical_strings: Vec<u32>,
    text_work: usize,
    maps: RefCell<HashMap<u32, HashMap<Key, usize>>>,
    sets: RefCell<HashMap<u32, CachedSet>>,
    properties: RefCell<HashMap<u32, HashMap<u32, usize>>>,
}
impl<'a> Arena<'a> {
    fn new(g: &'a mut ModelGraph) -> Self {
        let units = g.strings.iter().map(|s| s.0.len()).sum();
        let work = g
            .nodes
            .iter()
            .map(|n| {
                n.properties().len()
                    + match n {
                        Node::Map { entries, .. } => entries.len() * 2,
                        Node::Set { values, .. } => values.len(),
                        _ => 0,
                    }
            })
            .sum();
        let mut string_ids = HashMap::new();
        let mut canonical_strings = Vec::with_capacity(g.strings.len());
        for (i, s) in g.strings.iter().enumerate() {
            canonical_strings.push(*string_ids.entry(s.clone()).or_insert(i as u32));
        }
        Self {
            g,
            units,
            work,
            string_ids,
            canonical_strings,
            text_work: 0,
            maps: RefCell::default(),
            sets: RefCell::default(),
            properties: RefCell::default(),
        }
    }
    fn key(&self, a: &Atom) -> Key {
        match a {
            Atom::Undefined => Key::Undefined,
            Atom::Null => Key::Null,
            Atom::Bool(v) => Key::Bool(*v),
            Atom::Number(v) => Key::Number(if v.is_nan() {
                f64::NAN.to_bits()
            } else if *v == 0. {
                0
            } else {
                v.to_bits()
            }),
            Atom::String(i) => Key::String(self.canonical_strings[*i as usize]),
            Atom::Ref(i) => Key::Ref(*i),
            Atom::BuiltinFunction(i) => Key::Builtin(*i),
            Atom::HostReference(i) => Key::Host(*i),
        }
    }
    fn property_position(&self, id: u32, key: &str) -> Option<usize> {
        let canonical = *self.string_ids.get(&JsString::from(key))?;
        let mut cache = self.properties.borrow_mut();
        let index = cache.entry(id).or_insert_with(|| {
            self.g.nodes[id as usize]
                .properties()
                .iter()
                .enumerate()
                .map(|(i, p)| (self.canonical_strings[p.key as usize], i))
                .collect()
        });
        index.get(&canonical).copied()
    }
    fn map_position(&self, id: u32, key: &Atom) -> Result<Option<usize>, PublicError> {
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            return Err(ty("AgentFlow collection must be a Map"));
        };
        let key = self.key(key);
        let mut cache = self.maps.borrow_mut();
        let index = cache.entry(id).or_insert_with(|| {
            let mut index = HashMap::new();
            for (i, (k, _)) in entries.iter().enumerate() {
                index.entry(self.key(k)).or_insert(i);
            }
            index
        });
        Ok(index.get(&key).copied())
    }
    fn intern(&mut self, s: JsString) -> Result<u32, PublicError> {
        self.text_work = self.text_work.checked_add(s.0.len()).ok_or_else(limit)?;
        if self.text_work > MAX_TEXT {
            return Err(limit().into());
        }
        if let Some(id) = self.string_ids.get(&s) {
            return Ok(*id);
        }
        if self.g.strings.len() >= MAX_ITEMS || self.units.saturating_add(s.0.len()) > MAX_TEXT {
            return Err(limit().into());
        }
        self.units += s.0.len();
        let id = self.g.strings.len() as u32;
        self.string_ids.insert(s.clone(), id);
        self.canonical_strings.push(id);
        self.g.strings.push(s);
        Ok(id)
    }
    fn string(&mut self, s: impl Into<JsString>) -> Result<Atom, PublicError> {
        self.intern(s.into()).map(Atom::String)
    }
    fn node(&mut self, n: Node) -> Result<u32, PublicError> {
        if self.g.nodes.len() >= MAX_ITEMS {
            return Err(limit().into());
        }
        let id = self.g.nodes.len() as u32;
        self.g.nodes.push(n);
        Ok(id)
    }
    fn object(&mut self) -> Result<u32, PublicError> {
        self.node(Node::Object { properties: vec![] })
    }
    fn assign(&mut self, id: u32, key: &str, value: Atom) -> Result<(), PublicError> {
        if let Some(i) = self.property_position(id, key) {
            let p = &mut self.g.nodes[id as usize].properties_mut()[i];
            if p.flags & 2 == 0 {
                return Err(ty("Cannot assign to a read only AgentFlow property"));
            }
            p.value = value;
            return Ok(());
        }
        if self.work >= MAX_WORK {
            return Err(limit().into());
        }
        self.work += 1;
        let key = self.intern(key.into())?;
        let index = self.g.nodes[id as usize].properties().len();
        self.g.nodes[id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        });
        if let Some(properties) = self.properties.get_mut().get_mut(&id) {
            properties.insert(key, index);
        }

        Ok(())
    }
    fn field(&self, id: u32, key: &str) -> Atom {
        self.property_position(id, key)
            .map_or(Atom::Undefined, |i| {
                self.g.nodes[id as usize].properties()[i].value.clone()
            })
    }
    fn id(&self, a: &Atom) -> Result<u32, PublicError> {
        reference(a).map_err(|_| ty("AgentFlow value must be an object"))
    }
    fn field_id(&self, id: u32, key: &str) -> Result<u32, PublicError> {
        self.id(get(self.g, id, key))
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<u32, PublicError> {
        if values.len() > MAX_ITEMS {
            return Err(limit().into());
        }
        let id = self.node(Node::Array {
            length: values.len() as u32,
            properties: vec![],
        })?;
        for (i, v) in values.into_iter().enumerate() {
            self.assign(id, &i.to_string(), v)?
        }
        Ok(id)
    }
    fn strings(&mut self, values: &[String]) -> Result<u32, PublicError> {
        let values = values
            .iter()
            .map(|s| self.string(s.as_str()))
            .collect::<Result<Vec<_>, _>>()?;
        self.array(values)
    }
    fn record(&mut self, values: Vec<(&str, Atom)>) -> Result<u32, PublicError> {
        let id = self.object()?;
        for (k, v) in values {
            self.assign(id, k, v)?
        }
        Ok(id)
    }
    fn push(&mut self, id: u32, value: Atom) -> Result<(), PublicError> {
        if self.g.nodes[id as usize]
            .properties()
            .iter()
            .any(|p| self.g.strings[p.key as usize].is("push"))
        {
            return Err(ty(
                "Overridden AgentFlow collection push is not implemented",
            ));
        }
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            return Err(ty("AgentFlow collection push is not a function"));
        };
        if self.g.nodes[id as usize]
            .properties()
            .iter()
            .any(|p| self.g.strings[p.key as usize].is("length") && p.flags & 2 == 0)
        {
            return Err(ty(
                "Cannot assign to read only property 'length' of object '[object Array]'",
            ));
        }
        if length as usize >= MAX_ITEMS {
            return Err(limit().into());
        }
        self.assign(id, &length.to_string(), value)?;
        let Node::Array { length: len, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        *len = length + 1;
        for property in self.g.nodes[id as usize].properties_mut() {
            if self.g.strings[property.key as usize].is("length") {
                property.value = Atom::Number((length + 1) as f64);
            }
        }
        Ok(())
    }
    fn map(&mut self) -> Result<u32, PublicError> {
        self.node(Node::Map {
            entries: vec![],
            properties: vec![],
        })
    }
    fn map_get(&self, id: u32, key: &Atom) -> Result<Atom, PublicError> {
        let i = self.map_position(id, key)?;
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            unreachable!()
        };
        Ok(i.map_or(Atom::Undefined, |i| entries[i].1.clone()))
    }
    fn map_has(&self, id: u32, key: &Atom) -> Result<bool, PublicError> {
        Ok(self.map_position(id, key)?.is_some())
    }
    fn map_set(&mut self, id: u32, key: Atom, value: Atom) -> Result<(), PublicError> {
        let position = self.map_position(id, &key)?;
        let cache_key = self.key(&key);
        let Node::Map { entries, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        if let Some(i) = position {
            entries[i].1 = value
        } else {
            if entries.len() >= MAX_ITEMS || self.work + 2 > MAX_WORK {
                return Err(limit().into());
            }
            self.work += 2;
            let i = entries.len();
            entries.push((key, value));
            self.maps
                .get_mut()
                .get_mut(&id)
                .unwrap()
                .insert(cache_key, i);
        }
        Ok(())
    }
    fn value(&mut self, v: &Value) -> Result<Atom, PublicError> {
        Ok(match v {
            Value::Null => Atom::Null,
            Value::Bool(v) => Atom::Bool(*v),
            Value::Number(v) => Atom::Number(*v),
            Value::String(s) => self.string(s.as_str())?,
            Value::Array(values) => {
                let values = values
                    .iter()
                    .map(|v| self.value(v))
                    .collect::<Result<Vec<_>, _>>()?;
                Atom::Ref(self.array(values)?)
            }
            Value::Object(o) => {
                let id = self.object()?;
                for (k, v) in o.entries() {
                    let v = self.value(v)?;
                    self.assign(id, k, v)?
                }
                Atom::Ref(id)
            }
        })
    }
    fn opt(&mut self, s: Option<&String>) -> Result<Atom, PublicError> {
        s.map_or(Ok(Atom::Undefined), |s| self.string(s.as_str()))
    }
    fn opt_value(&mut self, v: Option<&Value>) -> Result<Atom, PublicError> {
        v.map_or(Ok(Atom::Undefined), |v| self.value(v))
    }
    fn ordered(
        &mut self,
        fields: Vec<(&'static str, Atom)>,
        order: &[&'static str],
    ) -> Result<u32, PublicError> {
        let id = self.object()?;
        for key in order {
            if let Some((_, v)) = fields.iter().find(|(k, _)| k == key) {
                self.assign(id, key, v.clone())?
            }
        }
        for (k, v) in fields {
            if !order.contains(&k) && !matches!(v, Atom::Undefined) {
                self.assign(id, k, v)?
            }
        }
        Ok(id)
    }
}
fn strict(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    if matches!(a,Atom::Number(v)if v.is_nan()) || matches!(b,Atom::Number(v)if v.is_nan()) {
        false
    } else {
        same(g, a, b)
    }
}
fn same(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Number(a), Atom::Number(b)) => a == b || a.is_nan() && b.is_nan(),
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize] == g.strings[*b as usize],
        (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
fn truthy(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(v) => *v,
        Atom::Number(v) => *v != 0. && !v.is_nan(),
        Atom::String(i) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
fn number(g: &ModelGraph, a: &Atom) -> Result<f64, PublicError> {
    Ok(match a {
        Atom::Undefined => f64::NAN,
        Atom::Null => 0.,
        Atom::Bool(v) => u8::from(*v) as f64,
        Atom::Number(v) => *v,
        Atom::String(_) => {
            let s = text(g, a)?;
            let s = s.trim_matches(space);
            if s.is_empty() {
                0.
            } else if matches!(s, "Infinity" | "+Infinity") {
                f64::INFINITY
            } else if s == "-Infinity" {
                f64::NEG_INFINITY
            } else if let Some((radix, rest)) = [
                ("0x", 16),
                ("0X", 16),
                ("0o", 8),
                ("0O", 8),
                ("0b", 2),
                ("0B", 2),
            ]
            .into_iter()
            .find_map(|(p, r)| s.strip_prefix(p).map(|s| (r, s)))
            {
                u64::from_str_radix(rest, radix).map_or(f64::NAN, |v| v as f64)
            } else {
                s.parse().unwrap_or(f64::NAN)
            }
        }
        _ => return Err(error("AgentFlow object numeric coercion is not implemented").into()),
    })
}
fn js_string(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    Ok(match a {
        Atom::Undefined => "undefined".into(),
        Atom::Null => "null".into(),
        Atom::Bool(v) => v.to_string(),
        Atom::Number(n) => Value::Number(*n).js_string(),
        Atom::String(_) => text(g, a)?,
        _ => return Err(error("AgentFlow object string coercion is not implemented").into()),
    })
}
fn raw_trim(g: &ModelGraph, a: &Atom, left_only: bool) -> Result<JsString, PublicError> {
    let Atom::String(i) = a else {
        return Err(ty("AgentFlow argument must be a string"));
    };
    let units = &g.strings[*i as usize].0;
    let blank = |u: &u16| char::from_u32(*u as u32).is_some_and(space);
    let start = units.iter().take_while(|u| blank(u)).count();
    let end = if left_only {
        units.len()
    } else {
        units.len() - units[start..].iter().rev().take_while(|u| blank(u)).count()
    };
    Ok(JsString(units[start..end].to_vec()))
}
fn unquote_units(mut text: JsString) -> JsString {
    if text.0.first() == Some(&34) && text.0.last() == Some(&34) && text.0.len() >= 2 {
        text.0.remove(0);
        text.0.pop();
    }
    text
}
fn arg(call: &MethodCall, n: usize) -> Atom {
    call.arguments.get(n).cloned().unwrap_or(Atom::Undefined)
}
fn diagnostic_id(id: DiagnosticId) -> &'static str {
    match id {
        DiagnosticId::ShapeUnsupported => "SHAPE_UNSUPPORTED",
        DiagnosticId::ShapeRemoved => "SHAPE_REMOVED",
        DiagnosticId::EdgeOperatorUnsupported => "EDGE_OPERATOR_UNSUPPORTED",
        DiagnosticId::ReferenceEdgeLabelRejected => "REFERENCE_EDGE_LABEL_REJECTED",
        DiagnosticId::ConnectorRefUnresolved => "CONNECTOR_REF_UNRESOLVED",
        DiagnosticId::ConnectorRefNotAConnector => "CONNECTOR_REF_NOT_A_CONNECTOR",
        DiagnosticId::MetadataKeyMisapplied => "METADATA_KEY_MISAPPLIED",
        DiagnosticId::DuplicateIdNode => "DUPLICATE_ID_NODE",
        DiagnosticId::ReservedSyntheticId => "RESERVED_SYNTHETIC_ID",
        DiagnosticId::ContainmentViolation => "CONTAINMENT_VIOLATION",
        DiagnosticId::EdgeSemanticContradiction => "EDGE_SEMANTIC_CONTRADICTION",
        DiagnosticId::FlowNoInput => "FLOW_NO_INPUT",
    }
}
impl Arena<'_> {
    fn source_metadata(
        &mut self,
        metadata: &Option<Object>,
        operations: Option<&Vec<Value>>,
    ) -> Result<Atom, PublicError> {
        if let Some(operations) = operations {
            let holder = self.object()?;
            for value in operations {
                let doc = self.value(value)?;
                self.merge_metadata(holder, &doc)?
            }
            Ok(self.field(holder, "metadata"))
        } else {
            metadata.as_ref().map_or(Ok(Atom::Undefined), |o| {
                self.value(&Value::Object(o.clone()))
            })
        }
    }
    fn position_value(&mut self, p: &ElementPosition) -> Result<u32, PublicError> {
        self.record(vec![
            ("startLine", Atom::Number(p.start_line as f64)),
            ("startColumn", Atom::Number(p.start_column as f64)),
            ("endLine", Atom::Number(p.end_line as f64)),
            ("endColumn", Atom::Number(p.end_column as f64)),
            ("startIndex", Atom::Number(p.start_index as f64)),
            ("endIndex", Atom::Number(p.end_index as f64)),
        ])
    }
    fn set_value(&mut self, values: Vec<Atom>) -> Result<u32, PublicError> {
        if values.len() > MAX_ITEMS {
            return Err(limit().into());
        }
        self.node(Node::Set {
            values,
            properties: vec![],
        })
    }
}
fn new_graph(
    m: &AgentflowModel,
    limit: Option<crate::compat::EdgeLimit>,
) -> Result<(ModelGraph, Vec<Option<u32>>), PublicError> {
    new_graph_with_options(m, limit, None)
}
fn new_graph_with_options(
    m: &AgentflowModel,
    limit: Option<crate::compat::EdgeLimit>,
    argument: Option<ConstructorArgument>,
) -> Result<(ModelGraph, Vec<Option<u32>>), PublicError> {
    let constructing = argument.is_some();
    let mut graph = ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec![],
        nodes: vec![],
    };
    let mut a = Arena::new(&mut graph);
    let root = a.node(Node::Instance {
        prototype: PROTOTYPE,
        properties: vec![],
    })?;
    a.g.root = Atom::Ref(root);
    let nodes = a.map()?;
    let connectors = a.map()?;
    let classes = a.map()?;
    let tooltips = a.map()?;
    let subgraphs = a.map()?;
    let edge_lookup = a.map()?;
    let pair_count = a.map()?;
    let first_mapping = a.map()?;
    let reported = a.map()?;
    let mut styles = HashMap::new();
    let mut node_ids = HashMap::new();
    for (i, (key, n)) in m.nodes.iter().enumerate() {
        let fields = vec![
            ("id", a.string(n.id.as_str())?),
            ("text", a.opt(n.text.as_ref())?),
            ("labelType", a.string(label(&n.label_type))?),
            ("type", a.opt(n.shape.as_ref())?),
            ("styles", Atom::Ref(a.strings(&n.styles)?)),
            ("classes", Atom::Ref(a.strings(&n.classes)?)),
            ("dir", a.opt(n.dir.as_ref())?),
            (
                "props",
                n.props
                    .as_ref()
                    .map_or(Ok(Atom::Undefined), |o| a.value(&Value::Object(o.clone())))?,
            ),
            ("link", a.opt(n.link.as_ref())?),
            ("linkTarget", a.opt(n.link_target.as_ref())?),
            (
                "metadata",
                a.source_metadata(&n.metadata, m.public_origin.raw_node_metadata.get(&i))?,
            ),
            (
                "isConnector",
                n.is_connector.map_or(Atom::Undefined, Atom::Bool),
            ),
        ];
        let order = m.public_origin.nodes.get(i).map_or(
            &[
                "id",
                "labelType",
                "styles",
                "classes",
                "text",
                "type",
                "props",
            ][..],
            Vec::as_slice,
        );
        let id = a.ordered(fields, order)?;
        node_ids.insert(key.clone(), id);
        let key = a.string(key.as_str())?;
        a.map_set(nodes, key, Atom::Ref(id))?;
    }
    for (key, _) in &m.connectors {
        let id = node_ids[key];
        let key = a.string(key.as_str())?;
        a.map_set(connectors, key, Atom::Ref(id))?;
    }
    let mut edge_ids = Vec::new();
    for (i, e) in m.edges.iter().enumerate() {
        let mut fields = vec![
            ("start", a.string(e.start.as_str())?),
            ("end", a.string(e.end.as_str())?),
            ("type", a.opt(e.edge_type.as_ref())?),
            ("text", a.string(e.text.as_str())?),
            ("labelType", a.string(label(&e.label_type))?),
            ("classes", Atom::Ref(a.strings(&e.classes)?)),
            ("isUserDefinedId", Atom::Bool(e.is_user_defined_id)),
            ("interpolate", a.opt_value(e.interpolate.as_ref())?),
            ("stroke", a.opt(e.stroke.as_ref())?),
            ("length", a.opt_value(e.length.as_ref())?),
            ("edgeSemantic", a.opt(e.edge_semantic.as_ref())?),
            ("id", a.opt(e.id.as_ref())?),
            ("animate", a.opt_value(e.animate.as_ref())?),
            ("animation", a.opt_value(e.animation.as_ref())?),
            (
                "metadata",
                a.source_metadata(&e.metadata, m.public_origin.raw_edge_metadata.get(&i))?,
            ),
        ];
        if let Some(values) = &e.style {
            let group = m.public_origin.edge_styles.get(i).copied().flatten();
            let id = if let Some(id) = group.and_then(|g| styles.get(&g)).copied() {
                id
            } else {
                let id = a.strings(values)?;
                if let Some(g) = group {
                    styles.insert(g, id);
                }
                id
            };
            fields.push(("style", Atom::Ref(id)));
        }
        let order = m.public_origin.edges.get(i).map_or(
            &[
                "start",
                "end",
                "type",
                "text",
                "labelType",
                "classes",
                "isUserDefinedId",
                "interpolate",
                "stroke",
                "length",
                "edgeSemantic",
                "id",
            ][..],
            Vec::as_slice,
        );
        let id = a.ordered(fields, order)?;
        edge_ids.push(Atom::Ref(id));
        if let Some(key) = &e.id {
            let key = a.string(key.as_str())?;
            if !a.map_has(edge_lookup, &key)? {
                a.map_set(edge_lookup, key, Atom::Ref(id))?
            }
        }
        let key = a.string(format!("{}\0{}", e.start, e.end).as_str())?;
        let count = if let Atom::Number(n) = a.map_get(pair_count, &key)? {
            n
        } else {
            0.
        };
        a.map_set(pair_count, key, Atom::Number(count + 1.))?;
    }
    let mut sg_ids = Vec::new();
    for (i, s) in m.subgraphs.iter().enumerate() {
        let fields = vec![
            ("id", a.string(s.id.as_str())?),
            ("nodes", Atom::Ref(a.strings(&s.nodes)?)),
            ("title", a.string(s.title.as_str())?),
            ("classes", Atom::Ref(a.strings(&s.classes)?)),
            ("dir", a.opt(s.dir.as_ref())?),
            ("labelType", a.string(label(&s.label_type))?),
            ("type", a.string(s.group_type.as_str())?),
            (
                "metadata",
                a.source_metadata(&s.metadata, m.public_origin.raw_group_metadata.get(&i))?,
            ),
        ];
        let id = a.ordered(
            fields,
            &[
                "id",
                "nodes",
                "title",
                "classes",
                "dir",
                "labelType",
                "type",
            ],
        )?;
        sg_ids.push(Atom::Ref(id));
        let key = a.string(s.id.as_str())?;
        a.map_set(subgraphs, key, Atom::Ref(id))?;
    }
    for (key, c) in &m.classes {
        let fields = vec![
            ("id", a.string(c.id.as_str())?),
            ("styles", Atom::Ref(a.strings(&c.styles)?)),
            ("textStyles", Atom::Ref(a.strings(&c.text_styles)?)),
        ];
        let id = a.record(fields)?;
        let key = a.string(key.as_str())?;
        a.map_set(classes, key, Atom::Ref(id))?;
    }
    for (key, value) in &m.tooltips {
        let key = a.string(key.as_str())?;
        let value = a.string(value.as_str())?;
        a.map_set(tooltips, key, value)?;
    }
    let mut mapping_ids = Vec::new();
    let mut position_ids = Vec::new();
    for mapping in &m.mappings {
        let position = a.position_value(&mapping.position)?;
        position_ids.push(position);
        let fields = vec![
            ("id", a.string(mapping.id.as_str())?),
            ("type", a.string(mapping.statement_type.as_str())?),
            ("position", Atom::Ref(position)),
        ];
        let id = a.record(fields)?;
        mapping_ids.push(Atom::Ref(id));
        let key = a.string(mapping.id.as_str())?;
        if !a.map_has(first_mapping, &key)? {
            a.map_set(first_mapping, key, Atom::Ref(id))?;
        }
    }
    let mut diagnostic_ids = Vec::new();
    for (i, d) in m.diagnostics.iter().enumerate() {
        let mut fields = vec![
            ("id", a.string(diagnostic_id(d.id))?),
            ("severity", a.string(d.severity.as_str())?),
            ("message", a.string(d.message.as_str())?),
        ];
        if let Some(id) = &d.node_id {
            fields.push(("nodeId", a.string(id.as_str())?));
        } else if let Some(id) = &d.edge_id {
            fields.push(("edgeId", a.string(id.as_str())?));
        }
        if let Some(position) = &d.position {
            let id = if let Some(index) = m
                .public_origin
                .diagnostic_mappings
                .get(i)
                .copied()
                .flatten()
            {
                position_ids[index]
            } else {
                a.position_value(position)?
            };
            fields.push(("position", Atom::Ref(id)));
        }
        let id = a.record(fields)?;
        diagnostic_ids.push(Atom::Ref(id));
        let anchor = a.string(d.node_id.as_deref().or(d.edge_id.as_deref()).unwrap_or(""))?;
        let list = match a.map_get(reported, &anchor)? {
            Atom::Ref(id) => id,
            _ => {
                let list = a.array(vec![])?;
                a.map_set(reported, anchor, Atom::Ref(list))?;
                list
            }
        };
        a.push(list, Atom::Ref(id))?;
    }
    let mappings = a.array(mapping_ids.clone())?;
    let diagnostics = a.array(diagnostic_ids)?;
    let bare = m
        .public_origin
        .bare_mappings
        .iter()
        .map(|i| mapping_ids[*i].clone())
        .collect();
    let bare = a.set_value(bare)?;
    let globals = m
        .public_origin
        .global_nodes
        .iter()
        .map(|s| a.string(s.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    let globals = a.set_value(globals)?;
    let members = m
        .public_origin
        .members
        .iter()
        .map(|s| a.string(s.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    let members = a.set_value(members)?;
    let edges = a.array(edge_ids)?;
    let groups = a.array(sg_ids)?;
    let default_style = if let Some(values) = &m.default_edge_style {
        let id = if let Some(id) = m
            .public_origin
            .default_style
            .and_then(|g| styles.get(&g))
            .copied()
        {
            id
        } else {
            a.strings(values)?
        };
        Atom::Ref(id)
    } else {
        Atom::Undefined
    };
    let options = match argument {
        Some(ConstructorArgument::Undefined) => Atom::Ref(a.object()?),
        Some(argument) => argument.into_atom(|value| a.string(value))?,
        None => {
            let options = a.object()?;
            a.assign(
                options,
                "inheritDir",
                Atom::Bool(m.public_origin.inherit_dir),
            )?;
            let dir = a.opt(m.public_origin.direction.as_ref())?;
            a.assign(options, "direction", dir)?;
            a.assign(options, "lineOffset", Atom::Number(m.line_offset as f64))?;
            let max_edges = match limit {
                Some(limit) => a.value(&limit.value())?,
                None => Atom::Number(m.public_origin.max_edges as f64),
            };
            a.assign(options, "maxEdges", max_edges)?;
            Atom::Ref(options)
        }
    };
    let fields = vec![
        ("type", a.string("agentflow")?),
        ("direction", a.opt(m.direction.as_ref())?),
        ("nodes", Atom::Ref(nodes)),
        ("edges", Atom::Ref(edges)),
        ("subgraphs", Atom::Ref(groups)),
        ("connectors", Atom::Ref(connectors)),
        ("classes", Atom::Ref(classes)),
        ("tooltips", Atom::Ref(tooltips)),
        ("defaultEdgeStyle", default_style),
        (
            "defaultInterpolate",
            a.opt_value(m.default_interpolate.as_ref())?,
        ),
        ("mappings", Atom::Ref(mappings)),
        ("diagnostics", Atom::Ref(diagnostics)),
        ("title", a.opt(m.title.as_ref())?),
        ("accTitle", a.opt(m.acc_title.as_ref())?),
        ("accDescr", a.opt(m.acc_descr.as_ref())?),
        ("lineOffset", Atom::Number(m.line_offset as f64)),
        ("subgraphLookup", Atom::Ref(subgraphs)),
        ("globalNodes", Atom::Ref(globals)),
        ("members", Atom::Ref(members)),
        ("edgeById", Atom::Ref(edge_lookup)),
        ("pairCount", Atom::Ref(pair_count)),
        ("subCount", Atom::Number(m.public_origin.sub_count as f64)),
        ("bareMappings", Atom::Ref(bare)),
        ("firstMapping", Atom::Ref(first_mapping)),
        ("reported", Atom::Ref(reported)),
        ("options", options),
    ];
    for (key, value) in fields {
        a.assign(root, key, value)?
    }
    let bindings = if constructing {
        vec![]
    } else {
        m.graph
            .as_ref()
            .ok_or_else(|| error("AgentFlow parsed graph is unavailable"))?
            .nodes
            .iter()
            .map(|n| {
                node_ids
                    .get(&n.id)
                    .map(|id| reference(get(a.g, *id, "styles")))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    Ok((graph, bindings))
}
fn project_render(
    g: &ModelGraph,
    base: &AgentflowModel,
    bindings: &[Option<u32>],
) -> Result<AgentflowModel, PeleError> {
    let root = reference(&g.root)?;
    let mut model = base.clone();
    let mut units = 0usize;
    let mut string = |a: &Atom| -> Result<Option<String>, PeleError> {
        if matches!(a, Atom::Undefined) {
            return Ok(None);
        }
        if let Atom::String(i) = a {
            units = units
                .checked_add(g.strings[*i as usize].0.len())
                .ok_or_else(limit)?;
            if units > MAX_TEXT {
                return Err(limit());
            }
        }
        text(g, a).map(Some)
    };
    model.direction = string(get(g, root, "direction"))?;
    model.title = string(get(g, root, "title"))?;
    model.acc_title = string(get(g, root, "accTitle"))?;
    model.acc_descr = string(get(g, root, "accDescr"))?;
    let graph = model
        .graph
        .as_mut()
        .ok_or_else(|| error("AgentFlow parsed graph is unavailable"))?;
    let mut work = 0usize;
    for (node, binding) in graph.nodes.iter_mut().zip(bindings) {
        if let Some(id) = binding {
            let values = dense(g, &Atom::Ref(*id))?;
            work = work.checked_add(values.len()).ok_or_else(limit)?;
            if work > MAX_WORK {
                return Err(limit());
            }
            node.css_styles = values
                .iter()
                .map(|a| {
                    string(a).and_then(|s| {
                        s.ok_or_else(|| error("AgentFlow cached style must be a string"))
                    })
                })
                .collect::<Result<_, _>>()?;
        }
    }
    Ok(model)
}
impl Arena<'_> {
    fn guard(&self, id: u32, name: &str, builtin: u32) -> Result<(), PublicError> {
        if let Some(p) = self.g.nodes[id as usize]
            .properties()
            .iter()
            .find(|p| self.g.strings[p.key as usize].is(name))
        {
            if matches!(p.value,Atom::BuiltinFunction(v)if v==builtin) {
                return Ok(());
            }
            return Err(
                if matches!(p.value, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                    error("AgentFlow overridden method dispatch is not implemented").into()
                } else {
                    ty("AgentFlow method is not a function")
                },
            );
        }
        Ok(())
    }
    fn map_field(&self, root: u32, name: &str, key: &Atom) -> Result<Atom, PublicError> {
        self.map_get(self.field_id(root, name)?, key)
    }
    fn set_class(&mut self, root: u32, ids: &str, class: Atom) -> Result<(), PublicError> {
        for id in ids.split(',') {
            let key = self.string(id)?;
            for name in ["nodes", "edgeById", "subgraphLookup"] {
                let node = self.map_field(root, name, &key)?;
                if !matches!(node, Atom::Undefined | Atom::Null) {
                    let node = self.id(&node)?;
                    let classes = self.field_id(node, "classes")?;
                    self.push(classes, class.clone())?
                }
            }
        }
        Ok(())
    }
    fn edge_at(&self, root: u32, pos: &Atom) -> Result<Atom, PublicError> {
        let id = self.field_id(root, "edges")?;
        let n = number(self.g, pos)?;
        let value = self.field(id, &Value::Number(n).js_string());
        if matches!(value, Atom::Undefined) {
            let length = match self.g.nodes[id as usize] {
                Node::Array { length, .. } => length as f64,
                _ => number(self.g, &self.field(id, "length"))?,
            };
            return Err(error(format!("The index {} for linkStyle is out of bounds. Valid indices for linkStyle are between 0 and {}. (Help: Ensure that the index is within the range of existing edges.)",js_string(self.g,pos)?,Value::Number(length-1.).js_string())).into());
        }
        Ok(value)
    }
    fn single_link(
        &mut self,
        root: u32,
        start: Atom,
        end: Atom,
        info: Atom,
        id: Atom,
    ) -> Result<(), PublicError> {
        let pending = self.single_link_prepare(root, start, end, info, id)?;
        let root = pending.root;
        let edges = self.field_id(root, "edges")?;
        let Node::Array { length, .. } = self.g.nodes[edges as usize] else {
            return Err(ty("AgentFlow edges must be an Array"));
        };
        let options = self.field_id(root, "options")?;
        let max = self.field(options, "maxEdges");
        if !matches!(max, Atom::Undefined) && length as f64 >= number(self.g, &max)? {
            let mut e = error(format!(
                "Edge limit exceeded. {length} edges found, but the limit is {}.",
                js_string(self.g, &max)?
            ));
            e.code = ErrorCode::Limit;
            return Err(e.into());
        }
        self.single_link_commit(pending)
    }
    fn single_link_prepare(
        &mut self,
        root: u32,
        start: Atom,
        end: Atom,
        info: Atom,
        id: Atom,
    ) -> Result<live_options::LinkPending, PublicError> {
        let classes = self.array(vec![])?;
        let interpolate = self.field(root, "defaultInterpolate");
        let empty = self.string("")?;
        let text_kind = self.string("text")?;
        let edge = self.record(vec![
            ("start", start.clone()),
            ("end", end.clone()),
            ("type", Atom::Undefined),
            ("text", empty),
            ("labelType", text_kind),
            ("classes", Atom::Ref(classes)),
            ("isUserDefinedId", Atom::Bool(false)),
            ("interpolate", interpolate),
        ])?;
        if !matches!(info, Atom::Undefined | Atom::Null) {
            let data = self.id(&info)?;
            let label = self.field(data, "text");
            if !matches!(label, Atom::Undefined) {
                let label = self.id(&label)?;
                let t = unquote_units(raw_trim(self.g, &self.field(label, "text"), false)?);
                let t = self.string(t)?;
                self.assign(edge, "text", t)?;
                let kind = self.field(label, "type");
                let kind = if ["text", "string", "markdown"]
                    .iter()
                    .any(|s| matches!(&kind,Atom::String(i)if self.g.strings[*i as usize].is(s)))
                {
                    kind
                } else {
                    self.string("markdown")?
                };
                self.assign(edge, "labelType", kind)?;
            }
            for key in ["type", "stroke"] {
                self.assign(edge, key, self.field(data, key))?
            }
            let length = self.field(data, "length");
            let length = if number(self.g, &length)? > 10. {
                Atom::Number(10.)
            } else {
                length
            };
            self.assign(edge, "length", length)?;
            let semantic = self.field(data, "edgeSemantic");
            if truthy(self.g, &semantic) {
                self.assign(edge, "edgeSemantic", semantic)?;
            }
        } else if matches!(info, Atom::Null) {
            return Err(ty("Cannot read AgentFlow link properties of null"));
        }
        let lookup = self.field_id(root, "edgeById")?;
        let custom = truthy(self.g, &id) && !self.map_has(lookup, &id)?;
        let pair = self.string(
            format!(
                "{}\0{}",
                js_string(self.g, &start)?,
                js_string(self.g, &end)?
            )
            .as_str(),
        )?;
        let old = self.map_field(root, "pairCount", &pair)?;
        let count = if matches!(old, Atom::Null | Atom::Undefined) {
            0.
        } else {
            number(self.g, &old)?
        };
        let id = if custom {
            id
        } else {
            self.string(
                format!(
                    "L_{}_{}_{}",
                    js_string(self.g, &start)?,
                    js_string(self.g, &end)?,
                    Value::Number(if count == 0. { 0. } else { count + 1. }).js_string()
                )
                .as_str(),
            )?
        };
        self.assign(edge, "id", id.clone())?;
        self.assign(edge, "isUserDefinedId", Atom::Bool(custom))?;
        Ok(live_options::LinkPending {
            root,
            edge,
            pair,
            count,
            id,
        })
    }
    fn single_link_commit(
        &mut self,
        pending: live_options::LinkPending,
    ) -> Result<(), PublicError> {
        let live_options::LinkPending {
            root,
            edge,
            pair,
            count,
            id,
        } = pending;
        let edges = self.field_id(root, "edges")?;
        self.push(edges, Atom::Ref(edge))?;
        let counts = self.field_id(root, "pairCount")?;
        self.map_set(counts, pair, Atom::Number(count + 1.))?;
        let lookup = self.field_id(root, "edgeById")?;
        if !self.map_has(lookup, &id)? {
            self.map_set(lookup, id, Atom::Ref(edge))?
        }
        Ok(())
    }
}
/// Direct conversion of freshly parsed YAML values for the source's String /
/// JSON.stringify metadata diagnostics. It is not a document or model codec.
fn yaml_value(
    g: &ModelGraph,
    atom: &Atom,
    work: &mut usize,
    units: &mut usize,
    depth: usize,
) -> Result<Value, PublicError> {
    *work = work.checked_add(1).ok_or_else(limit)?;
    if *work > MAX_WORK || depth > 128 {
        return Err(limit().into());
    }
    Ok(match atom {
        Atom::Null => Value::Null,
        Atom::Bool(v) => Value::Bool(*v),
        Atom::Number(v) => Value::Number(*v),
        Atom::String(i) => {
            *units = units
                .checked_add(g.strings[*i as usize].0.len())
                .ok_or_else(limit)?;
            if *units > MAX_TEXT {
                return Err(limit().into());
            }
            Value::String(text(g, atom)?)
        }
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { .. } => Value::Array(
                dense(g, atom)?
                    .iter()
                    .map(|v| yaml_value(g, v, work, units, depth + 1))
                    .collect::<Result<_, _>>()?,
            ),
            Node::Object { properties } | Node::Instance { properties, .. } => {
                let mut out = Object::default();
                for p in properties {
                    if p.flags & 1 != 0 && !matches!(p.value, Atom::Undefined) {
                        let key = text(g, &Atom::String(p.key))?;
                        out.insert(key, yaml_value(g, &p.value, work, units, depth + 1)?);
                    }
                }
                Value::Object(out)
            }
            _ => return Err(error("AgentFlow YAML value requires plain records").into()),
        },
        _ => return Err(error("AgentFlow YAML value is unavailable").into()),
    })
}
fn yaml_string(g: &ModelGraph, atom: &Atom) -> Result<String, PublicError> {
    yaml_value(g, atom, &mut 0, &mut 0, 0).map(|v| v.js_string())
}
fn yaml_json(g: &ModelGraph, atom: &Atom) -> Result<String, PublicError> {
    let value = yaml_value(g, atom, &mut 0, &mut 0, 0)?;
    serde_json::to_string(&value)
        .map_err(|_| error("AgentFlow metadata diagnostic serialization failed").into())
}
fn native_loc(g: &ModelGraph, a: &Atom) -> Result<Option<Loc>, PublicError> {
    if matches!(a, Atom::Undefined | Atom::Null) {
        return Ok(None);
    }
    let id = reference(a)?;
    let coord = |key| -> Result<usize, PublicError> {
        let n = number(g, get(g, id, key))?;
        if !n.is_finite() || n < 0. || n.fract() != 0. || n > usize::MAX as f64 {
            return Err(error(
                "AgentFlow metadata coordinates require finite nonnegative integers",
            )
            .into());
        }
        Ok(n as usize)
    };
    Ok(Some(Loc {
        first_line: coord("first_line")?,
        first_column: coord("first_column")?,
        last_line: coord("last_line")?,
        last_column: coord("last_column")?,
    }))
}
impl Arena<'_> {
    fn metadata(
        &mut self,
        root: u32,
        id: &Atom,
        source: &Atom,
        loc: &Atom,
    ) -> Result<Atom, PublicError> {
        let id = text(self.g, id)?;
        let source = text(self.g, source)?;
        let loc = native_loc(self.g, loc)?;
        let offset = number(self.g, &self.field(root, "lineOffset"))?;
        if !offset.is_finite() || offset < 0. || offset.fract() != 0. || offset > usize::MAX as f64
        {
            return Err(error(
                "AgentFlow metadata lineOffset requires a finite nonnegative integer",
            )
            .into());
        }
        let config = Object::default();
        let context = ParseContext {
            source: "",
            config: &config,
            title: None,
            line_offset: offset as usize,
            max_edges: MAX_ITEMS,
        };
        let db = AgentDb::new(&context);
        let value = db.metadata(&id, &source, loc)?;
        self.value(&value)
    }
    fn spread(&mut self, atom: &Atom) -> Result<Vec<(String, Atom)>, PublicError> {
        match atom {
            Atom::Null | Atom::Undefined | Atom::Bool(_) | Atom::Number(_) => Ok(vec![]),
            Atom::String(i) => {
                let units = self.g.strings[*i as usize].0.clone();
                units
                    .into_iter()
                    .enumerate()
                    .map(|(i, u)| self.string(JsString(vec![u])).map(|v| (i.to_string(), v)))
                    .collect()
            }
            Atom::Ref(id) => {
                let mut properties = self.g.nodes[*id as usize]
                    .properties()
                    .iter()
                    .filter(|p| p.flags & 1 != 0)
                    .cloned()
                    .collect::<Vec<_>>();
                properties.sort_by_key(|p| {
                    let s = self.g.strings[p.key as usize].well_formed();
                    s.as_ref()
                        .and_then(|s| {
                            s.parse::<u32>()
                                .ok()
                                .filter(|i| *i != u32::MAX && i.to_string() == *s)
                        })
                        .map_or((1, u32::MAX), |i| (0, i))
                });
                properties
                    .iter()
                    .map(|p| Ok((text(self.g, &Atom::String(p.key))?, p.value.clone())))
                    .collect()
            }
            _ => Err(error("AgentFlow function metadata spread is not implemented").into()),
        }
    }
    fn merge_metadata(&mut self, id: u32, doc: &Atom) -> Result<(), PublicError> {
        let old = self.field(id, "metadata");
        let target = if matches!(old, Atom::Undefined | Atom::Null) {
            self.object()?
        } else {
            self.id(&old)?
        };
        for (key, value) in self.spread(doc)? {
            self.assign(target, &key, value)?
        }
        self.assign(id, "metadata", Atom::Ref(target))
    }
    fn resolve_shape(&mut self, shape: Atom) -> Result<Atom, PublicError> {
        if let Atom::String(i) = shape {
            if let Some(s) = self.g.strings[i as usize].well_formed() {
                let resolved = super::shapes::resolve(&s);
                if resolved != s {
                    return self.string(resolved);
                }
            }
        }
        Ok(shape)
    }
    fn vertex(&mut self, root: u32, args: &[Atom]) -> Result<(), PublicError> {
        let argument = |n| args.get(n).cloned().unwrap_or(Atom::Undefined);
        let id = argument(0);
        if !truthy(self.g, &id) {
            return Ok(());
        }
        if raw_trim(self.g, &id, false)?.0.is_empty() {
            return Ok(());
        }
        let text_obj = argument(1);
        let shape = argument(2);
        let style = argument(3);
        let classes = argument(4);
        let dir = argument(5);
        let props = if matches!(argument(6), Atom::Undefined) {
            Atom::Ref(self.object()?)
        } else {
            argument(6)
        };
        let raw = argument(7);
        let loc = argument(8);
        let doc = if matches!(raw, Atom::Undefined) {
            Atom::Undefined
        } else {
            self.guard(root, "metadata", 1)?;
            self.metadata(root, &id, &raw, &loc)?
        };
        let fields = if let Atom::Ref(i) = doc {
            if matches!(
                self.g.nodes[i as usize],
                Node::Object { .. } | Node::Instance { .. }
            ) {
                Some(i)
            } else {
                None
            }
        } else {
            None
        };
        let mut authored = Atom::Undefined;
        if let Some(fields) = fields {
            let shape = self.field(fields, "shape");
            if matches!(shape, Atom::String(_)) {
                authored = shape.clone();
                let shape = self.resolve_shape(shape)?;
                self.assign(fields, "shape", shape)?
            }
        }
        for name in ["subgraphLookup", "connectors"] {
            let target = self.map_field(root, name, &id)?;
            if truthy(self.g, &target) && truthy(self.g, &doc) {
                let target = self.id(&target)?;
                return self.merge_metadata(target, &doc);
            }
        }
        if matches!(id,Atom::String(i)if self.g.strings[i as usize].is("connectors")) {
            return Ok(());
        }
        let edge = self.map_field(root, "edgeById", &id)?;
        if truthy(self.g, &edge) {
            let edge = self.id(&edge)?;
            if let Some(fields) = fields {
                for (from, to) in [
                    ("animate", "animate"),
                    ("animation", "animation"),
                    ("curve", "interpolate"),
                ] {
                    let v = self.field(fields, from);
                    if !matches!(v, Atom::Undefined) {
                        self.assign(edge, to, v)?
                    }
                }
            }
            if truthy(self.g, &doc) {
                self.merge_metadata(edge, &doc)?
            }
            return Ok(());
        }
        let nodes = self.field_id(root, "nodes")?;
        let old = self.map_get(nodes, &id)?;
        let vertex = if matches!(old, Atom::Undefined) {
            let styles = self.array(vec![])?;
            let classes = self.array(vec![])?;
            let kind = self.string("text")?;
            let vertex = self.record(vec![
                ("id", id.clone()),
                ("labelType", kind),
                ("styles", Atom::Ref(styles)),
                ("classes", Atom::Ref(classes)),
            ])?;
            self.map_set(nodes, id.clone(), Atom::Ref(vertex))?;
            vertex
        } else {
            self.id(&old)?
        };
        if !matches!(text_obj, Atom::Undefined) {
            let text_obj = self.id(&text_obj)?;
            self.assign(vertex, "labelType", self.field(text_obj, "type"))?;
            let text = unquote_units(raw_trim(self.g, &self.field(text_obj, "text"), false)?);
            let text = self.string(text)?;
            self.assign(vertex, "text", text)?;
        } else if matches!(self.field(vertex, "text"), Atom::Undefined) {
            self.assign(vertex, "text", id.clone())?
        }
        if !matches!(shape, Atom::Undefined) {
            self.assign(vertex, "type", shape)?
        }
        for (key, values) in [("styles", style), ("classes", classes)] {
            if !matches!(values, Atom::Undefined | Atom::Null) {
                let target = self.field_id(vertex, key)?;
                for value in dense(self.g, &values)? {
                    self.push(target, value)?
                }
            }
        }
        if !matches!(dir, Atom::Undefined) {
            self.assign(vertex, "dir", dir)?
        }
        let old = self.field(vertex, "props");
        if matches!(old, Atom::Undefined) {
            self.assign(vertex, "props", props)?
        } else if !matches!(props, Atom::Undefined | Atom::Null) {
            let target = self.id(&old)?;
            for (key, value) in self.spread(&props)? {
                self.assign(target, &key, value)?
            }
        }
        if !truthy(self.g, &doc) {
            return Ok(());
        }
        self.merge_metadata(vertex, &doc)?;
        let Some(fields) = fields else { return Ok(()) };
        let shape = self.field(fields, "shape");
        if truthy(self.g, &shape) {
            let raw = if matches!(authored, Atom::Undefined) {
                shape.clone()
            } else {
                authored
            };
            let name = if let Atom::String(_) = raw {
                text(self.g, &raw)?
            } else {
                let value = yaml_json(self.g, &raw)?;
                return Err(self.shape_failure(root, format!("No such shape: {value}."), &loc)?);
            };
            if name != name.to_lowercase() || name.contains('_') {
                return Err(self.shape_failure(
                    root,
                    format!("No such shape: {name}. Shape names should be lowercase."),
                    &loc,
                )?);
            }
            let shape_name = text(self.g, &shape)?;
            if !super::shapes::valid(&shape_name) {
                return Err(self.shape_failure(
                    root,
                    format!("No such shape: {shape_name}."),
                    &loc,
                )?);
            }
            self.assign(vertex, "type", shape)?;
        }
        let value = self.field(fields, "label");
        if truthy(self.g, &value) {
            let label = self.string(yaml_string(self.g, &value)?.as_str())?;
            self.assign(vertex, "text", label)?;
            let kind = self.field(fields, "labelType");
            let kind = if ["text", "string", "markdown"]
                .iter()
                .any(|s| matches!(&kind,Atom::String(i)if self.g.strings[*i as usize].is(s)))
            {
                kind
            } else {
                self.string("markdown")?
            };
            self.assign(vertex, "labelType", kind)?;
        }
        Ok(())
    }
    fn shape_failure(
        &self,
        root: u32,
        message: String,
        loc: &Atom,
    ) -> Result<PublicError, PublicError> {
        self.guard(root, "shapeError", 2)?;
        let mut e = error(message);
        if let Some(loc) = native_loc(self.g, loc)? {
            let offset = number(self.g, &self.field(root, "lineOffset"))?;
            if !offset.is_finite()
                || offset < 0.
                || offset.fract() != 0.
                || offset > usize::MAX as f64
            {
                return Err(error(
                    "AgentFlow shape error lineOffset is outside native integer coordinates",
                )
                .into());
            }
            e.line = loc.first_line.saturating_add(offset as usize);
            e.column = loc.first_column.saturating_add(1);
        }
        Ok(e.into())
    }
    fn ensure_set(&self, id: u32) -> Result<(), PublicError> {
        if self.sets.borrow().contains_key(&id) {
            return Ok(());
        }
        let Node::Set { values, .. } = &self.g.nodes[id as usize] else {
            return Err(ty("AgentFlow collection must be a Set"));
        };
        let mut index = HashMap::new();
        for (i, value) in values.iter().enumerate() {
            index.entry(self.key(value)).or_insert(i);
        }
        self.sets.borrow_mut().insert(
            id,
            CachedSet {
                values: values.iter().cloned().map(Some).collect(),
                index,
            },
        );
        Ok(())
    }
    fn set_has(&self, id: u32, value: &Atom) -> Result<bool, PublicError> {
        self.ensure_set(id)?;
        Ok(self.sets.borrow()[&id].index.contains_key(&self.key(value)))
    }
    fn set_add(&mut self, id: u32, value: Atom) -> Result<(), PublicError> {
        self.ensure_set(id)?;
        let key = self.key(&value);
        let set = self.sets.get_mut().get_mut(&id).unwrap();
        if !set.index.contains_key(&key) {
            if set.index.len() >= MAX_ITEMS || set.values.len() >= MAX_WORK {
                return Err(limit().into());
            }
            let index = set.values.len();
            set.values.push(Some(value));
            set.index.insert(key, index);
        }
        Ok(())
    }
    fn set_delete(&mut self, id: u32, value: &Atom) -> Result<bool, PublicError> {
        self.ensure_set(id)?;
        let key = self.key(value);
        let set = self.sets.get_mut().get_mut(&id).unwrap();
        if let Some(index) = set.index.remove(&key) {
            set.values[index] = None;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn flush_sets(&mut self) {
        for (id, set) in self.sets.get_mut().drain() {
            let Node::Set { values, .. } = &mut self.g.nodes[id as usize] else {
                unreachable!()
            };
            *values = set.values.into_iter().flatten().collect();
        }
    }
    fn list_nodes(&self, list: &Atom) -> Result<(Vec<Atom>, Atom), PublicError> {
        let mut nodes = Vec::new();
        let mut seen = HashSet::new();
        let mut dir = Atom::Undefined;
        let mut work = 0usize;
        for entry in dense(self.g, list)? {
            let values = match entry {
                Atom::String(_) => vec![entry],
                Atom::Ref(i) if matches!(self.g.nodes[i as usize], Node::Array { .. }) => {
                    dense(self.g, &entry)?
                }
                Atom::Ref(i) => {
                    dir = self.field(i, "value");
                    vec![]
                }
                _ => return Err(ty("AgentFlow document item must be an object or string")),
            };
            work = work.checked_add(values.len()).ok_or_else(limit)?;
            if work > MAX_WORK {
                return Err(limit().into());
            }
            for v in values {
                if !raw_trim(self.g, &v, false)?.0.is_empty() && seen.insert(self.key(&v)) {
                    nodes.push(v)
                }
                if nodes.len() > MAX_ITEMS {
                    return Err(limit().into());
                }
            }
        }
        Ok((nodes, dir))
    }
    fn add_subgraph(
        &mut self,
        root: u32,
        id_text: Atom,
        list: Atom,
        title_text: Atom,
    ) -> Result<Atom, PublicError> {
        let mut pending = self.subgraph_prepare(root, id_text, list, title_text)?;
        let mut dir = pending.dir;
        let options = self.field_id(root, "options")?;
        if matches!(dir, Atom::Undefined | Atom::Null)
            && truthy(self.g, &self.field(options, "inheritDir"))
        {
            let d = self.field(root, "direction");
            dir = if matches!(d, Atom::Undefined | Atom::Null) {
                self.field(options, "direction")
            } else {
                d
            };
        }
        pending.dir = dir;
        self.subgraph_finish(pending)
    }
    fn subgraph_prepare(
        &mut self,
        root: u32,
        id_text: Atom,
        list: Atom,
        title_text: Atom,
    ) -> Result<live_options::SubgraphPending, PublicError> {
        let mut id = if let Atom::Ref(i) = id_text {
            let text = self.field(i, "text");
            if matches!(text, Atom::Undefined | Atom::Null) {
                None
            } else {
                Some(raw_trim(self.g, &text, false)?)
            }
        } else {
            None
        };
        let title = if let Atom::Ref(i) = title_text {
            let text = self.field(i, "text");
            if truthy(self.g, &text) {
                text
            } else {
                self.string("")?
            }
        } else {
            self.string("")?
        };
        if !matches!(id_text, Atom::Undefined) && same(self.g, &id_text, &title_text) {
            if let Atom::String(i) = title {
                if self.g.strings[i as usize]
                    .0
                    .iter()
                    .any(|u| char::from_u32(*u as u32).is_some_and(space))
                {
                    id = None
                }
            }
        }
        let (nodes, dir) = self.list_nodes(&list)?;
        Ok(live_options::SubgraphPending {
            root,
            id,
            nodes,
            dir,
            title,
            title_text,
        })
    }
    fn subgraph_finish(
        &mut self,
        pending: live_options::SubgraphPending,
    ) -> Result<Atom, PublicError> {
        let live_options::SubgraphPending {
            root,
            id,
            nodes,
            dir,
            title,
            title_text,
        } = pending;
        let count = number(self.g, &self.field(root, "subCount"))?;
        let id = match id {
            Some(id) => self.string(id)?,
            None => {
                self.string(format!("subGraph{}", Value::Number(count).js_string()).as_str())?
            }
        };
        self.assign(root, "subCount", Atom::Number(count + 1.))?;
        let members = self.field_id(root, "members")?;
        let globals = self.field_id(root, "globalNodes")?;
        let mut kept = Vec::new();
        for n in nodes {
            if !same(self.g, &n, &id)
                && !self.set_has(globals, &n)?
                && !self.set_has(members, &n)?
            {
                kept.push(n)
            }
        }
        for n in &kept {
            self.set_add(members, n.clone())?
        }
        let lookup = self.field_id(root, "subgraphLookup")?;
        let existing = self.map_get(lookup, &id)?;
        let title = raw_trim(self.g, &title, false)?;
        if truthy(self.g, &existing) {
            let existing = self.id(&existing)?;
            let target = self.field_id(existing, "nodes")?;
            for n in kept {
                self.push(target, n)?
            }
            if !title.0.is_empty() {
                let title = self.string(title)?;
                self.assign(existing, "title", title)?
            }
            if truthy(self.g, &dir) {
                self.assign(existing, "dir", dir)?
            }
            return Ok(id);
        }
        let nodes = self.array(kept)?;
        let classes = self.array(vec![])?;
        let title = self.string(title)?;
        let kind = if let Atom::Ref(i) = title_text {
            self.field(i, "type")
        } else {
            Atom::Undefined
        };
        let kind = if ["text", "string", "markdown"]
            .iter()
            .any(|s| matches!(&kind,Atom::String(i)if self.g.strings[*i as usize].is(s)))
        {
            kind
        } else {
            self.string("markdown")?
        };
        let flow = self.string("flow")?;
        let sg = self.record(vec![
            ("id", id.clone()),
            ("nodes", Atom::Ref(nodes)),
            ("title", title),
            ("classes", Atom::Ref(classes)),
            ("dir", dir),
            ("labelType", kind),
            ("type", flow),
        ])?;
        let subgraphs = self.field_id(root, "subgraphs")?;
        self.push(subgraphs, Atom::Ref(sg))?;
        self.map_set(lookup, id.clone(), Atom::Ref(sg))?;
        Ok(id)
    }
    fn add_global(&mut self, root: u32, list: &Atom) -> Result<(), PublicError> {
        let globals = self.field_id(root, "globalNodes")?;
        let members = self.field_id(root, "members")?;
        let mut release = false;
        let mut work = 0usize;
        for entry in dense(self.g, list)? {
            let values = match entry {
                Atom::String(_) => vec![entry],
                Atom::Ref(i) if matches!(self.g.nodes[i as usize], Node::Array { .. }) => {
                    dense(self.g, &entry)?
                }
                _ => vec![],
            };
            work = work.checked_add(values.len()).ok_or_else(limit)?;
            if work > MAX_WORK {
                return Err(limit().into());
            }
            for item in values {
                let item = raw_trim(self.g, &item, false)?;
                if item.0.is_empty() {
                    continue;
                }
                let item = self.string(item)?;
                self.set_add(globals, item.clone())?;
                release |= self.set_delete(members, &item)?;
            }
        }
        if release {
            for sg in dense(self.g, &self.field(root, "subgraphs"))? {
                let sg = self.id(&sg)?;
                let nodes = dense(self.g, &self.field(sg, "nodes"))?;
                work = work.checked_add(nodes.len()).ok_or_else(limit)?;
                if work > MAX_WORK {
                    return Err(limit().into());
                }
                let mut kept = Vec::new();
                for node in nodes {
                    if !self.set_has(globals, &node)? {
                        kept.push(node)
                    }
                }
                let nodes = self.array(kept)?;
                self.assign(sg, "nodes", Atom::Ref(nodes))?;
            }
        }
        Ok(())
    }
    fn add_connector(
        &mut self,
        root: u32,
        text_obj: &Atom,
        title_obj: &Atom,
    ) -> Result<Atom, PublicError> {
        let id = if let Atom::Ref(i) = text_obj {
            let t = self.field(*i, "text");
            if matches!(t, Atom::Undefined | Atom::Null) {
                JsString(vec![])
            } else {
                raw_trim(self.g, &t, false)?
            }
        } else {
            JsString(vec![])
        };
        if id.0.is_empty() {
            return self.string("");
        }
        let title = if let Atom::Ref(i) = title_obj {
            let t = self.field(*i, "text");
            if matches!(t, Atom::Undefined | Atom::Null) {
                JsString(vec![])
            } else {
                raw_trim(self.g, &t, false)?
            }
        } else {
            JsString(vec![])
        };
        let id = self.string(id)?;
        let connectors = self.field_id(root, "connectors")?;
        let existing = self.map_get(connectors, &id)?;
        if !truthy(self.g, &existing) {
            let styles = self.array(vec![])?;
            let classes = self.array(vec![])?;
            let kind = self.string("text")?;
            let text = if title.0.is_empty() {
                id.clone()
            } else {
                self.string(title)?
            };
            let node = self.record(vec![
                ("id", id.clone()),
                ("labelType", kind),
                ("styles", Atom::Ref(styles)),
                ("classes", Atom::Ref(classes)),
                ("isConnector", Atom::Bool(true)),
                ("text", text),
            ])?;
            self.map_set(connectors, id.clone(), Atom::Ref(node))?;
            let nodes = self.field_id(root, "nodes")?;
            self.map_set(nodes, id.clone(), Atom::Ref(node))?;
        } else if !title.0.is_empty() {
            let node = self.id(&existing)?;
            let title = self.string(title)?;
            self.assign(node, "text", title)?;
        }
        Ok(id)
    }
}
impl Arena<'_> {
    fn position(&mut self, root: u32, loc: &Atom) -> Result<Atom, PublicError> {
        let loc = if let Atom::Ref(i) = loc {
            Some(*i)
        } else {
            None
        };
        let value = |key: &str| loc.map_or(Atom::Undefined, |i| self.field(i, key));
        let nullish = |a: Atom, fallback: Atom| {
            if matches!(a, Atom::Undefined | Atom::Null) {
                fallback
            } else {
                a
            }
        };
        let start = nullish(value("first_line"), Atom::Number(0.));
        let column = nullish(value("first_column"), Atom::Number(0.));
        let end = nullish(value("last_line"), start.clone());
        let end_column = nullish(value("last_column"), column.clone());
        let offset = number(self.g, &self.field(root, "lineOffset"))?;
        let start = number(self.g, &start)? + offset;
        let end = number(self.g, &end)? + offset;
        let out = self.record(vec![
            ("startLine", Atom::Number(start)),
            ("startColumn", column),
            ("endLine", Atom::Number(end)),
            ("endColumn", end_column),
            ("startIndex", Atom::Number(0.)),
            ("endIndex", Atom::Number(0.)),
        ])?;
        Ok(Atom::Ref(out))
    }
    fn push_mapping(
        &mut self,
        root: u32,
        id: Atom,
        kind: Atom,
        position: Atom,
    ) -> Result<Atom, PublicError> {
        let mapping = self.record(vec![
            ("id", id.clone()),
            ("type", kind),
            ("position", position),
        ])?;
        let mappings = self.field_id(root, "mappings")?;
        self.push(mappings, Atom::Ref(mapping))?;
        let first = self.field_id(root, "firstMapping")?;
        if !self.map_has(first, &id)? {
            self.map_set(first, id, Atom::Ref(mapping))?
        }
        Ok(Atom::Ref(mapping))
    }
    fn block_mapping(
        &mut self,
        root: u32,
        id: Atom,
        kind: Atom,
        start: Atom,
        end: Atom,
    ) -> Result<(), PublicError> {
        if !truthy(self.g, &id) {
            return Ok(());
        }
        self.guard(root, "position", 24)?;
        let start_position = self.position(root, &start)?;
        let end_position = if truthy(self.g, &end) {
            self.position(root, &end)?
        } else {
            start_position.clone()
        };
        let start = self.id(&start_position)?;
        let end = self.id(&end_position)?;
        let position = self.record(vec![
            ("startLine", self.field(start, "startLine")),
            ("startColumn", self.field(start, "startColumn")),
            ("endLine", self.field(end, "endLine")),
            ("endColumn", self.field(end, "endColumn")),
            ("startIndex", Atom::Number(0.)),
            ("endIndex", Atom::Number(0.)),
        ])?;
        self.guard(root, "pushMapping", 25)?;
        self.push_mapping(root, id, kind, Atom::Ref(position))?;
        Ok(())
    }
    fn emit(
        &mut self,
        root: u32,
        id: Atom,
        severity: Atom,
        message: Atom,
        ctx: Atom,
    ) -> Result<(), PublicError> {
        if self
            .emit_prepare(root, id, severity, message, ctx)?
            .is_none()
        {
            return Ok(());
        }
        let options = self.field_id(root, "options")?;
        let log = self.field(options, "log");
        if !matches!(log, Atom::Undefined | Atom::Null) {
            return Err(
                if matches!(log, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
                    error("AgentFlow diagnostic log callback dispatch is not implemented").into()
                } else {
                    ty("AgentFlow diagnostic log callback is not a function")
                },
            );
        }
        Ok(())
    }
    fn emit_prepare(
        &mut self,
        root: u32,
        id: Atom,
        severity: Atom,
        message: Atom,
        ctx: Atom,
    ) -> Result<Option<live_options::DiagnosticPending>, PublicError> {
        let (ctx_node, ctx_edge) = if let Atom::Ref(i) = ctx {
            (self.field(i, "nodeId"), self.field(i, "edgeId"))
        } else {
            (Atom::Undefined, Atom::Undefined)
        };
        let anchor = if matches!(ctx_node, Atom::Undefined | Atom::Null) {
            ctx_edge.clone()
        } else {
            ctx_node.clone()
        };
        let mut fields = vec![
            ("id", id.clone()),
            ("severity", severity.clone()),
            ("message", message.clone()),
        ];
        let (node, edge) = if truthy(self.g, &ctx_node) {
            fields.push(("nodeId", ctx_node.clone()));
            (ctx_node, Atom::Undefined)
        } else if truthy(self.g, &ctx_edge) {
            fields.push(("edgeId", ctx_edge.clone()));
            (Atom::Undefined, ctx_edge)
        } else {
            (Atom::Undefined, Atom::Undefined)
        };
        let mapping = if truthy(self.g, &anchor) {
            self.map_field(root, "firstMapping", &anchor)?
        } else {
            Atom::Undefined
        };
        if truthy(self.g, &mapping) {
            let mapping = self.id(&mapping)?;
            fields.push(("position", self.field(mapping, "position")));
        }
        let diagnostic = self.record(fields)?;
        let key = if matches!(anchor, Atom::Undefined | Atom::Null) {
            self.string("")?
        } else {
            anchor
        };
        let reported = self.field_id(root, "reported")?;
        let previous = self.map_get(reported, &key)?;
        if matches!(previous, Atom::Undefined) {
            let list = self.array(vec![Atom::Ref(diagnostic)])?;
            self.map_set(reported, key, Atom::Ref(list))?;
        } else {
            let list = self.id(&previous)?;
            for d in dense(self.g, &previous)? {
                let d = self.id(&d)?;
                if same(self.g, &self.field(d, "id"), &id)
                    && same(self.g, &self.field(d, "severity"), &severity)
                    && same(self.g, &self.field(d, "message"), &message)
                    && same(self.g, &self.field(d, "nodeId"), &node)
                    && same(self.g, &self.field(d, "edgeId"), &edge)
                {
                    return Ok(None);
                }
            }
            self.push(list, Atom::Ref(diagnostic))?;
        }
        let diagnostics = self.field_id(root, "diagnostics")?;
        self.push(diagnostics, Atom::Ref(diagnostic))?;
        Ok(Some(live_options::DiagnosticPending {
            id,
            severity,
            message,
        }))
    }
    fn vertex_kind(&self, vertex: u32) -> Result<&'static str, PublicError> {
        let shape = self.field(vertex, "type");
        let shape = if let Atom::String(i) = shape {
            self.g.strings[i as usize].well_formed()
        } else {
            None
        };
        let node = AgentNode {
            id: String::new(),
            text: None,
            label_type: LabelType::Text,
            shape,
            styles: vec![],
            classes: vec![],
            dir: None,
            props: None,
            link: None,
            link_target: None,
            metadata: None,
            is_connector: None,
        };
        Ok(super::model::vertex_kind(&node))
    }
    fn semantic_metadata(&mut self, metadata: Atom) -> Result<Option<u32>, PublicError> {
        if !truthy(self.g, &metadata) {
            return Ok(None);
        }
        let mut fields = Vec::new();
        for (key, value) in self.spread(&metadata)? {
            if ![
                "shape",
                "view",
                "icon",
                "img",
                "form",
                "pos",
                "w",
                "h",
                "class",
                "style",
                "labelType",
            ]
            .contains(&key.as_str())
            {
                fields.push((key, value));
            }
        }
        if fields.is_empty() {
            return Ok(None);
        }
        let id = self.object()?;
        for (key, value) in fields {
            self.assign(id, &key, value)?
        }
        Ok(Some(id))
    }
    fn semantic_model(&mut self, root: u32) -> Result<Atom, PublicError> {
        let mut vertices = Vec::new();
        let mut connectors = Vec::new();
        let nodes = self.field(root, "nodes");
        for (id, vertex) in entries(self.g, &nodes)? {
            let lookup = self.field_id(root, "subgraphLookup")?;
            if self.map_has(lookup, &id)? {
                continue;
            }
            let vertex = self.id(&vertex)?;
            let metadata = self.semantic_metadata(self.field(vertex, "metadata"))?;
            let mut fields = vec![("id", id.clone())];
            if truthy(self.g, &self.field(vertex, "isConnector")) {
                let text = self.field(vertex, "text");
                if !matches!(text, Atom::Undefined) && !same(self.g, &text, &id) {
                    fields.push(("title", text));
                }
                if let Some(m) = metadata {
                    fields.push(("metadata", Atom::Ref(m)));
                }
                connectors.push(Atom::Ref(self.record(fields)?));
            } else {
                let label = self.field(vertex, "text");
                if !matches!(label, Atom::Undefined) {
                    fields.push(("label", label));
                }
                let shape = self.resolve_shape(self.field(vertex, "type"))?;
                if !matches!(shape, Atom::Undefined) {
                    fields.push(("shape", shape));
                }
                self.guard(root, "vertexKind", 22)?;
                let kind = self.vertex_kind(vertex)?;
                fields.push(("vertexKind", self.string(kind)?));
                if let Some(m) = metadata {
                    fields.push(("metadata", Atom::Ref(m)));
                }
                vertices.push(Atom::Ref(self.record(fields)?));
            }
        }
        let mut edges = Vec::new();
        for edge in dense(self.g, &self.field(root, "edges"))? {
            let edge = self.id(&edge)?;
            let mut fields = vec![
                ("start", self.field(edge, "start")),
                ("end", self.field(edge, "end")),
            ];
            let id = self.field(edge, "id");
            if !matches!(id, Atom::Undefined) {
                fields.push(("id", id));
            }
            let text = self.field(edge, "text");
            let Atom::String(i) = text else {
                return Err(ty("AgentFlow semantic edge text must be a string"));
            };
            if !self.g.strings[i as usize].0.is_empty() {
                fields.push(("label", text));
            }
            for key in ["type", "stroke", "edgeSemantic", "length"] {
                let v = self.field(edge, key);
                if !matches!(v, Atom::Undefined) {
                    fields.push((key, v));
                }
            }
            let metadata = self.field(edge, "metadata");
            if truthy(self.g, &metadata) {
                let fields_metadata = self.spread(&metadata)?;
                if !fields_metadata.is_empty() {
                    let out = self.object()?;
                    for (key, value) in fields_metadata {
                        self.assign(out, &key, value)?
                    }
                    fields.push(("metadata", Atom::Ref(out)));
                }
            }
            edges.push(Atom::Ref(self.record(fields)?));
        }
        let mut groups = Vec::new();
        let mut work = 0usize;
        for group in dense(self.g, &self.field(root, "subgraphs"))? {
            let group = self.id(&group)?;
            let values = dense(self.g, &self.field(group, "nodes"))?;
            work = work.checked_add(values.len()).ok_or_else(limit)?;
            if work > MAX_WORK {
                return Err(limit().into());
            }
            let nodes = self.array(values)?;
            let mut fields = vec![
                ("id", self.field(group, "id")),
                ("nodes", Atom::Ref(nodes)),
                ("type", self.field(group, "type")),
                ("title", self.field(group, "title")),
            ];
            let direction = self.field(group, "dir");
            if !matches!(direction, Atom::Undefined) {
                fields.push(("direction", direction));
            }
            if let Some(metadata) = self.semantic_metadata(self.field(group, "metadata"))? {
                fields.push(("metadata", Atom::Ref(metadata)));
            }
            groups.push(Atom::Ref(self.record(fields)?));
        }
        let vertices = self.array(vertices)?;
        let edges = self.array(edges)?;
        let groups = self.array(groups)?;
        let connectors = self.array(connectors)?;
        let mut fields = vec![
            ("vertices", Atom::Ref(vertices)),
            ("edges", Atom::Ref(edges)),
            ("subGraphs", Atom::Ref(groups)),
            ("connectors", Atom::Ref(connectors)),
            ("diagnostics", self.field(root, "diagnostics")),
        ];
        let direction = self.field(root, "direction");
        if !matches!(direction, Atom::Undefined) {
            fields.push(("direction", direction));
        }
        Ok(Atom::Ref(self.record(fields)?))
    }
}
fn destruct_agent(
    end: &JsString,
    start: Option<&JsString>,
) -> (String, &'static str, f64, Option<&'static str>) {
    let units = &end.0;
    let mut line = &units[..units.len().saturating_sub(1)];
    let kind = match units.last() {
        Some(120) => "arrow_cross",
        Some(62) => "arrow_point",
        Some(45 | 46) => {
            line = units;
            "arrow_open"
        }
        _ => "arrow_open",
    };
    let dots = line.iter().filter(|u| **u == 46).count();
    let stroke = if dots > 0 { "dotted" } else { "normal" };
    let length = if dots > 0 {
        dots as f64
    } else {
        line.len() as f64 - 1.
    };
    if let Some(start) = start {
        if (if start.0.contains(&46) {
            "dotted"
        } else {
            "normal"
        }) != stroke
        {
            return ("INVALID".into(), "INVALID", length, None);
        }
    }
    let semantic = match (kind, stroke) {
        ("arrow_point", "normal") => Some("sequence"),
        ("arrow_cross", "normal") => Some("failure"),
        ("arrow_open", "dotted") => Some("reference"),
        _ => None,
    };
    (kind.into(), stroke, length, semantic)
}

fn invoke_common(a: &mut Arena, mut call: MethodCall) -> Result<Atom, PublicError> {
    call.method = MethodId(match call.method.0 {
        3 => 1,
        4 => 2,
        5 => 3,
        6 => 4,
        7 => 5,
        8 => 6,
        9 => 7,
        10 => 8,
        11 => 9,
        12 => 10,
        13 => 11,
        14 => 12,
        15 => 13,
        16 => 14,
        17 => 15,
        _ => return Err(error("Unknown AgentFlow native method").into()),
    });
    let root = a.id(&call.receiver)?;
    match call.method.0 {
        1 => a.vertex(root, &call.arguments)?,
        2 => a.single_link(
            root,
            arg(&call, 0),
            arg(&call, 1),
            arg(&call, 2),
            arg(&call, 3),
        )?,
        3 => {
            a.guard(root, "addSingleLink", 4)?;
            let starts = dense(a.g, &arg(&call, 0))?;
            let ends = dense(a.g, &arg(&call, 1))?;
            if starts
                .len()
                .checked_mul(ends.len())
                .is_none_or(|n| n > MAX_WORK)
            {
                return Err(limit().into());
            }
            let info = arg(&call, 2);
            let id = if let Atom::Ref(i) = info {
                let id = a.field(i, "id");
                if let Atom::String(s) = id {
                    let s = a.g.strings[s as usize].0.clone();
                    let mut out = Vec::with_capacity(s.len());
                    let mut removed = false;
                    for c in s {
                        if c == 64 && !removed {
                            removed = true
                        } else {
                            out.push(c)
                        }
                    }
                    a.string(JsString(out))?
                } else {
                    Atom::Undefined
                }
            } else {
                Atom::Undefined
            };
            for start in &starts {
                for end in &ends {
                    let id = if starts.last().is_some_and(|s| same(a.g, s, start))
                        && ends.first().is_some_and(|e| same(a.g, e, end))
                    {
                        id.clone()
                    } else {
                        Atom::Undefined
                    };
                    a.single_link(root, start.clone(), end.clone(), info.clone(), id)?
                }
            }
        }
        4 | 5 => {
            let value = arg(&call, 1);
            for pos in dense(a.g, &arg(&call, 0))? {
                if matches!(pos,Atom::String(i)if a.g.strings[i as usize].is("default")) {
                    a.assign(
                        root,
                        if call.method.0 == 4 {
                            "defaultInterpolate"
                        } else {
                            "defaultEdgeStyle"
                        },
                        value.clone(),
                    )?;
                    continue;
                }
                a.guard(root, "edgeAt", 8)?;
                let edge = a.edge_at(root, &pos)?;
                let edge = a.id(&edge)?;
                a.assign(
                    edge,
                    if call.method.0 == 4 {
                        "interpolate"
                    } else {
                        "style"
                    },
                    value.clone(),
                )?;
                if call.method.0 == 5 {
                    let styles = dense(a.g, &value)?;
                    if !styles.is_empty() {
                        let mut fill = false;
                        for s in styles {
                            if matches!(s, Atom::Undefined | Atom::Null) {
                                continue;
                            }
                            if text(a.g, &s)?.starts_with("fill") {
                                fill = true;
                                break;
                            }
                        }
                        if !fill {
                            let styles = a.id(&value)?;
                            let v = a.string("fill:none")?;
                            a.push(styles, v)?
                        }
                    }
                }
            }
        }
        6 => return a.edge_at(root, &arg(&call, 0)),
        7 => {
            let ids = text(a.g, &arg(&call, 0))?;
            let styles = dense(a.g, &arg(&call, 1))?;
            let mut units = styles.len().saturating_sub(1);
            for v in &styles {
                units = units
                    .checked_add(match v {
                        Atom::String(i) => a.g.strings[*i as usize].0.len(),
                        Atom::Null | Atom::Undefined => 0,
                        _ => js_string(a.g, v)?.encode_utf16().count(),
                    })
                    .ok_or_else(limit)?;
                if units > MAX_TEXT {
                    return Err(limit().into());
                }
            }
            let joined = styles
                .iter()
                .map(|v| {
                    if matches!(v, Atom::Null | Atom::Undefined) {
                        Ok(String::new())
                    } else {
                        js_string(a.g, v)
                    }
                })
                .collect::<Result<Vec<_>, PublicError>>()?
                .join(",")
                .replace("\\,", "§§§")
                .replace(',', ";")
                .replace("§§§", ",");
            let classes = a.field_id(root, "classes")?;
            for id in ids.split(',') {
                let key = a.string(id)?;
                let old = a.map_get(classes, &key)?;
                let node = if matches!(old, Atom::Undefined) {
                    let styles = a.array(vec![])?;
                    let text = a.array(vec![])?;
                    let node = a.record(vec![
                        ("id", key.clone()),
                        ("styles", Atom::Ref(styles)),
                        ("textStyles", Atom::Ref(text)),
                    ])?;
                    a.map_set(classes, key, Atom::Ref(node))?;
                    node
                } else {
                    a.id(&old)?
                };
                for style in joined.split(';') {
                    if style.contains("color") {
                        let s = a.string(style.replacen("fill", "bgFill", 1).as_str())?;
                        let target = a.field_id(node, "textStyles")?;
                        a.push(target, s)?;
                    }
                    let s = a.string(style)?;
                    let target = a.field_id(node, "styles")?;
                    a.push(target, s)?;
                }
            }
        }
        8 => {
            let mut d = raw_trim(a.g, &arg(&call, 0), false)?;
            for (c, to) in [(60, "RL"), (94, "BT"), (62, "LR"), (118, "TB")] {
                if d.0.contains(&c) {
                    d = to.into()
                }
            }
            if d.is("TD") {
                d = "TB".into()
            }
            let d = a.string(d)?;
            a.assign(root, "direction", d)?;
        }
        9 => {
            let ids = text(a.g, &arg(&call, 0))?;
            a.set_class(root, &ids, arg(&call, 1))?;
        }
        10 => {
            let tooltip = arg(&call, 1);
            if !matches!(tooltip, Atom::Undefined) {
                let ids = text(a.g, &arg(&call, 0))?;
                let map = a.field_id(root, "tooltips")?;
                for id in ids.split(',') {
                    let id = a.string(id)?;
                    a.map_set(map, id, tooltip.clone())?;
                }
            }
        }
        11 => {
            let ids = text(a.g, &arg(&call, 0))?;
            let url = raw_trim(a.g, &arg(&call, 1), false)?;
            for id in ids.split(',') {
                let key = a.string(id)?;
                let node = a.map_field(root, "nodes", &key)?;
                if !matches!(node, Atom::Undefined) {
                    let node = a.id(&node)?;
                    let link = if url.0.is_empty() {
                        Atom::Undefined
                    } else {
                        let url = url.well_formed().ok_or_else(|| {
                            error("AgentFlow URL UTF-16 rendering is not implemented")
                        })?;
                        a.string(crate::compat::source::url::safe_url(&url)?.as_str())?
                    };
                    a.assign(node, "link", link)?;
                    a.assign(node, "linkTarget", arg(&call, 2))?;
                }
            }
            a.guard(root, "setClass", 11)?;
            let clickable = a.string("clickable")?;
            a.set_class(root, &ids, clickable)?;
        }
        12 => {
            let ids = text(a.g, &arg(&call, 0))?;
            a.guard(root, "setClass", 11)?;
            let clickable = a.string("clickable")?;
            a.set_class(root, &ids, clickable)?;
        }
        13 | 14 => {
            let value = arg(&call, 0);
            let value = if call.method.0 == 13 {
                raw_trim(a.g, &value, true)?
            } else {
                let Atom::String(i) = value else {
                    return Err(ty("AgentFlow accessibility argument must be a string"));
                };
                let mut out = Vec::new();
                let mut after = false;
                for &c in &a.g.strings[i as usize].0 {
                    if after && char::from_u32(c as u32).is_some_and(space) {
                        continue;
                    }
                    out.push(c);
                    after = c == 10;
                }
                JsString(out)
            };
            let value = a.string(value)?;
            a.assign(
                root,
                if call.method.0 == 13 {
                    "accTitle"
                } else {
                    "accDescr"
                },
                value,
            )?;
        }
        15 => return a.add_subgraph(root, arg(&call, 0), arg(&call, 1), arg(&call, 2)),
        _ => return Err(error("AgentFlow public method is unavailable").into()),
    }
    Ok(Atom::Undefined)
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let mut arena = Arena::new(g);
    let result = invoke_agent(&mut arena, call);
    arena.flush_sets();
    result
}
fn invoke_agent(a: &mut Arena, call: MethodCall) -> Result<Atom, PublicError> {
    let root = a.id(&call.receiver)?;
    match call.method.0 {
        1 => a.metadata(root, &arg(&call, 0), &arg(&call, 1), &arg(&call, 2)),
        2 => {
            fn scalar(a: &mut Arena<'_>, v: &Atom) -> Result<JsString, PublicError> {
                if let Atom::String(i) = v {
                    return Ok(a.g.strings[*i as usize].clone());
                }
                Ok(js_string(a.g, v)?.as_str().into())
            }
            fn plus(a: &mut Arena<'_>, left: Atom, right: Atom) -> Result<Atom, PublicError> {
                if matches!(left, Atom::String(_)) || matches!(right, Atom::String(_)) {
                    let mut value = scalar(a, &left)?.0;
                    value.extend(scalar(a, &right)?.0);
                    a.string(JsString(value))
                } else {
                    Ok(Atom::Number(number(a.g, &left)? + number(a.g, &right)?))
                }
            }
            let loc = arg(&call, 1);
            let (line, column) = if truthy(a.g, &loc) {
                let (line, column) = if let Atom::Ref(id) = loc {
                    (a.field(id, "first_line"), a.field(id, "first_column"))
                } else {
                    (Atom::Undefined, Atom::Undefined)
                };
                let offset = a.field(root, "lineOffset");
                (plus(a, line, offset)?, plus(a, column, Atom::Number(1.))?)
            } else {
                (Atom::Number(0.), Atom::Number(0.))
            };
            // Error constructor stringifies its message before adding own fields.
            let message = arg(&call, 0);
            let mut fields = vec![];
            if !matches!(message, Atom::Undefined) {
                let message = scalar(a, &message)?;
                fields.push(("message", a.string(message)?));
            }
            fields.extend([
                ("code", a.string("semantic")?),
                ("type", a.string("agentflow")?),
                ("line", line),
                ("column", column),
                ("snippet", a.string("")?),
                ("name", a.string("PeleError")?),
            ]);
            let id = a.node(Node::Error {
                error_type: ErrorObjectKind::Pele,
                properties: vec![],
            })?;
            for (key, value) in fields {
                a.assign(id, key, value)?;
                if key == "message" {
                    let key = a.property_position(id, key).unwrap();
                    a.g.nodes[id as usize].properties_mut()[key].flags = 6;
                }
            }
            Ok(Atom::Ref(id))
        }
        3..=17 => invoke_common(a, call),
        18 => {
            a.add_global(root, &arg(&call, 0))?;
            Ok(Atom::Undefined)
        }
        19 => a.add_connector(root, &arg(&call, 0), &arg(&call, 1)),
        20 => {
            let end = raw_trim(a.g, &arg(&call, 0), false)?;
            let start = arg(&call, 1);
            let start = if truthy(a.g, &start) {
                let Atom::String(i) = start else {
                    return Err(ty("AgentFlow link start must be a string"));
                };
                Some(a.g.strings[i as usize].clone())
            } else {
                None
            };
            let (kind, stroke, length, semantic) = destruct_agent(&end, start.as_ref());
            let kind_atom = a.string(kind.as_str())?;
            let stroke_atom = a.string(stroke)?;
            let mut fields = vec![("type", kind_atom), ("stroke", stroke_atom)];
            if kind != "INVALID" {
                fields.push(("length", Atom::Number(length)));
                let semantic = semantic.map_or(Ok(Atom::Undefined), |s| a.string(s))?;
                fields.push(("edgeSemantic", semantic));
            }
            Ok(Atom::Ref(a.record(fields)?))
        }
        21 => a.map_field(root, "subgraphLookup", &arg(&call, 0)),
        22 => {
            let vertex = a.id(&arg(&call, 0))?;
            let kind = a.vertex_kind(vertex)?;
            a.string(kind)
        }
        23 => a.semantic_model(root),
        24 => a.position(root, &arg(&call, 0)),
        25 => a.push_mapping(root, arg(&call, 0), arg(&call, 1), arg(&call, 2)),
        26 => {
            let id = arg(&call, 0);
            if !truthy(a.g, &id) {
                return Ok(Atom::Undefined);
            }
            a.guard(root, "position", 24)?;
            let position = a.position(root, &arg(&call, 3))?;
            a.guard(root, "pushMapping", 25)?;
            let kind = a.string("vertex")?;
            let mapping = a.push_mapping(root, id, kind, position)?;
            if matches!(arg(&call, 2), Atom::Undefined | Atom::Null) {
                let bare = a.field_id(root, "bareMappings")?;
                a.set_add(bare, mapping)?;
            }
            Ok(Atom::Undefined)
        }
        27 => {
            let id = arg(&call, 0);
            let loc = arg(&call, 1);
            if !truthy(a.g, &id) || !truthy(a.g, &loc) {
                return Ok(Atom::Undefined);
            }
            a.guard(root, "position", 24)?;
            let end = a.position(root, &loc)?;
            let end = a.id(&end)?;
            let mappings = dense(a.g, &a.field(root, "mappings"))?;
            for mapping in mappings.into_iter().rev() {
                let m = a.id(&mapping)?;
                if !matches!(a.field(m,"type"),Atom::String(i)if a.g.strings[i as usize].is("vertex"))
                    || !same(a.g, &a.field(m, "id"), &id)
                {
                    continue;
                }
                let bare = a.field_id(root, "bareMappings")?;
                if a.set_delete(bare, &mapping)? {
                    let kind = a.string("attachment")?;
                    a.assign(m, "type", kind)?;
                }
                let p = a.field_id(m, "position")?;
                let end_line = a.field(end, "endLine");
                let old_line = a.field(p, "endLine");
                let end_column = a.field(end, "endColumn");
                let old_column = a.field(p, "endColumn");
                if number(a.g, &end_line)? > number(a.g, &old_line)?
                    || strict(a.g, &end_line, &old_line)
                        && number(a.g, &end_column)? > number(a.g, &old_column)?
                {
                    a.assign(p, "endLine", end_line)?;
                    a.assign(p, "endColumn", end_column)?;
                }
                return Ok(Atom::Undefined);
            }
            a.guard(root, "pushMapping", 25)?;
            let kind = a.string("attachment")?;
            a.push_mapping(root, id, kind, Atom::Ref(end))?;
            Ok(Atom::Undefined)
        }
        28 => {
            let to = arg(&call, 1);
            let mut ids = Vec::new();
            let mut units = 0usize;
            if let Atom::Ref(i) = to {
                if matches!(a.g.nodes[i as usize], Node::Array { .. }) {
                    for id in dense(a.g, &to)? {
                        if let Atom::String(i) = id {
                            if !a.g.strings[i as usize].0.is_empty() {
                                units = units
                                    .checked_add(a.g.strings[i as usize].0.len() + 1)
                                    .ok_or_else(limit)?;
                                if units > MAX_TEXT {
                                    return Err(limit().into());
                                }
                                ids.push(a.g.strings[i as usize].clone());
                            }
                        }
                    }
                }
            }
            let text = if ids.is_empty() {
                JsString::from("edge")
            } else {
                let mut text = Vec::new();
                for (i, id) in ids.into_iter().enumerate() {
                    if i > 0 {
                        text.push(62)
                    }
                    text.extend(id.0);
                }
                JsString(text)
            };
            let id = a.string(text)?;
            a.guard(root, "position", 24)?;
            let position = a.position(root, &arg(&call, 3))?;
            a.guard(root, "pushMapping", 25)?;
            let kind = a.string("edge")?;
            a.push_mapping(root, id, kind, position)?;
            Ok(Atom::Undefined)
        }
        29 => {
            a.block_mapping(
                root,
                arg(&call, 0),
                arg(&call, 1),
                arg(&call, 2),
                arg(&call, 3),
            )?;
            Ok(Atom::Undefined)
        }
        30 | 31 => {
            let id = arg(&call, 0);
            let id = if call.method.0 == 30 {
                if matches!(id, Atom::String(_)) {
                    id
                } else if let Atom::Ref(i) = id {
                    let text = a.field(i, "text");
                    if matches!(text, Atom::Undefined | Atom::Null) {
                        a.string("")?
                    } else {
                        text
                    }
                } else {
                    a.string("")?
                }
            } else if let Atom::Ref(i) = id {
                let text = a.field(i, "text");
                if matches!(text, Atom::Undefined | Atom::Null) {
                    a.string("")?
                } else {
                    let text = raw_trim(a.g, &text, false)?;
                    a.string(text)?
                }
            } else {
                a.string("")?
            };
            a.guard(root, "blockMapping", 29)?;
            let kind = a.string(if call.method.0 == 30 {
                "subgraph"
            } else {
                "connector"
            })?;
            a.block_mapping(root, id, kind, arg(&call, 2), arg(&call, 3))?;
            Ok(Atom::Undefined)
        }
        32 => {
            a.emit(
                root,
                arg(&call, 0),
                arg(&call, 1),
                arg(&call, 2),
                arg(&call, 3),
            )?;
            Ok(Atom::Undefined)
        }
        33 | 34 => {
            a.guard(root, "emit", 32)?;
            let severity = a.string(if call.method.0 == 33 {
                "warning"
            } else {
                "error"
            })?;
            a.emit(root, arg(&call, 0), severity, arg(&call, 1), arg(&call, 2))?;
            Ok(Atom::Undefined)
        }
        _ => Err(error("AgentFlow public method is unavailable").into()),
    }
}
fn create_state(
    m: &AgentflowModel,
    limit: Option<crate::compat::EdgeLimit>,
) -> Result<AgentflowPublicState, PeleError> {
    let (graph, style_bindings) = new_graph(m, limit).map_err(|e| match e {
        PublicError::Pele(e) => e,
        _ => error("AgentFlow public export failed"),
    })?;
    let projected = project_render(&graph, m, &style_bindings);
    Ok(AgentflowPublicState {
        graph,
        base: Some(m.clone()),
        constructed_cache: super::live_builder::cache(),
        style_bindings,
        projected,
    })
}
fn constructor_state(
    argument: ConstructorArgument,
    initializing: bool,
) -> Result<AgentflowPublicState, PeleError> {
    let model = AgentflowModel::empty();
    let (mut graph, _) =
        new_graph_with_options(&model, None, Some(argument)).map_err(|e| match e {
            PublicError::Pele(e) => e,
            _ => error("AgentFlow native construction failed"),
        })?;
    if initializing {
        let Atom::Ref(root) = graph.root else {
            unreachable!("native class root")
        };
        Arena::new(&mut graph)
            .assign(root, "lineOffset", Atom::Undefined)
            .map_err(|_| error("AgentFlow native initializer failed"))?;
    }
    Ok(AgentflowPublicState {
        graph,
        base: None,
        constructed_cache: super::live_builder::cache(),
        style_bindings: vec![],
        projected: Err(error("AgentFlow constructed graph has not been projected")),
    })
}
fn construct_default() -> Result<AgentflowPublicState, PeleError> {
    constructor_state(ConstructorArgument::Undefined, false)
}
fn construct_initial(argument: ConstructorArgument) -> Result<AgentflowPublicState, PeleError> {
    constructor_state(argument, true)
}
enum AgentConstruction {
    Start,
    Offset,
    Done,
}
fn resume_construction(
    state: &mut AgentflowPublicState,
    phase: &mut AgentConstruction,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    fn run(
        state: &mut AgentflowPublicState,
        phase: &mut AgentConstruction,
        reply: Option<PublicHostValue>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        let Atom::Ref(root) = state.graph.root else {
            unreachable!("native class root")
        };
        let mut arena = Arena::new(&mut state.graph);
        Ok(match std::mem::replace(phase, AgentConstruction::Done) {
            AgentConstruction::Start => {
                let options = arena.field(root, "options");
                *phase = AgentConstruction::Offset;
                PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: options,
                    key: "lineOffset",
                })
            }
            AgentConstruction::Offset => {
                let value = reply
                    .expect("awaited lineOffset")
                    .into_atom(|value| arena.string(value))?;
                let value = if matches!(value, Atom::Null | Atom::Undefined) {
                    Atom::Number(0.0)
                } else {
                    value
                };
                arena.assign(root, "lineOffset", value)?;
                PublicInvocationPhase::Complete(Ok(Atom::Undefined))
            }
            AgentConstruction::Done => {
                return Err(error("AgentFlow construction already finished").into())
            }
        })
    }
    let result = run(state, phase, reply);
    state.projected = Err(error(
        "AgentFlow constructed graph requires current projection",
    ));
    match result {
        Ok(step) => step,
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}
fn constructor_body(_: ConstructorArgument) -> DetachedPublicInvocation {
    DetachedPublicInvocation::new::<AgentFlow, AgentflowPublicState, _>(
        AgentConstruction::Start,
        resume_construction,
    )
}

impl PublicDiagramModule for AgentFlow {
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::agentflow())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        let Some(base) = state.base.as_ref() else {
            return Some(DetachedPublicRender::agentflow(
                super::AgentflowDetachedInput::constructed(
                    state.constructed_cache.clone(),
                    state.projected.as_ref().err(),
                ),
            ));
        };
        let graph = base.graph.as_ref()?;
        Some(DetachedPublicRender::agentflow(
            super::AgentflowDetachedInput::new(
                graph,
                &state.style_bindings,
                state.projected.as_ref().err(),
            ),
        ))
    }

    fn active_public_operation(
        &self,
        state: &mut Self::PublicState,
        operation: PublicActiveOperation,
        budget: &mut PublicActiveBudget,
    ) -> Result<PublicActiveValue, PublicError> {
        fn validate(g: &ModelGraph, v: &Atom) -> Result<(), PublicError> {
            g.validate_atom(v)?;
            if let Atom::BuiltinFunction(id) = v {
                if SCHEMA.method(MethodId(*id)).is_none() {
                    return Err(error("Unknown public builtin function").into());
                }
            }
            Ok(())
        }
        fn charge(b: &mut PublicActiveBudget, n: usize) -> Result<(), PublicError> {
            if n > b.remaining {
                return Err(limit().into());
            }
            b.remaining -= n;
            Ok(())
        }
        charge(budget, 1)?;
        let result = (|| {
            Ok(PublicActiveValue::Atom(match operation {
                PublicActiveOperation::Validate(v) => {
                    validate(&state.graph, &v)?;
                    Atom::Undefined
                }
                PublicActiveOperation::Intern(s) => {
                    charge(budget, s.0.len())?;
                    Arena::new(&mut state.graph).string(s)?
                }
                PublicActiveOperation::String(v) => {
                    validate(&state.graph, &v)?;
                    let Atom::String(id) = v else {
                        return Err(ty("Active model value is not a string"));
                    };
                    let s = &state.graph.strings[id as usize];
                    charge(budget, s.0.len())?;
                    return Ok(PublicActiveValue::String(s.clone()));
                }
                PublicActiveOperation::Object => Atom::Ref(Arena::new(&mut state.graph).object()?),
                PublicActiveOperation::Record(fields) => {
                    let units = fields
                        .iter()
                        .fold(fields.len(), |n, (k, _)| n.saturating_add(k.0.len()));
                    charge(budget, units)?;
                    for (_, value) in &fields {
                        validate(&state.graph, value)?;
                    }
                    let mut arena = Arena::new(&mut state.graph);
                    if arena.work.saturating_add(fields.len()) > MAX_WORK {
                        return Err(limit().into());
                    }
                    let id = arena.object()?;
                    for (key, value) in fields {
                        let key = arena.intern(key)?;
                        let properties = arena.g.nodes[id as usize].properties_mut();
                        if let Some(p) = properties.iter_mut().find(|p| p.key == key) {
                            p.value = value
                        } else {
                            properties.push(Property {
                                key,
                                value,
                                flags: 7,
                            });
                        }
                    }
                    Atom::Ref(id)
                }
                PublicActiveOperation::Read { receiver, key } => {
                    validate(&state.graph, &receiver)?;
                    charge(budget, key.len())?;
                    let id = reference(&receiver)?;
                    get(&state.graph, id, &key).clone()
                }
                PublicActiveOperation::Set {
                    receiver,
                    key,
                    value,
                } => {
                    validate(&state.graph, &receiver)?;
                    validate(&state.graph, &value)?;
                    charge(budget, key.len())?;
                    let id = reference(&receiver)?;
                    if matches!(state.graph.nodes[id as usize], Node::Array { .. }) {
                        return Err(error(
                            "Active Agentflow array structural edits are unavailable",
                        )
                        .into());
                    }
                    Arena::new(&mut state.graph).assign(id, &key, value)?;
                    Atom::Undefined
                }
                PublicActiveOperation::Builtin(call) if matches!(call.method.0, 21 | 22) => {
                    validate(&state.graph, &call.receiver)?;
                    for v in &call.arguments {
                        validate(&state.graph, v)?;
                    }
                    charge(budget, state.graph.nodes.len())?;
                    invoke(&mut state.graph, call)?
                }
                _ => return Err(error("Owned Agentflow active operation is unavailable").into()),
            }))
        })();
        if let Some(base) = &state.base {
            state.projected = project_render(&state.graph, base, &state.style_bindings);
        }
        result
    }

    fn detached_public_invocation(&self, call: MethodCall) -> Option<DetachedPublicInvocation> {
        live_options::program(call)
    }
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTOTYPE).then(|| {
            PublicConstructorFactory::first_value_body::<Self>(
                PROTOTYPE,
                construct_default,
                construct_initial,
                constructor_body,
                live_options::FIELDS,
            )
        })
    }

    type PublicState = AgentflowPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        Ok(AgentflowPublicState {
            graph,
            base: None,
            constructed_cache: super::live_builder::cache(),
            style_bindings: vec![],
            projected: Err(error("AgentFlow graph has not been built")),
        })
    }
    fn create_public(&self, m: &AgentflowModel) -> Result<Self::PublicState, PeleError> {
        create_state(m, None)
    }
    fn create_public_with_edge_limit(
        &self,
        m: &AgentflowModel,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        create_state(m, Some(limit))
    }
    fn create_public_prepared(
        &self,
        model: &AgentflowModel,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::PublicState, PeleError> {
        crate::compat::public::create_prepared_edge_public(self, model, limit)
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
        if let Some(base) = &s.base {
            s.projected = project_render(&g, base, &s.style_bindings);
        }
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke(&mut s.graph, call);
        if let Some(base) = &s.base {
            s.projected = project_render(&s.graph, base, &s.style_bindings);
        }
        result
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render_public_operation(s, c, o, r)
            .map_err(|error| match error {
                PublicError::Pele(e) => e,
                other => crate::compat::PeleError::new(
                    format!("AgentFlow render requires structured exception transport: {other:?}"),
                    ErrorCode::Semantic,
                ),
            })
    }
    fn render_public_operation(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        if s.base.is_none() {
            let (base, bindings) = fresh_render_model(&mut s.graph, c)?;
            s.projected = project_render(&s.graph, &base, &bindings);
            s.style_bindings = bindings;
            s.base = Some(base);
        }
        self.render(s.projected.as_ref().map_err(Clone::clone)?, c, o, r)
            .map_err(PublicError::from)
    }
}

// A native DB instance inherits these methods; a caller-authored plain model
// does not. An own replacement shadows the inherited native method.
fn native_db_method(g: &ModelGraph, root: u32, name: &str, method: u32) -> bool {
    let node = &g.nodes[root as usize];
    if let Some(property) = node
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(name))
    {
        return matches!(property.value, Atom::BuiltinFunction(id) if id == method);
    }
    matches!(node, Node::Instance { prototype, .. } if *prototype == PROTOTYPE)
}

// The declared AgentflowModel is data plus semanticModel(); it does not promise
// DB-only helpers. Missing helper calls must fail at graph construction, rather
// than silently inheriting a database prototype or a parser-created graph cache.
fn fresh_render_model(
    g: &mut ModelGraph,
    config: &Object,
) -> Result<(AgentflowModel, Vec<Option<u32>>), PublicError> {
    use super::model::{AgentClassDef, AgentDb, AgentEdge, AgentNode, AgentSubgraph};
    use crate::compat::diagrams::flowchart::LabelType as FlowLabel;
    fn label(v: FlowLabel) -> super::model::LabelType {
        match v {
            FlowLabel::Text => super::model::LabelType::Text,
            FlowLabel::String => super::model::LabelType::String,
            FlowLabel::Markdown => super::model::LabelType::Markdown,
        }
    }
    // The enclosing AgentFlow document already admitted this graph against its
    // own schema. Reuse field projection, not a foreign registration's schema.
    let common = crate::compat::diagrams::flowchart::import_public_graph(g)?;
    let root = reference(&g.root)?;
    let raw_nodes = entries(g, get(g, root, "nodes"))?;
    let mut styles = std::collections::HashMap::new();
    let mut nodes = vec![];
    for ((key, n), (_, raw)) in common.nodes.into_iter().zip(&raw_nodes) {
        let id = reference(raw)?;
        styles.insert(
            n.id.clone(),
            match get(g, id, "styles") {
                Atom::Ref(id) => Some(*id),
                _ => None,
            },
        );
        nodes.push((
            key,
            AgentNode {
                id: n.id,
                text: n.text,
                label_type: label(n.label_type),
                shape: n.shape,
                styles: n.styles,
                classes: n.classes,
                dir: n.dir,
                props: n.props,
                link: n.link,
                link_target: n.link_target,
                metadata: None,
                is_connector: match get(g, id, "isConnector") {
                    Atom::Bool(v) => Some(*v),
                    _ => None,
                },
            },
        ));
    }
    let connectors = entries(g, get(g, root, "connectors"))?
        .into_iter()
        .map(|(key, value)| {
            let key = text(g, &key)?;
            let id = reference(&value)?;
            let actual = text(g, get(g, id, "id"))?;
            let node = nodes
                .iter()
                .find(|(_, n)| n.id == actual)
                .map(|(_, n)| n.clone())
                .ok_or_else(|| error("AgentFlow connector is not present in nodes"))?;
            Ok((key, node))
        })
        .collect::<Result<Vec<_>, PeleError>>()?;
    let raw_groups = dense(g, get(g, root, "subgraphs"))?;
    let mut subgraphs = vec![];
    for (group, raw) in common.subgraphs.into_iter().zip(raw_groups) {
        let id = reference(&raw)?;
        subgraphs.push(AgentSubgraph {
            id: group.id,
            group_type: text(g, get(g, id, "type"))?,
            nodes: group.nodes,
            title: group.title,
            classes: group.classes,
            dir: group.dir,
            label_type: label(group.label_type),
            metadata: group.metadata,
        });
    }
    // The first collapsed walk calls the actual model's subgraph method.
    if subgraphs.iter().any(|s| {
        s.metadata.as_ref().and_then(|m| m.get("view")) == Some(&Value::String("collapsed".into()))
    }) && !native_db_method(g, root, "subgraph", 21)
    {
        return Err(ty("n.subgraph is not a function"));
    }
    let edges = common
        .edges
        .into_iter()
        .map(|e| AgentEdge {
            start: e.start,
            end: e.end,
            edge_type: e.edge_type,
            text: e.text,
            label_type: label(e.label_type),
            stroke: e.stroke,
            length: e.length,
            edge_semantic: None,
            metadata: None,
            style: e.style,
            classes: e.classes,
            id: e.id,
            is_user_defined_id: e.is_user_defined_id,
            interpolate: e.interpolate,
            animate: e.animate,
            animation: e.animation,
        })
        .collect();
    let model = AgentflowModel {
        public_origin: Default::default(),
        diagram_type: crate::compat::DiagramType::Agentflow,
        direction: common.direction,
        connectors,
        mappings: vec![],
        diagnostics: vec![],
        line_offset: 0,
        graph: None,
        nodes,
        edges,
        subgraphs,
        classes: common
            .classes
            .into_iter()
            .map(|(k, c)| {
                (
                    k,
                    AgentClassDef {
                        id: c.id,
                        styles: c.styles,
                        text_styles: c.text_styles,
                    },
                )
            })
            .collect(),
        tooltips: common.tooltips,
        default_edge_style: common.default_edge_style,
        default_interpolate: common.default_interpolate,
        title: common.title,
        acc_title: common.acc_title,
        acc_descr: common.acc_descr,
    };
    let mut db = AgentDb::from_render_model(model);
    let graph = super::graph::build(&mut db, config);
    // The graph builder selects diagnostics in native normalization order. A
    // fresh database inherits the emit helpers; a plain authored model does not.
    // Reuse the public helper body so reported deduplication, mapping aliases,
    // descriptor failures and diagnostic-before-log effects stay identical.
    if !db.model.diagnostics.is_empty() {
        let mut arena = Arena::new(g);
        for diagnostic in &db.model.diagnostics {
            let (name, method, missing) = if diagnostic.severity == "error" {
                ("emitError", 34, "t.emitError is not a function")
            } else if diagnostic.id == DiagnosticId::ContainmentViolation {
                ("emitWarning", 33, "n.emitWarning is not a function")
            } else {
                ("emitWarning", 33, "t.emitWarning is not a function")
            };
            if !native_db_method(arena.g, root, name, method) {
                return Err(ty(missing));
            }
            if !native_db_method(arena.g, root, "emit", 32) {
                return Err(ty("this.emit is not a function"));
            }
            let id = arena.string(diagnostic_id(diagnostic.id))?;
            let severity = arena.string(diagnostic.severity.as_str())?;
            let message = arena.string(diagnostic.message.as_str())?;
            let mut fields = Vec::new();
            if let Some(node) = &diagnostic.node_id {
                fields.push(("nodeId", arena.string(node.as_str())?));
            }
            if let Some(edge) = &diagnostic.edge_id {
                fields.push(("edgeId", arena.string(edge.as_str())?));
            }
            let ctx = Atom::Ref(arena.record(fields)?);
            arena.emit(root, id, severity, message, ctx)?;
        }
    }
    let connectors: std::collections::HashSet<_> = db
        .model
        .connectors
        .iter()
        .map(|(id, _)| id.as_str())
        .collect();
    if graph.nodes.iter().any(|n| {
        !n.is_group
            && n.shape != "collapsedGroup"
            && !connectors.contains(n.id.as_str())
            && db.model.nodes.iter().any(|(id, _)| id == &n.id)
    }) && !native_db_method(g, root, "vertexKind", 22)
    {
        return Err(ty("n.vertexKind is not a function"));
    }
    let bindings = graph
        .nodes
        .iter()
        .map(|n| styles.get(&n.id).copied().flatten())
        .collect();
    db.model.graph = Some(graph);
    Ok((db.model, bindings))
}
