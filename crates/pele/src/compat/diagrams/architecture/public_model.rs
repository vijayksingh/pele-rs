//! ArchitectureDb native public model, Pele 458322a5 (MIT kepano).
//! Constructors, exotic coercion/descriptors and active lone UTF-16 rendering
//! remain explicit gaps. Retained graph values are never normalized or reparsed.
use super::{ArchEdge, ArchGroup, ArchLayoutHint, ArchNode, Architecture, ArchitectureModel};
use crate::compat::{
    public::*, DiagramModule, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::{HashMap, HashSet};
const PROTO: PrototypeId = PrototypeId(1);
const REGISTER: MethodId = MethodId(0);
const SERVICE: MethodId = MethodId(1);
const JUNCTION: MethodId = MethodId(2);
const GROUP: MethodId = MethodId(3);
const EDGE: MethodId = MethodId(4);
const HINT: MethodId = MethodId(5);
const fn field_schema(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        optional: false,
        undefined,
        nullable: false,
        flags: 7,
    }
}
const fn member(name: &'static str, method: MethodId) -> MemberSchema {
    MemberSchema {
        name,
        flags: 6,
        member: MemberKind::Method { method },
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/architecture@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "Architecture",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false),
                field_schema("title", FieldKind::String, true),
                field_schema("accTitle", FieldKind::String, true),
                field_schema("accDescr", FieldKind::String, true),
                field_schema("nodes", FieldKind::Map, false),
                field_schema("groups", FieldKind::Map, false),
                field_schema("edges", FieldKind::Array, false),
                field_schema("layoutHints", FieldKind::Array, false),
                field_schema("registered", FieldKind::Map, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "ArchitectureService",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false),
                field_schema("type", FieldKind::String, false),
                field_schema("icon", FieldKind::String, true),
                field_schema("iconText", FieldKind::String, true),
                field_schema("title", FieldKind::String, true),
                field_schema("in", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "ArchitectureJunction",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false),
                field_schema("type", FieldKind::String, false),
                field_schema("in", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "ArchitectureGroup",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false),
                field_schema("icon", FieldKind::String, true),
                field_schema("title", FieldKind::String, true),
                field_schema("in", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "ArchitectureEdge",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("lhsId", FieldKind::String, false),
                field_schema("lhsDir", FieldKind::String, false),
                field_schema("lhsInto", FieldKind::Bool, true),
                field_schema("lhsGroup", FieldKind::Bool, true),
                field_schema("rhsId", FieldKind::String, false),
                field_schema("rhsDir", FieldKind::String, false),
                field_schema("rhsInto", FieldKind::Bool, true),
                field_schema("rhsGroup", FieldKind::Bool, true),
                field_schema("title", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(5),
            name: "ArchitectureLayoutHint",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("direction", FieldKind::String, false),
                field_schema("members", FieldKind::Array, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "ArchitectureDb",
        parent: PrototypeParent::Object,
        members: &[
            member("register", REGISTER),
            member("addService", SERVICE),
            member("addJunction", JUNCTION),
            member("addGroup", GROUP),
            member("addEdge", EDGE),
            member("addLayoutHint", HINT),
        ],
    }],
    methods: &[
        MethodSchema {
            id: REGISTER,
            name: "register",
            arity: 3,
        },
        MethodSchema {
            id: SERVICE,
            name: "addService",
            arity: 1,
        },
        MethodSchema {
            id: JUNCTION,
            name: "addJunction",
            arity: 1,
        },
        MethodSchema {
            id: GROUP,
            name: "addGroup",
            arity: 1,
        },
        MethodSchema {
            id: EDGE,
            name: "addEdge",
            arity: 1,
        },
        MethodSchema {
            id: HINT,
            name: "addLayoutHint",
            arity: 1,
        },
    ],
};
pub struct ArchitecturePublicState {
    graph: ModelGraph,
    model: Result<ArchitectureModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("architecture".into());
    e
}
fn limit() -> PeleError {
    let mut e = PeleError::new(
        "Architecture public operation exceeds native work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("architecture".into());
    e
}
fn js(kind: JavaScriptErrorKind, message: impl Into<String>) -> PublicError {
    PublicError::JavaScript {
        name: kind,
        message: JsString::from(message.into().as_str()),
    }
}
fn type_error(message: impl Into<String>) -> PublicError {
    js(JavaScriptErrorKind::TypeError, message)
}
fn property<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn number(n: f64) -> String {
    if n.is_nan() {
        "NaN".into()
    } else if n == 0. {
        "0".into()
    } else if n == f64::INFINITY {
        "Infinity".into()
    } else if n == f64::NEG_INFINITY {
        "-Infinity".into()
    } else {
        ryu_js::Buffer::new().format(n).into()
    }
}
fn primitive(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    match a {
        Atom::Undefined => Ok("undefined".into()),
        Atom::Null => Ok("null".into()),
        Atom::Bool(v) => Ok(v.to_string()),
        Atom::Number(n) => Ok(number(*n)),
        Atom::String(id) => g.strings[*id as usize].well_formed().ok_or_else(|| {
            invalid("Architecture runtime error text containing lone UTF-16 is not implemented")
                .into()
        }),
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Object { .. } | Node::Instance { .. } => Ok("[object Object]".into()),
            Node::Map { .. } => Ok("[object Map]".into()),
            Node::Set { .. } => Ok("[object Set]".into()),
            _ => Err(invalid("Architecture arbitrary object coercion is not implemented").into()),
        },
        _ => Err(invalid("Architecture arbitrary object coercion is not implemented").into()),
    }
}
fn truthy(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(b) => *b,
        Atom::Number(n) => *n != 0. && !n.is_nan(),
        Atom::String(id) => !g.strings[*id as usize].0.is_empty(),
        _ => true,
    }
}
fn read(g: &ModelGraph, a: &Atom, key: &str) -> Result<Atom, PublicError> {
    match a {
        Atom::Null | Atom::Undefined => Err(type_error(format!(
            "Cannot read properties of {} (reading '{key}')",
            if matches!(a, Atom::Null) {
                "null"
            } else {
                "undefined"
            }
        ))),
        Atom::Ref(id) => {
            if key == "length" {
                if let Node::Array { length, .. } = &g.nodes[*id as usize] {
                    return Ok(Atom::Number(*length as f64));
                }
            }
            Ok(property(g, *id, key).cloned().unwrap_or(Atom::Undefined))
        }
        Atom::String(id) if key == "length" => {
            Ok(Atom::Number(g.strings[*id as usize].0.len() as f64))
        }
        Atom::String(id)
            if key
                .parse::<usize>()
                .ok()
                .is_some_and(|i| i.to_string() == key && i < g.strings[*id as usize].0.len()) =>
        {
            Err(invalid("Architecture String collection indexing is not implemented").into())
        }
        _ => Ok(Atom::Undefined),
    }
}
fn intern(g: &mut ModelGraph, s: JsString) -> Result<u32, PublicError> {
    if let Some(i) = g.strings.iter().position(|v| *v == s) {
        return Ok(i as u32);
    }
    if g.strings.len() >= 1_000_000 || s.0.len() > 16_000_000 {
        return Err(limit().into());
    }
    let id = g.strings.len() as u32;
    g.strings.push(s);
    Ok(id)
}
fn node(g: &mut ModelGraph, n: Node) -> Result<Atom, PublicError> {
    if g.nodes.len() >= 1_000_000 {
        return Err(limit().into());
    }
    let id = g.nodes.len() as u32;
    g.nodes.push(n);
    Ok(Atom::Ref(id))
}
fn set(g: &mut ModelGraph, a: &Atom, key: &str, value: Atom) -> Result<(), PublicError> {
    let id = match a {
        Atom::Ref(id) => *id,
        Atom::Null | Atom::Undefined => {
            return Err(type_error(format!(
                "Cannot set properties of {} (setting '{key}')",
                if matches!(a, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            )))
        }
        Atom::Bool(_) | Atom::Number(_) | Atom::String(_) => {
            return Err(type_error(format!(
                "Cannot create property '{key}' on {} '{}'",
                match a {
                    Atom::Bool(_) => "boolean",
                    Atom::Number(_) => "number",
                    _ => "string",
                },
                primitive(g, a)?
            )))
        }
        _ => return Err(invalid("Architecture borrowed host receiver is not implemented").into()),
    };
    let i = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key));
    if let Some(i) = i {
        let p = &mut g.nodes[id as usize].properties_mut()[i];
        if p.flags & 2 == 0 {
            return Err(type_error(format!(
                "Cannot assign to read only property '{key}'"
            )));
        }
        p.value = value;
    } else {
        let key = intern(g, key.into())?;
        g.nodes[id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        })
    }
    Ok(())
}
fn push(g: &mut ModelGraph, a: Atom, value: Atom, message: &str) -> Result<(), PublicError> {
    let id = match a {
        Atom::Ref(id) => id,
        Atom::Null | Atom::Undefined => {
            read(g, &a, "push")?;
            unreachable!()
        }
        _ => return Err(type_error(message)),
    };
    let Node::Array { length, .. } = g.nodes[id as usize] else {
        return Err(type_error(message));
    };
    if let Some(value) = property(g, id, "push") {
        return Err(match value {
            Atom::BuiltinFunction(_) | Atom::HostReference(_) => {
                invalid("Architecture overridden Array callbacks are not implemented").into()
            }
            _ => type_error(message),
        });
    }

    set(g, &a, &length.to_string(), value)?;
    let Some(next) = length.checked_add(1) else {
        return Err(js(JavaScriptErrorKind::RangeError, "Invalid array length"));
    };
    if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
        *length = next;
    }
    Ok(())
}
fn array(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PeleError> {
    let Atom::Ref(id) = a else {
        return Err(invalid("Architecture active collections must be Arrays"));
    };
    let Node::Array { length, properties } = &g.nodes[*id as usize] else {
        return Err(invalid("Architecture active collections must be Arrays"));
    };
    if *length > 1_000_000 {
        return Err(limit());
    }
    let mut values = HashMap::new();
    for p in properties {
        if let Some(key) = g.strings[p.key as usize].well_formed() {
            if let Ok(i) = key.parse::<u32>() {
                if i < *length && i.to_string() == key {
                    values.insert(i, p.value.clone());
                }
            }
        }
    }
    if values.len() != *length as usize {
        return Err(invalid(
            "Architecture sparse active arrays are not implemented",
        ));
    }
    Ok((0..*length).map(|i| values.remove(&i).unwrap()).collect())
}
fn equal(g: &ModelGraph, a: &Atom, b: &Atom, nan: bool) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Number(a), Atom::Number(b)) => a == b || (nan && a.is_nan() && b.is_nan()),
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize] == g.strings[*b as usize],
        (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
fn map_id(g: &ModelGraph, a: &Atom, op: &str, origin: &str) -> Result<u32, PublicError> {
    match a {
        Atom::Ref(id) if matches!(g.nodes[*id as usize], Node::Map { .. }) => {
            if let Some(value) = property(g, *id, op) {
                return Err(match value {
                    Atom::BuiltinFunction(_) | Atom::HostReference(_) => {
                        invalid("Architecture overridden Map callbacks are not implemented").into()
                    }
                    _ => type_error(format!("{origin}.{op} is not a function")),
                });
            }
            Ok(*id)
        }
        Atom::Null | Atom::Undefined => {
            read(g, a, op)?;
            unreachable!()
        }
        _ => Err(type_error(format!("{origin}.{op} is not a function"))),
    }
}
fn map_has(g: &ModelGraph, a: &Atom, key: &Atom, origin: &str) -> Result<bool, PublicError> {
    let id = map_id(g, a, "has", origin)?;
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    Ok(entries.iter().any(|(k, _)| equal(g, k, key, true)))
}
fn map_get(g: &ModelGraph, a: &Atom, key: &Atom, origin: &str) -> Result<Atom, PublicError> {
    let id = map_id(g, a, "get", origin)?;
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    Ok(entries
        .iter()
        .find(|(k, _)| equal(g, k, key, true))
        .map(|(_, v)| v.clone())
        .unwrap_or(Atom::Undefined))
}
fn map_set(
    g: &mut ModelGraph,
    a: &Atom,
    mut key: Atom,
    value: Atom,
    origin: &str,
) -> Result<(), PublicError> {
    let id = map_id(g, a, "set", origin)?;
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    let found = entries.iter().position(|(k, _)| equal(g, k, &key, true));
    if let Atom::Number(n) = key {
        if n == 0. {
            key = Atom::Number(0.)
        }
    }
    let Node::Map { entries, .. } = &mut g.nodes[id as usize] else {
        unreachable!()
    };
    if let Some(i) = found {
        entries[i] = (key, value)
    } else {
        if entries.len() >= 1_000_000 {
            return Err(limit().into());
        }
        entries.push((key, value))
    }
    Ok(())
}
fn field(g: &ModelGraph, r: &Atom, key: &str) -> Result<Atom, PublicError> {
    read(g, r, key)
}
fn is(g: &ModelGraph, a: &Atom, s: &str) -> bool {
    matches!(a,Atom::String(id)if g.strings[*id as usize].is(s))
}
fn register(
    g: &mut ModelGraph,
    r: &Atom,
    kind: Atom,
    id: Atom,
    parent: Atom,
) -> Result<(), PublicError> {
    let reg = field(g, r, "registered")?;
    if map_has(g, &reg, &id, "this.registered")? {
        let prior = map_get(g, &reg, &id, "this.registered")?;
        return Err(invalid(&format!(
            "The {} id [{}] is already in use by another {}",
            primitive(g, &kind)?,
            primitive(g, &id)?,
            primitive(g, &prior)?
        ))
        .into());
    }
    if !matches!(parent, Atom::Undefined) {
        if equal(g, &id, &parent, false) {
            return Err(invalid(&format!(
                "The {} [{}] cannot be placed within itself",
                primitive(g, &kind)?,
                primitive(g, &id)?
            ))
            .into());
        }
        if !map_has(g, &reg, &parent, "this.registered")? {
            let kind = primitive(g, &kind)?;
            return Err(invalid(&format!("The {kind} [{}]'s parent does not exist. Please make sure the parent is created before this {kind}",primitive(g,&id)?)).into());
        }
        if is(g, &map_get(g, &reg, &parent, "this.registered")?, "node") {
            return Err(invalid(&format!(
                "The {} [{}]'s parent is not a group",
                primitive(g, &kind)?,
                primitive(g, &id)?
            ))
            .into());
        }
    }
    let marker = intern(
        g,
        if is(g, &kind, "group") {
            "group".into()
        } else {
            "node".into()
        },
    )?;
    map_set(g, &reg, id, Atom::String(marker), "this.registered")
}
fn register_dispatch(g: &ModelGraph, receiver: &Atom) -> Result<(), PublicError> {
    let own = match receiver {
        Atom::Ref(id) => g.nodes[*id as usize]
            .properties()
            .iter()
            .find(|p| g.strings[p.key as usize].is("register"))
            .map(|p| p.value.clone()),
        _ => {
            read(g, receiver, "register")?;
            None
        }
    };
    match own {
        Some(Atom::BuiltinFunction(id)) if id == REGISTER.0 => Ok(()),
        Some(Atom::BuiltinFunction(_) | Atom::HostReference(_)) => {
            Err(invalid("Architecture overridden register callbacks are not implemented").into())
        }
        Some(_) => Err(type_error("this.register is not a function")),
        None => {
            if matches!(receiver,Atom::Ref(id)if matches!(g.nodes[*id as usize],Node::Instance{prototype:PROTO,..}))
            {
                Ok(())
            } else {
                Err(type_error("this.register is not a function"))
            }
        }
    }
}
fn destructure(g: &ModelGraph, a: &Atom, first: &str) -> Result<(), PublicError> {
    if matches!(a, Atom::Null | Atom::Undefined) {
        return Err(type_error(format!(
            "Cannot destructure property '{first}' of '{}' as it is {}.",
            if matches!(a, Atom::Null) {
                "object null"
            } else {
                "undefined"
            },
            if matches!(a, Atom::Null) {
                "null"
            } else {
                "undefined"
            }
        )));
    }
    g.validate_atom(a)?;
    Ok(())
}
fn record(
    g: &mut ModelGraph,
    fields: impl IntoIterator<Item = (&'static str, Atom)>,
) -> Result<Atom, PublicError> {
    let a = node(g, Node::Object { properties: vec![] })?;
    for (k, v) in fields {
        set(g, &a, k, v)?
    }
    Ok(a)
}
fn members(g: &mut ModelGraph, a: Atom) -> Result<Vec<Atom>, PublicError> {
    match a {
        Atom::Ref(id) => match &g.nodes[id as usize] {
            Node::Array { .. } => array(g, &a).map_err(Into::into),
            Node::Set { values, .. } => Ok(values.clone()),
            _ => Err(type_error("t.members is not iterable")),
        },
        Atom::String(id) => {
            let raw = g.strings[id as usize].0.clone();
            if raw.len() > 1_000_000 {
                return Err(limit().into());
            }
            let mut values = vec![];
            let mut i = 0;
            while i < raw.len() {
                let paired = (0xd800..=0xdbff).contains(&raw[i])
                    && raw
                        .get(i + 1)
                        .is_some_and(|u| (0xdc00..=0xdfff).contains(u));
                let next = i + if paired { 2 } else { 1 };
                values.push(Atom::String(intern(g, JsString(raw[i..next].to_vec()))?));
                i = next;
            }
            Ok(values)
        }
        _ => Err(type_error("t.members is not iterable")),
    }
}
fn invoke(g: &mut ModelGraph, c: MethodCall) -> Result<Atom, PublicError> {
    let arg = |i| c.arguments.get(i).cloned().unwrap_or(Atom::Undefined);
    match c.method {
        REGISTER => register(g, &c.receiver, arg(0), arg(1), arg(2))?,
        SERVICE | JUNCTION | GROUP => {
            let a = arg(0);
            destructure(g, &a, "id")?;
            let id = field(g, &a, "id")?;
            let parent = field(g, &a, "in")?;
            let kind = match c.method {
                SERVICE => "service",
                JUNCTION => "junction",
                _ => "group",
            };
            let kind_atom = Atom::String(intern(g, kind.into())?);
            let icon = if c.method != JUNCTION {
                field(g, &a, "icon")?
            } else {
                Atom::Undefined
            };
            let icon_text = if c.method == SERVICE {
                field(g, &a, "iconText")?
            } else {
                Atom::Undefined
            };
            let title = if c.method != JUNCTION {
                field(g, &a, "title")?
            } else {
                Atom::Undefined
            };
            register_dispatch(g, &c.receiver)?;
            register(
                g,
                &c.receiver,
                kind_atom.clone(),
                id.clone(),
                parent.clone(),
            )?;
            let target = field(
                g,
                &c.receiver,
                if c.method == GROUP { "groups" } else { "nodes" },
            )?;
            map_id(
                g,
                &target,
                "set",
                if c.method == GROUP {
                    "this.groups"
                } else {
                    "this.nodes"
                },
            )?;
            let values = if c.method == GROUP {
                vec![
                    ("id", id.clone()),
                    ("icon", icon),
                    ("title", title),
                    ("in", parent),
                ]
            } else if c.method == JUNCTION {
                vec![("id", id.clone()), ("type", kind_atom), ("in", parent)]
            } else {
                vec![
                    ("id", id.clone()),
                    ("type", kind_atom),
                    ("icon", icon),
                    ("iconText", icon_text),
                    ("title", title),
                    ("in", parent),
                ]
            };
            let value = record(g, values)?;
            map_set(
                g,
                &target,
                id,
                value,
                if c.method == GROUP {
                    "this.groups"
                } else {
                    "this.nodes"
                },
            )?;
        }
        EDGE => {
            let a = arg(0);
            destructure(g, &a, "lhsId")?;
            let lhs = field(g, &a, "lhsId")?;
            let rhs = field(g, &a, "rhsId")?;
            let ld = field(g, &a, "lhsDir")?;
            let rd = field(g, &a, "rhsDir")?;
            let li = field(g, &a, "lhsInto")?;
            let ri = field(g, &a, "rhsInto")?;
            let lg = field(g, &a, "lhsGroup")?;
            let rg = field(g, &a, "rhsGroup")?;
            let title = field(g, &a, "title")?;
            for (side, id) in [("left", &lhs), ("right", &rhs)] {
                let reg = field(g, &c.receiver, "registered")?;
                if !map_has(g, &reg, id, "this.registered")? {
                    return Err(invalid(&format!("The {side}-hand id [{}] does not yet exist. Please create the service/group before declaring an edge to it.",primitive(g,id)?)).into());
                }
            }
            for (side, id) in [("left", &lhs), ("right", &rhs)] {
                let nodes = field(g, &c.receiver, "nodes")?;
                if !map_has(g, &nodes, id, "this.nodes")? {
                    return Err(invalid(&format!("The {side}-hand id [{}] is a group. Add {{group}} to a service inside it instead.",primitive(g,id)?)).into());
                }
            }
            let nodes = field(g, &c.receiver, "nodes")?;
            let parent = read(g, &map_get(g, &nodes, &lhs, "this.nodes")?, "in")?;
            if !matches!(parent, Atom::Undefined) {
                let other = read(g, &map_get(g, &nodes, &rhs, "this.nodes")?, "in")?;
                if equal(g, &parent, &other, false) {
                    for (side, id, crosses) in [("left", &lhs, &lg), ("right", &rhs, &rg)] {
                        if truthy(g, crosses) {
                            return Err(invalid(&format!("The {side}-hand id [{}] is modified to traverse the group boundary, but the edge does not pass through two groups.",primitive(g,id)?)).into());
                        }
                    }
                }
            }
            let edges = field(g, &c.receiver, "edges")?;
            let edge = record(
                g,
                [
                    ("lhsId", lhs),
                    ("lhsDir", ld),
                    ("lhsInto", li),
                    ("lhsGroup", lg),
                    ("rhsId", rhs),
                    ("rhsDir", rd),
                    ("rhsInto", ri),
                    ("rhsGroup", rg),
                    ("title", title),
                ],
            )?;
            push(g, edges, edge, "this.edges.push is not a function")?;
        }
        HINT => {
            let hint = arg(0);
            let collection = read(g, &hint, "members")?;
            let length = read(g, &collection, "length")?;
            if !matches!(length, Atom::Number(_) | Atom::Undefined) {
                return Err(
                    invalid("Architecture collection length coercion is not implemented").into(),
                );
            }
            if matches!(length,Atom::Number(n)if n<2.) {
                return Err(invalid(&format!(
                    "An align directive requires at least two members; got {}",
                    primitive(g, &length)?
                ))
                .into());
            }
            let values = members(g, collection)?;
            let reg = field(g, &c.receiver, "registered")?;
            let reg_id = map_id(g, &reg, "get", "this.registered")?;
            let Node::Map { entries, .. } = &g.nodes[reg_id as usize] else {
                unreachable!()
            };
            if values
                .len()
                .saturating_mul(entries.len().saturating_add(values.len()))
                > 32_000_000
            {
                return Err(limit().into());
            }
            let mut seen = vec![];
            for id in values {
                let marker = map_get(g, &reg, &id, "this.registered")?;
                if !is(g, &marker, "node") {
                    let direction = primitive(g, &read(g, &hint, "direction")?)?;
                    return Err(invalid(&format!(
                        "align {direction} references [{}], which is not a service or junction",
                        primitive(g, &id)?
                    ))
                    .into());
                }
                if seen.iter().any(|a| equal(g, a, &id, true)) {
                    let direction = primitive(g, &read(g, &hint, "direction")?)?;
                    return Err(invalid(&format!(
                        "align {direction} lists [{}] more than once",
                        primitive(g, &id)?
                    ))
                    .into());
                }
                seen.push(id)
            }
            let list = field(g, &c.receiver, "layoutHints")?;
            push(g, list, hint, "this.layoutHints.push is not a function")?;
        }
        _ => return Err(invalid("Unknown Architecture public method").into()),
    }
    Ok(Atom::Undefined)
}
struct Text(usize);
impl Text {
    fn get(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
        optional: bool,
    ) -> Result<Option<String>, PeleError> {
        if optional && matches!(a, None | Some(Atom::Undefined)) {
            return Ok(None);
        }
        let Some(Atom::String(id)) = a else {
            return Err(invalid(
                "Architecture rendering of non-string active text is not implemented",
            ));
        };
        let raw = &g.strings[*id as usize];
        self.0 = self.0.checked_add(raw.0.len()).ok_or_else(limit)?;
        if self.0 > 16_000_000 {
            return Err(limit());
        }
        raw.well_formed().map(Some).ok_or_else(|| {
            invalid("Architecture rendering of lone UTF-16 surrogates is not implemented")
        })
    }
}
fn map_entries(g: &ModelGraph, a: Option<&Atom>) -> Result<Vec<(Atom, Atom)>, PeleError> {
    let Some(Atom::Ref(id)) = a else {
        return Err(invalid("Architecture active maps must be Maps"));
    };
    let Node::Map { entries, .. } = &g.nodes[*id as usize] else {
        return Err(invalid("Architecture active maps must be Maps"));
    };
    if property(g, *id, "values").is_some() {
        return Err(invalid(
            "Architecture overridden Map value iterators are not implemented",
        ));
    }
    if entries.len() > 1_000_000 {
        return Err(limit());
    }
    Ok(entries.clone())
}
fn object(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = a {
        Ok(*id)
    } else {
        Err(invalid("Architecture record must be an object"))
    }
}
fn optional_parent(g: &ModelGraph, a: Option<&Atom>) -> Option<String> {
    if let Some(Atom::String(id)) = a {
        g.strings[*id as usize].well_formed()
    } else {
        None
    }
}
fn optional_icon(
    g: &ModelGraph,
    text: &mut Text,
    a: Option<&Atom>,
) -> Result<Option<String>, PeleError> {
    match a {
        Some(a) if !truthy(g, a) => Ok(None),
        a => text.get(g, a, true),
    }
}
fn direction(g: &ModelGraph, a: Option<&Atom>) -> Result<char, PeleError> {
    match a {
        Some(Atom::String(id)) => match g.strings[*id as usize].well_formed().as_deref() {
            Some("L") => Ok('L'),
            Some("R") => Ok('R'),
            Some("T") => Ok('T'),
            Some("B") => Ok('B'),
            _ => Err(invalid(
                "Architecture non-cardinal edge directions are not implemented",
            )),
        },
        _ => Err(invalid(
            "Architecture non-cardinal edge directions are not implemented",
        )),
    }
}
fn ancestry(groups: &[(String, ArchGroup)]) -> Result<(), PeleError> {
    let ids: HashMap<_, _> = groups
        .iter()
        .enumerate()
        .map(|(i, (_, g))| (g.id.as_str(), i))
        .collect();
    let parents = groups
        .iter()
        .enumerate()
        .map(|(i, (_, g))| {
            g.parent
                .as_deref()
                .and_then(|p| ids.get(p).copied())
                .filter(|p| *p != i)
        })
        .collect::<Vec<_>>();
    let mut states = vec![0u8; groups.len()];
    for start in 0..groups.len() {
        if states[start] != 0 {
            continue;
        }
        let mut path = vec![];
        let mut i = Some(start);
        while let Some(k) = i {
            if states[k] == 1 {
                return Err(invalid(
                    "Architecture cyclic group ancestry is not implemented",
                ));
            }
            if states[k] == 2 {
                break;
            }
            states[k] = 1;
            path.push(k);
            i = parents[k];
        }
        for k in path {
            states[k] = 2;
        }
    }
    Ok(())
}
fn imported(g: &ModelGraph) -> Result<ArchitectureModel, PeleError> {
    let root = object(&g.root)?;
    let mut text = Text(0);
    let mut nodes = vec![];
    for (_key, a) in map_entries(g, property(g, root, "nodes"))? {
        let id = object(&a)?;
        let get = |k| property(g, id, k);
        let kind = match get("type") {
            Some(Atom::String(s)) => g.strings[*s as usize].well_formed().unwrap_or_default(),
            _ => String::new(),
        };
        let icon = if kind == "junction" {
            None
        } else {
            optional_icon(g, &mut text, get("icon"))?
        };
        let icon_text = if kind == "junction" || icon.is_some() {
            None
        } else {
            optional_icon(g, &mut text, get("iconText"))?
        };
        let title = if kind == "service" {
            text.get(g, get("title"), true)?
        } else {
            None
        };
        nodes.push((
            text.get(g, get("id"), false)?.unwrap(),
            ArchNode {
                id: text.get(g, get("id"), false)?.unwrap(),
                node_type: kind,
                icon,
                icon_text,
                title,
                parent: optional_parent(g, get("in")),
            },
        ));
    }
    let mut groups = vec![];
    for (_key, a) in map_entries(g, property(g, root, "groups"))? {
        let id = object(&a)?;
        let get = |k| property(g, id, k);
        groups.push((
            text.get(g, get("id"), false)?.unwrap(),
            ArchGroup {
                id: text.get(g, get("id"), false)?.unwrap(),
                icon: optional_icon(g, &mut text, get("icon"))?,
                title: text.get(g, get("title"), true)?,
                parent: optional_parent(g, get("in")),
            },
        ));
    }
    let used: HashSet<_> = nodes
        .iter()
        .flat_map(|(k, n)| [k.as_str(), n.id.as_str()])
        .collect();
    let mut missing = "\0native-unmatched".to_string();
    while used.contains(missing.as_str()) {
        missing.push('\0')
    }
    let lookup = |a: Option<&Atom>| {
        if let Some(Atom::String(s)) = a {
            g.strings[*s as usize]
                .well_formed()
                .unwrap_or_else(|| missing.clone())
        } else {
            missing.clone()
        }
    };
    let mut edges = vec![];
    for a in array(g, property(g, root, "edges").unwrap_or(&Atom::Undefined))? {
        let id = object(&a)?;
        let get = |k| property(g, id, k);
        let lhs = lookup(get("lhsId"));
        let rhs = lookup(get("rhsId"));
        let active = used.contains(lhs.as_str()) && used.contains(rhs.as_str());
        edges.push(ArchEdge {
            lhs_id: lhs,
            rhs_id: rhs,
            lhs_dir: if active {
                direction(g, get("lhsDir"))?
            } else {
                'L'
            },
            rhs_dir: if active {
                direction(g, get("rhsDir"))?
            } else {
                'R'
            },
            lhs_into: get("lhsInto").is_some_and(|a| truthy(g, a)),
            rhs_into: get("rhsInto").is_some_and(|a| truthy(g, a)),
            lhs_group: get("lhsGroup").is_some_and(|a| truthy(g, a)),
            rhs_group: get("rhsGroup").is_some_and(|a| truthy(g, a)),
            title: text.get(g, get("title"), true)?,
        });
    }
    let mut layout_hints = vec![];
    for hint in array(
        g,
        property(g, root, "layoutHints").unwrap_or(&Atom::Undefined),
    )? {
        let id = object(&hint)?;
        let member = property(g, id, "members").unwrap_or(&Atom::Undefined);
        let values = match member {
            Atom::String(s) => g.strings[*s as usize]
                .0
                .iter()
                .map(|u| String::from_utf16(&[*u]).unwrap_or_else(|_| missing.clone()))
                .collect(),
            Atom::Ref(id)
                if matches!(g.nodes[*id as usize], Node::Set { .. } | Node::Map { .. })
                    && property(g, *id, "length").is_none() =>
            {
                vec![]
            }
            _ => array(g, member)?.iter().map(|a| lookup(Some(a))).collect(),
        };
        layout_hints.push(ArchLayoutHint {
            direction: if is(
                g,
                property(g, id, "direction").unwrap_or(&Atom::Undefined),
                "row",
            ) {
                "row".into()
            } else {
                "column".into()
            },
            members: values,
        });
    }
    let count = nodes.len().saturating_add(groups.len());
    if count.saturating_mul(
        count
            .saturating_add(edges.len())
            .saturating_add(layout_hints.iter().map(|h| h.members.len()).sum::<usize>())
            .saturating_add(1),
    ) > 32_000_000
    {
        return Err(limit());
    }
    ancestry(&groups)?;
    let metadata = |text: &mut Text, key| match property(g, root, key) {
        Some(a) if key != "title" && !truthy(g, a) => Ok(None),
        a => text.get(g, a, true),
    };
    Ok(ArchitectureModel {
        title: metadata(&mut text, "title")?,
        acc_title: metadata(&mut text, "accTitle")?,
        acc_descr: metadata(&mut text, "accDescr")?,
        nodes,
        groups,
        edges,
        layout_hints,
        registered: vec![],
    })
}
fn export_record(
    out: &mut GraphWriter,
    values: impl IntoIterator<Item = (&'static str, Atom)>,
) -> Atom {
    let properties = values
        .into_iter()
        .map(|(k, v)| out.property(k, v))
        .collect();
    Atom::Ref(out.node(Node::Object { properties }))
}
fn export_array(out: &mut GraphWriter, values: Vec<Atom>) -> Atom {
    let properties = values
        .into_iter()
        .enumerate()
        .map(|(i, a)| out.property(&i.to_string(), a))
        .collect::<Vec<_>>();
    Atom::Ref(out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    }))
}
fn export_string(out: &mut GraphWriter, s: &Option<String>) -> Atom {
    s.as_deref()
        .map_or(Atom::Undefined, |s| Atom::String(out.string(s)))
}
impl PublicDiagramModule for Architecture {
    type PublicState = ArchitecturePublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(ArchitecturePublicState { graph, model })
    }
    fn create_public(&self, m: &ArchitectureModel) -> Result<ArchitecturePublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut node_entries = vec![];
        for (k, n) in &m.nodes {
            let key = Atom::String(out.string(k.as_str()));
            let id = Atom::String(out.string(n.id.as_str()));
            let kind = Atom::String(out.string(n.node_type.as_str()));
            let parent = export_string(&mut out, &n.parent);
            let values = if n.node_type == "junction" {
                vec![("id", id), ("type", kind), ("in", parent)]
            } else {
                vec![
                    ("id", id),
                    ("type", kind),
                    ("icon", export_string(&mut out, &n.icon)),
                    ("iconText", export_string(&mut out, &n.icon_text)),
                    ("title", export_string(&mut out, &n.title)),
                    ("in", parent),
                ]
            };
            let value = export_record(&mut out, values);
            node_entries.push((key, value));
        }
        let nodes = Atom::Ref(out.node(Node::Map {
            entries: node_entries,
            properties: vec![],
        }));
        let mut group_entries = vec![];
        for (k, n) in &m.groups {
            let key = Atom::String(out.string(k.as_str()));
            let values = [
                ("id", Atom::String(out.string(n.id.as_str()))),
                ("icon", export_string(&mut out, &n.icon)),
                ("title", export_string(&mut out, &n.title)),
                ("in", export_string(&mut out, &n.parent)),
            ];
            let value = export_record(&mut out, values);
            group_entries.push((key, value));
        }
        let groups = Atom::Ref(out.node(Node::Map {
            entries: group_entries,
            properties: vec![],
        }));
        let mut edge_values = vec![];
        for e in &m.edges {
            let values = [
                ("lhsId", Atom::String(out.string(e.lhs_id.as_str()))),
                (
                    "lhsDir",
                    Atom::String(out.string(e.lhs_dir.to_string().as_str())),
                ),
                ("lhsInto", Atom::Bool(e.lhs_into)),
                ("lhsGroup", Atom::Bool(e.lhs_group)),
                ("rhsId", Atom::String(out.string(e.rhs_id.as_str()))),
                (
                    "rhsDir",
                    Atom::String(out.string(e.rhs_dir.to_string().as_str())),
                ),
                ("rhsInto", Atom::Bool(e.rhs_into)),
                ("rhsGroup", Atom::Bool(e.rhs_group)),
                ("title", export_string(&mut out, &e.title)),
            ];
            edge_values.push(export_record(&mut out, values));
        }
        let edges = export_array(&mut out, edge_values);
        let mut hint_values = vec![];
        for hint in &m.layout_hints {
            let values = hint
                .members
                .iter()
                .map(|s| Atom::String(out.string(s.as_str())))
                .collect();
            let members = export_array(&mut out, values);
            let direction = Atom::String(out.string(hint.direction.as_str()));
            hint_values.push(export_record(
                &mut out,
                [("direction", direction), ("members", members)],
            ));
        }
        let hints = export_array(&mut out, hint_values);
        let registered = m
            .registered
            .iter()
            .map(|(k, v)| {
                (
                    Atom::String(out.string(k.as_str())),
                    Atom::String(out.string(v.as_str())),
                )
            })
            .collect();
        let registered = Atom::Ref(out.node(Node::Map {
            entries: registered,
            properties: vec![],
        }));
        let kind = Atom::String(out.string("architecture"));
        let title = export_string(&mut out, &m.title);
        let acc_title = export_string(&mut out, &m.acc_title);
        let acc_descr = export_string(&mut out, &m.acc_descr);
        let properties = vec![
            out.property("type", kind),
            out.property("title", title),
            out.property("accTitle", acc_title),
            out.property("accDescr", acc_descr),
            out.property("nodes", nodes),
            out.property("groups", groups),
            out.property("edges", edges),
            out.property("layoutHints", hints),
            out.property("registered", registered),
        ];
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let graph = out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(ArchitecturePublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &ArchitecturePublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(
        &self,
        s: &mut ArchitecturePublicState,
        g: ModelGraph,
    ) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut ArchitecturePublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke(&mut s.graph, c);
        s.model = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut ArchitecturePublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
