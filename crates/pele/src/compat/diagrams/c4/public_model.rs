//! C4Db returned instances, real prototype helpers and shared native arena.
//! Pele 458322a5 (MIT kepano). No JavaScript runtime or alternate renderer.
use super::{C4Boundary, C4Element, C4Model, C4Named, C4Rel, C4Shape, ElementRef, C4};
use crate::compat::{
    public::*, source::space, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
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
    id: "pele/c4@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "C4",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false, false),
                field_schema("c4Type", FieldKind::String, false, true),
                field_schema("title", FieldKind::String, false, true),
                field_schema("accTitle", FieldKind::String, false, true),
                field_schema("accDescr", FieldKind::String, false, true),
                field_schema("shapes", FieldKind::Array, false, false),
                field_schema("boundaries", FieldKind::Array, false, false),
                field_schema("rels", FieldKind::Array, false, false),
                field_schema("shapeInRow", FieldKind::Number, false, false),
                field_schema("boundaryInRow", FieldKind::Number, false, false),
                field_schema("shapeIndex", FieldKind::Map, false, false),
                field_schema("boundaryIndex", FieldKind::Map, false, false),
                field_schema("relIndex", FieldKind::Map, false, false),
                field_schema("current", FieldKind::Object, false, false),
                field_schema("stack", FieldKind::Array, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "C4Shape",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("alias", FieldKind::String, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("aliasAttr", FieldKind::Object, true, false),
                field_schema("labelAttr", FieldKind::Object, true, true),
                field_schema("parent", FieldKind::Object, false, true),
                field_schema("type", FieldKind::String, true, false),
                field_schema("descr", FieldKind::String, true, false),
                field_schema("techn", FieldKind::String, true, false),
                field_schema("sprite", FieldKind::String, true, false),
                field_schema("tags", FieldKind::String, true, false),
                field_schema("link", FieldKind::String, true, false),
                field_schema("bgColor", FieldKind::String, true, false),
                field_schema("fontColor", FieldKind::String, true, false),
                field_schema("borderColor", FieldKind::String, true, false),
                field_schema("shadowing", FieldKind::String, true, false),
                field_schema("shape", FieldKind::String, true, false),
                field_schema("legendText", FieldKind::String, true, false),
                field_schema("legendSprite", FieldKind::String, true, false),
                field_schema("extra", FieldKind::Map, true, false),
                field_schema("kind", FieldKind::String, false, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "C4Boundary",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("alias", FieldKind::String, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("aliasAttr", FieldKind::Object, true, false),
                field_schema("labelAttr", FieldKind::Object, true, true),
                field_schema("parent", FieldKind::Object, false, true),
                field_schema("type", FieldKind::String, true, false),
                field_schema("descr", FieldKind::String, true, false),
                field_schema("techn", FieldKind::String, true, false),
                field_schema("sprite", FieldKind::String, true, false),
                field_schema("tags", FieldKind::String, true, false),
                field_schema("link", FieldKind::String, true, false),
                field_schema("bgColor", FieldKind::String, true, false),
                field_schema("fontColor", FieldKind::String, true, false),
                field_schema("borderColor", FieldKind::String, true, false),
                field_schema("shadowing", FieldKind::String, true, false),
                field_schema("shape", FieldKind::String, true, false),
                field_schema("legendText", FieldKind::String, true, false),
                field_schema("legendSprite", FieldKind::String, true, false),
                field_schema("extra", FieldKind::Map, true, false),
                field_schema("nodeType", FieldKind::String, true, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "C4Rel",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("type", FieldKind::String, false, false),
                field_schema("from", FieldKind::String, false, false),
                field_schema("to", FieldKind::String, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("fromAttr", FieldKind::Object, true, false),
                field_schema("toAttr", FieldKind::Object, true, false),
                field_schema("labelAttr", FieldKind::Object, true, true),
                field_schema("techn", FieldKind::String, true, false),
                field_schema("descr", FieldKind::String, true, false),
                field_schema("sprite", FieldKind::String, true, false),
                field_schema("tags", FieldKind::String, true, false),
                field_schema("link", FieldKind::String, true, false),
                field_schema("textColor", FieldKind::String, true, false),
                field_schema("lineColor", FieldKind::String, true, false),
                field_schema("offsetX", FieldKind::Number, true, false),
                field_schema("offsetY", FieldKind::Number, true, false),
                field_schema("extra", FieldKind::Map, true, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "C4Named",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("key", FieldKind::String, false, false),
                field_schema("value", FieldKind::String, false, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "C4Db",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "setC4Type",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(0),
                },
            },
            MemberSchema {
                name: "setTitle",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(1),
                },
            },
            MemberSchema {
                name: "setAccDescription",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(2),
                },
            },
            MemberSchema {
                name: "element",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(3),
                },
            },
            MemberSchema {
                name: "addPersonOrSystem",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(4),
                },
            },
            MemberSchema {
                name: "addContainer",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(5),
                },
            },
            MemberSchema {
                name: "addComponent",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(6),
                },
            },
            MemberSchema {
                name: "finishShape",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(7),
                },
            },
            MemberSchema {
                name: "boundary",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(8),
                },
            },
            MemberSchema {
                name: "open",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(9),
                },
            },
            MemberSchema {
                name: "addPersonOrSystemBoundary",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(10),
                },
            },
            MemberSchema {
                name: "addContainerBoundary",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(11),
                },
            },
            MemberSchema {
                name: "addDeploymentNode",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(12),
                },
            },
            MemberSchema {
                name: "popBoundaryParseStack",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(13),
                },
            },
            MemberSchema {
                name: "addRel",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(14),
                },
            },
            MemberSchema {
                name: "getC4Shape",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(15),
                },
            },
            MemberSchema {
                name: "updateElStyle",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(16),
                },
            },
            MemberSchema {
                name: "updateRelStyle",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(17),
                },
            },
            MemberSchema {
                name: "updateLayoutConfig",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(18),
                },
            },
        ],
    }],
    methods: &[
        MethodSchema {
            id: MethodId(0),
            name: "setC4Type",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(1),
            name: "setTitle",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(2),
            name: "setAccDescription",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(3),
            name: "element",
            arity: 4,
        },
        MethodSchema {
            id: MethodId(4),
            name: "addPersonOrSystem",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(5),
            name: "addContainer",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(6),
            name: "addComponent",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(7),
            name: "finishShape",
            arity: 4,
        },
        MethodSchema {
            id: MethodId(8),
            name: "boundary",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(9),
            name: "open",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(10),
            name: "addPersonOrSystemBoundary",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(11),
            name: "addContainerBoundary",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(12),
            name: "addDeploymentNode",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(13),
            name: "popBoundaryParseStack",
            arity: 0,
        },
        MethodSchema {
            id: MethodId(14),
            name: "addRel",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(15),
            name: "getC4Shape",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(16),
            name: "updateElStyle",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(17),
            name: "updateRelStyle",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(18),
            name: "updateLayoutConfig",
            arity: 2,
        },
    ],
};
pub struct C4PublicState {
    graph: ModelGraph,
    model: Result<C4Model, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("c4".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("C4 public operation exceeds native work budget");
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
            invalid("C4 runtime error text containing lone UTF-16 is not implemented").into()
        }),
        _ => Err(invalid("C4 arbitrary object coercion is not implemented").into()),
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
            Err(invalid("C4 String collection indexing is not implemented").into())
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
        _ => return Err(invalid("C4 borrowed host receiver is not implemented").into()),
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
    if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
        *length = next;
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
        _ => Err(invalid("C4 object-to-number coercion is not implemented").into()),
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
                invalid("C4 replaced collection callbacks are not implemented").into()
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
                invalid("C4 replaced Map callbacks are not implemented").into()
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
fn new_map(g: &mut ModelGraph) -> Result<Atom, PublicError> {
    node(
        g,
        Node::Map {
            entries: vec![],
            properties: vec![],
        },
    )
}
fn key(g: &mut ModelGraph, name: &str) -> Result<Atom, PublicError> {
    Ok(Atom::String(intern(g, name.into())?))
}
fn attr(g: &ModelGraph, a: &Atom, i: usize) -> Result<Atom, PublicError> {
    read(g, a, &i.to_string())
}
fn string_kind(a: &Atom) -> bool {
    matches!(a, Atom::String(_))
}
fn nullish(a: &Atom) -> bool {
    matches!(a, Atom::Null | Atom::Undefined)
}
fn parse_integer(g: &ModelGraph, a: &Atom) -> Result<f64, PublicError> {
    match a {
        Atom::String(id) => {
            // parseInt consumes an ASCII prefix after whitespace. A trailing lone UTF16
            // cannot change that prefix, so replace only the unrepresentable tail.
            let raw = String::from_utf16_lossy(&g.strings[*id as usize].0);
            Ok(super::model::parse_int(&raw))
        }
        _ => Ok(super::model::parse_int(&primitive(g, a)?)),
    }
}
fn named(g: &mut ModelGraph, target: &Atom, a: &Atom) -> Result<(), PublicError> {
    if matches!(a, Atom::Null | Atom::Undefined) {
        return Err(type_error(format!(
            "Cannot destructure property 'key' of 't' as it is {}.",
            if matches!(a, Atom::Null) {
                "null"
            } else {
                "undefined"
            }
        )));
    }
    let k = read(g, a, "key")?;
    let v = read(g, a, "value")?;
    let Atom::Ref(id) = target else {
        return Err(type_error("C4 named attribute target must be an object"));
    };
    let rel = property(g, *id, "from").is_some();
    let name = if let Atom::String(id) = k {
        g.strings[id as usize].well_formed()
    } else {
        None
    };
    if rel && matches!(name.as_deref(), Some("offsetX" | "offsetY")) {
        let n = parse_integer(g, &v)?;
        return set(g, target, name.as_deref().unwrap(), Atom::Number(n));
    }
    let fields: &[&str] = if rel {
        &[
            "label",
            "descr",
            "techn",
            "sprite",
            "tags",
            "link",
            "textColor",
            "lineColor",
        ]
    } else {
        &[
            "label",
            "type",
            "descr",
            "techn",
            "sprite",
            "tags",
            "link",
            "bgColor",
            "fontColor",
            "borderColor",
            "shadowing",
            "shape",
            "legendText",
            "legendSprite",
        ]
    };
    if let Some(name) = name.as_deref().filter(|name| fields.contains(name)) {
        return set(g, target, name, v);
    }
    let mut extra = read(g, target, "extra")?;
    if nullish(&extra) {
        extra = new_map(g)?;
        set(g, target, "extra", extra.clone())?
    }
    map_set(
        g,
        &extra,
        k,
        v,
        "(intermediate value)(intermediate value).set is not a function",
    )
}
fn put(g: &mut ModelGraph, target: &Atom, field: &str, a: Atom) -> Result<(), PublicError> {
    if matches!(a, Atom::Undefined) {
        Ok(())
    } else if string_kind(&a) {
        set(g, target, field, a)
    } else {
        named(g, target, &a)
    }
}
fn label(g: &mut ModelGraph, target: &Atom, a: Atom) -> Result<(), PublicError> {
    set(g, target, "labelAttr", Atom::Undefined)?;
    if matches!(a, Atom::Ref(_) | Atom::Null) {
        let empty = key(g, "")?;
        set(g, target, "label", empty)?;
        set(g, target, "labelAttr", a.clone())?;
        named(g, target, &a)
    } else {
        let value = if nullish(&a) { key(g, "")? } else { a };
        set(g, target, "label", value)
    }
}
fn text(g: &mut ModelGraph, target: &Atom, field: &str, a: Atom) -> Result<(), PublicError> {
    if matches!(a, Atom::Undefined) {
        if nullish(&read(g, target, field)?) {
            let empty = key(g, "")?;
            set(g, target, field, empty)?
        }
        Ok(())
    } else {
        put(g, target, field, a)
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
    let id = if let Atom::Ref(id) = r {
        id
    } else {
        read(g, &r, name)?;
        return Err(type_error(format!("this.{name} is not a function")));
    };
    let resolved = match property(g, id, name) {
        Some(Atom::BuiltinFunction(id)) => MethodId(*id),
        Some(Atom::HostReference(_)) => {
            return Err(invalid("C4 replaced host methods are not implemented").into())
        }
        Some(_) => return Err(type_error(format!("this.{name} is not a function"))),
        None => {
            if matches!(
                g.nodes[id as usize],
                Node::Instance {
                    prototype: PROTO,
                    ..
                }
            ) {
                method
            } else {
                return Err(type_error(format!("this.{name} is not a function")));
            }
        }
    };
    invoke_inner(
        g,
        MethodCall {
            method: resolved,
            receiver: r,
            arguments,
        },
        depth + 1,
    )
}
fn element(
    g: &mut ModelGraph,
    list: Atom,
    index: Atom,
    alias: Atom,
    title: Atom,
) -> Result<Atom, PublicError> {
    let mut el = if string_kind(&alias) {
        map_get(g, &index, &alias, "n.get is not a function")?
    } else {
        Atom::Undefined
    };
    if matches!(el, Atom::Undefined) {
        let empty = key(g, "")?;
        el = new_object(
            g,
            &[
                ("alias", empty.clone()),
                ("label", empty),
                ("parent", Atom::Undefined),
            ],
        )?;
        push(g, list, el.clone(), "e.push is not a function")?;
        if string_kind(&alias) {
            set(g, &el, "alias", alias.clone())?;
            map_set(g, &index, alias, el.clone(), "n.set is not a function")?
        } else {
            set(g, &el, "aliasAttr", alias.clone())?;
            named(g, &el, &alias)?
        }
    }
    label(g, &el, title)?;
    Ok(el)
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
        0 => set(g, &r, "c4Type", first)?,
        1 => set(g, &r, "title", first)?,
        2 => {
            let Atom::String(id) = first else {
                read(g, &first, "replace")?;
                return Err(type_error("t.replace is not a function"));
            };
            let raw = &g.strings[id as usize].0;
            let mut units = Vec::with_capacity(raw.len());
            let mut after = false;
            for &u in raw {
                if after && char::from_u32(u as u32).is_some_and(space) {
                    continue;
                }
                units.push(u);
                after = u == 10
            }
            let value = Atom::String(intern(g, JsString(units))?);
            set(g, &r, "accDescr", value)?
        }
        3 => {
            return element(
                g,
                first,
                second,
                a.get(2).cloned().unwrap_or(Atom::Undefined),
                a.get(3).cloned().unwrap_or(Atom::Undefined),
            )
        }
        4 | 5 => {
            let list = read(g, &r, "shapes")?;
            let index = read(g, &r, "shapeIndex")?;
            let alias = attr(g, &second, 0)?;
            let title = attr(g, &second, 1)?;
            let el = internal(
                g,
                r.clone(),
                "element",
                vec![list, index, alias, title],
                depth,
            )?;
            let at = if c.method.0 == 5 {
                let a = attr(g, &second, 2)?;
                text(g, &el, "techn", a)?;
                let a = attr(g, &second, 3)?;
                text(g, &el, "descr", a)?;
                4
            } else {
                let a = attr(g, &second, 2)?;
                text(g, &el, "descr", a)?;
                3
            };
            internal(
                g,
                r,
                "finishShape",
                vec![el, first, second, Atom::Number(at as f64)],
                depth,
            )?;
        }
        6 => {
            internal(g, r, "addContainer", vec![first, second], depth)?;
        }
        7 => {
            let shape = first;
            let kind = second;
            let attrs = a.get(2).cloned().unwrap_or(Atom::Undefined);
            let at = a.get(3).cloned().unwrap_or(Atom::Undefined);
            let Atom::Number(n) = at else {
                return Err(
                    invalid("C4 nonnumeric finishShape indexing is not implemented").into(),
                );
            };
            if !n.is_finite() || n < 0. || n > 1_000_000. || n.fract() != 0. {
                return Err(invalid("C4 arbitrary finishShape indexing is not implemented").into());
            }
            for (i, k) in ["sprite", "tags", "link"].iter().enumerate() {
                let v = attr(g, &attrs, n as usize + i)?;
                put(g, &shape, k, v)?
            }
            set(g, &shape, "kind", kind)?;
            let current = read(g, &r, "current")?;
            set(g, &shape, "parent", current)?;
        }
        8 => {
            let list = read(g, &r, "boundaries")?;
            let index = read(g, &r, "boundaryIndex")?;
            let alias = attr(g, &first, 0)?;
            let title = attr(g, &first, 1)?;
            let boundary = internal(
                g,
                r.clone(),
                "element",
                vec![list, index, alias, title],
                depth,
            )?;
            let typ = attr(g, &first, 2)?;
            if matches!(typ, Atom::Undefined) {
                set(g, &boundary, "type", second)?
            } else {
                put(g, &boundary, "type", typ)?
            }
            return Ok(boundary);
        }
        9 => {
            let current = read(g, &r, "current")?;
            set(g, &first, "parent", current)?;
            let stack = read(g, &r, "stack")?;
            let current = read(g, &r, "current")?;
            push(g, stack, current, "this.stack.push is not a function")?;
            set(g, &r, "current", first)?
        }
        10 => {
            let fallback = if matches!(second, Atom::Undefined) {
                key(g, "system")?
            } else {
                second
            };
            let boundary = internal(
                g,
                r.clone(),
                "boundary",
                vec![first.clone(), fallback],
                depth,
            )?;
            for (i, k) in [(3, "tags"), (4, "link")] {
                let v = attr(g, &first, i)?;
                put(g, &boundary, k, v)?
            }
            internal(g, r, "open", vec![boundary], depth)?;
        }
        11 => {
            let fallback = key(g, "container")?;
            internal(
                g,
                r,
                "addPersonOrSystemBoundary",
                vec![first, fallback],
                depth,
            )?;
        }
        12 => {
            let fallback = key(g, "node")?;
            let boundary = internal(
                g,
                r.clone(),
                "boundary",
                vec![second.clone(), fallback],
                depth,
            )?;
            let v = attr(g, &second, 3)?;
            text(g, &boundary, "descr", v)?;
            for (i, k) in [(4, "sprite"), (5, "tags"), (6, "link")] {
                let v = attr(g, &second, i)?;
                put(g, &boundary, k, v)?
            }
            set(g, &boundary, "nodeType", first)?;
            internal(g, r, "open", vec![boundary], depth)?;
        }
        13 => {
            let stack = read(g, &r, "stack")?;
            let id = array_method(g, &stack, "pop", "this.stack.pop is not a function")?;
            let Node::Array { length, .. } = g.nodes[id as usize] else {
                unreachable!()
            };
            let readonly = g.nodes[id as usize]
                .properties()
                .iter()
                .any(|p| g.strings[p.key as usize].is("length") && p.flags & 2 == 0);
            let mut current = if length == 0 {
                if readonly {
                    return Err(type_error(
                        "Cannot assign to read only property 'length' of object '[object Array]'",
                    ));
                }
                Atom::Undefined
            } else {
                let v = read(g, &stack, &(length - 1).to_string())?;
                let key = (length - 1).to_string();
                let i = g.nodes[id as usize]
                    .properties()
                    .iter()
                    .position(|p| g.strings[p.key as usize].is(&key));
                if let Some(i) = i {
                    if g.nodes[id as usize].properties()[i].flags & 4 == 0 {
                        return Err(type_error(format!(
                            "Cannot delete property '{key}' of [object Array]"
                        )));
                    }
                    g.nodes[id as usize].properties_mut().remove(i);
                }
                if readonly {
                    return Err(type_error(
                        "Cannot assign to read only property 'length' of object '[object Array]'",
                    ));
                }
                if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
                    *length -= 1
                }
                v
            };
            if nullish(&current) {
                let boundaries = read(g, &r, "boundaries")?;
                current = attr(g, &boundaries, 0)?
            }
            set(g, &r, "current", current)?;
        }
        14 => {
            let from = attr(g, &second, 0)?;
            let to = attr(g, &second, 1)?;
            if matches!(from, Atom::Undefined)
                || matches!(to, Atom::Undefined)
                || matches!(attr(g, &second, 2)?, Atom::Undefined)
            {
                return Ok(Atom::Undefined);
            }
            let known = string_kind(&from) && string_kind(&to);
            let index = read(g, &r, "relIndex")?;
            let mut targets = Atom::Undefined;
            let mut rel = if known {
                targets = map_get(g, &index, &from, "this.relIndex.get is not a function")?;
                if nullish(&targets) {
                    Atom::Undefined
                } else {
                    map_get(
                        g,
                        &targets,
                        &to,
                        "this.relIndex.get(...)?.get is not a function",
                    )?
                }
            } else {
                Atom::Undefined
            };
            if matches!(rel, Atom::Undefined) {
                let empty = key(g, "")?;
                rel = new_object(
                    g,
                    &[
                        ("type", first.clone()),
                        ("from", empty.clone()),
                        ("to", empty.clone()),
                        ("label", empty),
                    ],
                )?;
                let rels = read(g, &r, "rels")?;
                push(g, rels, rel.clone(), "this.rels.push is not a function")?;
                if known {
                    if matches!(targets, Atom::Undefined) {
                        targets = new_map(g)?;
                        map_set(
                            g,
                            &index,
                            from.clone(),
                            targets.clone(),
                            "this.relIndex.set is not a function",
                        )?
                    }
                    map_set(
                        g,
                        &targets,
                        to.clone(),
                        rel.clone(),
                        "n.set is not a function",
                    )?
                }
                if string_kind(&from) {
                    set(g, &rel, "from", from)?
                } else {
                    set(g, &rel, "fromAttr", from)?
                }
                if string_kind(&to) {
                    set(g, &rel, "to", to)?
                } else {
                    set(g, &rel, "toAttr", to)?
                }
            }
            set(g, &rel, "type", first)?;
            let l = attr(g, &second, 2)?;
            label(g, &rel, l)?;
            let a = attr(g, &second, 3)?;
            text(g, &rel, "techn", a)?;
            let a = attr(g, &second, 4)?;
            text(g, &rel, "descr", a)?;
            for (i, k) in [(5, "sprite"), (6, "tags"), (7, "link")] {
                let a = attr(g, &second, i)?;
                put(g, &rel, k, a)?
            }
        }
        15 => {
            let shapes = read(g, &r, "shapeIndex")?;
            let result = map_get(g, &shapes, &first, "this.shapeIndex.get is not a function")?;
            return if nullish(&result) {
                let boundaries = read(g, &r, "boundaryIndex")?;
                map_get(
                    g,
                    &boundaries,
                    &first,
                    "this.boundaryIndex.get is not a function",
                )
            } else {
                Ok(result)
            };
        }
        16 => {
            let alias = attr(g, &second, 0)?;
            let el = if string_kind(&alias) {
                internal(g, r, "getC4Shape", vec![alias], depth)?
            } else {
                Atom::Undefined
            };
            if matches!(el, Atom::Undefined) {
                return Ok(Atom::Undefined);
            }
            for (i, k) in [
                "bgColor",
                "fontColor",
                "borderColor",
                "shadowing",
                "shape",
                "sprite",
                "techn",
                "legendText",
                "legendSprite",
            ]
            .iter()
            .enumerate()
            {
                let v = attr(g, &second, i + 1)?;
                put(g, &el, k, v)?
            }
        }
        17 => {
            let from = attr(g, &second, 0)?;
            let to = attr(g, &second, 1)?;
            let rel = if string_kind(&from) && string_kind(&to) {
                let index = read(g, &r, "relIndex")?;
                let targets = map_get(g, &index, &from, "this.relIndex.get is not a function")?;
                if nullish(&targets) {
                    Atom::Undefined
                } else {
                    map_get(
                        g,
                        &targets,
                        &to,
                        "this.relIndex.get(...)?.get is not a function",
                    )?
                }
            } else {
                Atom::Undefined
            };
            if matches!(rel, Atom::Undefined) {
                return Ok(Atom::Undefined);
            }
            for (i, k) in [(2, "textColor"), (3, "lineColor")] {
                let v = attr(g, &second, i)?;
                put(g, &rel, k, v)?
            }
            for (i, k) in [(4, "offsetX"), (5, "offsetY")] {
                let v = attr(g, &second, i)?;
                if string_kind(&v) {
                    let n = parse_integer(g, &v)?;
                    set(g, &rel, k, Atom::Number(n))?
                } else if !matches!(v, Atom::Undefined) {
                    named(g, &rel, &v)?
                }
            }
        }
        18 => {
            let mut counts = vec![];
            for i in 0..2 {
                let a = attr(g, &second, i)?;
                counts.push(if matches!(a, Atom::Undefined) {
                    f64::NAN
                } else if string_kind(&a) {
                    parse_integer(g, &a)?
                } else {
                    let v = read(g, &a, "value")?;
                    parse_integer(g, &v)?
                })
            }
            for (field, n) in ["shapeInRow", "boundaryInRow"].into_iter().zip(counts) {
                if n >= 1. {
                    set(g, &r, field, Atom::Number(n))?
                }
            }
        }
        _ => return Err(invalid("Unknown C4 method").into()),
    }
    Ok(Atom::Undefined)
}
struct Reader {
    values: usize,
    scans: usize,
    text: usize,
}
impl Reader {
    fn record(&mut self, g: &ModelGraph, a: &Atom, reads: usize) -> Result<u32, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("C4 active record requires an object"));
        };
        self.scans = self.scans.saturating_add(
            g.nodes[*id as usize]
                .properties()
                .len()
                .saturating_mul(reads),
        );
        if self.scans > 32_000_000 {
            return Err(limit());
        }
        Ok(*id)
    }
    fn array(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
        allow_set: bool,
    ) -> Result<Vec<Atom>, PeleError> {
        let Some(Atom::Ref(id)) = a else {
            return Err(invalid("C4 active collection requires an object"));
        };
        self.scans = self
            .scans
            .saturating_add(g.nodes[*id as usize].properties().len());
        let (length, properties) = match &g.nodes[*id as usize] {
            Node::Array { length, properties } => (*length, properties),
            Node::Set { values, .. } if allow_set => {
                self.values = self.values.saturating_add(values.len());
                if self.values > 1_000_000 {
                    return Err(limit());
                }
                return Ok(values.clone());
            }
            _ => return Err(invalid("C4 active collection requires an Array")),
        };
        self.values = self.values.saturating_add(length as usize);
        if self.values > 1_000_000 || self.scans > 32_000_000 {
            return Err(limit());
        }
        let mut indexed = HashMap::new();
        for p in properties {
            let s = &g.strings[p.key as usize];
            if s.0.len() > 10 {
                continue;
            }
            if let Some(k) = s.well_formed() {
                if let Ok(i) = k.parse::<u32>() {
                    if i < length && i.to_string() == k {
                        indexed.insert(i, p.value.clone());
                    }
                }
            }
        }
        if indexed.len() != length as usize {
            return Err(invalid("C4 sparse active collections are not implemented"));
        }
        Ok((0..length).map(|i| indexed.remove(&i).unwrap()).collect())
    }
    fn string(
        &mut self,
        g: &ModelGraph,
        a: Option<&Atom>,
        undefined_empty: bool,
    ) -> Result<String, PeleError> {
        match a {
            None | Some(Atom::Undefined) if undefined_empty => Ok(String::new()),
            Some(Atom::String(i)) => {
                let s = &g.strings[*i as usize];
                self.text = self.text.saturating_add(s.0.len());
                if self.text > 16_000_000 {
                    return Err(limit());
                }
                s.well_formed()
                    .ok_or_else(|| invalid("C4 rendering of lone UTF-16 text is not implemented"))
            }
            _ => Err(invalid("C4 active text coercion is not implemented")),
        }
    }
    fn optional(
        &mut self,
        g: &ModelGraph,
        id: u32,
        key: &str,
        skip_falsy: bool,
        coerce: bool,
    ) -> Result<Option<String>, PeleError> {
        let a = property(g, id, key);
        if matches!(a, None | Some(Atom::Undefined)) || skip_falsy && !truthy(g, a.unwrap()) {
            return Ok(None);
        }
        if coerce {
            let s = primitive(g, a.unwrap())
                .map_err(|_| invalid("C4 active object text coercion is not implemented"))?;
            self.text = self.text.saturating_add(s.encode_utf16().count());
            if self.text > 16_000_000 {
                return Err(limit());
            }
            Ok(Some(s))
        } else {
            self.string(g, a, false).map(Some)
        }
    }
    fn element(&mut self, g: &ModelGraph, id: u32, shape: bool) -> Result<C4Element, PeleError> {
        let mut el = C4Element::new();
        el.alias = self.string(g, property(g, id, "alias"), false)?;
        el.label = self.string(g, property(g, id, "label"), true)?;
        el.descr = self.optional(g, id, "descr", false, false)?;
        el.techn = self.optional(g, id, "techn", true, true)?;
        for (k, target) in [
            ("link", &mut el.link),
            ("bgColor", &mut el.bg_color),
            ("fontColor", &mut el.font_color),
            ("borderColor", &mut el.border_color),
        ] {
            *target = self.optional(g, id, k, true, k != "link")?
        }
        if shape {
            let shape_raw = property(g, id, "shape");
            el.shape = if matches!(shape_raw, None | Some(Atom::Null | Atom::Undefined)) {
                None
            } else {
                Some(self.string(g, shape_raw, false)?)
            };
            let form = |s: &str| {
                matches!(
                    s.to_lowercase().as_str(),
                    "person"
                        | "box"
                        | "rounded"
                        | "component"
                        | "cylinder"
                        | "database"
                        | "db"
                        | "queue"
                        | "pipe"
                )
            };
            let explicit = el.shape.as_deref().is_some_and(form);
            if !explicit {
                let sprite = property(g, id, "sprite");
                el.sprite = if matches!(sprite, None | Some(Atom::Null | Atom::Undefined)) {
                    None
                } else {
                    Some(self.string(g, sprite, false)?)
                };
            }
            if !explicit && !el.sprite.as_deref().is_some_and(form) {
                el.tags = self.optional(g, id, "tags", true, false)?;
            }
        } else {
            el.element_type = self.optional(g, id, "type", true, true)?;
        }
        Ok(el)
    }
}
fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(id) = a {
        Ok(*id)
    } else {
        Err(invalid("C4 active record requires an object"))
    }
}
fn imported(g: &ModelGraph) -> Result<C4Model, PeleError> {
    let mut b = Reader {
        values: 0,
        scans: 0,
        text: 0,
    };
    let r = b.record(g, &g.root, 15)?;
    let boundaries = b.array(g, property(g, r, "boundaries"), false)?;
    if boundaries.is_empty() {
        return Err(invalid("C4 rendering requires the top-level boundary"));
    }
    let shapes = b.array(g, property(g, r, "shapes"), true)?;
    let rels = b.array(g, property(g, r, "rels"), false)?;
    let rel_array = reference(property(g, r, "rels").unwrap())?;
    if ["map", "forEach"]
        .iter()
        .any(|name| property(g, rel_array, name).is_some())
    {
        return Err(invalid(
            "C4 replaced relation Array render callbacks are not implemented",
        ));
    }
    let boundary_refs = boundaries
        .iter()
        .map(reference)
        .collect::<Result<Vec<_>, _>>()?;
    let mut boundary_index = HashMap::new();
    let mut boundary_identity = vec![];
    for (i, &id) in boundary_refs.iter().enumerate() {
        boundary_identity.push(*boundary_index.entry(id).or_insert(i));
    }
    let dummy = boundary_refs.len();
    let mut m = super::model::Db::new(None).model;
    m.boundaries.clear();
    m.title = b.optional(g, r, "title", false, false)?;
    m.acc_title = b.optional(g, r, "accTitle", true, false)?;
    m.acc_descr = b.optional(g, r, "accDescr", true, false)?;
    m.c4_type = if matches!(property(g,r,"c4Type"),Some(Atom::String(i))if g.strings[*i as usize].is("C4Dynamic"))
    {
        Some("C4Dynamic".into())
    } else {
        None
    };
    m.shape_in_row = numeric(g, property(g, r, "shapeInRow").unwrap_or(&Atom::Undefined))
        .map_err(|_| invalid("C4 object layout-count coercion is not implemented"))?;
    m.boundary_in_row = numeric(
        g,
        property(g, r, "boundaryInRow").unwrap_or(&Atom::Undefined),
    )
    .map_err(|_| invalid("C4 object layout-count coercion is not implemented"))?;
    if m.shape_in_row.is_nan() || m.boundary_in_row.is_nan() {
        return Err(invalid(
            "C4 rendering of NaN layout counts is not implemented",
        ));
    }
    let mut foreign_parent = false;
    for a in &boundaries {
        let id = b.record(g, a, 18)?;
        let parent = property(g, id, "parent").unwrap_or(&Atom::Undefined);
        let parent = if truthy(g, parent) {
            match parent {
                Atom::Ref(id) if boundary_index.contains_key(id) => Some(boundary_index[id]),
                _ => {
                    foreign_parent = true;
                    Some(dummy)
                }
            }
        } else {
            None
        };
        let el = if parent.is_some() {
            b.element(g, id, false)?
        } else {
            C4Element::new()
        };
        let mut node = C4Boundary {
            element: el,
            node_type: if truthy(g, property(g, id, "nodeType").unwrap_or(&Atom::Undefined)) {
                Some("node".into())
            } else {
                None
            },
        };
        node.element.parent = parent;
        m.boundaries.push(node);
    }
    if foreign_parent {
        m.boundaries.push(C4Boundary {
            element: C4Element::new(),
            node_type: None,
        });
        boundary_identity.push(dummy)
    }
    // Only cycles reachable from the root expand the upstream breadth-first walk.
    // Unreachable self-parented boundaries retain their real blank rendering.
    let mut children = vec![vec![]; m.boundaries.len()];
    for (i, boundary) in m.boundaries.iter().enumerate() {
        if boundary_identity[i] == i {
            if let Some(p) = boundary.parent {
                children[p].push(i)
            }
        }
    }
    let mut order = vec![0];
    let mut seen = HashSet::new();
    let mut at = 0;
    while at < order.len() {
        let i = order[at];
        at += 1;
        if !seen.insert(i) {
            return Err(invalid(
                "C4 cyclic reachable boundary parents exceed native work budget",
            ));
        }
        order.extend_from_slice(&children[i]);
        if seen.len() > 1_000_000 {
            return Err(limit());
        }
    }
    let mut shape_refs = vec![];
    for a in &shapes {
        let id = match a {
            Atom::Ref(id) => Some(*id),
            Atom::Null | Atom::Undefined => {
                return Err(invalid(
                    "C4 null or absent active shapes are not implemented",
                ))
            }
            _ => None,
        };
        let parent = id.and_then(|id| property(g, id, "parent")).and_then(|a| {
            if let Atom::Ref(id) = a {
                boundary_index.get(id).copied()
            } else {
                None
            }
        });
        let (el, kind) = if let (Some(id), Some(_)) = (id, parent) {
            b.record(g, a, 18)?;
            (
                b.element(g, id, true)?,
                b.string(g, property(g, id, "kind"), false)?,
            )
        } else {
            (C4Element::new(), String::new())
        };
        let mut shape = C4Shape { element: el, kind };
        shape.element.parent = parent;
        m.shapes.push(shape);
        shape_refs.push(id);
    }
    m.render_boundary_identity = Some(boundary_identity);
    // C4Element identity keys the upstream lookup Map, but obstacles compare
    // actual layout Boxes. Preserve distinct Boxes for repeated shape rows,
    // then point every record lookup at the last Box set in group BFS order.
    let mut identities = HashMap::new();
    let mut nodes = vec![vec![]; m.boundaries.len()];
    for (i, shape) in m.shapes.iter().enumerate() {
        if let Some(p) = shape.parent {
            nodes[p].push(i)
        }
    }
    for group in order {
        for &i in &nodes[group] {
            if let Some(id) = shape_refs[i] {
                identities.insert(id, ElementRef::Shape(i));
            }
        }
        if group != 0 {
            identities.insert(boundary_refs[group], ElementRef::Boundary(group));
        }
    }
    let dummy_named = || C4Named {
        key: String::new(),
        value: String::new(),
    };
    let mut rel_keys = vec![];
    for a in rels {
        let id = b.record(g, &a, 15)?;
        let mut rel = C4Rel::new("");
        rel.rel_type = if let Some(Atom::String(i)) = property(g, id, "type") {
            if g.strings[*i as usize].is("birel") {
                "birel".into()
            } else if g.strings[*i as usize].is("rel_b") {
                "rel_b".into()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        rel.from = b.string(g, property(g, id, "from"), false)?;
        rel.to = b.string(g, property(g, id, "to"), false)?;
        // The relation label is always concatenated with a string before layout.
        rel.label = primitive(g, property(g, id, "label").unwrap_or(&Atom::Undefined))
            .map_err(|_| invalid("C4 relation label object coercion is not implemented"))?;
        rel.from_attr =
            truthy(g, property(g, id, "fromAttr").unwrap_or(&Atom::Undefined)).then(dummy_named);
        rel.to_attr =
            truthy(g, property(g, id, "toAttr").unwrap_or(&Atom::Undefined)).then(dummy_named);
        rel.techn = b.optional(g, id, "techn", true, true)?;
        rel.descr = b.optional(g, id, "descr", true, false)?;
        rel.link = b.optional(g, id, "link", true, false)?;
        rel.text_color = b.optional(g, id, "textColor", true, true)?;
        rel.line_color = b.optional(g, id, "lineColor", true, true)?;
        for (k, target) in [
            ("offsetX", &mut rel.offset_x),
            ("offsetY", &mut rel.offset_y),
        ] {
            let raw = property(g, id, k).unwrap_or(&Atom::Undefined);
            if truthy(g, raw) {
                *target = Some(
                    numeric(g, raw)
                        .map_err(|_| invalid("C4 object offset coercion is not implemented"))?,
                );
            }
        }
        rel_keys.push((
            property(g, id, "from").unwrap().clone(),
            property(g, id, "to").unwrap().clone(),
        ));
        m.rels.push(rel);
    }
    let mut lookup = HashMap::new();
    if !m.rels.is_empty() {
        let shape_map = property(g, r, "shapeIndex").unwrap_or(&Atom::Undefined);
        'relations: for (rel, (from, to)) in m.rels.iter().zip(&rel_keys) {
            for (alias, key, blocked) in [
                (&rel.from, from, rel.from_attr.is_some()),
                (&rel.to, to, rel.to_attr.is_some()),
            ] {
                if blocked {
                    break 'relations;
                }
                if lookup.contains_key(alias) {
                    continue;
                }
                let default_get = match property(g, r, "getC4Shape") {
                    Some(Atom::BuiltinFunction(15)) => true,
                    None => matches!(
                        g.nodes[r as usize],
                        Node::Instance {
                            prototype: PROTO,
                            ..
                        }
                    ),
                    _ => false,
                };
                if !default_get {
                    if matches!(g.nodes[r as usize], Node::Object { .. })
                        && matches!(property(g, r, "getC4Shape"), None | Some(Atom::Undefined))
                    {
                        // The plain-model helper read occurs after layout's host
                        // measurements, so retain data and defer this failure.
                        break 'relations;
                    }
                    return Err(invalid(
                        "C4 missing or replaced getC4Shape render callbacks are not implemented",
                    ));
                }
                let charge = |b: &mut Reader, map: &Atom| -> Result<(), PeleError> {
                    if let Atom::Ref(id) = map {
                        b.scans = b
                            .scans
                            .saturating_add(g.nodes[*id as usize].properties().len())
                            .saturating_add(
                                if let Node::Map { entries, .. } = &g.nodes[*id as usize] {
                                    entries.len()
                                } else {
                                    0
                                },
                            );
                    }
                    if b.scans > 32_000_000 {
                        return Err(limit());
                    }
                    Ok(())
                };
                charge(&mut b, shape_map)?;
                let mut value = map_get(g, shape_map, key, "this.shapeIndex.get is not a function")
                    .map_err(|_| invalid("C4 active shape-index callbacks are not implemented"))?;
                if nullish(&value) {
                    let map = property(g, r, "boundaryIndex").unwrap_or(&Atom::Undefined);
                    charge(&mut b, map)?;
                    value = map_get(g, map, key, "this.boundaryIndex.get is not a function")
                        .map_err(|_| {
                            invalid("C4 active boundary-index callbacks are not implemented")
                        })?;
                }
                if let Atom::Ref(id) = value {
                    if let Some(identity) = identities.get(&id) {
                        lookup.insert(alias.clone(), *identity);
                        continue;
                    }
                }
                // The native renderer throws on this first unresolved relation
                // before visiting later lookups. Preserve that order and bound
                // invalid-corpus work instead of scanning irrelevant Maps.
                break 'relations;
            }
        }
    }
    m.render_lookup = Some(lookup);
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
fn named_out(out: &mut GraphWriter, n: Option<&C4Named>) -> Atom {
    n.map_or(Atom::Undefined, |n| {
        let key = out.string(n.key.as_str());
        let value = out.string(n.value.as_str());
        let properties = vec![
            out.property("key", Atom::String(key)),
            out.property("value", Atom::String(value)),
        ];
        Atom::Ref(out.node(Node::Object { properties }))
    })
}
fn extra_out(out: &mut GraphWriter, extra: Option<&Vec<(String, String)>>) -> Atom {
    extra.map_or(Atom::Undefined, |entries| {
        let entries = entries
            .iter()
            .map(|(k, v)| {
                (
                    Atom::String(out.string(k.as_str())),
                    Atom::String(out.string(v.as_str())),
                )
            })
            .collect();
        Atom::Ref(out.node(Node::Map {
            entries,
            properties: vec![],
        }))
    })
}
fn element_out(
    out: &mut GraphWriter,
    el: &C4Element,
    boundaries: &[u32],
    kind: Option<&str>,
    node_type: Option<&str>,
) -> Result<Vec<Property>, PeleError> {
    let parent =
        match el.parent {
            Some(i) => Atom::Ref(*boundaries.get(i).ok_or_else(|| {
                invalid("C4 typed factory contains an invalid boundary reference")
            })?),
            None => Atom::Undefined,
        };
    let mut fields = vec![
        ("alias", Atom::String(out.string(el.alias.as_str()))),
        ("label", Atom::String(out.string(el.label.as_str()))),
        ("parent", parent),
        ("aliasAttr", named_out(out, el.alias_attr.as_ref())),
        ("labelAttr", named_out(out, el.label_attr.as_ref())),
    ];
    for (k, v) in [
        ("type", el.element_type.as_deref()),
        ("descr", el.descr.as_deref()),
        ("techn", el.techn.as_deref()),
        ("sprite", el.sprite.as_deref()),
        ("tags", el.tags.as_deref()),
        ("link", el.link.as_deref()),
        ("bgColor", el.bg_color.as_deref()),
        ("fontColor", el.font_color.as_deref()),
        ("borderColor", el.border_color.as_deref()),
        ("shadowing", el.shadowing.as_deref()),
        ("shape", el.shape.as_deref()),
        ("legendText", el.legend_text.as_deref()),
        ("legendSprite", el.legend_sprite.as_deref()),
    ] {
        fields.push((k, option_out(out, v)))
    }
    fields.push(("extra", extra_out(out, el.extra.as_ref())));
    if let Some(kind) = kind {
        fields.push(("kind", Atom::String(out.string(kind))))
    }
    if let Some(typ) = node_type {
        fields.push(("nodeType", Atom::String(out.string(typ))))
    }
    let mut properties = vec![];
    let mut used = HashSet::new();
    for k in &el.own_keys {
        if let Some((_, v)) = fields.iter().find(|(name, _)| *name == k) {
            properties.push(out.property(k, v.clone()));
            used.insert(k.as_str());
        }
    }
    for (k, v) in fields {
        if !used.contains(k) && !matches!(v, Atom::Undefined) {
            properties.push(out.property(k, v));
        }
    }
    Ok(properties)
}
fn rel_out(out: &mut GraphWriter, rel: &C4Rel) -> Vec<Property> {
    let mut fields = vec![
        ("type", Atom::String(out.string(rel.rel_type.as_str()))),
        ("from", Atom::String(out.string(rel.from.as_str()))),
        ("to", Atom::String(out.string(rel.to.as_str()))),
        ("label", Atom::String(out.string(rel.label.as_str()))),
        ("fromAttr", named_out(out, rel.from_attr.as_ref())),
        ("toAttr", named_out(out, rel.to_attr.as_ref())),
        ("labelAttr", named_out(out, rel.label_attr.as_ref())),
    ];
    for (k, v) in [
        ("techn", rel.techn.as_deref()),
        ("descr", rel.descr.as_deref()),
        ("sprite", rel.sprite.as_deref()),
        ("tags", rel.tags.as_deref()),
        ("link", rel.link.as_deref()),
        ("textColor", rel.text_color.as_deref()),
        ("lineColor", rel.line_color.as_deref()),
    ] {
        fields.push((k, option_out(out, v)))
    }
    for (k, v) in [("offsetX", rel.offset_x), ("offsetY", rel.offset_y)] {
        fields.push((k, v.map_or(Atom::Undefined, Atom::Number)))
    }
    fields.push(("extra", extra_out(out, rel.extra.as_ref())));
    let mut properties = vec![];
    let mut used = HashSet::new();
    for k in &rel.own_keys {
        if let Some((_, v)) = fields.iter().find(|(name, _)| *name == k) {
            properties.push(out.property(k, v.clone()));
            used.insert(k.as_str());
        }
    }
    for (k, v) in fields {
        if !used.contains(k) && !matches!(v, Atom::Undefined) {
            properties.push(out.property(k, v));
        }
    }
    properties
}
fn operation_budget(g: &ModelGraph, c: &MethodCall) -> Result<(), PublicError> {
    if matches!(c.method.0, 0 | 1 | 2) {
        return Ok(());
    }
    let count = g.nodes.iter().fold(g.strings.len(), |n, node| {
        n.saturating_add(node.properties().len())
            .saturating_add(match node {
                Node::Map { entries, .. } => entries.len().saturating_mul(2),
                Node::Set { values, .. } => values.len(),
                _ => 0,
            })
    });
    if count > 2_000_000 {
        return Err(limit().into());
    }
    Ok(())
}
fn construct_default() -> Result<C4PublicState, PeleError> {
    // Implicit Pele constructor: field defaults only, no source or argument reads.
    C4.create_public(&super::model::Db::new(None).finish())
}

impl PublicDiagramModule for C4 {
    type PublicState = C4PublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<crate::compat::public::PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTO).then(|| {
            crate::compat::public::PublicConstructorFactory::ignoring_arguments::<Self>(
                PROTO,
                construct_default,
            )
        })
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(C4PublicState { graph, model })
    }
    fn create_public(&self, m: &C4Model) -> Result<C4PublicState, PeleError> {
        let mut out = GraphWriter::default();
        let shapes = m
            .shapes
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        let boundaries = m
            .boundaries
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        let rels = m
            .rels
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        let mut filled = vec![];
        for (i, shape) in m.shapes.iter().enumerate() {
            filled.push((
                shapes[i],
                element_out(
                    &mut out,
                    &shape.element,
                    &boundaries,
                    Some(&shape.kind),
                    None,
                )?,
            ));
        }
        for (i, boundary) in m.boundaries.iter().enumerate() {
            filled.push((
                boundaries[i],
                element_out(
                    &mut out,
                    &boundary.element,
                    &boundaries,
                    None,
                    boundary.node_type.as_deref(),
                )?,
            ));
        }
        for (i, rel) in m.rels.iter().enumerate() {
            filled.push((rels[i], rel_out(&mut out, rel)));
        }
        let bookkeeping = m
            .bookkeeping
            .as_ref()
            .ok_or_else(|| invalid("C4 typed factory requires retained parser bookkeeping"))?;
        let shape_entries = bookkeeping
            .shape_index
            .iter()
            .map(|(k, i)| {
                Ok((
                    Atom::String(out.string(k.as_str())),
                    Atom::Ref(*shapes.get(*i).ok_or_else(|| {
                        invalid(
                            "C4 typed factory structural edits invalidate private shape references",
                        )
                    })?),
                ))
            })
            .collect::<Result<Vec<_>, PeleError>>()?;
        let shape_index = out.node(Node::Map {
            entries: shape_entries,
            properties: vec![],
        });
        let boundary_entries=bookkeeping.boundary_index.iter().map(|(k,i)|Ok((Atom::String(out.string(k.as_str())),Atom::Ref(*boundaries.get(*i).ok_or_else(||invalid("C4 typed factory structural edits invalidate private boundary references"))?)))).collect::<Result<Vec<_>,PeleError>>()?;
        let boundary_index = out.node(Node::Map {
            entries: boundary_entries,
            properties: vec![],
        });
        let mut outer: Vec<(Atom, Atom)> = vec![];
        let mut outer_index = HashMap::new();
        let mut targets: Vec<(u32, Vec<(Atom, Atom)>)> = vec![];
        for (from, to, i) in &bookkeeping.rel_index {
            let at = if let Some(&i) = outer_index.get(from) {
                i
            } else {
                let id = out.node(Node::Map {
                    entries: vec![],
                    properties: vec![],
                });
                outer.push((Atom::String(out.string(from.as_str())), Atom::Ref(id)));
                targets.push((id, vec![]));
                let i = targets.len() - 1;
                outer_index.insert(from, i);
                i
            };
            targets[at].1.push((Atom::String(out.string(to.as_str())),Atom::Ref(*rels.get(*i).ok_or_else(||invalid("C4 typed factory structural edits invalidate private relationship references"))?)));
        }
        let rel_index = out.node(Node::Map {
            entries: outer,
            properties: vec![],
        });
        let current = *boundaries
            .get(bookkeeping.current)
            .ok_or_else(|| invalid("C4 typed factory contains an invalid current boundary"))?;
        let stack =
            bookkeeping
                .stack
                .iter()
                .map(|i| {
                    boundaries.get(*i).copied().map(Atom::Ref).ok_or_else(|| {
                        invalid("C4 typed factory contains an invalid boundary stack")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
        let stack = array_out(&mut out, stack);
        let shape_array = array_out(&mut out, shapes.into_iter().map(Atom::Ref).collect());
        let boundary_array = array_out(&mut out, boundaries.into_iter().map(Atom::Ref).collect());
        let rel_array = array_out(&mut out, rels.into_iter().map(Atom::Ref).collect());
        let kind = out.string("c4");
        let c4_type = option_out(&mut out, m.c4_type.as_deref());
        let title = option_out(&mut out, m.title.as_deref());
        let acc_title = option_out(&mut out, m.acc_title.as_deref());
        let acc_descr = option_out(&mut out, m.acc_descr.as_deref());
        let properties = vec![
            out.property("type", Atom::String(kind)),
            out.property("c4Type", c4_type),
            out.property("title", title),
            out.property("accTitle", acc_title),
            out.property("accDescr", acc_descr),
            out.property("shapes", Atom::Ref(shape_array)),
            out.property("boundaries", Atom::Ref(boundary_array)),
            out.property("rels", Atom::Ref(rel_array)),
            out.property("shapeInRow", Atom::Number(m.shape_in_row)),
            out.property("boundaryInRow", Atom::Number(m.boundary_in_row)),
            out.property("shapeIndex", Atom::Ref(shape_index)),
            out.property("boundaryIndex", Atom::Ref(boundary_index)),
            out.property("relIndex", Atom::Ref(rel_index)),
            out.property("current", Atom::Ref(current)),
            out.property("stack", Atom::Ref(stack)),
        ];
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let mut graph = out.finish(Atom::Ref(root))?;
        for (id, properties) in filled {
            *graph.nodes[id as usize].properties_mut() = properties
        }
        for (id, entries) in targets {
            if let Node::Map {
                entries: target, ..
            } = &mut graph.nodes[id as usize]
            {
                *target = entries
            }
        }
        graph.validate_schema(&SCHEMA)?;
        let model = imported(&graph);
        Ok(C4PublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &C4PublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut C4PublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut C4PublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = operation_budget(&s.graph, &c).and_then(|_| invoke_inner(&mut s.graph, c, 0));
        s.model = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut C4PublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let model = s.model.as_ref().map_err(Clone::clone)?;
        if let Atom::Ref(root) = s.graph.root {
            if matches!(s.graph.nodes[root as usize], Node::Object { .. })
                && matches!(
                    property(&s.graph, root, "getC4Shape"),
                    None | Some(Atom::Undefined)
                )
                && model
                    .rels
                    .iter()
                    .any(|rel| rel.from_attr.is_none() || rel.to_attr.is_none())
            {
                // Preserve the legacy PeleError-only API's explicit callback
                // boundary. Raw operation rendering below retains the pinned
                // TypeError at its actual post-layout call site instead.
                return Err(invalid(
                    "C4 missing or replaced getC4Shape render callbacks are not implemented",
                ));
            }
        }
        super::render::render(model, c, o, r)
    }
    fn render_public_operation(
        &self,
        state: &mut Self::PublicState,
        config: &Object,
        options: &RenderSettings,
        host: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        if matches!(state.graph.root, Atom::Ref(id)
            if matches!(state.graph.nodes[id as usize], Node::Object { .. })
                && matches!(property(&state.graph, id, "getC4Shape"), None | Some(Atom::Undefined)))
        {
            return super::render::render_plain(
                state.model.as_ref().map_err(Clone::clone)?,
                config,
                options,
                host,
            );
        }
        self.render_public(state, config, options, host)
            .map_err(Into::into)
    }
}
