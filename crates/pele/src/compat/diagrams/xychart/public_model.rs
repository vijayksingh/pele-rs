//! Mutable XyChartDb codec and native builders, Pele 458322a5 (MIT, kepano).
use super::{LinearAxis, Orientation, Plot, PlotType, XAxis, XyChart, XyChartModel};
use crate::compat::{
    public::*, source::space, DiagramModule, DiagramType, ErrorCode, Object, PeleError,
    RenderResources, RenderSettings, Rendered,
};
use std::collections::HashMap;
const PROTOTYPE: PrototypeId = PrototypeId(1);
const MAX_ITEMS: usize = 1_000_000;
const MAX_EXPANSION: usize = 8_000_000;
const MAX_TEXT_UNITS: usize = 16_000_000;
fn count_text(g: &ModelGraph, a: Option<&Atom>, units: &mut usize) -> Result<(), PeleError> {
    if let Some(Atom::String(id)) = a {
        *units = units
            .checked_add(g.strings[*id as usize].0.len())
            .ok_or_else(limit)?;
        if *units > MAX_TEXT_UNITS {
            return Err(limit());
        }
    }
    Ok(())
}
fn text_projection_budget(g: &ModelGraph, root: u32) -> Result<(), PeleError> {
    let mut units = 0;
    for key in ["title", "accTitle", "accDescr"] {
        count_text(g, value(g, root, key), &mut units)?
    }
    for key in ["xAxis", "yAxis"] {
        let id = object(value(g, root, key))?;
        count_text(g, value(g, id, "title"), &mut units)?;
        if key == "xAxis"
            && matches!(value(g,id,"type"),Some(Atom::String(s))if g.strings[*s as usize].is("band"))
        {
            for a in array(g, value(g, id, "categories"))? {
                count_text(g, Some(a), &mut units)?
            }
        }
    }
    for p in array(g, value(g, root, "plots"))? {
        let p = object(Some(p))?;
        count_text(g, value(g, p, "title"), &mut units)?;
        if let Some(a) = value(g, p, "pointLabels") {
            if !matches!(a, Atom::Undefined | Atom::Null) {
                for a in array(g, Some(a))? {
                    count_text(g, Some(a), &mut units)?
                }
            }
        }
    }
    Ok(())
}
const fn field_schema(
    name: &'static str,
    kind: FieldKind,
    undefined: bool,
    optional: bool,
) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        undefined,
        optional,
        nullable: false,
        flags: 7,
    }
}
static FIELDS: &[FieldSchema] = &[
    field_schema("type", FieldKind::String, false, false),
    field_schema("title", FieldKind::String, true, false),
    field_schema("accTitle", FieldKind::String, true, false),
    field_schema("accDescr", FieldKind::String, true, false),
    field_schema("orientation", FieldKind::String, true, false),
    field_schema("xAxis", FieldKind::Object, false, false),
    field_schema("yAxis", FieldKind::Object, false, false),
    field_schema("hasSetXAxis", FieldKind::Bool, false, false),
    field_schema("hasSetYAxis", FieldKind::Bool, false, false),
    field_schema("plots", FieldKind::Array, false, false),
];
const fn method(id: u32, name: &'static str, arity: u32) -> MethodSchema {
    MethodSchema {
        id: MethodId(id),
        name,
        arity,
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
static METHODS: &[MethodSchema] = &[
    method(1, "setOrientation", 1),
    method(2, "setXAxisTitle", 1),
    method(3, "setXAxisRangeData", 2),
    method(4, "setXAxisBand", 1),
    method(5, "setYAxisTitle", 1),
    method(6, "setYAxisRangeData", 2),
    method(7, "plotData", 1),
    method(8, "addPlot", 3),
    method(9, "setLineData", 2),
    method(10, "setBarData", 2),
    method(11, "setDiagramTitle", 1),
    method(12, "setAccTitle", 1),
    method(13, "setAccDescription", 1),
];
static MEMBERS: &[MemberSchema] = &[
    member(1, "setOrientation"),
    member(2, "setXAxisTitle"),
    member(3, "setXAxisRangeData"),
    member(4, "setXAxisBand"),
    member(5, "setYAxisTitle"),
    member(6, "setYAxisRangeData"),
    member(7, "plotData"),
    member(8, "addPlot"),
    member(9, "setLineData"),
    member(10, "setBarData"),
    member(11, "setDiagramTitle"),
    member(12, "setAccTitle"),
    member(13, "setAccDescription"),
];
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/xychart@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "XyChartModel",
        kind: RecordKind::Object,
        prototype: Some(PROTOTYPE),
        fields: FIELDS,
        members: &[],
    }],
    prototypes: &[PrototypeSchema {
        id: PROTOTYPE,
        name: "XyChartDb",
        parent: PrototypeParent::Object,
        members: MEMBERS,
    }],
    methods: METHODS,
};
pub struct XyChartPublicState {
    graph: ModelGraph,
    imported: Result<XyChartModel, PeleError>,
}
fn invalid(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("xychart".into());
    e
}
fn limit() -> PeleError {
    let mut e = invalid("XYChart public model expansion exceeds the work budget");
    e.code = ErrorCode::Limit;
    e
}
fn value<'a>(g: &'a ModelGraph, id: u32, key: &str) -> Option<&'a Atom> {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find_map(|p| g.strings[p.key as usize].is(key).then_some(&p.value))
}
fn object(atom: Option<&Atom>) -> Result<u32, PeleError> {
    match atom {
        Some(Atom::Ref(id)) => Ok(*id),
        _ => Err(invalid(
            "XYChart axis, plot and point records must be objects",
        )),
    }
}
fn text(g: &ModelGraph, atom: &Atom) -> Result<String, PeleError> {
    g.text(atom)?
        .well_formed()
        .ok_or_else(|| invalid("XYChart rendering of lone UTF-16 surrogates is not implemented"))
}
fn optional_text(g: &ModelGraph, atom: Option<&Atom>) -> Result<Option<String>, PeleError> {
    match atom {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => Ok(None),
        Some(Atom::Number(n)) if *n == 0. || n.is_nan() => Ok(None),
        Some(a) => text(g, a).map(Some),
    }
}
fn required_text(g: &ModelGraph, atom: Option<&Atom>) -> Result<String, PeleError> {
    text(
        g,
        atom.ok_or_else(|| invalid("XYChart text field is missing"))?,
    )
}
fn number(atom: Option<&Atom>) -> f64 {
    if let Some(Atom::Number(n)) = atom {
        *n
    } else {
        f64::NAN
    }
}
fn truthy(g: &ModelGraph, atom: Option<&Atom>) -> bool {
    match atom {
        None | Some(Atom::Undefined | Atom::Null | Atom::Bool(false)) => false,
        Some(Atom::Number(n)) => *n != 0. && !n.is_nan(),
        Some(Atom::String(id)) => !g.strings[*id as usize].0.is_empty(),
        _ => true,
    }
}
fn array<'a>(g: &'a ModelGraph, atom: Option<&Atom>) -> Result<Vec<&'a Atom>, PeleError> {
    let id = object(atom)?;
    let Node::Array { length, properties } = &g.nodes[id as usize] else {
        return Err(invalid("XYChart chart collections must be Arrays"));
    };
    if *length as usize > MAX_ITEMS {
        return Err(limit());
    }
    let mut entries = HashMap::new();
    for p in properties {
        let key = &g.strings[p.key as usize].0;
        if key.is_empty() || key.len() > 10 || (key.len() > 1 && key[0] == 48) {
            continue;
        }
        let mut i = 0u64;
        let mut valid = true;
        for &c in key {
            if !(48..=57).contains(&c) {
                valid = false;
                break;
            }
            i = i * 10 + u64::from(c - 48);
        }
        if valid && i < u64::from(*length) {
            entries.entry(i as u32).or_insert(&p.value);
        }
    }
    if entries.len() != *length as usize {
        return Err(invalid(
            "XYChart chart collection contains a missing array record",
        ));
    }
    Ok((0..*length).map(|i| entries[&i]).collect())
}
fn axes(g: &ModelGraph, root: u32) -> Result<XyChartModel, PeleError> {
    let x = object(value(g, root, "xAxis"))?;
    let y = object(value(g, root, "yAxis"))?;
    let x_title = optional_text(g, value(g, x, "title"))?.unwrap_or_default();
    let x_axis = if matches!(value(g,x,"type"),Some(Atom::String(id))if g.strings[*id as usize].is("band"))
    {
        let categories = array(g, value(g, x, "categories"))?;
        let mut units = 0;
        for a in &categories {
            count_text(g, Some(a), &mut units)?
        }
        XAxis::Band {
            title: x_title,
            categories: categories
                .into_iter()
                .map(|v| text(g, v))
                .collect::<Result<_, _>>()?,
        }
    } else {
        XAxis::Linear {
            title: x_title,
            min: number(value(g, x, "min")),
            max: number(value(g, x, "max")),
        }
    };

    Ok(XyChartModel {
        diagram_type: DiagramType::Xychart,
        title: None,
        acc_title: None,
        acc_descr: None,
        orientation: None,
        x_axis,
        y_axis: LinearAxis {
            axis_type: "linear",
            title: optional_text(g, value(g, y, "title"))?.unwrap_or_default(),
            min: number(value(g, y, "min")),
            max: number(value(g, y, "max")),
        },
        has_set_x_axis: truthy(g, value(g, root, "hasSetXAxis")),
        has_set_y_axis: truthy(g, value(g, root, "hasSetYAxis")),
        plots: vec![],
    })
}
fn imported(g: &ModelGraph) -> Result<XyChartModel, PeleError> {
    let root = object(Some(&g.root))?;
    text_projection_budget(g, root)?;
    let mut m = axes(g, root)?;
    let mut work = 0usize;
    m.title = optional_text(g, value(g, root, "title"))?;
    m.acc_title = optional_text(g, value(g, root, "accTitle"))?;
    m.acc_descr = optional_text(g, value(g, root, "accDescr"))?;
    m.orientation = match value(g, root, "orientation") {
        None | Some(Atom::Undefined | Atom::Null) => None,
        Some(Atom::String(id)) if g.strings[*id as usize].is("horizontal") => {
            Some(Orientation::Horizontal)
        }
        _ => Some(Orientation::Vertical),
    };
    for p in array(g, value(g, root, "plots"))? {
        let p = object(Some(p))?;
        let mut data = Vec::new();
        let entries = array(g, value(g, p, "data"))?;
        work = work.checked_add(entries.len()).ok_or_else(limit)?;
        if work > MAX_EXPANSION {
            return Err(limit());
        }
        for point in entries {
            let point = object(Some(point))?;
            if !matches!(g.nodes[point as usize], Node::Array { .. }) {
                return Err(invalid("XYChart data points must be tuple Arrays"));
            }
            // The renderer ignores the first tuple item entirely. Preserve it
            // losslessly in the arena without decoding or copying unused keys.
            let n = match value(g, point, "1") {
                Some(Atom::Number(n)) => Some(*n),
                _ => None,
            };
            data.push((String::new(), n));
        }
        let point_labels = match value(g, p, "pointLabels") {
            None | Some(Atom::Undefined | Atom::Null) => None,
            Some(a) => Some(
                array(g, Some(a))?
                    .into_iter()
                    .map(|a| text(g, a))
                    .collect::<Result<_, _>>()?,
            ),
        };
        m.plots.push(Plot{plot_type:if matches!(value(g,p,"type"),Some(Atom::String(id))if g.strings[*id as usize].is("bar")){PlotType::Bar}else{PlotType::Line},title:required_text(g,value(g,p,"title"))?,data,point_labels});
    }
    Ok(m)
}
fn array_node(out: &mut GraphWriter, values: impl Iterator<Item = Atom>) -> u32 {
    let properties: Vec<_> = values
        .enumerate()
        .map(|(i, v)| out.property(&i.to_string(), v))
        .collect();
    out.node(Node::Array {
        length: properties.len() as u32,
        properties,
    })
}
fn str_atom(out: &mut GraphWriter, s: &str) -> Atom {
    Atom::String(out.string(s))
}
fn axis(out: &mut GraphWriter, axis: &XAxis) -> u32 {
    let (kind, title) = match axis {
        XAxis::Band { title, .. } => ("band", title),
        XAxis::Linear { title, .. } => ("linear", title),
    };
    let kind = str_atom(out, kind);
    let title = str_atom(out, title);
    let mut properties = vec![out.property("type", kind), out.property("title", title)];
    match axis {
        XAxis::Band { categories, .. } => {
            let values: Vec<_> = categories.iter().map(|s| str_atom(out, s)).collect();
            let cats = array_node(out, values.into_iter());
            properties.push(out.property("categories", Atom::Ref(cats)));
        }
        XAxis::Linear { min, max, .. } => {
            properties.push(out.property("min", Atom::Number(*min)));
            properties.push(out.property("max", Atom::Number(*max)));
        }
    }
    out.node(Node::Object { properties })
}
fn data_node(out: &mut GraphWriter, data: &[(String, Option<f64>)]) -> u32 {
    let mut points = Vec::new();
    for (key, v) in data {
        let key = str_atom(out, key);
        points.push(Atom::Ref(array_node(
            out,
            [key, v.map_or(Atom::Undefined, Atom::Number)].into_iter(),
        )));
    }
    array_node(out, points.into_iter())
}
fn plot(out: &mut GraphWriter, p: &Plot) -> u32 {
    let data = data_node(out, &p.data);
    let kind = str_atom(
        out,
        if p.plot_type == PlotType::Bar {
            "bar"
        } else {
            "line"
        },
    );
    let title = str_atom(out, &p.title);
    let mut properties = vec![
        out.property("type", kind),
        out.property("title", title),
        out.property("data", Atom::Ref(data)),
    ];
    if let Some(labels) = &p.point_labels {
        let labels: Vec<_> = labels.iter().map(|s| str_atom(out, s)).collect();
        let labels = array_node(out, labels.into_iter());
        properties.push(out.property("pointLabels", Atom::Ref(labels)));
    }
    out.node(Node::Object { properties })
}
fn type_error(message: impl Into<JsString>) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn put_string(g: &mut ModelGraph, text: JsString) -> Result<Atom, PublicError> {
    if let Some(i) = g.strings.iter().position(|s| *s == text) {
        return Ok(Atom::String(i as u32));
    }
    if g.strings.len() >= MAX_ITEMS
        || g.strings
            .iter()
            .map(|s| s.0.len())
            .sum::<usize>()
            .saturating_add(text.0.len())
            > 16_000_000
    {
        return Err(limit().into());
    }
    let id = g.strings.len() as u32;
    g.strings.push(text);
    Ok(Atom::String(id))
}
fn assign(g: &mut ModelGraph, id: u32, key: &str, v: Atom) -> Result<(), PublicError> {
    if let Some(i) = g.nodes[id as usize]
        .properties()
        .iter()
        .position(|p| g.strings[p.key as usize].is(key))
    {
        if g.nodes[id as usize].properties()[i].flags & 2 == 0 {
            return Err(type_error("Cannot assign to read only XYChart property"));
        }
        g.nodes[id as usize].properties_mut()[i].value = v;
        return Ok(());
    }
    if graph_work(g) >= MAX_EXPANSION {
        return Err(limit().into());
    }
    let Atom::String(key) = put_string(g, key.into())? else {
        unreachable!()
    };
    g.nodes[id as usize].properties_mut().push(Property {
        key,
        value: v,
        flags: 7,
    });
    Ok(())
}
fn graph_work(g: &ModelGraph) -> usize {
    g.nodes
        .iter()
        .map(|n| {
            n.properties().len()
                + match n {
                    Node::Map { entries, .. } => entries.len() * 2,
                    Node::Set { values, .. } => values.len(),
                    _ => 0,
                }
        })
        .sum()
}
fn append(g: &mut ModelGraph, other: ModelGraph) -> Result<Atom, PublicError> {
    if g.nodes.len().saturating_add(other.nodes.len()) > MAX_ITEMS {
        return Err(limit().into());
    }
    if graph_work(g).saturating_add(graph_work(&other)) > MAX_EXPANSION {
        return Err(limit().into());
    }
    let mut strings: HashMap<JsString, u32> = g
        .strings
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, s)| (s, i as u32))
        .collect();
    let mut units = g.strings.iter().map(|s| s.0.len()).sum::<usize>();
    let mut map = Vec::with_capacity(other.strings.len());
    for s in other.strings {
        let id = if let Some(id) = strings.get(&s) {
            *id
        } else {
            units = units.saturating_add(s.0.len());
            if units > 16_000_000 || g.strings.len() >= MAX_ITEMS {
                return Err(limit().into());
            }
            let id = g.strings.len() as u32;
            g.strings.push(s.clone());
            strings.insert(s, id);
            id
        };
        map.push(id);
    }
    let base = g.nodes.len() as u32;
    let atom = |v: &mut Atom| match v {
        Atom::String(id) => *id = map[*id as usize],
        Atom::Ref(id) => *id += base,
        _ => {}
    };
    for mut node in other.nodes {
        for p in node.properties_mut() {
            p.key = map[p.key as usize];
            atom(&mut p.value)
        }
        match &mut node {
            Node::Map { entries, .. } => {
                for (k, v) in entries {
                    atom(k);
                    atom(v)
                }
            }
            Node::Set { values, .. } => {
                for v in values {
                    atom(v)
                }
            }
            _ => {}
        }
        g.nodes.push(node);
    }
    let mut root = other.root;
    atom(&mut root);
    Ok(root)
}
fn encode_axis(g: &mut ModelGraph, x: &XAxis) -> Result<Atom, PublicError> {
    let mut out = GraphWriter::default();
    let id = axis(&mut out, x);
    append(g, out.finish(Atom::Ref(id))?)
}
fn strict_number(atom: &Atom) -> Result<f64, PublicError> {
    match atom {
        Atom::Number(n) => Ok(*n),
        _ => Err(type_error(
            "XYChart builder numeric arguments must be numbers",
        )),
    }
}
fn argument(call: &MethodCall, i: usize) -> &Atom {
    call.arguments.get(i).unwrap_or(&Atom::Undefined)
}
fn trimmed(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    if !matches!(a, Atom::String(_)) {
        return Err(type_error("XYChart text argument must be a string"));
    }
    Ok(text(g, a)?.trim_matches(space).into())
}
fn trim_units(raw: &JsString) -> JsString {
    let whitespace = |u: &u16| char::from_u32(*u as u32).is_some_and(space);
    let start = raw
        .0
        .iter()
        .position(|u| !whitespace(u))
        .unwrap_or(raw.0.len());
    let end = raw
        .0
        .iter()
        .rposition(|u| !whitespace(u))
        .map_or(start, |i| i + 1);
    JsString(raw.0[start..end].to_vec())
}
fn xy_text_units(g: &ModelGraph, a: &Atom) -> Result<JsString, PublicError> {
    let Atom::Ref(id) = a else {
        return Err(type_error("XYChart text argument requires an object"));
    };
    let Some(Atom::String(s)) = value(g, *id, "text") else {
        return Err(type_error("XYChart text argument must be a string"));
    };
    Ok(trim_units(&g.strings[*s as usize]))
}
fn xy_text(g: &ModelGraph, a: &Atom) -> Result<String, PublicError> {
    xy_text_units(g, a)?.well_formed().ok_or_else(|| {
        invalid("XYChart builder rendering of lone UTF-16 surrogates is not implemented").into()
    })
}
fn new_record(g: &mut ModelGraph, fields: Vec<(&str, Atom)>) -> Result<Atom, PublicError> {
    if g.nodes.len() >= MAX_ITEMS {
        return Err(limit().into());
    }
    if graph_work(g).saturating_add(fields.len()) > MAX_EXPANSION {
        return Err(limit().into());
    }
    let mut properties = Vec::with_capacity(fields.len());
    for (key, value) in fields {
        let Atom::String(key) = put_string(g, key.into())? else {
            unreachable!()
        };
        properties.push(Property {
            key,
            value,
            flags: 7,
        });
    }
    let id = g.nodes.len() as u32;
    g.nodes.push(Node::Object { properties });
    Ok(Atom::Ref(id))
}
fn array_push(g: &mut ModelGraph, id: u32, item: Atom) -> Result<(), PublicError> {
    let Node::Array { length, .. } = g.nodes[id as usize] else {
        return Err(type_error("this.plots.push is not a function"));
    };
    if length as usize >= MAX_ITEMS {
        return Err(limit().into());
    }
    assign(g, id, &length.to_string(), item)?;
    let Node::Array { length: len, .. } = &mut g.nodes[id as usize] else {
        unreachable!()
    };
    *len = length + 1;
    Ok(())
}
fn dynamic_guard(g: &ModelGraph, id: u32, key: &str) -> Result<(), PublicError> {
    if let Some(a) = value(g, id, key) {
        let expected = match key {
            "setXAxisRangeData" => 3,
            "plotData" => 7,
            "addPlot" => 8,
            _ => 0,
        };
        if matches!(a,Atom::BuiltinFunction(id)if *id==expected) {
            return Ok(());
        }
        return Err(PublicError::Pele(invalid(format!(
            "XYChart overridden {key} dispatch is not implemented"
        ))));
    }
    Ok(())
}
fn synchronize_plot_axes(
    g: &mut ModelGraph,
    root: u32,
    model: &XyChartModel,
    values_nonempty: bool,
    x_set: bool,
    y_set: bool,
) -> Result<(), PublicError> {
    if values_nonempty && !x_set {
        let x = encode_axis(g, &model.x_axis)?;
        assign(g, root, "xAxis", x)?;
        assign(g, root, "hasSetXAxis", Atom::Bool(true))?;
    }
    if values_nonempty && !y_set {
        let y = encode_axis(
            g,
            &XAxis::Linear {
                title: model.y_axis.title.clone(),
                min: model.y_axis.min,
                max: model.y_axis.max,
            },
        )?;
        assign(g, root, "yAxis", y)?;
    }
    Ok(())
}
fn invoke(g: &mut ModelGraph, call: MethodCall) -> Result<Atom, PublicError> {
    let root = match call.receiver {
        Atom::Ref(id) => id,
        _ => return Err(type_error("XYChart method requires an object receiver")),
    };
    let id = call.method.0;
    if id == 11 {
        assign(g, root, "title", argument(&call, 0).clone())?;
        return Ok(Atom::Undefined);
    }
    if id == 1 {
        let horizontal =
            matches!(argument(&call,0),Atom::String(i)if g.strings[*i as usize].is("horizontal"));
        let a = put_string(g, if horizontal { "horizontal" } else { "vertical" }.into())?;
        assign(g, root, "orientation", a)?;
        return Ok(Atom::Undefined);
    }
    if id == 12 || id == 13 {
        // Preserve exact UTF-16, including isolated surrogates, during setters.
        let Atom::String(raw) = argument(&call, 0) else {
            return Err(type_error(
                "XYChart accessibility argument must be a string",
            ));
        };
        let raw = g.strings[*raw as usize].clone();
        let whitespace = |u: u16| char::from_u32(u as u32).is_some_and(space);
        let units = if id == 12 {
            raw.0.into_iter().skip_while(|u| whitespace(*u)).collect()
        } else {
            let mut out = Vec::new();
            let mut after_lf = false;
            for u in raw.0 {
                if after_lf && whitespace(u) {
                    continue;
                }
                out.push(u);
                after_lf = u == 10;
            }
            out
        };
        let a = put_string(g, JsString(units))?;
        assign(g, root, if id == 12 { "accTitle" } else { "accDescr" }, a)?;
        return Ok(Atom::Undefined);
    }
    if id == 2 || id == 5 {
        let title = xy_text_units(g, argument(&call, 0))?;
        let axis = object(value(g, root, if id == 2 { "xAxis" } else { "yAxis" }))?;
        let a = put_string(g, title)?;
        assign(g, axis, "title", a)?;
        return Ok(Atom::Undefined);
    }
    if id == 3 || id == 4 || id == 6 {
        let key = if id == 6 { "yAxis" } else { "xAxis" };
        let old = object(value(g, root, key))?;
        // These native builders only replace a record and set its flag. Keep
        // exact argument atoms, including omitted values and UTF-16 titles.
        let title = value(g, old, "title").cloned().unwrap_or(Atom::Undefined);
        let kind = put_string(g, if id == 4 { "band" } else { "linear" }.into())?;
        let mut fields = vec![("type", kind), ("title", title)];
        if id == 4 {
            let input = array(g, Some(argument(&call, 0)))?;
            let mut units = 0;
            for a in &input {
                let id = object(Some(a))?;
                count_text(g, value(g, id, "text"), &mut units)?
            }
            let categories = input
                .into_iter()
                .map(|a| xy_text_units(g, a))
                .collect::<Result<Vec<_>, _>>()?;
            let mut out = GraphWriter::default();
            let values: Vec<_> = categories
                .into_iter()
                .map(|s| Atom::String(out.string(s)))
                .collect();
            let cats = array_node(&mut out, values.into_iter());
            let cats = append(g, out.finish(Atom::Ref(cats))?)?;
            fields.push(("categories", cats));
        } else {
            fields.push(("min", argument(&call, 0).clone()));
            fields.push(("max", argument(&call, 1).clone()));
        }
        let a = new_record(g, fields)?;
        assign(g, root, key, a)?;
        assign(
            g,
            root,
            if id == 6 {
                "hasSetYAxis"
            } else {
                "hasSetXAxis"
            },
            Atom::Bool(true),
        )?;
        return Ok(Atom::Undefined);
    }
    if (7..=10).contains(&id) {
        if id == 9 || id == 10 {
            dynamic_guard(g, root, "addPlot")?
        }
        if id != 7 {
            dynamic_guard(g, root, "plotData")?
        }
        let mut m = axes(g, root)?;
        let x_set = m.has_set_x_axis;
        let y_set = m.has_set_y_axis;
        let kind = if id == 10
            || (id == 8
                && matches!(argument(&call,0),Atom::String(i)if g.strings[*i as usize].is("bar")))
        {
            PlotType::Bar
        } else {
            PlotType::Line
        };
        let title = if id == 7 {
            String::new()
        } else {
            xy_text(g, argument(&call, if id == 8 { 1 } else { 0 }))?
        };
        let input = array(
            g,
            Some(argument(
                &call,
                if id == 8 {
                    2
                } else if id == 7 {
                    0
                } else {
                    1
                },
            )),
        )?;
        if id != 7 && kind == PlotType::Line {
            let mut units = title.encode_utf16().count();
            for a in &input {
                let id = object(Some(a))?;
                count_text(g, value(g, id, "label"), &mut units)?
            }
        }
        let mut values = Vec::with_capacity(input.len());
        let mut labels = Vec::new();
        let mut label_error = None;
        for a in input {
            if id == 7 {
                values.push(strict_number(a)?)
            } else {
                let point = object(Some(a))?;
                values.push(strict_number(
                    value(g, point, "value").unwrap_or(&Atom::Undefined),
                )?);
                if kind == PlotType::Line {
                    match trimmed(g, value(g, point, "label").unwrap_or(&Atom::Undefined)) {
                        Ok(v) => labels.push(v),
                        Err(e) => {
                            labels.push(String::new());
                            label_error.get_or_insert(e);
                        }
                    }
                }
            }
        }
        if !values.is_empty() && !x_set {
            dynamic_guard(g, root, "setXAxisRangeData")?
        }
        let count = values.len();
        let points = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, labels.get(i).cloned().unwrap_or_default()))
            .collect();
        m.add_plot(kind, title, points);
        synchronize_plot_axes(g, root, &m, count != 0, x_set, y_set)?;
        if let Some(e) = label_error {
            return Err(e);
        }
        let built = m.plots.pop().expect("native builder appends one plot");
        let mut out = GraphWriter::default();
        if id == 7 {
            let data = data_node(&mut out, &built.data);
            return append(g, out.finish(Atom::Ref(data))?);
        }
        let p = plot(&mut out, &built);
        let p = append(g, out.finish(Atom::Ref(p))?)?;
        let target = object(value(g, root, "plots"))?;
        array_push(g, target, p)?;
        return Ok(Atom::Undefined);
    }
    Err(invalid("XYChart public method is unavailable").into())
}
fn construct_default() -> Result<XyChartPublicState, PeleError> {
    // Implicit Pele constructor: field defaults only, no source or argument reads.
    XyChart.create_public(&XyChartModel::new(None))
}

impl PublicDiagramModule for XyChart {
    type PublicState = XyChartPublicState;
    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn public_constructor(
        &self,
        prototype: PrototypeId,
    ) -> Option<crate::compat::public::PublicConstructorFactory<Self::PublicState>> {
        (prototype == PROTOTYPE).then(|| {
            crate::compat::public::PublicConstructorFactory::ignoring_arguments::<Self>(
                PROTOTYPE,
                construct_default,
            )
        })
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let imported = imported(&graph);
        Ok(XyChartPublicState { graph, imported })
    }
    fn create_public(&self, m: &XyChartModel) -> Result<Self::PublicState, PeleError> {
        let mut out = GraphWriter::default();
        let x = axis(&mut out, &m.x_axis);
        let y = axis(
            &mut out,
            &XAxis::Linear {
                title: m.y_axis.title.clone(),
                min: m.y_axis.min,
                max: m.y_axis.max,
            },
        );
        let plots: Vec<_> = m
            .plots
            .iter()
            .map(|p| Atom::Ref(plot(&mut out, p)))
            .collect();
        let plots = array_node(&mut out, plots.into_iter());
        let kind = str_atom(&mut out, "xychart");
        let mut properties = vec![out.property("type", kind)];
        for (key, v) in [
            ("title", &m.title),
            ("accTitle", &m.acc_title),
            ("accDescr", &m.acc_descr),
        ] {
            let v = v
                .as_deref()
                .map_or(Atom::Undefined, |s| str_atom(&mut out, s));
            properties.push(out.property(key, v));
        }
        let orientation = m.orientation.map_or(Atom::Undefined, |v| {
            str_atom(
                &mut out,
                if v == Orientation::Horizontal {
                    "horizontal"
                } else {
                    "vertical"
                },
            )
        });
        properties.extend([
            out.property("orientation", orientation),
            out.property("xAxis", Atom::Ref(x)),
            out.property("yAxis", Atom::Ref(y)),
            out.property("hasSetXAxis", Atom::Bool(m.has_set_x_axis)),
            out.property("hasSetYAxis", Atom::Bool(m.has_set_y_axis)),
            out.property("plots", Atom::Ref(plots)),
        ]);
        let root = out.node(Node::Instance {
            prototype: PROTOTYPE,
            properties,
        });
        let graph = out.finish(Atom::Ref(root))?;
        let imported = imported(&graph);
        Ok(XyChartPublicState { graph, imported })
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
        s.imported = imported(&g);
        s.graph = g;
        Ok(())
    }
    fn invoke_public(
        &self,
        s: &mut Self::PublicState,
        call: MethodCall,
        _: &Object,
        _: &mut dyn RenderResources,
    ) -> Result<Atom, PublicError> {
        let result = invoke(&mut s.graph, call);
        s.imported = imported(&s.graph);
        result
    }
    fn detached_public_render_capability(
        &self,
    ) -> Option<crate::compat::public::DetachedRenderCapability> {
        Some(crate::compat::public::DetachedRenderCapability::xychart())
    }
    fn detached_public_render(
        &self,
        s: &Self::PublicState,
    ) -> Option<crate::compat::public::DetachedPublicRender> {
        Some(crate::compat::public::DetachedPublicRender::xychart(
            super::XYChartDetachedInput::new(s.imported.as_ref().err()),
        ))
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render(s.imported.as_ref().map_err(Clone::clone)?, c, o, h)
    }
}
