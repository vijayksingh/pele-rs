//! SeqDb native mutable graph and real builder methods, Pele 458322a5 (MIT, kepano).
use super::model::SequencePublicOrigin;
use super::{MessageText, SeqActor, SeqBox, SeqMessage, SeqNumbering, Sequence, SequenceModel};
use crate::compat::{
    public::*,
    source::{space, yaml},
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered, Value,
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
    id: "pele/sequence@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "SequenceModel",
        kind: RecordKind::Object,
        prototype: Some(PROTOTYPE),
        members: &[],
        fields: &[
            f("type", FieldKind::String, false),
            f("actors", FieldKind::Map, false),
            f("createdActors", FieldKind::Map, false),
            f("destroyedActors", FieldKind::Map, false),
            f("boxes", FieldKind::Array, false),
            f("messages", FieldKind::Array, false),
            f("notes", FieldKind::Array, false),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, true),
            f("accDescr", FieldKind::String, true),
            f("prevActor", FieldKind::String, true),
            f("currentBox", FieldKind::Object, true),
            f("lastCreated", FieldKind::String, true),
            f("lastDestroyed", FieldKind::String, true),
            FieldSchema {
                nullable: true,
                ..f("wrapEnabled", FieldKind::Any, true)
            },
            FieldSchema {
                nullable: true,
                ..f("sequenceWrap", FieldKind::Any, true)
            },
            f("active", FieldKind::Map, false),
        ],
    }],
    prototypes: &[PrototypeSchema {
        id: PROTOTYPE,
        name: "SeqDb",
        parent: PrototypeParent::Object,
        members: &[
            p(1, "autoWrap"),
            p(2, "setAccTitle"),
            p(3, "setAccDescription"),
            p(4, "setDiagramTitle"),
            p(5, "parseMessage"),
            p(6, "parseBoxData"),
            p(7, "addBox"),
            p(8, "addActor"),
            p(9, "addMessage"),
            p(10, "addSignal"),
            p(11, "addNote"),
            p(12, "addLinks"),
            p(13, "addALink"),
            p(14, "addProperties"),
            p(15, "addDetails"),
            p(16, "apply"),
            p(17, "applyOne"),
        ],
    }],
    methods: &[
        m(1, "autoWrap", 0),
        m(2, "setAccTitle", 1),
        m(3, "setAccDescription", 1),
        m(4, "setDiagramTitle", 1),
        m(5, "parseMessage", 1),
        m(6, "parseBoxData", 1),
        m(7, "addBox", 1),
        m(8, "addActor", 5),
        m(9, "addMessage", 4),
        m(10, "addSignal", 4),
        m(11, "addNote", 3),
        m(12, "addLinks", 2),
        m(13, "addALink", 2),
        m(14, "addProperties", 2),
        m(15, "addDetails", 2),
        m(16, "apply", 1),
        m(17, "applyOne", 1),
    ],
};
pub struct SequencePublicState {
    graph: ModelGraph,
    projected: Result<SequenceModel, PeleError>,
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("sequence".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Sequence public model expansion exceeds the work budget");
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
        Err(error("Sequence public model requires an object reference"))
    }
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| error("Sequence public text must be a well-formed string"))
}
fn dense(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PeleError> {
    let id = reference(a)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(error("Sequence public collection must be an Array"));
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
        .map(|v| v.ok_or_else(|| error("Sparse Sequence public collections are not implemented")))
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
        Err(error("Sequence public collection must be a Map"))
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
            return Err(ty("Sequence collection must be a Map"));
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
                return Err(ty("Cannot assign to a read only Sequence array length"));
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
                return Err(ty("Cannot assign to a read only Sequence property"));
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
        reference(a).map_err(|_| ty("Sequence value must be an object"))
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
            return Err(ty("Overridden Sequence collection push is not implemented"));
        }
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            return Err(ty("Sequence collection push is not a function"));
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
        _ => return Err(error("Sequence object numeric coercion is not implemented").into()),
    })
}
fn js_string(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    Ok(match a {
        Atom::Undefined => "undefined".into(),
        Atom::Null => "null".into(),
        Atom::Bool(v) => v.to_string(),
        Atom::Number(n) => Value::Number(*n).js_string(),
        Atom::String(_) => text(g, a)?,
        _ => return Err(error("Sequence object string coercion is not implemented").into()),
    })
}
fn raw_trim(g: &ModelGraph, a: &Atom, left_only: bool) -> Result<JsString, PublicError> {
    let Atom::String(i) = a else {
        return Err(ty("Sequence argument must be a string"));
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
fn strict(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    if matches!(a,Atom::Number(v)if v.is_nan()) || matches!(b,Atom::Number(v)if v.is_nan()) {
        false
    } else {
        same(g, a, b)
    }
}
fn is(g: &ModelGraph, a: &Atom, s: &str) -> bool {
    matches!(a,Atom::String(i)if g.strings[*i as usize].is(s))
}
fn units(g: &ModelGraph, a: &Atom) -> Result<JsString, PublicError> {
    let Atom::String(i) = a else {
        return Err(ty("Sequence argument must be a string"));
    };
    Ok(g.strings[*i as usize].clone())
}
fn nullish(a: Atom, fallback: Atom) -> Atom {
    if matches!(a, Atom::Undefined | Atom::Null) {
        fallback
    } else {
        a
    }
}
fn slice(s: &JsString, from: isize, to: isize) -> JsString {
    let len = s.0.len() as isize;
    let norm = |n: isize| if n < 0 { (len + n).max(0) } else { n.min(len) };
    let from = norm(from);
    let to = norm(to).max(from);
    JsString(s.0[from as usize..to as usize].to_vec())
}
impl Arena<'_> {
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.checked_add(n).ok_or_else(limit)?;
        if self.work > MAX_WORK {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn guard(&self, root: u32, name: &str, id: u32) -> Result<(), PublicError> {
        let value = self.g.nodes[root as usize]
            .properties()
            .iter()
            .find(|p| self.g.strings[p.key as usize].is(name));
        match value {
            None => Ok(()),
            Some(p) if matches!(p.value,Atom::BuiltinFunction(i)if i==id) => Ok(()),
            Some(p) if matches!(p.value, Atom::BuiltinFunction(_) | Atom::HostReference(_)) => Err(
                error("Sequence overridden builder callback dispatch is not implemented").into(),
            ),
            _ => Err(ty("Sequence builder member is not a function")),
        }
    }
    fn map_has(&self, id: u32, key: &Atom) -> Result<bool, PublicError> {
        Ok(self.map_position(id, key)?.is_some())
    }
    fn map_field(&self, root: u32, name: &str, key: &Atom) -> Result<Atom, PublicError> {
        self.map_get(self.field_id(root, name)?, key)
    }
    fn auto_wrap(&self, root: u32) -> Result<Atom, PublicError> {
        self.guard(root, "autoWrap", 1)?;
        Ok(nullish(
            self.field(root, "wrapEnabled"),
            self.field(root, "sequenceWrap"),
        ))
    }
    fn json(&mut self, input: JsString) -> Result<Option<Atom>, PublicError> {
        let parsed = match super::json::parse_graph(&input) {
            Ok(v) => v,
            Err(super::json::GraphJsonError::Syntax) => return Ok(None),
            Err(super::json::GraphJsonError::Limit) => return Err(limit().into()),
        };
        fn atom(
            a: &mut Arena<'_>,
            g: &ModelGraph,
            value: &Atom,
            ids: &mut HashMap<u32, u32>,
            pending: &mut Vec<u32>,
        ) -> Result<Atom, PublicError> {
            Ok(match value {
                Atom::String(i) => a.string(g.strings[*i as usize].clone())?,
                Atom::Ref(i) => {
                    let n = if let Some(n) = ids.get(i) {
                        *n
                    } else {
                        let n = a.object()?;
                        ids.insert(*i, n);
                        pending.push(*i);
                        n
                    };
                    Atom::Ref(n)
                }
                v => v.clone(),
            })
        }
        let mut ids = HashMap::new();
        let mut pending = Vec::new();
        let root = atom(self, &parsed, &parsed.root, &mut ids, &mut pending)?;
        let mut at = 0;
        while at < pending.len() {
            let old = pending[at];
            at += 1;
            let id = ids[&old];
            let mut props = Vec::new();
            for p in parsed.nodes[old as usize].properties() {
                let key = self.intern(parsed.strings[p.key as usize].clone())?;
                let value = atom(self, &parsed, &p.value, &mut ids, &mut pending)?;
                props.push(Property {
                    key,
                    value,
                    flags: p.flags,
                });
                self.tick(1)?
            }
            self.g.nodes[id as usize] = match &parsed.nodes[old as usize] {
                Node::Object { .. } => Node::Object { properties: props },
                Node::Array { length, .. } => Node::Array {
                    length: *length,
                    properties: props,
                },
                _ => return Err(error("Sequence JSON graph must contain data records").into()),
            };
        }
        Ok(Some(root))
    }
    fn parsed_entries(&mut self, value: Atom) -> Result<Vec<(Atom, Atom)>, PublicError> {
        let mut out = Vec::new();
        match value {
            Atom::String(i) => {
                let values = self.g.strings[i as usize].0.clone();
                let mut at = 0;
                while at < values.len() {
                    let len = if (0xd800..=0xdbff).contains(&values[at])
                        && values
                            .get(at + 1)
                            .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
                    {
                        2
                    } else {
                        1
                    };
                    let key = self.string(out.len().to_string().as_str())?;
                    let value = self.string(JsString(values[at..at + len].to_vec()))?;
                    out.push((key, value));
                    at += len
                }
            }
            Atom::Ref(i) => {
                for p in self.g.nodes[i as usize].properties().to_vec() {
                    if p.flags & 1 != 0 {
                        out.push((Atom::String(p.key), p.value))
                    }
                }
            }
            _ => {}
        }
        Ok(out)
    }
    fn menu(
        &mut self,
        root: u32,
        actor_id: Atom,
        text: Atom,
        kind: &str,
    ) -> Result<(), PublicError> {
        match self.menu_inner(root, actor_id, text, kind) {
            Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                ..
            }) if kind == "links" || kind == "properties" => Ok(()),
            result => result,
        }
    }
    fn menu_inner(
        &mut self,
        root: u32,
        actor_id: Atom,
        text: Atom,
        kind: &str,
    ) -> Result<(), PublicError> {
        let actor = self.map_field(root, "actors", &actor_id)?;
        if !truthy(self.g, &actor) {
            return Ok(());
        }
        let actor = self.id(&actor)?;
        let text_obj = self.id(&text)?;
        let raw = self.field(text_obj, "text");
        if kind == "details" {
            self.assign(actor, "details", raw)?;
            return Ok(());
        }
        let collection = self.field_id(
            actor,
            if kind == "properties" {
                "properties"
            } else {
                "links"
            },
        )?;
        if kind == "link" {
            let raw = units(self.g, &raw)?;
            let sep = raw
                .0
                .iter()
                .position(|u| *u == 64)
                .map_or(-1, |i| i as isize);
            let one = self.string(raw_trim_units(slice(&raw, 0, sep - 1)))?;
            let two = self.string(raw_trim_units(slice(&raw, sep + 1, raw.0.len() as isize)))?;
            self.map_set(collection, one, two)?;
            return Ok(());
        }
        let raw = match raw {
            Atom::String(i) => self.g.strings[i as usize].clone(),
            ref v => js_string(self.g, v)?.as_str().into(),
        };
        let Some(value) = self.json(raw)? else {
            return Ok(());
        };
        for (k, v) in self.parsed_entries(value)? {
            self.map_set(collection, k, v)?
        }
        Ok(())
    }
    fn parse_message(&mut self, raw: Atom) -> Result<Atom, PublicError> {
        let mut text = raw_trim(self.g, &raw, false)?;
        let mut wrap = Atom::Undefined;
        for (prefix, value) in [
            (":nowrap:", false),
            ("nowrap:", false),
            (":wrap:", true),
            ("wrap:", true),
        ] {
            let prefix: Vec<_> = prefix.encode_utf16().collect();
            if text.0.starts_with(&prefix) {
                text.0.drain(..prefix.len());
                text = raw_trim_units(text);
                wrap = Atom::Bool(value);
                break;
            }
        }
        let text = self.string(text)?;
        Ok(Atom::Ref(
            self.record(vec![("text", text), ("wrap", wrap)])?,
        ))
    }
    fn parse_box(&mut self, raw: Atom) -> Result<Atom, PublicError> {
        let raw = units(self.g, &raw)?;
        let word = raw
            .0
            .iter()
            .take_while(|u| matches!(u,48..=57|65..=90|97..=122|95))
            .count();
        let mut parts = None;
        for prefix in ["rgb", "rgba", "hsl", "hsla"] {
            let prefix: Vec<_> = prefix.encode_utf16().collect();
            if raw.0.starts_with(&prefix) {
                let mut at = prefix.len();
                while raw
                    .0
                    .get(at)
                    .is_some_and(|u| char::from_u32(*u as u32).is_some_and(space))
                {
                    at += 1
                }
                if raw.0.get(at) == Some(&40) {
                    let start = at + 1;
                    if let Some(close) = raw.0.iter().rposition(|u| *u == 41) {
                        if close >= start
                            && !raw.0[start..].iter().any(|u| matches!(u, 13 | 8232 | 8233))
                        {
                            parts = Some((close + 1, close + 1));
                            break;
                        }
                    }
                }
            }
        }
        if parts.is_none() && !raw.0.iter().any(|u| matches!(u, 13 | 8232 | 8233)) {
            parts = Some((word, word))
        }
        let (mut color, mut title) = if let Some((end, start)) = parts {
            let color = if end == 0 {
                JsString::from("transparent")
            } else {
                raw_trim_units(JsString(raw.0[..end].to_vec()))
            };
            let title = if start == raw.0.len() {
                None
            } else {
                Some(raw_trim_units(JsString(raw.0[start..].to_vec())))
            };
            (color, title)
        } else {
            ("transparent".into(), None)
        };
        if !color
            .well_formed()
            .is_some_and(|s| super::colors::is_css_color(&s))
        {
            color = "transparent".into();
            title = Some(raw_trim_units(raw))
        }
        let color = self.string(color)?;
        let (text, wrap) = if let Some(title) = title {
            let title = self.string(title)?;
            let parsed = self.parse_message(title)?;
            let i = self.id(&parsed)?;
            let text = self.field(i, "text");
            (
                if truthy(self.g, &text) {
                    text
                } else {
                    Atom::Undefined
                },
                self.field(i, "wrap"),
            )
        } else {
            (Atom::Undefined, Atom::Undefined)
        };
        Ok(Atom::Ref(self.record(vec![
            ("text", text),
            ("color", color),
            ("wrap", wrap),
        ])?))
    }
    fn add_box(&mut self, root: u32, data: Atom) -> Result<(), PublicError> {
        let data = self.id(&data)?;
        let wrap = self.field(data, "wrap");
        let wrap = if matches!(wrap, Atom::Undefined | Atom::Null) {
            self.auto_wrap(root)?
        } else {
            wrap
        };
        let actors = self.array(vec![])?;
        let box_id = self.record(vec![
            ("name", self.field(data, "text")),
            ("wrap", wrap),
            ("fill", self.field(data, "color")),
            ("actorKeys", Atom::Ref(actors)),
        ])?;
        self.assign(root, "currentBox", Atom::Ref(box_id))?;
        let boxes = self.field(root, "boxes");
        let boxes = match boxes {
            Atom::Null => {
                return Err(ty("Cannot read properties of null (reading 'push')"));
            }
            Atom::Undefined => {
                return Err(ty("Cannot read properties of undefined (reading 'push')"));
            }
            _ => self.id(&boxes)?,
        };
        self.push(boxes, Atom::Ref(box_id))
    }
    fn add_actor(
        &mut self,
        root: u32,
        id: Atom,
        name: Atom,
        mut description: Atom,
        mut kind: Atom,
        metadata: Atom,
    ) -> Result<(), PublicError> {
        let mut assigned = self.field(root, "currentBox");
        if !matches!(metadata, Atom::Undefined) {
            let source = text(self.g, &metadata)?;
            let body = if source.contains('\n') {
                format!("{source}\n")
            } else {
                format!("{{\n{source}\n}}")
            };
            let doc = match yaml::parse(&body) {
                Ok(doc) => doc,
                Err(detail) => {
                    let id = js_string(self.g, &id)?;
                    let mut e = error(format!(
                        "Config of participant \"{id}\" is not valid YAML. {detail}"
                    ));
                    e.code = ErrorCode::Syntax;
                    return Err(e.into());
                }
            };
            if let Value::Object(doc) = doc {
                if let Some(value) = doc.get("type").filter(|v| !matches!(v, Value::Null)) {
                    kind = self.string(value.js_string().as_str())?
                }
                if let Some(value) = doc.get("alias").filter(|v| v.truthy()) {
                    let old_text = if let Atom::Ref(i) = description {
                        self.field(i, "text")
                    } else {
                        Atom::Undefined
                    };
                    if !truthy(self.g, &description) || strict(self.g, &old_text, &name) {
                        let text = self.string(value.js_string().as_str())?;
                        let wrap = if let Atom::Ref(i) = description {
                            self.field(i, "wrap")
                        } else {
                            Atom::Undefined
                        };
                        description = Atom::Ref(self.record(vec![
                            ("text", text),
                            ("wrap", wrap),
                            ("type", kind.clone()),
                        ])?)
                    }
                }
            }
        }
        let actors = self.field_id(root, "actors")?;
        let old = self.map_get(actors, &id)?;
        if truthy(self.g, &old) {
            let old_id = self.id(&old)?;
            let previous = self.field(old_id, "box");
            let current = self.field(root, "currentBox");
            if truthy(self.g, &current)
                && truthy(self.g, &previous)
                && !same(self.g, &current, &previous)
            {
                let previous = self.id(&previous)?;
                let current = self.id(&current)?;
                return Err(error(format!("A same participant should only be defined in one Box: {} can't be in '{}' and in '{}' at the same time.",js_string(self.g,&self.field(old_id,"name"))?,js_string(self.g,&self.field(previous,"name"))?,js_string(self.g,&self.field(current,"name"))?)).into());
            }
            assigned = if truthy(self.g, &previous) {
                previous
            } else {
                current
            };
            self.assign(old_id, "box", assigned.clone())?;
            if strict(self.g, &name, &self.field(old_id, "name"))
                && matches!(description, Atom::Undefined | Atom::Null)
            {
                return Ok(());
            }
        }
        let description_text = if let Atom::Ref(i) = description {
            self.field(i, "text")
        } else {
            Atom::Undefined
        };
        if matches!(description_text, Atom::Undefined | Atom::Null)
            || matches!(kind, Atom::Undefined | Atom::Null)
        {
            description =
                Atom::Ref(self.record(vec![("text", name.clone()), ("type", kind.clone())])?)
        }
        let description = self.id(&description)?;
        let wrap = self.field(description, "wrap");
        let wrap = if matches!(wrap, Atom::Undefined | Atom::Null) {
            self.auto_wrap(root)?
        } else {
            wrap
        };
        let links = self.map()?;
        let props = self.map()?;
        let kind = if matches!(kind, Atom::Undefined | Atom::Null) {
            self.string("participant")?
        } else {
            kind
        };
        let actor = self.record(vec![
            ("box", assigned),
            ("name", name),
            ("description", self.field(description, "text")),
            ("wrap", wrap),
            ("prevActor", self.field(root, "prevActor")),
            ("links", Atom::Ref(links)),
            ("properties", Atom::Ref(props)),
            ("type", kind),
        ])?;
        self.map_set(actors, id.clone(), Atom::Ref(actor))?;
        let prev = self.field(root, "prevActor");
        if truthy(self.g, &prev) {
            let prev = self.map_get(actors, &prev)?;
            if truthy(self.g, &prev) {
                let prev = self.id(&prev)?;
                self.assign(prev, "nextActor", id.clone())?
            }
        }
        let current = self.field(root, "currentBox");
        if truthy(self.g, &current) {
            let current = self.id(&current)?;
            let list = self.field_id(current, "actorKeys")?;
            self.push(list, id.clone())?
        }
        self.assign(root, "prevActor", id)?;
        Ok(())
    }
    fn add_message(
        &mut self,
        root: u32,
        from: Atom,
        to: Atom,
        message: Atom,
        answer: Atom,
    ) -> Result<(), PublicError> {
        let message = self.id(&message)?;
        let wrap = self.field(message, "wrap");
        let wrap = if matches!(wrap, Atom::Undefined | Atom::Null) {
            self.auto_wrap(root)?
        } else {
            wrap
        };
        let list = self.field_id(root, "messages")?;
        let Node::Array { length, .. } = self.g.nodes[list as usize] else {
            return Err(ty("Sequence messages must be an Array"));
        };
        let id = self.string(length.to_string().as_str())?;
        let record = self.record(vec![
            ("id", id),
            ("from", from),
            ("to", to),
            ("message", self.field(message, "text")),
            ("wrap", wrap),
            ("answer", answer),
        ])?;
        self.push(list, Atom::Ref(record))
    }
    fn add_signal(
        &mut self,
        root: u32,
        from: Atom,
        to: Atom,
        message: Atom,
        kind: Atom,
        activate: Atom,
        central: Atom,
    ) -> Result<Atom, PublicError> {
        let active = self.field_id(root, "active")?;
        if matches!(kind,Atom::Number(n)if n==18.) {
            let count = if truthy(self.g, &from) {
                nullish(self.map_get(active, &from)?, Atom::Number(0.))
            } else {
                Atom::Number(0.)
            };
            let count = number(self.g, &count)?;
            if count < 1. {
                return Err(error(format!(
                    "Trying to inactivate an inactive participant ({})",
                    js_string(self.g, &from)?
                ))
                .into());
            }
            self.map_set(active, from.clone(), Atom::Number(count - 1.))?
        } else if matches!(kind,Atom::Number(n)if n==17.) && truthy(self.g, &from) {
            let old = nullish(self.map_get(active, &from)?, Atom::Number(0.));
            let count = match old {
                Atom::String(i) => {
                    let mut s = self.g.strings[i as usize].clone();
                    s.0.push(49);
                    self.string(s)?
                }
                _ => Atom::Number(number(self.g, &old)? + 1.),
            };
            self.map_set(active, from.clone(), count)?
        }
        let (text, wrap) = if let Atom::Ref(i) = message {
            (
                nullish(self.field(i, "text"), self.string("")?),
                self.field(i, "wrap"),
            )
        } else {
            (self.string("")?, Atom::Undefined)
        };
        let wrap = if matches!(wrap, Atom::Undefined | Atom::Null) {
            self.auto_wrap(root)?
        } else {
            wrap
        };
        let list = self.field_id(root, "messages")?;
        let Node::Array { length, .. } = self.g.nodes[list as usize] else {
            return Err(ty("Sequence messages must be an Array"));
        };
        let id = self.string(length.to_string().as_str())?;
        let central = nullish(central, Atom::Number(0.));
        let record = self.record(vec![
            ("id", id),
            ("from", from),
            ("to", to),
            ("message", text),
            ("wrap", wrap),
            ("type", kind),
            ("activate", activate),
            ("centralConnection", central),
        ])?;
        self.push(list, Atom::Ref(record))?;
        Ok(Atom::Bool(true))
    }
    fn add_note(
        &mut self,
        root: u32,
        actor: Atom,
        placement: Atom,
        message: Atom,
    ) -> Result<(), PublicError> {
        let message = self.id(&message)?;
        let wrap = self.field(message, "wrap");
        let wrap = if matches!(wrap, Atom::Undefined | Atom::Null) {
            self.auto_wrap(root)?
        } else {
            wrap
        };
        let text = self.field(message, "text");
        let note = self.record(vec![
            ("actor", actor.clone()),
            ("placement", placement.clone()),
            ("message", text.clone()),
            ("wrap", wrap.clone()),
        ])?;
        let (from, to) = if matches!(actor, Atom::String(_)) {
            (actor.clone(), actor)
        } else {
            let id = self.id(&actor)?;
            (self.field(id, "0"), self.field(id, "1"))
        };
        let notes = self.field_id(root, "notes")?;
        self.push(notes, Atom::Ref(note))?;
        let messages = self.field_id(root, "messages")?;
        let Node::Array { length, .. } = self.g.nodes[messages as usize] else {
            return Err(ty("Sequence messages must be an Array"));
        };
        let id = self.string(length.to_string().as_str())?;
        let message = self.record(vec![
            ("id", id),
            ("from", from),
            ("to", to),
            ("message", text),
            ("wrap", wrap),
            ("type", Atom::Number(2.)),
            ("placement", placement),
        ])?;
        self.push(messages, Atom::Ref(message))
    }
}
fn raw_trim_units(s: JsString) -> JsString {
    let blank = |u: &u16| char::from_u32(*u as u32).is_some_and(space);
    let start = s.0.iter().take_while(|u| blank(u)).count();
    let end = s.0.len() - s.0[start..].iter().rev().take_while(|u| blank(u)).count();
    JsString(s.0[start..end].to_vec())
}
impl Arena<'_> {
    fn apply_one(&mut self, root: u32, param: Atom) -> Result<(), PublicError> {
        let p = match param {
            Atom::Ref(i) => i,
            Atom::Null | Atom::Undefined => {
                return Err(ty("Cannot read Sequence statement properties"))
            }
            _ => return Ok(()),
        };
        let actor = self.field(p, "actor");
        let kind = self.field(p, "type");
        if is(self.g, &kind, "sequenceIndex") {
            let spec = self.record(vec![
                ("start", self.field(p, "sequenceIndex")),
                ("step", self.field(p, "sequenceIndexStep")),
                (
                    "visible",
                    Atom::Bool(matches!(self.field(p, "sequenceVisible"), Atom::Bool(true))),
                ),
            ])?;
            let list = self.field_id(root, "messages")?;
            let Node::Array { length, .. } = self.g.nodes[list as usize] else {
                return Err(ty("Sequence messages must be an Array"));
            };
            let id = self.string(length.to_string().as_str())?;
            let message = self.record(vec![
                ("id", id),
                ("from", Atom::Undefined),
                ("to", Atom::Undefined),
                ("message", Atom::Ref(spec)),
                ("wrap", Atom::Bool(false)),
                ("type", self.field(p, "signalType")),
            ])?;
            self.push(list, Atom::Ref(message))?;
            return Ok(());
        }
        if is(self.g, &kind, "addParticipant") || is(self.g, &kind, "createParticipant") {
            if is(self.g, &kind, "createParticipant") {
                let actors = self.field_id(root, "actors")?;
                if self.map_has(actors, &actor)? {
                    return Err(error("It is not possible to have actors with the same id, even if one is destroyed before the next is created. Use 'AS' aliases to simulate the behavior").into());
                }
                self.assign(root, "lastCreated", actor.clone())?
            }
            self.guard(root, "addActor", 8)?;
            self.add_actor(
                root,
                actor.clone(),
                actor.clone(),
                self.field(p, "description"),
                self.field(p, "draw"),
                self.field(p, "config"),
            )?;
            if is(self.g, &kind, "createParticipant") {
                let list = self.field_id(root, "messages")?;
                let Node::Array { length, .. } = self.g.nodes[list as usize] else {
                    return Err(ty("Sequence messages must be an Array"));
                };
                let created = self.field_id(root, "createdActors")?;
                self.map_set(created, actor, Atom::Number(length as f64))?
            }
            return Ok(());
        }
        if is(self.g, &kind, "destroyParticipant") {
            self.assign(root, "lastDestroyed", actor.clone())?;
            let list = self.field_id(root, "messages")?;
            let Node::Array { length, .. } = self.g.nodes[list as usize] else {
                return Err(ty("Sequence messages must be an Array"));
            };
            let destroyed = self.field_id(root, "destroyedActors")?;
            self.map_set(destroyed, actor, Atom::Number(length as f64))?;
            return Ok(());
        }
        if [
            "activeStart",
            "activeEnd",
            "centralConnection",
            "centralConnectionReverse",
        ]
        .iter()
        .any(|s| is(self.g, &kind, s))
        {
            self.guard(root, "addSignal", 10)?;
            self.add_signal(
                root,
                actor,
                Atom::Undefined,
                Atom::Undefined,
                self.field(p, "signalType"),
                Atom::Bool(false),
                Atom::Undefined,
            )?;
            return Ok(());
        }
        if is(self.g, &kind, "addNote") {
            self.guard(root, "addNote", 11)?;
            return self.add_note(
                root,
                actor,
                self.field(p, "placement"),
                self.field(p, "text"),
            );
        }
        for (name, id, menu) in [
            ("addLinks", 12, "links"),
            ("addALink", 13, "link"),
            ("addProperties", 14, "properties"),
            ("addDetails", 15, "details"),
        ] {
            if is(self.g, &kind, name) {
                self.guard(root, name, id)?;
                return self.menu(root, actor, self.field(p, "text"), menu);
            }
        }
        if is(self.g, &kind, "addMessage") {
            let created = self.field(root, "lastCreated");
            let destroyed = self.field(root, "lastDestroyed");
            if truthy(self.g, &created) {
                if !strict(self.g, &self.field(p, "to"), &created) {
                    return Err(error(format!("The created participant {} does not have an associated creating message after its declaration. Please check the sequence diagram.",js_string(self.g,&created)?)).into());
                }
                self.assign(root, "lastCreated", Atom::Undefined)?
            } else if truthy(self.g, &destroyed) {
                if !strict(self.g, &self.field(p, "to"), &destroyed)
                    && !strict(self.g, &self.field(p, "from"), &destroyed)
                {
                    return Err(error(format!("The destroyed participant {} does not have an associated destroying message after its declaration. Please check the sequence diagram.",js_string(self.g,&destroyed)?)).into());
                }
                self.assign(root, "lastDestroyed", Atom::Undefined)?
            }
            self.guard(root, "addSignal", 10)?;
            let activate = match self.field(p, "activate") {
                Atom::Undefined => Atom::Bool(false),
                v => v,
            };
            self.add_signal(
                root,
                self.field(p, "from"),
                self.field(p, "to"),
                self.field(p, "msg"),
                self.field(p, "signalType"),
                activate,
                self.field(p, "centralConnection"),
            )?;
            return Ok(());
        }
        if is(self.g, &kind, "boxStart") {
            self.guard(root, "addBox", 7)?;
            return self.add_box(root, self.field(p, "boxData"));
        }
        if is(self.g, &kind, "boxEnd") {
            self.assign(root, "currentBox", Atom::Undefined)?;
            return Ok(());
        }
        let field = if is(self.g, &kind, "loopStart") {
            Some("loopText")
        } else if is(self.g, &kind, "rectStart") {
            Some("color")
        } else if is(self.g, &kind, "optStart") {
            Some("optText")
        } else if is(self.g, &kind, "altStart") || is(self.g, &kind, "else") {
            Some("altText")
        } else if is(self.g, &kind, "parStart") || is(self.g, &kind, "and") {
            Some("parText")
        } else if is(self.g, &kind, "criticalStart") {
            Some("criticalText")
        } else if is(self.g, &kind, "option") {
            Some("optionText")
        } else if is(self.g, &kind, "breakStart") {
            Some("breakText")
        } else {
            None
        };
        if let Some(field) = field {
            self.guard(root, "addSignal", 10)?;
            self.add_signal(
                root,
                Atom::Undefined,
                Atom::Undefined,
                self.field(p, field),
                self.field(p, "signalType"),
                Atom::Bool(false),
                Atom::Undefined,
            )?;
        } else if [
            "loopEnd",
            "rectEnd",
            "optEnd",
            "altEnd",
            "parEnd",
            "criticalEnd",
            "breakEnd",
        ]
        .iter()
        .any(|s| is(self.g, &kind, s))
        {
            self.guard(root, "addSignal", 10)?;
            self.add_signal(
                root,
                Atom::Undefined,
                Atom::Undefined,
                Atom::Undefined,
                self.field(p, "signalType"),
                Atom::Bool(false),
                Atom::Undefined,
            )?;
        }
        Ok(())
    }
    fn apply(&mut self, root: u32, input: Atom) -> Result<(), PublicError> {
        enum Entry {
            Visit(Atom),
            Leave(u32),
        }
        let mut pending = vec![Entry::Visit(input)];
        let mut active = HashSet::new();
        while let Some(entry) = pending.pop() {
            self.tick(1)?;
            let item = match entry {
                Entry::Leave(i) => {
                    active.remove(&i);
                    continue;
                }
                Entry::Visit(item) => item,
            };
            if matches!(item, Atom::Undefined | Atom::String(_)) {
                continue;
            }
            if let Atom::Ref(i) = item {
                if let Node::Array { length, properties } = &self.g.nodes[i as usize] {
                    if !active.insert(i) {
                        return Err(limit().into());
                    }
                    if *length as usize > MAX_ITEMS {
                        return Err(limit().into());
                    }
                    let mut values = vec![Atom::Undefined; *length as usize];
                    for p in properties {
                        if let Some(n) = self.g.strings[p.key as usize]
                            .well_formed()
                            .and_then(|s| s.parse::<usize>().ok())
                        {
                            if n < values.len() && self.g.strings[p.key as usize].is(&n.to_string())
                            {
                                values[n] = p.value.clone()
                            }
                        }
                    }
                    self.tick(values.len())?;
                    pending.push(Entry::Leave(i));
                    pending.extend(values.into_iter().rev().map(Entry::Visit));
                    continue;
                }
            }
            self.guard(root, "applyOne", 17)?;
            self.apply_one(root, item)?
        }
        Ok(())
    }
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let mut a = Arena::new(g);
    let root = a.id(&call.receiver)?;
    match call.method.0 {
        1 => Ok(nullish(
            a.field(root, "wrapEnabled"),
            a.field(root, "sequenceWrap"),
        )),
        2 => {
            let text = raw_trim(a.g, &arg(&call, 0), true)?;
            let text = a.string(text)?;
            a.assign(root, "accTitle", text)?;
            Ok(Atom::Undefined)
        }
        3 => {
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
        4 => {
            a.assign(root, "title", arg(&call, 0))?;
            Ok(Atom::Undefined)
        }
        5 => a.parse_message(arg(&call, 0)),
        6 => a.parse_box(arg(&call, 0)),
        7 => {
            a.add_box(root, arg(&call, 0))?;
            Ok(Atom::Undefined)
        }
        8 => {
            a.add_actor(
                root,
                arg(&call, 0),
                arg(&call, 1),
                arg(&call, 2),
                arg(&call, 3),
                arg(&call, 4),
            )?;
            Ok(Atom::Undefined)
        }
        9 => {
            a.add_message(
                root,
                arg(&call, 0),
                arg(&call, 1),
                arg(&call, 2),
                arg(&call, 3),
            )?;
            Ok(Atom::Undefined)
        }
        10 => a.add_signal(
            root,
            arg(&call, 0),
            arg(&call, 1),
            arg(&call, 2),
            arg(&call, 3),
            match arg(&call, 4) {
                Atom::Undefined => Atom::Bool(false),
                v => v,
            },
            arg(&call, 5),
        ),
        11 => {
            a.add_note(root, arg(&call, 0), arg(&call, 1), arg(&call, 2))?;
            Ok(Atom::Undefined)
        }
        12..=15 => {
            a.menu(
                root,
                arg(&call, 0),
                arg(&call, 1),
                match call.method.0 {
                    12 => "links",
                    13 => "link",
                    14 => "properties",
                    _ => "details",
                },
            )?;
            Ok(Atom::Undefined)
        }
        16 => {
            a.apply(root, arg(&call, 0))?;
            Ok(Atom::Undefined)
        }
        17 => {
            a.apply_one(root, arg(&call, 0))?;
            Ok(Atom::Undefined)
        }
        _ => Err(error("Sequence native method is unavailable").into()),
    }
}
fn new_graph(m: &SequenceModel) -> Result<ModelGraph, PublicError> {
    let mut g = ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec![],
        nodes: vec![],
    };
    let mut a = Arena::new(&mut g);
    let root = a.node(Node::Instance {
        prototype: PROTOTYPE,
        properties: vec![],
    })?;
    a.g.root = Atom::Ref(root);
    let actors = a.map()?;
    let created = a.map()?;
    let destroyed = a.map()?;
    let active = a.map()?;
    let boxes = a.array(vec![])?;
    let messages = a.array(vec![])?;
    let notes = a.array(vec![])?;
    let kind = a.string("sequence")?;
    let fields = vec![
        ("type", kind),
        ("actors", Atom::Ref(actors)),
        ("createdActors", Atom::Ref(created)),
        ("destroyedActors", Atom::Ref(destroyed)),
        ("boxes", Atom::Ref(boxes)),
        ("messages", Atom::Ref(messages)),
        ("notes", Atom::Ref(notes)),
        ("title", a.opt(m.title.as_ref())?),
        ("accTitle", a.opt(m.acc_title.as_ref())?),
        ("accDescr", a.opt(m.acc_descr.as_ref())?),
        ("prevActor", a.opt(m.public_origin.prev_actor.as_ref())?),
        ("currentBox", Atom::Undefined),
        ("lastCreated", a.opt(m.public_origin.last_created.as_ref())?),
        (
            "lastDestroyed",
            a.opt(m.public_origin.last_destroyed.as_ref())?,
        ),
        (
            "wrapEnabled",
            m.public_origin
                .wrap_enabled
                .map_or(Atom::Undefined, Atom::Bool),
        ),
        ("sequenceWrap", Atom::Bool(m.public_origin.sequence_wrap)),
        ("active", Atom::Ref(active)),
    ];
    for (k, v) in fields {
        a.assign(root, k, v)?
    }
    let mut box_ids = Vec::new();
    for b in &m.boxes {
        let keys = a.strings(&b.actor_keys)?;
        let name = a.opt(b.name.as_ref())?;
        let fill = a.string(b.fill.as_str())?;
        let id = a.record(vec![
            ("name", name),
            ("wrap", Atom::Bool(b.wrap)),
            ("fill", fill),
            ("actorKeys", Atom::Ref(keys)),
        ])?;
        box_ids.push(id);
        a.push(boxes, Atom::Ref(id))?
    }
    if let Some(i) = m.public_origin.current_box {
        let id = *box_ids
            .get(i)
            .ok_or_else(|| error("Sequence source box provenance is out of bounds"))?;
        a.assign(root, "currentBox", Atom::Ref(id))?
    }
    for (index, (key, actor)) in m.actors.iter().enumerate() {
        let links = a.map()?;
        let properties = a.map()?;
        let fields = vec![
            (
                "box",
                actor
                    .box_index
                    .map_or(Atom::Undefined, |i| Atom::Ref(box_ids[i])),
            ),
            ("name", a.string(actor.name.as_str())?),
            ("description", a.string(actor.description.as_str())?),
            ("wrap", Atom::Bool(actor.wrap)),
            ("prevActor", a.opt(actor.prev_actor.as_ref())?),
            ("links", Atom::Ref(links)),
            ("properties", Atom::Ref(properties)),
            ("type", a.string(actor.r#type.as_str())?),
            ("nextActor", a.opt(actor.next_actor.as_ref())?),
            ("details", a.opt(actor.details.as_ref())?),
        ];
        let id = a.object()?;
        let order = m.public_origin.actor_keys.get(index).map_or(
            &[
                "box",
                "name",
                "description",
                "wrap",
                "prevActor",
                "links",
                "properties",
                "type",
            ][..],
            Vec::as_slice,
        );
        for name in order {
            if let Some((_, v)) = fields.iter().find(|(k, _)| k == name) {
                a.assign(id, name, v.clone())?
            }
        }
        for (k, v) in fields {
            if !order.contains(&k) && !matches!(v, Atom::Undefined) {
                a.assign(id, k, v)?
            }
        }
        let key = a.string(key.as_str())?;
        a.map_set(actors, key.clone(), Atom::Ref(id))?;
        if let Some(operations) = m.public_origin.menus.get(index) {
            for (kind, text) in operations {
                let text = a.string(text.as_str())?;
                let object = a.record(vec![("text", text)])?;
                a.menu(root, key.clone(), Atom::Ref(object), kind)?
            }
        } else {
            for (source, target) in [(&actor.links, links), (&actor.properties, properties)] {
                for (k, v) in source {
                    let key = a.string(k.as_str())?;
                    let value = a
                        .json(v.json().into())?
                        .ok_or_else(|| error("Sequence source menu JSON must be valid"))?;
                    a.map_set(target, key, value)?
                }
            }
        }
    }
    for (map, values) in [
        (created, &m.created_actors),
        (destroyed, &m.destroyed_actors),
        (active, &m.public_origin.active),
    ] {
        for (k, v) in values {
            let k = a.string(k.as_str())?;
            a.map_set(map, k, Atom::Number(*v as f64))?
        }
    }
    for message in &m.messages {
        let id = a.string(message.id.as_str())?;
        let text = match &message.message {
            MessageText::Text(s) => a.string(s.as_str())?,
            MessageText::Numbering(s) => Atom::Ref(a.record(vec![
                ("start", s.start.map_or(Atom::Undefined, Atom::Number)),
                ("step", s.step.map_or(Atom::Undefined, Atom::Number)),
                ("visible", Atom::Bool(s.visible)),
            ])?),
        };
        let mut fields = vec![
            ("id", id),
            ("from", a.opt(message.from.as_ref())?),
            ("to", a.opt(message.to.as_ref())?),
            ("message", text),
            ("wrap", Atom::Bool(message.wrap)),
        ];
        if let Some(answer) = &message.answer {
            let value = a
                .json(answer.json().into())?
                .ok_or_else(|| error("Sequence source answer JSON must be valid"))?;
            fields.push(("answer", value))
        } else {
            fields.push((
                "type",
                message
                    .r#type
                    .map_or(Atom::Undefined, |v| Atom::Number(v as f64)),
            ));
            if let Some(v) = message.activate {
                fields.push(("activate", Atom::Bool(v)))
            }
            if let Some(v) = message.central_connection {
                fields.push(("centralConnection", Atom::Number(v as f64)))
            }
            if let Some(v) = message.placement {
                fields.push(("placement", Atom::Number(v as f64)))
            }
        }
        let id = a.record(fields)?;
        a.push(messages, Atom::Ref(id))?
    }
    for note in &m.notes {
        let actor = match &note.actor {
            super::NoteActors::Actor(s) => a.string(s.as_str())?,
            super::NoteActors::Actors(v) => Atom::Ref(a.strings(v)?),
        };
        let text = a.string(note.message.as_str())?;
        let note = a.record(vec![
            ("actor", actor),
            ("placement", Atom::Number(note.placement as f64)),
            ("message", text),
            ("wrap", Atom::Bool(note.wrap)),
        ])?;
        a.push(notes, Atom::Ref(note))?
    }
    Ok(g)
}
fn public_to_pele(e: PublicError) -> PeleError {
    match e {
        PublicError::Pele(e) => e,
        PublicError::JavaScript { message, .. } => error(
            message
                .well_formed()
                .unwrap_or_else(|| "Sequence native TypeError".into()),
        ),
        _ => error("Sequence native host exception"),
    }
}
fn project(g: &ModelGraph) -> Result<SequenceModel, PeleError> {
    struct Import<'a> {
        g: &'a ModelGraph,
        units: std::cell::Cell<usize>,
    }
    impl Import<'_> {
        fn text(&self, a: &Atom) -> Result<String, PeleError> {
            let raw = self.g.text(a)?;
            let units = self
                .units
                .get()
                .checked_add(raw.0.len())
                .ok_or_else(limit)?;
            if units > MAX_TEXT {
                return Err(limit());
            }
            self.units.set(units);
            raw.well_formed()
                .ok_or_else(|| error("Sequence rendering requires well-formed UTF16 text"))
        }
        fn opt(&self, a: &Atom) -> Result<Option<String>, PeleError> {
            if matches!(a, Atom::Undefined | Atom::Null) {
                Ok(None)
            } else {
                self.text(a).map(Some)
            }
        }
        fn byte(&self, a: &Atom) -> Result<Option<u8>, PeleError> {
            match a {
                Atom::Undefined => Ok(None),
                Atom::Number(n) if n.is_finite() && *n >= 0. && *n <= 255. && n.fract() == 0. => {
                    Ok(Some(*n as u8))
                }
                _ => Err(error(
                    "Sequence renderer numeric kind/placement requires an integer in 0..255",
                )),
            }
        }
        fn count(&self, a: &Atom) -> Result<usize, PeleError> {
            match a {
                Atom::Number(n)
                    if n.is_finite() && *n >= 0. && n.fract() == 0. && *n <= usize::MAX as f64 =>
                {
                    Ok(*n as usize)
                }
                _ => Err(error(
                    "Sequence renderer lifecycle index requires a nonnegative integer",
                )),
            }
        }
    }
    let i = Import {
        g,
        units: std::cell::Cell::new(0),
    };
    let root = reference(&g.root)?;
    let mut boxes = Vec::new();
    let mut box_ids = HashMap::new();
    let mut actors = Vec::new();
    for (k, v) in entries(g, get(g, root, "actors"))? {
        let id = reference(&v)?;
        let key = i.text(&k)?;
        let boxed = get(g, id, "box");
        let box_index = if truthy(g, boxed) {
            let boxed = reference(boxed)?;
            Some(if let Some(index) = box_ids.get(&boxed) {
                *index
            } else {
                let index = boxes.len();
                boxes.push(SeqBox {
                    name: i.opt(get(g, boxed, "name"))?,
                    wrap: truthy(g, get(g, boxed, "wrap")),
                    fill: i.text(get(g, boxed, "fill"))?,
                    actor_keys: vec![],
                });
                box_ids.insert(boxed, index);
                index
            })
        } else {
            None
        };
        let mut properties = Vec::new();
        let p = reference(get(g, id, "properties"))?;
        let Node::Map { entries, .. } = &g.nodes[p as usize] else {
            return Err(error(
                "Sequence rendering requires the actor properties Map",
            ));
        };
        if g.nodes[p as usize]
            .properties()
            .iter()
            .any(|p| g.strings[p.key as usize].is("get"))
        {
            return Err(error(
                "Sequence overridden properties getter dispatch is not implemented",
            ));
        }
        if let Some((_, value)) = entries.iter().find(|(k, _)| is(g, k, "class")) {
            if matches!(value, Atom::String(_)) {
                properties.push((
                    "class".into(),
                    super::json::SeqValue::value(Value::String(i.text(value)?)),
                ));
            }
        }
        let kind = match get(g, id, "type") {
            Atom::String(s) => g.strings[*s as usize].well_formed().unwrap_or_default(),
            _ => String::new(),
        };
        let description = i.opt(get(g, id, "description"))?.unwrap_or_default();
        actors.push((
            key,
            SeqActor {
                r#box: None,
                box_index,
                name: String::new(),
                description,
                wrap: truthy(g, get(g, id, "wrap")),
                prev_actor: None,
                next_actor: None,
                links: vec![],
                properties,
                details: None,
                r#type: kind,
            },
        ));
    }
    let mut messages = Vec::new();
    for v in dense(g, get(g, root, "messages"))? {
        let id = reference(&v)?;
        let message = get(g, id, "message");
        let message = if let Atom::Ref(spec) = message {
            let scalar = |key: &str| match get(g, *spec, key) {
                Atom::Undefined => Ok(None),
                Atom::Number(n) => Ok(Some(*n)),
                _ => Err(error("Sequence numbering start/step must be numeric")),
            };
            MessageText::Numbering(SeqNumbering {
                start: scalar("start")?,
                step: scalar("step")?,
                visible: truthy(g, get(g, *spec, "visible")),
            })
        } else {
            MessageText::Text(i.opt(message)?.unwrap_or_default())
        };
        messages.push(SeqMessage {
            id: i.text(get(g, id, "id"))?,
            from: i.opt(get(g, id, "from"))?,
            to: i.opt(get(g, id, "to"))?,
            message,
            wrap: truthy(g, get(g, id, "wrap")),
            r#type: i.byte(get(g, id, "type"))?,
            activate: match get(g, id, "activate") {
                Atom::Undefined => None,
                v => Some(truthy(g, v)),
            },
            placement: i.byte(get(g, id, "placement"))?,
            central_connection: i.byte(get(g, id, "centralConnection"))?,
            answer: None,
        });
    }
    let actor_keys: HashSet<&str> = actors.iter().map(|(key, _)| key.as_str()).collect();
    let counts = |name: &str| -> Result<Vec<(String, usize)>, PeleError> {
        let mut out = Vec::new();
        for (k, v) in entries(g, get(g, root, name))? {
            if let Atom::String(s) = k {
                if g.strings[s as usize]
                    .well_formed()
                    .is_some_and(|s| actor_keys.contains(s.as_str()))
                {
                    out.push((i.text(&Atom::String(s))?, i.count(&v)?))
                }
            }
        }
        Ok(out)
    };
    let created_actors = counts("createdActors")?;
    let destroyed_actors = counts("destroyedActors")?;
    let shared_boxes: Vec<_> = boxes.iter().cloned().map(std::sync::Arc::new).collect();
    for (_, actor) in &mut actors {
        actor.r#box = actor.box_index.map(|n| shared_boxes[n].clone())
    }

    Ok(SequenceModel {
        public_origin: SequencePublicOrigin::default(),
        diagram_type: DiagramType::Sequence,
        actors,
        created_actors,
        destroyed_actors,
        boxes,
        messages,
        notes: vec![],
        title: i.opt(get(g, root, "title"))?,
        acc_title: i.opt(get(g, root, "accTitle"))?,
        acc_descr: i.opt(get(g, root, "accDescr"))?,
    })
}
// Source preparation accepts arbitrary native message edits. Its renderer raises
// TypeError when an unfinished frame or a rect is used as a divider owner. Keep
// that operation boundary out of the typed renderer's unchecked label indexing.
fn validate_frame_labels(m: &SequenceModel) -> Result<(), PublicError> {
    use super::kinds::*;
    let start = |kind| {
        matches!(
            kind,
            Some(
                LOOP_START
                    | ALT_START
                    | OPT_START
                    | PAR_START
                    | PAR_OVER_START
                    | CRITICAL_START
                    | BREAK_START
                    | RECT_START
            )
        )
    };
    let end = |kind| {
        matches!(
            kind,
            Some(LOOP_END | ALT_END | OPT_END | PAR_END | CRITICAL_END | BREAK_END | RECT_END)
        )
    };
    let mut open = Vec::new();
    let mut closed = HashSet::new();
    for (index, message) in m.messages.iter().enumerate() {
        if start(message.r#type) {
            open.push(index)
        } else if end(message.r#type) {
            if let Some(index) = open.pop() {
                closed.insert(index);
            }
        }
    }
    open.clear();
    for (index, message) in m.messages.iter().enumerate() {
        if start(message.r#type) {
            open.push(index)
        } else if end(message.r#type) {
            open.pop();
        } else if matches!(message.r#type, Some(ALT_ELSE | PAR_AND | CRITICAL_OPTION)) {
            if let Some(owner) = open.last() {
                if !closed.contains(owner) || m.messages[*owner].r#type == Some(RECT_START) {
                    return Err(ty("Cannot read properties of undefined (reading 'height')"));
                }
            }
        }
    }
    Ok(())
}
fn construct_default() -> Result<SequencePublicState, PeleError> {
    let graph = new_graph(&super::model::Db::new(&Object::default(), None).model)
        .map_err(public_to_pele)?;
    let projected = project(&graph);
    Ok(SequencePublicState { graph, projected })
}
fn construct_initial(_: ConstructorArgument) -> Result<SequencePublicState, PeleError> {
    let mut state = construct_default()?;
    let Atom::Ref(root) = state.graph.root else {
        unreachable!("native class root")
    };
    Arena::new(&mut state.graph)
        .assign(root, "sequenceWrap", Atom::Undefined)
        .map_err(public_to_pele)?;
    state.projected = project(&state.graph);
    Ok(state)
}
enum SequenceConstruction {
    Start(ConstructorArgument),
    Wrap(Atom),
    SequenceWrap,
    Done,
}
fn resume_construction(
    state: &mut SequencePublicState,
    phase: &mut SequenceConstruction,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    fn run(
        state: &mut SequencePublicState,
        phase: &mut SequenceConstruction,
        reply: Option<PublicHostValue>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        let Atom::Ref(root) = state.graph.root else {
            unreachable!("native class root")
        };
        let mut arena = Arena::new(&mut state.graph);
        Ok(match std::mem::replace(phase, SequenceConstruction::Done) {
            SequenceConstruction::Start(argument) => {
                let options = if matches!(argument, ConstructorArgument::Undefined) {
                    Atom::Ref(arena.object()?)
                } else {
                    argument.into_atom(|value| arena.string(value))?
                };
                *phase = SequenceConstruction::Wrap(options.clone());
                PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: options,
                    key: "wrap",
                })
            }
            SequenceConstruction::Wrap(options) => {
                let value = reply
                    .expect("awaited wrap")
                    .into_atom(|value| arena.string(value))?;
                arena.assign(root, "wrapEnabled", value)?;
                *phase = SequenceConstruction::SequenceWrap;
                PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: options,
                    key: "sequenceWrap",
                })
            }
            SequenceConstruction::SequenceWrap => {
                let value = reply
                    .expect("awaited sequenceWrap")
                    .into_atom(|value| arena.string(value))?;
                let value = if matches!(value, Atom::Undefined | Atom::Null) {
                    Atom::Bool(false)
                } else {
                    value
                };
                arena.assign(root, "sequenceWrap", value)?;
                PublicInvocationPhase::Complete(Ok(Atom::Undefined))
            }
            SequenceConstruction::Done => {
                return Err(error("Sequence construction already finished").into())
            }
        })
    }
    let result = run(state, phase, reply);
    state.projected = project(&state.graph);
    match result {
        Ok(step) => step,
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}
fn constructor_body(argument: ConstructorArgument) -> DetachedPublicInvocation {
    DetachedPublicInvocation::new::<Sequence, SequencePublicState, _>(
        SequenceConstruction::Start(argument),
        resume_construction,
    )
}

impl PublicDiagramModule for Sequence {
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
                &[],
            )
        })
    }

    type PublicState = SequencePublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let projected = project(&graph);
        Ok(SequencePublicState { graph, projected })
    }
    fn create_public(&self, m: &SequenceModel) -> Result<Self::PublicState, PeleError> {
        let graph = new_graph(m).map_err(public_to_pele)?;
        let projected = project(&graph);
        Ok(SequencePublicState { graph, projected })
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
    fn render_public_operation(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        let model = s.projected.as_ref().map_err(Clone::clone)?;
        validate_frame_labels(model)?;
        self.render(model, c, o, h).map_err(Into::into)
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render_public_operation(s, c, o, h)
            .map_err(public_to_pele)
    }
}
