//! Native mutable MindmapDb, Pele 458322a5 (MIT kepano).
//! Exotic runtime coercion/descriptors, detached child records,
//! active lone UTF-16 and sparse active arrays remain explicit gaps.
use super::{Mindmap, MindmapModel, MindmapNode};
use crate::compat::{
    public::*, source::space, DiagramModule, ErrorCode, Object, PeleError, RenderResources,
    RenderSettings, Rendered,
};
use std::collections::HashMap;
const PROTO: PrototypeId = PrototypeId(1);
const GET_TYPE: MethodId = MethodId(0);
const ADD_NODE: MethodId = MethodId(1);
const DECORATE: MethodId = MethodId(2);
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
    id: "pele/mindmap@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "Mindmap",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false, false),
                field_schema("title", FieldKind::String, false, true),
                field_schema("nodes", FieldKind::Array, false, false),
                field_schema("baseLevel", FieldKind::Number, false, false),
                field_schema("open", FieldKind::Array, false, false),
            ],
            members: &[MemberSchema {
                name: "getType",
                flags: 7,
                member: MemberKind::Method { method: GET_TYPE },
            }],
        },
        RecordSchema {
            id: RecordId(1),
            name: "MindmapNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::Number, false, false),
                field_schema("nodeId", FieldKind::String, false, false),
                field_schema("level", FieldKind::Number, false, false),
                field_schema("descr", FieldKind::String, false, false),
                field_schema("type", FieldKind::Number, false, false),
                field_schema("children", FieldKind::Array, false, false),
                field_schema("isRoot", FieldKind::Bool, false, false),
                field_schema("section", FieldKind::Number, true, true),
                field_schema("class", FieldKind::String, true, false),
                field_schema("icon", FieldKind::String, true, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "MindmapDb",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "addNode",
                flags: 6,
                member: MemberKind::Method { method: ADD_NODE },
            },
            MemberSchema {
                name: "decorateNode",
                flags: 6,
                member: MemberKind::Method { method: DECORATE },
            },
        ],
    }],
    methods: &[
        MethodSchema {
            id: GET_TYPE,
            name: "getType",
            arity: 2,
        },
        MethodSchema {
            id: ADD_NODE,
            name: "addNode",
            arity: 4,
        },
        MethodSchema {
            id: DECORATE,
            name: "decorateNode",
            arity: 1,
        },
    ],
};
pub struct MindmapPublicState {
    graph: ModelGraph,
    model: Result<MindmapModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("mindmap".into());
    e
}
fn limit() -> PeleError {
    let mut e = PeleError::new(
        "Mindmap public operation exceeds native work budget",
        ErrorCode::Limit,
    );
    e.diagram_type = Some("mindmap".into());
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
            invalid("Mindmap runtime error text containing lone UTF-16 is not implemented").into()
        }),
        _ => Err(invalid("Mindmap arbitrary object coercion is not implemented").into()),
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
            Err(invalid("Mindmap String collection indexing is not implemented").into())
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
        _ => return Err(invalid("Mindmap borrowed host receiver is not implemented").into()),
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
        return Err(invalid("Mindmap active collections must be Arrays"));
    };
    let Node::Array { length, properties } = &g.nodes[*id as usize] else {
        return Err(invalid("Mindmap active collections must be Arrays"));
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
        return Err(invalid("Mindmap sparse active arrays are not implemented"));
    }
    Ok((0..*length).map(|i| values.remove(&i).unwrap()).collect())
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
                "Mindmap rendering of non-string active text is not implemented",
            ));
        };
        let raw = &g.strings[*id as usize];
        self.0 = self.0.checked_add(raw.0.len()).ok_or_else(limit)?;
        if self.0 > 16_000_000 {
            return Err(limit());
        }
        raw.well_formed().map(Some).ok_or_else(|| {
            invalid("Mindmap rendering of lone UTF-16 surrogates is not implemented")
        })
    }
}
fn index(a: Option<&Atom>, n: usize) -> Result<usize, PeleError> {
    match a {
        Some(Atom::Number(v))
            if v.is_finite() && *v >= 0. && v.fract() == 0. && *v < (n as f64) =>
        {
            Ok(*v as usize)
        }
        _ => Err(invalid(
            "Mindmap rendering of non-index child IDs is not implemented",
        )),
    }
}
fn imported(g: &ModelGraph) -> Result<MindmapModel, PeleError> {
    let Atom::Ref(r) = g.root else {
        return Err(invalid("Mindmap root must be an object"));
    };
    let flat = array(g, property(g, r, "nodes").unwrap_or(&Atom::Undefined))?;
    let mut text = Text(0);
    let mut nodes = vec![];
    let mut edges = 0usize;
    for (i, a) in flat.iter().enumerate() {
        let Atom::Ref(id) = a else {
            return Err(invalid("Mindmap nodes must be objects"));
        };
        let get = |key| property(g, *id, key);
        let mut children = vec![];
        for child in array(g, get("children").unwrap_or(&Atom::Undefined))? {
            let Atom::Ref(child_ref) = child else {
                return Err(invalid("Mindmap child must be an object"));
            };
            let child_id = index(property(g, child_ref, "id"), flat.len())?;
            if !matches!(flat[child_id],Atom::Ref(found)if found==child_ref) {
                return Err(invalid(
                    "Mindmap detached child records with numeric IDs are not implemented",
                ));
            }
            children.push(child_id);
            edges += 1;
            if edges > 8_000_000 {
                return Err(limit());
            }
        }
        let node_type = match get("type") {
            Some(Atom::Number(n)) if (1. ..=6.).contains(n) && n.fract() == 0. => *n as u8,
            Some(Atom::String(id)) => match g.strings[*id as usize].well_formed().as_deref() {
                Some("1") => 1,
                Some("2") => 2,
                Some("3") => 3,
                Some("4") => 4,
                Some("5") => 5,
                Some("6") => 6,
                Some("length" | "constructor" | "__proto__" | "map" | "toString") => {
                    return Err(invalid(
                        "Mindmap shape-array prototype lookup is not implemented",
                    ))
                }
                Some("0") => 0,
                _ => {
                    return Err(invalid(
                        "Mindmap non-number shape coercion is not implemented",
                    ))
                }
            },
            _ => 0,
        };
        let section = match get("section") {
            None | Some(Atom::Null | Atom::Undefined) => None,
            Some(Atom::Number(n))
                if n.is_finite() && *n >= 0. && n.fract() == 0. && *n < (usize::MAX as f64) =>
            {
                Some(*n as usize)
            }
            _ => {
                return Err(invalid(
                    "Mindmap fractional/negative/non-number sections are not implemented",
                ))
            }
        };
        nodes.push(MindmapNode {
            id: i,
            node_id: text.get(g, get("nodeId"), false)?.unwrap(),
            level: 0,
            descr: text.get(g, get("descr"), true)?.unwrap_or_default(),
            node_type,
            children,
            is_root: get("isRoot").is_some_and(|a| truthy(g, a)),
            section,
            class: match get("class") {
                Some(Atom::Null) => None,
                a => text.get(g, a, true)?,
            },
            icon: text.get(g, get("icon"), true)?,
            public_property_order: vec![],
        });
    }
    Ok(MindmapModel {
        title: text.get(g, property(g, r, "title"), true)?,
        nodes,
        base_level: 0,
        open: vec![],
    })
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
        _ => Err(invalid("Mindmap object-to-number coercion is not implemented").into()),
    }
}
fn set_length(g: &mut ModelGraph, a: &Atom, depth: f64) -> Result<(), PublicError> {
    if let Atom::Ref(id) = a {
        if let Node::Array { properties, .. } = &g.nodes[*id as usize] {
            if !depth.is_finite() || depth < 0. || depth.fract() != 0. || depth > u32::MAX as f64 {
                return Err(js(JavaScriptErrorKind::RangeError, "Invalid array length"));
            }
            let length = depth as u32;
            let mut kept = vec![];
            for p in properties {
                let key = g.strings[p.key as usize].well_formed();
                let removed = key
                    .and_then(|k| {
                        k.parse::<u32>()
                            .ok()
                            .filter(|n| *n != u32::MAX && n.to_string() == k)
                    })
                    .is_some_and(|n| n >= length);
                if removed {
                    if p.flags & 4 == 0 {
                        return Err(invalid("Mindmap truncation of nonconfigurable array indices is not implemented").into());
                    }
                } else {
                    kept.push(p.clone())
                }
            }
            g.nodes[*id as usize] = Node::Array {
                length,
                properties: kept,
            };
            return Ok(());
        }
    }
    set(g, a, "length", Atom::Number(depth))
}
fn invoke(g: &mut ModelGraph, c: MethodCall) -> Result<Atom, PublicError> {
    let arg = |i| c.arguments.get(i).cloned().unwrap_or(Atom::Undefined);
    match c.method {
        GET_TYPE => {
            let start = arg(0);
            let is = |text| matches!(&start,Atom::String(id)if g.strings[*id as usize].is(text));
            let n = if is("[") {
                2.
            } else if is("(") {
                if matches!(arg(1),Atom::String(id)if g.strings[id as usize].is(")")) {
                    1.
                } else {
                    4.
                }
            } else if is("((") {
                3.
            } else if is(")") {
                4.
            } else if is("))") {
                5.
            } else if is("{{") {
                6.
            } else {
                0.
            };
            return Ok(Atom::Number(n));
        }
        DECORATE => {
            let nodes = read(g, &c.receiver, "nodes")?;
            let length = numeric(g, &read(g, &nodes, "length")?)?;
            let last = read(g, &nodes, &number(length - 1.))?;
            let decoration = arg(0);
            if !truthy(g, &decoration) || !truthy(g, &last) {
                return Ok(Atom::Undefined);
            }
            let icon = read(g, &decoration, "icon")?;
            if truthy(g, &icon) {
                set(g, &last, "icon", icon)?
            }
            let class = read(g, &decoration, "class")?;
            if truthy(g, &class) {
                set(g, &last, "class", class)?
            }
        }
        ADD_NODE => {
            let nodes = read(g, &c.receiver, "nodes")?;
            let length = read(g, &nodes, "length")?;
            let is_root = matches!(length,Atom::Number(n)if n==0.);
            if is_root {
                set(g, &c.receiver, "baseLevel", arg(0))?
            }
            let level = numeric(g, &arg(0))? - numeric(g, &read(g, &c.receiver, "baseLevel")?)?;
            let open = read(g, &c.receiver, "open")?;
            let mut depth = numeric(g, &read(g, &open, "length")?)?;
            if depth > 1_000_000. {
                return Err(limit().into());
            }
            while depth > 0. {
                let candidate = read(g, &open, &number(depth - 1.))?;
                let old_level = numeric(g, &read(g, &candidate, "level")?)?;
                if old_level >= level {
                    depth -= 1.;
                } else {
                    break;
                }
            }
            let parent = read(g, &open, &number(depth - 1.))?;
            if !truthy(g, &parent) && !is_root {
                let descr = primitive(g, &arg(2))?;
                let mut e = invalid(&format!(
                    "There can be only one root. No parent could be found for (\"{descr}\")"
                ));
                e.code = ErrorCode::Semantic;
                return Err(e.into());
            }
            let children = node(
                g,
                Node::Array {
                    length: 0,
                    properties: vec![],
                },
            )?;
            let n = node(g, Node::Object { properties: vec![] })?;
            let id = read(g, &nodes, "length")?;
            for (key, value) in [
                ("id", id),
                ("nodeId", arg(1)),
                ("level", Atom::Number(level)),
                ("descr", arg(2)),
                ("type", arg(3)),
                ("children", children),
                ("isRoot", Atom::Bool(is_root)),
            ] {
                set(g, &n, key, value)?
            }
            if truthy(g, &parent) {
                let root = read(g, &parent, "isRoot")?;
                let section = if truthy(g, &root) {
                    let children = read(g, &parent, "children")?;
                    Atom::Number(numeric(g, &read(g, &children, "length")?)? % 11.)
                } else {
                    read(g, &parent, "section")?
                };
                set(g, &n, "section", section)?;
                let children = read(g, &parent, "children")?;
                push(g, children, n.clone(), "c.children.push is not a function")?;
            }
            set_length(g, &open, depth)?;
            push(g, open, n.clone(), "a.push is not a function")?;
            let nodes = read(g, &c.receiver, "nodes")?;
            push(g, nodes, n, "this.nodes.push is not a function")?;
        }
        _ => return Err(invalid("Unknown Mindmap public method").into()),
    }
    Ok(Atom::Undefined)
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
fn construct_default() -> Result<MindmapPublicState, PeleError> {
    Mindmap.create_public(&MindmapModel::empty(None))
}

impl PublicDiagramModule for Mindmap {
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

    fn function_shapes(&self) -> &'static [FunctionShape] {
        &[FunctionShape {
            method: GET_TYPE,
            constructible: true,
        }]
    }
    fn shared_functions(&self) -> &'static [SharedFunction] {
        &[SharedFunction {
            method: MethodId(0),
            identity: "pele/common/outline/getType@1",
        }]
    }

    type PublicState = MindmapPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(MindmapPublicState { graph, model })
    }
    fn create_public(&self, m: &MindmapModel) -> Result<MindmapPublicState, PeleError> {
        if m.nodes.len() > 1_000_000 {
            return Err(limit());
        }
        if m.nodes
            .iter()
            .any(|n| n.children.iter().any(|c| *c >= m.nodes.len()))
            || m.open.iter().any(|i| *i >= m.nodes.len())
        {
            return Err(invalid(
                "Mindmap typed model contains an invalid node reference",
            ));
        }
        let mut out = GraphWriter::default();
        let mut filled = vec![];
        let ids = m
            .nodes
            .iter()
            .map(|_| out.node(Node::Object { properties: vec![] }))
            .collect::<Vec<_>>();
        for (i, n) in m.nodes.iter().enumerate() {
            let children = export_array(
                &mut out,
                n.children.iter().map(|c| Atom::Ref(ids[*c])).collect(),
            );
            let mut values = HashMap::new();
            values.insert("id", Atom::Number(n.id as f64));
            values.insert("nodeId", Atom::String(out.string(n.node_id.as_str())));
            values.insert("level", Atom::Number(n.level as f64));
            values.insert("descr", Atom::String(out.string(n.descr.as_str())));
            values.insert("type", Atom::Number(n.node_type as f64));
            values.insert("children", children);
            values.insert("isRoot", Atom::Bool(n.is_root));
            if let Some(v) = n.section {
                values.insert("section", Atom::Number(v as f64));
            }
            for (key, v) in [("class", &n.class), ("icon", &n.icon)] {
                if let Some(v) = v {
                    values.insert(key, Atom::String(out.string(v.as_str())));
                }
            }
            let mut properties = vec![];
            for key in &n.public_property_order {
                if let Some(value) = values.remove(key.as_str()) {
                    properties.push(out.property(key, value));
                }
            }
            for key in [
                "id", "nodeId", "level", "descr", "type", "children", "isRoot", "section", "class",
                "icon",
            ] {
                if let Some(value) = values.remove(key) {
                    properties.push(out.property(key, value));
                }
            }
            filled.push((ids[i], Node::Object { properties }));
        }
        let nodes = export_array(&mut out, ids.iter().map(|i| Atom::Ref(*i)).collect());
        let open = export_array(
            &mut out,
            m.open.iter().map(|i| Atom::Ref(ids[*i])).collect(),
        );
        let kind = out.string("mindmap");
        let title = m
            .title
            .as_deref()
            .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
        let properties = vec![
            out.property("type", Atom::String(kind)),
            out.property("title", title),
            out.property("nodes", nodes),
            out.property("getType", Atom::BuiltinFunction(GET_TYPE.0)),
            out.property("baseLevel", Atom::Number(m.base_level as f64)),
            out.property("open", open),
        ];
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let mut graph = out.finish(Atom::Ref(root))?;
        for (id, node) in filled {
            graph.nodes[id as usize] = node;
        }
        graph.validate()?;
        let model = imported(&graph);
        Ok(MindmapPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &MindmapPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut MindmapPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut MindmapPublicState,
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
        s: &mut MindmapPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
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
