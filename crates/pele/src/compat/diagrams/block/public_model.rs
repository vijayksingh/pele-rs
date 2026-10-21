//! BlockDb native public graph and actual iterative hierarchy builder.
//! Pele 458322a5 (MIT kepano). Rendering reuses the existing Block arena.
use super::{Block, BlockClassDef, BlockModel, BlockNode};
use crate::compat::{
    public::*, source::space, DiagramType, ErrorCode, Object, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use std::collections::{HashMap, HashSet};
const PROTO: PrototypeId = PrototypeId(1);
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
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/block@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "BlockDb",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false, false),
                field_schema("title", FieldKind::String, false, true),
                field_schema("root", FieldKind::Object, false, false),
                field_schema("blocks", FieldKind::Array, false, false),
                field_schema("edges", FieldKind::Array, false, false),
                field_schema("classes", FieldKind::Map, false, false),
                field_schema("byId", FieldKind::Map, false, false),
                field_schema("warn", FieldKind::Function, false, true),
                field_schema("pending", FieldKind::Set, false, false),
                field_schema("edgeCount", FieldKind::Map, false, false),
                field_schema("ids", FieldKind::Number, false, false),
                field_schema("colors", FieldKind::Number, false, false),
                field_schema("spaces", FieldKind::Number, false, false),
            ],
            members: &[
                MemberSchema {
                    name: "typeStr2Type",
                    flags: 7,
                    member: MemberKind::Method {
                        method: MethodId(0),
                    },
                },
                MemberSchema {
                    name: "edgeStrToEdgeData",
                    flags: 7,
                    member: MemberKind::Method {
                        method: MethodId(1),
                    },
                },
                MemberSchema {
                    name: "edgeStrToEdgeStartData",
                    flags: 7,
                    member: MemberKind::Method {
                        method: MethodId(2),
                    },
                },
                MemberSchema {
                    name: "edgeStrToThickness",
                    flags: 7,
                    member: MemberKind::Method {
                        method: MethodId(3),
                    },
                },
                MemberSchema {
                    name: "edgeStrToPattern",
                    flags: 7,
                    member: MemberKind::Method {
                        method: MethodId(4),
                    },
                },
            ],
        },
        RecordSchema {
            id: RecordId(1),
            name: "Block",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, true, true),
                field_schema("type", FieldKind::String, true, true),
                field_schema("label", FieldKind::String, true, true),
                field_schema("children", FieldKind::Array, true, true),
                field_schema("columns", FieldKind::Number, true, true),
                field_schema("widthInColumns", FieldKind::Number, true, true),
                field_schema("width", FieldKind::Number, true, true),
                field_schema("directions", FieldKind::Array, true, true),
                field_schema("classes", FieldKind::Array, true, true),
                field_schema("styles", FieldKind::Array, true, true),
                field_schema("colorIndex", FieldKind::Number, true, true),
                field_schema("start", FieldKind::String, true, true),
                field_schema("end", FieldKind::String, true, true),
                field_schema("arrowTypeStart", FieldKind::String, true, true),
                field_schema("arrowTypeEnd", FieldKind::String, true, true),
                field_schema("thickness", FieldKind::String, true, true),
                field_schema("pattern", FieldKind::String, true, true),
                field_schema("css", FieldKind::String, true, true),
                field_schema("styleClass", FieldKind::String, true, true),
                field_schema("stylesStr", FieldKind::String, true, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "BlockClassDef",
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
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "BlockDb",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "generateId",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(5),
                },
            },
            MemberSchema {
                name: "addStyleClass",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(6),
                },
            },
            MemberSchema {
                name: "addStyle2Node",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(7),
                },
            },
            MemberSchema {
                name: "setCssClass",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(8),
                },
            },
            MemberSchema {
                name: "named",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(9),
                },
            },
            MemberSchema {
                name: "setHierarchy",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(10),
                },
            },
        ],
    }],
    methods: &[
        MethodSchema {
            id: MethodId(0),
            name: "typeStr2Type",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(1),
            name: "edgeStrToEdgeData",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(2),
            name: "edgeStrToEdgeStartData",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(3),
            name: "edgeStrToThickness",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(4),
            name: "edgeStrToPattern",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(5),
            name: "generateId",
            arity: 0,
        },
        MethodSchema {
            id: MethodId(6),
            name: "addStyleClass",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(7),
            name: "addStyle2Node",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(8),
            name: "setCssClass",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(9),
            name: "named",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(10),
            name: "setHierarchy",
            arity: 1,
        },
    ],
};
pub struct BlockPublicState {
    graph: ModelGraph,
    model: Result<BlockModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("block".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Block public operation exceeds native work budget");
    e.code = ErrorCode::Limit;
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
            invalid("Block runtime error text containing lone UTF-16 is not implemented").into()
        }),
        _ => Err(invalid("Block arbitrary object coercion is not implemented").into()),
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
            Err(invalid("Block String collection indexing is not implemented").into())
        }
        _ => Ok(Atom::Undefined),
    }
}
fn intern(g: &mut ModelGraph, s: JsString) -> Result<u32, PublicError> {
    if let Some(i) = g.strings.iter().position(|v| *v == s) {
        return Ok(i as u32);
    }
    let units = g
        .strings
        .iter()
        .fold(0usize, |n, v| n.saturating_add(v.0.len()));
    if g.strings.len() >= 1_000_000 || units.saturating_add(s.0.len()) > 16_000_000 {
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
        _ => return Err(invalid("Block borrowed host receiver is not implemented").into()),
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
fn push_raw(g: &mut ModelGraph, a: Atom, value: Atom, message: &str) -> Result<(), PublicError> {
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
    if g.nodes[id as usize]
        .properties()
        .iter()
        .any(|p| g.strings[p.key as usize].is("length") && p.flags & 2 == 0)
    {
        return Err(type_error(
            "Cannot assign to read only property 'length' of object '[object Array]'",
        ));
    }
    set(g, &a, &length.to_string(), value)?;
    let Some(next) = length.checked_add(1) else {
        return Err(js(JavaScriptErrorKind::RangeError, "Invalid array length"));
    };
    let explicit_length = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is("length"));
    if let Node::Array { length, properties } = &mut g.nodes[id as usize] {
        *length = next;
        if let Some(i) = explicit_length {
            properties[i].value = Atom::Number(next as f64);
        }
    }
    Ok(())
}
fn numeric(g: &ModelGraph, a: &Atom) -> Result<f64, PublicError> {
    match a {
        Atom::Undefined => Ok(f64::NAN),
        Atom::Null => Ok(0.),
        Atom::Bool(v) => Ok(if *v { 1. } else { 0. }),
        Atom::Number(v) => Ok(*v),
        Atom::String(id) => Ok(g.strings[*id as usize]
            .well_formed()
            .map_or(f64::NAN, |s| js_number(&s))),
        _ => Err(invalid("Block object-to-number coercion is not implemented").into()),
    }
}
fn js_number(raw: &str) -> f64 {
    let raw = raw.trim_matches(space);
    if raw.is_empty() {
        return 0.;
    }
    if raw == "Infinity" || raw == "+Infinity" {
        return f64::INFINITY;
    }
    if raw == "-Infinity" {
        return f64::NEG_INFINITY;
    }
    for (prefix, radix) in [
        ("0x", 16),
        ("0X", 16),
        ("0b", 2),
        ("0B", 2),
        ("0o", 8),
        ("0O", 8),
    ] {
        if let Some(digits) = raw.strip_prefix(prefix) {
            if digits.is_empty() {
                return f64::NAN;
            }
            // JavaScript Number rounds the complete binary integer once, to nearest-even.
            // Accumulating floating point digits would round at every step for long literals.
            let mut bit_count = 0usize;
            let mut leading = 0u64;
            let mut guard = false;
            let mut sticky = false;
            let digit_bits = match radix {
                2 => 1,
                8 => 3,
                _ => 4,
            };
            for c in digits.chars() {
                let Some(digit) = c.to_digit(radix) else {
                    return f64::NAN;
                };
                for shift in (0..digit_bits).rev() {
                    let bit = digit >> shift & 1;
                    if bit_count == 0 && bit == 0 {
                        continue;
                    }
                    bit_count += 1;
                    if bit_count <= 53 {
                        leading = leading * 2 + bit as u64;
                    } else if bit_count == 54 {
                        guard = bit != 0;
                    } else {
                        sticky |= bit != 0;
                    }
                }
            }
            if bit_count > 1024 {
                return f64::INFINITY;
            }
            if bit_count <= 53 {
                return leading as f64;
            }
            if guard && (sticky || leading & 1 != 0) {
                leading += 1;
            }
            return leading as f64 * 2f64.powi((bit_count - 53) as i32);
        }
    }
    static DECIMAL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    if !DECIMAL
        .get_or_init(|| {
            regex::Regex::new(r"^[+-]?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:[eE][+-]?[0-9]+)?$").unwrap()
        })
        .is_match(raw)
    {
        return f64::NAN;
    }
    raw.parse().unwrap_or(f64::NAN)
}

fn new_object(g: &mut ModelGraph, fields: &[(&str, Atom)]) -> Result<Atom, PublicError> {
    let a = node(g, Node::Object { properties: vec![] })?;
    for (k, v) in fields {
        set(g, &a, k, v.clone())?
    }
    Ok(a)
}
fn array_method(g: &ModelGraph, a: &Atom, key: &str, message: &str) -> Result<u32, PublicError> {
    let id = match a {
        Atom::Ref(id) => *id,
        Atom::Null | Atom::Undefined => {
            read(g, a, key)?;
            unreachable!()
        }
        _ => return Err(type_error(message)),
    };
    if let Some(v) = property(g, id, key) {
        return Err(
            if matches!(v, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                invalid("Block replaced collection callbacks are not implemented").into()
            } else {
                type_error(message)
            },
        );
    }
    if !matches!(g.nodes[id as usize], Node::Array { .. }) {
        return Err(type_error(message));
    }
    Ok(id)
}
fn push(g: &mut ModelGraph, a: Atom, v: Atom, message: &str) -> Result<(), PublicError> {
    array_method(g, &a, "push", message)?;
    push_raw(g, a, v, message)
}
fn equal(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Number(a), Atom::Number(b)) => a == b || (a.is_nan() && b.is_nan()),
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize] == g.strings[*b as usize],
        (Atom::Ref(a), Atom::Ref(b))
        | (Atom::BuiltinFunction(a), Atom::BuiltinFunction(b))
        | (Atom::HostReference(a), Atom::HostReference(b)) => a == b,
        _ => false,
    }
}
fn map_id(g: &ModelGraph, a: &Atom, method: &str, message: &str) -> Result<u32, PublicError> {
    let Atom::Ref(id) = a else {
        read(g, a, method)?;
        return Err(type_error(message));
    };
    if let Some(v) = property(g, *id, method) {
        return Err(
            if matches!(v, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                invalid("Block replaced Map callbacks are not implemented").into()
            } else {
                type_error(message)
            },
        );
    }
    if !matches!(g.nodes[*id as usize], Node::Map { .. }) {
        return Err(type_error(message));
    }
    Ok(*id)
}
fn map_get(g: &ModelGraph, a: &Atom, key: &Atom, message: &str) -> Result<Atom, PublicError> {
    let id = map_id(g, a, "get", message)?;
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if entries.len() > 1_000_000 {
        return Err(limit().into());
    }
    Ok(entries
        .iter()
        .find_map(|(k, v)| equal(g, k, key).then(|| v.clone()))
        .unwrap_or(Atom::Undefined))
}
fn map_set(
    g: &mut ModelGraph,
    a: &Atom,
    key: Atom,
    value: Atom,
    message: &str,
) -> Result<(), PublicError> {
    let id = map_id(g, a, "set", message)?;
    let Node::Map { entries, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if entries.len() >= 1_000_000 {
        return Err(limit().into());
    }
    let i = entries.iter().position(|(k, _)| equal(g, k, &key));
    if let Node::Map { entries, .. } = &mut g.nodes[id as usize] {
        if let Some(i) = i {
            entries[i].1 = value
        } else {
            entries.push((key, value))
        }
    }
    Ok(())
}
fn key(g: &mut ModelGraph, s: &str) -> Result<Atom, PublicError> {
    Ok(Atom::String(intern(g, s.into())?))
}
fn js_text(g: &ModelGraph, a: &Atom) -> Result<JsString, PublicError> {
    if let Atom::String(id) = a {
        Ok(g.strings[*id as usize].clone())
    } else {
        Ok(JsString::from(primitive(g, a)?.as_str()))
    }
}
fn concatenate(g: &mut ModelGraph, values: &[Atom]) -> Result<Atom, PublicError> {
    let mut units = vec![];
    for a in values {
        let s = js_text(g, a)?;
        if units.len().saturating_add(s.0.len()) > 16_000_000 {
            return Err(limit().into());
        }
        units.extend(s.0)
    }
    Ok(Atom::String(intern(g, JsString(units))?))
}
fn add_values(g: &mut ModelGraph, a: &Atom, b: &Atom) -> Result<Atom, PublicError> {
    if matches!(a, Atom::String(_)) || matches!(b, Atom::String(_)) {
        concatenate(g, &[a.clone(), b.clone()])
    } else {
        Ok(Atom::Number(numeric(g, a)? + numeric(g, b)?))
    }
}
fn nullish(a: &Atom) -> bool {
    matches!(a, Atom::Undefined | Atom::Null)
}
fn strict(g: &ModelGraph, a: &Atom, s: &str) -> bool {
    matches!(a,Atom::String(i)if g.strings[*i as usize].is(s))
}
fn new_array(g: &mut ModelGraph, values: Vec<Atom>) -> Result<Atom, PublicError> {
    let n = values.len();
    let a = node(
        g,
        Node::Array {
            length: n as u32,
            properties: vec![],
        },
    )?;
    for (i, v) in values.into_iter().enumerate() {
        set(g, &a, &i.to_string(), v)?
    }
    Ok(a)
}
fn trim_units(s: &[u16]) -> &[u16] {
    let ws = |u: u16| char::from_u32(u as u32).is_some_and(space);
    let mut from = 0;
    let mut to = s.len();
    while from < to && ws(s[from]) {
        from += 1
    }
    while to > from && ws(s[to - 1]) {
        to -= 1
    }
    &s[from..to]
}
fn string_units(g: &ModelGraph, a: &Atom, method: &str) -> Result<Vec<u16>, PublicError> {
    if let Atom::String(i) = a {
        Ok(g.strings[*i as usize].0.clone())
    } else {
        read(g, a, method)?;
        Err(type_error(format!("e.{method} is not a function")))
    }
}
fn split(g: &mut ModelGraph, a: &Atom, message: &str) -> Result<Vec<Atom>, PublicError> {
    let raw = if let Atom::String(i) = a {
        g.strings[*i as usize].0.clone()
    } else {
        read(g, a, "split")?;
        return Err(type_error(message));
    };
    let count = 1 + raw.iter().filter(|u| **u == 44).count();
    if count.saturating_mul(count) > 32_000_000 {
        return Err(limit().into());
    }
    raw.split(|u| *u == 44)
        .map(|s| Ok(Atom::String(intern(g, JsString(s.to_vec()))?)))
        .collect()
}
fn set_id(g: &ModelGraph, a: &Atom, method: &str, message: &str) -> Result<u32, PublicError> {
    let Atom::Ref(id) = a else {
        read(g, a, method)?;
        return Err(type_error(message));
    };
    if let Some(v) = property(g, *id, method) {
        return Err(
            if matches!(v, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                invalid("Block replaced Set callbacks are not implemented").into()
            } else {
                type_error(message)
            },
        );
    }
    if !matches!(g.nodes[*id as usize], Node::Set { .. }) {
        return Err(type_error(message));
    }
    Ok(*id)
}
fn set_add(g: &mut ModelGraph, a: &Atom, value: Atom) -> Result<(), PublicError> {
    let id = set_id(g, a, "add", "this.pending.add is not a function")?;
    let Node::Set { values, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if values.len() > 1_000_000 {
        return Err(limit().into());
    }
    let present = values.iter().any(|a| equal(g, a, &value));
    if !present {
        if let Node::Set { values, .. } = &mut g.nodes[id as usize] {
            values.push(value)
        }
    }
    Ok(())
}
fn set_delete(g: &mut ModelGraph, a: &Atom, value: &Atom) -> Result<bool, PublicError> {
    let id = set_id(g, a, "delete", "this.pending.delete is not a function")?;
    let Node::Set { values, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    let i = values.iter().position(|a| equal(g, a, value));
    if let Some(i) = i {
        if let Node::Set { values, .. } = &mut g.nodes[id as usize] {
            values.remove(i);
        }
        Ok(true)
    } else {
        Ok(false)
    }
}
fn internal(
    g: &mut ModelGraph,
    r: Atom,
    name: &str,
    arguments: Vec<Atom>,
    depth: usize,
) -> Result<Atom, PublicError> {
    if depth > 64 {
        return Err(limit().into());
    }
    let method = SCHEMA.methods.iter().find(|m| m.name == name).unwrap().id;
    let Atom::Ref(id) = r else {
        read(g, &r, name)?;
        return Err(type_error(format!("this.{name} is not a function")));
    };
    let method = match property(g, id, name) {
        Some(Atom::BuiltinFunction(id)) => MethodId(*id),
        Some(Atom::HostReference(_)) => {
            return Err(invalid("Block replaced host methods are not implemented").into())
        }
        Some(_) => return Err(type_error(format!("this.{name} is not a function"))),
        None => {
            if matches!(
                g.nodes[id as usize],
                Node::Instance {
                    prototype: PROTO,
                    ..
                }
            ) && method.0 >= 5
            {
                method
            } else {
                return Err(type_error(format!("this.{name} is not a function")));
            }
        }
    };
    invoke_inner(
        g,
        MethodCall {
            method,
            receiver: r,
            arguments,
        },
        depth + 1,
    )
}
fn indexed(g: &ModelGraph, a: &Atom, method: &str) -> Result<Vec<Atom>, PublicError> {
    let message = format!("e.{method} is not a function");
    let id = array_method(g, a, method, &message)?;
    elements(g, id)
}
fn elements(g: &ModelGraph, id: u32) -> Result<Vec<Atom>, PublicError> {
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if *length > 1_000_000 || properties.len() > 1_000_000 {
        return Err(limit().into());
    }
    let mut v = vec![];
    for p in properties {
        let s = &g.strings[p.key as usize];
        if s.0.len() > 10 {
            continue;
        }
        if let Some(k) = s.well_formed() {
            if let Ok(i) = k.parse::<u32>() {
                if i < *length && i.to_string() == k {
                    v.push((i, p.value.clone()))
                }
            }
        }
    }
    v.sort_by_key(|(i, _)| *i);
    Ok(v.into_iter().map(|(_, a)| a).collect())
}
struct Frame {
    list: Vec<Atom>,
    at: usize,
    parent: Atom,
    children: Atom,
    columns: Atom,
}
fn frame(g: &mut ModelGraph, statements: &Atom, parent: Atom) -> Result<Frame, PublicError> {
    let mut list = vec![];
    let mut work = 0usize;
    for value in indexed(g, statements, "flat")? {
        if let Atom::Ref(id) = value {
            if matches!(g.nodes[id as usize], Node::Array { .. }) {
                let values = elements(g, id)?;
                work = work.saturating_add(values.len());
                if work > 1_000_000 {
                    return Err(limit().into());
                }
                list.extend(values);
                continue;
            }
        }
        work += 1;
        if work.saturating_mul(g.strings.len().max(1)) > 32_000_000 {
            return Err(limit().into());
        }
        if work > 1_000_000 {
            return Err(limit().into());
        }
        list.push(value)
    }
    let mut columns = Atom::Number(-1.);
    for b in &list {
        if strict(g, &read(g, b, "type")?, "column-setting") {
            let raw = read(g, b, "columns")?;
            columns = if nullish(&raw) {
                Atom::Number(-1.)
            } else {
                raw
            };
            break;
        }
    }
    let children = new_array(g, vec![])?;
    Ok(Frame {
        list,
        at: 0,
        parent,
        children,
        columns,
    })
}
fn hierarchy(
    g: &mut ModelGraph,
    r: Atom,
    statements: Atom,
    depth: usize,
) -> Result<(), PublicError> {
    let root = read(g, &r, "root")?;
    let first = frame(g, &statements, root)?;
    let mut stack = vec![first];
    let mut work = 0usize;
    while let Some(f) = stack.last_mut() {
        if f.at == f.list.len() {
            let f = stack.pop().unwrap();
            set(g, &f.parent, "children", f.children)?;
            continue;
        }
        work += 1;
        if work.saturating_mul(g.strings.len().max(1)) > 32_000_000 {
            return Err(limit().into());
        }
        if work > 1_000_000 {
            return Err(limit().into());
        }
        let block = f.list[f.at].clone();
        f.at += 1;
        let typ = read(g, &block, "type")?;
        let columns = f.columns.clone();
        let width = read(g, &block, "widthInColumns")?;
        if numeric(g, &columns)? > 0.
            && !strict(g, &typ, "column-setting")
            && matches!(width, Atom::Number(_))
            && numeric(g, &width)? > numeric(g, &columns)?
        {
            let warn = read(g, &r, "warn")?;
            if !nullish(&warn) {
                let id = read(g, &block, "id")?;
                let prefix = key(g, "Block ")?;
                let mid = key(g, " width ")?;
                let tail = key(g, " exceeds configured column width ")?;
                let message = concatenate(g, &[prefix, id, mid, width, tail, columns])?;
                match warn {
                    Atom::BuiltinFunction(method) => {
                        invoke_inner(
                            g,
                            MethodCall {
                                method: MethodId(method),
                                receiver: r.clone(),
                                arguments: vec![message],
                            },
                            depth + 1,
                        )?;
                    }
                    Atom::HostReference(_) => {
                        return Err(
                            invalid("Block foreign warning callbacks are not implemented").into(),
                        )
                    }
                    _ => return Err(type_error("this.warn is not a function")),
                }
            }
        }
        if strict(g, &typ, "classDef") {
            let id = read(g, &block, "id")?;
            let css = read(g, &block, "css")?;
            internal(g, r.clone(), "addStyleClass", vec![id, css], depth)?;
            continue;
        }
        if strict(g, &typ, "applyClass") {
            let id = read(g, &block, "id")?;
            let style = read(g, &block, "styleClass")?;
            let style = if nullish(&style) { key(g, "")? } else { style };
            internal(g, r.clone(), "setCssClass", vec![id, style], depth)?;
            continue;
        }
        if strict(g, &typ, "applyStyles") {
            let styles = read(g, &block, "stylesStr")?;
            if truthy(g, &styles) {
                let id = read(g, &block, "id")?;
                internal(g, r.clone(), "addStyle2Node", vec![id, styles], depth)?;
            }
            continue;
        }
        if strict(g, &typ, "column-setting") {
            let value = read(g, &block, "columns")?;
            let value = if nullish(&value) {
                Atom::Number(-1.)
            } else {
                value
            };
            let parent = stack.last().unwrap().parent.clone();
            set(g, &parent, "columns", value)?;
            continue;
        }
        if strict(g, &typ, "edge") {
            let id = read(g, &block, "id")?;
            let counts = read(g, &r, "edgeCount")?;
            let old = map_get(g, &counts, &id, "this.edgeCount.get is not a function")?;
            let old = if nullish(&old) { Atom::Number(0.) } else { old };
            let count = add_values(g, &old, &Atom::Number(1.))?;
            map_set(
                g,
                &counts,
                id.clone(),
                count.clone(),
                "this.edgeCount.set is not a function",
            )?;
            let dash = key(g, "-")?;
            let id = concatenate(g, &[count, dash, id])?;
            set(g, &block, "id", id)?;
            let edges = read(g, &r, "edges")?;
            push(g, edges, block, "this.edges.push is not a function")?;
            continue;
        }
        let mut id = read(g, &block, "id")?;
        if nullish(&id) {
            id = internal(g, r.clone(), "generateId", vec![], depth)?;
            set(g, &block, "id", id.clone())?
        }
        if !truthy(g, &read(g, &block, "label")?) {
            let label = if strict(g, &typ, "composite") {
                key(g, "")?
            } else {
                id.clone()
            };
            set(g, &block, "label", label)?;
        }
        let by_id = read(g, &r, "byId")?;
        let known = map_get(g, &by_id, &id, "this.byId.get is not a function")?;
        let fresh = if matches!(known, Atom::Undefined) {
            true
        } else {
            let pending = read(g, &r, "pending")?;
            set_delete(g, &pending, &id)?
        };
        if !fresh {
            if !strict(g, &typ, "na") {
                set(g, &known, "type", typ.clone())?
            }
            let label = read(g, &block, "label")?;
            if !equal(g, &label, &id) {
                set(g, &known, "label", label)?
            }
        } else {
            if truthy(g, &known) {
                let classes = read(g, &known, "classes")?;
                set(g, &block, "classes", classes)?;
                let styles = read(g, &known, "styles")?;
                set(g, &block, "styles", styles)?
            }
            if strict(g, &typ, "composite") {
                let colors = read(g, &r, "colors")?;
                let old = numeric(g, &colors)?;
                set(g, &r, "colors", Atom::Number(old + 1.))?;
                set(g, &block, "colorIndex", Atom::Number(old))?
            }
            map_set(
                g,
                &by_id,
                id.clone(),
                block.clone(),
                "this.byId.set is not a function",
            )?;
        }
        let inner = read(g, &block, "children")?;
        let children = new_array(g, vec![])?;
        set(g, &block, "children", children)?;
        if strict(g, &typ, "space") {
            let raw = read(g, &block, "width")?;
            let width = if nullish(&raw) { Atom::Number(1.) } else { raw };
            let spaces = read(g, &r, "spaces")?;
            let n = numeric(g, &width)?;
            if !(n <= 100_000. - numeric(g, &spaces)?) {
                let mut e = invalid("A block diagram can hold at most 100000 space blocks.");
                e.code = ErrorCode::Limit;
                return Err(e.into());
            }
            let delta = Atom::Number(if n.is_nan() { f64::NAN } else { n.max(0.) });
            let total = add_values(g, &spaces, &delta)?;
            set(g, &r, "spaces", total)?;
            if !n.is_finite() && n > 0. || n > 1_000_000. {
                return Err(limit().into());
            }
            let count = if n > 0. { n.ceil() as usize } else { 0 };
            work = work.saturating_add(count);
            if work > 1_000_000 {
                return Err(limit().into());
            }
            if work.saturating_mul(g.strings.len().saturating_add(count).max(1)) > 32_000_000 {
                return Err(limit().into());
            }
            for j in 0..count {
                let Atom::Ref(source) = block else {
                    return Err(
                        invalid("Block spread of non-object spaces is not implemented").into(),
                    );
                };
                let properties = g.nodes[source as usize]
                    .properties()
                    .iter()
                    .filter(|p| p.flags & 1 != 0)
                    .map(|p| Property {
                        key: p.key,
                        value: p.value.clone(),
                        flags: 7,
                    })
                    .collect();
                let copy = node(g, Node::Object { properties })?;
                let dash = key(g, "-")?;
                let id = concatenate(g, &[id.clone(), dash, Atom::Number(j as f64)])?;
                set(g, &copy, "id", id.clone())?;
                let children = new_array(g, vec![])?;
                set(g, &copy, "children", children)?;
                map_set(
                    g,
                    &by_id,
                    id,
                    copy.clone(),
                    "this.byId.set is not a function",
                )?;
                let children = stack.last().unwrap().children.clone();
                push(
                    g,
                    children,
                    copy,
                    "Block native frame children.push is not a function",
                )?;
            }
        } else if fresh {
            let children = stack.last().unwrap().children.clone();
            push(
                g,
                children,
                block.clone(),
                "Block native frame children.push is not a function",
            )?;
        }
        if truthy(g, &inner) && numeric(g, &read(g, &inner, "length")?)? > 0. {
            let f = frame(g, &inner, block)?;
            stack.push(f);
        }
    }
    let root = read(g, &r, "root")?;
    let children = read(g, &root, "children")?;
    let children = if nullish(&children) {
        new_array(g, vec![])?
    } else {
        children
    };
    set(g, &r, "blocks", children)?;
    Ok(())
}
fn invoke_inner(g: &mut ModelGraph, c: MethodCall, depth: usize) -> Result<Atom, PublicError> {
    if depth > 64 {
        return Err(limit().into());
    }
    let r = c.receiver;
    let a = c.arguments;
    let first = a.first().cloned().unwrap_or(Atom::Undefined);
    let second = a.get(1).cloned().unwrap_or(Atom::Undefined);
    match c.method.0 {
        0 => {
            let names = [
                ("[]", "square"),
                ("()", "round"),
                ("(())", "circle"),
                (">]", "rect_left_inv_arrow"),
                ("{}", "diamond"),
                ("{{}}", "hexagon"),
                ("([])", "stadium"),
                ("[[]]", "subroutine"),
                ("[()]", "cylinder"),
                ("((()))", "doublecircle"),
                ("[//]", "lean_right"),
                ("[\\\\]", "lean_left"),
                ("[/\\]", "trapezoid"),
                ("[\\/]", "inv_trapezoid"),
                ("<[]>", "block_arrow"),
            ];
            let value = names
                .iter()
                .find_map(|(from, to)| strict(g, &first, from).then_some(*to))
                .unwrap_or("na");
            return key(g, value);
        }
        1 | 2 => {
            let raw = string_units(g, &first, "trim")?;
            let trimmed = trim_units(&raw);
            let value = match if c.method.0 == 1 {
                trimmed.last()
            } else {
                trimmed.first()
            } {
                Some(120) => "arrow_cross",
                Some(111) => "arrow_circle",
                Some(62) if c.method.0 == 1 => "arrow_point",
                Some(60) if c.method.0 == 2 => "arrow_point",
                _ => {
                    if c.method.0 == 1 {
                        ""
                    } else {
                        "arrow_open"
                    }
                }
            };
            return key(g, value);
        }
        3 | 4 => {
            let raw = string_units(g, &first, "includes")?;
            let contains = raw
                .windows(2)
                .any(|s| s == if c.method.0 == 3 { [61, 61] } else { [46, 45] });
            return key(
                g,
                if c.method.0 == 3 {
                    if contains {
                        "thick"
                    } else {
                        "normal"
                    }
                } else if contains {
                    "dotted"
                } else {
                    "solid"
                },
            );
        }
        5 => {
            let raw = read(g, &r, "ids")?;
            let n = numeric(g, &raw)? + 1.;
            set(g, &r, "ids", Atom::Number(n))?;
            let prefix = key(g, "id-")?;
            return concatenate(g, &[prefix, Atom::Number(n)]);
        }
        6 => {
            let classes = read(g, &r, "classes")?;
            let mut found = map_get(g, &classes, &first, "this.classes.get is not a function")?;
            if !truthy(g, &found) {
                let styles = new_array(g, vec![])?;
                let text_styles = new_array(g, vec![])?;
                found = new_object(
                    g,
                    &[
                        ("id", first.clone()),
                        ("styles", styles),
                        ("textStyles", text_styles),
                    ],
                )?;
                map_set(
                    g,
                    &classes,
                    first,
                    found.clone(),
                    "this.classes.set is not a function",
                )?
            }
            let styles = if matches!(second, Atom::Undefined) {
                key(g, "")?
            } else {
                second
            };
            let attributes = split(
                g,
                &styles,
                if c.method.0 == 6 {
                    "n.split is not a function or its return value is not iterable"
                } else {
                    "n.split is not a function"
                },
            )?;
            for attribute in attributes {
                let Atom::String(id) = attribute else {
                    unreachable!()
                };
                let raw = g.strings[id as usize].0.clone();
                let mut fixed = raw.clone();
                if let Some(i) = fixed.iter().position(|u| *u == 59) {
                    fixed.remove(i);
                }
                let fixed = trim_units(&fixed).to_vec();
                if raw.windows(5).any(|s| s == [99, 111, 108, 111, 114]) {
                    let mut text = fixed.clone();
                    for (from, to) in [("fill", "bgFill"), ("color", "fill")] {
                        let from: Vec<u16> = from.encode_utf16().collect();
                        if let Some(i) = text.windows(from.len()).position(|s| s == from) {
                            text.splice(i..i + from.len(), to.encode_utf16());
                        }
                    }
                    let value = Atom::String(intern(g, JsString(text))?);
                    let target = read(g, &found, "textStyles")?;
                    push(g, target, value, "r.textStyles.push is not a function")?;
                }
                let value = Atom::String(intern(g, JsString(fixed))?);
                let target = read(g, &found, "styles")?;
                push(g, target, value, "r.styles.push is not a function")?;
            }
        }
        7 => {
            let block = internal(g, r, "named", vec![first], depth)?;
            let styles = if matches!(second, Atom::Undefined) {
                key(g, "")?
            } else {
                second
            };
            let values = split(
                g,
                &styles,
                if c.method.0 == 6 {
                    "n.split is not a function or its return value is not iterable"
                } else {
                    "n.split is not a function"
                },
            )?;
            let styles = new_array(g, values)?;
            set(g, &block, "styles", styles)?;
        }
        8 => {
            for id in split(
                g,
                &first,
                "t.split is not a function or its return value is not iterable",
            )? {
                let Atom::String(id) = id else { unreachable!() };
                let value = JsString(trim_units(&g.strings[id as usize].0).to_vec());
                let id = Atom::String(intern(g, value)?);
                let block = internal(g, r.clone(), "named", vec![id], depth)?;
                let mut classes = read(g, &block, "classes")?;
                if nullish(&classes) {
                    classes = new_array(g, vec![])?;
                    set(g, &block, "classes", classes.clone())?
                }
                push(
                    g,
                    classes,
                    second.clone(),
                    "this.named(...).classes.push is not a function",
                )?;
            }
        }
        9 => {
            let by_id = read(g, &r, "byId")?;
            let mut block = map_get(g, &by_id, &first, "this.byId.get is not a function")?;
            if matches!(block, Atom::Undefined) {
                let na = key(g, "na")?;
                let children = new_array(g, vec![])?;
                block = new_object(
                    g,
                    &[("id", first.clone()), ("type", na), ("children", children)],
                )?;
                map_set(
                    g,
                    &by_id,
                    first.clone(),
                    block.clone(),
                    "this.byId.set is not a function",
                )?;
                let pending = read(g, &r, "pending")?;
                set_add(g, &pending, first)?;
            }
            return Ok(block);
        }
        10 => hierarchy(g, r, first, depth)?,
        _ => return Err(invalid("Unknown Block method").into()),
    }
    Ok(Atom::Undefined)
}

fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = a {
        Ok(*id)
    } else {
        Err(invalid("Block active record must be an object"))
    }
}
struct Reader {
    scans: usize,
    text: usize,
}
impl Reader {
    fn record(&mut self, g: &ModelGraph, a: &Atom) -> Result<u32, PeleError> {
        let id = reference(a)?;
        self.scans = self
            .scans
            .saturating_add(g.nodes[id as usize].properties().len());
        if self.scans > 32_000_000 {
            return Err(limit());
        }
        Ok(id)
    }
    fn text(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Option<String>, PeleError> {
        match a {
            None | Some(Atom::Undefined | Atom::Null) => Ok(None),
            Some(Atom::String(i)) => {
                let s = &g.strings[*i as usize];
                self.text = self.text.saturating_add(s.0.len());
                if self.text > 16_000_000 {
                    return Err(limit());
                }
                Ok(Some(s.well_formed().ok_or_else(|| {
                    invalid("Block active lone UTF-16 rendering is not implemented")
                })?))
            }
            _ => Err(invalid(
                "Block active non-string text rendering is not implemented",
            )),
        }
    }
    fn array(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
        optional: bool,
    ) -> Result<Vec<Atom>, PeleError> {
        if optional && a.is_none_or(nullish) {
            return Ok(vec![]);
        }
        let id = self.record(
            g,
            a.ok_or_else(|| invalid("Block active Array is missing"))?,
        )?;
        let Node::Array { length, properties } = &g.nodes[id as usize] else {
            return Err(invalid(
                "Block active non-Array iteration is not implemented",
            ));
        };
        if *length > 1_000_000 {
            return Err(limit());
        }
        let mut values = vec![None; *length as usize];
        for p in properties {
            if let Some(k) = g.strings[p.key as usize].well_formed() {
                if let Ok(i) = k.parse::<u32>() {
                    if i < *length && i.to_string() == k {
                        values[i as usize] = Some(p.value.clone())
                    }
                }
            }
        }
        values
            .into_iter()
            .map(|v| {
                v.ok_or_else(|| invalid("Block active sparse Array rendering is not implemented"))
            })
            .collect()
    }
    fn strings(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
    ) -> Result<Option<Vec<String>>, PeleError> {
        if a.is_none_or(nullish) {
            return Ok(None);
        }
        let a = self.array(g, a, false)?;
        a.iter()
            .map(|a| {
                self.text(g, Some(a))?.ok_or_else(|| {
                    invalid("Block nullish active style/direction item is not implemented")
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some)
    }
    fn numeric(&mut self, g: &ModelGraph, id: u32, key: &str) -> Result<Option<f64>, PeleError> {
        match property(g, id, key) {
            None | Some(Atom::Undefined | Atom::Null) => Ok(None),
            Some(Atom::Number(n)) => Ok(Some(*n)),
            _ => Err(invalid(
                "Block coerced active layout values are not implemented",
            )),
        }
    }
}
fn imported(g: &ModelGraph) -> Result<BlockModel, PeleError> {
    let mut b = Reader { scans: 0, text: 0 };
    let root = b.record(g, &g.root)?;
    let mut m = BlockModel {
        diagram_type: DiagramType::Block,
        title: b.text(g, property(g, root, "title"))?,
        nodes: vec![],
        root: 0,
        blocks: vec![],
        edges: vec![],
        classes: vec![],
        by_id: vec![],
        lookup: HashMap::new(),
        positions: HashMap::new(),
        pending: vec![],
        edge_count: vec![],
        ids: 0.,
        colors: 0.,
        spaces: 0.,
        next_array: 0,
    };
    let raw_root = property(g, root, "root").ok_or_else(|| invalid("Block root is missing"))?;
    let start = b.record(g, raw_root)?;
    let mut identities = HashMap::from([(start, 0usize)]);
    let mut todo = vec![start];
    m.nodes.push(BlockNode::default());
    let mut at = 0;
    let mut adjacency = vec![];
    while at < todo.len() {
        let raw = todo[at];
        let id = b.record(g, &Atom::Ref(raw))?;
        let kind = b.text(g, property(g, id, "type"))?;
        let space = kind.as_deref() == Some("space");
        let composite = kind.as_deref() == Some("composite");
        let mut n = BlockNode {
            id: b.text(g, property(g, id, "id"))?,
            kind,
            label: if space {
                None
            } else {
                b.text(g, property(g, id, "label"))?
            },
            ..BlockNode::default()
        };
        if !space {
            if at != 0 {
                if let Some(Atom::Ref(classes)) = property(g, id, "classes") {
                    if property(g, *classes, "join").is_some() {
                        return Err(invalid(
                            "Block replaced active classes.join callbacks are not implemented",
                        ));
                    }
                }
            }
            n.classes = b.strings(g, property(g, id, "classes"))?;
            n.styles = b.strings(g, property(g, id, "styles"))?;
            n.width_in_columns = b.numeric(g, id, "widthInColumns")?;
            if n.kind.as_deref() == Some("block_arrow") {
                n.directions = b.strings(g, property(g, id, "directions"))?
            }
        }
        let mut children = vec![];
        if composite {
            n.columns = b.numeric(g, id, "columns")?;
            for a in b.array(g, property(g, id, "children"), true)? {
                let child = b.record(g, &a)?;
                let next = if let Some(i) = identities.get(&child) {
                    *i
                } else {
                    if todo.len() >= 1_000_000 {
                        return Err(limit());
                    }
                    let next = todo.len();
                    identities.insert(child, next);
                    todo.push(child);
                    m.nodes.push(BlockNode::default());
                    next
                };
                children.push(next)
            }
            n.children = Some(children.clone())
        }
        adjacency.push(children);
        m.nodes[at] = n;
        at += 1;
    }
    // A repeated record produces a separate View per occurrence. Bound expanded
    // work before the unchanged renderer traverses it, rejecting cycles as well.
    let mut active = vec![false; adjacency.len()];
    let mut done = vec![false; adjacency.len()];
    let mut sizes = vec![0usize; adjacency.len()];
    let mut expanded_text = vec![0usize; adjacency.len()];
    let mut stack = vec![(0usize, false)];
    while let Some((i, exit)) = stack.pop() {
        if exit {
            let mut n = 1usize;
            for &c in &adjacency[i] {
                n = n.saturating_add(sizes[c]);
                if n > 1_000_000 {
                    return Err(limit());
                }
            }
            let own = m.nodes[i]
                .label
                .as_ref()
                .map_or(0, |s| s.len())
                .saturating_add(
                    m.nodes[i]
                        .styles
                        .as_ref()
                        .map_or(0, |v| v.iter().map(String::len).sum::<usize>()),
                );
            let mut text = own;
            for &child in &adjacency[i] {
                text = text.saturating_add(expanded_text[child]);
            }
            if text > 32_000_000 {
                return Err(limit());
            }
            expanded_text[i] = text;
            sizes[i] = n;
            active[i] = false;
            done[i] = true;
            continue;
        }
        if done[i] {
            continue;
        }
        if active[i] {
            return Err(invalid(
                "Block cyclic child rendering exceeds native work budget",
            ));
        }
        active[i] = true;
        stack.push((i, true));
        for &c in adjacency[i].iter().rev() {
            if active[c] {
                return Err(invalid(
                    "Block cyclic child rendering exceeds native work budget",
                ));
            }
            if !done[c] {
                stack.push((c, false))
            }
        }
    }
    let classes =
        property(g, root, "classes").ok_or_else(|| invalid("Block classes Map is missing"))?;
    let class_id = b.record(g, classes)?;
    let Node::Map { entries, .. } = &g.nodes[class_id as usize] else {
        return Err(invalid("Block active classes must be a Map"));
    };
    if property(g, class_id, "get").is_some() {
        return Err(invalid(
            "Block replaced classes Map callbacks are not implemented",
        ));
    }
    let mut used = HashSet::new();
    for n in &m.nodes {
        if n.kind.as_deref() == Some("space") {
            continue;
        }
        if let Some(names) = n.classes.as_ref().filter(|a| !a.is_empty()) {
            used.extend(names.iter().cloned())
        } else {
            used.insert("default".into());
        }
    }
    for (k, v) in entries {
        let Atom::String(key) = k else { continue };
        let Some(key) = g.strings[*key as usize].well_formed() else {
            continue;
        };
        if !used.contains(&key) || !truthy(g, v) {
            continue;
        }
        let id = b.record(g, v)?;
        let styles = b
            .strings(g, property(g, id, "styles"))?
            .ok_or_else(|| invalid("Block active class styles must be iterable"))?;
        m.classes.push((
            key.clone(),
            BlockClassDef {
                id: key,
                styles,
                text_styles: vec![],
            },
        ));
    }
    let mut endpoints = HashSet::new();
    for n in m.nodes.iter().skip(1) {
        if n.kind.as_deref() != Some("space") {
            if let Some(id) = &n.id {
                endpoints.insert(id.clone());
            }
        }
    }
    for e in b.array(g, property(g, root, "edges"), false)? {
        let id = b.record(g, &e)?;
        let start = b.text(g, property(g, id, "start"))?;
        let end = b.text(g, property(g, id, "end"))?;
        if !start.as_ref().is_some_and(|s| endpoints.contains(s))
            || !end.as_ref().is_some_and(|s| endpoints.contains(s))
        {
            continue;
        }
        let edge = BlockNode {
            id: b.text(g, property(g, id, "id"))?,
            start,
            end,
            label: b.text(g, property(g, id, "label"))?,
            arrow_type_start: b.text(g, property(g, id, "arrowTypeStart"))?,
            arrow_type_end: b.text(g, property(g, id, "arrowTypeEnd"))?,
            thickness: b.text(g, property(g, id, "thickness"))?,
            pattern: b.text(g, property(g, id, "pattern"))?,
            ..BlockNode::default()
        };
        let i = m.nodes.len();
        m.nodes.push(edge);
        m.edges.push(i);
    }
    Ok(m)
}
fn array_out(out: &mut GraphWriter, values: Vec<Atom>) -> u32 {
    let properties = values
        .into_iter()
        .enumerate()
        .map(|(i, a)| out.property(&i.to_string(), a))
        .collect::<Vec<_>>();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn option_out(out: &mut GraphWriter, s: Option<&str>) -> Atom {
    s.map_or(Atom::Undefined, |s| Atom::String(out.string(s)))
}
fn array_origin_out(
    out: &mut GraphWriter,
    origins: &mut HashMap<usize, (u32, Vec<Atom>)>,
    origin: Option<usize>,
    values: Vec<Atom>,
) -> Atom {
    if let Some(origin) = origin {
        if let Some((id, original)) = origins.get(&origin) {
            let same = original.len() == values.len()
                && original.iter().zip(&values).all(|(a, b)| match (a, b) {
                    (Atom::String(a), Atom::String(b)) | (Atom::Ref(a), Atom::Ref(b)) => a == b,
                    (Atom::Number(a), Atom::Number(b)) => a.to_bits() == b.to_bits(),
                    (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
                    (Atom::Bool(a), Atom::Bool(b)) => a == b,
                    _ => false,
                });
            // Native Vec fields can be edited independently. A changed vector
            // detaches rather than silently replacing another aliased vector.
            if same {
                return Atom::Ref(*id);
            }
            return Atom::Ref(array_out(out, values));
        }
        let id = array_out(out, values.clone());
        origins.insert(origin, (id, values));
        Atom::Ref(id)
    } else {
        Atom::Ref(array_out(out, values))
    }
}
fn node_out(
    out: &mut GraphWriter,
    n: &BlockNode,
    refs: &[u32],
    origins: &mut HashMap<usize, (u32, Vec<Atom>)>,
) -> Result<Vec<Property>, PeleError> {
    let mut keys = n.own_keys.clone();
    // Typed caller-added fields remain visible. Parser-captured order keeps
    // presentUndefined distinct from absent and survives shallow copies.
    for (key, present) in [
        ("id", n.id.is_some()),
        ("type", n.kind.is_some()),
        ("label", n.label.is_some()),
        ("children", n.children.is_some()),
        ("columns", n.columns.is_some()),
        ("widthInColumns", n.width_in_columns.is_some()),
        ("width", n.width.is_some()),
        ("directions", n.directions.is_some()),
        ("classes", n.classes.is_some()),
        ("styles", n.styles.is_some()),
        ("colorIndex", n.color_index.is_some()),
        ("start", n.start.is_some()),
        ("end", n.end.is_some()),
        ("arrowTypeStart", n.arrow_type_start.is_some()),
        ("arrowTypeEnd", n.arrow_type_end.is_some()),
        ("thickness", n.thickness.is_some()),
        ("pattern", n.pattern.is_some()),
        ("css", n.css.is_some()),
        ("styleClass", n.style_class.is_some()),
        ("stylesStr", n.styles_str.is_some()),
    ] {
        if present && !keys.iter().any(|k| k == key) {
            keys.push(key.into())
        }
    }
    let mut properties = vec![];
    let mut extra = n.extra.iter().collect::<Vec<_>>();
    extra.sort_by_key(|(k, _)| k.parse::<u32>().unwrap_or(u32::MAX));
    for (k, i) in extra {
        properties.push(
            out.property(
                k,
                Atom::Ref(
                    *refs
                        .get(*i)
                        .ok_or_else(|| invalid("Block typed arena index is invalid"))?,
                ),
            ),
        )
    }
    for key in keys {
        let value = match key.as_str() {
            "id" => option_out(out, n.id.as_deref()),
            "type" => option_out(out, n.kind.as_deref()),
            "label" => option_out(out, n.label.as_deref()),
            "start" => option_out(out, n.start.as_deref()),
            "end" => option_out(out, n.end.as_deref()),
            "arrowTypeStart" => option_out(out, n.arrow_type_start.as_deref()),
            "arrowTypeEnd" => option_out(out, n.arrow_type_end.as_deref()),
            "thickness" => option_out(out, n.thickness.as_deref()),
            "pattern" => option_out(out, n.pattern.as_deref()),
            "css" => option_out(out, n.css.as_deref()),
            "styleClass" => option_out(out, n.style_class.as_deref()),
            "stylesStr" => option_out(out, n.styles_str.as_deref()),
            "columns" => n.columns.map_or(Atom::Undefined, Atom::Number),
            "widthInColumns" => n.width_in_columns.map_or(Atom::Undefined, Atom::Number),
            "width" => n.width.map_or(Atom::Undefined, Atom::Number),
            "colorIndex" => n
                .color_index
                .map_or(Atom::Undefined, |v| Atom::Number(v as f64)),
            "directions" | "classes" | "styles" => {
                let (i, values) = match key.as_str() {
                    "directions" => (0, &n.directions),
                    "classes" => (1, &n.classes),
                    _ => (2, &n.styles),
                };
                if let Some(values) = values {
                    let a = values
                        .iter()
                        .map(|s| Atom::String(out.string(s.as_str())))
                        .collect();
                    array_origin_out(out, origins, n.array_origins[i], a)
                } else {
                    Atom::Undefined
                }
            }
            "children" => {
                if let Some(values) = &n.children {
                    let a = values
                        .iter()
                        .map(|i| {
                            refs.get(*i)
                                .copied()
                                .map(Atom::Ref)
                                .ok_or_else(|| invalid("Block typed child arena index is invalid"))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    array_origin_out(out, origins, n.array_origins[3], a)
                } else {
                    Atom::Undefined
                }
            }
            _ => return Err(invalid("Block typed provenance contains an unknown key")),
        };
        properties.push(out.property(&key, value));
    }
    Ok(properties)
}
fn construct_default() -> Result<BlockPublicState, PeleError> {
    let mut out = GraphWriter::default();
    let children = array_out(&mut out, vec![]);
    let root_id = out.string("root");
    let composite = out.string("composite");
    let properties = vec![
        out.property("id", Atom::String(root_id)),
        out.property("type", Atom::String(composite)),
        out.property("children", Atom::Ref(children)),
        out.property("columns", Atom::Number(-1.0)),
    ];
    let hierarchy_root = out.node(Node::Object { properties });
    // Fresh BlockDb has distinct root.children and blocks arrays. Source parsing
    // later aliases these through setHierarchy; construction must not do so.
    let blocks = array_out(&mut out, vec![]);
    let edges = array_out(&mut out, vec![]);
    let classes = out.node(Node::Map {
        entries: vec![],
        properties: vec![],
    });
    let by_id = out.node(Node::Map {
        entries: vec![],
        properties: vec![],
    });
    let pending = out.node(Node::Set {
        values: vec![],
        properties: vec![],
    });
    let edge_count = out.node(Node::Map {
        entries: vec![],
        properties: vec![],
    });
    let kind = out.string("block");
    let mut properties = vec![
        out.property("type", Atom::String(kind)),
        out.property("title", Atom::Undefined),
        out.property("root", Atom::Ref(hierarchy_root)),
        out.property("blocks", Atom::Ref(blocks)),
        out.property("edges", Atom::Ref(edges)),
        out.property("classes", Atom::Ref(classes)),
        out.property("byId", Atom::Ref(by_id)),
    ];
    for (id, name) in [
        "typeStr2Type",
        "edgeStrToEdgeData",
        "edgeStrToEdgeStartData",
        "edgeStrToThickness",
        "edgeStrToPattern",
    ]
    .into_iter()
    .enumerate()
    {
        properties.push(out.property(name, Atom::BuiltinFunction(id as u32)));
    }
    properties.extend([
        out.property("warn", Atom::Undefined),
        out.property("pending", Atom::Ref(pending)),
        out.property("edgeCount", Atom::Ref(edge_count)),
        out.property("ids", Atom::Number(0.0)),
        out.property("colors", Atom::Number(0.0)),
        out.property("spaces", Atom::Number(0.0)),
    ]);
    let root = out.node(Node::Instance {
        prototype: PROTO,
        properties,
    });
    let graph = out.finish(Atom::Ref(root))?;
    let model = imported(&graph);
    Ok(BlockPublicState { graph, model })
}
fn construct_initial(_: ConstructorArgument) -> Result<BlockPublicState, PeleError> {
    construct_default()
}
enum BlockConstruction {
    Start(ConstructorArgument),
    Warn,
    Done,
}
fn resume_construction(
    state: &mut BlockPublicState,
    phase: &mut BlockConstruction,
    reply: Option<PublicHostValue>,
) -> PublicInvocationPhase {
    fn run(
        state: &mut BlockPublicState,
        phase: &mut BlockConstruction,
        reply: Option<PublicHostValue>,
    ) -> Result<PublicInvocationPhase, PublicError> {
        Ok(match std::mem::replace(phase, BlockConstruction::Done) {
            BlockConstruction::Start(argument) => {
                let options = if matches!(argument, ConstructorArgument::Undefined) {
                    new_object(&mut state.graph, &[])?
                } else {
                    argument.into_atom(|value| intern(&mut state.graph, value).map(Atom::String))?
                };
                *phase = BlockConstruction::Warn;
                PublicInvocationPhase::Host(PublicHostOperation::Get {
                    receiver: options,
                    key: "warn",
                })
            }
            BlockConstruction::Warn => {
                let value = reply
                    .expect("awaited warn")
                    .into_atom(|value| intern(&mut state.graph, value).map(Atom::String))?;
                let root = state.graph.root.clone();
                set(&mut state.graph, &root, "warn", value)?;
                PublicInvocationPhase::Complete(Ok(Atom::Undefined))
            }
            BlockConstruction::Done => {
                return Err(invalid("Block construction already finished").into())
            }
        })
    }
    let result = run(state, phase, reply);
    state.model = imported(&state.graph);
    match result {
        Ok(step) => step,
        Err(error) => PublicInvocationPhase::Complete(Err(error)),
    }
}
fn constructor_body(argument: ConstructorArgument) -> DetachedPublicInvocation {
    DetachedPublicInvocation::new::<Block, BlockPublicState, _>(
        BlockConstruction::Start(argument),
        resume_construction,
    )
}

impl PublicDiagramModule for Block {
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTO).then(|| {
            PublicConstructorFactory::first_value_body::<Self>(
                PROTO,
                construct_default,
                construct_initial,
                constructor_body,
                &[],
            )
        })
    }

    type PublicState = BlockPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn shared_functions(&self) -> &'static [SharedFunction] {
        &[
            SharedFunction {
                method: MethodId(0),
                identity: "pele/block/typeStr2Type@1",
            },
            SharedFunction {
                method: MethodId(1),
                identity: "pele/block/edgeStrToEdgeData@1",
            },
            SharedFunction {
                method: MethodId(2),
                identity: "pele/block/edgeStrToEdgeStartData@1",
            },
            SharedFunction {
                method: MethodId(3),
                identity: "pele/block/edgeStrToThickness@1",
            },
            SharedFunction {
                method: MethodId(4),
                identity: "pele/block/edgeStrToPattern@1",
            },
        ]
    }
    fn function_shapes(&self) -> &'static [FunctionShape] {
        &[
            FunctionShape {
                method: MethodId(0),
                constructible: true,
            },
            FunctionShape {
                method: MethodId(1),
                constructible: true,
            },
            FunctionShape {
                method: MethodId(2),
                constructible: true,
            },
            FunctionShape {
                method: MethodId(3),
                constructible: true,
            },
            FunctionShape {
                method: MethodId(4),
                constructible: true,
            },
        ]
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(BlockPublicState { graph, model })
    }
    fn create_public(&self, m: &BlockModel) -> Result<BlockPublicState, PeleError> {
        if m.nodes.len() > 1_000_000 || m.root >= m.nodes.len() {
            return Err(limit());
        }
        let mut out = GraphWriter::default();
        let refs = m
            .nodes
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        let mut origins = HashMap::new();
        let mut fills = vec![];
        for (i, n) in m.nodes.iter().enumerate() {
            fills.push((refs[i], node_out(&mut out, n, &refs, &mut origins)?))
        }
        let blocks = m
            .blocks
            .iter()
            .map(|i| {
                refs.get(*i)
                    .copied()
                    .map(Atom::Ref)
                    .ok_or_else(|| invalid("Block typed blocks arena index is invalid"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let blocks = if m.nodes[m.root].children.as_ref() == Some(&m.blocks) {
            array_origin_out(
                &mut out,
                &mut origins,
                m.nodes[m.root].array_origins[3],
                blocks,
            )
        } else {
            Atom::Ref(array_out(&mut out, blocks))
        };
        let edges = m
            .edges
            .iter()
            .map(|i| {
                refs.get(*i)
                    .copied()
                    .map(Atom::Ref)
                    .ok_or_else(|| invalid("Block typed edge arena index is invalid"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let edges = Atom::Ref(array_out(&mut out, edges));
        let mut entries = vec![];
        for (key, c) in &m.classes {
            let styles = c
                .styles
                .iter()
                .map(|s| Atom::String(out.string(s.as_str())))
                .collect();
            let styles = array_out(&mut out, styles);
            let text = c
                .text_styles
                .iter()
                .map(|s| Atom::String(out.string(s.as_str())))
                .collect();
            let text = array_out(&mut out, text);
            let id = out.string(c.id.as_str());
            let properties = vec![
                out.property("id", Atom::String(id)),
                out.property("styles", Atom::Ref(styles)),
                out.property("textStyles", Atom::Ref(text)),
            ];
            let object = out.node(Node::Object { properties });
            entries.push((Atom::String(out.string(key.as_str())), Atom::Ref(object)));
        }
        let classes = out.node(Node::Map {
            entries,
            properties: vec![],
        });
        let entries = m
            .by_id
            .iter()
            .map(|(k, i)| {
                Ok((
                    Atom::String(out.string(k.as_str())),
                    Atom::Ref(
                        *refs
                            .get(*i)
                            .ok_or_else(|| invalid("Block typed byId arena index is invalid"))?,
                    ),
                ))
            })
            .collect::<Result<Vec<_>, PeleError>>()?;
        let by_id = out.node(Node::Map {
            entries,
            properties: vec![],
        });
        let values = m
            .pending
            .iter()
            .map(|s| Atom::String(out.string(s.as_str())))
            .collect();
        let pending = out.node(Node::Set {
            values,
            properties: vec![],
        });
        let entries = m
            .edge_count
            .iter()
            .map(|(k, v)| {
                (
                    Atom::String(out.string(k.as_str())),
                    Atom::Number(*v as f64),
                )
            })
            .collect();
        let counts = out.node(Node::Map {
            entries,
            properties: vec![],
        });
        let title = option_out(&mut out, m.title.as_deref());
        let typ = out.string("block");
        let mut properties = vec![
            out.property("type", Atom::String(typ)),
            out.property("title", title),
            out.property("root", Atom::Ref(refs[m.root])),
            out.property("blocks", blocks),
            out.property("edges", edges),
            out.property("classes", Atom::Ref(classes)),
            out.property("byId", Atom::Ref(by_id)),
        ];
        for (i, name) in [
            "typeStr2Type",
            "edgeStrToEdgeData",
            "edgeStrToEdgeStartData",
            "edgeStrToThickness",
            "edgeStrToPattern",
        ]
        .into_iter()
        .enumerate()
        {
            properties.push(out.property(name, Atom::BuiltinFunction(i as u32)))
        }
        properties.extend([
            out.property("warn", Atom::Undefined),
            out.property("pending", Atom::Ref(pending)),
            out.property("edgeCount", Atom::Ref(counts)),
            out.property("ids", Atom::Number(m.ids)),
            out.property("colors", Atom::Number(m.colors)),
            out.property("spaces", Atom::Number(m.spaces)),
        ]);
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let mut graph = out.finish(Atom::Ref(root))?;
        for (i, p) in fills {
            *graph.nodes[i as usize].properties_mut() = p
        }
        graph.validate_schema(&SCHEMA)?;
        let model = imported(&graph);
        Ok(BlockPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &BlockPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut BlockPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut BlockPublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let n = s.graph.nodes.iter().fold(s.graph.strings.len(), |n, v| {
            n.saturating_add(v.properties().len())
        });
        let result = if n > 2_000_000 {
            Err(limit().into())
        } else {
            invoke_inner(&mut s.graph, c, 0)
        };
        s.model = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut BlockPublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let root = reference(&s.graph.root)?;
        let mut b = Reader { scans: 0, text: 0 };
        let title = b.text(&s.graph, property(&s.graph, root, "accTitle"))?;
        let descr = b.text(&s.graph, property(&s.graph, root, "accDescr"))?;
        super::render::render_accessible(
            s.model.as_ref().map_err(Clone::clone)?,
            c,
            o,
            r,
            title.as_deref(),
            descr.as_deref(),
        )
    }
}
