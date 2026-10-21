//! IshikawaDb's actual addNode, stack aliases and flat native fishbone cache.
//! Pele 458322a5 (MIT kepano). Exotic coercion and active lone UTF16
//! text remain explicit gaps; cyclic/expanded workloads have native budgets.
use super::{Ishikawa, IshikawaModel, IshikawaNode};
use crate::compat::{
    public::*, source::space, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::HashMap;
const PROTO: PrototypeId = PrototypeId(1);
const ADD: MethodId = MethodId(0);
const fn fs(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
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
    id: "pele/ishikawa@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "Ishikawa",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                fs("type", FieldKind::String, false),
                fs("title", FieldKind::String, true),
                fs("root", FieldKind::Record(RecordId(1)), true),
                fs("stack", FieldKind::Array, false),
                fs("baseLevel", FieldKind::Number, true),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(1),
            name: "IshikawaNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                fs("text", FieldKind::String, false),
                fs("children", FieldKind::Array, false),
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "IshikawaAncestor",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                fs("level", FieldKind::Number, false),
                fs("node", FieldKind::Record(RecordId(1)), false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "IshikawaDb",
        parent: PrototypeParent::Object,
        members: &[MemberSchema {
            name: "addNode",
            flags: 6,
            member: MemberKind::Method { method: ADD },
        }],
    }],
    methods: &[MethodSchema {
        id: ADD,
        name: "addNode",
        arity: 2,
    }],
};
struct Cache {
    model: IshikawaModel,
}
pub struct IshikawaPublicState {
    graph: ModelGraph,
    cache: Result<Cache, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("ishikawa".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("Ishikawa public operation exceeds native work budget");
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
            invalid("Ishikawa runtime error text containing lone UTF-16 is not implemented").into()
        }),
        _ => Err(invalid("Ishikawa arbitrary object coercion is not implemented").into()),
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
            Err(invalid("Ishikawa String collection indexing is not implemented").into())
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
        _ => return Err(invalid("Ishikawa borrowed host receiver is not implemented").into()),
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
        _ => Err(invalid("Ishikawa object-to-number coercion is not implemented").into()),
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

fn reference(a: &Atom) -> Result<u32, PeleError> {
    if let Atom::Ref(i) = a {
        Ok(*i)
    } else {
        Err(invalid("Ishikawa active record requires an object"))
    }
}
fn dense(g: &ModelGraph, a: &Atom) -> Result<Vec<Atom>, PeleError> {
    let id = reference(a)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(invalid("Ishikawa active children must be Arrays"));
    };
    if *length > 1_000_000 {
        return Err(limit());
    }
    let mut values = HashMap::new();
    for p in properties {
        let s = &g.strings[p.key as usize];
        if s.0.is_empty() || s.0.len() > 10 {
            continue;
        }
        if let Some(k) = s.well_formed() {
            if let Ok(i) = k.parse::<u32>() {
                if i < *length && i.to_string() == k {
                    values.insert(i, p.value.clone());
                }
            }
        }
    }
    if values.len() != *length as usize {
        return Err(invalid(
            "Ishikawa sparse active children are not implemented",
        ));
    }
    Ok((0..*length).map(|i| values.remove(&i).unwrap()).collect())
}
struct Text(usize);
impl Text {
    fn get(&mut self, g: &ModelGraph, a: Option<&Atom>) -> Result<Option<String>, PeleError> {
        match a {
            None | Some(Atom::Undefined) => Ok(None),
            Some(Atom::String(i)) => {
                let s = &g.strings[*i as usize];
                self.0 = self.0.saturating_add(s.0.len());
                if self.0 > 16_000_000 {
                    return Err(limit());
                }
                s.well_formed().map(Some).ok_or_else(|| {
                    invalid("Ishikawa rendering of lone UTF-16 text is not implemented")
                })
            }
            _ => Err(invalid(
                "Ishikawa rendering of non-string active text is not implemented",
            )),
        }
    }
}
fn imported(g: &ModelGraph) -> Result<Cache, PeleError> {
    let r = reference(&g.root)?;
    let mut text = Text(0);
    let mut model = IshikawaModel::new(None);
    model.title = text.get(g, property(g, r, "title"))?;
    // Both upstream svgDocument calls receive {}, so caller-added accTitle/
    // accDescr extras are retained but have no effect on this family's SVG.
    let a = property(g, r, "root").cloned().unwrap_or(Atom::Undefined);
    if !truthy(g, &a) {
        return Ok(Cache { model });
    }
    let root = reference(&a)?;
    let mut pending = vec![(root, false)];
    let mut state = HashMap::new();
    let mut index = HashMap::new();
    let mut edges: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut sizes: HashMap<u32, usize> = HashMap::new();
    let mut work = 0usize;
    while let Some((id, exit)) = pending.pop() {
        if exit {
            let size = edges[&id]
                .iter()
                .fold(1usize, |n, c| n.saturating_add(sizes[c]));
            if size > 1_000_000 {
                return Err(limit());
            }
            sizes.insert(id, size);
            state.insert(id, 2);
            continue;
        }
        match state.get(&id) {
            Some(1) => return Err(invalid("Ishikawa cyclic active children are not supported")),
            Some(2) => continue,
            _ => {}
        }
        work = work.saturating_add(g.nodes[id as usize].properties().len().saturating_mul(4));
        if work > 32_000_000 || index.len() >= 1_000_000 {
            return Err(limit());
        }
        let raw_text = property(g, id, "text");
        if matches!(raw_text, None | Some(Atom::Undefined)) {
            return Err(invalid(
                "Ishikawa rendering of absent node text is not implemented",
            ));
        }
        let label = text.get(g, raw_text)?.unwrap_or_default();
        let children = property(g, id, "children")
            .ok_or_else(|| invalid("Ishikawa active children are absent"))?;
        work = work.saturating_add(g.nodes[reference(children)? as usize].properties().len());
        if work > 32_000_000 {
            return Err(limit());
        }
        let children = dense(g, children)?
            .iter()
            .map(reference)
            .collect::<Result<Vec<_>, _>>()?;
        index.insert(id, model.nodes.len());
        model.nodes.push(IshikawaNode {
            text: label,
            children: vec![],
        });
        state.insert(id, 1);
        pending.push((id, true));
        for &c in children.iter().rev() {
            pending.push((c, false))
        }
        edges.insert(id, children);
    }
    for (id, children) in edges {
        model.nodes[index[&id]].children = children.into_iter().map(|c| index[&c]).collect();
    }
    model.root = Some(index[&root]);
    Ok(Cache { model })
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
        push_raw(g, a.clone(), v, "Ishikawa native array push")?
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
                invalid("Ishikawa replaced collection callbacks are not implemented").into()
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
fn pop(g: &mut ModelGraph, a: &Atom) -> Result<(), PublicError> {
    let id = array_method(g, a, "pop", "s.pop is not a function")?;
    let Node::Array { length, .. } = g.nodes[id as usize] else {
        unreachable!()
    };
    if length == 0 {
        return Ok(());
    }
    let key = (length - 1).to_string();
    let properties = g.nodes[id as usize].properties_mut();
    if let Some(i) = properties
        .iter()
        .position(|p| g.strings[p.key as usize].is(&key))
    {
        if properties[i].flags & 4 == 0 {
            return Err(invalid(
                "Ishikawa pop of nonconfigurable array indices is not implemented",
            )
            .into());
        }
        properties.remove(i);
    }
    if let Node::Array { length, .. } = &mut g.nodes[id as usize] {
        *length -= 1
    }
    Ok(())
}
fn add(g: &mut ModelGraph, receiver: Atom, arguments: Vec<Atom>) -> Result<Atom, PublicError> {
    let level_arg = arguments.first().cloned().unwrap_or(Atom::Undefined);
    let label = arguments.get(1).cloned().unwrap_or(Atom::Undefined);
    let children = new_array(g, vec![])?;
    let n = new_object(g, &[("text", label), ("children", children)])?;
    if !truthy(g, &read(g, &receiver, "root")?) {
        set(g, &receiver, "root", n.clone())?;
        let entry = new_object(g, &[("level", Atom::Number(0.)), ("node", n)])?;
        let stack = new_array(g, vec![entry])?;
        set(g, &receiver, "stack", stack)?;
        return Ok(Atom::Undefined);
    }
    let base = read(g, &receiver, "baseLevel")?;
    let base = if matches!(base, Atom::Null | Atom::Undefined) {
        set(g, &receiver, "baseLevel", level_arg.clone())?;
        level_arg.clone()
    } else {
        base
    };
    let delta = numeric(g, &level_arg)? - numeric(g, &base)? + 1.;
    let level = if delta.is_nan() { delta } else { delta.max(1.) };
    let stack = read(g, &receiver, "stack")?;
    let mut work = 0usize;
    loop {
        let len = numeric(g, &read(g, &stack, "length")?)?;
        if len <= 1. || len.is_nan() {
            break;
        }
        if let Atom::Ref(id) = stack {
            work = work.saturating_add(g.nodes[id as usize].properties().len().saturating_mul(4));
            if work > 32_000_000 {
                return Err(limit().into());
            }
        }
        let last = read(g, &stack, &number(len - 1.))?;
        let old = numeric(g, &read(g, &last, "level")?)?;
        if old < level || old.is_nan() || level.is_nan() {
            break;
        }
        pop(g, &stack)?;
    }
    let len = numeric(g, &read(g, &stack, "length")?)?;
    let last = read(g, &stack, &number(len - 1.))?;
    let parent = read(g, &last, "node")?;
    let children = read(g, &parent, "children")?;
    push(
        g,
        children,
        n.clone(),
        "s[(s.length - 1)].node.children.push is not a function",
    )?;
    let entry = new_object(g, &[("level", Atom::Number(level)), ("node", n)])?;
    push(g, stack, entry, "s.push is not a function")?;
    Ok(Atom::Undefined)
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
fn construct_default() -> Result<IshikawaPublicState, PeleError> {
    // Implicit Pele constructor: field defaults only, no source or argument reads.
    Ishikawa.create_public(&IshikawaModel::new(None))
}

impl PublicDiagramModule for Ishikawa {
    type PublicState = IshikawaPublicState;
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::ishikawa())
    }
    fn detached_public_render(
        &self,
        state: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::ishikawa(
            super::IshikawaDetachedInput::new(state.cache.as_ref().err()),
        ))
    }

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
        let cache = imported(&graph);
        Ok(IshikawaPublicState { graph, cache })
    }
    fn create_public(&self, m: &IshikawaModel) -> Result<IshikawaPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let ids = m
            .nodes
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        let mut filled = vec![];
        for (i, n) in m.nodes.iter().enumerate() {
            let text = out.string(n.text.as_str());
            let children = array_out(
                &mut out,
                n.children.iter().map(|&i| Atom::Ref(ids[i])).collect(),
            );
            filled.push((
                ids[i],
                vec![
                    out.property("text", Atom::String(text)),
                    out.property("children", Atom::Ref(children)),
                ],
            ));
        }
        let mut stack = vec![];
        for (level, id) in &m.stack {
            let properties = vec![
                out.property("level", Atom::Number(*level)),
                out.property("node", Atom::Ref(ids[*id])),
            ];
            stack.push(Atom::Ref(out.node(Node::Object { properties })));
        }
        let stack = array_out(&mut out, stack);
        let kind = out.string("ishikawa");
        let title = m
            .title
            .as_deref()
            .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
        let properties = vec![
            out.property("type", Atom::String(kind)),
            out.property("title", title),
            out.property(
                "root",
                m.root.map_or(Atom::Undefined, |i| Atom::Ref(ids[i])),
            ),
            out.property("stack", Atom::Ref(stack)),
            out.property(
                "baseLevel",
                m.base_level.map_or(Atom::Undefined, Atom::Number),
            ),
        ];
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let mut graph = out.finish(Atom::Ref(root))?;
        for (id, properties) in filled {
            *graph.nodes[id as usize].properties_mut() = properties
        }
        graph.validate_schema(&SCHEMA)?;
        let cache = imported(&graph);
        Ok(IshikawaPublicState { graph, cache })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &IshikawaPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut IshikawaPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.cache = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut IshikawaPublicState,
        c: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = if c.method == ADD {
            add(&mut s.graph, c.receiver, c.arguments)
        } else {
            Err(invalid("Unknown Ishikawa method").into())
        };
        s.cache = imported(&s.graph);
        result
    }
    fn render_public(
        &self,
        s: &mut IshikawaPublicState,
        _: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        let c = s.cache.as_ref().map_err(Clone::clone)?;
        super::render::render(&c.model, o, r)
    }
}
