//! Opt-in native ClassDb/ClassMember codec, Pele 458322a5 (MIT, kepano).
//! Object identity belongs to the arena; render imports the existing concrete
//! ClassModel without reparsing. Namespace maps and tag Sets retain live aliases.
//! Root and member construction use captured native factories. Member bodies
//! yield virtual method reads/calls without retaining a state borrow.
//! Own readonly Array length is represented by an additional flags0 property;
//! default writable flags2 is implicit alongside scalar Array.length.
use super::model::RelationType;
use super::*;
use crate::compat::{
    public::{
        ActiveModel, Atom, FieldKind, FieldSchema, GraphWriter, JavaScriptErrorKind, JsString,
        MemberKind, MemberSchema, MethodCall, MethodId, MethodSchema, ModelDelta, ModelGraph,
        ModelSchema, Node, Property, PrototypeId, PrototypeParent, PrototypeSchema,
        PublicConstructorFactory, PublicDiagramModule, PublicError, RecordId, RecordKind,
        RecordSchema,
    },
    ErrorCode,
};
use std::collections::{BTreeSet, HashMap, HashSet};
#[path = "public_model/display_continuation.rs"]
mod display_continuation;
#[path = "public_model/member_constructor.rs"]
mod member_constructor;
const PROTO: PrototypeId = PrototypeId(1);
const MEMBER: PrototypeId = PrototypeId(2);
const fn field_schema(
    name: &'static str,
    kind: FieldKind,
    optional: bool,
    undefined: bool,
) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        optional,
        undefined,
        nullable: false,
        flags: 7,
    }
}
const fn member(id: u32, name: &'static str) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method {
            method: MethodId(id),
        },
    }
}
const fn method(id: u32, name: &'static str, arity: u32) -> MethodSchema {
    MethodSchema {
        id: MethodId(id),
        name,
        arity,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/class@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "ClassDb",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false, false),
                field_schema("lineType", FieldKind::Object, false, false),
                field_schema("relationType", FieldKind::Object, false, false),
                field_schema("direction", FieldKind::String, false, false),
                field_schema("classes", FieldKind::Map, false, false),
                field_schema("relations", FieldKind::Array, false, false),
                field_schema("notes", FieldKind::Map, false, false),
                field_schema("namespaces", FieldKind::Map, false, false),
                field_schema("interfaces", FieldKind::Array, false, false),
                field_schema("styleClasses", FieldKind::Map, false, false),
                field_schema("title", FieldKind::String, false, true),
                field_schema("accTitle", FieldKind::String, false, true),
                field_schema("accDescr", FieldKind::String, false, true),
                field_schema("namespaceStack", FieldKind::Array, false, false),
                field_schema("withCssClass", FieldKind::Map, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "ClassMember",
            kind: RecordKind::Object,
            prototype: Some(MEMBER),
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("memberType", FieldKind::String, false, false),
                field_schema("visibility", FieldKind::String, false, false),
                field_schema("classifier", FieldKind::String, false, false),
                field_schema("parameters", FieldKind::String, false, false),
                field_schema("returnType", FieldKind::String, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "ClassNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("type", FieldKind::String, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("cssClasses", FieldKind::String, false, false),
                field_schema("methods", FieldKind::Array, false, false),
                field_schema("members", FieldKind::Array, false, false),
                field_schema("annotations", FieldKind::Array, false, false),
                field_schema("styles", FieldKind::Array, false, false),
                field_schema("parent", FieldKind::String, true, true),
                field_schema("link", FieldKind::String, true, true),
                field_schema("linkTarget", FieldKind::String, true, true),
                field_schema("haveCallback", FieldKind::Bool, true, true),
                field_schema("tooltip", FieldKind::String, true, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "NamespaceNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("classes", FieldKind::Map, false, false),
                field_schema("notes", FieldKind::Map, false, false),
                field_schema("children", FieldKind::Map, false, false),
                field_schema("parent", FieldKind::String, false, true),
                field_schema("explicit", FieldKind::Bool, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(5),
            name: "ClassNote",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("class", FieldKind::String, false, true),
                field_schema("text", FieldKind::String, false, false),
                field_schema("index", FieldKind::Number, false, false),
                field_schema("parent", FieldKind::String, true, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(6),
            name: "ClassRelation",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id1", FieldKind::String, false, false),
                field_schema("id2", FieldKind::String, false, false),
                field_schema("relation", FieldKind::Object, false, false),
                field_schema("relationTitle1", FieldKind::String, false, false),
                field_schema("relationTitle2", FieldKind::String, false, false),
                field_schema("title", FieldKind::String, true, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(7),
            name: "ClassInterface",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("classId", FieldKind::String, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(8),
            name: "StyleClass",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("styles", FieldKind::Array, false, false),
                field_schema("textStyles", FieldKind::Array, false, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[
        PrototypeSchema {
            id: PROTO,
            name: "ClassDb",
            parent: PrototypeParent::Object,
            members: &[
                member(1, "addClass"),
                member(2, "tag"),
                member(3, "ensure"),
                member(4, "setClassLabel"),
                member(5, "addRelation"),
                member(6, "addInterface"),
                member(7, "addAnnotation"),
                member(8, "addMember"),
                member(9, "addMembers"),
                member(10, "addNote"),
                member(11, "cleanupLabel"),
                member(12, "setCssClass"),
                member(13, "defineClass"),
                member(14, "setTooltip"),
                member(15, "setLink"),
                member(16, "setClickEvent"),
                member(17, "setDirection"),
                member(18, "setAccTitle"),
                member(19, "setAccDescription"),
                member(20, "addNamespace"),
                member(21, "popNamespace"),
                member(22, "addClassesToNamespace"),
                member(23, "setCssStyle"),
            ],
        },
        PrototypeSchema {
            id: MEMBER,
            name: "ClassMember",
            parent: PrototypeParent::Object,
            members: &[
                member(24, "parseMethod"),
                member(25, "parseAttribute"),
                member(26, "getDisplayDetails"),
                member(27, "parseClassifier"),
            ],
        },
    ],
    methods: &[
        method(1, "addClass", 1),
        method(2, "tag", 2),
        method(3, "ensure", 1),
        method(4, "setClassLabel", 2),
        method(5, "addRelation", 1),
        method(6, "addInterface", 2),
        method(7, "addAnnotation", 2),
        method(8, "addMember", 2),
        method(9, "addMembers", 2),
        method(10, "addNote", 2),
        method(11, "cleanupLabel", 1),
        method(12, "setCssClass", 2),
        method(13, "defineClass", 2),
        method(14, "setTooltip", 2),
        method(15, "setLink", 3),
        method(16, "setClickEvent", 3),
        method(17, "setDirection", 1),
        method(18, "setAccTitle", 1),
        method(19, "setAccDescription", 1),
        method(20, "addNamespace", 2),
        method(21, "popNamespace", 0),
        method(22, "addClassesToNamespace", 3),
        method(23, "setCssStyle", 2),
        method(24, "parseMethod", 1),
        method(25, "parseAttribute", 1),
        method(26, "getDisplayDetails", 0),
        method(27, "parseClassifier", 0),
    ],
};
pub struct ClassPublicState {
    graph: ModelGraph,
    model: Result<ClassModel, PeleError>,
}
fn invalid(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("class".into());
    e
}
fn limit() -> PeleError {
    PeleError::new(
        "Class native graph operation exceeds work budget",
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
struct Context<'a> {
    g: &'a mut ModelGraph,
    strings: HashMap<JsString, u32>,
    canonical: Vec<u32>,
    maps: HashMap<u32, HashMap<Key, usize>>,
    sets: HashMap<u32, HashSet<Key>>,
    work: usize,
    refs: usize,
    units: usize,
    sync: Option<SyncJournal>,
    depth: usize,
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
            sets: HashMap::new(),
            work: 0,
            refs,
            units,
            sync: None,
            depth: 0,
        }
    }
    fn dirty(&mut self, id: u32) {
        if let Some(sync) = &mut self.sync {
            sync.dirty.insert(id);
        }
    }
    fn tick(&mut self, n: usize) -> Result<(), PublicError> {
        self.work = self.work.saturating_add(n);
        if self.work > 1_000_000 {
            Err(limit().into())
        } else {
            Ok(())
        }
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
    fn own(&mut self, id: u32, k: &str) -> Result<Option<Atom>, PublicError> {
        self.tick(self.g.nodes[id as usize].properties().len())?;
        Ok(own(self.g, id, k).cloned())
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
                        invalid("Class host property receiver dispatch is unavailable").into(),
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
        self.dirty(*id);
        self.tick(self.g.nodes[*id as usize].properties().len())?;
        let at = self.g.nodes[*id as usize]
            .properties()
            .iter()
            .position(|p| self.g.strings[p.key as usize].is(k));
        if let Some(at) = at {
            if self.g.nodes[*id as usize].properties()[at].flags & 2 == 0 {
                return Err(type_error(&format!(
                    "Cannot assign to read only property '{k}' of object '#<Object>'"
                )));
            }
            self.g.nodes[*id as usize].properties_mut()[at].value = v;
        } else {
            self.room(1)?;
            let Atom::String(key) = self.text(k)? else {
                unreachable!()
            };
            self.g.nodes[*id as usize].properties_mut().push(Property {
                key,
                value: v,
                flags: 7,
            });
        }
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
        self.dirty(id);
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
                "Class overridden {expr} callback dispatch is unavailable"
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
        self.dirty(id);
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
        self.dirty(id);
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
    fn includes_color(&mut self, a: &Atom) -> Result<bool, PublicError> {
        if let Atom::String(id) = a {
            self.tick(self.g.strings[*id as usize].0.len())?;
            return Ok(self.g.strings[*id as usize]
                .0
                .windows(5)
                .any(|w| w == [99, 111, 108, 111, 114]));
        }
        let id = self.native(a, "includes", "style.includes", 1)?;
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            unreachable!()
        };
        for n in 0..length {
            self.tick(1)?;
            if matches!(self.read(a,&n.to_string())?,Atom::String(id) if self.g.strings[id as usize].is("color"))
            {
                return Ok(true);
            }
        }
        Ok(false)
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
            _ => Err(invalid("Class object primitive-conversion dispatch is unavailable").into()),
        }
    }
}
fn required<'a>(g: &'a ModelGraph, a: &Atom, k: &str) -> Result<&'a Atom, PeleError> {
    let Atom::Ref(id) = a else {
        return Err(invalid("Class drawable record must be an object"));
    };
    own(g, *id, k).ok_or_else(|| invalid(&format!("Class drawable field {k} is missing")))
}
struct Import<'a> {
    g: &'a ModelGraph,
    bytes: usize,
    work: usize,
}
impl Import<'_> {
    fn text(&mut self, a: &Atom) -> Result<String, PeleError> {
        let text = self.g.text(a)?;
        self.bytes = self.bytes.saturating_add(text.0.len().saturating_mul(3));
        if self.bytes > 128 * 1024 * 1024 {
            return Err(limit());
        }
        text.well_formed()
            .ok_or_else(|| invalid("Class rendering of lone UTF-16 surrogates is not implemented"))
    }
    fn field(&mut self, a: &Atom, k: &str) -> Result<String, PeleError> {
        self.text(required(self.g, a, k)?)
    }
    fn optional(&mut self, a: &Atom, k: &str) -> Result<Option<String>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Class drawable record must be an object"));
        };
        match own(self.g, *id, k) {
            None | Some(Atom::Undefined | Atom::Null) => Ok(None),
            Some(a) => self.text(a).map(Some),
        }
    }
    fn array(&mut self, a: &Atom) -> Result<Vec<Atom>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Class drawable list must be an Array"));
        };
        let Node::Array { length, properties } = &self.g.nodes[*id as usize] else {
            return Err(invalid("Class drawable list must be an Array"));
        };
        self.work = self
            .work
            .saturating_add(*length as usize)
            .saturating_add(properties.len());
        if self.work > 32_000_000 {
            return Err(limit());
        }
        let mut out = vec![Atom::Undefined; *length as usize];
        for p in properties {
            if let Some(index) = self.g.strings[p.key as usize]
                .well_formed()
                .and_then(|s| s.parse::<u32>().ok())
            {
                if index < *length && self.g.strings[p.key as usize].is(&index.to_string()) {
                    out[index as usize] = p.value.clone()
                }
            }
        }
        Ok(out)
    }
    fn texts(&mut self, a: &Atom) -> Result<Vec<String>, PeleError> {
        self.array(a)?.iter().map(|a| self.text(a)).collect()
    }
    fn map(&mut self, a: &Atom) -> Result<Vec<(Atom, Atom)>, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Class drawable lookup must be a Map"));
        };
        let Node::Map { entries, .. } = &self.g.nodes[*id as usize] else {
            return Err(invalid("Class drawable lookup must be a Map"));
        };
        self.work = self.work.saturating_add(entries.len());
        if self.work > 32_000_000 {
            return Err(limit());
        }
        Ok(entries.clone())
    }
    fn member(&mut self, a: &Atom) -> Result<ClassMember, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Class members must expose getDisplayDetails"));
        };
        let native = matches!(
            self.g.nodes[*id as usize],
            Node::Instance {
                prototype: MEMBER,
                ..
            }
        );
        for (name, method) in [("getDisplayDetails", 26), ("parseClassifier", 27)] {
            if !matches!(own(self.g,*id,name),None if native)
                && !matches!(own(self.g,*id,name),Some(Atom::BuiltinFunction(id))if *id==method)
            {
                return Err(invalid("Class member overridden/missing native method requires render dispatch capability"));
            }
        }
        let member_type = self.field(a, "memberType")?;
        let method = member_type == "method";
        // Only classifier equality is consumed by getDisplayDetails/row CSS.
        // Unused parameter/return slots on attributes never enter the renderer.
        let classifier = match own(self.g, *id, "classifier") {
            Some(Atom::String(s)) if self.g.strings[*s as usize].is("*") => "*",
            Some(Atom::String(s)) if self.g.strings[*s as usize].is("$") => "$",
            _ => "",
        }
        .to_owned();
        let parameters = if method {
            self.field(a, "parameters")?
        } else {
            String::new()
        };
        let return_type =
            if method && own(self.g, *id, "returnType").is_some_and(|a| truth(self.g, a)) {
                self.field(a, "returnType")?
            } else {
                String::new()
            };
        Ok(ClassMember {
            id: self.field(a, "id")?,
            member_type,
            visibility: self.field(a, "visibility")?,
            classifier,
            parameters,
            return_type,
        })
    }
}
fn imported(g: &ModelGraph) -> Result<ClassModel, PeleError> {
    imported_mode(g, false)
}
fn imported_mode(g: &ModelGraph, live: bool) -> Result<ClassModel, PeleError> {
    let mut i = Import {
        g,
        bytes: 0,
        work: 0,
    };
    let root = &g.root;
    let mut m = ClassModel::new(None);
    m.direction = i.field(root, "direction")?;
    m.title = i.optional(root, "title")?;
    m.acc_title = i.optional(root, "accTitle")?;
    m.acc_descr = i.optional(root, "accDescr")?;
    let mut used = HashSet::new();
    let mut ids = HashSet::new();
    for (key, a) in i.map(required(g, root, "classes")?)? {
        let key = i.text(&key)?;
        let id = i.field(&a, "id")?;
        ids.insert(id.clone());
        let css_classes = i.field(&a, "cssClasses")?;
        used.extend(css_classes.split(' ').map(str::to_owned));
        let methods = if live {
            vec![]
        } else {
            i.array(required(g, &a, "methods")?)?
                .iter()
                .map(|a| i.member(a))
                .collect::<Result<_, _>>()?
        };
        let members = if live {
            vec![]
        } else {
            i.array(required(g, &a, "members")?)?
                .iter()
                .map(|a| i.member(a))
                .collect::<Result<_, _>>()?
        };
        let callback = match a {
            Atom::Ref(id) => own(g, id, "haveCallback").map(|a| truth(g, a)),
            _ => None,
        };
        m.classes.push((
            key,
            ClassNode {
                id,
                generic: i.field(&a, "type")?,
                label: i.field(&a, "label")?,
                css_classes,
                methods,
                members,
                annotations: i.texts(required(g, &a, "annotations")?)?,
                styles: i.texts(required(g, &a, "styles")?)?,
                parent: i.optional(&a, "parent")?,
                link: i.optional(&a, "link")?,
                link_target: i.optional(&a, "linkTarget")?,
                have_callback: callback,
                tooltip: i.optional(&a, "tooltip")?,
            },
        ));
    }
    for (key, a) in i.map(required(g, root, "namespaces")?)? {
        let key = i.text(&key)?;
        m.namespaces.push((
            key,
            NamespaceNode {
                id: i.field(&a, "id")?,
                label: i.field(&a, "label")?,
                classes: vec![],
                notes: vec![],
                children: vec![],
                parent: i.optional(&a, "parent")?,
                explicit: truth(g, required(g, &a, "explicit")?),
            },
        ));
    }
    // Namespace ownership maps are not read by the flat typed renderer; their
    // aliases and cyclic extras stay solely in the authoritative arena.
    for (key, a) in i.map(required(g, root, "notes")?)? {
        let key = i.text(&key)?;
        let class_name = i.optional(&a, "class")?;
        let index = match required(g, &a, "index")? {
            Atom::Number(n) => {
                m.set_public_note_index(key.clone(), *n);
                if n.is_finite() && *n >= 0. && n.fract() == 0. && *n < (usize::MAX as f64) {
                    *n as usize
                } else {
                    0
                }
            }
            _ if !class_name.as_deref().is_some_and(|k| ids.contains(k)) => 0,
            _ => {
                return Err(invalid(
                    "Class attached note index must be numeric for the native renderer",
                ))
            }
        };
        m.notes.push((
            key,
            ClassNote {
                id: i.field(&a, "id")?,
                class_name,
                text: i.field(&a, "text")?,
                index,
                parent: i.optional(&a, "parent")?,
            },
        ));
    }
    for a in i.array(required(g, root, "interfaces")?)? {
        m.interfaces.push(ClassInterface {
            id: i.field(&a, "id")?,
            label: i.field(&a, "label")?,
            class_id: i.field(&a, "classId")?,
        });
    }
    for (key, a) in i.map(required(g, root, "styleClasses")?)? {
        let Atom::String(id) = key else { continue };
        let Some(key) = g.strings[id as usize].well_formed() else {
            continue;
        };
        if !used.contains(&key) {
            continue;
        }
        m.style_classes.push((
            key,
            StyleClass {
                id: String::new(),
                styles: i.texts(required(g, &a, "styles")?)?,
                text_styles: vec![],
            },
        ));
    }
    for a in i.array(required(g, root, "relations")?)? {
        let spec = required(g, &a, "relation")?;
        let mut end = |name| -> Result<RelationEnd, PeleError> {
            Ok(match required(g, spec, name)? {
                Atom::String(_) => RelationEnd::None(i.field(spec, name)?),
                Atom::Number(n) if n.is_finite() && *n >= 0. && *n <= 255. && n.fract() == 0. => {
                    RelationEnd::Number(*n as u8)
                }
                Atom::Number(n) => RelationEnd::RawNumber(*n),
                _ => return Err(invalid("Class relation marker must be a number or string")),
            })
        };
        let type1 = end("type1")?;
        let type2 = end("type2")?;
        // Only this exact predicate is consumed by the concrete renderer. The graph
        // keeps the unmodified IEEE lineType, including NaN and signed zero.
        let line_type = u8::from(matches!(required(g,spec,"lineType")?,Atom::Number(n)if *n==1.));
        m.relations.push(ClassRelation {
            id1: i.field(&a, "id1")?,
            id2: i.field(&a, "id2")?,
            relation_title1: i.field(&a, "relationTitle1")?,
            relation_title2: i.field(&a, "relationTitle2")?,
            title: i.optional(&a, "title")?,
            relation: RelationType {
                type1,
                type2,
                line_type,
            },
        });
    }
    i.bytes = i
        .bytes
        .saturating_add(super::render::projected_style_bytes(&m));
    if i.bytes > 128 * 1024 * 1024 {
        return Err(limit());
    }
    fn walk<'a>(
        starts: impl Iterator<Item = &'a str>,
        next: impl Fn(&'a str) -> Option<&'a str>,
        i: &mut Import<'_>,
    ) -> Result<(), PeleError> {
        let mut done = HashSet::new();
        for start in starts {
            let mut path = HashSet::new();
            let mut at = Some(start);
            while let Some(id) = at {
                if done.contains(id) {
                    break;
                }
                if !path.insert(id) {
                    return Err(limit());
                }
                i.work = i.work.saturating_add(1);
                if i.work > 32_000_000 {
                    return Err(limit());
                }
                at = next(id);
            }
            done.extend(path);
        }
        Ok(())
    }
    // Hierarchical geometry resolves namespace IDs; the flat ancestor walk
    // instead resolves Map keys and stops at explicit namespaces. Edited IDs
    // can differ from retained keys, so those two graphs must not be conflated.
    let by_id: HashMap<_, _> = m
        .namespaces
        .iter()
        .map(|(_, n)| (n.id.as_str(), n.parent.as_deref()))
        .collect();
    walk(
        m.namespaces.iter().map(|(_, n)| n.id.as_str()),
        |id| by_id.get(id).copied().flatten(),
        &mut i,
    )?;
    let by_key: HashMap<_, _> = m
        .namespaces
        .iter()
        .map(|(key, n)| (key.as_str(), n))
        .collect();
    walk(
        m.classes
            .iter()
            .filter_map(|(_, n)| n.parent.as_deref())
            .chain(m.notes.iter().filter_map(|(_, n)| n.parent.as_deref())),
        |id| {
            by_key
                .get(id)
                .filter(|n| !n.explicit)
                .and_then(|n| n.parent.as_deref())
        },
        &mut i,
    )?;
    m.reindex_public();
    Ok(m)
}
fn rec(w: &mut GraphWriter, fields: Vec<(&str, Atom)>, prototype: Option<PrototypeId>) -> u32 {
    let properties = fields.into_iter().map(|(k, v)| w.property(k, v)).collect();
    w.node(if let Some(prototype) = prototype {
        Node::Instance {
            prototype,
            properties,
        }
    } else {
        Node::Object { properties }
    })
}
fn arr(w: &mut GraphWriter, values: impl IntoIterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .into_iter()
        .enumerate()
        .map(|(n, v)| w.property(&n.to_string(), v))
        .collect();
    w.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn text(w: &mut GraphWriter, s: &str) -> Atom {
    Atom::String(w.string(s))
}
fn texts(w: &mut GraphWriter, strings: &[String]) -> u32 {
    let values = strings.iter().map(|s| text(w, s)).collect::<Vec<_>>();
    arr(w, values)
}
fn member_node(w: &mut GraphWriter, m: &ClassMember) -> u32 {
    let fields = vec![
        ("id", text(w, &m.id)),
        ("memberType", text(w, &m.member_type)),
        ("visibility", text(w, &m.visibility)),
        ("classifier", text(w, &m.classifier)),
        ("parameters", text(w, &m.parameters)),
        ("returnType", text(w, &m.return_type)),
    ];
    rec(w, fields, Some(MEMBER))
}
fn initial(m: &ClassModel) -> Result<ModelGraph, PeleError> {
    // Reject unavoidable oversized string tables before allocating UTF16 copies.
    // These strings are exported verbatim and interned, so count each once.
    let mut seen = HashSet::new();
    let mut units = 0usize;
    for (key, namespace) in &m.namespaces {
        for value in [
            Some(key.as_str()),
            Some(namespace.id.as_str()),
            Some(namespace.label.as_str()),
            namespace.parent.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if seen.insert(value) {
                units = units.saturating_add(value.encode_utf16().count());
                if units > 16_000_000 {
                    return Err(PeleError::new(
                        "Public model graph exceeds work budget",
                        ErrorCode::Limit,
                    ));
                }
            }
        }
    }
    let mut w = GraphWriter::default();
    let line_type = rec(
        &mut w,
        vec![
            ("LINE", Atom::Number(0.)),
            ("DOTTED_LINE", Atom::Number(1.)),
        ],
        None,
    );
    let relation_type = rec(
        &mut w,
        vec![
            ("AGGREGATION", Atom::Number(0.)),
            ("EXTENSION", Atom::Number(1.)),
            ("COMPOSITION", Atom::Number(2.)),
            ("DEPENDENCY", Atom::Number(3.)),
            ("LOLLIPOP", Atom::Number(4.)),
        ],
        None,
    );
    let mut classes = vec![];
    let mut class_ids = HashMap::new();
    for (key, n) in &m.classes {
        let methods = n
            .methods
            .iter()
            .map(|m| Atom::Ref(member_node(&mut w, m)))
            .collect::<Vec<_>>();
        let methods = arr(&mut w, methods);
        let members = n
            .members
            .iter()
            .map(|m| Atom::Ref(member_node(&mut w, m)))
            .collect::<Vec<_>>();
        let members = arr(&mut w, members);
        let annotations = texts(&mut w, &n.annotations);
        let styles = texts(&mut w, &n.styles);
        let mut fields = vec![
            ("id", text(&mut w, &n.id)),
            ("type", text(&mut w, &n.generic)),
            ("label", text(&mut w, &n.label)),
            ("cssClasses", text(&mut w, &n.css_classes)),
            ("methods", Atom::Ref(methods)),
            ("members", Atom::Ref(members)),
            ("annotations", Atom::Ref(annotations)),
            ("styles", Atom::Ref(styles)),
        ];
        let mut optional = vec![];
        let mut seen = HashSet::new();
        if let Some(keys) = m.public_optional_fields(key) {
            for k in keys {
                seen.insert(k.as_str());
                optional.push(k.as_str());
            }
        }
        for (name, present) in [
            ("parent", n.parent.is_some()),
            ("link", n.link.is_some()),
            ("linkTarget", n.link_target.is_some()),
            ("haveCallback", n.have_callback.is_some()),
            ("tooltip", n.tooltip.is_some()),
        ] {
            if present && !seen.contains(name) {
                optional.push(name)
            }
        }
        for name in optional {
            let a = match name {
                "parent" => n.parent.as_deref().map(|s| text(&mut w, s)),
                "link" => n.link.as_deref().map(|s| text(&mut w, s)),
                "linkTarget" => n.link_target.as_deref().map(|s| text(&mut w, s)),
                "haveCallback" => n.have_callback.map(Atom::Bool),
                "tooltip" => n.tooltip.as_deref().map(|s| text(&mut w, s)),
                _ => None,
            };
            fields.push((name, a.unwrap_or(Atom::Undefined)));
        }
        let id = rec(&mut w, fields, None);
        class_ids.insert(key.as_str(), id);
        classes.push((text(&mut w, key), Atom::Ref(id)));
    }
    let classes = w.node(Node::Map {
        entries: classes,
        properties: vec![],
    });
    let mut notes = vec![];
    let mut note_ids = HashMap::new();
    for (key, n) in &m.notes {
        let class = n
            .class_name
            .as_deref()
            .map_or(Atom::Undefined, |s| text(&mut w, s));
        let mut fields = vec![
            ("id", text(&mut w, &n.id)),
            ("class", class),
            ("text", text(&mut w, &n.text)),
            ("index", Atom::Number(n.index as f64)),
        ];
        if let Some(p) = &n.parent {
            fields.push(("parent", text(&mut w, p)))
        }
        let id = rec(&mut w, fields, None);
        notes.push((text(&mut w, key), Atom::Ref(id)));
        note_ids.insert(key.as_str(), id);
    }
    let notes = w.node(Node::Map {
        entries: notes,
        properties: vec![],
    });
    let mut namespace_ids = HashMap::new();
    for (key, _) in &m.namespaces {
        namespace_ids.insert(key.as_str(), w.node(Node::Object { properties: vec![] }));
    }
    let mut namespaces = vec![];
    for (key, n) in &m.namespaces {
        let mut map = |names: &[String], ids: &HashMap<&str, u32>| {
            let entries = names
                .iter()
                .filter_map(|name| {
                    ids.get(name.as_str())
                        .map(|id| (text(&mut w, name), Atom::Ref(*id)))
                })
                .collect();
            w.node(Node::Map {
                entries,
                properties: vec![],
            })
        };
        let classes = map(&n.classes, &class_ids);
        let notes = map(&n.notes, &note_ids);
        let children = map(&n.children, &namespace_ids);
        let parent = n
            .parent
            .as_deref()
            .map_or(Atom::Undefined, |s| text(&mut w, s));
        let fields = vec![
            ("id", text(&mut w, &n.id)),
            ("label", text(&mut w, &n.label)),
            ("classes", Atom::Ref(classes)),
            ("notes", Atom::Ref(notes)),
            ("children", Atom::Ref(children)),
            ("parent", parent),
            ("explicit", Atom::Bool(n.explicit)),
        ];
        let properties = fields.into_iter().map(|(k, v)| w.property(k, v)).collect();
        let id = namespace_ids[key.as_str()];
        // GraphWriter exposes no mutation; replace the placeholder after finishing.
        namespaces.push((key.as_str(), id, properties));
    }
    let namespace_entries = m
        .namespaces
        .iter()
        .map(|(key, _)| (text(&mut w, key), Atom::Ref(namespace_ids[key.as_str()])))
        .collect();
    let namespace_map = w.node(Node::Map {
        entries: namespace_entries,
        properties: vec![],
    });
    let mut relations = vec![];
    for r in &m.relations {
        let endpoint = |w: &mut GraphWriter, e: &RelationEnd| match e {
            RelationEnd::Number(n) => Atom::Number(f64::from(*n)),
            RelationEnd::RawNumber(n) => Atom::Number(*n),
            RelationEnd::None(s) => text(w, s),
        };
        let fields = vec![
            ("type1", endpoint(&mut w, &r.relation.type1)),
            ("type2", endpoint(&mut w, &r.relation.type2)),
            ("lineType", Atom::Number(f64::from(r.relation.line_type))),
        ];
        let relation = rec(&mut w, fields, None);
        let mut fields = vec![
            ("id1", text(&mut w, &r.id1)),
            ("id2", text(&mut w, &r.id2)),
            ("relation", Atom::Ref(relation)),
            ("relationTitle1", text(&mut w, &r.relation_title1)),
            ("relationTitle2", text(&mut w, &r.relation_title2)),
        ];
        if let Some(t) = &r.title {
            fields.push(("title", text(&mut w, t)))
        }
        relations.push(Atom::Ref(rec(&mut w, fields, None)));
    }
    let relations = arr(&mut w, relations);
    let interfaces = m
        .interfaces
        .iter()
        .map(|n| {
            let fields = vec![
                ("id", text(&mut w, &n.id)),
                ("label", text(&mut w, &n.label)),
                ("classId", text(&mut w, &n.class_id)),
            ];
            Atom::Ref(rec(&mut w, fields, None))
        })
        .collect::<Vec<_>>();
    let interfaces = arr(&mut w, interfaces);
    let mut style_classes = vec![];
    for (key, n) in &m.style_classes {
        let styles = texts(&mut w, &n.styles);
        let text_styles = texts(&mut w, &n.text_styles);
        let fields = vec![
            ("id", text(&mut w, &n.id)),
            ("styles", Atom::Ref(styles)),
            ("textStyles", Atom::Ref(text_styles)),
        ];
        let id = rec(&mut w, fields, None);
        style_classes.push((text(&mut w, key), Atom::Ref(id)));
    }
    let style_classes = w.node(Node::Map {
        entries: style_classes,
        properties: vec![],
    });
    let stack = texts(&mut w, &m.namespace_stack);
    let mut tags = vec![];
    for (key, names) in &m.with_css_class {
        let values = names
            .iter()
            .filter_map(|name| class_ids.get(name.as_str()).map(|id| Atom::Ref(*id)))
            .collect();
        let set = w.node(Node::Set {
            values,
            properties: vec![],
        });
        tags.push((text(&mut w, key), Atom::Ref(set)));
    }
    let tags = w.node(Node::Map {
        entries: tags,
        properties: vec![],
    });
    let mut fields = vec![
        ("type", text(&mut w, "class")),
        ("lineType", Atom::Ref(line_type)),
        ("relationType", Atom::Ref(relation_type)),
        ("direction", text(&mut w, &m.direction)),
        ("classes", Atom::Ref(classes)),
        ("relations", Atom::Ref(relations)),
        ("notes", Atom::Ref(notes)),
        ("namespaces", Atom::Ref(namespace_map)),
        ("interfaces", Atom::Ref(interfaces)),
        ("styleClasses", Atom::Ref(style_classes)),
    ];
    for (key, value) in [
        ("title", &m.title),
        ("accTitle", &m.acc_title),
        ("accDescr", &m.acc_descr),
    ] {
        fields.push((
            key,
            value
                .as_deref()
                .map_or(Atom::Undefined, |s| text(&mut w, s)),
        ))
    }
    fields.extend([
        ("namespaceStack", Atom::Ref(stack)),
        ("withCssClass", Atom::Ref(tags)),
    ]);
    let root = rec(&mut w, fields, Some(PROTO));
    let mut graph = w.finish(Atom::Ref(root))?;
    for (_, id, properties) in namespaces {
        graph.nodes[id as usize] = Node::Object { properties };
    }
    graph.validate()?;
    Ok(graph)
}
fn construct_default() -> Result<ClassPublicState, PeleError> {
    Class.create_public(&ClassModel::new(None))
}
impl PublicDiagramModule for Class {
    type PublicState = ClassPublicState;
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::class())
    }
    fn detached_public_render(
        &self,
        state: &ClassPublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::class(
            super::live_phase::ClassDetachedInput::new(state.model.as_ref().err()),
        ))
    }
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        match prototype {
            PROTO => Some(PublicConstructorFactory::ignoring_arguments::<Self>(
                PROTO,
                construct_default,
            )),
            MEMBER => Some(member_constructor::factory()),
            _ => None,
        }
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(ClassPublicState { graph, model })
    }
    fn create_public(&self, m: &ClassModel) -> Result<ClassPublicState, PeleError> {
        let graph = initial(m)?;
        let model = imported(&graph);
        Ok(ClassPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &ClassPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut ClassPublicState, g: ModelGraph) -> Result<(), PeleError> {
        g.validate()?;
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn active_public_operation(
        &self,
        state: &mut ClassPublicState,
        operation: crate::compat::public::PublicActiveOperation,
        budget: &mut crate::compat::public::PublicActiveBudget,
    ) -> Result<crate::compat::public::PublicActiveValue, PublicError> {
        use crate::compat::public::{PublicActiveOperation, PublicActiveValue};
        let mut context = Context::new(&mut state.graph);
        context.work = 1_000_000 - budget.remaining;
        let result = match operation {
            PublicActiveOperation::Builtin(call) if call.method.0 != 26 => (|| {
                context.validate(&call.receiver)?;
                for atom in &call.arguments {
                    context.validate(atom)?;
                }
                context.invoke(call).map(PublicActiveValue::Atom)
            })(),
            operation => operation.run(&mut context),
        };
        budget.remaining = 1_000_000 - context.work.min(1_000_000);
        state.model = imported(&state.graph);
        result
    }
    fn detached_public_invocation(
        &self,
        call: MethodCall,
    ) -> Option<crate::compat::public::DetachedPublicInvocation> {
        display_continuation::prepare(call)
    }
    fn invoke_public(
        &self,
        s: &mut ClassPublicState,
        c: MethodCall,
        _: &Object,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let mut context = Context::new(&mut s.graph);
        context.begin_sync()?;
        let result = context.invoke_live(c, host);
        s.model = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut ClassPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render_public_operation(s, c, o, h)
            .map_err(legacy_error)
    }
    fn render_public_operation(
        &self,
        s: &mut ClassPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        if needs_live(&s.graph) {
            render_live_state(s, c, o, h, false)
        } else {
            self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
                .map_err(Into::into)
        }
    }
    fn render_public_raw_operation(
        &self,
        s: &mut ClassPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        if needs_live(&s.graph) {
            return render_live_state(s, c, o, h, true);
        }
        self.render_raw(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
            .map_err(Into::into)
    }
    fn render_public_operation_with_output_limit(
        &self,
        s: &mut ClassPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
        limit: OutputLimit,
    ) -> Result<Rendered, PublicError> {
        if needs_live(&s.graph) {
            return render_live_state(s, c, o, h, true);
        }
        self.render_with_output_limit(s.model.as_ref().map_err(Clone::clone)?, c, o, h, limit)
            .map_err(Into::into)
    }
}
impl Context<'_> {
    fn helper(
        &mut self,
        r: &Atom,
        name: &str,
        id: u32,
        args: Vec<Atom>,
    ) -> Result<Atom, PublicError> {
        let value = self.read(r, name)?;
        let prototype = if id >= 24 { MEMBER } else { PROTO };
        let native = matches!(r,Atom::Ref(n)if matches!(self.g.nodes[*n as usize],Node::Instance{prototype:p,..}if p==prototype));
        let own = match r {
            Atom::Ref(n) => self.own(*n, name)?.is_some(),
            _ => false,
        };
        if matches!(value,Atom::BuiltinFunction(method)if method==id) || native && !own {
            self.invoke(MethodCall {
                method: MethodId(id),
                receiver: r.clone(),
                arguments: args,
            })
        } else if matches!(value, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
            Err(invalid(&format!(
                "Class overridden this.{name} callback dispatch is unavailable"
            ))
            .into())
        } else {
            Err(type_error(&format!("this.{name} is not a function")))
        }
    }
    fn split_name(&mut self, a: &Atom) -> Result<(Atom, Atom), PublicError> {
        let units = self.string_units(a, "id.indexOf")?;
        if let Some(at) = units.iter().position(|c| *c == 126).filter(|p| *p > 0) {
            let end = units[at + 1..]
                .iter()
                .position(|c| *c == 126)
                .map_or(units.len(), |n| n + at + 1);
            Ok((
                self.intern(JsString(units[..at].to_vec()))?,
                self.intern(JsString(units[at + 1..end].to_vec()))?,
            ))
        } else {
            Ok((a.clone(), self.text("")?))
        }
    }
    fn lookup_id(&mut self, a: &Atom) -> Result<Atom, PublicError> {
        let units = self.string_units(a, "id.charCodeAt")?;
        if units.first().is_some_and(|c| (48..=57).contains(c)) {
            let mut prefixed = "classId-".encode_utf16().collect::<Vec<_>>();
            prefixed.extend(units);
            let id = self.intern(JsString(prefixed))?;
            Ok(self.split_name(&id)?.0)
        } else {
            Ok(self.split_name(a)?.0)
        }
    }
    fn split(&mut self, a: &Atom, separator: u16, expr: &str) -> Result<Vec<Atom>, PublicError> {
        let units = self.string_units(a, expr)?;
        units
            .split(|c| *c == separator)
            .map(|s| self.intern(JsString(s.to_vec())))
            .collect()
    }
    fn trim(&mut self, a: &Atom, expr: &str) -> Result<Atom, PublicError> {
        let units = self.string_units(a, expr)?;
        self.intern(JsString(trimmed(&units).to_vec()))
    }
    fn delete(&mut self, id: u32, key: &str) -> Result<(), PublicError> {
        self.dirty(id);
        self.tick(self.g.nodes[id as usize].properties().len())?;
        let at = self.g.nodes[id as usize]
            .properties()
            .iter()
            .position(|p| self.g.strings[p.key as usize].is(key));
        if let Some(at) = at {
            if self.g.nodes[id as usize].properties()[at].flags & 4 == 0 {
                return Err(type_error(&format!(
                    "Cannot delete property '{key}' of [object Array]"
                )));
            }
            self.g.nodes[id as usize].properties_mut().remove(at);
        }
        Ok(())
    }
    fn length(&mut self, id: u32, length: u32) -> Result<(), PublicError> {
        self.dirty(id);
        if let Some(p) = self.g.nodes[id as usize]
            .properties()
            .iter()
            .find(|p| self.g.strings[p.key as usize].is("length"))
        {
            if p.flags & 2 == 0 {
                return Err(type_error(
                    "Cannot assign to read only property 'length' of object '[object Array]'",
                ));
            }
        }
        let Node::Array {
            length: len,
            properties,
        } = &mut self.g.nodes[id as usize]
        else {
            unreachable!()
        };
        *len = length;
        for p in properties {
            if self.g.strings[p.key as usize].is("length") {
                p.value = Atom::Number(f64::from(length))
            }
        }
        Ok(())
    }
    fn reverse(&mut self, a: &Atom) -> Result<(), PublicError> {
        let id = self.native(a, "reverse", "members.reverse", 1)?;
        let Node::Array { length, .. } = self.g.nodes[id as usize] else {
            unreachable!()
        };
        for lower in 0..length / 2 {
            self.tick(1)?;
            let upper = length - lower - 1;
            let lo = lower.to_string();
            let hi = upper.to_string();
            let l = self.own(id, &lo)?;
            let u = self.own(id, &hi)?;
            match (l, u) {
                (Some(l), Some(u)) => {
                    self.set(a, &lo, u)?;
                    self.set(a, &hi, l)?
                }
                (None, Some(u)) => {
                    self.set(a, &lo, u)?;
                    self.delete(id, &hi)?
                }
                (Some(l), None) => {
                    self.delete(id, &lo)?;
                    self.set(a, &hi, l)?
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn new_member(&mut self, input: Atom, method: bool) -> Result<Atom, PublicError> {
        let empty = self.text("")?;
        let ty = self.text(if method { "method" } else { "attribute" })?;
        let a = self.node(Node::Instance {
            prototype: MEMBER,
            properties: vec![],
        })?;
        for (k, v) in [
            ("id", empty.clone()),
            ("memberType", ty),
            ("visibility", empty.clone()),
            ("classifier", empty.clone()),
            ("parameters", empty.clone()),
            ("returnType", empty),
        ] {
            self.set(&a, k, v)?
        }
        let parsed = if method {
            self.helper(&a, "parseMethod", 24, vec![input.clone()])?
        } else {
            Atom::Bool(false)
        };
        if !truth(self.g, &parsed) {
            self.helper(&a, "parseAttribute", 25, vec![input])?;
        }
        let id = self.read(&a, "id")?;
        let units = self.string_units(&id, "this.id.startsWith")?;
        let mut id = trimmed(&units).to_vec();
        if units.first() == Some(&32) {
            id.insert(0, 32)
        }
        let id = self.intern(JsString(id))?;
        self.set(&a, "id", id)?;
        Ok(a)
    }
    fn add_class(&mut self, r: &Atom, id: Atom) -> Result<Atom, PublicError> {
        let (name, ty) = self.split_name(&id)?;
        let map = self.read(r, "classes")?;
        if matches!(
            self.map_get(&map, &name, true, "this.classes.has")?,
            Atom::Bool(true)
        ) {
            return Ok(Atom::Undefined);
        }
        let default = self.text("default")?;
        let methods = self.array(vec![])?;
        let members = self.array(vec![])?;
        let annotations = self.array(vec![])?;
        let styles = self.array(vec![])?;
        let node = self.record(vec![
            ("id", name.clone()),
            ("type", ty),
            ("label", name.clone()),
            ("cssClasses", default.clone()),
            ("methods", methods),
            ("members", members),
            ("annotations", annotations),
            ("styles", styles),
        ])?;
        let map = self.read(r, "classes")?;
        self.map_set(&map, name, node.clone(), "this.classes.set")?;
        self.helper(r, "tag", 2, vec![node, default])?;
        Ok(Atom::Undefined)
    }
    fn tag(&mut self, r: &Atom, node: Atom, css: Atom) -> Result<Atom, PublicError> {
        let map = self.read(r, "withCssClass")?;
        let tagged = self.map_get(&map, &css, false, "this.withCssClass.get")?;
        if truth(self.g, &tagged) {
            self.set_add(&tagged, node)?;
        } else {
            let values = vec![node];
            let set = self.node(Node::Set {
                values,
                properties: vec![],
            })?;
            let map = self.read(r, "withCssClass")?;
            self.map_set(&map, css, set, "this.withCssClass.set")?;
        }
        Ok(Atom::Undefined)
    }
    fn ensure(&mut self, r: &Atom, id: Atom) -> Result<Atom, PublicError> {
        self.helper(r, "addClass", 1, vec![id.clone()])?;
        let map = self.read(r, "classes")?;
        let name = self.split_name(&id)?.0;
        self.map_get(&map, &name, false, "this.classes.get")
    }
    fn add_interface(
        &mut self,
        r: &Atom,
        label: Atom,
        class_id: Atom,
    ) -> Result<Atom, PublicError> {
        let interfaces = self.read(r, "interfaces")?;
        let length = self.read(&interfaces, "length")?;
        let mut units = "interface".encode_utf16().collect::<Vec<_>>();
        units.extend(self.coercion(&length)?);
        let id = self.intern(JsString(units))?;
        let value = self.record(vec![
            ("id", id.clone()),
            ("label", label),
            ("classId", class_id),
        ])?;
        let interfaces = self.read(r, "interfaces")?;
        self.push(&interfaces, value, "this.interfaces.push")?;
        Ok(id)
    }
    fn css_class(&mut self, r: &Atom, ids: Atom, name: Atom) -> Result<Atom, PublicError> {
        for id in self.split(&ids, 44, "ids.split")? {
            let map = self.read(r, "classes")?;
            let key = self.lookup_id(&id)?;
            let node = self.map_get(&map, &key, false, "this.classes.get")?;
            if matches!(node, Atom::Undefined) {
                continue;
            }
            let current = self.read(&node, "cssClasses")?;
            let mut units = self.coercion(&current)?;
            units.push(32);
            units.extend(self.coercion(&name)?);
            let value = self.intern(JsString(units))?;
            self.set(&node, "cssClasses", value)?;
            for word in self.split(&name, 32, "className.split")? {
                self.helper(r, "tag", 2, vec![node.clone(), word])?;
            }
        }
        Ok(Atom::Undefined)
    }
    fn define_class(&mut self, r: &Atom, ids: Atom, styles: Atom) -> Result<Atom, PublicError> {
        let mut ids = self.iter(&ids)?;
        while let Some(id) = self.next(&mut ids)? {
            let map = self.read(r, "styleClasses")?;
            let mut def = self.map_get(&map, &id, false, "this.styleClasses.get")?;
            if matches!(def, Atom::Undefined) {
                let styles = self.array(vec![])?;
                let text = self.array(vec![])?;
                def = self.record(vec![
                    ("id", id.clone()),
                    ("styles", styles),
                    ("textStyles", text),
                ])?;
                let map = self.read(r, "styleClasses")?;
                self.map_set(&map, id.clone(), def.clone(), "this.styleClasses.set")?
            }
            if !truth(self.g, &styles) {
                continue;
            }
            let mut iter = self.iter(&styles)?;
            while let Some(style) = self.next(&mut iter)? {
                if self.includes_color(&style)? {
                    let list = self.read(&def, "textStyles")?;
                    self.native(&list, "push", "styleClass.textStyles.push", 1)?;
                    let units = self.string_units(&style, "s.replace")?;
                    let mut replaced = units.clone();
                    if let Some(at) = units.windows(4).position(|w| w == [102, 105, 108, 108]) {
                        replaced.splice(at..at + 4, "bgFill".encode_utf16());
                    }
                    let style = self.intern(JsString(replaced))?;
                    self.push(&list, style, "styleClass.textStyles.push")?
                }
                let list = self.read(&def, "styles")?;
                self.push(&list, style, "styleClass.styles.push")?;
            }
            let map = self.read(r, "withCssClass")?;
            let tagged = self.map_get(&map, &id, false, "this.withCssClass.get")?;
            if matches!(tagged, Atom::Undefined) {
                continue;
            }
            let list = self.native(&styles, "flatMap", "style.flatMap", 1)?;
            let Node::Array { length, .. } = self.g.nodes[list as usize] else {
                unreachable!()
            };
            let mut parts = vec![];
            for n in 0..length {
                self.tick(1)?;
                if let Some(style) = self.own(list, &n.to_string())? {
                    parts.extend(self.split(&style, 44, "s.split")?);
                    if parts.len() > 1_000_000 {
                        return Err(limit().into());
                    }
                }
            }
            let mut nodes = self.iter(&tagged)?;
            while let Some(node) = self.next(&mut nodes)? {
                for part in &parts {
                    let list = self.read(&node, "styles")?;
                    self.push(&list, part.clone(), "node.styles.push")?;
                }
            }
        }
        Ok(Atom::Undefined)
    }
    fn namespace(&mut self, r: &Atom, id: Atom, label: Atom) -> Result<Atom, PublicError> {
        let stack = self.read(r, "namespaceStack")?;
        let length = self.read(&stack, "length")?;
        let at = match length {
            Atom::Number(n) => crate::compat::svg::js_num(n - 1.),
            _ => {
                return Err(invalid(
                    "Class nonnumeric namespace stack length coercion is unavailable",
                )
                .into())
            }
        };
        let prefix = self.read(&stack, &at)?;
        let qualified = if truth(self.g, &prefix) {
            let mut units = self.coercion(&prefix)?;
            units.push(46);
            units.extend(self.coercion(&id)?);
            self.intern(JsString(units))?
        } else {
            id
        };
        let stack = self.read(r, "namespaceStack")?;
        self.push(&stack, qualified.clone(), "this.namespaceStack.push")?;
        let map = self.read(r, "namespaces")?;
        let existing = self.map_get(&map, &qualified, false, "this.namespaces.get")?;
        if truth(self.g, &existing) {
            self.set(&existing, "explicit", Atom::Bool(true))?;
            if truth(self.g, &label) {
                self.set(&existing, "label", label)?
            }
            return Ok(qualified);
        }
        let units = self.string_units(&qualified, "qualified.indexOf")?;
        let mut parent = if truth(self.g, &prefix) {
            prefix.clone()
        } else {
            Atom::Undefined
        };
        let mut from = if truth(self.g, &prefix) {
            self.string_units(&prefix, "prefix.length")?.len() + 1
        } else {
            0
        };
        loop {
            self.tick(1)?;
            let dot = units
                .get(from..)
                .and_then(|s| s.iter().position(|c| *c == 46))
                .map(|n| n + from);
            let leaf = dot.is_none();
            let end = dot.unwrap_or(units.len());
            let current = self.intern(JsString(units[..end].to_vec()))?;
            let map = self.read(r, "namespaces")?;
            let mut node = self.map_get(&map, &current, false, "this.namespaces.get")?;
            if matches!(node, Atom::Undefined) {
                let part = self.intern(JsString(units[from..end].to_vec()))?;
                let label = if leaf && truth(self.g, &label) {
                    label.clone()
                } else {
                    part
                };
                let classes = self.node(Node::Map {
                    entries: vec![],
                    properties: vec![],
                })?;
                let notes = self.node(Node::Map {
                    entries: vec![],
                    properties: vec![],
                })?;
                let children = self.node(Node::Map {
                    entries: vec![],
                    properties: vec![],
                })?;
                node = self.record(vec![
                    ("id", current.clone()),
                    ("label", label),
                    ("classes", classes),
                    ("notes", notes),
                    ("children", children),
                    ("parent", parent.clone()),
                    ("explicit", Atom::Bool(leaf)),
                ])?;
                let map = self.read(r, "namespaces")?;
                self.map_set(&map, current.clone(), node.clone(), "this.namespaces.set")?;
            } else if leaf {
                self.set(&node, "explicit", Atom::Bool(true))?;
            }
            let parent_node = if truth(self.g, &parent) {
                let map = self.read(r, "namespaces")?;
                self.map_get(&map, &parent, false, "this.namespaces.get")?
            } else {
                Atom::Undefined
            };
            if truth(self.g, &parent_node) {
                let children = self.read(&parent_node, "children")?;
                if !matches!(
                    self.map_get(&children, &current, true, "parent.children.has")?,
                    Atom::Bool(true)
                ) {
                    let children = self.read(&parent_node, "children")?;
                    self.map_set(
                        &children,
                        current.clone(),
                        node.clone(),
                        "parent.children.set",
                    )?
                }
                let old = self.read(&node, "parent")?;
                if matches!(old, Atom::Undefined | Atom::Null) {
                    self.set(&node, "parent", parent.clone())?
                }
            }
            if leaf {
                return Ok(qualified);
            }
            parent = current;
            from = end + 1;
        }
    }
    fn parse_method(&mut self, r: &Atom, input: Atom) -> Result<Atom, PublicError> {
        let length = self.read(&input, "length")?;
        let Atom::Number(length) = length else {
            let attribute = self.text("attribute")?;
            self.set(r, "memberType", attribute)?;
            return Ok(Atom::Bool(false));
        };
        if length > 1_000_000. {
            return Err(limit().into());
        }
        if !(length > 0.) {
            let attribute = self.text("attribute")?;
            self.set(r, "memberType", attribute)?;
            return Ok(Atom::Bool(false));
        }
        let units = self.string_units(&input, "input.charCodeAt")?;
        let mut start = 0;
        while start < units.len() {
            self.tick(1)?;
            let end = units[start..]
                .iter()
                .position(|c| matches!(*c, 10 | 13 | 0x2028 | 0x2029))
                .map_or(units.len(), |n| n + start);
            let close = units[start..end]
                .iter()
                .rposition(|c| *c == 41)
                .map(|n| n + start);
            let open = close
                .and_then(|close| {
                    units[start..close]
                        .iter()
                        .rposition(|c| *c == 40)
                        .map(|n| n + start)
                })
                .filter(|open| *open > start);
            if let (Some(close), Some(open)) = (close, open) {
                let visible = matches!(units[start], 35 | 43 | 126 | 45) && open > start + 1;
                if visible {
                    let v = self.intern(JsString(vec![units[start]]))?;
                    self.set(r, "visibility", v)?
                }
                let id = self.intern(JsString(
                    units[if visible { start + 1 } else { start }..open].to_vec(),
                ))?;
                self.set(r, "id", id)?;
                let params = self.intern(JsString(trimmed(&units[open + 1..close]).to_vec()))?;
                self.set(r, "parameters", params)?;
                let mut rest = close + 1;
                let mut classifier = vec![];
                if let Some(next) = units.get(rest) {
                    if matches!(*next, 36 | 42) || space(*next) {
                        if !space(*next) {
                            classifier.push(*next)
                        }
                        rest += 1;
                    }
                }
                let stop = units[rest..]
                    .iter()
                    .position(|c| matches!(*c, 10 | 13 | 0x2028 | 0x2029))
                    .map_or(units.len(), |n| n + rest);
                let mut return_type = trimmed(&units[rest..stop]).to_vec();
                let value = self.intern(JsString(return_type.clone()))?;
                self.set(r, "returnType", value)?;
                if classifier.is_empty()
                    && return_type.last().is_some_and(|c| matches!(*c, 36 | 42))
                {
                    classifier.push(return_type.pop().unwrap());
                    let value = self.intern(JsString(return_type))?;
                    self.set(r, "returnType", value)?
                }
                let value = self.intern(JsString(classifier))?;
                self.set(r, "classifier", value)?;
                return Ok(Atom::Bool(true));
            }
            start = end + 1;
        }
        let attribute = self.text("attribute")?;
        self.set(r, "memberType", attribute)?;
        Ok(Atom::Bool(false))
    }
    fn parse_attribute(&mut self, r: &Atom, input: Atom) -> Result<Atom, PublicError> {
        let units = self.string_units(&input, "input.slice")?;
        if units
            .first()
            .is_some_and(|c| matches!(*c, 35 | 43 | 126 | 45))
        {
            let value = self.intern(JsString(vec![units[0]]))?;
            self.set(r, "visibility", value)?
        }
        if units.last().is_some_and(|c| matches!(*c, 36 | 42)) {
            let value = self.intern(JsString(vec![*units.last().unwrap()]))?;
            self.set(r, "classifier", value)?
        }
        let visibility = self.read(r, "visibility")?;
        let classifier = self.read(r, "classifier")?;
        let start = usize::from(
            !matches!(visibility,Atom::String(id)if self.g.strings[id as usize].is("")),
        )
        .min(units.len());
        let end = if matches!(classifier,Atom::String(id)if self.g.strings[id as usize].is("")) {
            units.len()
        } else {
            units.len().saturating_sub(1)
        };
        // String.substring swaps reversed endpoints, unlike slice.
        let id = self.intern(JsString(units[start.min(end)..start.max(end)].to_vec()))?;
        self.set(r, "id", id)?;
        Ok(Atom::Undefined)
    }
    fn generic(&mut self, a: &Atom) -> Result<Vec<u16>, PublicError> {
        let units = self.string_units(a, "input.includes")?;
        fn pair(mut s: Vec<u16>) -> Vec<u16> {
            let count = s.iter().filter(|c| **c == 126).count();
            if count <= 1 {
                return s;
            }
            let lead = count % 2 != 0 && s.first() == Some(&126);
            let positions = s
                .iter()
                .enumerate()
                .filter_map(|(i, c)| (*c == 126 && (!lead || i > 0)).then_some(i))
                .collect::<Vec<_>>();
            for n in 0..positions.len() / 2 {
                s[positions[n]] = 60;
                s[positions[positions.len() - 1 - n]] = 62;
            }
            s
        }
        let mut sets = vec![];
        let mut start = 0;
        for (n, c) in units.iter().enumerate() {
            if *c == 44 {
                sets.push(units[start..n].to_vec());
                sets.push(vec![44]);
                start = n + 1
            }
        }
        sets.push(units[start..].to_vec());
        let mut out: Vec<Vec<u16>> = vec![];
        let mut n = 0;
        while n < sets.len() {
            if sets[n] == [44]
                && n > 0
                && n + 1 < sets.len()
                && sets[n - 1].iter().filter(|c| **c == 126).count() == 1
                && sets[n + 1].iter().filter(|c| **c == 126).count() == 1
            {
                out.pop();
                let mut value = sets[n - 1].clone();
                value.push(44);
                value.extend(&sets[n + 1]);
                out.push(pair(value));
                n += 2;
            } else {
                out.push(pair(sets[n].clone()));
                n += 1
            }
        }
        Ok(out.into_iter().flatten().collect())
    }
    fn display_text(&mut self, r: &Atom) -> Result<Atom, PublicError> {
        let visibility = self.read(r, "visibility")?;
        let id = self.read(r, "id")?;
        let generic = self.generic(&id)?;
        let mut display = self.coercion(&visibility)?;
        display.extend(generic);
        let member_type = self.read(r, "memberType")?;
        if matches!(member_type,Atom::String(id)if self.g.strings[id as usize].is("method")) {
            let parameters = self.read(r, "parameters")?;
            let params = self.trim(&parameters, "this.parameters.trim")?;
            display.push(40);
            display.extend(self.generic(&params)?);
            display.push(41);
            let return_type = self.read(r, "returnType")?;
            if truth(self.g, &return_type) {
                display.extend(" : ".encode_utf16());
                display.extend(self.generic(&return_type)?);
            }
        }
        self.intern(JsString(trimmed(&display).to_vec()))
    }
    fn display(&mut self, r: &Atom) -> Result<Atom, PublicError> {
        let display = self.display_text(r)?;
        let css = self.helper(r, "parseClassifier", 27, vec![])?;
        self.record(vec![("displayText", display), ("cssStyle", css)])
    }
    fn invoke(&mut self, c: MethodCall) -> Result<Atom, PublicError> {
        self.tick(1)?;
        let arg = |n| c.arguments.get(n).cloned().unwrap_or(Atom::Undefined);
        let r = &c.receiver;
        match c.method.0 {
            1 => self.add_class(r, arg(0)),
            2 => self.tag(r, arg(0), arg(1)),
            3 => self.ensure(r, arg(0)),
            4 => {
                let node = self.helper(r, "ensure", 3, vec![arg(0)])?;
                self.set(&node, "label", arg(1))?;
                Ok(Atom::Undefined)
            }
            5 => {
                let relation = arg(0);
                let spec = self.read(&relation, "relation")?;
                let a = self.read(&spec, "type1")?;
                let b = self.read(&spec, "type2")?;
                if matches!(a,Atom::Number(n)if n==4.)
                    && matches!(b,Atom::String(id)if self.g.strings[id as usize].is("none"))
                {
                    let id2 = self.read(&relation, "id2")?;
                    self.helper(r, "addClass", 1, vec![id2.clone()])?;
                    let id1 = self.read(&relation, "id1")?;
                    let interface = self.helper(r, "addInterface", 6, vec![id1, id2])?;
                    self.set(&relation, "id1", interface)?;
                } else if matches!(b,Atom::Number(n)if n==4.)
                    && matches!(a,Atom::String(id)if self.g.strings[id as usize].is("none"))
                {
                    let id1 = self.read(&relation, "id1")?;
                    self.helper(r, "addClass", 1, vec![id1.clone()])?;
                    let id2 = self.read(&relation, "id2")?;
                    let interface = self.helper(r, "addInterface", 6, vec![id2, id1])?;
                    self.set(&relation, "id2", interface)?;
                } else {
                    let id1 = self.read(&relation, "id1")?;
                    self.helper(r, "addClass", 1, vec![id1])?;
                    let id2 = self.read(&relation, "id2")?;
                    self.helper(r, "addClass", 1, vec![id2])?;
                }
                for key in ["id1", "id2"] {
                    let id = self.read(&relation, key)?;
                    let value = self.split_name(&id)?.0;
                    self.set(&relation, key, value)?
                }
                for key in ["relationTitle1", "relationTitle2"] {
                    let title = self.read(&relation, key)?;
                    let value = self.trim(&title, &format!("relation.{key}.trim"))?;
                    self.set(&relation, key, value)?
                }
                let list = self.read(r, "relations")?;
                self.push(&list, relation, "this.relations.push")?;
                Ok(Atom::Undefined)
            }
            6 => self.add_interface(r, arg(0), arg(1)),
            7 => {
                let node = self.helper(r, "ensure", 3, vec![arg(0)])?;
                let annotations = self.read(&node, "annotations")?;
                self.push(&annotations, arg(1), "node.annotations.push")?;
                Ok(Atom::Undefined)
            }
            8 => {
                let node = self.helper(r, "ensure", 3, vec![arg(0)])?;
                let member = arg(1);
                if !matches!(member, Atom::String(_)) {
                    return Ok(Atom::Undefined);
                }
                let text = self.trim(&member, "member.trim")?;
                let units = self.string_units(&text, "text.startsWith")?;
                if units.starts_with(&[60, 60]) && units.ends_with(&[62, 62]) {
                    let start = 2.min(units.len());
                    let end = units.len().saturating_sub(2);
                    let value =
                        self.intern(JsString(units[start.min(end)..start.max(end)].to_vec()))?;
                    let list = self.read(&node, "annotations")?;
                    self.push(&list, value, "node.annotations.push")?;
                } else if units.iter().position(|c| *c == 41).is_some_and(|n| n > 0) {
                    let member = self.new_member(text, true)?;
                    let ty = self.read(&member, "memberType")?;
                    let list = self.read(
                        &node,
                        if matches!(ty,Atom::String(id)if self.g.strings[id as usize].is("method"))
                        {
                            "methods"
                        } else {
                            "members"
                        },
                    )?;
                    self.push(&list, member, "node.members.push")?;
                } else if !units.is_empty() {
                    let list = self.read(&node, "members")?;
                    self.native(&list, "push", "node.members.push", 1)?;
                    let member = self.new_member(text, false)?;
                    self.push(&list, member, "node.members.push")?;
                }
                Ok(Atom::Undefined)
            }
            9 => {
                let members = arg(1);
                if !matches!(members,Atom::Ref(id)if matches!(self.g.nodes[id as usize],Node::Array{..}))
                {
                    return Ok(Atom::Undefined);
                }
                self.reverse(&members)?;
                let mut iter = self.iter(&members)?;
                while let Some(member) = self.next(&mut iter)? {
                    self.helper(r, "addMember", 8, vec![arg(0), member])?;
                }
                Ok(Atom::Undefined)
            }
            10 => {
                let notes = self.read(r, "notes")?;
                let index = self.read(&notes, "size")?;
                let mut units = "note".encode_utf16().collect::<Vec<_>>();
                units.extend(self.coercion(&index)?);
                let id = self.intern(JsString(units))?;
                let node = self.record(vec![
                    ("id", id.clone()),
                    ("class", arg(1)),
                    ("text", arg(0)),
                    ("index", index),
                ])?;
                let notes = self.read(r, "notes")?;
                self.map_set(&notes, id.clone(), node, "this.notes.set")?;
                Ok(id)
            }
            11 => {
                let label = arg(0);
                let units = self.string_units(&label, "label.startsWith")?;
                self.intern(JsString(
                    trimmed(if units.first() == Some(&58) {
                        &units[1..]
                    } else {
                        &units
                    })
                    .to_vec(),
                ))
            }
            12 => self.css_class(r, arg(0), arg(1)),
            13 => self.define_class(r, arg(0), arg(1)),
            14 => {
                if matches!(arg(1), Atom::Undefined) {
                    return Ok(Atom::Undefined);
                }
                for id in self.split(&arg(0), 44, "ids.split")? {
                    let map = self.read(r, "classes")?;
                    let id = self.split_name(&id)?.0;
                    let node = self.map_get(&map, &id, false, "this.classes.get")?;
                    if truth(self.g, &node) {
                        self.set(&node, "tooltip", arg(1))?
                    }
                }
                Ok(Atom::Undefined)
            }
            15 => {
                let units = self.string_units(&arg(1), "text.includes")?;
                let link=String::from_utf16(&units).map_err(|_|invalid("Class lone-surrogate URL decoding/sanitization requires UTF16 URL capability"))?;
                let decoded = decode_link(&link)?;
                let url = decoded.trim_matches(crate::compat::source::space);
                let value = if url.is_empty() {
                    Atom::Undefined
                } else {
                    self.text(&crate::compat::source::url::sanitize_url(url))?
                };
                for id in self.split(&arg(0), 44, "ids.split")? {
                    let map = self.read(r, "classes")?;
                    let id = self.lookup_id(&id)?;
                    let node = self.map_get(&map, &id, false, "this.classes.get")?;
                    if truth(self.g, &node) {
                        self.set(&node, "link", value.clone())?;
                        let target = if matches!(arg(2), Atom::String(_)) {
                            arg(2)
                        } else {
                            self.text("_blank")?
                        };
                        self.set(&node, "linkTarget", target)?
                    }
                }
                let clickable = self.text("clickable")?;
                self.helper(r, "setCssClass", 12, vec![arg(0), clickable])?;
                Ok(Atom::Undefined)
            }
            16 => {
                for id in self.split(&arg(0), 44, "ids.split")? {
                    let map = self.read(r, "classes")?;
                    let id = self.split_name(&id)?.0;
                    let node = self.map_get(&map, &id, false, "this.classes.get")?;
                    if truth(self.g, &node) {
                        self.set(&node, "haveCallback", Atom::Bool(true))?
                    }
                }
                let clickable = self.text("clickable")?;
                self.helper(r, "setCssClass", 12, vec![arg(0), clickable])?;
                Ok(Atom::Undefined)
            }
            17 => {
                self.set(r, "direction", arg(0))?;
                Ok(Atom::Undefined)
            }
            18 | 19 => {
                let units = self.string_units(&arg(0), "text.replace")?;
                let result = if c.method.0 == 18 {
                    let from = units.iter().position(|c| !space(*c)).unwrap_or(units.len());
                    units[from..].to_vec()
                } else {
                    let mut out = vec![];
                    let mut n = 0;
                    while n < units.len() {
                        out.push(units[n]);
                        if units[n] == 10 && units.get(n + 1).is_some_and(|c| space(*c)) {
                            n += 1;
                            while n < units.len() && space(units[n]) {
                                n += 1
                            }
                            continue;
                        }
                        n += 1;
                    }
                    out
                };
                let value = self.intern(JsString(result))?;
                self.set(
                    r,
                    if c.method.0 == 18 {
                        "accTitle"
                    } else {
                        "accDescr"
                    },
                    value,
                )?;
                Ok(Atom::Undefined)
            }
            20 => self.namespace(r, arg(0), arg(1)),
            21 => {
                let stack = self.read(r, "namespaceStack")?;
                let id = self.native(&stack, "pop", "this.namespaceStack.pop", 1)?;
                let Node::Array { length, .. } = self.g.nodes[id as usize] else {
                    unreachable!()
                };
                if length > 0 {
                    self.delete(id, &(length - 1).to_string())?;
                    self.length(id, length - 1)?
                } else {
                    self.length(id, 0)?
                }
                Ok(Atom::Undefined)
            }
            22 => {
                let map = self.read(r, "namespaces")?;
                let namespace = self.map_get(&map, &arg(0), false, "this.namespaces.get")?;
                if matches!(namespace, Atom::Undefined) {
                    return Ok(Atom::Undefined);
                }
                let mut classes = self.iter(&arg(1))?;
                while let Some(name) = self.next(&mut classes)? {
                    let id = self.split_name(&name)?.0;
                    let map = self.read(r, "classes")?;
                    let node = self.map_get(&map, &id, false, "this.classes.get")?;
                    if matches!(node, Atom::Undefined) {
                        continue;
                    }
                    self.set(&node, "parent", arg(0))?;
                    let classes = self.read(&namespace, "classes")?;
                    self.map_set(&classes, id, node, "namespace.classes.set")?;
                }
                let mut notes = self.iter(&arg(2))?;
                while let Some(name) = self.next(&mut notes)? {
                    let map = self.read(r, "notes")?;
                    let note = self.map_get(&map, &name, false, "this.notes.get")?;
                    if matches!(note, Atom::Undefined) {
                        continue;
                    }
                    self.set(&note, "parent", arg(0))?;
                    let notes = self.read(&namespace, "notes")?;
                    self.map_set(&notes, name, note, "namespace.notes.set")?;
                }
                Ok(Atom::Undefined)
            }
            23 => {
                let map = self.read(r, "classes")?;
                let node = self.map_get(&map, &arg(0), false, "this.classes.get")?;
                if !truth(self.g, &arg(1)) || !truth(self.g, &node) {
                    return Ok(Atom::Undefined);
                }
                let mut styles = self.iter(&arg(1))?;
                while let Some(style) = self.next(&mut styles)? {
                    for part in self.split(&style, 44, "s.split")? {
                        let list = self.read(&node, "styles")?;
                        self.push(&list, part, "node.styles.push")?;
                    }
                }
                Ok(Atom::Undefined)
            }
            24 => self.parse_method(r, arg(0)),
            25 => self.parse_attribute(r, arg(0)),
            26 => self.display(r),
            27 => {
                let classifier = self.read(r, "classifier")?;
                self.text(if matches!(classifier,Atom::String(id)if self.g.strings[id as usize].is("*")){"font-style:italic;"}else if matches!(classifier,Atom::String(id)if self.g.strings[id as usize].is("$")){"text-decoration:underline;"}else{""})
            }
            _ => Err(invalid("Class native method is unavailable").into()),
        }
    }
}

// The shared scalar decoder reports its native transport Limit for the one
// String.fromCodePoint fractional case. At a real builder invocation retain
// the actual JavaScript RangeError class/message before any model mutation.
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

impl ActiveModel for Context<'_> {
    fn begin_sync(&mut self) -> Result<(), PublicError> {
        if self.sync.is_some() {
            return Ok(());
        }
        self.sync = Some(SyncJournal {
            strings: self.g.strings.len(),
            nodes: self.g.nodes.len(),
            dirty: BTreeSet::new(),
        });
        Ok(())
    }
    fn apply_delta(&mut self, delta: ModelDelta) -> Result<(), PublicError> {
        let c = &mut *self;
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
        let c = &mut *self;
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
        self.g.root.clone()
    }
    fn validate(&self, value: &Atom) -> Result<(), PublicError> {
        self.g.validate_atom(value)?;
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
        self.tick(key.len())?;
        self.read(receiver, key)
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
            if let Node::Array { length, .. } = self.g.nodes[*id as usize] {
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
        self.tick(key.len())?;
        self.set(receiver, key, value)
    }
    fn intern(&mut self, value: JsString) -> Result<Atom, PublicError> {
        self.tick(value.0.len())?;
        self.intern(value)
    }
    fn string(&mut self, value: &Atom) -> Result<JsString, PublicError> {
        self.validate(value)?;
        self.tick(1)?;
        match value {
            Atom::String(id) => {
                self.tick(self.g.strings[*id as usize].0.len())?;
                Ok(self.g.strings[*id as usize].clone())
            }
            _ => Err(type_error("Active model value is not a string")),
        }
    }
    fn object(&mut self) -> Result<Atom, PublicError> {
        self.node(Node::Object { properties: vec![] })
    }
    fn array(&mut self) -> Result<Atom, PublicError> {
        self.array(vec![])
    }
    fn date(&mut self, milliseconds: f64) -> Result<Atom, PublicError> {
        let milliseconds = if milliseconds.is_finite() && milliseconds.abs() <= 8.64e15 {
            milliseconds.trunc() + 0.0
        } else {
            f64::NAN
        };
        self.node(Node::Date {
            milliseconds,
            properties: vec![],
        })
    }
    fn date_millis(&mut self, value: &Atom) -> Result<f64, PublicError> {
        self.validate(value)?;
        self.tick(1)?;
        match value {
            Atom::Ref(id) => match self.g.nodes[*id as usize] {
                Node::Date { milliseconds, .. } => Ok(milliseconds),
                _ => Err(type_error("Active model value is not a Date")),
            },
            _ => Err(type_error("Active model value is not a Date")),
        }
    }
    fn push(&mut self, array: &Atom, value: Atom) -> Result<(), PublicError> {
        self.validate(array)?;
        self.validate(&value)?;
        self.push(array, value, "array.push")
    }
    fn invoke_builtin(
        &mut self,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        self.tick(call.arguments.len())?;
        self.validate(&call.receiver)?;
        for value in &call.arguments {
            self.validate(value)?;
        }
        self.invoke_live(call, host)
    }
}
impl Context<'_> {
    fn invoke_live(
        &mut self,
        call: MethodCall,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        if self.depth >= 64 {
            return Err(limit().into());
        }
        self.depth += 1;
        let result = if call.method.0 == 26 {
            (|| {
                self.tick(1)?;
                let display = self.display_text(&call.receiver)?;
                let css = self.call_member(&call.receiver, "parseClassifier", 27, host)?;
                self.record(vec![("displayText", display), ("cssStyle", css)])
            })()
        } else {
            self.invoke(call)
        };
        self.depth -= 1;
        result
    }
    fn call_member(
        &mut self,
        receiver: &Atom,
        name: &str,
        method: u32,
        host: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        self.tick(1)?;
        if self.depth >= 64 {
            return Err(limit().into());
        }
        let value = self.read(receiver, name)?;
        let native = matches!(receiver, Atom::Ref(id) if matches!(self.g.nodes[*id as usize], Node::Instance { prototype: MEMBER, .. }) && own(self.g,*id,name).is_none());
        match value {
            Atom::HostReference(reference) => {
                let callable = host.host_function(reference)?;
                self.depth += 1;
                let result = callable.call(self, receiver.clone(), &[], host);
                self.depth -= 1;
                if let Ok(value) = &result {
                    self.validate(value)?;
                }
                result
            }
            Atom::BuiltinFunction(id) => self.invoke_live(
                MethodCall {
                    method: MethodId(id),
                    receiver: receiver.clone(),
                    arguments: vec![],
                },
                host,
            ),
            _ if native => self.invoke_live(
                MethodCall {
                    method: MethodId(method),
                    receiver: receiver.clone(),
                    arguments: vec![],
                },
                host,
            ),
            _ => Err(type_error(if method == 26 {
                "member.getDisplayDetails is not a function"
            } else {
                "this.parseClassifier is not a function"
            })),
        }
    }
}

// Only public rows with replaced methods use the live adapter. The immutable
// typed model and ordinary parsed public models keep their existing fast path.
fn needs_live(graph: &ModelGraph) -> bool {
    graph.nodes.iter().any(|node| {
        node.properties().iter().any(|property| {
            let key = &graph.strings[property.key as usize];
            (key.is("getDisplayDetails") && !matches!(property.value, Atom::BuiltinFunction(26)))
                || (key.is("parseClassifier")
                    && !matches!(property.value, Atom::BuiltinFunction(27)))
        })
    })
}
struct Live<'a> {
    context: Context<'a>,
    classes: Vec<Atom>,
    error: Option<PublicError>,
}
impl Live<'_> {
    fn result<T>(&mut self, value: Result<T, PublicError>) -> Result<T, PeleError> {
        value.map_err(|error| {
            let legacy = match &error {
                PublicError::Pele(error) => error.clone(),
                _ => invalid("Class callback failed"),
            };
            self.error = Some(error);
            legacy
        })
    }
    fn class_value(&self, index: usize) -> Result<Atom, PublicError> {
        self.classes
            .get(index)
            .cloned()
            .ok_or_else(|| invalid("Class identity is unavailable").into())
    }
}
impl super::render::LiveMembers for Live<'_> {
    fn begin_draw(&mut self) -> Result<ClassModel, PeleError> {
        let model = imported_mode(self.context.g, true)?;
        let mut import = Import {
            g: self.context.g,
            bytes: 0,
            work: 0,
        };
        self.classes = import
            .map(required(import.g, &import.g.root, "classes")?)?
            .into_iter()
            .map(|(_, value)| value)
            .collect();
        Ok(model)
    }
    fn class(&mut self, index: usize) -> Result<ClassNode, PeleError> {
        let value = self.class_value(index);
        let a = self.result(value)?;
        let g = &*self.context.g;
        let mut i = Import {
            g,
            bytes: 0,
            work: 0,
        };
        Ok(ClassNode {
            id: i.field(&a, "id")?,
            generic: i.field(&a, "type")?,
            label: i.field(&a, "label")?,
            css_classes: i.field(&a, "cssClasses")?,
            methods: vec![],
            members: vec![],
            annotations: i.texts(required(g, &a, "annotations")?)?,
            styles: i.texts(required(g, &a, "styles")?)?,
            parent: i.optional(&a, "parent")?,
            link: i.optional(&a, "link")?,
            link_target: i.optional(&a, "linkTarget")?,
            have_callback: match a {
                Atom::Ref(id) => own(g, id, "haveCallback").map(|v| truth(g, v)),
                _ => None,
            },
            tooltip: i.optional(&a, "tooltip")?,
        })
    }
    fn annotations(&mut self, index: usize) -> Result<Vec<String>, PeleError> {
        let value = self.class_value(index);
        let class = self.result(value)?;
        let graph = &*self.context.g;
        let mut import = Import {
            g: graph,
            bytes: 0,
            work: 0,
        };
        import.texts(required(graph, &class, "annotations")?)
    }
    fn paint(&mut self, index: usize) -> Result<ClassNode, PeleError> {
        self.class(index)
    }
    fn metadata(&mut self, key: &str) -> Result<Option<String>, PeleError> {
        let graph = &*self.context.g;
        let mut import = Import {
            g: graph,
            bytes: 0,
            work: 0,
        };
        import.optional(&graph.root, key)
    }
    fn definition(&mut self, name: &str) -> Result<Vec<String>, PeleError> {
        let result = (|| {
            let root = self.context.g.root.clone();
            let map = self.context.read(&root, "styleClasses")?;
            let key = self.context.text(name)?;
            let value = self
                .context
                .map_get(&map, &key, false, "db.styleClasses.get")?;
            if !truth(self.context.g, &value) {
                return Ok(vec![]);
            }
            let styles = self.context.read(&value, "styles")?;
            let mut import = Import {
                g: self.context.g,
                bytes: 0,
                work: 0,
            };
            Ok(import.texts(&styles)?)
        })();
        self.result(result)
    }
    fn lines(
        &mut self,
        index: usize,
        field: &str,
        size: f64,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Vec<super::render::Line>, PeleError> {
        let result = (|| {
            let class = self.class_value(index)?;
            let array = self.context.read(&class, field)?;
            let id = self
                .context
                .native(&array, "map", &format!("cls.{field}.map"), 1)?;
            let Node::Array { length, .. } = self.context.g.nodes[id as usize] else {
                unreachable!()
            };
            if length > 1_000_000 {
                return Err(limit().into());
            }
            let mut lines = Vec::new();
            for index in 0..length {
                self.context.tick(1)?;
                let Some(member) = self.context.own(id, &index.to_string())? else {
                    continue;
                };
                let display = self
                    .context
                    .call_member(&member, "getDisplayDetails", 26, host)?;
                let text = self.context.read(&display, "displayText")?;
                let text = match text {
                    Atom::String(id) => self.context.g.strings[id as usize]
                        .well_formed()
                        .ok_or_else(|| invalid("Class display text must be valid UTF-16"))?,
                    Atom::Undefined => {
                        return Err(type_error(
                            "Cannot read properties of undefined (reading 'includes')",
                        ))
                    }
                    Atom::Null => {
                        return Err(type_error(
                            "Cannot read properties of null (reading 'includes')",
                        ))
                    }
                    _ => return Err(type_error("text.includes is not a function")),
                };
                let text = crate::compat::source::decode_entities(&text)?
                    .trim_matches(crate::compat::source::space)
                    .to_owned();
                let classifier = self.context.read(&member, "classifier")?;
                let italic = matches!(classifier, Atom::String(id) if self.context.g.strings[id as usize].is("*"));
                let width = crate::compat::text::measure(
                    host,
                    options,
                    &text,
                    size,
                    if italic { 2 } else { 0 },
                )?;
                let classifier = self.context.read(&member, "classifier")?;
                let classifier = match classifier {
                    Atom::String(id) if self.context.g.strings[id as usize].is("*") => "*",
                    Atom::String(id) if self.context.g.strings[id as usize].is("$") => "$",
                    _ => "",
                }
                .to_owned();
                lines.push(super::render::Line {
                    text,
                    width,
                    classifier,
                });
            }
            Ok(lines)
        })();
        self.result(result)
    }
}
fn render_live_state(
    state: &mut ClassPublicState,
    config: &Object,
    options: &RenderSettings,
    host: &mut dyn RenderResources,
    exact: bool,
) -> Result<Rendered, PublicError> {
    let model = imported_mode(&state.graph, true)?;
    let mut context = Context::new(&mut state.graph);
    context.begin_sync()?;
    let mut live = Live {
        context,
        classes: vec![],
        error: None,
    };
    let result = super::render::render_live(&model, config, options, host, exact, &mut live);
    let error = live.error.take();
    drop(live);
    state.model = imported(&state.graph);
    match error {
        Some(error) => Err(error),
        None => result.map_err(Into::into),
    }
}
fn legacy_error(error: PublicError) -> PeleError {
    match error {
        PublicError::Pele(error) => error,
        other => invalid(&format!(
            "Class public operation requires structured exception transport: {other:?}"
        )),
    }
}
