//! Native mutable GanttDb codec; Pele 458322a5 (MIT, kepano).
//! Gantt scheduler owns behavior; arena retains task/Date identity and cycles.
//! Default Array length flags2 implicit; nondefault length descriptor retained.
//! Source parse preserves real aliases. From an externally edited typed model,
//! the codec reconstructs canonical caches; erased object/history identity cannot
//! be recovered from a task Vec and millisecond numbers.
//! Host getTasks overrides use a borrowed active arena; browser synchronization
//! journals changed records. Arbitrary other method overrides and active lone-
//! surrogate scalar drawing/calendar remain explicit capabilities.
//! Pinned npm diagnostic labels do not promise Function.name or function source.
use super::{
    dates::Calendar,
    scheduler::{self, Flag, Store, Time},
    *,
};
use crate::compat::{
    public::{
        ActiveModel, Atom, ConstructorArgument, DetachedPublicInvocation, FieldKind, FieldSchema,
        JavaScriptErrorKind, JsString, LiveConstructorField, MemberKind, MemberSchema, MethodCall,
        MethodId, MethodSchema, ModelDelta, ModelGraph, ModelSchema, Node, Property, PrototypeId,
        PrototypeParent, PrototypeSchema, PublicConstructorFactory, PublicDiagramModule,
        PublicError, PublicHostOperation, PublicHostResult, PublicHostValue, PublicInvocationPhase,
        RecordId, RecordKind, RecordSchema,
    },
    ErrorCode,
};
use std::collections::{BTreeSet, HashMap, HashSet};
const PROTO: PrototypeId = PrototypeId(1);
pub struct GanttPublicState {
    graph: ModelGraph,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("gantt".into());
    e
}
fn limit() -> PeleError {
    PeleError::new(
        "Gantt native graph operation exceeds work budget",
        ErrorCode::Limit,
    )
}
fn own<'a>(g: &'a ModelGraph, id: u32, k: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(k).then_some(&p.value))
}
fn type_error(s: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: JsString::from(s),
    }
}
fn truth(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(b) => *b,
        Atom::Number(n) => *n != 0. && !n.is_nan(),
        Atom::String(id) => !g.strings[*id as usize].0.is_empty(),
        _ => true,
    }
}
fn space(c: u16) -> bool {
    matches!(c,0x9..=0xd|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
fn trimmed(v: &[u16]) -> &[u16] {
    let start = v.iter().position(|c| !space(*c)).unwrap_or(v.len());
    let end = v.iter().rposition(|c| !space(*c)).map_or(start, |n| n + 1);
    &v[start..end]
}
#[derive(Clone, Hash, PartialEq, Eq)]
enum Key {
    Undefined,
    Null,
    Bool(bool),
    Number(u64),
    String(u32),
    Ref(u32),
    Function(u32),
    Host(u32),
}
// Small records already provide contiguous canonical-key lookup storage.
// Keep an operation-local marker so first-touch work charges remain identical.
const LINEAR_PROPERTY_LIMIT: usize = 32;
enum PropertyIndex {
    Linear,
    Hashed(HashMap<u32, usize>),
}
impl PropertyIndex {
    fn hashed(properties: &[Property], canonical: &[u32]) -> HashMap<u32, usize> {
        let mut index = HashMap::new();
        for (position, property) in properties.iter().enumerate() {
            index
                .entry(canonical[property.key as usize])
                .or_insert(position);
        }
        index
    }
    fn new(properties: &[Property], canonical: &[u32]) -> Self {
        if properties.len() <= LINEAR_PROPERTY_LIMIT {
            Self::Linear
        } else {
            Self::Hashed(Self::hashed(properties, canonical))
        }
    }
    fn position(&self, key: u32, properties: &[Property], canonical: &[u32]) -> Option<usize> {
        match self {
            Self::Linear => properties
                .iter()
                .position(|property| canonical[property.key as usize] == key),
            Self::Hashed(index) => index.get(&key).copied(),
        }
    }
    // Called only for a key already proven absent, before its property is appended.
    fn insert(&mut self, key: u32, position: usize, properties: &[Property], canonical: &[u32]) {
        match self {
            Self::Linear if position < LINEAR_PROPERTY_LIMIT => {}
            Self::Linear => {
                let mut index = Self::hashed(properties, canonical);
                index.insert(key, position);
                *self = Self::Hashed(index);
            }
            Self::Hashed(index) => {
                index.insert(key, position);
            }
        }
    }
}
fn charge_work(work: &mut usize, amount: usize) -> Result<(), PublicError> {
    *work = work.saturating_add(amount);
    if *work > 200_000_000 {
        Err(limit().into())
    } else {
        Ok(())
    }
}
struct Context<'a> {
    g: &'a mut ModelGraph,
    strings: HashMap<JsString, u32>,
    canonical: Vec<u32>,
    maps: HashMap<u32, HashMap<Key, usize>>,
    properties: HashMap<u32, PropertyIndex>,
    sets: HashMap<u32, HashSet<Key>>,
    work: usize,
    refs: usize,
    units: usize,
    sync: Option<SyncJournal>,
}
struct SyncJournal {
    strings: usize,
    nodes: usize,
    dirty: BTreeSet<u32>,
}
enum Iter {
    Array(u32, u32),
    Map(u32, usize),
    Set(u32, usize),
    String(u32, usize),
}
impl<'a> Context<'a> {
    fn new(g: &'a mut ModelGraph) -> Self {
        let mut strings = HashMap::new();
        let mut canonical = vec![];
        let mut units = 0;
        let mut refs = 0;
        for (n, s) in g.strings.iter().enumerate() {
            units += s.0.len();
            canonical.push(*strings.entry(s.clone()).or_insert(n as u32));
        }
        for n in &g.nodes {
            refs += n.properties().len()
                + match n {
                    Node::Map { entries, .. } => entries.len() * 2,
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                };
        }
        Self {
            g,
            strings,
            canonical,
            maps: HashMap::new(),
            properties: HashMap::new(),
            sets: HashMap::new(),
            work: 0,
            refs,
            units,
            sync: None,
        }
    }
    fn changed(&mut self, id: u32) {
        if let Some(sync) = &mut self.sync {
            sync.dirty.insert(id);
        }
    }
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        charge_work(&mut self.work, n)
    }
    fn room(&mut self, refs: usize) -> Result<(), PublicError> {
        self.refs = self.refs.saturating_add(refs);
        if self.refs > 8_000_000 || self.g.nodes.len() >= 1_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
    }
    fn intern(&mut self, s: JsString) -> Result<Atom, PublicError> {
        if let Some(id) = self.strings.get(&s) {
            return Ok(Atom::String(*id));
        }
        if self.g.strings.len() >= 1_000_000 || self.units.saturating_add(s.0.len()) > 16_000_000 {
            return Err(limit().into());
        }
        self.units += s.0.len();
        let id = self.g.strings.len() as u32;
        self.strings.insert(s.clone(), id);
        self.canonical.push(id);
        self.g.strings.push(s);
        Ok(Atom::String(id))
    }
    fn text(&mut self, s: &str) -> Result<Atom, PublicError> {
        let mut buffer = [0; 64];
        if let Some(units) = JsString::short_utf16(s, &mut buffer) {
            if let Some(id) = self.strings.get(units).copied() {
                // intern charges graph/string capacity only on insertion. An
                // existing ID needs no temporary owned UTF-16 string or charge.
                return Ok(Atom::String(id));
            }
            // Reuse the already encoded units on a miss; intern still owns
            // canonical insertion, its duplicate check and admission budgets.
            return self.intern(JsString(units.to_vec()));
        }
        self.intern(JsString::from(s))
    }
    fn key(&self, a: &Atom) -> Key {
        match a {
            Atom::Undefined => Key::Undefined,
            Atom::Null => Key::Null,
            Atom::Bool(b) => Key::Bool(*b),
            Atom::Number(n) => Key::Number(if n.is_nan() {
                f64::NAN.to_bits()
            } else if *n == 0. {
                0
            } else {
                n.to_bits()
            }),
            Atom::String(id) => Key::String(self.canonical[*id as usize]),
            Atom::Ref(id) => Key::Ref(*id),
            Atom::BuiltinFunction(id) => Key::Function(*id),
            Atom::HostReference(id) => Key::Host(*id),
        }
    }
    fn property_position(&mut self, id: u32, key: u32) -> Result<Option<usize>, PublicError> {
        use std::collections::hash_map::Entry;
        let properties = self.g.nodes[id as usize].properties();
        let canonical = &self.canonical;
        let index = match self.properties.entry(id) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                charge_work(&mut self.work, properties.len())?;
                entry.insert(PropertyIndex::new(properties, canonical))
            }
        };
        Ok(index.position(key, properties, canonical))
    }
    fn own(&mut self, id: u32, key: &str) -> Result<Option<Atom>, PublicError> {
        self.tick(1)?;
        let mut buffer = [0; 64];
        let key = match JsString::short_utf16(key, &mut buffer) {
            Some(units) => self.strings.get(units),
            None => self.strings.get(&JsString::from(key)),
        };
        let Some(key) = key.copied() else {
            return Ok(None);
        };
        Ok(self.property_position(id, key)?.map(|position| {
            self.g.nodes[id as usize].properties()[position]
                .value
                .clone()
        }))
    }
    fn read(&mut self, a: &Atom, k: &str) -> Result<Atom, PublicError> {
        match a {
            Atom::Undefined | Atom::Null => Err(type_error(&format!(
                "Cannot read properties of {} (reading '{k}')",
                if matches!(a, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            ))),
            Atom::Ref(id) => {
                if k == "length" {
                    if let Node::Array { length, .. } = self.g.nodes[*id as usize] {
                        return Ok(Atom::Number(length as f64));
                    }
                }
                if let Some(value) = self.own(*id, k)? {
                    return Ok(value);
                }
                if k == "size" {
                    if let Node::Map { entries, .. } = &self.g.nodes[*id as usize] {
                        return Ok(Atom::Number(entries.len() as f64));
                    }
                }
                Ok(Atom::Undefined)
            }
            Atom::String(id) => {
                let units = &self.g.strings[*id as usize].0;
                if k == "length" {
                    return Ok(Atom::Number(units.len() as f64));
                }
                if let Some(n) = k.parse::<usize>().ok().filter(|n| n.to_string() == k) {
                    if let Some(c) = units.get(n).copied() {
                        return self.intern(JsString(vec![c]));
                    }
                }
                Ok(Atom::Undefined)
            }
            _ => Ok(Atom::Undefined),
        }
    }
    fn set(&mut self, a: &Atom, k: &str, v: Atom) -> Result<(), PublicError> {
        let Atom::Ref(id) = a else {
            if matches!(a, Atom::Null | Atom::Undefined) {
                return Err(type_error(&format!(
                    "Cannot set properties of {} (setting '{k}')",
                    if matches!(a, Atom::Null) {
                        "null"
                    } else {
                        "undefined"
                    }
                )));
            }
            let kind = match a {
                Atom::String(_) => "string",
                Atom::Bool(_) => "boolean",
                Atom::Number(_) => "number",
                _ => {
                    return Err(
                        invalid("Gantt host property receiver dispatch is unavailable").into(),
                    )
                }
            };
            let mut units = format!("Cannot create property '{k}' on {kind} '")
                .encode_utf16()
                .collect::<Vec<_>>();
            units.extend(self.coercion(a)?);
            units.push(39);
            return Err(PublicError::JavaScript {
                name: JavaScriptErrorKind::TypeError,
                message: JsString(units),
            });
        };
        self.tick(1)?;
        let Atom::String(key) = self.text(k)? else {
            unreachable!()
        };
        let at = self.property_position(*id, key)?;
        if let Some(at) = at {
            if self.g.nodes[*id as usize].properties()[at].flags & 2 == 0 {
                let label = if matches!(
                    self.g.nodes[*id as usize],
                    Node::Instance {
                        prototype: PROTO,
                        ..
                    }
                ) {
                    // Pinned npm 1.6.0 constructor label in native assignment errors.
                    "#<ac>"
                } else {
                    "#<Object>"
                };
                return Err(type_error(&format!(
                    "Cannot assign to read only property '{k}' of object '{label}'"
                )));
            }
            self.g.nodes[*id as usize].properties_mut()[at].value = v;
        } else {
            self.room(1)?;
            let index = self.g.nodes[*id as usize].properties().len();
            self.properties.get_mut(id).unwrap().insert(
                key,
                index,
                self.g.nodes[*id as usize].properties(),
                &self.canonical,
            );
            self.g.nodes[*id as usize].properties_mut().push(Property {
                key,
                value: v,
                flags: 7,
            });
        }
        self.changed(*id);
        Ok(())
    }
    fn node(&mut self, n: Node) -> Result<Atom, PublicError> {
        self.room(
            n.properties().len()
                + match &n {
                    Node::Set { values, .. } => values.len(),
                    Node::Map { entries, .. } => entries.len().saturating_mul(2),
                    _ => 0,
                },
        )?;
        let id = self.g.nodes.len() as u32;
        self.g.nodes.push(n);
        self.changed(id);
        Ok(Atom::Ref(id))
    }
    fn record(&mut self, fields: Vec<(&str, Atom)>) -> Result<Atom, PublicError> {
        let a = self.node(Node::Object { properties: vec![] })?;
        for (k, v) in fields {
            self.set(&a, k, v)?
        }
        Ok(a)
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<Atom, PublicError> {
        let a = self.node(Node::Array {
            length: 0,
            properties: vec![],
        })?;
        for v in values {
            self.push(&a, v, "array.push")?
        }
        Ok(a)
    }
    fn native(&mut self, a: &Atom, p: &str, expr: &str, kind: u8) -> Result<u32, PublicError> {
        let found = self.read(a, p)?;
        if matches!(found, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            return Err(invalid(&format!(
                "Gantt overridden {expr} callback dispatch is unavailable"
            ))
            .into());
        }
        let Atom::Ref(id) = a else {
            return Err(type_error(&format!("{expr} is not a function")));
        };
        let good = matches!(
            (&self.g.nodes[*id as usize], kind),
            (Node::Map { .. }, 0) | (Node::Array { .. }, 1) | (Node::Set { .. }, 2)
        );
        if !good || self.own(*id, p)?.is_some() {
            return Err(type_error(&format!("{expr} is not a function")));
        }
        Ok(*id)
    }
    fn map_index(&mut self, id: u32) -> Result<(), PublicError> {
        if !self.maps.contains_key(&id) {
            let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
                unreachable!()
            };
            let entries = entries.clone();
            self.tick(entries.len())?;
            let mut index = HashMap::new();
            for (n, (k, _)) in entries.iter().enumerate() {
                index.insert(self.key(k), n);
            }
            self.maps.insert(id, index);
        }
        Ok(())
    }
    fn map_get(&mut self, a: &Atom, k: &Atom, has: bool, expr: &str) -> Result<Atom, PublicError> {
        let id = self.native(a, if has { "has" } else { "get" }, expr, 0)?;
        self.map_index(id)?;
        let at = self.maps[&id].get(&self.key(k));
        let Node::Map { entries, .. } = &self.g.nodes[id as usize] else {
            unreachable!()
        };
        Ok(if has {
            Atom::Bool(at.is_some())
        } else {
            at.map_or(Atom::Undefined, |n| entries[*n].1.clone())
        })
    }
    fn map_set(&mut self, a: &Atom, k: Atom, v: Atom, expr: &str) -> Result<(), PublicError> {
        let id = self.native(a, "set", expr, 0)?;
        self.map_index(id)?;
        let key = self.key(&k);
        let at = self.maps[&id].get(&key).copied();
        if at.is_none() {
            self.room(2)?
        }
        let Node::Map { entries, .. } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        if let Some(n) = at {
            entries[n].1 = v
        } else {
            let n = entries.len();
            entries.push((
                if matches!(k,Atom::Number(n) if n==0.) {
                    Atom::Number(0.)
                } else {
                    k
                },
                v,
            ));
            self.maps.get_mut(&id).unwrap().insert(key, n);
        }
        self.changed(id);
        Ok(())
    }
    fn set_index(&mut self, id: u32) -> Result<(), PublicError> {
        if !self.sets.contains_key(&id) {
            let Node::Set { values, .. } = &self.g.nodes[id as usize] else {
                unreachable!()
            };
            let values = values.clone();
            self.tick(values.len())?;
            self.sets
                .insert(id, values.iter().map(|a| self.key(a)).collect());
        }
        Ok(())
    }
    fn set_add(&mut self, a: &Atom, k: Atom) -> Result<(), PublicError> {
        let id = self.native(a, "add", "tagged.add", 2)?;
        self.set_index(id)?;
        let key = self.key(&k);
        if !self.sets[&id].contains(&key) {
            self.room(1)?;
            self.sets.get_mut(&id).unwrap().insert(key);
            let Node::Set { values, .. } = &mut self.g.nodes[id as usize] else {
                unreachable!()
            };
            values.push(if matches!(k,Atom::Number(n) if n==0.) {
                Atom::Number(0.)
            } else {
                k
            });
        }
        self.changed(id);
        Ok(())
    }
    fn push(&mut self, a: &Atom, v: Atom, expr: &str) -> Result<(), PublicError> {
        let id = self.native(a, "push", expr, 1)?;
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            unreachable!()
        };
        if let Some(Atom::Number(_)) = self.own(id, "length")? {
            let p = self.g.nodes[id as usize]
                .properties()
                .iter()
                .find(|p| self.g.strings[p.key as usize].is("length"))
                .unwrap();
            if p.flags & 2 == 0 {
                return Err(type_error(
                    "Cannot assign to read only property 'length' of object '[object Array]'",
                ));
            }
        }
        if length >= 1_000_000 {
            return Err(limit().into());
        }
        self.set(a, &length.to_string(), v)?;
        if let Node::Array {
            length: len,
            properties,
        } = &mut self.g.nodes[id as usize]
        {
            *len = length + 1;
            for p in properties {
                if self.g.strings[p.key as usize].is("length") {
                    p.value = Atom::Number(*len as f64)
                }
            }
        }
        Ok(())
    }
    fn iter(&mut self, a: &Atom) -> Result<Iter, PublicError> {
        match a {
            Atom::Ref(id) => match self.g.nodes[*id as usize] {
                Node::Array { .. } => Ok(Iter::Array(*id, 0)),
                Node::Set { .. } => Ok(Iter::Set(*id, 0)),
                Node::Map { .. } => Ok(Iter::Map(*id, 0)),
                _ => Err(type_error("value is not iterable")),
            },
            Atom::String(id) => Ok(Iter::String(*id, 0)),
            _ => Err(type_error("value is not iterable")),
        }
    }
    fn next(&mut self, it: &mut Iter) -> Result<Option<Atom>, PublicError> {
        self.tick(1)?;
        match it {
            Iter::Array(id, n) => {
                let Node::Array { length, .. } = self.g.nodes[*id as usize] else {
                    unreachable!()
                };
                if *n >= length {
                    return Ok(None);
                }
                let a = self.read(&Atom::Ref(*id), &n.to_string())?;
                *n += 1;
                Ok(Some(a))
            }
            Iter::Map(id, n) => {
                let Node::Map { entries, .. } = &self.g.nodes[*id as usize] else {
                    unreachable!()
                };
                let value = entries.get(*n).cloned();
                *n += 1;
                value.map(|(k, v)| self.array(vec![k, v])).transpose()
            }
            Iter::Set(id, n) => {
                let Node::Set { values, .. } = &self.g.nodes[*id as usize] else {
                    unreachable!()
                };
                let v = values.get(*n).cloned();
                *n += 1;
                Ok(v)
            }
            Iter::String(id, n) => {
                let s = &self.g.strings[*id as usize].0;
                if *n >= s.len() {
                    return Ok(None);
                }
                let count = if (0xd800..=0xdbff).contains(&s[*n])
                    && s.get(*n + 1).is_some_and(|c| (0xdc00..=0xdfff).contains(c))
                {
                    2
                } else {
                    1
                };
                let t = JsString(s[*n..*n + count].to_vec());
                *n += count;
                self.intern(t).map(Some)
            }
        }
    }
    fn string_units(&mut self, a: &Atom, expr: &str) -> Result<Vec<u16>, PublicError> {
        if let Atom::String(id) = a {
            self.tick(self.g.strings[*id as usize].0.len())?;
            Ok(self.g.strings[*id as usize].0.clone())
        } else {
            let property = expr.rsplit('.').next().unwrap_or(expr);
            self.read(a, property)?;
            Err(type_error(&format!("{expr} is not a function")))
        }
    }
    fn coercion(&mut self, a: &Atom) -> Result<Vec<u16>, PublicError> {
        match a {
            Atom::String(id) => {
                self.tick(self.g.strings[*id as usize].0.len())?;
                Ok(self.g.strings[*id as usize].0.clone())
            }
            Atom::Undefined => Ok("undefined".encode_utf16().collect()),
            Atom::Null => Ok("null".encode_utf16().collect()),
            Atom::Bool(b) => Ok(if *b { "true" } else { "false" }.encode_utf16().collect()),
            Atom::Number(n) => Ok(crate::compat::svg::js_num(*n).encode_utf16().collect()),
            _ => Err(invalid("Gantt object primitive-conversion dispatch is unavailable").into()),
        }
    }
}
#[derive(Clone)]
enum DateValue {
    Existing(Atom),
    Fresh(f64),
}
struct Arena<'a> {
    c: Context<'a>,
    receiver: Atom,
    direct_entry: Option<&'static str>,
}
impl Context<'_> {
    fn scalar(&mut self, a: &Atom, expr: &str) -> Result<String, PublicError> {
        let units = self.string_units(a, expr)?;
        String::from_utf16(&units).map_err(|_|invalid("Gantt calendar parsing of lone UTF16 surrogates requires scalar calendar transport").into())
    }
    fn numeric(&mut self, a: &Atom) -> Result<f64, PublicError> {
        Ok(match a {
            Atom::Undefined => f64::NAN,
            Atom::Null => 0.,
            Atom::Bool(v) => u8::from(*v) as f64,
            Atom::Number(n) => *n,
            Atom::String(id) => {
                let Some(text) = self.g.strings[*id as usize].well_formed() else {
                    return Ok(f64::NAN);
                };
                let text = text.trim_matches(crate::compat::source::space);
                if text.is_empty() {
                    0.
                } else if matches!(text, "Infinity" | "+Infinity") {
                    f64::INFINITY
                } else if text == "-Infinity" {
                    f64::NEG_INFINITY
                } else if let Some((digits, radix)) = text
                    .strip_prefix("0x")
                    .or_else(|| text.strip_prefix("0X"))
                    .map(|s| (s, 16))
                    .or_else(|| {
                        text.strip_prefix("0o")
                            .or_else(|| text.strip_prefix("0O"))
                            .map(|s| (s, 8))
                    })
                    .or_else(|| {
                        text.strip_prefix("0b")
                            .or_else(|| text.strip_prefix("0B"))
                            .map(|s| (s, 2))
                    })
                {
                    if digits.is_empty() {
                        f64::NAN
                    } else {
                        let mut n = 0.;
                        for c in digits.chars() {
                            let Some(d) = c.to_digit(radix) else {
                                return Ok(f64::NAN);
                            };
                            n = n * radix as f64 + d as f64;
                        }
                        n
                    }
                } else if text
                    .chars()
                    .all(|c| matches!(c, '0'..='9' | 'e' | 'E' | '+' | '-' | '.'))
                {
                    text.parse().unwrap_or(f64::NAN)
                } else {
                    f64::NAN
                }
            }
            Atom::Ref(id) => {
                if own(self.g, *id, "valueOf").is_some() || own(self.g, *id, "toString").is_some() {
                    return Err(invalid(
                        "Gantt overridden Date primitive conversion requires callback dispatch",
                    )
                    .into());
                }
                match self.g.nodes[*id as usize] {
                    Node::Date { milliseconds, .. } => milliseconds,
                    _ => {
                        return Err(invalid(
                            "Gantt object numeric conversion requires callback dispatch",
                        )
                        .into())
                    }
                }
            }
            _ => {
                return Err(
                    invalid("Gantt host numeric conversion requires callback dispatch").into(),
                )
            }
        })
    }
    fn values(&mut self, a: &Atom) -> Result<Vec<Atom>, PublicError> {
        let mut it = self.iter(a)?;
        let mut values = vec![];
        while let Some(v) = self.next(&mut it)? {
            values.push(v)
        }
        Ok(values)
    }
    fn date_value(&mut self, value: DateValue) -> Result<Atom, PublicError> {
        match value {
            DateValue::Existing(a) => Ok(a),
            DateValue::Fresh(milliseconds) => self.node(Node::Date {
                milliseconds: super::dates::clip(milliseconds),
                properties: vec![],
            }),
        }
    }
}
impl Arena<'_> {
    fn get(&mut self, key: &str) -> Result<Atom, PublicError> {
        self.c.read(&self.receiver, key)
    }
    fn set(&mut self, key: &str, value: Atom) -> Result<(), PublicError> {
        self.c.set(&self.receiver, key, value)
    }
    fn spend_atom(&mut self, units: Atom) -> Result<f64, PublicError> {
        let prior = self.get("work")?;
        let value = if matches!(prior, Atom::String(_)) || matches!(units, Atom::String(_)) {
            let mut text = self.c.coercion(&prior)?;
            text.extend(self.c.coercion(&units)?);
            self.c.intern(JsString(text))?
        } else {
            let a = self.c.numeric(&prior)?;
            let b = self.c.numeric(&units)?;
            Atom::Number(a + b)
        };
        self.set("work", value.clone())?;
        self.c.numeric(&value)
    }
    fn get_number(&mut self, key: &str) -> Result<f64, PublicError> {
        let value = self.get(key)?;
        self.c.numeric(&value)
    }
    fn scalar_field(&mut self, key: &str) -> Result<String, PublicError> {
        let value = self.get(key)?;
        self.c.scalar(&value, &format!("this.{key}.trim"))
    }
}
impl Store for Arena<'_> {
    type Task = Atom;
    type Date = DateValue;
    type Token = Atom;
    type Error = PublicError;
    fn method(&mut self, name: &str) -> Result<(), PublicError> {
        if self.direct_entry == Some(name) {
            self.direct_entry = None;
            Ok(())
        } else {
            self.native_method(name)
        }
    }
    fn now(&mut self) -> Result<f64, PublicError> {
        self.get_number("now")
    }
    fn set_now(&mut self, n: f64) -> Result<(), PublicError> {
        self.set("now", Atom::Number(n))
    }
    fn add_work(&mut self, units: f64) -> Result<f64, PublicError> {
        self.spend_atom(Atom::Number(units))
    }
    fn work(&mut self) -> Result<f64, PublicError> {
        self.get_number("work")
    }
    fn set_work(&mut self, n: f64) -> Result<(), PublicError> {
        self.set("work", Atom::Number(n))
    }
    fn tokens(&mut self, include: bool) -> Result<Vec<Atom>, PublicError> {
        let a = self.get(if include { "includes" } else { "excludes" })?;
        self.c.values(&a)
    }
    fn token_is(&self, token: &Atom, text: &str) -> bool {
        matches!(token,Atom::String(id)if self.c.g.strings[*id as usize].is(text))
    }
    fn token_length(&mut self, include: bool) -> Result<f64, PublicError> {
        let a = self.get(if include { "includes" } else { "excludes" })?;
        let length = self.c.read(&a, "length")?;
        self.c.numeric(&length)
    }
    fn set_tokens(&mut self, include: bool, tokens: Vec<Atom>) -> Result<(), PublicError> {
        let set = self.c.node(Node::Set {
            values: vec![],
            properties: vec![],
        })?;
        for token in tokens {
            self.c.set_add(&set, token)?;
        }
        self.set(if include { "includeSet" } else { "excludeSet" }, set)
    }
    fn contains(&mut self, include: bool, text: &str) -> Result<bool, PublicError> {
        let set = self.get(if include { "includeSet" } else { "excludeSet" })?;
        let id = self.c.native(&set, "has", "set.has", 2)?;
        self.c.set_index(id)?;
        // A Set lookup does not publish its temporary formatted string. If no
        // equal string is interned, no existing Set entry can equal this key.
        let Some(value) = self.c.strings.get(&JsString::from(text)).copied() else {
            return Ok(false);
        };
        Ok(self.c.sets[&id].contains(&Key::String(value)))
    }
    fn weekend_start(&mut self) -> Result<Option<usize>, PublicError> {
        let value = self.get("weekend")?;
        Ok(if self.token_is(&value, "friday") {
            Some(5)
        } else if self.token_is(&value, "saturday") {
            Some(6)
        } else {
            None
        })
    }
    fn set_excluded_days(&mut self, days: Vec<bool>) -> Result<(), PublicError> {
        let days = self.c.array(days.into_iter().map(Atom::Bool).collect())?;
        self.set("excludedDays", days)
    }
    fn excluded_day(&mut self, day: Option<usize>) -> Result<Option<bool>, PublicError> {
        let days = self.get("excludedDays")?;
        let key = day.map_or("NaN".into(), |day| day.to_string());
        let value = self.c.read(&days, &key)?;
        Ok(if matches!(value, Atom::Undefined) {
            None
        } else {
            Some(truth(self.c.g, &value))
        })
    }

    fn dated_tokens(&mut self) -> Result<bool, PublicError> {
        let value = self.get("datedTokens")?;
        Ok(truth(self.c.g, &value))
    }
    fn set_dated_tokens(&mut self, value: bool) -> Result<(), PublicError> {
        self.set("datedTokens", Atom::Bool(value))
    }
    fn date_format(&mut self) -> Result<String, PublicError> {
        self.scalar_field("dateFormat")
    }
    fn inclusive(&mut self) -> Result<bool, PublicError> {
        let value = self.get("inclusiveEndDates")?;
        Ok(truth(self.c.g, &value))
    }
    fn tasks(&mut self) -> Result<Vec<Atom>, PublicError> {
        let value = self.get("tasks")?;
        self.c.values(&value)
    }
    fn previous(&mut self, task: &Atom) -> Result<Option<Atom>, PublicError> {
        let id = self.c.read(task, "prevTaskId")?;
        if matches!(id, Atom::Undefined) {
            return Ok(None);
        }
        let map = self.get("byId")?;
        let value = self.c.map_get(&map, &id, false, "this.byId.get")?;
        Ok(truth(self.c.g, &value).then_some(value))
    }
    fn lookup(&mut self, id: &str) -> Result<Option<Atom>, PublicError> {
        let key = self.c.text(id)?;
        let map = self.get("byId")?;
        let value = self.c.map_get(&map, &key, false, "this.byId.get")?;
        Ok(truth(self.c.g, &value).then_some(value))
    }
    fn raw_start(&mut self, task: &Atom) -> Result<Option<String>, PublicError> {
        let raw = self.c.read(task, "raw")?;
        let value = self.c.read(&raw, "start")?;
        if matches!(value, Atom::Undefined) {
            Ok(None)
        } else {
            self.c.scalar(&value, "raw.trim").map(Some)
        }
    }
    fn raw_end(&mut self, task: &Atom) -> Result<String, PublicError> {
        let raw = self.c.read(task, "raw")?;
        let value = self.c.read(&raw, "end")?;
        self.c.scalar(&value, "raw.trim")
    }
    fn time(&mut self, task: &Atom, field: Time) -> Result<Option<DateValue>, PublicError> {
        let value = self.c.read(task, time_key(field))?;
        Ok(if matches!(value, Atom::Undefined) {
            None
        } else {
            Some(DateValue::Existing(value))
        })
    }
    fn set_time(
        &mut self,
        task: &Atom,
        field: Time,
        value: Option<DateValue>,
    ) -> Result<(), PublicError> {
        let value = if let Some(value) = value {
            self.c.date_value(value)?
        } else if matches!(field, Time::RenderEnd) {
            Atom::Null
        } else {
            Atom::Undefined
        };
        self.c.set(task, time_key(field), value)
    }
    fn date(&mut self, ms: f64) -> Result<DateValue, PublicError> {
        Ok(DateValue::Fresh(super::dates::clip(ms)))
    }
    fn milliseconds(&mut self, date: &DateValue) -> Result<f64, PublicError> {
        match date {
            DateValue::Fresh(ms) => Ok(*ms),
            DateValue::Existing(value) => self.c.numeric(value),
        }
    }
    fn date_truth(&self, date: &DateValue) -> bool {
        match date {
            DateValue::Fresh(_) => true,
            DateValue::Existing(value) => truth(self.c.g, value),
        }
    }
    fn manual(&mut self, task: &Atom) -> Result<bool, PublicError> {
        let value = self.c.read(task, "manualEndTime")?;
        Ok(truth(self.c.g, &value))
    }
    fn processed(&mut self, task: &Atom) -> Result<bool, PublicError> {
        let value = self.c.read(task, "processed")?;
        Ok(truth(self.c.g, &value))
    }
    fn set_flag(&mut self, task: &Atom, flag: Flag, value: bool) -> Result<(), PublicError> {
        self.c.set(
            task,
            match flag {
                Flag::Processed => "processed",
                Flag::ManualEnd => "manualEndTime",
            },
            Atom::Bool(value),
        )
    }
    fn warn(&mut self, _message: String) -> Result<(), PublicError> {
        let options = self.get("options")?;
        let callback = self.c.read(&options, "warn")?;
        if truth(self.c.g, &callback) {
            return Err(invalid("Gantt options.warn requires callback dispatch").into());
        }
        Ok(())
    }
}
fn time_key(field: Time) -> &'static str {
    match field {
        Time::Start => "startTime",
        Time::End => "endTime",
        Time::RenderEnd => "renderEndTime",
    }
}

const METHODS: &[MethodSchema] = &[
    MethodSchema {
        id: MethodId(1),
        name: "setDateFormat",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(2),
        name: "enableInclusiveEndDates",
        arity: 0,
    },
    MethodSchema {
        id: MethodId(3),
        name: "TopAxis",
        arity: 0,
    },
    MethodSchema {
        id: MethodId(4),
        name: "setAxisFormat",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(5),
        name: "setTickInterval",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(6),
        name: "setTodayMarker",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(7),
        name: "setIncludes",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(8),
        name: "setExcludes",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(9),
        name: "setWeekday",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(10),
        name: "setWeekend",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(11),
        name: "setDiagramTitle",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(12),
        name: "setAccTitle",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(13),
        name: "setAccDescription",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(14),
        name: "addSection",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(15),
        name: "addTask",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(16),
        name: "setLink",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(17),
        name: "setClickEvent",
        arity: 3,
    },
    MethodSchema {
        id: MethodId(18),
        name: "setClass",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(19),
        name: "getTasks",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(20),
        name: "isInvalidDate",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(21),
        name: "spend",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(22),
        name: "parse",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(23),
        name: "today",
        arity: 0,
    },
    MethodSchema {
        id: MethodId(24),
        name: "warn",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(25),
        name: "compileTasks",
        arity: 0,
    },
    MethodSchema {
        id: MethodId(26),
        name: "compileTask",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(27),
        name: "references",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(28),
        name: "getStartDate",
        arity: 1,
    },
    MethodSchema {
        id: MethodId(29),
        name: "checked",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(30),
        name: "getEndDate",
        arity: 2,
    },
    MethodSchema {
        id: MethodId(31),
        name: "checkTaskDates",
        arity: 1,
    },
];
const MEMBERS: &[MemberSchema] = &[
    MemberSchema {
        name: "setDateFormat",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(1),
        },
    },
    MemberSchema {
        name: "enableInclusiveEndDates",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(2),
        },
    },
    MemberSchema {
        name: "TopAxis",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(3),
        },
    },
    MemberSchema {
        name: "setAxisFormat",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(4),
        },
    },
    MemberSchema {
        name: "setTickInterval",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(5),
        },
    },
    MemberSchema {
        name: "setTodayMarker",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(6),
        },
    },
    MemberSchema {
        name: "setIncludes",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(7),
        },
    },
    MemberSchema {
        name: "setExcludes",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(8),
        },
    },
    MemberSchema {
        name: "setWeekday",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(9),
        },
    },
    MemberSchema {
        name: "setWeekend",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(10),
        },
    },
    MemberSchema {
        name: "setDiagramTitle",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(11),
        },
    },
    MemberSchema {
        name: "setAccTitle",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(12),
        },
    },
    MemberSchema {
        name: "setAccDescription",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(13),
        },
    },
    MemberSchema {
        name: "addSection",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(14),
        },
    },
    MemberSchema {
        name: "addTask",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(15),
        },
    },
    MemberSchema {
        name: "setLink",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(16),
        },
    },
    MemberSchema {
        name: "setClickEvent",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(17),
        },
    },
    MemberSchema {
        name: "setClass",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(18),
        },
    },
    MemberSchema {
        name: "getTasks",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(19),
        },
    },
    MemberSchema {
        name: "isInvalidDate",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(20),
        },
    },
    MemberSchema {
        name: "spend",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(21),
        },
    },
    MemberSchema {
        name: "parse",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(22),
        },
    },
    MemberSchema {
        name: "today",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(23),
        },
    },
    MemberSchema {
        name: "warn",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(24),
        },
    },
    MemberSchema {
        name: "compileTasks",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(25),
        },
    },
    MemberSchema {
        name: "compileTask",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(26),
        },
    },
    MemberSchema {
        name: "references",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(27),
        },
    },
    MemberSchema {
        name: "getStartDate",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(28),
        },
    },
    MemberSchema {
        name: "checked",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(29),
        },
    },
    MemberSchema {
        name: "getEndDate",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(30),
        },
    },
    MemberSchema {
        name: "checkTaskDates",
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(31),
        },
    },
];
const ROOT_FIELDS: &[FieldSchema] = &[
    FieldSchema {
        name: "type",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "title",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "accTitle",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "accDescr",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "dateFormat",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "axisFormat",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "tickInterval",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "todayMarker",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "includes",
        kind: FieldKind::Array,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "excludes",
        kind: FieldKind::Array,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "inclusiveEndDates",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "topAxis",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "displayMode",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "weekday",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "weekend",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "sections",
        kind: FieldKind::Array,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "links",
        kind: FieldKind::Map,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "tasks",
        kind: FieldKind::Array,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "byId",
        kind: FieldKind::Map,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "currentSection",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "taskCount",
        kind: FieldKind::Number,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "lastOrder",
        kind: FieldKind::Number,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "lastTaskId",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "options",
        kind: FieldKind::Any,
        optional: false,
        undefined: true,
        nullable: true,
        flags: 7,
    },
    FieldSchema {
        name: "now",
        kind: FieldKind::Number,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "work",
        kind: FieldKind::Number,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "includeSet",
        kind: FieldKind::Set,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "excludeSet",
        kind: FieldKind::Set,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "excludedDays",
        kind: FieldKind::Array,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "datedTokens",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
];
const TASK_FIELDS: &[FieldSchema] = &[
    FieldSchema {
        name: "id",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "task",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "section",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "startTime",
        kind: FieldKind::Date,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "endTime",
        kind: FieldKind::Date,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "renderEndTime",
        kind: FieldKind::Date,
        optional: false,
        undefined: false,
        nullable: true,
        flags: 7,
    },
    FieldSchema {
        name: "manualEndTime",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "processed",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "active",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "done",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "crit",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "milestone",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "vert",
        kind: FieldKind::Bool,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "order",
        kind: FieldKind::Number,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "classes",
        kind: FieldKind::Array,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "prevTaskId",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "raw",
        kind: FieldKind::Record(RecordId(3)),
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
];
const RAW_FIELDS: &[FieldSchema] = &[
    FieldSchema {
        name: "data",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "start",
        kind: FieldKind::String,
        optional: false,
        undefined: true,
        nullable: false,
        flags: 7,
    },
    FieldSchema {
        name: "end",
        kind: FieldKind::String,
        optional: false,
        undefined: false,
        nullable: false,
        flags: 7,
    },
];
const SCHEMA: ModelSchema = ModelSchema {
    id: "pele/gantt@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "GanttDb",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: ROOT_FIELDS,
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "GanttTask",
            kind: RecordKind::Object,
            prototype: None,
            fields: TASK_FIELDS,
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "GanttTaskRaw",
            kind: RecordKind::Object,
            prototype: None,
            fields: RAW_FIELDS,
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "Date",
            kind: RecordKind::Date,
            prototype: None,
            fields: &[],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "GanttDb",
        parent: PrototypeParent::Object,
        members: MEMBERS,
    }],
    methods: METHODS,
};
// Construction-only token: the fresh object cannot be finished twice or used
// as an imported receiver. Values/child nodes may be created before finishing
// the root, preserving the canonical graph's existing allocation order.
struct InitialObject(u32);
impl Context<'_> {
    fn initial_object(
        &mut self,
        prototype: Option<PrototypeId>,
    ) -> Result<InitialObject, PublicError> {
        let node = match prototype {
            Some(prototype) => Node::Instance {
                prototype,
                properties: vec![],
            },
            None => Node::Object { properties: vec![] },
        };
        let Atom::Ref(id) = self.node(node)? else {
            unreachable!()
        };
        Ok(InitialObject(id))
    }
    fn finish_initial_object(
        &mut self,
        object: InitialObject,
        fields: impl ExactSizeIterator<Item = (&'static str, Atom)>,
    ) -> Result<Atom, PublicError> {
        let id = object.0;
        debug_assert!(self.g.nodes[id as usize].properties().is_empty());
        // Fixed small construction records. Keep duplicate assignment semantics
        // without allocating the general imported-object property HashMap.
        self.g.nodes[id as usize]
            .properties_mut()
            .reserve(fields.len());
        for (name, value) in fields {
            self.tick(1)?;
            let Atom::String(key) = self.text(name)? else {
                unreachable!()
            };
            let previous = self.g.nodes[id as usize]
                .properties()
                .iter()
                .position(|property| property.key == key);
            if let Some(previous) = previous {
                self.g.nodes[id as usize].properties_mut()[previous].value = value;
            } else {
                self.room(1)?;
                self.g.nodes[id as usize].properties_mut().push(Property {
                    key,
                    value,
                    flags: 7,
                });
            }
            self.changed(id);
        }
        Ok(Atom::Ref(id))
    }
    fn initial_record(
        &mut self,
        fields: impl ExactSizeIterator<Item = (&'static str, Atom)>,
    ) -> Result<Atom, PublicError> {
        let object = self.initial_object(None)?;
        self.finish_initial_object(object, fields)
    }
    fn initial_array(&mut self, values: Vec<Atom>) -> Result<Atom, PublicError> {
        let array = self.node(Node::Array {
            length: 0,
            properties: vec![],
        })?;
        let Atom::Ref(id) = array else { unreachable!() };
        for value in values {
            self.initial_push(id, value)?;
        }
        Ok(Atom::Ref(id))
    }
}
impl Context<'_> {
    fn initial_strings_array(&mut self, values: &[String]) -> Result<Atom, PublicError> {
        let values = values
            .iter()
            .map(|s| self.text(s))
            .collect::<Result<_, _>>()?;
        self.initial_array(values)
    }
    fn optional_text(&mut self, value: Option<&str>) -> Result<Atom, PublicError> {
        value.map_or(Ok(Atom::Undefined), |s| self.text(s))
    }
    fn task_record(&mut self, t: &GanttTask) -> Result<Atom, PublicError> {
        let raw_data = self.text(&t.raw.data)?;
        let raw_start = self.optional_text(t.raw.start.as_deref())?;
        let raw_end = self.text(&t.raw.end)?;
        let raw = self.initial_record(
            [("data", raw_data), ("start", raw_start), ("end", raw_end)].into_iter(),
        )?;
        // Evaluate child values in canonical arena order before creating the
        // task object. Fixed records need no temporary growable field vector.
        let id = self.text(&t.id)?;
        let task = self.text(&t.task)?;
        let section = self.text(&t.section)?;
        let start = match t.start_time {
            Some(ms) => self.date_value(DateValue::Fresh(ms))?,
            None => Atom::Undefined,
        };
        let end = match t.end_time {
            Some(ms) => self.date_value(DateValue::Fresh(ms))?,
            None => Atom::Undefined,
        };
        let render_end = match t.render_end_time {
            Some(ms) => self.date_value(DateValue::Fresh(ms))?,
            None => Atom::Null,
        };
        let fields = [
            ("id", id),
            ("task", task),
            ("section", section),
            ("startTime", start),
            ("endTime", end),
            ("renderEndTime", render_end),
            ("manualEndTime", Atom::Bool(t.manual_end_time)),
            ("processed", Atom::Bool(t.processed)),
            ("active", Atom::Bool(t.active)),
            ("done", Atom::Bool(t.done)),
            ("crit", Atom::Bool(t.crit)),
            ("milestone", Atom::Bool(t.milestone)),
            ("vert", Atom::Bool(t.vert)),
            ("order", Atom::Number(t.order as f64)),
            ("classes", self.initial_strings_array(&t.classes)?),
            ("prevTaskId", self.optional_text(t.prev_task_id.as_deref())?),
            ("raw", raw),
        ];
        self.initial_record(fields.into_iter())
    }
}
fn initial(m: &GanttModel) -> Result<ModelGraph, PeleError> {
    let mut graph = ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec![],
        nodes: vec![],
    };
    let result = (|| -> Result<(), PublicError> {
        let mut c = Context::new(&mut graph);
        let tasks = m
            .tasks
            .iter()
            .map(|t| c.task_record(t))
            .collect::<Result<Vec<_>, _>>()?;
        let by_id = m
            .by_id
            .iter()
            .map(|(id, n)| Ok((c.text(id)?, tasks[*n].clone())))
            .collect::<Result<Vec<_>, PublicError>>()?;
        let by_id = c.node(Node::Map {
            entries: by_id,
            properties: vec![],
        })?;
        let tasks = c.initial_array(tasks)?;
        let links = m
            .links
            .iter()
            .map(|(k, v)| Ok((c.text(k)?, c.text(v)?)))
            .collect::<Result<_, PublicError>>()?;
        let links = c.node(Node::Map {
            entries: links,
            properties: vec![],
        })?;
        finish_initial_root(&mut c, m, tasks, by_id, links, m.public_builder_state())?;
        Ok(())
    })();
    result.map_err(legacy_error)?;
    graph.validate_schema(&SCHEMA)?;
    Ok(graph)
}
fn finish_initial_root(
    c: &mut Context<'_>,
    m: &GanttModel,
    tasks: Atom,
    by_id: Atom,
    links: Atom,
    builder: (&str, usize, usize, Option<&str>),
) -> Result<(), PublicError> {
    let root = c.initial_object(Some(PROTO))?;
    let mut fields = Vec::with_capacity(ROOT_FIELDS.len());
    fields.push(("type", c.text("gantt")?));
    for (k, v) in [
        ("title", m.title.as_deref()),
        ("accTitle", m.acc_title.as_deref()),
        ("accDescr", m.acc_descr.as_deref()),
    ] {
        fields.push((k, c.optional_text(v)?));
    }
    fields.extend([
        ("dateFormat", c.text(&m.date_format)?),
        ("axisFormat", c.text(&m.axis_format)?),
        ("tickInterval", c.optional_text(m.tick_interval.as_deref())?),
        ("todayMarker", c.text(&m.today_marker)?),
        ("includes", c.initial_strings_array(&m.includes)?),
        ("excludes", c.initial_strings_array(&m.excludes)?),
        ("inclusiveEndDates", Atom::Bool(m.inclusive_end_dates)),
        ("topAxis", Atom::Bool(m.top_axis)),
        ("displayMode", c.text(&m.display_mode)?),
        ("weekday", c.optional_text(m.weekday.as_deref())?),
        ("weekend", c.text(&m.weekend)?),
        ("sections", c.initial_strings_array(&m.sections)?),
        ("links", links),
        ("tasks", tasks),
        ("byId", by_id),
    ]);
    let (section, count, order, last) = builder;
    let options = c.initial_record([].into_iter())?;
    fields.extend([
        ("currentSection", c.text(section)?),
        ("taskCount", Atom::Number(count as f64)),
        ("lastOrder", Atom::Number(order as f64)),
        ("lastTaskId", c.optional_text(last)?),
        ("options", options),
        ("now", Atom::Number(m.now)),
        ("work", Atom::Number(m.work)),
    ]);
    let (includes, excludes, days, dated) = m.public_schedule_cache();
    for (k, values) in [("includeSet", includes), ("excludeSet", excludes)] {
        let values = values.iter().map(|s| c.text(s)).collect::<Result<_, _>>()?;
        let set = c.node(Node::Set {
            values,
            properties: vec![],
        })?;
        fields.push((k, set));
    }
    let days = c.initial_array(days.into_iter().map(Atom::Bool).collect())?;
    fields.extend([("excludedDays", days), ("datedTokens", Atom::Bool(dated))]);
    let root = c.finish_initial_object(root, fields.into_iter())?;
    c.g.root = root;
    Ok(())
}
fn legacy_error(error: PublicError) -> PeleError {
    match error {
        PublicError::Pele(e) => e,
        other => invalid(&format!(
            "Gantt public operation requires structured exception transport: {other:?}"
        )),
    }
}
impl Arena<'_> {
    fn native_method(&mut self, name: &str) -> Result<(), PublicError> {
        let value = self.c.read(&self.receiver, name)?;
        let expected = METHODS.iter().find(|m| m.name == name).unwrap().id.0;
        if matches!(value,Atom::BuiltinFunction(id)if id==expected) {
            return Ok(());
        }
        if matches!(value, Atom::Undefined)
            && matches!(self.receiver,Atom::Ref(id)if matches!(self.c.g.nodes[id as usize],Node::Instance{prototype:PROTO,..})&&own(self.c.g,id,name).is_none())
        {
            return Ok(());
        }
        if matches!(value, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            return Err(invalid(&format!(
                "Gantt overridden {name} requires callback dispatch"
            ))
            .into());
        }
        Err(type_error(&format!("this.{name} is not a function")))
    }
    fn merge_tokens(&mut self, include: bool, input: &Atom) -> Result<(), PublicError> {
        let existing = self.get(if include { "includes" } else { "excludes" })?;
        let values = self.c.values(&existing)?;
        let mut unique = HashSet::new();
        let mut merged = vec![];
        for value in values {
            if unique.insert(self.c.key(&value)) {
                merged.push(value)
            }
        }
        let units = self.c.string_units(input, "txt.toLowerCase")?;
        // Lowercase each scalar segment, preserving isolated UTF16 code units.
        let mut lower = vec![];
        let mut scalar = vec![];
        let flush = |scalar: &mut Vec<u16>, out: &mut Vec<u16>| {
            if !scalar.is_empty() {
                let text = String::from_utf16(scalar).unwrap().to_lowercase();
                out.extend(text.encode_utf16());
                scalar.clear()
            }
        };
        for value in char::decode_utf16(units) {
            match value {
                Ok(c) => {
                    let mut buf = [0; 2];
                    scalar.extend_from_slice(c.encode_utf16(&mut buf));
                }
                Err(e) => {
                    flush(&mut scalar, &mut lower);
                    lower.push(e.unpaired_surrogate())
                }
            }
        }
        flush(&mut scalar, &mut lower);
        for token in lower
            .split(|c| space(*c) || *c == 44)
            .filter(|t| !t.is_empty())
        {
            let value = self.c.intern(JsString(token.to_vec()))?;
            if unique.insert(self.c.key(&value)) {
                merged.push(value)
            }
        }
        let result = self.c.array(merged)?;
        self.set(if include { "includes" } else { "excludes" }, result)
    }
    fn set_class(&mut self, ids: &Atom, class: &Atom) -> Result<(), PublicError> {
        let units = self.c.string_units(ids, "ids.split")?;
        for id in units.split(|c| *c == 44) {
            let key = self.c.intern(JsString(id.to_vec()))?;
            let map = self.get("byId")?;
            let task = self.c.map_get(&map, &key, false, "this.byId.get")?;
            if matches!(task, Atom::Undefined | Atom::Null) {
                continue;
            }
            let classes = self.c.read(&task, "classes")?;
            self.c
                .push(&classes, class.clone(), "this.byId.get(...)?.classes.push")?;
        }
        Ok(())
    }
    fn add_task(&mut self, descr: Atom, data: Atom) -> Result<(), PublicError> {
        let units = self.c.string_units(&data, "dataStr.startsWith")?;
        let units = units.strip_prefix(&[58]).unwrap_or(&units);
        let mut fields = units
            .split(|c| *c == 44)
            .map(|s| s.to_vec())
            .collect::<Vec<_>>();
        let section = self.get("currentSection")?;
        let previous = self.get("lastTaskId")?;
        let mut tags = HashSet::new();
        while let Some(first) = fields.first() {
            let tag = trimmed(first);
            let known = ["active", "done", "crit", "milestone", "vert"]
                .iter()
                .find(|name| tag.iter().copied().eq(name.encode_utf16()))
                .copied();
            if let Some(tag) = known {
                tags.insert(tag);
                fields.remove(0);
            } else {
                break;
            }
        }
        if fields.is_empty() || fields.len() > 3 {
            let text = self.c.string_units(&descr, "descr.trim")?;
            let mut message = "Task \"".encode_utf16().collect::<Vec<_>>();
            message.extend_from_slice(trimmed(&text));
            message.extend(
                if fields.is_empty() {
                    "\" needs a duration or an end date."
                } else {
                    "\" has too many fields. A task takes an id, a start, and an end or duration."
                }
                .encode_utf16(),
            );
            return Err(crate::compat::public::PublicPeleError {
                message: JsString(message),
                code: ErrorCode::Semantic,
                diagram_type: Some(JsString::from("gantt")),
                line: 0,
                column: 0,
                snippet: JsString(vec![]),
            }
            .into());
        }
        let fields = fields
            .iter()
            .map(|s| trimmed(s).to_vec())
            .collect::<Vec<_>>();
        let id = if fields.len() == 3 {
            self.c.intern(JsString(fields[0].clone()))?
        } else {
            let count = self.get_number("taskCount")? + 1.;
            self.set("taskCount", Atom::Number(count))?;
            self.c
                .text(&format!("task{}", crate::compat::svg::js_num(count)))?
        };
        let start = if fields.len() > 1 {
            self.c.intern(JsString(fields[fields.len() - 2].clone()))?
        } else {
            Atom::Undefined
        };
        let end = self.c.intern(JsString(fields.last().unwrap().clone()))?;
        let raw = self
            .c
            .record(vec![("data", data), ("start", start), ("end", end)])?;
        let classes = self.c.array(vec![])?;
        let order = if tags.contains("vert") {
            -1.
        } else {
            let order = self.get_number("lastOrder")?;
            self.set("lastOrder", Atom::Number(order + 1.))?;
            order
        };
        let task = self.c.record(vec![
            ("id", id.clone()),
            ("task", descr),
            ("section", section),
            ("startTime", Atom::Undefined),
            ("endTime", Atom::Undefined),
            ("renderEndTime", Atom::Null),
            ("manualEndTime", Atom::Bool(false)),
            ("processed", Atom::Bool(false)),
            ("active", Atom::Bool(tags.contains("active"))),
            ("done", Atom::Bool(tags.contains("done"))),
            ("crit", Atom::Bool(tags.contains("crit"))),
            ("milestone", Atom::Bool(tags.contains("milestone"))),
            ("vert", Atom::Bool(tags.contains("vert"))),
            ("order", Atom::Number(order)),
            ("classes", classes),
            ("prevTaskId", previous),
            ("raw", raw),
        ])?;
        let tasks = self.get("tasks")?;
        self.c.push(&tasks, task.clone(), "this.tasks.push")?;
        self.set("lastTaskId", id.clone())?;
        let by_id = self.get("byId")?;
        self.c.map_set(&by_id, id, task, "this.byId.set")
    }
    fn call(
        &mut self,
        id: u32,
        args: &[Atom],
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        self.direct_entry = METHODS.iter().find(|m| m.id.0 == id).map(|m| m.name);
        let a = args.first().cloned().unwrap_or(Atom::Undefined);
        let b = args.get(1).cloned().unwrap_or(Atom::Undefined);
        match id {
            1 | 4 | 5 | 6 | 9 | 10 | 11 => self.set(
                match id {
                    1 => "dateFormat",
                    4 => "axisFormat",
                    5 => "tickInterval",
                    6 => "todayMarker",
                    9 => "weekday",
                    10 => "weekend",
                    _ => "title",
                },
                a,
            )?,
            2 | 3 => self.set(
                if id == 2 {
                    "inclusiveEndDates"
                } else {
                    "topAxis"
                },
                Atom::Bool(true),
            )?,
            7 | 8 => self.merge_tokens(id == 7, &a)?,
            12 | 13 => {
                let units = self.c.string_units(&a, "txt.replace")?;
                let units = if id == 12 {
                    units
                        .iter()
                        .position(|c| !space(*c))
                        .map_or(vec![], |n| units[n..].to_vec())
                } else {
                    let mut out = vec![];
                    let mut n = 0;
                    while n < units.len() {
                        out.push(units[n]);
                        n += 1;
                        if units[n - 1] == 10 {
                            while n < units.len() && space(units[n]) {
                                n += 1
                            }
                        }
                    }
                    out
                };
                let value = self.c.intern(JsString(units))?;
                self.set(if id == 12 { "accTitle" } else { "accDescr" }, value)?;
            }
            14 => {
                self.set("currentSection", a.clone())?;
                let sections = self.get("sections")?;
                self.c.push(&sections, a, "this.sections.push")?;
            }
            15 => self.add_task(a, b)?,
            16 => {
                let link = self.c.scalar(&b, "link.replace")?;
                let href = crate::compat::source::url::sanitize_url(&decode_link(&link)?);
                let href = self.c.text(&href)?;
                let ids = self.c.string_units(&a, "ids.split")?;
                for key in ids.split(|c| *c == 44) {
                    let key = self.c.intern(JsString(key.to_vec()))?;
                    let by_id = self.get("byId")?;
                    if matches!(
                        self.c.map_get(&by_id, &key, true, "this.byId.has")?,
                        Atom::Bool(true)
                    ) {
                        let links = self.get("links")?;
                        self.c
                            .map_set(&links, key, href.clone(), "this.links.set")?;
                    }
                }
                self.native_method("setClass")?;
                let clickable = self.c.text("clickable")?;
                self.set_class(&a, &clickable)?;
            }
            17 => {
                self.native_method("setClass")?;
                let clickable = self.c.text("clickable")?;
                self.set_class(&a, &clickable)?;
            }
            18 => self.set_class(&a, &b)?,
            19 => {
                let number = self.c.numeric(&a)?;
                let now = if number.is_finite() {
                    number
                } else {
                    host.direct_now_millis().map_err(PublicError::from)?
                };
                let mut cal = Calendar::public(host);
                let result = scheduler::schedule(self, now, &mut cal);
                result?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return self.get("tasks");
            }
            20 => {
                if !matches!(&a,Atom::Ref(n)if matches!(self.c.g.nodes[*n as usize],Node::Date{..}))
                {
                    self.c.read(&a, "getDay")?;
                    return Err(type_error("date.getDay is not a function"));
                }
                let own_method = self.c.read(&a, "getDay")?;
                if !matches!(own_method, Atom::Undefined) {
                    return Err(
                        if matches!(
                            own_method,
                            Atom::HostReference(_) | Atom::BuiltinFunction(_)
                        ) {
                            invalid("Gantt overridden Date.getDay requires callback dispatch")
                                .into()
                        } else {
                            type_error("date.getDay is not a function")
                        },
                    );
                }
                let milliseconds = self.c.numeric(&a)?;
                let mut cal = Calendar::public(host);
                let key = cal
                    .fields(milliseconds)
                    .map_or("NaN".into(), |f| f.weekday.to_string());
                cal.check().map_err(PublicError::from)?;
                let days = self.get("excludedDays")?;
                let weekday = self.c.read(&days, &key)?;
                if !self.dated_tokens()? {
                    return Ok(weekday);
                }
                let value = scheduler::invalid(self, &DateValue::Existing(a), &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return Ok(if value == Some(true) && truth(self.c.g, &weekday) {
                    weekday
                } else {
                    value.map_or(Atom::Undefined, Atom::Bool)
                });
            }
            21 => {
                if self.spend_atom(a)? > 4e8 {
                    return Err(super::model::error(
                        "The gantt chart takes too much work to schedule.",
                        ErrorCode::Limit,
                    )
                    .into());
                }
            }
            22 => {
                let text = self.c.scalar(&a, "str.length")?;
                let template = self.c.scalar(&b, "template.length")?;
                let mut cal = Calendar::public(host);
                let date = scheduler::parse(self, &text, &template, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return date.map_or(Ok(Atom::Undefined), |date| self.c.date_value(date));
            }
            23 => {
                let mut cal = Calendar::public(host);
                let date = scheduler::today(self, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return self.c.date_value(date);
            }
            24 => {
                let options = self.get("options")?;
                let warn = self.c.read(&options, "warn")?;
                if truth(self.c.g, &warn) {
                    return Err(invalid(
                        "Gantt warning producer and options.warn require callback dispatch",
                    )
                    .into());
                }
            }
            25 => {
                let mut cal = Calendar::public(host);
                let all = scheduler::compile_tasks(self, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return Ok(Atom::Bool(all));
            }
            26 => {
                let mut cal = Calendar::public(host);
                scheduler::compile_task(self, &a, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
            }
            27 => {
                let keyword_units = self.c.coercion(&a)?;
                let keyword = String::from_utf16(&keyword_units).map_err(|_| {
                    invalid("Gantt UTF16 warning interpolation requires callback transport")
                })?;
                let list = self.c.scalar(&b, "list.split")?;
                let tasks = scheduler::references(self, &keyword, &list)?;
                return self.c.array(tasks);
            }
            28 => {
                let raw = self.c.scalar(&a, "raw.trim")?;
                let mut cal = Calendar::public(host);
                let date = scheduler::start(self, &raw, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return date.map_or(Ok(Atom::Undefined), |date| self.c.date_value(date));
            }
            29 => {
                let date = self.c.read(&a, "getTime")?;
                let Atom::Ref(n) = a else {
                    return Err(type_error("date.getTime is not a function"));
                };
                if !matches!(date, Atom::Undefined) {
                    return Err(invalid(
                        "Gantt overridden Date.getTime requires callback dispatch",
                    )
                    .into());
                }
                let Node::Date { milliseconds, .. } = self.c.g.nodes[n as usize] else {
                    return Err(type_error("date.getTime is not a function"));
                };
                if milliseconds.is_nan() {
                    let text = self.c.coercion(&b)?;
                    let mut message = "Invalid date:".encode_utf16().collect::<Vec<_>>();
                    message.extend(text);
                    return Err(crate::compat::public::PublicPeleError {
                        message: JsString(message),
                        code: ErrorCode::Semantic,
                        diagram_type: Some(JsString::from("gantt")),
                        line: 0,
                        column: 0,
                        snippet: JsString(vec![]),
                    }
                    .into());
                }
                return Ok(Atom::Ref(n));
            }
            30 => {
                let raw = self.c.scalar(&b, "raw.trim")?;
                let mut cal = Calendar::public(host);
                let date = scheduler::end(self, &DateValue::Existing(a), &raw, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
                return date.map_or(Ok(Atom::Undefined), |date| self.c.date_value(date));
            }
            31 => {
                let mut cal = Calendar::public(host);
                scheduler::check_dates(self, &a, &mut cal)?;
                if let Some(error) = cal.error {
                    return Err(error.into());
                }
            }
            _ => return Err(invalid("Unknown Gantt method").into()),
        }
        Ok(Atom::Undefined)
    }
}
impl ActiveModel for Arena<'_> {
    fn begin_sync(&mut self) -> Result<(), PublicError> {
        if self.c.sync.is_some() {
            return Ok(());
        }
        self.c.sync = Some(SyncJournal {
            strings: self.c.g.strings.len(),
            nodes: self.c.g.nodes.len(),
            dirty: BTreeSet::new(),
        });
        Ok(())
    }
    fn apply_delta(&mut self, delta: ModelDelta) -> Result<(), PublicError> {
        let c = &mut self.c;
        let sync = c
            .sync
            .as_ref()
            .ok_or_else(|| invalid("Active model synchronization has not started"))?;
        if !sync.dirty.is_empty()
            || sync.strings != c.g.strings.len()
            || sync.nodes != c.g.nodes.len()
        {
            return Err(invalid("Drain native edits before applying host edits").into());
        }
        let old_strings = c.g.strings.len();
        let old_nodes = c.g.nodes.len();
        if delta.strings_start as usize != old_strings || delta.nodes_start as usize != old_nodes {
            return Err(invalid("Stale active model delta").into());
        }
        let units = delta
            .strings
            .iter()
            .try_fold(c.units, |n, s| n.checked_add(s.0.len()))
            .ok_or_else(limit)?;
        if old_strings.saturating_add(delta.strings.len()) > 1_000_000
            || units > 16_000_000
            || delta.nodes.len() > 1_000_000
        {
            return Err(limit().into());
        }
        let mut ids = HashSet::new();
        let mut final_nodes = old_nodes;
        for (id, node) in &delta.nodes {
            if !ids.insert(*id) {
                return Err(invalid("Duplicate active model node").into());
            }
            if *id as usize >= old_nodes {
                if *id as usize != final_nodes {
                    return Err(invalid("Non-contiguous active model nodes").into());
                }
                final_nodes += 1;
            } else {
                let old = &c.g.nodes[*id as usize];
                if std::mem::discriminant(old) != std::mem::discriminant(node) {
                    return Err(invalid("Active model object identity changed kind").into());
                }
                if let (Node::Instance { prototype: a, .. }, Node::Instance { prototype: b, .. }) =
                    (old, node)
                {
                    if a != b {
                        return Err(invalid("Active model object prototype changed").into());
                    }
                }
            }
        }
        if final_nodes > 1_000_000 {
            return Err(limit().into());
        }
        let delta_work = delta.nodes.iter().fold(0usize, |n, (_, node)| {
            n.saturating_add(node.properties().len())
                .saturating_add(match node {
                    Node::Map { entries, .. } => entries.len().saturating_mul(2),
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                })
        });
        c.tick(
            c.refs
                .saturating_add(units)
                .saturating_add(final_nodes)
                .saturating_add(delta_work),
        )?;
        // Swap only changed records. Invalid deltas roll back without copying
        // the rest of the arena or invalidating its existing native indexes.
        c.g.strings.extend(delta.strings);
        let mut previous = Vec::new();
        for (id, node) in delta.nodes {
            if (id as usize) < old_nodes {
                previous.push((id, std::mem::replace(&mut c.g.nodes[id as usize], node)));
            } else {
                c.g.nodes.push(node);
            }
        }
        if let Err(error) = c.g.validate_schema(&SCHEMA) {
            for (id, node) in previous {
                c.g.nodes[id as usize] = node;
            }
            c.g.nodes.truncate(old_nodes);
            c.g.strings.truncate(old_strings);
            return Err(error.into());
        }
        fn refs(n: &Node) -> usize {
            n.properties().len()
                + match n {
                    Node::Map { entries, .. } => entries.len() * 2,
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                }
        }
        for (id, old) in previous {
            c.refs = c.refs - refs(&old) + refs(&c.g.nodes[id as usize]);
        }
        for node in &c.g.nodes[old_nodes..] {
            c.refs += refs(node);
        }
        for id in ids {
            c.maps.remove(&id);
            c.sets.remove(&id);
            c.properties.remove(&id);
        }
        for (id, string) in c.g.strings.iter().enumerate().skip(old_strings) {
            c.canonical
                .push(*c.strings.entry(string.clone()).or_insert(id as u32));
        }
        c.units = units;
        let sync = c.sync.as_mut().unwrap();
        sync.strings = c.g.strings.len();
        sync.nodes = c.g.nodes.len();
        Ok(())
    }
    fn take_delta(&mut self) -> Result<ModelDelta, PublicError> {
        let c = &mut self.c;
        let sync = c
            .sync
            .as_ref()
            .ok_or_else(|| invalid("Active model synchronization has not started"))?;
        let work = sync
            .dirty
            .iter()
            .fold(0usize, |n, id| {
                let node = &c.g.nodes[*id as usize];
                n.saturating_add(node.properties().len() + 1)
                    .saturating_add(match node {
                        Node::Map { entries, .. } => entries.len() * 2,
                        Node::Set { values, .. } => values.len(),
                        _ => 0,
                    })
            })
            .saturating_add(c.g.strings[sync.strings..].iter().map(|s| s.0.len()).sum());
        c.tick(work)?;
        let sync = c.sync.as_mut().unwrap();
        let delta = ModelDelta {
            strings_start: sync.strings as u32,
            strings: c.g.strings[sync.strings..].to_vec(),
            nodes_start: sync.nodes as u32,
            nodes: std::mem::take(&mut sync.dirty)
                .into_iter()
                .map(|id| (id, c.g.nodes[id as usize].clone()))
                .collect(),
        };
        sync.strings = c.g.strings.len();
        sync.nodes = c.g.nodes.len();
        Ok(delta)
    }

    fn root(&self) -> Atom {
        self.c.g.root.clone()
    }
    fn validate(&self, value: &Atom) -> Result<(), PublicError> {
        self.c.g.validate_atom(value)?;
        if let Atom::BuiltinFunction(id) = value {
            if SCHEMA.method(MethodId(*id)).is_none() {
                return Err(invalid("Unknown public builtin function").into());
            }
        }
        Ok(())
    }
    fn read(&mut self, receiver: &Atom, key: &str) -> Result<Atom, PublicError> {
        self.validate(receiver)?;
        if key.len() > 1024 * 1024 {
            return Err(limit().into());
        }
        self.c.tick(key.len())?;
        self.c.read(receiver, key)
    }
    fn set(&mut self, receiver: &Atom, key: &str, value: Atom) -> Result<(), PublicError> {
        self.validate(receiver)?;
        self.validate(&value)?;
        if key.len() > 1024 * 1024 {
            return Err(limit().into());
        }
        // Array structural edits use push; writing existing elements preserves
        // the current length and descriptors. Arbitrary length assignment is
        // intentionally not a partial implementation of ArraySetLength.
        if let Atom::Ref(id) = receiver {
            if let Node::Array { length, .. } = self.c.g.nodes[*id as usize] {
                if key == "length"
                    || key
                        .parse::<u32>()
                        .ok()
                        .is_some_and(|n| n.to_string() == key && n >= length)
                {
                    return Err(invalid("Use active array push for structural array edits").into());
                }
            }
        }
        self.c.tick(key.len())?;
        self.c.set(receiver, key, value)
    }
    fn intern(&mut self, value: JsString) -> Result<Atom, PublicError> {
        self.c.tick(value.0.len())?;
        self.c.intern(value)
    }
    fn string(&mut self, value: &Atom) -> Result<JsString, PublicError> {
        self.validate(value)?;
        self.c.tick(1)?;
        match value {
            Atom::String(id) => {
                self.c.tick(self.c.g.strings[*id as usize].0.len())?;
                Ok(self.c.g.strings[*id as usize].clone())
            }
            _ => Err(type_error("Active model value is not a string")),
        }
    }
    fn object(&mut self) -> Result<Atom, PublicError> {
        self.c.node(Node::Object { properties: vec![] })
    }
    fn array(&mut self) -> Result<Atom, PublicError> {
        self.c.array(vec![])
    }
    fn date(&mut self, milliseconds: f64) -> Result<Atom, PublicError> {
        let milliseconds = if milliseconds.is_finite() && milliseconds.abs() <= 8.64e15 {
            milliseconds.trunc() + 0.0
        } else {
            f64::NAN
        };
        self.c.node(Node::Date {
            milliseconds,
            properties: vec![],
        })
    }
    fn date_millis(&mut self, value: &Atom) -> Result<f64, PublicError> {
        self.validate(value)?;
        self.c.tick(1)?;
        match value {
            Atom::Ref(id) => match self.c.g.nodes[*id as usize] {
                Node::Date { milliseconds, .. } => Ok(milliseconds),
                _ => Err(type_error("Active model value is not a Date")),
            },
            _ => Err(type_error("Active model value is not a Date")),
        }
    }
    fn push(&mut self, array: &Atom, value: Atom) -> Result<(), PublicError> {
        self.validate(array)?;
        self.validate(&value)?;
        self.c.push(array, value, "array.push")
    }
    fn invoke_builtin(
        &mut self,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        self.c.tick(call.arguments.len())?;
        self.validate(&call.receiver)?;
        for argument in &call.arguments {
            self.validate(argument)?;
        }
        // Captured methods use the same arena/indexes and restore the active
        // receiver even when scheduling fails after partially changing state.
        let receiver = std::mem::replace(&mut self.receiver, call.receiver);
        let direct_entry = self.direct_entry;
        let result = self.call(call.method.0, &call.arguments, host);
        self.receiver = receiver;
        self.direct_entry = direct_entry;
        result
    }
}
fn imported(
    c: &mut Context<'_>,
    tasks: Atom,
    compact: bool,
    top_axis: bool,
    render_now: f64,
) -> Result<GanttModel, PublicError> {
    let root = c.g.root.clone();
    let mut m = GanttModel::new(None, String::new());
    let mut bytes = 0usize;
    fn text(c: &mut Context<'_>, a: &Atom, bytes: &mut usize) -> Result<String, PublicError> {
        let value = c.scalar(a, "Gantt render scalar string")?;
        *bytes = bytes.saturating_add(value.len());
        if *bytes > 128 * 1024 * 1024 {
            return Err(limit().into());
        }
        Ok(value)
    }
    fn field(
        c: &mut Context<'_>,
        a: &Atom,
        k: &str,
        bytes: &mut usize,
    ) -> Result<String, PublicError> {
        let v = c.read(a, k)?;
        text(c, &v, bytes)
    }
    for key in ["title", "accTitle", "accDescr", "tickInterval", "weekday"] {
        let value = c.read(&root, key)?;
        let value = if matches!(value, Atom::Undefined | Atom::Null) {
            None
        } else {
            Some(text(c, &value, &mut bytes)?)
        };
        match key {
            "title" => m.title = value,
            "accTitle" => m.acc_title = value,
            "accDescr" => m.acc_descr = value,
            "tickInterval" => m.tick_interval = value,
            _ => m.weekday = value,
        }
    }
    m.date_format = field(c, &root, "dateFormat", &mut bytes)?;
    m.axis_format = field(c, &root, "axisFormat", &mut bytes)?;
    m.today_marker = field(c, &root, "todayMarker", &mut bytes)?;
    m.display_mode = if compact {
        "compact".into()
    } else {
        String::new()
    };
    // The ready-to-draw renderer consumes only token-list emptiness; scheduling
    // caches above retain the actual entries (including arbitrary edited values).
    for key in ["includes", "excludes"] {
        let list = c.read(&root, key)?;
        let length = c.read(&list, "length")?;
        let values = if truth(c.g, &length) {
            vec![String::new()]
        } else {
            vec![]
        };
        if key == "includes" {
            m.includes = values
        } else {
            m.excludes = values
        }
    }
    let inclusive = c.read(&root, "inclusiveEndDates")?;
    m.inclusive_end_dates = truth(c.g, &inclusive);
    m.top_axis = top_axis;
    let links = c.read(&root, "links")?;
    let Atom::Ref(id) = links else {
        return Err(invalid("Gantt links must be a Map").into());
    };
    let Node::Map { entries, .. } = &c.g.nodes[id as usize] else {
        return Err(invalid("Gantt links must be a Map").into());
    };
    let entries = entries.clone();
    for (k, v) in entries {
        if let Atom::String(_) = k {
            m.links
                .push((text(c, &k, &mut bytes)?, text(c, &v, &mut bytes)?));
        }
    }
    // Preserve the pinned npm renderer diagnostic identifier.
    c.native(&tasks, "filter", "x.filter", 1)?;
    for task in c.values(&tasks)? {
        let mut times = [None, None, None];
        for (n, key) in ["startTime", "endTime", "renderEndTime"].iter().enumerate() {
            let value = c.read(&task, key)?;
            if !matches!(value, Atom::Undefined | Atom::Null) {
                if !matches!(value,Atom::Ref(id)if matches!(c.g.nodes[id as usize],Node::Date{..}))
                {
                    return Err(type_error(&format!("task.{key}.getTime is not a function")));
                }
                times[n] = Some(c.numeric(&value)?);
            }
        }
        let mut flag = |key: &str| -> Result<bool, PublicError> {
            let a = c.read(&task, key)?;
            Ok(truth(c.g, &a))
        };
        let manual = false;
        let processed = false;
        let active = flag("active")?;
        let done = flag("done")?;
        let crit = flag("crit")?;
        let milestone = flag("milestone")?;
        let vert = flag("vert")?;
        let classes = c.read(&task, "classes")?;
        let clickable = if let Atom::String(id) = classes {
            c.g.strings[id as usize]
                .0
                .windows(9)
                .any(|v| v.iter().copied().eq("clickable".encode_utf16()))
        } else {
            c.native(&classes, "includes", "task.classes.includes", 1)?;
            c.values(&classes)?
                .iter()
                .any(|v| matches!(v,Atom::String(id)if c.g.strings[*id as usize].is("clickable")))
        };
        let classes = if clickable {
            vec!["clickable".into()]
        } else {
            vec![]
        };
        let number = if m.display_mode == "compact" {
            let order = c.read(&task, "order")?;
            c.numeric(&order)?
        } else {
            0.
        };
        m.set_public_task_order(m.tasks.len(), number);
        if let Atom::Ref(id) = &task {
            m.set_public_task_identity(m.tasks.len(), *id);
        }
        m.tasks.push(GanttTask {
            id: field(c, &task, "id", &mut bytes)?,
            task: field(c, &task, "task", &mut bytes)?,
            section: field(c, &task, "section", &mut bytes)?,
            start_time: times[0],
            end_time: times[1],
            render_end_time: times[2],
            manual_end_time: manual,
            processed,
            active,
            done,
            crit,
            milestone,
            vert,
            order: number as isize,
            classes,
            prev_task_id: None,
            raw: TaskRaw {
                data: String::new(),
                start: None,
                end: String::new(),
            },
        });
    }
    // Native drawing caches are DB implementation details. An ordinary declared
    // model need not own them; its real missing date-filter call is checked by
    // draw, after the preceding host callbacks.
    if matches!(root,Atom::Ref(id) if matches!(c.g.nodes[id as usize],Node::Object{..})) {
        m.import_public_cache(render_now, 0., vec![], vec![], vec![], false);
        return Ok(m);
    }
    let mut cache = vec![];
    for key in ["includeSet", "excludeSet"] {
        let set = c.read(&root, key)?;
        let values = c.values(&set)?;
        let mut tokens = vec![];
        for value in values {
            if let Atom::String(_) = value {
                if let Atom::String(id) = value {
                    if let Some(value) = c.g.strings[id as usize].well_formed() {
                        bytes = bytes.saturating_add(value.len());
                        if bytes > 128 * 1024 * 1024 {
                            return Err(limit().into());
                        }
                        tokens.push(value);
                    }
                }
            }
        }
        cache.push(tokens)
    }
    let days = c.read(&root, "excludedDays")?;
    let days = c
        .values(&days)?
        .iter()
        .map(|value| truth(c.g, value))
        .collect();
    let dated = c.read(&root, "datedTokens")?;
    let dated = truth(c.g, &dated);
    let work = c.read(&root, "work")?;
    let work = c.numeric(&work)?;
    m.import_public_cache(
        render_now,
        work,
        cache.remove(0),
        cache.remove(0),
        days,
        dated,
    );
    Ok(m)
}
// Retain options without inspecting them. Undefined uses the fresh empty object
// already supplied by the shared field initializer.
fn construct_with(argument: ConstructorArgument) -> Result<GanttPublicState, PeleError> {
    let mut graph = initial(&GanttModel::new(None, String::new()))?;
    if !matches!(argument, ConstructorArgument::Undefined) {
        let root = graph.root.clone();
        let mut context = Context::new(&mut graph);
        let value = argument
            .into_atom(|value| context.intern(value))
            .map_err(legacy_error)?;
        context.set(&root, "options", value).map_err(legacy_error)?;
    }
    Ok(GanttPublicState { graph })
}
const LIVE_FIELDS: &[LiveConstructorField] = &[LiveConstructorField {
    key: "options",
    methods: &[MethodId(24)],
}];
// Preserve this.options.warn && this.options.warn(producer()) evaluation.
// Each request owns values and runs after the graph borrow has ended.
enum WarningPhase {
    Start { receiver: Atom, producer: Atom },
    FirstOptions { receiver: Atom, producer: Atom },
    FirstWarn { receiver: Atom, producer: Atom },
    FirstTest { receiver: Atom, producer: Atom },
    SecondOptions { producer: Atom },
    SecondWarn { receiver: Atom, producer: Atom },
    Producer { receiver: Atom, function: Atom },
    Callback,
    Done,
}
struct WarningInvocation {
    phase: WarningPhase,
}
impl WarningInvocation {
    fn step(&mut self, _graph: &ModelGraph, reply: Option<Atom>) -> PublicInvocationPhase {
        use WarningPhase::*;
        let get =
            |receiver, key| PublicInvocationPhase::Host(PublicHostOperation::Get { receiver, key });
        match std::mem::replace(&mut self.phase, Done) {
            Start { receiver, producer } => {
                self.phase = FirstOptions {
                    receiver: receiver.clone(),
                    producer,
                };
                get(receiver, "options")
            }
            FirstOptions { receiver, producer } => {
                self.phase = FirstWarn { receiver, producer };
                get(reply.expect("awaited options"), "warn")
            }
            FirstWarn { receiver, producer } => {
                self.phase = FirstTest { receiver, producer };
                PublicInvocationPhase::Host(PublicHostOperation::Truthy {
                    value: reply.expect("awaited first warn"),
                })
            }
            FirstTest { receiver, producer } => {
                if !matches!(reply, Some(Atom::Bool(true))) {
                    return PublicInvocationPhase::Complete(Ok(Atom::Undefined));
                }
                self.phase = SecondOptions { producer };
                get(receiver, "options")
            }
            SecondOptions { producer } => {
                let receiver = reply.expect("awaited second options");
                self.phase = SecondWarn {
                    receiver: receiver.clone(),
                    producer,
                };
                get(receiver, "warn")
            }
            SecondWarn { receiver, producer } => {
                self.phase = Producer {
                    receiver,
                    function: reply.expect("awaited second warn"),
                };
                PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function: producer,
                    result: PublicHostResult::Retain,
                    not_callable_error: "e is not a function",
                    receiver: Atom::Undefined,
                    arguments: vec![],
                })
            }
            Producer { receiver, function } => {
                self.phase = Callback;
                PublicInvocationPhase::Host(PublicHostOperation::Call {
                    function,
                    result: PublicHostResult::Discard,
                    not_callable_error: "this.options.warn is not a function",
                    receiver,
                    arguments: vec![reply.expect("awaited message producer")],
                })
            }
            Callback => PublicInvocationPhase::Complete(Ok(Atom::Undefined)),
            Done => PublicInvocationPhase::Complete(Err(invalid(
                "Gantt warning invocation already finished",
            )
            .into())),
        }
    }
}
fn resume_warning(
    state: &mut GanttPublicState,
    invocation: &mut WarningInvocation,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    let reply = reply
        .map(|value| value.into_atom(|value| Context::new(&mut state.graph).intern(value)))
        .transpose();
    match reply {
        Ok(reply) => invocation.step(&state.graph, reply),
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}

impl PublicDiagramModule for Gantt {
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTO).then(|| {
            PublicConstructorFactory::first_value::<Self>(PROTO, construct_with, LIVE_FIELDS)
        })
    }
    fn detached_public_invocation(&self, call: MethodCall) -> Option<DetachedPublicInvocation> {
        (call.method == MethodId(24)).then(|| {
            let producer = call.arguments.first().cloned().unwrap_or(Atom::Undefined);
            DetachedPublicInvocation::new::<Self, GanttPublicState, _>(
                WarningInvocation {
                    phase: WarningPhase::Start {
                        receiver: call.receiver,
                        producer,
                    },
                },
                resume_warning,
            )
        })
    }

    type PublicState = GanttPublicState;
    fn try_parse_public(
        &self,
        context: ParseContext<'_>,
        _edge_limit: Option<crate::compat::EdgeLimit>,
    ) -> Option<Result<Self::PublicState, PeleError>> {
        try_parse_public_direct(context)
    }

    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        Ok(GanttPublicState { graph })
    }
    fn create_public(&self, model: &GanttModel) -> Result<GanttPublicState, PeleError> {
        Ok(GanttPublicState {
            graph: initial(model)?,
        })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, state: &GanttPublicState) -> Result<ModelGraph, PeleError> {
        state.graph.validate_schema(&SCHEMA)?;
        Ok(state.graph.clone())
    }
    fn apply_public(
        &self,
        state: &mut GanttPublicState,
        graph: ModelGraph,
    ) -> Result<(), PeleError> {
        graph.validate_schema(&SCHEMA)?;
        state.graph = graph;
        Ok(())
    }
    fn invoke_public(
        &self,
        state: &mut GanttPublicState,
        call: MethodCall,
        _: &Object,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let mut arena = Arena {
            c: Context::new(&mut state.graph),
            receiver: call.receiver,
            direct_entry: None,
        };
        arena.call(call.method.0, &call.arguments, host)
    }
    fn render_public(
        &self,
        state: &mut GanttPublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render_public_operation(state, config, options, host)
            .map_err(legacy_error)
    }
    fn render_public_operation(
        &self,
        state: &mut GanttPublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        let root = state.graph.root.clone();
        let mut arena = Arena {
            c: Context::new(&mut state.graph),
            receiver: root,
            direct_entry: None,
        };
        let now = match options.now.filter(|value| value.is_finite()) {
            Some(value) => value,
            None => host.now_millis().map_err(PublicError::from)?,
        };
        // Pinned render captures both fields before calling the live override.
        let mode = arena.get("displayMode")?;
        let compact =
            matches!(mode, Atom::String(id) if arena.c.g.strings[id as usize].is("compact"));
        let top = arena.get("topAxis")?;
        let top_axis = truth(arena.c.g, &top);
        let implementation = arena.get("getTasks")?;
        let arguments = [Atom::Number(now)];
        let tasks = match implementation {
            Atom::Undefined => {
                arena.native_method("getTasks")?;
                arena.call(19, &arguments, host)?
            }
            Atom::BuiltinFunction(id) => arena.call(id, &arguments, host)?,
            Atom::HostReference(reference) => {
                let function = host.host_function(reference)?;
                let receiver = arena.receiver.clone();
                let result = function.call(&mut arena, receiver, &arguments, host)?;
                arena.validate(&result)?;
                result
            }
            _ => return Err(type_error("db.getTasks is not a function")),
        };
        let mut model = imported(&mut arena.c, tasks, compact, top_axis, now)?;
        let filter = arena.get("isInvalidDate")?;
        let filter_callable = matches!(filter, Atom::BuiltinFunction(_) | Atom::HostReference(_))
            || (matches!(filter, Atom::Undefined)
                && matches!(arena.receiver,Atom::Ref(id) if matches!(arena.c.g.nodes[id as usize],Node::Instance{prototype:PROTO,..}) && own(arena.c.g,id,"isInvalidDate").is_none()));
        let result = super::render::render_scheduled_checked(
            &mut model,
            config,
            options,
            host,
            filter_callable,
        );
        if matches!(arena.receiver,Atom::Ref(id) if matches!(arena.c.g.nodes[id as usize],Node::Instance{prototype:PROTO,..}))
        {
            arena.set("work", Atom::Number(model.work))?;
        }
        result
    }
}

fn decode_link(link: &str) -> Result<String, PublicError> {
    match crate::compat::source::decode_entities(link) {
        Ok(value) => Ok(value),
        Err(error) => {
            let mut cursor = 0;
            while let Some(start) = link[cursor..].find("ﬂ°").map(|n| n + cursor) {
                let body = start + "ﬂ°".len();
                let Some(stop) = link[body..].find('¶').map(|n| n + body) else {
                    break;
                };
                if link[stop..].starts_with("¶ß") && link[body..].starts_with('°') {
                    if let Ok(number) = link[body + '°'.len_utf8()..stop]
                        .trim_matches(crate::compat::source::space)
                        .parse::<f64>()
                    {
                        if number > 0.
                            && number <= 1114111.
                            && !(55296. ..=57343.).contains(&number)
                            && number.fract() != 0.
                        {
                            return Err(PublicError::JavaScript {
                                name: JavaScriptErrorKind::RangeError,
                                message: JsString::from(
                                    format!(
                                        "Invalid code point {}",
                                        crate::compat::svg::js_num(number)
                                    )
                                    .as_str(),
                                ),
                            });
                        }
                    }
                }
                cursor = stop + '¶'.len_utf8();
            }
            Err(error.into())
        }
    }
}

// Ordinary source parsing can construct its final public records without an
// owned Vec<GanttTask>. Imported/typed/prepared documents keep their old paths.
struct DirectParse<'a> {
    c: Context<'a>,
    header: GanttModel,
    tasks: Vec<Atom>,
    by_id: Vec<(Atom, Atom)>,
    latest: HashMap<u32, (usize, u32)>,
    links: Vec<(u32, String)>,
    generated: usize,
    order: usize,
    previous: Option<u32>,
    storage_failed: bool,
}
impl Context<'_> {
    fn initial_push(&mut self, id: u32, value: Atom) -> Result<(), PublicError> {
        let index = match &self.g.nodes[id as usize] {
            Node::Array { length, .. } => *length as usize,
            _ => unreachable!("private fresh array handle"),
        };
        self.tick(1)?;
        self.tick(1)?;
        self.tick(1)?;
        if index >= 1_000_000 {
            return Err(limit().into());
        }
        self.tick(1)?;
        let Atom::String(key) = self.text(&index.to_string())? else {
            unreachable!()
        };
        self.room(1)?;
        let Node::Array { length, properties } = &mut self.g.nodes[id as usize] else {
            unreachable!()
        };
        properties.push(Property {
            key,
            value,
            flags: 7,
        });
        *length = (index + 1) as u32;
        self.changed(id);
        Ok(())
    }
    fn initial_text_id(&self, value: &str) -> Option<u32> {
        let mut buffer = [0; 64];
        match JsString::short_utf16(value, &mut buffer) {
            Some(units) => self.strings.get(units).copied(),
            None => self.strings.get(&JsString::from(value)).copied(),
        }
    }
}
impl parser::Target for DirectParse<'_> {
    fn header(&mut self) -> &mut GanttModel {
        &mut self.header
    }
    fn add_task(&mut self, text: String, data: String) -> Result<(), PeleError> {
        let declaration = model::task_declaration(&text, &data)?;
        let generated;
        let id_text = match declaration.id {
            Some(id) => id,
            None => {
                self.generated += 1;
                generated = format!("task{}", self.generated);
                &generated
            }
        };
        let order = if declaration.vert {
            -1
        } else {
            let order = self.order as isize;
            self.order += 1;
            order
        };
        let result = (|| -> Result<(), PublicError> {
            let raw_data = self.c.text(&data)?;
            let raw_start = self.c.optional_text(declaration.start)?;
            let raw_end = self.c.text(declaration.end)?;
            let raw = self.c.initial_record(
                [("data", raw_data), ("start", raw_start), ("end", raw_end)].into_iter(),
            )?;
            let id = self.c.text(id_text)?;
            let Atom::String(string_id) = id else {
                unreachable!()
            };
            let task = self.c.text(&text)?;
            let section = self.c.text(self.header.public_builder_state().0)?;
            let classes = self.c.initial_array(vec![])?;
            let Atom::Ref(classes_id) = classes else {
                unreachable!()
            };
            let record = self.c.initial_record(
                [
                    ("id", id.clone()),
                    ("task", task),
                    ("section", section),
                    ("startTime", Atom::Undefined),
                    ("endTime", Atom::Undefined),
                    ("renderEndTime", Atom::Null),
                    ("manualEndTime", Atom::Bool(false)),
                    ("processed", Atom::Bool(false)),
                    ("active", Atom::Bool(declaration.active)),
                    ("done", Atom::Bool(declaration.done)),
                    ("crit", Atom::Bool(declaration.crit)),
                    ("milestone", Atom::Bool(declaration.milestone)),
                    ("vert", Atom::Bool(declaration.vert)),
                    ("order", Atom::Number(order as f64)),
                    ("classes", classes),
                    (
                        "prevTaskId",
                        self.previous.map_or(Atom::Undefined, Atom::String),
                    ),
                    ("raw", raw),
                ]
                .into_iter(),
            )?;
            if let Some((position, _)) = self.latest.get(&string_id).copied() {
                self.by_id[position].1 = record.clone();
                self.latest.insert(string_id, (position, classes_id));
            } else {
                self.latest
                    .insert(string_id, (self.by_id.len(), classes_id));
                self.by_id.push((id, record.clone()));
            }
            self.tasks.push(record);
            self.previous = Some(string_id);
            Ok(())
        })();
        result.map_err(|error| {
            self.storage_failed = true;
            legacy_error(error)
        })
    }
    fn class(&mut self, ids: &str) -> Result<(), PeleError> {
        let result = (|| -> Result<(), PublicError> {
            for name in ids.split(',') {
                let Some(id) = self.c.initial_text_id(name) else {
                    continue;
                };
                if let Some((_, classes)) = self.latest.get(&id).copied() {
                    let clickable = self.c.text("clickable")?;
                    self.c.initial_push(classes, clickable)?;
                }
            }
            Ok(())
        })();
        result.map_err(|error| {
            self.storage_failed = true;
            legacy_error(error)
        })
    }
    fn link(&mut self, ids: &str, url: &str) -> Result<(), PeleError> {
        // Sanitize before checking task existence, exactly as the typed model.
        let href = crate::compat::source::url::safe_url(url)?;
        for name in ids.split(',') {
            let Some(id) = self.c.initial_text_id(name) else {
                continue;
            };
            if self.latest.contains_key(&id) {
                if let Some((_, value)) = self.links.iter_mut().find(|(key, _)| *key == id) {
                    *value = href.clone();
                } else {
                    self.links.push((id, href.clone()));
                }
            }
        }
        self.class(ids)
    }
}
impl DirectParse<'_> {
    fn finish(mut self) -> Result<(), PublicError> {
        let by_id = self.c.node(Node::Map {
            entries: self.by_id,
            properties: vec![],
        })?;
        let tasks = self.c.initial_array(self.tasks)?;
        let links = self
            .links
            .iter()
            .map(|(id, value)| Ok((Atom::String(*id), self.c.text(value)?)))
            .collect::<Result<Vec<_>, PublicError>>()?;
        let links = self.c.node(Node::Map {
            entries: links,
            properties: vec![],
        })?;
        // Source strings are UTF8; this only recovers one last-task scalar for
        // the shared root layout. No per-task UTF16->UTF8 reconstruction.
        let last = self.previous.map(|id| {
            String::from_utf16(&self.c.g.strings[id as usize].0).expect("source text is UTF8")
        });
        let section = self.header.public_builder_state().0;
        finish_initial_root(
            &mut self.c,
            &self.header,
            tasks,
            by_id,
            links,
            (section, self.generated, self.order, last.as_deref()),
        )?;
        // Conservative interior of every historical construction budget. Close
        // to a boundary, the caller executes the exact legacy constructor.
        if self.c.g.nodes.len() > 100_000
            || self.c.g.strings.len() > 100_000
            || self.c.refs > 1_000_000
            || self.c.units > 4_000_000
            || self.c.work > 10_000_000
        {
            return Err(limit().into());
        }
        Ok(())
    }
}
fn try_parse_public_direct(
    context: ParseContext<'_>,
) -> Option<Result<GanttPublicState, PeleError>> {
    let mut graph = ModelGraph {
        version: 1,
        root: Atom::Undefined,
        strings: vec![],
        nodes: vec![],
    };
    let mut target = DirectParse {
        c: Context::new(&mut graph),
        header: parse_header(&context),
        tasks: vec![],
        by_id: vec![],
        latest: HashMap::new(),
        links: vec![],
        generated: 0,
        order: 0,
        previous: None,
        storage_failed: false,
    };
    if let Err(error) = parser::parse_into(context.source, &mut target) {
        return if target.storage_failed {
            None
        } else {
            Some(Err(error))
        };
    }
    if target.finish().is_err() {
        return None;
    }
    Some(
        graph
            .validate_schema(&SCHEMA)
            .map(|()| GanttPublicState { graph }),
    )
}
