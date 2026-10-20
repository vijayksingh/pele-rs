//! StateDb native identity graph, builder methods and cached graph, Pele 458322a5 (MIT, kepano).
use super::{
    ClickId, Description, State, StateEdge, StateGraph, StateLink, StateModel, StateNode,
    StateStmt, Stmt,
};
use crate::compat::{
    public::*, source::space, DiagramModule, DiagramType, ErrorCode, Object, PeleError,
    RenderResources, RenderSettings, Rendered, Value,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};
const MAX_ITEMS: usize = 1_000_000;
const MAX_WORK: usize = 8_000_000;
const MAX_TEXT: usize = 16_000_000;
const PROTOTYPE: PrototypeId = PrototypeId(1);
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
    id: "pele/state@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "StateModel",
        members: &[],
        kind: RecordKind::Object,
        prototype: Some(PROTOTYPE),
        fields: &[
            f("type", FieldKind::String, false),
            f("rootDoc", FieldKind::Array, false),
            f("states", FieldKind::Map, false),
            f("relations", FieldKind::Array, false),
            f("classes", FieldKind::Map, false),
            f("links", FieldKind::Map, false),
            f("graph", FieldKind::Object, false),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, true),
            f("accDescr", FieldKind::String, true),
            f("dividerCount", FieldKind::Number, false),
        ],
    }],
    prototypes: &[PrototypeSchema {
        id: PROTOTYPE,
        name: "StateDb",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "direction",
                flags: 4,
                member: MemberKind::Accessor {
                    get: Some(MethodId(1)),
                    set: None,
                },
            },
            p(2, "setRootDoc"),
            p(3, "extract"),
            p(4, "translate"),
            p(5, "addState"),
            p(6, "getState"),
            p(7, "addLink"),
            p(8, "addRelation"),
            p(9, "addDescription"),
            p(10, "getDividerId"),
            p(11, "addStyleClass"),
            p(12, "setCssClass"),
            p(13, "getDirection"),
            p(14, "setDirection"),
            p(15, "trimColon"),
            p(16, "setAccTitle"),
            p(17, "setAccDescription"),
            p(18, "named"),
        ],
    }],
    methods: &[
        m(1, "get direction", 0),
        m(2, "setRootDoc", 1),
        m(3, "extract", 1),
        m(4, "translate", 0),
        m(5, "addState", 1),
        m(6, "getState", 1),
        m(7, "addLink", 3),
        m(8, "addRelation", 2),
        m(9, "addDescription", 2),
        m(10, "getDividerId", 0),
        m(11, "addStyleClass", 1),
        m(12, "setCssClass", 2),
        m(13, "getDirection", 0),
        m(14, "setDirection", 1),
        m(15, "trimColon", 1),
        m(16, "setAccTitle", 1),
        m(17, "setAccDescription", 1),
        m(18, "named", 1),
    ],
};
pub struct StatePublicState {
    graph: ModelGraph,
    projected: Result<StateModel, PeleError>,
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("state".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("State public model expansion exceeds the work budget");
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
        Err(error("State public model requires an object reference"))
    }
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| error("State public text must be a well-formed string"))
}
fn dense(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PeleError> {
    let id = reference(a)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(error("State public collection must be an Array"));
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
        .map(|v| v.ok_or_else(|| error("Sparse State public collections are not implemented")))
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
        Err(error("State public collection must be a Map"))
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
struct Arena<'a> {
    g: &'a mut ModelGraph,
    units: usize,
    work: usize,
    string_ids: HashMap<JsString, u32>,
    canonical_strings: Vec<u32>,
    text_work: usize,
    maps: RefCell<HashMap<u32, HashMap<Key, usize>>>,
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
            return Err(ty("State collection must be a Map"));
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
        let grows = if let Node::Array { length, .. } = &self.g.nodes[id as usize] {
            key.parse::<u32>()
                .ok()
                .filter(|n| *n < u32::MAX && n.to_string() == key && *n >= *length)
                .map(|n| n + 1)
        } else {
            None
        };
        if let Some(length) = grows {
            let length_index = self.property_position(id, "length");
            if length_index
                .is_some_and(|i| self.g.nodes[id as usize].properties()[i].flags & 2 == 0)
            {
                return Err(ty("Cannot assign to a read only State array length"));
            }
            if let Node::Array {
                length: value,
                properties,
            } = &mut self.g.nodes[id as usize]
            {
                *value = length;
                if let Some(i) = length_index {
                    properties[i].value = Atom::Number(length as f64)
                }
            }
        }
        if let Some(i) = self.property_position(id, key) {
            let p = &mut self.g.nodes[id as usize].properties_mut()[i];
            if p.flags & 2 == 0 {
                return Err(ty("Cannot assign to a read only State property"));
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
        reference(a).map_err(|_| ty("State value must be an object"))
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
        if self.property_position(id, "push").is_some() {
            return Err(ty("Overridden State collection push is not implemented"));
        }
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            return Err(ty("State collection push is not a function"));
        };
        if length as usize >= MAX_ITEMS {
            return Err(limit().into());
        }
        self.assign(id, &length.to_string(), value)?;
        let Node::Array { length: len, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        *len = length + 1;
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
    fn opt(&mut self, s: Option<&String>) -> Result<Atom, PublicError> {
        s.map_or(Ok(Atom::Undefined), |s| self.string(s.as_str()))
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
        _ => return Err(error("State object numeric coercion is not implemented").into()),
    })
}
fn js_string(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    Ok(match a {
        Atom::Undefined => "undefined".into(),
        Atom::Null => "null".into(),
        Atom::Bool(v) => v.to_string(),
        Atom::Number(n) => Value::Number(*n).js_string(),
        Atom::String(_) => text(g, a)?,
        _ => return Err(error("State object string coercion is not implemented").into()),
    })
}
fn raw_trim(g: &ModelGraph, a: &Atom, left_only: bool) -> Result<JsString, PublicError> {
    let Atom::String(i) = a else {
        return Err(ty("State argument must be a string"));
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
fn arg(call: &MethodCall, n: usize) -> Atom {
    call.arguments.get(n).cloned().unwrap_or(Atom::Undefined)
}
fn native_dense(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PublicError> {
    dense(g, a).map_err(|e| {
        if e.code == ErrorCode::Limit {
            PublicError::Pele(e)
        } else {
            PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: e.message.as_str().into(),
            }
        }
    })
}
fn is(g: &ModelGraph, a: &Atom, s: &str) -> bool {
    matches!(a,Atom::String(i)if g.strings[*i as usize].is(s))
}
fn units(g: &ModelGraph, a: &Atom) -> Result<JsString, PublicError> {
    let Atom::String(i) = a else {
        return Err(ty("State argument must be a string"));
    };
    Ok(g.strings[*i as usize].clone())
}
fn trim_units(s: JsString) -> JsString {
    let b = |u: &u16| char::from_u32(*u as u32).is_some_and(space);
    let first = s.0.iter().take_while(|u| b(u)).count();
    let last = s.0.len() - s.0[first..].iter().rev().take_while(|u| b(u)).count();
    JsString(s.0[first..last].to_vec())
}
fn split(g: &ModelGraph, a: &Atom, delimiter: u16) -> Result<Vec<JsString>, PublicError> {
    Ok(units(g, a)?
        .0
        .split(|u| *u == delimiter)
        .map(|v| JsString(v.to_vec()))
        .collect())
}
fn replace(s: &JsString, from: &[u16], to: &[u16], all: bool) -> JsString {
    let mut out = Vec::new();
    let mut at = 0;
    while at < s.0.len() {
        if s.0[at..].starts_with(from) {
            out.extend_from_slice(to);
            at += from.len();
            if !all {
                out.extend_from_slice(&s.0[at..]);
                break;
            }
        } else {
            out.push(s.0[at]);
            at += 1
        }
    }
    JsString(out)
}
impl Arena<'_> {
    fn guard(&self, root: u32, name: &str, method: u32) -> Result<(), PublicError> {
        let p = self.g.nodes[root as usize]
            .properties()
            .iter()
            .find(|p| self.g.strings[p.key as usize].is(name));
        match p {
            None => Ok(()),
            Some(p) if matches!(p.value,Atom::BuiltinFunction(i)if i==method) => Ok(()),
            Some(p) if matches!(p.value, Atom::HostReference(_) | Atom::BuiltinFunction(_)) => {
                Err(error("State overridden builder callback dispatch is not implemented").into())
            }
            _ => Err(ty("State builder member is not a function")),
        }
    }
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.checked_add(n).ok_or_else(limit)?;
        if self.work > MAX_WORK {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn strings_atom(&mut self, strings: Vec<JsString>) -> Result<Atom, PublicError> {
        let v = strings
            .into_iter()
            .map(|s| self.string(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Atom::Ref(self.array(v)?))
    }
    fn map_field(&self, root: u32, key: &str, id: &Atom) -> Result<Atom, PublicError> {
        self.map_get(self.field_id(root, key)?, id)
    }
    fn direction(&self, root: u32, last: bool) -> Result<Option<Atom>, PublicError> {
        let mut result = None;
        for item in native_dense(self.g, &self.field(root, "rootDoc"))? {
            if let Atom::Ref(i) = item {
                if is(self.g, &self.field(i, "stmt"), "dir") {
                    result = Some(self.field(i, "value"));
                    if !last {
                        break;
                    }
                }
            }
        }
        Ok(result)
    }
    fn add_state(&mut self, root: u32, args: &[Atom]) -> Result<(), PublicError> {
        let arg = |i: usize| args.get(i).cloned().unwrap_or(Atom::Undefined);
        let id = raw_trim(self.g, &arg(0), false)?;
        let id = self.string(id)?;
        let kind = match arg(1) {
            Atom::Undefined => self.string("default")?,
            v => v,
        };
        let doc = arg(2);
        let descr = arg(3);
        let note = arg(4);
        let classes = arg(5);
        let states = self.field_id(root, "states")?;
        let found = self.map_get(states, &id)?;
        let state = if !truthy(self.g, &found) {
            let descriptions = self.array(vec![])?;
            let allclasses = self.array(vec![])?;
            let styles = self.array(vec![])?;
            let text_styles = self.array(vec![])?;
            let stmt = self.string("state")?;
            let s = self.record(vec![
                ("stmt", stmt),
                ("id", id.clone()),
                ("descriptions", Atom::Ref(descriptions)),
                ("type", kind.clone()),
                ("doc", doc.clone()),
                ("note", note.clone()),
                ("classes", Atom::Ref(allclasses)),
                ("styles", Atom::Ref(styles)),
                ("textStyles", Atom::Ref(text_styles)),
            ])?;
            self.map_set(states, id.clone(), Atom::Ref(s))?;
            s
        } else {
            let s = self.id(&found)?;
            if !truthy(self.g, &self.field(s, "doc")) {
                self.assign(s, "doc", doc)?
            }
            if !truthy(self.g, &self.field(s, "type")) {
                self.assign(s, "type", kind)?
            }
            s
        };
        if truthy(self.g, &descr) {
            self.guard(root, "addDescription", 9)?;
            let values = if matches!(&descr,Atom::Ref(i)if matches!(self.g.nodes[*i as usize],Node::Array{..}))
            {
                native_dense(self.g, &descr)?
            } else {
                vec![descr]
            };
            for d in values {
                let text = raw_trim(self.g, &d, false)?;
                let text = self.string(text)?;
                self.add_description(root, id.clone(), text)?
            }
        }
        if truthy(self.g, &note) {
            self.assign(state, "note", note)?
        }
        if truthy(self.g, &classes) {
            self.guard(root, "setCssClass", 12)?;
            for c in native_dense(self.g, &classes)? {
                let name = raw_trim(self.g, &c, false)?;
                let name = self.string(name)?;
                self.set_class(root, id.clone(), name)?
            }
        }
        Ok(())
    }
    fn add_description(&mut self, root: u32, id: Atom, descr: Atom) -> Result<(), PublicError> {
        let state = self.map_field(root, "states", &id)?;
        if matches!(state, Atom::Undefined | Atom::Null) {
            return Ok(());
        }
        let state = self.id(&state)?;
        let out = self.field(state, "descriptions");
        if matches!(out, Atom::Undefined | Atom::Null) {
            return Ok(());
        }
        let mut text = units(self.g, &descr)?;
        if text.0.first() == Some(&58) {
            text.0.remove(0);
            text = trim_units(text)
        }
        let text = self.string(text)?;
        let out = self.id(&out)?;
        self.push(out, text)
    }
    fn named(&mut self, root: u32, ids: Atom) -> Result<Vec<Atom>, PublicError> {
        let mut out = Vec::new();
        for id in split(self.g, &ids, 44)? {
            let raw = self.string(id.clone())?;
            let found = self.map_field(root, "states", &raw)?;
            let found = if !truthy(self.g, &found) {
                self.guard(root, "addState", 5)?;
                self.add_state(root, &[raw])?;
                let trimmed = self.string(trim_units(id))?;
                self.map_field(root, "states", &trimmed)?
            } else {
                found
            };
            if truthy(self.g, &found) {
                out.push(found)
            }
        }
        Ok(out)
    }
    fn set_class(&mut self, root: u32, ids: Atom, class: Atom) -> Result<(), PublicError> {
        self.guard(root, "named", 18)?;
        for state in self.named(root, ids)? {
            let s = self.id(&state)?;
            let list = self.field(s, "classes");
            if !matches!(list, Atom::Undefined | Atom::Null) {
                let i = self.id(&list)?;
                self.push(i, class.clone())?
            }
        }
        Ok(())
    }
    fn add_style(&mut self, root: u32, id: Atom, style: Atom) -> Result<(), PublicError> {
        let classes = self.field_id(root, "classes")?;
        let found = self.map_get(classes, &id)?;
        let found = if !truthy(self.g, &found) {
            let styles = self.array(vec![])?;
            let text = self.array(vec![])?;
            let new = self.record(vec![
                ("id", id.clone()),
                ("styles", Atom::Ref(styles)),
                ("textStyles", Atom::Ref(text)),
            ])?;
            self.map_set(classes, id, Atom::Ref(new))?;
            new
        } else {
            self.id(&found)?
        };
        if !truthy(self.g, &style) {
            return Ok(());
        }
        for attr in split(self.g, &style, 44)? {
            let fixed = trim_units(replace(&attr, &[59], &[], false));
            if attr.0.windows(5).any(|w| w == [99, 111, 108, 111, 114]) {
                let text = replace(
                    &replace(
                        &fixed,
                        &[102, 105, 108, 108],
                        &[98, 103, 70, 105, 108, 108],
                        false,
                    ),
                    &[99, 111, 108, 111, 114],
                    &[102, 105, 108, 108],
                    false,
                );
                let text = self.string(text)?;
                let list = self.field_id(found, "textStyles")?;
                self.push(list, text)?
            }
            let attr = self.string(fixed)?;
            let list = self.field_id(found, "styles")?;
            self.push(list, attr)?
        }
        Ok(())
    }
    fn add_link(
        &mut self,
        root: u32,
        id: Atom,
        url: Atom,
        tooltip: Atom,
    ) -> Result<(), PublicError> {
        let link = self.record(vec![("url", url), ("tooltip", tooltip)])?;
        let links = self.field_id(root, "links")?;
        self.map_set(links, id, Atom::Ref(link))
    }
    fn add_relation(
        &mut self,
        root: u32,
        one: Atom,
        two: Atom,
        title: Atom,
    ) -> Result<(), PublicError> {
        self.guard(root, "addState", 5)?;
        for item in [&one, &two] {
            let i = self.id(item)?;
            self.add_state(
                root,
                &[
                    self.field(i, "id"),
                    self.field(i, "type"),
                    self.field(i, "doc"),
                    self.field(i, "description"),
                    self.field(i, "note"),
                    self.field(i, "classes"),
                ],
            )?
        }
        let one = self.id(&one)?;
        let two = self.id(&two)?;
        let id1 = raw_trim(self.g, &self.field(one, "id"), false)?;
        let id2 = raw_trim(self.g, &self.field(two, "id"), false)?;
        let id1 = self.string(id1)?;
        let id2 = self.string(id2)?;
        let title = if matches!(title, Atom::Undefined) {
            self.string("")?
        } else {
            title
        };
        let relation = self.record(vec![("id1", id1), ("id2", id2), ("relationTitle", title)])?;
        let list = self.field_id(root, "relations")?;
        self.push(list, Atom::Ref(relation))
    }
    fn divider(&mut self, root: u32) -> Result<Atom, PublicError> {
        let n = number(self.g, &self.field(root, "dividerCount"))? + 1.;
        self.assign(root, "dividerCount", Atom::Number(n))?;
        self.string(format!("divider-id-{}", Value::Number(n).js_string()).as_str())
    }
    fn translate(&mut self, root: u32) -> Result<(), PublicError> {
        let root_doc = self.field(root, "rootDoc");
        let stmt = self.string("root")?;
        let id = self.string("root")?;
        let holder = self.record(vec![("stmt", stmt), ("id", id), ("doc", root_doc)])?;
        let parent = self.string("root")?;
        let mut pending = vec![(parent, Atom::Ref(holder), true, false)];
        let mut active = HashSet::new();
        while let Some((parent, item, first, leaving)) = pending.pop() {
            self.tick(1)?;
            let Atom::Ref(i) = item else { continue };
            if leaving {
                active.remove(&i);
                continue;
            }
            if !active.insert(i) {
                return Err(limit().into());
            }
            pending.push((parent.clone(), Atom::Ref(i), first, true));
            let stmt = self.field(i, "stmt");
            if is(self.g, &stmt, "relation") {
                pending.push((parent.clone(), self.field(i, "state2"), false, false));
                pending.push((parent, self.field(i, "state1"), true, false));
                continue;
            }
            if is(self.g, &stmt, "state") {
                let old = self.field(i, "id");
                let id = if is(self.g, &old, "[*]") {
                    let mut id = units(self.g, &parent)?;
                    id.0.extend(if first { "_start" } else { "_end" }.encode_utf16());
                    self.string(id)?
                } else {
                    let id = raw_trim(self.g, &old, false)?;
                    self.string(id)?
                };
                let initial = is(self.g, &old, "[*]");
                self.assign(i, "id", id)?;
                if initial {
                    self.assign(i, "start", Atom::Bool(first))?;
                }
            } else if !is(self.g, &stmt, "root") {
                continue;
            }
            let doc = self.field(i, "doc");
            if !truthy(self.g, &doc) {
                continue;
            }
            let mut current = Vec::new();
            let mut regions = Vec::new();
            for entry in native_dense(self.g, &doc)? {
                let divider = if let Atom::Ref(s) = entry {
                    is(self.g, &self.field(s, "stmt"), "state")
                        && is(self.g, &self.field(s, "type"), "divider")
                        && !truthy(self.g, &self.field(s, "doc"))
                } else {
                    false
                };
                if divider {
                    let s = self.id(&entry)?;
                    let region = self.object()?;
                    let properties = self.g.nodes[s as usize].properties().to_vec();
                    for p in properties {
                        if p.flags & 1 != 0 {
                            let key=self.g.strings[p.key as usize].well_formed().ok_or_else(||error("State raw property key spread requires the UTF16 field seam"))?;
                            self.assign(region, &key, p.value)?
                        }
                    }
                    let doc = self.array(std::mem::take(&mut current))?;
                    self.assign(region, "doc", Atom::Ref(doc))?;
                    regions.push(Atom::Ref(region))
                } else {
                    current.push(entry)
                }
            }
            if !regions.is_empty() {
                if !current.is_empty() {
                    self.guard(root, "getDividerId", 10)?;
                    let id = self.divider(root)?;
                    let doc = self.array(current)?;
                    let stmt = self.string("state")?;
                    let kind = self.string("divider")?;
                    let region = self.record(vec![
                        ("stmt", stmt),
                        ("id", id),
                        ("type", kind),
                        ("doc", Atom::Ref(doc)),
                    ])?;
                    regions.push(Atom::Ref(region))
                }
                let doc = self.array(regions)?;
                self.assign(i, "doc", Atom::Ref(doc))?
            }
            let doc = self.field(i, "doc");
            let parent = self.field(i, "id");
            for child in native_dense(self.g, &doc)?.into_iter().rev() {
                pending.push((parent.clone(), child, true, false))
            }
        }
        // The temporary root wrapper may have acquired regions; translate() mirrors
        // upstream's deliberate omission of assigning that wrapper doc to rootDoc.
        Ok(())
    }
    fn extract(&mut self, root: u32, input: Atom) -> Result<(), PublicError> {
        let states = self.map()?;
        self.assign(root, "states", Atom::Ref(states))?;
        let relations = self.array(vec![])?;
        self.assign(root, "relations", Atom::Ref(relations))?;
        let classes = self.map()?;
        self.assign(root, "classes", Atom::Ref(classes))?;
        let doc = if matches!(&input,Atom::Ref(i)if matches!(self.g.nodes[*i as usize],Node::Array{..}))
        {
            input
        } else {
            let i = self.id(&input)?;
            self.field(i, "doc")
        };
        for item in native_dense(self.g, &doc)? {
            self.tick(1)?;
            let Atom::Ref(i) = item else { continue };
            let kind = self.field(i, "stmt");
            if is(self.g, &kind, "state") {
                self.guard(root, "addState", 5)?;
                let id = raw_trim(self.g, &self.field(i, "id"), false)?;
                let id = self.string(id)?;
                self.add_state(
                    root,
                    &[
                        id,
                        self.field(i, "type"),
                        self.field(i, "doc"),
                        self.field(i, "description"),
                        self.field(i, "note"),
                    ],
                )?
            } else if is(self.g, &kind, "relation") {
                self.guard(root, "addRelation", 8)?;
                self.add_relation(
                    root,
                    self.field(i, "state1"),
                    self.field(i, "state2"),
                    self.field(i, "description"),
                )?
            } else if is(self.g, &kind, "classDef") {
                self.guard(root, "addStyleClass", 11)?;
                let id = raw_trim(self.g, &self.field(i, "id"), false)?;
                let id = self.string(id)?;
                self.add_style(root, id, self.field(i, "classes"))?
            } else if is(self.g, &kind, "style") {
                let raw = self.field(i, "styleClass");
                let styles = split(self.g, &raw, 44)?
                    .into_iter()
                    .map(|s| trim_units(replace(&s, &[59], &[], true)))
                    .collect::<Vec<_>>();
                self.guard(root, "named", 18)?;
                let id = raw_trim(self.g, &self.field(i, "id"), false)?;
                let id = self.string(id)?;
                for state in self.named(root, id)? {
                    let styles = self.strings_atom(styles.clone())?;
                    let s = self.id(&state)?;
                    self.assign(s, "styles", styles)?
                }
            } else if is(self.g, &kind, "applyClass") {
                self.guard(root, "setCssClass", 12)?;
                let id = raw_trim(self.g, &self.field(i, "id"), false)?;
                let id = self.string(id)?;
                self.set_class(root, id, self.field(i, "styleClass"))?
            } else if is(self.g, &kind, "click") {
                self.guard(root, "addLink", 7)?;
                let id = self.field(i, "id");
                let id = if matches!(id, Atom::String(_)) {
                    id
                } else {
                    let i = self.id(&id)?;
                    self.field(i, "id")
                };
                self.add_link(root, id, self.field(i, "url"), self.field(i, "tooltip"))?
            }
        }
        let graph = self.build_graph(root, doc)?;
        self.assign(root, "graph", Atom::Ref(graph))?;
        Ok(())
    }
}
// Graph construction is private until extract() publishes its result. Keep
// superseded draft arrays here, rather than retaining unreachable arena nodes.
// Only final nodes become public objects; aliases into public styles stay live.
enum FullDescription {
    Text(Atom),
    Lines(Vec<Atom>),
}
struct Draft {
    shape: Atom,
    description: FullDescription,
    css_classes: Atom,
    css_styles: Atom,
    kind: Atom,
    dir: Atom,
    color: Atom,
    unstyled: bool,
}
enum DrawValue {
    Atom(Atom),
    Array(Vec<Atom>),
    Description(JsString),
    Compiled(std::sync::Arc<Vec<Atom>>),
}
struct DrawNode {
    fields: Vec<(&'static str, DrawValue)>,
}
impl DrawNode {
    fn get(&self, key: &str) -> Atom {
        self.fields
            .iter()
            .find(|(k, _)| *k == key)
            .map_or(Atom::Undefined, |(_, v)| match v {
                DrawValue::Atom(v) => v.clone(),
                _ => Atom::Undefined,
            })
    }
    fn set(&mut self, key: &'static str, value: DrawValue) {
        if let Some((_, v)) = self.fields.iter_mut().find(|(k, _)| *k == key) {
            *v = value
        } else {
            self.fields.push((key, value))
        }
    }
}
#[derive(Default)]
struct Drawing {
    nodes: Vec<DrawNode>,
    edges: Vec<Atom>,
    by_id: HashMap<JsString, usize>,
    drafts: HashMap<JsString, Draft>,
    compiled: HashMap<JsString, std::sync::Arc<Vec<Atom>>>,
    slots: HashMap<JsString, Atom>,
    counter: usize,
    next_slot: usize,
}
impl Arena<'_> {
    fn joined(&mut self, values: &[Atom], separator: &str) -> Result<Atom, PublicError> {
        let mut out = Vec::new();
        for (i, v) in values.iter().enumerate() {
            if i > 0 {
                out.extend(separator.encode_utf16())
            }
            if !matches!(v, Atom::Undefined | Atom::Null) {
                out.extend(units(self.g, v)?.0)
            }
            if out.len() > MAX_TEXT {
                return Err(limit().into());
            }
        }
        self.string(JsString(out))
    }
    fn append_text(&mut self, base: Atom, suffix: &str) -> Result<Atom, PublicError> {
        let mut out = units(self.g, &base)?;
        out.0.extend(suffix.encode_utf16());
        self.string(out)
    }
    fn insert_graph(
        &mut self,
        root: u32,
        d: &mut Drawing,
        mut node: DrawNode,
    ) -> Result<(), PublicError> {
        let id = units(self.g, &node.get("id"))?;
        if id.0.is_empty() || id.is("</join></fork>") || id.is("</choice>") {
            return Ok(());
        }
        let names = node.get("cssClasses");
        if truthy(self.g, &names) {
            let key = units(self.g, &names)?;
            let compiled = if let Some(v) = d.compiled.get(&key) {
                v.clone()
            } else {
                let mut compiled = Vec::new();
                for name in split(self.g, &names, 32)? {
                    let name = self.string(name)?;
                    let def = self.map_field(root, "classes", &name)?;
                    if !truthy(self.g, &def) {
                        continue;
                    }
                    let def = self.id(&def)?;
                    let styles = native_dense(self.g, &self.field(def, "styles"))?;
                    self.tick(styles.len())?;
                    compiled.extend(styles);
                    if compiled.len() > MAX_ITEMS {
                        return Err(limit().into());
                    }
                }
                let compiled = std::sync::Arc::new(compiled);
                d.compiled.insert(key, compiled.clone());
                compiled
            };
            node.set("cssCompiledStyles", DrawValue::Compiled(compiled));
        }
        if let Some(&old) = d.by_id.get(&id) {
            let parent = node.get("parentId");
            let parent = if matches!(parent, Atom::Undefined | Atom::Null) {
                d.nodes[old].get("parentId")
            } else {
                parent
            };
            for (k, v) in node.fields {
                d.nodes[old].set(k, v)
            }
            if !matches!(parent, Atom::Undefined) {
                d.nodes[old].set("parentId", DrawValue::Atom(parent))
            }
        } else {
            d.by_id.insert(id, d.nodes.len());
            d.nodes.push(node)
        }
        Ok(())
    }
    fn graph_visit(
        &mut self,
        root: u32,
        d: &mut Drawing,
        parent: Option<u32>,
        item: u32,
        alt: bool,
    ) -> Result<(), PublicError> {
        self.tick(1)?;
        let id = self.field(item, "id");
        if is(self.g, &id, "root") {
            return Ok(());
        }
        let key = units(self.g, &id)?;
        let mut draft = if let Some(v) = d.drafts.remove(&key) {
            v
        } else {
            let known = self.map_field(root, "states", &id)?;
            let (class_str, styles) = if let Atom::Ref(s) = known {
                let classes = self.field(s, "classes");
                let values = if matches!(classes, Atom::Undefined | Atom::Null) {
                    vec![]
                } else {
                    native_dense(self.g, &classes)?
                };
                let class_str = self.joined(&values, " ")?;
                let styles = self.field(s, "styles");
                let styles = if matches!(styles, Atom::Undefined | Atom::Null) {
                    Atom::Ref(self.array(vec![])?)
                } else {
                    styles
                };
                (class_str, styles)
            } else {
                (self.string("")?, Atom::Ref(self.array(vec![])?))
            };
            let unstyled = trim_units(units(self.g, &class_str)?).0.is_empty()
                && native_dense(self.g, &styles)?.is_empty();
            let kind = self.field(item, "type");
            let shape = if !is(self.g, &kind, "default") {
                kind
            } else {
                self.string(match self.field(item, "start") {
                    Atom::Bool(true) => "stateStart",
                    Atom::Bool(false) => "stateEnd",
                    _ => "rect",
                })?
            };
            Draft {
                shape,
                description: FullDescription::Text(id.clone()),
                css_classes: self.append_text(class_str, " statediagram-state")?,
                css_styles: styles,
                kind: Atom::Undefined,
                dir: Atom::Undefined,
                color: Atom::Undefined,
                unstyled,
            }
        };
        let descr = self.field(item, "description");
        if truthy(self.g, &descr) {
            let additions = if matches!(&descr,Atom::Ref(i)if matches!(self.g.nodes[*i as usize],Node::Array{..}))
            {
                native_dense(self.g, &descr)?
            } else {
                vec![descr.clone()]
            };
            match &mut draft.description {
                FullDescription::Lines(values) => {
                    draft.shape = self.string("rectWithTitle")?;
                    values.extend(additions)
                }
                FullDescription::Text(old) => {
                    if !units(self.g, old)?.0.is_empty() {
                        draft.shape = self.string("rectWithTitle")?;
                        let mut values = if same(self.g, old, &id) {
                            vec![]
                        } else {
                            vec![old.clone()]
                        };
                        values.extend(additions);
                        draft.description = FullDescription::Lines(values)
                    } else {
                        draft.shape = self.string("rect")?;
                        draft.description = if matches!(&descr,Atom::Ref(i)if matches!(self.g.nodes[*i as usize],Node::Array{..}))
                        {
                            FullDescription::Lines(additions)
                        } else {
                            FullDescription::Text(descr)
                        }
                    }
                }
            }
        }
        let length = match &draft.description {
            FullDescription::Text(a) => units(self.g, a)?.0.len(),
            FullDescription::Lines(v) => v.len(),
        };
        if length > MAX_ITEMS {
            return Err(limit().into());
        }
        if length == 1 && is(self.g, &draft.shape, "rectWithTitle") {
            draft.shape = self.string(if is(self.g, &draft.kind, "group") {
                "roundedWithTitle"
            } else {
                "rect"
            })?
        }
        let doc = self.field(item, "doc");
        if !truthy(self.g, &draft.kind) && truthy(self.g, &doc) {
            draft.kind = self.string("group")?;
            let mut dir = self.string("TB")?;
            for stmt in native_dense(self.g, &doc)? {
                if let Atom::Ref(i) = stmt {
                    if is(self.g, &self.field(i, "stmt"), "dir") {
                        dir = self.field(i, "value")
                    }
                }
            }
            draft.dir = dir;
            draft.shape = self.string(if is(self.g, &self.field(item, "type"), "divider") {
                "divider"
            } else {
                "roundedWithTitle"
            })?;
            let parent_slot = parent
                .and_then(|p| units(self.g, &self.field(p, "id")).ok())
                .and_then(|key| d.slots.get(&key).cloned());
            if is(self.g, &draft.shape, "divider") && parent_slot.is_some() {
                draft.color = parent_slot.unwrap()
            } else {
                let slot = d.next_slot;
                d.next_slot += 1;
                draft.color = if draft.unstyled {
                    Atom::Number(slot as f64)
                } else {
                    Atom::Undefined
                }
            }
            d.slots.insert(key.clone(), draft.color.clone());
            draft.css_classes = self.append_text(
                draft.css_classes.clone(),
                if alt {
                    " statediagram-cluster statediagram-cluster-alt"
                } else {
                    " statediagram-cluster "
                },
            )?
        }
        let label = match &draft.description {
            FullDescription::Text(v) => v.clone(),
            FullDescription::Lines(v) => v.first().cloned().unwrap_or(Atom::Undefined),
        };
        let av = DrawValue::Atom;
        let mut node = DrawNode {
            fields: vec![
                ("id", av(id.clone())),
                ("shape", av(draft.shape.clone())),
                ("label", av(label)),
                ("description", DrawValue::Description(key.clone())),
                ("cssClasses", av(draft.css_classes.clone())),
                ("cssCompiledStyles", DrawValue::Array(vec![])),
                ("cssStyles", av(draft.css_styles.clone())),
                ("type", av(draft.kind.clone())),
                ("isGroup", av(Atom::Bool(is(self.g, &draft.kind, "group")))),
                ("dir", av(draft.dir.clone())),
                ("colorIndex", av(draft.color.clone())),
            ],
        };
        if is(self.g, &draft.shape, "divider") {
            node.set("label", av(self.string("")?))
        }
        if let Some(parent) = parent {
            let parent = self.field(parent, "id");
            if !is(self.g, &parent, "root") {
                node.set("parentId", av(parent))
            }
        }
        let note = self.field(item, "note");
        if !truthy(self.g, &note) {
            self.insert_graph(root, d, node)?;
            d.drafts.insert(key, draft);
            return Ok(());
        }
        let note = self.id(&note)?;
        let note_id = self.append_text(id.clone(), &format!("----note-{}", d.counter))?;
        d.counter += 1;
        let parent_id = self.append_text(id.clone(), "----parent")?;
        let aside = |a: &mut Arena<'_>,
                     id: Atom,
                     shape: &str,
                     classes: Atom,
                     group: bool|
         -> Result<DrawNode, PublicError> {
            let shape = a.string(shape)?;
            let kind = a.string(if group { "group" } else { "node" })?;
            Ok(DrawNode {
                fields: vec![
                    ("id", av(id)),
                    ("shape", av(shape)),
                    ("label", av(a.field(note, "text"))),
                    ("description", DrawValue::Array(vec![])),
                    ("cssClasses", av(classes)),
                    ("cssCompiledStyles", DrawValue::Array(vec![])),
                    ("cssStyles", DrawValue::Array(vec![])),
                    ("type", av(kind)),
                    ("isGroup", av(Atom::Bool(group))),
                    ("dir", av(Atom::Undefined)),
                    ("position", av(a.field(note, "position"))),
                ],
            })
        };
        let note_classes = self.string("statediagram-note")?;
        let mut note_node = aside(self, note_id.clone(), "note", note_classes, false)?;
        note_node.set("parentId", av(parent_id.clone()));
        note_node.set("noteFor", av(id.clone()));
        let group = aside(
            self,
            parent_id,
            "noteGroup",
            draft.css_classes.clone(),
            true,
        )?;
        self.insert_graph(root, d, group)?;
        self.insert_graph(root, d, note_node)?;
        self.insert_graph(root, d, node)?;
        let left = is(self.g, &self.field(note, "position"), "left of");
        let (from, to) = if left {
            (note_id, id.clone())
        } else {
            (id.clone(), note_id)
        };
        let mut edge_id = units(self.g, &from)?;
        edge_id.0.push(45);
        edge_id.0.extend(units(self.g, &to)?.0);
        let edge_id = self.string(edge_id)?;
        let label = self.string("")?;
        let classes = self.string("transition note-edge")?;
        let pattern = self.string("dashed")?;
        let edge = self.record(vec![
            ("id", edge_id),
            ("start", from),
            ("end", to),
            ("label", label),
            ("classes", classes),
            ("pattern", pattern),
        ])?;
        d.edges.push(Atom::Ref(edge));
        d.drafts.insert(key, draft);
        Ok(())
    }
    fn build_graph(&mut self, root: u32, doc: Atom) -> Result<u32, PublicError> {
        let mut d = Drawing::default();
        struct Frame {
            parent: Option<u32>,
            id: u32,
            values: Vec<Atom>,
            at: usize,
            alt: bool,
        }
        let mut pending = vec![Frame {
            parent: None,
            id: self.id(&doc)?,
            values: native_dense(self.g, &doc)?,
            at: 0,
            alt: true,
        }];
        let mut active = HashSet::from([self.id(&doc)?]);
        while let Some(frame) = pending.last_mut() {
            if frame.at >= frame.values.len() {
                let id = frame.id;
                pending.pop();
                active.remove(&id);
                continue;
            }
            let entry = frame.values[frame.at].clone();
            frame.at += 1;
            let parent = frame.parent;
            let alt = frame.alt;
            self.tick(1)?;
            let Atom::Ref(item) = entry else { continue };
            let stmt = self.field(item, "stmt");
            if is(self.g, &stmt, "state") {
                self.graph_visit(root, &mut d, parent, item, alt)?;
                let child = self.field(item, "doc");
                if truthy(self.g, &child) {
                    let id = self.id(&child)?;
                    if !active.insert(id) {
                        return Err(limit().into());
                    }
                    let values = native_dense(self.g, &child)?;
                    pending.push(Frame {
                        parent: Some(item),
                        id,
                        values,
                        at: 0,
                        alt: !alt,
                    })
                }
            } else if is(self.g, &stmt, "relation") {
                let one = self.id(&self.field(item, "state1"))?;
                let two = self.id(&self.field(item, "state2"))?;
                self.graph_visit(root, &mut d, parent, one, alt)?;
                self.graph_visit(root, &mut d, parent, two, alt)?;
                let id = self.string(format!("edge{}", d.counter).as_str())?;
                d.counter += 1;
                let label = self.field(item, "description");
                let label = if matches!(label, Atom::Undefined | Atom::Null) {
                    self.string("")?
                } else {
                    label
                };
                let classes = self.string("transition")?;
                let edge = self.record(vec![
                    ("id", id),
                    ("start", self.field(one, "id")),
                    ("end", self.field(two, "id")),
                    ("label", label),
                    ("classes", classes),
                ])?;
                d.edges.push(Atom::Ref(edge))
            }
        }
        let mut nodes = Vec::new();
        for node in d.nodes {
            let group = truthy(self.g, &node.get("isGroup"));
            let has_description = match node
                .fields
                .iter()
                .find(|(k, _)| *k == "description")
                .map(|(_, v)| v)
            {
                Some(DrawValue::Description(key)) => {
                    matches!(&d.drafts[key].description,FullDescription::Lines(v)if v.len()>1)
                }
                Some(DrawValue::Array(v)) => !v.is_empty(),
                _ => false,
            };
            if group && has_description {
                return Err(error(format!("Group nodes can only have label. Remove the additional description for node [{}]",js_string(self.g,&node.get("id"))?)).into());
            }
            let id = self.object()?;
            for (k, v) in node.fields {
                let value = match v {
                    DrawValue::Atom(v) => v,
                    DrawValue::Array(values) => Atom::Ref(self.array(values)?),
                    DrawValue::Compiled(values) => Atom::Ref(self.array(values.as_ref().clone())?),
                    DrawValue::Description(key) => {
                        let values = match &d.drafts[&key].description {
                            FullDescription::Lines(v) if v.len() > 1 => v[1..].to_vec(),
                            _ => vec![],
                        };
                        Atom::Ref(self.array(values)?)
                    }
                };
                self.assign(id, k, value)?
            }
            nodes.push(Atom::Ref(id))
        }
        let nodes = self.array(nodes)?;
        let edges = self.array(d.edges)?;
        self.record(vec![
            ("nodes", Atom::Ref(nodes)),
            ("edges", Atom::Ref(edges)),
        ])
    }
}
fn source_doc(a: &mut Arena<'_>, input: &[Stmt]) -> Result<Atom, PublicError> {
    fn state<'a>(
        a: &mut Arena<'_>,
        s: &'a StateStmt,
        pending: &mut Vec<(&'a [Stmt], u32)>,
    ) -> Result<Atom, PublicError> {
        let stmt = a.string("state")?;
        let id = a.string(s.id.as_str())?;
        let i = a.record(vec![("stmt", stmt), ("id", id)])?;
        if let Some(classes) = &s.classes {
            let list = a.strings(classes)?;
            a.assign(i, "classes", Atom::Ref(list))?
        }
        if let Some(kind) = &s.state_type {
            let kind = a.string(kind.as_str())?;
            a.assign(i, "type", kind)?
        }
        if let Some(description) = &s.description {
            let text = match description {
                Description::Text(s) => a.string(s.as_str())?,
                Description::Lines(v) => Atom::Ref(a.strings(v)?),
            };
            a.assign(i, "description", text)?
        }
        if let Some(doc) = &s.doc {
            let doc_id = a.array(vec![])?;
            a.assign(i, "doc", Atom::Ref(doc_id))?;
            pending.push((doc, doc_id))
        }
        if let Some(note) = &s.note {
            let position = a.opt(note.position.as_ref())?;
            let text = a.string(note.text.as_str())?;
            let note = a.record(vec![("position", position), ("text", text)])?;
            a.assign(i, "note", Atom::Ref(note))?
        }
        if let Some(start) = s.start {
            a.assign(i, "start", Atom::Bool(start))?
        }
        Ok(Atom::Ref(i))
    }
    let root = a.array(vec![])?;
    let mut pending = vec![(input, root)];
    while let Some((doc, id)) = pending.pop() {
        for item in doc {
            a.tick(1)?;
            let value = match item {
                Stmt::Text(s) => a.string(s.as_str())?,
                Stmt::State(s) => state(a, s, &mut pending)?,
                Stmt::Relation(r) => {
                    let one = state(a, &r.state1, &mut pending)?;
                    let two = state(a, &r.state2, &mut pending)?;
                    let stmt = a.string("relation")?;
                    let i = a.record(vec![("stmt", stmt), ("state1", one), ("state2", two)])?;
                    if let Some(description) = &r.description {
                        let text = a.string(description.as_str())?;
                        a.assign(i, "description", text)?
                    }
                    Atom::Ref(i)
                }
                Stmt::ClassDef(c) => {
                    let stmt = a.string("classDef")?;
                    let id = a.string(c.id.as_str())?;
                    let classes = a.string(c.classes.as_str())?;
                    Atom::Ref(a.record(vec![("stmt", stmt), ("id", id), ("classes", classes)])?)
                }
                Stmt::Style(s) => {
                    let stmt = a.string(s.stmt)?;
                    let id = a.string(s.id.as_str())?;
                    let class = a.string(s.style_class.as_str())?;
                    Atom::Ref(a.record(vec![("stmt", stmt), ("id", id), ("styleClass", class)])?)
                }
                Stmt::Direction(s) => {
                    let stmt = a.string("dir")?;
                    let value = a.string(s.value.as_str())?;
                    Atom::Ref(a.record(vec![("stmt", stmt), ("value", value)])?)
                }
                Stmt::Click(c) => {
                    let id = match &c.id {
                        ClickId::String(s) => a.string(s.as_str())?,
                        ClickId::State(s) => state(a, s, &mut pending)?,
                    };
                    let stmt = a.string("click")?;
                    let url = a.string(c.url.as_str())?;
                    let tooltip = a.string(c.tooltip.as_str())?;
                    Atom::Ref(a.record(vec![
                        ("stmt", stmt),
                        ("id", id),
                        ("url", url),
                        ("tooltip", tooltip),
                    ])?)
                }
            };
            a.push(id, value)?
        }
    }
    Ok(Atom::Ref(root))
}
fn empty_graph() -> ModelGraph {
    ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec![],
        nodes: vec![],
    }
}
// Shared field-initializer boundary. Parsed models perform extraction afterward;
// a constructed StateDb exposes these fresh Maps/arrays and empty drawing as-is.
fn initial_root(
    a: &mut Arena<'_>,
    root_doc: &[Stmt],
    title: Option<&String>,
    acc_title: Option<&String>,
    acc_descr: Option<&String>,
    divider_count: usize,
) -> Result<(u32, Atom), PublicError> {
    let root = a.node(Node::Instance {
        prototype: PROTOTYPE,
        properties: vec![],
    })?;
    a.g.root = Atom::Ref(root);
    let kind = a.string("state")?;
    let doc = source_doc(a, root_doc)?;
    let states = a.map()?;
    let relations = a.array(vec![])?;
    let classes = a.map()?;
    let links = a.map()?;
    let nodes = a.array(vec![])?;
    let edges = a.array(vec![])?;
    let drawing = a.record(vec![
        ("nodes", Atom::Ref(nodes)),
        ("edges", Atom::Ref(edges)),
    ])?;
    let fields = vec![
        ("type", kind),
        ("rootDoc", doc.clone()),
        ("states", Atom::Ref(states)),
        ("relations", Atom::Ref(relations)),
        ("classes", Atom::Ref(classes)),
        ("links", Atom::Ref(links)),
        ("graph", Atom::Ref(drawing)),
        ("title", a.opt(title)?),
        ("accTitle", a.opt(acc_title)?),
        ("accDescr", a.opt(acc_descr)?),
        ("dividerCount", Atom::Number(divider_count as f64)),
    ];
    for (k, v) in fields {
        a.assign(root, k, v)?
    }
    Ok((root, doc))
}
fn new_graph(model: &StateModel) -> Result<ModelGraph, PublicError> {
    let mut g = empty_graph();
    let mut a = Arena::new(&mut g);
    let (root, doc) = initial_root(
        &mut a,
        &model.root_doc,
        model.title.as_ref(),
        model.acc_title.as_ref(),
        model.acc_descr.as_ref(),
        model.public_divider_count,
    )?;
    a.extract(root, doc)?;
    Ok(g)
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let mut a = Arena::new(g);
    let root = a.id(&call.receiver)?;
    match call.method.0 {
        1 | 13 => {
            let value = a.direction(root, call.method.0 == 1)?;
            value.map_or_else(|| a.string("TB"), Ok)
        }
        2 => {
            let doc = arg(&call, 0);
            a.assign(root, "rootDoc", doc.clone())?;
            a.guard(root, "translate", 4)?;
            a.translate(root)?;
            a.guard(root, "extract", 3)?;
            a.extract(root, doc)?;
            Ok(Atom::Undefined)
        }
        3 => {
            a.extract(root, arg(&call, 0))?;
            Ok(Atom::Undefined)
        }
        4 => {
            a.translate(root)?;
            Ok(Atom::Undefined)
        }
        5 => {
            a.add_state(root, &call.arguments)?;
            Ok(Atom::Undefined)
        }
        6 => a.map_field(root, "states", &arg(&call, 0)),
        7 => {
            a.add_link(root, arg(&call, 0), arg(&call, 1), arg(&call, 2))?;
            Ok(Atom::Undefined)
        }
        8 => {
            a.add_relation(root, arg(&call, 0), arg(&call, 1), arg(&call, 2))?;
            Ok(Atom::Undefined)
        }
        9 => {
            a.add_description(root, arg(&call, 0), arg(&call, 1))?;
            Ok(Atom::Undefined)
        }
        10 => a.divider(root),
        11 => {
            a.add_style(root, arg(&call, 0), arg(&call, 1))?;
            Ok(Atom::Undefined)
        }
        12 => {
            a.set_class(root, arg(&call, 0), arg(&call, 1))?;
            Ok(Atom::Undefined)
        }
        14 => {
            let doc = a.field(root, "rootDoc");
            let mut found = None;
            for item in native_dense(a.g, &doc)? {
                if let Atom::Ref(i) = item {
                    if is(a.g, &a.field(i, "stmt"), "dir") {
                        found = Some(i);
                        break;
                    }
                }
            }
            if let Some(i) = found {
                a.assign(i, "value", arg(&call, 0))?
            } else {
                let kind = a.string("dir")?;
                let item = a.record(vec![("stmt", kind), ("value", arg(&call, 0))])?;
                let mut values = native_dense(a.g, &doc)?;
                values.insert(0, Atom::Ref(item));
                let i = a.id(&doc)?;
                if a.g.nodes[i as usize]
                    .properties()
                    .iter()
                    .any(|p| a.g.strings[p.key as usize].is("unshift"))
                {
                    return Err(error(
                        "State overridden array unshift dispatch is not implemented",
                    )
                    .into());
                }
                for (n, v) in values.into_iter().enumerate().rev() {
                    a.assign(i, &n.to_string(), v)?;
                }
            }
            Ok(Atom::Undefined)
        }
        15 => {
            let mut text = units(a.g, &arg(&call, 0))?;
            if text.0.first() == Some(&58) {
                text.0.remove(0);
            }
            a.string(trim_units(text))
        }
        16 => {
            let text = raw_trim(a.g, &arg(&call, 0), true)?;
            let text = a.string(text)?;
            a.assign(root, "accTitle", text)?;
            Ok(Atom::Undefined)
        }
        17 => {
            let text = units(a.g, &arg(&call, 0))?;
            let mut out = Vec::new();
            let mut after = false;
            for c in text.0 {
                if after && char::from_u32(c as u32).is_some_and(space) {
                    continue;
                }
                out.push(c);
                after = c == 10
            }
            let text = a.string(JsString(out))?;
            a.assign(root, "accDescr", text)?;
            Ok(Atom::Undefined)
        }
        18 => {
            let values = a.named(root, arg(&call, 0))?;
            Ok(Atom::Ref(a.array(values)?))
        }
        _ => Err(error("State native method is unavailable").into()),
    }
}
fn project(g: &ModelGraph) -> Result<StateModel, PeleError> {
    struct Import<'a> {
        g: &'a ModelGraph,
        units: std::cell::Cell<usize>,
        work: std::cell::Cell<usize>,
    }
    impl Import<'_> {
        fn text(&self, a: &Atom) -> Result<String, PeleError> {
            let s = self.g.text(a)?;
            let total = self.units.get().checked_add(s.0.len()).ok_or_else(limit)?;
            if total > MAX_TEXT {
                return Err(limit());
            }
            self.units.set(total);
            s.well_formed().ok_or_else(|| {
                error("State rendering of lone UTF16 surrogates requires well-formed text")
            })
        }
        fn opt(&self, a: &Atom) -> Result<Option<String>, PeleError> {
            if matches!(a, Atom::Undefined) {
                Ok(None)
            } else {
                self.text(a).map(Some)
            }
        }
        fn strings(&self, a: &Atom) -> Result<Vec<String>, PeleError> {
            let values = dense(self.g, a)?;
            let work = self
                .work
                .get()
                .checked_add(values.len())
                .ok_or_else(limit)?;
            if work > MAX_WORK {
                return Err(limit());
            }
            self.work.set(work);
            values.iter().map(|a| self.text(a)).collect()
        }
    }
    let b = Import {
        g,
        units: std::cell::Cell::new(0),
        work: std::cell::Cell::new(0),
    };
    let root = reference(&g.root)?;
    let direction = if g.nodes[root as usize]
        .properties()
        .iter()
        .any(|p| g.strings[p.key as usize].is("direction"))
    {
        b.text(get(g, root, "direction"))?
    } else {
        let mut value = None;
        for item in dense(g, get(g, root, "rootDoc"))? {
            if let Atom::Ref(i) = item {
                if is(g, get(g, i, "stmt"), "dir") {
                    value = Some(get(g, i, "value"));
                }
            }
        }
        match value {
            None | Some(Atom::Undefined) => "TB".into(),
            Some(v) => b.text(v)?,
        }
    };
    let graph = reference(get(g, root, "graph"))?;
    let mut nodes = Vec::new();
    for n in dense(g, get(g, graph, "nodes"))? {
        let i = reference(&n)?;
        let color = get(g, i, "colorIndex");
        let color_index = match color {
            Atom::Undefined => None,
            Atom::Number(n)
                if n.is_finite() && *n >= 0. && n.fract() == 0. && *n <= usize::MAX as f64 =>
            {
                Some(*n as usize)
            }
            _ => {
                return Err(error(
                    "State renderer colorIndex requires a nonnegative integer",
                ))
            }
        };
        nodes.push(StateNode {
            id: b.text(get(g, i, "id"))?,
            shape: b.opt(get(g, i, "shape"))?,
            label: b.opt(get(g, i, "label"))?,
            description: b.strings(get(g, i, "description"))?,
            css_classes: b.text(get(g, i, "cssClasses"))?,
            css_compiled_styles: b.strings(get(g, i, "cssCompiledStyles"))?,
            css_styles: b.strings(get(g, i, "cssStyles"))?,
            node_type: b.opt(get(g, i, "type"))?,
            is_group: truthy(g, get(g, i, "isGroup")),
            parent_id: b.opt(get(g, i, "parentId"))?,
            dir: b.opt(get(g, i, "dir"))?,
            position: b.opt(get(g, i, "position"))?,
            note_for: b.opt(get(g, i, "noteFor"))?,
            color_index,
        });
    }
    let mut edges = Vec::new();
    for e in dense(g, get(g, graph, "edges"))? {
        let i = reference(&e)?;
        edges.push(StateEdge {
            id: b.text(get(g, i, "id"))?,
            start: b.text(get(g, i, "start"))?,
            end: b.text(get(g, i, "end"))?,
            label: b.text(get(g, i, "label"))?,
            classes: b.text(get(g, i, "classes"))?,
            pattern: b.opt(get(g, i, "pattern"))?,
        })
    }
    let visible: HashSet<JsString> = nodes
        .iter()
        .filter(|n| !matches!(n.shape.as_deref(), Some("noteGroup" | "divider")))
        .map(|n| n.id.as_str().into())
        .collect();
    let mut links = Vec::new();
    for (k, v) in entries(g, get(g, root, "links"))? {
        if !matches!(&k,Atom::String(i)if visible.contains(&g.strings[*i as usize]))
            || !truthy(g, &v)
        {
            continue;
        }
        let i = reference(&v)?;
        links.push((
            b.text(&k)?,
            StateLink {
                url: b.text(get(g, i, "url"))?,
                tooltip: b.text(get(g, i, "tooltip"))?,
            },
        ))
    }
    Ok(StateModel {
        public_divider_count: 0,
        diagram_type: DiagramType::State,
        direction,
        root_doc: vec![],
        states: vec![],
        relations: vec![],
        classes: vec![],
        links,
        graph: StateGraph { nodes, edges },
        title: b.opt(get(g, root, "title"))?,
        acc_title: b.opt(get(g, root, "accTitle"))?,
        acc_descr: b.opt(get(g, root, "accDescr"))?,
    })
}
fn public_to_pele(e: PublicError) -> PeleError {
    match e {
        PublicError::Pele(e) => e,
        PublicError::JavaScript { message, .. } => error(
            message
                .well_formed()
                .unwrap_or_else(|| "State native TypeError".into()),
        ),
        _ => error("State native host exception"),
    }
}
fn construct_default() -> Result<StatePublicState, PeleError> {
    // No ParseContext, source, translation or extract call: StateDb ignores
    // constructor arguments and its drawing cache starts independently empty.
    let mut graph = empty_graph();
    initial_root(&mut Arena::new(&mut graph), &[], None, None, None, 0).map_err(public_to_pele)?;
    let projected = project(&graph);
    Ok(StatePublicState { graph, projected })
}

impl PublicDiagramModule for State {
    type PublicState = StatePublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTOTYPE).then(|| {
            PublicConstructorFactory::ignoring_arguments::<Self>(PROTOTYPE, construct_default)
        })
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let projected = project(&graph);
        Ok(StatePublicState { graph, projected })
    }
    fn create_public(&self, m: &StateModel) -> Result<Self::PublicState, PeleError> {
        let graph = new_graph(m).map_err(public_to_pele)?;
        let projected = project(&graph);
        Ok(StatePublicState { graph, projected })
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
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let out = invoke(&mut s.graph, c);
        s.projected = project(&s.graph);
        out
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
    fn render_public_operation(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        let root = reference(&state.graph.root)?;
        if matches!(state.graph.nodes[root as usize], Node::Object { .. })
            && matches!(get(&state.graph, root, "graph"), Atom::Undefined)
        {
            return Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: "Cannot read properties of undefined (reading 'nodes')".into(),
            });
        }
        self.render_public(state, config, options, host)
            .map_err(Into::into)
    }
}
