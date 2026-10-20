//! Mutable QuadrantDb own-field codec, Pele 458322a5 (MIT, kepano).
//! The graph owns identity and descriptors. Rendering projects concrete values;
//! class styles stay live while point overrides remain copied own properties.
use super::{PointStyle, QuadrantChart, QuadrantModel, QuadrantPoint, QuadrantText};
use crate::compat::{
    public::{
        Atom, FieldKind, FieldSchema, GraphWriter, JavaScriptErrorKind, JsString, MemberKind,
        MemberSchema, MethodCall, MethodId, MethodSchema, ModelGraph, ModelSchema, Node, Property,
        PrototypeId, PrototypeParent, PrototypeSchema, PublicDiagramModule, PublicError,
        PublicPeleError, RecordId, RecordKind, RecordSchema,
    },
    DiagramModule, DiagramType, ErrorCode, Object, PeleError, RenderResources, RenderSettings,
    Rendered,
};
use std::collections::HashMap;

const QUADRANT_PROTO: PrototypeId = PrototypeId(1);
// Native root construction is an explicit capability. Every real
// instance builder method below has pinned name, arity and flags6 placement.
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/quadrantChart@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[
        RecordSchema {
            id: RecordId(1),
            name: "QuadrantDb",
            kind: RecordKind::Object,
            prototype: Some(QUADRANT_PROTO),
            fields: &[
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
                    name: "quadrants",
                    kind: FieldKind::Array,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "xAxisLeft",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "xAxisRight",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "yAxisBottom",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "yAxisTop",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: true,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "points",
                    kind: FieldKind::Array,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "classes",
                    kind: FieldKind::Map,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(2),
            name: "QuadrantText",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                FieldSchema {
                    name: "text",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "type",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(3),
            name: "QuadrantPoint",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                FieldSchema {
                    name: "x",
                    kind: FieldKind::Number,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "y",
                    kind: FieldKind::Number,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "text",
                    kind: FieldKind::Record(RecordId(2)),
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "className",
                    kind: FieldKind::String,
                    optional: false,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "radius",
                    kind: FieldKind::Number,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "color",
                    kind: FieldKind::String,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "strokeColor",
                    kind: FieldKind::String,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "strokeWidth",
                    kind: FieldKind::String,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
        RecordSchema {
            id: RecordId(4),
            name: "PointStyle",
            kind: RecordKind::Object,
            prototype: None,
            fields: &[
                FieldSchema {
                    name: "radius",
                    kind: FieldKind::Number,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "color",
                    kind: FieldKind::String,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "strokeColor",
                    kind: FieldKind::String,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
                FieldSchema {
                    name: "strokeWidth",
                    kind: FieldKind::String,
                    optional: true,
                    undefined: false,
                    nullable: false,
                    flags: 7,
                },
            ],
            members: &[],
        },
    ],
    prototypes: &[PrototypeSchema {
        id: QUADRANT_PROTO,
        name: "QuadrantDb",
        parent: PrototypeParent::Object,
        members: &[
            MemberSchema {
                name: "setQuadrant1Text",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(1),
                },
            },
            MemberSchema {
                name: "setQuadrant2Text",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(2),
                },
            },
            MemberSchema {
                name: "setQuadrant3Text",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(3),
                },
            },
            MemberSchema {
                name: "setQuadrant4Text",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(4),
                },
            },
            MemberSchema {
                name: "setXAxisLeftText",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(5),
                },
            },
            MemberSchema {
                name: "setXAxisRightText",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(6),
                },
            },
            MemberSchema {
                name: "setYAxisBottomText",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(7),
                },
            },
            MemberSchema {
                name: "setYAxisTopText",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(8),
                },
            },
            MemberSchema {
                name: "addPoint",
                flags: 6,
                member: MemberKind::Method {
                    method: MethodId(9),
                },
            },
            MemberSchema {
                name: "addClass",
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
        ],
    }],
    methods: &[
        MethodSchema {
            id: MethodId(1),
            name: "setQuadrant1Text",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(2),
            name: "setQuadrant2Text",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(3),
            name: "setQuadrant3Text",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(4),
            name: "setQuadrant4Text",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(5),
            name: "setXAxisLeftText",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(6),
            name: "setXAxisRightText",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(7),
            name: "setYAxisBottomText",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(8),
            name: "setYAxisTopText",
            arity: 1,
        },
        MethodSchema {
            id: MethodId(9),
            name: "addPoint",
            arity: 5,
        },
        MethodSchema {
            id: MethodId(10),
            name: "addClass",
            arity: 2,
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
    ],
};
pub struct QuadrantPublicState {
    graph: ModelGraph,
    model: Result<QuadrantModel, PeleError>,
}
fn invalid(s: &str) -> PeleError {
    let mut e = PeleError::new(s, ErrorCode::Semantic);
    e.diagram_type = Some("quadrantChart".into());
    e
}
fn reference(a: &Atom) -> Result<u32, PeleError> {
    match a {
        Atom::Ref(id) => Ok(*id),
        _ => Err(invalid("Quadrant records must be objects")),
    }
}
fn field<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes
        .get(id as usize)?
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn text(g: &ModelGraph, a: &Atom) -> Result<String, PeleError> {
    g.text(a)?
        .well_formed()
        .ok_or_else(|| invalid("Quadrant rendering of lone UTF-16 surrogates is not implemented"))
}
fn optional(g: &ModelGraph, a: Option<&Atom>) -> Result<Option<String>, PeleError> {
    match a {
        None | Some(Atom::Null | Atom::Undefined) => Ok(None),
        Some(a) => text(g, a).map(Some),
    }
}
fn label(g: &ModelGraph, a: Option<&Atom>) -> Result<Option<QuadrantText>, PeleError> {
    let Some(a) = a.filter(|a| !matches!(a, Atom::Undefined | Atom::Null)) else {
        return Ok(None);
    };
    let id = reference(a)?;
    Ok(Some(QuadrantText {
        text: optional(g, field(g, id, "text"))?.unwrap_or_default(),
        label_type: optional(g, field(g, id, "type"))?.unwrap_or_default(),
    }))
}
fn array<'a>(g: &'a ModelGraph, a: &Atom) -> Result<Vec<Option<&'a Atom>>, PeleError> {
    let Some(Node::Array { length, properties }) = g.nodes.get(reference(a)? as usize) else {
        return Err(invalid("Quadrant collections must be Arrays"));
    };
    if *length > 1_000_000 {
        let mut e = invalid("Quadrant collection exceeds native work budget");
        e.code = ErrorCode::Limit;
        return Err(e);
    }
    let mut values = vec![None; *length as usize];
    for p in properties {
        let key = &g.strings[p.key as usize];
        if let Some(key) = key.well_formed() {
            if let Ok(at) = key.parse::<usize>() {
                if at < values.len() && at.to_string() == key && values[at].is_none() {
                    values[at] = Some(&p.value)
                }
            }
        }
    }
    Ok(values)
}
fn number(a: Option<&Atom>) -> f64 {
    match a {
        Some(Atom::Number(n)) => *n,
        Some(Atom::Null) => 0.,
        Some(Atom::Bool(b)) => {
            if *b {
                1.
            } else {
                0.
            }
        }
        _ => f64::NAN,
    }
}
fn style(g: &ModelGraph, id: u32) -> Result<PointStyle, PeleError> {
    let mut s = PointStyle::default();
    for p in g.nodes[id as usize].properties() {
        let key = &g.strings[p.key as usize];
        if key.is("radius") {
            if !matches!(p.value, Atom::Null | Atom::Undefined) {
                s.radius = Some(number(Some(&p.value)));
                s.order.push("radius")
            }
        }
        for (name, slot) in [
            ("color", &mut s.color),
            ("strokeColor", &mut s.stroke_color),
            ("strokeWidth", &mut s.stroke_width),
        ] {
            if key.is(name) {
                *slot = optional(g, Some(&p.value))?;
                if slot.is_some() {
                    s.order.push(name)
                }
            }
        }
    }
    Ok(s)
}
fn imported(g: &ModelGraph) -> Result<QuadrantModel, PeleError> {
    let qs = array(
        g,
        g.field("quadrants")?
            .ok_or_else(|| invalid("Quadrant quadrants must be an Array"))?,
    )?;
    let mut quadrants: [Option<QuadrantText>; 4] = Default::default();
    for (k, q) in quadrants.iter_mut().enumerate() {
        *q = label(g, qs.get(k).copied().flatten())?
    }
    let mut points = vec![];
    let mut classes = vec![];
    let mut styles: HashMap<u32, PointStyle> = HashMap::new();
    for a in array(
        g,
        g.field("points")?
            .ok_or_else(|| invalid("Quadrant points must be an Array"))?,
    )? {
        let id = reference(a.ok_or_else(|| invalid("Quadrant points contains a missing record"))?)?;
        let x = number(field(g, id, "x"));
        let y = number(field(g, id, "y"));
        // Upstream skips invalid coordinates before touching labels or the Map.
        if !(x >= 0. && x <= 1. && y >= 0. && y <= 1.) {
            points.push(QuadrantPoint {
                x,
                y,
                text: QuadrantText {
                    text: String::new(),
                    label_type: String::new(),
                },
                class_name: String::new(),
                style: PointStyle::default(),
            });
            continue;
        }
        let class_name = text(
            g,
            field(g, id, "className")
                .ok_or_else(|| invalid("Quadrant point className must be a string"))?,
        )?;
        let Some(Node::Map { entries, .. }) = g.field("classes")?.and_then(|a| {
            if let Atom::Ref(i) = a {
                g.nodes.get(*i as usize)
            } else {
                None
            }
        }) else {
            return Err(invalid("Quadrant classes must be a Map"));
        };
        // Native rendering needs only styles reachable by a visible point.
        // Detached Map values may be arbitrary application-owned objects.
        if !classes.iter().any(|(k, _)| k == &class_name) {
            if let Some((_, value)) = entries.iter().find(
                |(key, _)| matches!(key,Atom::String(k) if g.strings[*k as usize].is(&class_name)),
            ) {
                if !matches!(value, Atom::Undefined | Atom::Null) {
                    let sid = reference(value)?;
                    let s = if let Some(s) = styles.get(&sid) {
                        s.clone()
                    } else {
                        let s = style(g, sid)?;
                        styles.insert(sid, s.clone());
                        s
                    };
                    classes.push((class_name.clone(), s));
                }
            }
        }
        points.push(QuadrantPoint {
            x,
            y,
            text: label(g, field(g, id, "text"))?
                .ok_or_else(|| invalid("Quadrant point text must be an object"))?,
            class_name,
            style: style(g, id)?,
        });
    }
    Ok(QuadrantModel {
        diagram_type: DiagramType::QuadrantChart,
        title: optional(g, g.field("title")?)?,
        acc_title: optional(g, g.field("accTitle")?)?,
        acc_descr: optional(g, g.field("accDescr")?)?,
        quadrants,
        x_axis_left: label(g, g.field("xAxisLeft")?)?,
        x_axis_right: label(g, g.field("xAxisRight")?)?,
        y_axis_bottom: label(g, g.field("yAxisBottom")?)?,
        y_axis_top: label(g, g.field("yAxisTop")?)?,
        points,
        classes,
    })
}
fn label_atom(w: &mut GraphWriter, label: Option<&QuadrantText>) -> Atom {
    if let Some(label) = label {
        let text = w.string(label.text.as_str());
        let kind = w.string(label.label_type.as_str());
        let properties = vec![
            w.property("text", Atom::String(text)),
            w.property("type", Atom::String(kind)),
        ];
        Atom::Ref(w.node(Node::Object { properties }))
    } else {
        Atom::Undefined
    }
}
fn style_properties(w: &mut GraphWriter, s: &PointStyle) -> Vec<Property> {
    let mut keys = s.order.clone();
    for key in ["radius", "color", "strokeColor", "strokeWidth"] {
        if !keys.contains(&key) {
            keys.push(key)
        }
    }
    let mut properties = vec![];
    for key in keys {
        let a = match key {
            "radius" => s.radius.map(Atom::Number),
            "color" => s.color.as_ref().map(|s| Atom::String(w.string(s.as_str()))),
            "strokeColor" => s
                .stroke_color
                .as_ref()
                .map(|s| Atom::String(w.string(s.as_str()))),
            "strokeWidth" => s
                .stroke_width
                .as_ref()
                .map(|s| Atom::String(w.string(s.as_str()))),
            _ => None,
        };
        if let Some(a) = a {
            properties.push(w.property(key, a))
        }
    }
    properties
}
fn array_node(w: &mut GraphWriter, values: Vec<Atom>) -> u32 {
    let properties = values
        .into_iter()
        .enumerate()
        .map(|(i, a)| w.property(&i.to_string(), a))
        .collect::<Vec<_>>();
    w.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn construct_default() -> Result<QuadrantPublicState, PeleError> {
    QuadrantChart.create_public(&QuadrantModel::empty(None))
}

impl PublicDiagramModule for QuadrantChart {
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<crate::compat::public::PublicConstructorFactory<Self::PublicState>> {
        (prototype == QUADRANT_PROTO).then(|| {
            crate::compat::public::PublicConstructorFactory::ignoring_arguments::<Self>(
                QUADRANT_PROTO,
                construct_default,
            )
        })
    }

    type PublicState = QuadrantPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke_graph(&mut s.graph, call);
        // Methods may recover malformed models, or fail after a visible edit.
        // Import never runs source parsing or converts stored UTF-16 lossily.
        s.model = imported(&s.graph);
        result
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let model = imported(&graph);
        Ok(QuadrantPublicState { graph, model })
    }
    fn create_public(&self, m: &QuadrantModel) -> Result<Self::PublicState, PeleError> {
        let mut w = GraphWriter::default();
        let values = m
            .quadrants
            .iter()
            .map(|q| label_atom(&mut w, q.as_ref()))
            .collect();
        let qs = array_node(&mut w, values);
        let axes = [
            &m.x_axis_left,
            &m.x_axis_right,
            &m.y_axis_bottom,
            &m.y_axis_top,
        ]
        .map(|q| label_atom(&mut w, q.as_ref()));
        let mut points = vec![];
        for p in &m.points {
            let text = label_atom(&mut w, Some(&p.text));
            let class = w.string(p.class_name.as_str());
            let mut properties = vec![
                w.property("x", Atom::Number(p.x)),
                w.property("y", Atom::Number(p.y)),
                w.property("text", text),
                w.property("className", Atom::String(class)),
            ];
            properties.extend(style_properties(&mut w, &p.style));
            points.push(Atom::Ref(w.node(Node::Object { properties })));
        }
        let points = array_node(&mut w, points);
        let mut entries = vec![];
        for (name, s) in &m.classes {
            let key = Atom::String(w.string(name.as_str()));
            let properties = style_properties(&mut w, s);
            let value = Atom::Ref(w.node(Node::Object { properties }));
            entries.push((key, value));
        }
        let classes = w.node(Node::Map {
            entries,
            properties: vec![],
        });
        let kind = w.string("quadrantChart");
        let mut properties = vec![w.property("type", Atom::String(kind))];
        for (key, value) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let value = value
                .as_ref()
                .map_or(Atom::Undefined, |s| Atom::String(w.string(s.as_str())));
            properties.push(w.property(key, value));
        }
        properties.push(w.property("quadrants", Atom::Ref(qs)));
        for (key, a) in ["xAxisLeft", "xAxisRight", "yAxisBottom", "yAxisTop"]
            .into_iter()
            .zip(axes)
        {
            properties.push(w.property(key, a))
        }
        properties.push(w.property("points", Atom::Ref(points)));
        properties.push(w.property("classes", Atom::Ref(classes)));
        let root = w.node(Node::Instance {
            prototype: QUADRANT_PROTO,
            properties,
        });
        let graph = w.finish(Atom::Ref(root))?;
        let model = imported(&graph);
        Ok(QuadrantPublicState { graph, model })
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
        g.validate()?;
        s.model = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.model.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}

fn js_error(message: impl Into<JsString>) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn js_space(c: u16) -> bool {
    matches!(c,9..=13|32|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000|0xfeff)
}
fn trim_units(s: &[u16]) -> &[u16] {
    let start = s.iter().position(|c| !js_space(*c)).unwrap_or(s.len());
    let end = s
        .iter()
        .rposition(|c| !js_space(*c))
        .map_or(start, |i| i + 1);
    &s[start..end]
}
fn intern(g: &mut ModelGraph, value: JsString) -> u32 {
    if let Some(id) = g.strings.iter().position(|s| s == &value) {
        return id as u32;
    }
    let id = g.strings.len() as u32;
    g.strings.push(value);
    id
}
fn read(g: &ModelGraph, target: &Atom, key: &str) -> Result<Atom, PublicError> {
    match target {
        Atom::Undefined | Atom::Null => Err(js_error(JsString::from(
            format!(
                "Cannot read properties of {} (reading '{key}')",
                if matches!(target, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            )
            .as_str(),
        ))),
        Atom::Ref(id) => Ok(field(g, *id, key).cloned().unwrap_or(Atom::Undefined)),
        _ => Ok(Atom::Undefined),
    }
}
fn assignment(g: &ModelGraph, target: &Atom, key: &str) -> Result<u32, PublicError> {
    let id = match target {
        Atom::Ref(id) => *id,
        Atom::Undefined | Atom::Null => {
            return Err(js_error(JsString::from(
                format!(
                    "Cannot set properties of {} (setting '{key}')",
                    if matches!(target, Atom::Null) {
                        "null"
                    } else {
                        "undefined"
                    }
                )
                .as_str(),
            )))
        }
        _ => {
            return Err(js_error(JsString::from(
                format!("Cannot create property '{key}' on a primitive receiver").as_str(),
            )))
        }
    };
    let n = &g.nodes[id as usize];
    if let Some(p) = n
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
    {
        if p.flags & 2 == 0 {
            return Err(js_error(JsString::from(
                format!("Cannot assign to read only property '{key}'").as_str(),
            )));
        }
    }
    if let Node::Array { length, .. } = n {
        if key
            .parse::<u32>()
            .is_ok_and(|at| at >= *length && at.to_string() == key && at < u32::MAX)
        {
            if n.properties()
                .iter()
                .any(|p| g.strings[p.key as usize].is("length") && p.flags & 2 == 0)
            {
                return Err(js_error("Cannot assign to read only property 'length'"));
            }
        }
    }
    Ok(id)
}
fn assign(g: &mut ModelGraph, target: &Atom, key: &str, value: Atom) -> Result<(), PublicError> {
    let id = assignment(g, target, key)?;
    let at = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key));
    if let Some(at) = at {
        g.nodes[id as usize].properties_mut()[at].value = value
    } else {
        let key = intern(g, JsString::from(key));
        g.nodes[id as usize].properties_mut().push(Property {
            key,
            value,
            flags: 7,
        })
    }
    if let Node::Array { length, properties } = &mut g.nodes[id as usize] {
        if let Ok(at) = key.parse::<u32>() {
            if at.to_string() == key && at >= *length && at < u32::MAX {
                *length = at + 1;
                for p in properties {
                    if g.strings[p.key as usize].is("length") {
                        p.value = Atom::Number(f64::from(*length))
                    }
                }
            }
        }
    }
    Ok(())
}
fn string_units<'a>(
    g: &'a ModelGraph,
    value: &Atom,
    method: &str,
    local: &str,
) -> Result<&'a [u16], PublicError> {
    match value {
        Atom::String(id) => Ok(&g.strings[*id as usize].0),
        Atom::Null | Atom::Undefined => Err(js_error(JsString::from(
            format!(
                "Cannot read properties of {} (reading '{method}')",
                if matches!(value, Atom::Null) {
                    "null"
                } else {
                    "undefined"
                }
            )
            .as_str(),
        ))),
        _ => Err(js_error(JsString::from(
            format!("{local}.{method} is not a function").as_str(),
        ))),
    }
}
struct TrimmedLabel {
    text: JsString,
    kind: Atom,
}
fn trimmed(g: &ModelGraph, a: &Atom) -> Result<TrimmedLabel, PublicError> {
    let raw = read(g, a, "text")?;
    let text = JsString(trim_units(string_units(g, &raw, "trim", "e.text")?).to_vec());
    Ok(TrimmedLabel {
        text,
        kind: read(g, a, "type")?,
    })
}
fn commit_label(g: &mut ModelGraph, label: TrimmedLabel) -> Atom {
    let text = intern(g, label.text);
    let text_key = intern(g, JsString::from("text"));
    let type_key = intern(g, JsString::from("type"));
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object {
        properties: vec![
            Property {
                key: text_key,
                value: Atom::String(text),
                flags: 7,
            },
            Property {
                key: type_key,
                value: label.kind,
                flags: 7,
            },
        ],
    });
    Atom::Ref(id)
}
fn numeric(g: &ModelGraph, a: &Atom) -> f64 {
    let Atom::String(id) = a else {
        return match a {
            Atom::Ref(id) => match &g.nodes[*id as usize] {
                Node::Date { milliseconds, .. } => *milliseconds,
                _ => f64::NAN,
            },
            _ => number(Some(a)),
        };
    };
    let units = trim_units(&g.strings[*id as usize].0);
    if units.is_empty() {
        return 0.;
    }
    let Ok(s) = String::from_utf16(units) else {
        return f64::NAN;
    };
    let radix = if s.starts_with("0x") || s.starts_with("0X") {
        Some((16, 4))
    } else if s.starts_with("0b") || s.starts_with("0B") {
        Some((2, 1))
    } else if s.starts_with("0o") || s.starts_with("0O") {
        Some((8, 3))
    } else {
        None
    };
    if let Some((radix, width)) = radix {
        if s.len() == 2 {
            return f64::NAN;
        }
        let (mut bits, mut mant, mut round, mut sticky) = (0usize, 0u64, false, false);
        for c in s[2..].chars() {
            let Some(digit) = c.to_digit(radix) else {
                return f64::NAN;
            };
            for k in (0..width).rev() {
                let bit = (digit >> k) & 1;
                if bits == 0 && bit == 0 {
                    continue;
                }
                bits += 1;
                if bits <= 53 {
                    mant = (mant << 1) | u64::from(bit)
                } else if bits == 54 {
                    round = bit != 0
                } else {
                    sticky |= bit != 0
                }
            }
        }
        return if bits <= 53 {
            mant as f64
        } else if bits > 1024 {
            f64::INFINITY
        } else {
            if round && (sticky || mant & 1 != 0) {
                mant += 1
            }
            (mant as f64) * 2_f64.powi((bits - 53) as i32)
        };
    }
    match s.as_str() {
        "Infinity" | "+Infinity" => f64::INFINITY,
        "-Infinity" => f64::NEG_INFINITY,
        _ if s
            .chars()
            .any(|c| !matches!(c, '0'..='9' | '+' | '-' | '.' | 'e' | 'E')) =>
        {
            f64::NAN
        }
        _ => s.parse().unwrap_or(f64::NAN),
    }
}
enum StyleValue {
    Number(f64),
    Text(JsString),
}
fn semantic(parts: &[&[u16]]) -> PublicError {
    let message = JsString(parts.concat());
    if let Some(message) = message.well_formed() {
        return invalid(&message).into();
    }
    PublicPeleError {
        message,
        code: ErrorCode::Semantic,
        diagram_type: Some(JsString::from("quadrantChart")),
        line: 0,
        column: 0,
        snippet: JsString(vec![]),
    }
    .into()
}
fn parse_style(
    g: &ModelGraph,
    a: &Atom,
    out: &mut Vec<(&'static str, StyleValue)>,
) -> Result<(), PublicError> {
    let s = trim_units(string_units(g, a, "trim", "e")?);
    let colon = s.iter().position(|c| *c == 58);
    let (key, value) = if let Some(colon) = colon {
        let key = &s[..colon];
        let key = &key[..key.iter().rposition(|c| !js_space(*c)).map_or(0, |i| i + 1)];
        let rest = &s[colon + 1..];
        let start = rest
            .iter()
            .position(|c| !js_space(*c))
            .unwrap_or(rest.len());
        let rest = &rest[start..];
        let value = if let Some(next) = rest.iter().position(|c| *c == 58) {
            let r = &rest[..next];
            &r[..r.iter().rposition(|c| !js_space(*c)).map_or(0, |i| i + 1)]
        } else {
            rest
        };
        (key, value)
    } else {
        (s, &[117, 110, 100, 101, 102, 105, 110, 101, 100][..])
    };
    let key_string = JsString(key.to_vec());
    let digit = |s: &[u16]| !s.is_empty() && s.iter().all(|c| matches!(c, 48..=57));
    let (field, valid, kind) = if key_string.is("radius") {
        ("radius", digit(value), "number")
    } else if key_string.is("color") || key_string.is("stroke-color") {
        let hex = if value.first() == Some(&35) {
            &value[1..]
        } else {
            value
        };
        (
            if key_string.is("color") {
                "color"
            } else {
                "strokeColor"
            },
            matches!(hex.len(), 3 | 6) && hex.iter().all(|c| matches!(c,48..=57|65..=70|97..=102)),
            "hex code",
        )
    } else if key_string.is("stroke-width") {
        (
            "strokeWidth",
            value.strip_suffix(&[112, 120]).is_some_and(digit),
            "number of pixels (eg. 10px)",
        )
    } else {
        return Err(semantic(&[
            &JsString::from("style named ").0,
            key,
            &JsString::from(" is not supported.").0,
        ]));
    };
    if !valid {
        return Err(semantic(&[
            &JsString::from("value for ").0,
            key,
            &[32],
            value,
            &JsString::from(format!(" is invalid, please use a valid {kind}").as_str()).0,
        ]));
    }
    let value = if field == "radius" {
        StyleValue::Number(
            String::from_utf16(value)
                .unwrap()
                .parse()
                .unwrap_or(f64::INFINITY),
        )
    } else {
        StyleValue::Text(JsString(value.to_vec()))
    };
    if let Some(old) = out.iter_mut().find(|(key, _)| *key == field) {
        old.1 = value
    } else {
        out.push((field, value))
    }
    Ok(())
}
fn parse_styles(g: &ModelGraph, a: &Atom) -> Result<Vec<(&'static str, StyleValue)>, PublicError> {
    let mut out = vec![];
    match a {
        Atom::Ref(id) => match &g.nodes[*id as usize] {
            Node::Array { .. } => {
                for value in array(g, a)? {
                    parse_style(g, value.unwrap_or(&Atom::Undefined), &mut out)?
                }
            }
            Node::Set { values, .. } => {
                for value in values {
                    parse_style(g, value, &mut out)?
                }
            }
            Node::Map { entries, .. } if entries.is_empty() => {}
            Node::Map { .. } => return Err(js_error("e.trim is not a function")),
            _ => return Err(js_error("e is not iterable")),
        },
        Atom::String(id) => {
            let s = &g.strings[*id as usize].0;
            if !s.is_empty() {
                let n = if s.len() > 1
                    && matches!(s[0], 0xd800..=0xdbff)
                    && matches!(s[1], 0xdc00..=0xdfff)
                {
                    2
                } else {
                    1
                };
                return Err(semantic(&[
                    &JsString::from("style named ").0,
                    &s[..n],
                    &JsString::from(" is not supported.").0,
                ]));
            }
        }
        _ => return Err(js_error("e is not iterable")),
    }
    Ok(out)
}
fn commit_style(g: &mut ModelGraph, styles: Vec<(&str, StyleValue)>) -> Vec<Property> {
    styles
        .into_iter()
        .map(|(key, v)| {
            let value = match v {
                StyleValue::Number(n) => Atom::Number(n),
                StyleValue::Text(s) => Atom::String(intern(g, s)),
            };
            Property {
                key: intern(g, JsString::from(key)),
                value,
                flags: 7,
            }
        })
        .collect()
}
fn same(g: &ModelGraph, a: &Atom, b: &Atom) -> bool {
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
fn room(
    g: &ModelGraph,
    nodes: usize,
    values: &[&[u16]],
    references: usize,
) -> Result<(), PublicError> {
    let mut added: Vec<&[u16]> = vec![];
    for value in values {
        if !g.strings.iter().any(|s| s.0 == **value) && !added.contains(value) {
            added.push(value)
        }
    }
    let units = g
        .strings
        .iter()
        .fold(0usize, |n, s| n.saturating_add(s.0.len()))
        .saturating_add(added.iter().map(|s| s.len()).sum::<usize>());
    let work = g.nodes.iter().fold(0usize, |n, node| {
        n.saturating_add(node.properties().len())
            .saturating_add(match node {
                Node::Map { entries, .. } => entries.len().saturating_mul(2),
                Node::Set { values, .. } => values.len(),
                _ => 0,
            })
    });
    if g.nodes.len().saturating_add(nodes) > 1_000_000
        || g.strings.len().saturating_add(added.len()) > 1_000_000
        || units > 16_000_000
        || work.saturating_add(references) > 8_000_000
    {
        return Err(PeleError::new(
            "Quadrant method exceeds native graph work budget",
            ErrorCode::Limit,
        )
        .into());
    }
    Ok(())
}
fn label_room(g: &ModelGraph, label: &TrimmedLabel, key: &str) -> Result<(), PublicError> {
    let names = [
        JsString::from("text"),
        JsString::from("type"),
        JsString::from(key),
    ];
    room(
        g,
        1,
        &[&label.text.0, &names[0].0, &names[1].0, &names[2].0],
        3,
    )
}
fn style_room(
    g: &ModelGraph,
    styles: &[(&str, StyleValue)],
    label: Option<&TrimmedLabel>,
    index: Option<&str>,
    new_entry: bool,
) -> Result<(), PublicError> {
    let names = ["text", "type", "x", "y", "className"].map(JsString::from);
    let keys = styles
        .iter()
        .map(|(k, _)| JsString::from(*k))
        .collect::<Vec<_>>();
    let index = index.map(JsString::from);
    let mut values = keys.iter().map(|s| s.0.as_slice()).collect::<Vec<_>>();
    for (_, v) in styles {
        if let StyleValue::Text(s) = v {
            values.push(&s.0)
        }
    }
    if let Some(label) = label {
        values.push(&label.text.0);
        values.extend(names.iter().map(|s| s.0.as_slice()))
    }
    if let Some(index) = &index {
        values.push(&index.0)
    }
    room(
        g,
        if label.is_some() { 2 } else { 1 },
        &values,
        styles.len()
            + if label.is_some() {
                7
            } else if new_entry {
                2
            } else {
                0
            },
    )
}
fn invoke_graph(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let arg = |i| call.arguments.get(i).cloned().unwrap_or(Atom::Undefined);
    let receiver = &call.receiver;
    match call.method.0 {
        1..=4 => {
            let target = read(g, receiver, "quadrants")?;
            let label = trimmed(g, &arg(0))?;
            let key = (call.method.0 - 1).to_string();
            assignment(g, &target, &key)?;
            label_room(g, &label, &key)?;
            let label = commit_label(g, label);
            assign(g, &target, &key, label)?;
        }
        5..=8 => {
            let key = ["xAxisLeft", "xAxisRight", "yAxisBottom", "yAxisTop"]
                [(call.method.0 - 5) as usize];
            let label = trimmed(g, &arg(0))?;
            assignment(g, receiver, key)?;
            label_room(g, &label, key)?;
            let label = commit_label(g, label);
            assign(g, receiver, key, label)?;
        }
        9 => {
            let points = read(g, receiver, "points")?;
            let push = read(g, &points, "push")?;
            let x = numeric(g, &arg(2));
            let y = numeric(g, &arg(3));
            let label = trimmed(g, &arg(0))?;
            let styles = parse_styles(g, &arg(4))?;
            let id = if let Atom::Ref(id) = points {
                id
            } else {
                return Err(js_error("this.points.push is not a function"));
            };
            if matches!(push, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
                return Err(
                    invalid("Quadrant overridden push callback dispatch is unavailable").into(),
                );
            }
            let length = match &g.nodes[id as usize] {
                Node::Array { length, .. }
                    if matches!(push, Atom::Undefined) && field(g, id, "push").is_none() =>
                {
                    *length
                }
                _ => return Err(js_error("this.points.push is not a function")),
            };
            if length >= 1_000_000 {
                return Err(PeleError::new(
                    "Quadrant point collection exceeds native work budget",
                    ErrorCode::Limit,
                )
                .into());
            }
            let key = length.to_string();
            assignment(g, &points, &key)?;
            style_room(g, &styles, Some(&label), Some(&key), false)?;
            let text = commit_label(g, label);
            let fields = [
                ("x", Atom::Number(x)),
                ("y", Atom::Number(y)),
                ("text", text),
                ("className", arg(1)),
            ];
            let mut properties = fields
                .into_iter()
                .map(|(key, value)| Property {
                    key: intern(g, JsString::from(key)),
                    value,
                    flags: 7,
                })
                .collect::<Vec<_>>();
            properties.extend(commit_style(g, styles));
            let point = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties });
            assign(g, &points, &key, Atom::Ref(point))?;
        }
        10 => {
            let classes = read(g, receiver, "classes")?;
            let set = read(g, &classes, "set")?;
            let styles = parse_styles(g, &arg(1))?;
            let id = if let Atom::Ref(id) = classes {
                id
            } else {
                return Err(js_error("this.classes.set is not a function"));
            };
            if matches!(set, Atom::HostReference(_) | Atom::BuiltinFunction(_)) {
                return Err(
                    invalid("Quadrant overridden set callback dispatch is unavailable").into(),
                );
            }
            if !matches!(g.nodes[id as usize], Node::Map { .. })
                || !matches!(set, Atom::Undefined)
                || field(g, id, "set").is_some()
            {
                return Err(js_error("this.classes.set is not a function"));
            }
            let class = arg(0);
            let existing = if let Node::Map { entries, .. } = &g.nodes[id as usize] {
                entries.iter().position(|(key, _)| same(g, key, &class))
            } else {
                unreachable!()
            };
            style_room(g, &styles, None, None, existing.is_none())?;
            let properties = commit_style(g, styles);
            let style = g.nodes.len() as u32;
            g.nodes.push(Node::Object { properties });
            if let Node::Map { entries, .. } = &mut g.nodes[id as usize] {
                if let Some(at) = existing {
                    entries[at].1 = Atom::Ref(style)
                } else {
                    entries.push((class, Atom::Ref(style)))
                }
            }
        }
        11 => {
            assignment(g, receiver, "title")?;
            room(g, 0, &[&JsString::from("title").0], 1)?;
            assign(g, receiver, "title", arg(0))?;
        }
        12 | 13 => {
            let raw = arg(0);
            let units = string_units(g, &raw, "replace", "t")?;
            let value = if call.method.0 == 12 {
                let start = units
                    .iter()
                    .position(|c| !js_space(*c))
                    .unwrap_or(units.len());
                units[start..].to_vec()
            } else {
                let mut out = vec![];
                let mut i = 0;
                while i < units.len() {
                    let c = units[i];
                    out.push(c);
                    i += 1;
                    if c == 10 && i < units.len() && js_space(units[i]) {
                        while i < units.len() && js_space(units[i]) {
                            i += 1
                        }
                    }
                }
                out
            };
            let key = if call.method.0 == 12 {
                "accTitle"
            } else {
                "accDescr"
            };
            assignment(g, receiver, key)?;
            room(g, 0, &[&value, &JsString::from(key).0], 1)?;
            let value = intern(g, JsString(value));
            assign(g, receiver, key, Atom::String(value))?;
        }
        _ => return Err(invalid("Quadrant public method is unavailable").into()),
    }
    Ok(Atom::Undefined)
}
