//! Native mutable KanbanDb, Pele 458322a5 (MIT kepano). No diagram logic in JS.
//! Arbitrary prototype replacement/coercion and active lone UTF-16
//! rendering remain explicit gaps; the arena itself preserves all code units.
use super::{Kanban, KanbanModel, KanbanNode, KanbanSection};
use crate::compat::{
    public::*, source::yaml, DiagramModule, DiagramType, ErrorCode, Object, PeleError,
    RenderResources, RenderSettings, Rendered,
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
    id: "pele/kanban@1",
    version: 1,
    root: Some(RecordId(0)),
    records: &[
        RecordSchema {
            id: RecordId(0),
            name: "Kanban",
            kind: RecordKind::Object,
            prototype: Some(PROTO),
            fields: &[
                field_schema("type", FieldKind::String, false, false),
                field_schema("title", FieldKind::String, false, true),
                field_schema("sections", FieldKind::Array, false, false),
                field_schema("last", FieldKind::Object, false, true),
                field_schema("sectionLevel", FieldKind::Number, false, false),
                field_schema("misplaced", FieldKind::Object, false, true),
            ],
            members: &[MemberSchema {
                name: "getType",
                flags: 7,
                member: MemberKind::Method { method: GET_TYPE },
            }],
        },
        RecordSchema {
            id: RecordId(1),
            name: "KanbanNode",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                field_schema("id", FieldKind::String, false, false),
                field_schema("level", FieldKind::Number, false, false),
                field_schema("label", FieldKind::String, false, false),
                field_schema("icon", FieldKind::String, true, false),
                field_schema("assigned", FieldKind::String, true, false),
                field_schema("ticket", FieldKind::String, true, false),
                field_schema("priority", FieldKind::String, true, false),
                field_schema("cssClasses", FieldKind::String, true, false),
                field_schema("items", FieldKind::Array, true, false),
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: PROTO,
        name: "KanbanDb",
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
            arity: 5,
        },
        MethodSchema {
            id: DECORATE,
            name: "decorateNode",
            arity: 1,
        },
    ],
};
pub struct KanbanPublicState {
    graph: ModelGraph,
    model: Result<KanbanModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    super::error(s.into(), ErrorCode::Semantic)
}
fn limit() -> PeleError {
    super::error(
        "Kanban public operation exceeds native work budget".into(),
        ErrorCode::Limit,
    )
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
            invalid("Kanban runtime error text containing lone UTF-16 is not implemented").into()
        }),
        _ => Err(invalid("Kanban arbitrary object coercion is not implemented").into()),
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
fn same(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
    match (a, b) {
        (Atom::Undefined, Atom::Undefined) | (Atom::Null, Atom::Null) => true,
        (Atom::Number(a), Atom::Number(b)) => a == b,
        (Atom::String(a), Atom::String(b)) => g.strings[*a as usize] == g.strings[*b as usize],
        (Atom::Bool(a), Atom::Bool(b)) => a == b,
        (Atom::Ref(a), Atom::Ref(b)) => a == b,
        _ => false,
    }
}
fn less(g: &ModelGraph, a: &Atom, b: &Atom) -> Result<bool, PublicError> {
    match (a, b) {
        (Atom::Number(a), Atom::Number(b)) => Ok(a < b),
        (Atom::String(a), Atom::String(b)) => {
            Ok(g.strings[*a as usize].0 < g.strings[*b as usize].0)
        }
        _ => Err(invalid("Kanban mixed-type indentation coercion is not implemented").into()),
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
        _ => return Err(invalid("Kanban borrowed host receiver is not implemented").into()),
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
        return Err(invalid("Kanban active collections must be Arrays"));
    };
    let Node::Array { length, properties } = &g.nodes[*id as usize] else {
        return Err(invalid("Kanban active collections must be Arrays"));
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
        return Err(invalid("Kanban sparse active arrays are not implemented"));
    }
    Ok((0..*length).map(|i| values.remove(&i).unwrap()).collect())
}
struct Text(usize);
impl Text {
    fn text(
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
                "Kanban rendering of non-string active text is not implemented",
            ));
        };
        let units = &g.strings[*id as usize];
        self.0 = self.0.checked_add(units.0.len()).ok_or_else(limit)?;
        if self.0 > 16_000_000 {
            return Err(limit());
        }
        units
            .well_formed()
            .map(Some)
            .ok_or_else(|| invalid("Kanban rendering of lone UTF-16 surrogates is not implemented"))
    }
}
fn imported_node(g: &ModelGraph, a: &Atom, text: &mut Text) -> Result<KanbanNode, PeleError> {
    let Atom::Ref(id) = a else {
        return Err(invalid("Kanban node must be an object"));
    };
    let required = |key| property(g, *id, key);
    Ok(KanbanNode {
        id: text.text(g, required("id"), false)?.unwrap(),
        label: text.text(g, required("label"), true)?.unwrap_or_default(),
        level: 0,
        icon: match required("icon") {
            Some(a) if !truthy(g, a) => None,
            a => text.text(g, a, true)?,
        },
        assigned: text.text(g, required("assigned"), true)?,
        ticket: text.text(g, required("ticket"), true)?,
        priority: match required("priority") {
            Some(Atom::Null) => None,
            a => text.text(g, a, true)?,
        },
        css_classes: match required("cssClasses") {
            Some(Atom::Null) => None,
            a => text.text(g, a, true)?,
        },
        public_property_order: g.nodes[*id as usize]
            .properties()
            .iter()
            .filter_map(|p| g.strings[p.key as usize].well_formed())
            .collect(),
    })
}
fn imported(g: &ModelGraph) -> Result<KanbanModel, PeleError> {
    let Atom::Ref(root) = g.root else {
        return Err(invalid("Kanban root must be an object"));
    };
    let mut text = Text(0);
    let mut sections = vec![];
    for section in array(g, property(g, root, "sections").unwrap_or(&Atom::Undefined))? {
        let id = if let Atom::Ref(id) = section {
            id
        } else {
            return Err(invalid("Kanban section must be an object"));
        };
        let mut items = vec![];
        for a in array(g, property(g, id, "items").unwrap_or(&Atom::Undefined))? {
            items.push(imported_node(g, &a, &mut text)?)
        }
        sections.push(KanbanSection {
            node: imported_node(g, &section, &mut text)?,
            items,
        });
    }
    Ok(KanbanModel {
        diagram_type: DiagramType::Kanban,
        title: text.text(g, property(g, root, "title"), true)?,
        sections,
        last: None,
        section_level: 0,
        misplaced: None,
    })
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
            let decoration = arg(0);
            let last = read(g, &c.receiver, "last")?;
            if !truthy(g, &decoration) || !truthy(g, &last) {
                return Ok(Atom::Undefined);
            }
            let icon = read(g, &decoration, "icon")?;
            if truthy(g, &icon) {
                set(g, &last, "icon", icon)?;
            }
            let class = read(g, &decoration, "class")?;
            if truthy(g, &class) {
                set(g, &last, "cssClasses", class)?;
            }
        }
        ADD_NODE => {
            let n = node(g, Node::Object { properties: vec![] })?;
            for (key, a) in [("id", arg(1)), ("level", arg(0)), ("label", arg(2))] {
                set(g, &n, key, a)?;
            }
            let data = arg(4);
            if !matches!(data, Atom::Undefined) {
                let raw = match &data {
                    Atom::String(id) => g.strings[*id as usize].well_formed().ok_or_else(|| {
                        invalid("Kanban YAML metadata containing lone UTF-16 is not implemented")
                    })?,
                    Atom::Null => {
                        return Err(type_error(
                            "Cannot read properties of null (reading 'includes')",
                        ))
                    }
                    _ => return Err(type_error("s.includes is not a function")),
                };
                if raw.len() > 16_000_000 {
                    return Err(limit().into());
                }
                let raw = if raw.contains('\n') {
                    format!("{raw}\n")
                } else {
                    format!("{{\n{raw}\n}}")
                };
                let doc = yaml::parse(&raw).map_err(|e| {
                    primitive(g, &arg(1)).map(|id| {
                        super::error(
                            format!("Metadata of \"{id}\" is not valid YAML. {e}"),
                            ErrorCode::Syntax,
                        )
                    })
                });
                let doc = match doc {
                    Ok(v) => v,
                    Err(e) => return Err(e?.into()),
                };
                if let Some(doc) = doc.as_object() {
                    let shape = doc
                        .get("shape")
                        .filter(|v| v.truthy())
                        .map(|v| v.js_string())
                        .unwrap_or_default();
                    if shape != shape.to_lowercase() || shape.contains('_') {
                        return Err(super::error(
                            format!("No such shape: {shape}. Shape names should be lowercase."),
                            ErrorCode::Semantic,
                        )
                        .into());
                    }
                    for key in ["label", "icon", "assigned", "ticket", "priority"] {
                        if let Some(value) = doc.get(key).filter(|v| v.truthy()) {
                            let value = Atom::String(intern(g, value.js_string().as_str().into())?);
                            set(g, &n, key, value)?;
                        }
                    }
                }
            }
            let misplaced = read(g, &c.receiver, "misplaced")?;
            if truthy(g, &misplaced) {
                let label = primitive(g, &read(g, &misplaced, "label")?)?;
                return Err(super::error(
                    format!("Items without section detected, found section (\"{label}\")"),
                    ErrorCode::Semantic,
                )
                .into());
            }
            let sections = read(g, &c.receiver, "sections")?;
            let length = read(g, &sections, "length")?;
            if matches!(length,Atom::Number(n)if n==0.) {
                set(g, &c.receiver, "sectionLevel", arg(0))?;
            }
            set(g, &c.receiver, "last", n.clone())?;
            let level = read(g, &c.receiver, "sectionLevel")?;
            if same(g, &arg(0), &level) {
                let items = node(
                    g,
                    Node::Array {
                        length: 0,
                        properties: vec![],
                    },
                )?;
                set(g, &n, "items", items)?;
                push(g, sections, n, "i.push is not a function")?;
            } else {
                let key = match length {
                    Atom::Number(n) => number(n - 1.),
                    Atom::Undefined => "NaN".into(),
                    _ => {
                        return Err(
                            invalid("Kanban array-like length coercion is not implemented").into(),
                        )
                    }
                };
                let section = read(g, &sections, &key)?;
                let items = read(g, &section, "items")?;
                push(
                    g,
                    items,
                    n.clone(),
                    "i[(i.length - 1)].items.push is not a function",
                )?;
                if less(g, &arg(0), &level)? {
                    set(g, &c.receiver, "misplaced", n)?;
                }
            }
        }
        _ => return Err(invalid("Unknown Kanban public method").into()),
    }
    Ok(Atom::Undefined)
}
fn export_node(out: &mut GraphWriter, n: &KanbanNode, items: Option<Atom>) -> Atom {
    let mut values = HashMap::new();
    values.insert("id", Atom::String(out.string(n.id.as_str())));
    values.insert("level", Atom::Number(n.level as f64));
    values.insert("label", Atom::String(out.string(n.label.as_str())));
    for (key, v) in [
        ("icon", &n.icon),
        ("assigned", &n.assigned),
        ("ticket", &n.ticket),
        ("priority", &n.priority),
        ("cssClasses", &n.css_classes),
    ] {
        if let Some(v) = v {
            values.insert(key, Atom::String(out.string(v.as_str())));
        }
    }
    if let Some(items) = items {
        values.insert("items", items);
    }
    let mut properties = vec![];
    for key in &n.public_property_order {
        if let Some(value) = values.remove(key.as_str()) {
            properties.push(out.property(key, value));
        }
    } // User-built typed models can omit parser provenance; stable declared-field fallback.
    for key in [
        "id",
        "level",
        "label",
        "icon",
        "assigned",
        "ticket",
        "priority",
        "cssClasses",
        "items",
    ] {
        if let Some(value) = values.remove(key) {
            properties.push(out.property(key, value));
        }
    }
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
fn construct_default() -> Result<KanbanPublicState, PeleError> {
    Kanban.create_public(&KanbanModel::empty(None))
}

impl PublicDiagramModule for Kanban {
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

    type PublicState = KanbanPublicState;
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::kanban())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        Some(DetachedPublicRender::kanban(
            super::KanbanDetachedInput::new(state.model.as_ref().err().cloned()),
        ))
    }
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(KanbanPublicState { graph, model })
    }
    fn create_public(&self, m: &KanbanModel) -> Result<KanbanPublicState, PeleError> {
        let mut out = GraphWriter::default();
        let mut sections = vec![];
        let mut ids = vec![];
        for section in &m.sections {
            let items = section
                .items
                .iter()
                .map(|n| export_node(&mut out, n, None))
                .collect::<Vec<_>>();
            let item_ids = items.clone();
            let items = export_array(&mut out, items);
            let s = export_node(&mut out, &section.node, Some(items));
            ids.push((s.clone(), item_ids));
            sections.push(s);
        }
        let last = m
            .last
            .as_ref()
            .map(|(s, i)| i.map_or_else(|| ids[*s].0.clone(), |i| ids[*s].1[i].clone()))
            .unwrap_or(Atom::Undefined);
        let misplaced = m
            .misplaced
            .as_ref()
            .map(|(s, i)| ids[*s].1[*i].clone())
            .unwrap_or(Atom::Undefined);
        let sections = export_array(&mut out, sections);
        let kind = out.string("kanban");
        let title = m
            .title
            .as_deref()
            .map_or(Atom::Undefined, |s| Atom::String(out.string(s)));
        let properties = vec![
            out.property("type", Atom::String(kind)),
            out.property("title", title),
            out.property("sections", sections),
            out.property("getType", Atom::BuiltinFunction(GET_TYPE.0)),
            out.property("last", last),
            out.property("sectionLevel", Atom::Number(m.section_level as f64)),
            out.property("misplaced", misplaced),
        ];
        let root = out.node(Node::Instance {
            prototype: PROTO,
            properties,
        });
        let graph = out.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(KanbanPublicState { graph, model })
    }
    fn borrow_public_graph<'a>(&self, state: &'a Self::PublicState) -> Option<&'a ModelGraph> {
        Some(&state.graph)
    }
    fn unchanged_public_graph_is_noop(&self) -> bool {
        true
    }
    fn export_public(&self, s: &KanbanPublicState) -> Result<ModelGraph, PeleError> {
        Ok(s.graph.clone())
    }
    fn apply_public(&self, s: &mut KanbanPublicState, g: ModelGraph) -> Result<(), PeleError> {
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut KanbanPublicState,
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
        s: &mut KanbanPublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
