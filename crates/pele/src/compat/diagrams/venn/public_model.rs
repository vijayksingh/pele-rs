//! VennDb's actual fields, Set/current aliases and eight prototype methods.
//! Pele 458322a5 (MIT kepano). Runtime rendering stays in the existing Rust
//! circle renderer. Exotic coercion/iterators, active sparse or lone UTF16
//! text and JavaScript render error identity remain explicit gaps.
use super::{Venn, VennModel, VennStyle, VennSubset, VennText};
use crate::compat::{
    public::*, source::space, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::{HashMap, HashSet};
const PROTO: PrototypeId = PrototypeId(1);
const fn f(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        undefined,
        optional: false,
        nullable: false,
        flags: 7,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/venn@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "Venn",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                f("type", FieldKind::String, false),
                f("title", FieldKind::String, true),
                f("subsets", FieldKind::Array, false),
                f("textNodes", FieldKind::Array, false),
                f("styles", FieldKind::Array, false),
                f("known", FieldKind::Set, false),
                f("current", FieldKind::Array, true),
                f("indent", FieldKind::Bool, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "VennSubset",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("sets", FieldKind::Array, false),
                f("size", FieldKind::Number, false),
                f("label", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "VennText",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("sets", FieldKind::Array, false),
                f("id", FieldKind::String, false),
                f("label", FieldKind::String, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "VennStyle",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                f("targets", FieldKind::Array, false),
                f("styles", FieldKind::Map, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "VennDb",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "setDiagramTitle",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(0),
                },
            },
            MemberSchema {
                name: "addSubsetData",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(1),
                },
            },
            MemberSchema {
                name: "addTextData",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(2),
                },
            },
            MemberSchema {
                name: "addStyleData",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(3),
                },
            },
            MemberSchema {
                name: "validateUnionIdentifiers",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(4),
                },
            },
            MemberSchema {
                name: "getCurrentSets",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(5),
                },
            },
            MemberSchema {
                name: "getIndentMode",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(6),
                },
            },
            MemberSchema {
                name: "setIndentMode",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(7),
                },
            },
        ],
    }],
    methods: &[
        MethodSchema {
            id: MethodId(0),
            name: "setDiagramTitle",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(1),
            name: "addSubsetData",
            arity: 3,
        },
        MethodSchema {
            id: MethodId(2),
            name: "addTextData",
            arity: 3,
        },
        MethodSchema {
            id: MethodId(3),
            name: "addStyleData",
            arity: 2,
        },
        MethodSchema {
            id: MethodId(4),
            name: "validateUnionIdentifiers",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(5),
            name: "getCurrentSets",
            arity: 0,
        },
        MethodSchema {
            id: MethodId(6),
            name: "getIndentMode",
            arity: 0,
        },
        MethodSchema {
            id: MethodId(7),
            name: "setIndentMode",
            arity: 1,
        },
    ],
};
pub struct VennPublicState {
    graph: ModelGraph,
    model: Result<VennModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("venn".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Venn public operation exceeds native work budget");
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
            invalid("Venn runtime error text containing lone UTF-16 is not implemented").into()
        }),
        _ => Err(invalid("Venn arbitrary object coercion is not implemented").into()),
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
            Err(invalid("Venn String collection indexing is not implemented").into())
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
        _ => return Err(invalid("Venn borrowed host receiver is not implemented").into()),
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
        _ => Err(invalid("Venn object-to-number coercion is not implemented").into()),
    }
}
pub(super) fn js_number(raw: &str) -> f64 {
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
fn new_array(g: &mut ModelGraph, values: Vec<Atom>) -> Result<Atom, PublicError> {
    let a = node(
        g,
        Node::Array {
            length: 0,
            properties: vec![],
        },
    )?;
    for v in values {
        push_raw(g, a.clone(), v, "Venn native array push")?
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
                invalid("Venn replaced collection callbacks are not implemented").into()
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
// Array.map skips holes. Sort moves them behind all normalized strings.
fn elements(g: &ModelGraph, a: &Atom) -> Result<Vec<Option<Atom>>, PublicError> {
    let id = array_method(g, a, "map", "e.map is not a function")?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if *length > 1_000_000 || properties.len() > 1_000_000 {
        return Err(limit().into());
    }
    let mut values = vec![None; *length as usize];
    for p in properties {
        let s = &g.strings[p.key as usize];
        if s.0.len() > 10 {
            continue;
        }
        if let Some(k) = s.well_formed() {
            if let Ok(i) = k.parse::<u32>() {
                if i < *length && i.to_string() == k {
                    values[i as usize] = Some(p.value.clone())
                }
            }
        }
    }
    Ok(values)
}
fn normalized(g: &mut ModelGraph, a: &Atom) -> Result<Atom, PublicError> {
    let Atom::String(id) = a else {
        if matches!(a, Atom::Null | Atom::Undefined) {
            read(g, a, "trim")?;
        }
        return Err(type_error("e.trim is not a function"));
    };
    let raw = &g.strings[*id as usize].0;
    let ws = |u: u16| char::from_u32(u as u32).is_some_and(space);
    let mut from = 0;
    let mut to = raw.len();
    while from < to && ws(raw[from]) {
        from += 1
    }
    while to > from && ws(raw[to - 1]) {
        to -= 1
    }
    if to - from >= 2 && raw[from] == 34 && raw[to - 1] == 34 {
        from += 1;
        to -= 1
    }
    let s = JsString(raw[from..to].to_vec());
    Ok(Atom::String(intern(g, s)?))
}
fn mapped(g: &mut ModelGraph, a: &Atom, sort: bool) -> Result<Atom, PublicError> {
    let elements = elements(g, a)?;
    let length = elements.len() as u32;
    let mut normalized_values = Vec::new();
    for (i, v) in elements.into_iter().enumerate() {
        if let Some(v) = v {
            normalized_values.push((i, normalized(g, &v)?))
        }
    }
    if sort {
        normalized_values.sort_by(|a, b| {
            let Atom::String(a) = a.1 else { unreachable!() };
            let Atom::String(b) = b.1 else { unreachable!() };
            g.strings[a as usize].0.cmp(&g.strings[b as usize].0)
        });
    }
    let a = node(
        g,
        Node::Array {
            length,
            properties: vec![],
        },
    )?;
    for (i, (original, v)) in normalized_values.into_iter().enumerate() {
        set(
            g,
            &a,
            &if sort {
                i.to_string()
            } else {
                original.to_string()
            },
            v,
        )?
    }
    Ok(a)
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
fn set_collection(g: &ModelGraph, a: &Atom, key: &str, message: &str) -> Result<u32, PublicError> {
    let id = if let Atom::Ref(id) = a {
        *id
    } else {
        read(g, a, key)?;
        return Err(type_error(message));
    };
    if let Some(value) = property(g, id, key) {
        return Err(
            if matches!(value, Atom::BuiltinFunction(_) | Atom::HostReference(_)) {
                invalid("Venn replaced collection callbacks are not implemented").into()
            } else {
                type_error(message)
            },
        );
    }
    if !matches!(g.nodes[id as usize], Node::Set { .. }) {
        return Err(type_error(message));
    }
    Ok(id)
}
fn known_add(g: &mut ModelGraph, a: &Atom, v: Atom) -> Result<(), PublicError> {
    let id = set_collection(g, a, "add", "this.known.add is not a function")?;
    let Node::Set { values, .. } = &g.nodes[id as usize] else {
        unreachable!()
    };
    if values.len() >= 1_000_000 {
        return Err(limit().into());
    }
    let present = values.iter().any(|a| equal(g, a, &v));
    if !present {
        if let Node::Set { values, .. } = &mut g.nodes[id as usize] {
            values.push(v)
        }
    }
    Ok(())
}
fn iterable(g: &mut ModelGraph, a: &Atom, message: &str) -> Result<Vec<Atom>, PublicError> {
    match a {
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { .. } => Ok(elements(g, a)?
                .into_iter()
                .map(|a| a.unwrap_or(Atom::Undefined))
                .collect()),
            Node::Set { values, .. } => {
                if values.len() > 1_000_000 {
                    return Err(limit().into());
                }
                Ok(values.clone())
            }
            Node::Map { entries, .. } => {
                if entries.len() > 1_000_000 {
                    return Err(limit().into());
                }
                let entries = entries.clone();
                entries
                    .into_iter()
                    .map(|(k, v)| new_array(g, vec![k, v]))
                    .collect()
            }
            _ => Err(type_error(message)),
        },
        Atom::String(id) => {
            let units = g.strings[*id as usize].0.clone();
            if units.len() > 1_000_000 {
                return Err(limit().into());
            }
            let mut out = vec![];
            let mut i = 0;
            while i < units.len() {
                let n = if (0xd800..=0xdbff).contains(&units[i])
                    && i + 1 < units.len()
                    && (0xdc00..=0xdfff).contains(&units[i + 1])
                {
                    2
                } else {
                    1
                };
                out.push(Atom::String(intern(g, JsString(units[i..i + n].to_vec()))?));
                i += n
            }
            Ok(out)
        }
        _ => Err(type_error(message)),
    }
}
// Conservative preflight bounds this scalar graph interpreter's string
// interning, collection scans and Map/Set equality work. Unused application
// extras remain lossless; very large invocations receive an explicit Limit.
fn operation_budget(g: &ModelGraph, c: &MethodCall) -> Result<(), PublicError> {
    if !matches!(c.method.0, 1..=4) {
        return Ok(());
    }
    let hint = |a: Option<&Atom>| -> usize {
        match a {
            Some(Atom::Ref(id)) => match &g.nodes[*id as usize] {
                Node::Array { length, .. } => *length as usize,
                Node::Map { entries, .. } => entries.len(),
                Node::Set { values, .. } => values.len(),
                _ => 1,
            },
            Some(Atom::String(id)) => g.strings[*id as usize].0.len(),
            _ => 1,
        }
    };
    let mut operations = hint(c.arguments.first()).saturating_add(2);
    if c.method.0 == 3 {
        operations = operations.saturating_add(hint(c.arguments.get(1)).saturating_mul(4))
    }
    let graph_work = g.nodes.iter().fold(g.strings.len(), |n, node| {
        n.saturating_add(node.properties().len())
            .saturating_add(match node {
                Node::Map { entries, .. } => entries.len().saturating_mul(2),
                Node::Set { values, .. } => values.len(),
                _ => 0,
            })
    });
    let units = g
        .strings
        .iter()
        .fold(0usize, |n, s| n.saturating_add(s.0.len()));
    if operations.saturating_mul(graph_work.saturating_add(operations)) > 32_000_000
        || operations.saturating_mul(units) > 128_000_000
    {
        return Err(limit().into());
    }
    Ok(())
}
fn invoke(g: &mut ModelGraph, c: MethodCall) -> Result<Atom, PublicError> {
    operation_budget(g, &c)?;
    let r = c.receiver;
    let a = c.arguments;
    let first = a.first().cloned().unwrap_or(Atom::Undefined);
    match c.method.0 {
        0 => set(g, &r, "title", first)?,
        5 => return read(g, &r, "current"),
        6 => return read(g, &r, "indent"),
        7 => set(g, &r, "indent", first)?,
        1 => {
            let sets = mapped(g, &first, true)?;
            set(g, &r, "current", sets.clone())?;
            if matches!(read(g,&sets,"length")?,Atom::Number(n)if n==1.) {
                let known = read(g, &r, "known")?;
                let v = read(g, &sets, "0")?;
                known_add(g, &known, v)?
            }
            let subsets = read(g, &r, "subsets")?;
            // Member access precedes argument evaluation; .push callability is checked last.
            read(g, &subsets, "push")?;
            let raw_size = a.get(2).cloned().unwrap_or(Atom::Undefined);
            let size = if matches!(raw_size, Atom::Null | Atom::Undefined) {
                let len = numeric(g, &read(g, &first, "length")?)?;
                Atom::Number(10. / len.powi(2))
            } else {
                raw_size
            };
            let raw_label = a.get(1).cloned().unwrap_or(Atom::Undefined);
            let label = if truthy(g, &raw_label) {
                normalized(g, &raw_label)?
            } else {
                Atom::Undefined
            };
            let subset = new_object(g, &[("sets", sets), ("size", size), ("label", label)])?;
            push(g, subsets, subset, "this.subsets.push is not a function")?;
        }
        2 => {
            let texts = read(g, &r, "textNodes")?;
            read(g, &texts, "push")?;
            let sets = mapped(g, &first, true)?;
            let id = normalized(g, &a.get(1).cloned().unwrap_or(Atom::Undefined))?;
            let raw = a.get(2).cloned().unwrap_or(Atom::Undefined);
            let label = if truthy(g, &raw) {
                normalized(g, &raw)?
            } else {
                Atom::Undefined
            };
            let text = new_object(g, &[("sets", sets), ("id", id), ("label", label)])?;
            push(g, texts, text, "this.textNodes.push is not a function")?;
        }
        3 => {
            let data = a.get(1).cloned().unwrap_or(Atom::Undefined);
            let map = node(
                g,
                Node::Map {
                    entries: vec![],
                    properties: vec![],
                },
            )?;
            let Atom::Ref(map) = map else { unreachable!() };
            for pair in iterable(g, &data, "n is not iterable")? {
                let pair = iterable(g, &pair, ".for is not iterable")?;
                let key = pair.first().cloned().unwrap_or(Atom::Undefined);
                let raw = pair.get(1).cloned().unwrap_or(Atom::Undefined);
                let value = if truthy(g, &raw) {
                    normalized(g, &raw)?
                } else {
                    raw
                };
                let Node::Map { entries, .. } = &g.nodes[map as usize] else {
                    unreachable!()
                };
                let index = entries.iter().position(|(k, _)| equal(g, k, &key));
                if let Node::Map { entries, .. } = &mut g.nodes[map as usize] {
                    if let Some(i) = index {
                        entries[i].1 = value
                    } else {
                        entries.push((key, value))
                    }
                }
            }
            let styles = read(g, &r, "styles")?;
            read(g, &styles, "push")?;
            let targets = mapped(g, &first, true)?;
            let style = new_object(g, &[("targets", targets), ("styles", Atom::Ref(map))])?;
            push(g, styles, style, "this.styles.push is not a function")?;
        }
        4 => {
            let mapped = mapped(g, &first, false)?;
            let values = elements(g, &mapped)?;
            let mut unknown = Vec::new();
            for value in values.into_iter().flatten() {
                let known = read(g, &r, "known")?;
                let id = set_collection(g, &known, "has", "this.known.has is not a function")?;
                let Node::Set { values, .. } = &g.nodes[id as usize] else {
                    unreachable!()
                };
                if !values.iter().any(|a| equal(g, a, &value)) {
                    let Atom::String(i) = value else {
                        unreachable!()
                    };
                    unknown.push(g.strings[i as usize].clone())
                }
            }
            if !unknown.is_empty() {
                let mut units: Vec<u16> = "unknown set identifier: ".encode_utf16().collect();
                for (i, s) in unknown.into_iter().enumerate() {
                    if i > 0 {
                        units.extend([44, 32])
                    }
                    units.extend(s.0);
                    if units.len() > 16_000_000 {
                        return Err(limit().into());
                    }
                }
                let message = JsString(units);
                if let Some(message) = message.well_formed() {
                    return Err(invalid(&message).into());
                }
                return Err(PublicPeleError {
                    message,
                    code: ErrorCode::Semantic,
                    diagram_type: Some("venn".into()),
                    line: 0,
                    column: 0,
                    snippet: "".into(),
                }
                .into());
            }
        }
        _ => return Err(invalid("Unknown Venn method").into()),
    }
    Ok(Atom::Undefined)
}
struct Reader {
    values: usize,
    scans: usize,
    text: usize,
}
impl Reader {
    fn array(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Vec<Atom>, PeleError> {
        let Some(Atom::Ref(id)) = a else {
            return Err(invalid("Venn active collection requires an Array"));
        };
        self.scans = self
            .scans
            .saturating_add(g.nodes[*id as usize].properties().len());
        let Node::Array { length, properties } = &g.nodes[*id as usize] else {
            return Err(invalid("Venn active collection requires an Array"));
        };
        self.values = self.values.saturating_add(*length as usize);
        if self.values > 1_000_000 || self.scans > 32_000_000 {
            return Err(limit());
        }
        let mut values = HashMap::new();
        for p in properties {
            let raw = &g.strings[p.key as usize];
            if raw.0.len() > 10 {
                continue;
            }
            if let Some(k) = raw.well_formed() {
                if let Ok(i) = k.parse::<u32>() {
                    if i < *length && i.to_string() == k {
                        values.insert(i, p.value.clone());
                    }
                }
            }
        }
        if values.len() != *length as usize {
            return Err(invalid(
                "Venn sparse active collections are not implemented",
            ));
        }
        Ok((0..*length).map(|i| values.remove(&i).unwrap()).collect())
    }
    fn record(&mut self, g: &ModelGraph, a: &Atom, reads: usize) -> Result<u32, PeleError> {
        let Atom::Ref(id) = a else {
            return Err(invalid("Venn active record requires an object"));
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
    fn text(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<String, PeleError> {
        let Some(Atom::String(id)) = a else {
            return Err(invalid("Venn active text coercion is not implemented"));
        };
        let s = &g.strings[*id as usize];
        self.text = self.text.saturating_add(s.0.len());
        if self.text > 16_000_000 {
            return Err(limit());
        }
        s.well_formed()
            .ok_or_else(|| invalid("Venn rendering of lone UTF-16 text is not implemented"))
    }
    fn strings(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Vec<String>, PeleError> {
        self.array(g, a)?
            .iter()
            .map(|a| self.text(g, Some(a)))
            .collect()
    }
    fn label(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Option<String>, PeleError> {
        match a {
            None | Some(Atom::Undefined | Atom::Null) => Ok(None),
            Some(a) if !truthy(g, a) => Ok(Some(String::new())),
            _ => self.text(g, a).map(Some),
        }
    }
}
fn imported(g: &ModelGraph) -> Result<VennModel, PeleError> {
    let mut b = Reader {
        values: 0,
        scans: 0,
        text: 0,
    };
    let r = b.record(g, &g.root, 5)?;
    let mut m = VennModel::new(None);
    m.title = match property(g, r, "title") {
        None | Some(Atom::Undefined) => None,
        a => Some(b.text(g, a)?),
    };
    let mut nonnumber_singleton = false;
    for subset in b.array(g, property(g, r, "subsets"))? {
        let id = b.record(g, &subset, 3)?;
        let sets = b.strings(g, property(g, id, "sets"))?;
        let raw_size = property(g, id, "size").unwrap_or(&Atom::Undefined);
        let number_size = matches!(raw_size, Atom::Number(_));
        nonnumber_singleton |= sets.len() == 1 && !number_size;
        let size = numeric(g, raw_size)
            .map_err(|_| invalid("Venn active object size coercion is not implemented"))?;
        let label = b.label(g, property(g, id, "label"))?;
        // Number.isFinite is strict for singleton circle radii; overlap area
        // arithmetic later coerces primitives. A triple needs both views and
        // cannot use this existing scalar-only cache for nonnumeric singletons.
        let size = if sets.len() == 1 && !number_size {
            f64::NAN
        } else {
            size
        };
        m.subsets.push(VennSubset { sets, size, label });
    }
    if nonnumber_singleton && m.subsets.iter().any(|s| s.sets.len() >= 3) {
        return Err(invalid(
            "Venn triple inference with nonnumeric singleton sizes is not implemented",
        ));
    }
    let identifiers: HashSet<_> = m.subsets.iter().flat_map(|s| s.sets.iter()).collect();
    let circles_work = identifiers.len().saturating_mul(identifiers.len());
    if circles_work > 128_000_000 {
        return Err(limit());
    }
    let pair_work = m
        .subsets
        .iter()
        .filter(|s| s.sets.len() >= 3)
        .fold(0usize, |n, s| {
            n.saturating_add(s.sets.len().saturating_mul(s.sets.len() - 1) / 2)
        });
    // Preserve the existing renderer's exact inferred-pair Limit error. It
    // executes before region geometry, so do not preempt it with a new guard.
    if pair_work <= 3_000_000 {
        let geometry_work = m.subsets.iter().fold(circles_work, |n, s| {
            n.saturating_add(s.sets.len().saturating_pow(3))
        });
        if geometry_work > 128_000_000 {
            return Err(limit());
        }
    }
    let regions: HashSet<_> = m.subsets.iter().map(|s| s.sets.clone()).collect();
    for text in b.array(g, property(g, r, "textNodes"))? {
        let id = b.record(g, &text, 3)?;
        let sets = b.strings(g, property(g, id, "sets"))?;
        // Upstream only lays out id/label if this ordered region key exists.
        if !regions.contains(&sets) {
            continue;
        }
        let raw_label = property(g, id, "label");
        if !matches!(
            raw_label,
            None | Some(Atom::String(_) | Atom::Null | Atom::Undefined)
        ) {
            return Err(invalid(
                "Venn non-string active text labels are not implemented",
            ));
        }
        let label = b.label(g, raw_label)?;
        let name = b.text(g, property(g, id, "id"))?;
        m.text_nodes.push(VennText {
            sets,
            id: name,
            label,
        });
    }
    for style in b.array(g, property(g, r, "styles"))? {
        let id = b.record(g, &style, 2)?;
        let targets = b.strings(g, property(g, id, "targets"))?;
        let Some(Atom::Ref(map)) = property(g, id, "styles") else {
            return Err(invalid("Venn active styles require a Map"));
        };
        let Node::Map { entries, .. } = &g.nodes[*map as usize] else {
            return Err(invalid("Venn active styles require a Map"));
        };
        b.values = b.values.saturating_add(entries.len());
        if b.values > 1_000_000 {
            return Err(limit());
        }
        let mut styles = vec![];
        for (k, v) in entries {
            let k = primitive(g, k)
                .map_err(|_| invalid("Venn style object coercion is not implemented"))?;
            let v = primitive(g, v)
                .map_err(|_| invalid("Venn style object coercion is not implemented"))?;
            b.text = b
                .text
                .saturating_add(k.encode_utf16().count())
                .saturating_add(v.encode_utf16().count());
            if b.text > 16_000_000 {
                return Err(limit());
            }
            styles.push((k, v))
        }
        m.styles.push(VennStyle { targets, styles });
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
fn strings_out(out: &mut GraphWriter, values: &[String]) -> u32 {
    let values = values
        .iter()
        .map(|s| Atom::String(out.string(s.as_str())))
        .collect();
    array_out(out, values)
}
fn construct_default() -> Result<VennPublicState, PeleError> {
    // Implicit Pele constructor: field defaults only, no source or argument reads.
    Venn.create_public(&VennModel::new(None))
}

impl PublicDiagramModule for Venn {
    type PublicState = VennPublicState;
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
        Ok(VennPublicState { graph, model })
    }
    fn create_public(&self, m: &VennModel) -> Result<VennPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut subsets = vec![];
        let known = m
            .known_order
            .iter()
            .filter(|s| m.known.contains(*s))
            .map(|s| Atom::String(out.string(s.as_str())))
            .collect();
        let current_index = m
            .current
            .as_ref()
            .and_then(|current| m.subsets.iter().rposition(|s| s.sets == *current));
        let mut current = Atom::Undefined;
        for (index, subset) in m.subsets.iter().enumerate() {
            let sets = strings_out(&mut out, &subset.sets);
            if Some(index) == current_index {
                current = Atom::Ref(sets)
            }
            let label = subset
                .label
                .as_deref()
                .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
            let properties = vec![
                out.property("sets", Atom::Ref(sets)),
                out.property("size", Atom::Number(subset.size)),
                out.property("label", label),
            ];
            subsets.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        if current_index.is_none() {
            if let Some(sets) = &m.current {
                current = Atom::Ref(strings_out(&mut out, sets))
            }
        }
        let subsets = array_out(&mut out, subsets);
        let known = out.node(Node::Set {
            values: known,
            properties: vec![],
        });
        let mut texts = vec![];
        for text in &m.text_nodes {
            let sets = strings_out(&mut out, &text.sets);
            let id = out.string(text.id.as_str());
            let label = text
                .label
                .as_deref()
                .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
            let properties = vec![
                out.property("sets", Atom::Ref(sets)),
                out.property("id", Atom::String(id)),
                out.property("label", label),
            ];
            texts.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        let texts = array_out(&mut out, texts);
        let mut styles = vec![];
        for style in &m.styles {
            let targets = strings_out(&mut out, &style.targets);
            let entries = style
                .styles
                .iter()
                .map(|(k, v)| {
                    (
                        Atom::String(out.string(k.as_str())),
                        Atom::String(out.string(v.as_str())),
                    )
                })
                .collect();
            let map = out.node(Node::Map {
                entries,
                properties: vec![],
            });
            let properties = vec![
                out.property("targets", Atom::Ref(targets)),
                out.property("styles", Atom::Ref(map)),
            ];
            styles.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        let styles = array_out(&mut out, styles);
        let kind = out.string("venn");
        let title = m
            .title
            .as_deref()
            .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
        let properties = vec![
            out.property("type", Atom::String(kind)),
            out.property("title", title),
            out.property("subsets", Atom::Ref(subsets)),
            out.property("textNodes", Atom::Ref(texts)),
            out.property("styles", Atom::Ref(styles)),
            out.property("known", Atom::Ref(known)),
            out.property("current", current),
            out.property("indent", Atom::Bool(m.indent)),
        ];
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let graph = out.finish(Atom::Ref(root))?;
        graph.validate_schema(&SCHEMA)?;
        let model = imported(&graph);
        Ok(VennPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &VennPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut VennPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut VennPublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke(&mut s.graph, c);
        s.model = imported(&s.graph);
        result
    }
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::venn())
    }
    fn detached_public_render(
        &self,
        state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::venn(
            super::VennDetachedInput::new(state.model.as_ref().err().cloned()),
        ))
    }
    fn render_public(
        &self,
        s: &mut VennPublicState,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        super::render::render(s.model.as_ref().map_err(Clone::clone)?, c, o, r)
    }
}
