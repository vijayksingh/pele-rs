//! Native mutable Wardley returned plain model, Pele 458322a5 (MIT).
//! WardleyBuilder is discarded upstream; it is not a model prototype API.
use super::*;
use crate::compat::{public::*, ErrorCode};
use std::collections::HashMap;
const ITEMS: usize = 1_000_000;
const TEXT: usize = 16_000_000;
const fn f(name: &'static str, kind: FieldKind, undefined: bool) -> FieldSchema {
    FieldSchema {
        name,
        kind,
        optional: false,
        undefined,
        nullable: false,
        flags: 7,
    }
}
static SCHEMA: ModelSchema = ModelSchema {
    id: "pele/wardley@1",
    version: 1,
    root: Some(RecordId(1)),
    records: &[RecordSchema {
        id: RecordId(1),
        name: "WardleyModel",
        kind: RecordKind::Object,
        prototype: None,
        members: &[],
        fields: &[
            f("type", FieldKind::String, false),
            f("title", FieldKind::String, true),
            f("accTitle", FieldKind::String, true),
            f("accDescr", FieldKind::String, true),
            f("nodes", FieldKind::Array, false),
            f("links", FieldKind::Array, false),
            f("trends", FieldKind::Array, false),
            f("pipelines", FieldKind::Array, false),
            f("annotations", FieldKind::Array, false),
            f("notes", FieldKind::Array, false),
            f("accelerators", FieldKind::Array, false),
            f("deaccelerators", FieldKind::Array, false),
            f("annotationsBox", FieldKind::Object, true),
            f("axes", FieldKind::Object, false),
            f("size", FieldKind::Object, true),
        ],
    }],
    prototypes: &[],
    methods: &[],
};
pub struct WardleyPublicState {
    graph: ModelGraph,
    projected: Result<WardleyModel, PublicError>,
}
fn error(message: impl Into<String>) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("wardley".into());
    e
}
fn limit() -> PeleError {
    let mut e = error("Wardley public model exceeds native graph/work budget");
    e.code = ErrorCode::Limit;
    e
}
fn ty(message: &str) -> PublicError {
    PublicError::JavaScript {
        name: JavaScriptErrorKind::TypeError,
        message: message.into(),
    }
}
fn legacy(e: PublicError) -> PeleError {
    match e {
        PublicError::Pele(e) => e,
        PublicError::JavaScript { message, .. } => error(
            message
                .well_formed()
                .unwrap_or_else(|| "Wardley native TypeError".into()),
        ),
        _ => error("Wardley native host exception"),
    }
}
fn reference(a: &Atom) -> Option<u32> {
    if let Atom::Ref(i) = a {
        Some(*i)
    } else {
        None
    }
}
fn get<'a>(g: &'a ModelGraph, id: u32, key: &str) -> &'a Atom {
    g.nodes[id as usize]
        .properties()
        .iter()
        .find(|p| g.strings[p.key as usize].is(key))
        .map_or(&Atom::Undefined, |p| &p.value)
}
fn truth(g: &ModelGraph, a: &Atom) -> bool {
    match a {
        Atom::Undefined | Atom::Null => false,
        Atom::Bool(v) => *v,
        Atom::Number(v) => *v != 0. && !v.is_nan(),
        Atom::String(i) => !g.strings[*i as usize].0.is_empty(),
        _ => true,
    }
}
struct Writer {
    out: GraphWriter,
    strings: HashMap<JsString, u32>,
    units: usize,
    items: usize,
    work: usize,
}
macro_rules! rec {($w:expr; $($k:literal => $v:expr),* $(,)?) => {{let fields=vec![$(($k,$v)),*];$w.record(fields)}};}
impl Writer {
    fn new() -> Self {
        Self {
            out: GraphWriter::default(),
            strings: HashMap::new(),
            units: 0,
            items: 0,
            work: 0,
        }
    }
    fn text(&mut self, value: impl Into<JsString>) -> Result<Atom, PeleError> {
        let value = value.into();
        if let Some(i) = self.strings.get(&value) {
            return Ok(Atom::String(*i));
        }
        self.units = self.units.saturating_add(value.0.len());
        if self.strings.len() >= ITEMS || self.units > TEXT {
            return Err(limit());
        }
        let i = self.out.string(value.clone());
        self.strings.insert(value, i);
        Ok(Atom::String(i))
    }
    fn opt(&mut self, s: Option<&String>) -> Result<Atom, PeleError> {
        s.map_or(Ok(Atom::Undefined), |s| self.text(s.as_str()))
    }
    fn node(&mut self, node: Node) -> Result<Atom, PeleError> {
        self.items += 1;
        self.work += node.properties().len();
        if self.items > ITEMS || self.work > 8_000_000 {
            return Err(limit());
        }
        Ok(Atom::Ref(self.out.node(node)))
    }
    fn record(&mut self, fields: Vec<(&str, Atom)>) -> Result<Atom, PeleError> {
        let mut properties = vec![];
        for (k, value) in fields {
            let Atom::String(key) = self.text(k)? else {
                unreachable!()
            };
            properties.push(Property {
                key,
                value,
                flags: 7,
            })
        }
        self.node(Node::Object { properties })
    }
    fn array(&mut self, values: Vec<Atom>) -> Result<Atom, PeleError> {
        if values.len() > ITEMS {
            return Err(limit());
        }
        let length = values.len() as u32;
        let mut properties = vec![];
        for (i, value) in values.into_iter().enumerate() {
            let Atom::String(key) = self.text(i.to_string().as_str())? else {
                unreachable!()
            };
            properties.push(Property {
                key,
                value,
                flags: 7,
            })
        }
        self.node(Node::Array { length, properties })
    }
    fn point(&mut self, p: &WardleyPoint) -> Result<Atom, PeleError> {
        rec!(self;"x"=>Atom::Number(p.x),"y"=>Atom::Number(p.y))
    }
    fn force(&mut self, p: &WardleyForce) -> Result<Atom, PeleError> {
        rec!(self;"name"=>self.text(p.name.as_str())?,"x"=>Atom::Number(p.x),"y"=>Atom::Number(p.y))
    }
}
fn new_graph(m: &WardleyModel) -> Result<ModelGraph, PeleError> {
    let mut w = Writer::new();
    let mut nodes = vec![];
    for n in &m.nodes {
        let fallback = [
            "id",
            "label",
            "x",
            "y",
            "className",
            "labelOffsetX",
            "labelOffsetY",
            "inertia",
            "sourceStrategy",
            "inPipeline",
            "isPipelineParent",
        ];
        let keys = if n.public_keys.is_empty() {
            fallback.as_slice()
        } else {
            n.public_keys.as_slice()
        };
        let mut fields = vec![];
        for key in keys {
            let a = match *key {
                "id" => w.text(n.id.as_str())?,
                "label" => w.text(n.label.as_str())?,
                "x" => Atom::Number(n.x),
                "y" => Atom::Number(n.y),
                "className" => w.opt(n.class_name.as_ref())?,
                "labelOffsetX" => n.label_offset_x.map_or(Atom::Undefined, Atom::Number),
                "labelOffsetY" => n.label_offset_y.map_or(Atom::Undefined, Atom::Number),
                "inertia" => n.inertia.map_or(Atom::Undefined, Atom::Bool),
                "sourceStrategy" => w.opt(n.source_strategy.as_ref())?,
                "inPipeline" => n.in_pipeline.map_or(Atom::Undefined, Atom::Bool),
                "isPipelineParent" => n.is_pipeline_parent.map_or(Atom::Undefined, Atom::Bool),
                _ => continue,
            };
            fields.push((*key, a));
        }
        nodes.push(w.record(fields)?);
    }
    let nodes = w.array(nodes)?;
    let mut links = vec![];
    for l in &m.links {
        links.push(rec!(w;"source"=>w.text(l.source.as_str())?,"target"=>w.text(l.target.as_str())?,"dashed"=>Atom::Bool(l.dashed),"label"=>w.opt(l.label.as_ref())?,"flow"=>w.opt(l.flow.as_ref())?)?)
    }
    let links = w.array(links)?;
    let mut trends = vec![];
    for t in &m.trends {
        trends.push(rec!(w;"nodeId"=>w.text(t.node_id.as_str())?,"targetX"=>Atom::Number(t.target_x),"targetY"=>Atom::Number(t.target_y))?)
    }
    let trends = w.array(trends)?;
    let mut pipelines = vec![];
    for p in &m.pipelines {
        let mut ids = vec![];
        for id in &p.component_ids {
            ids.push(w.text(id.as_str())?)
        }
        let ids = w.array(ids)?;
        pipelines.push(rec!(w;"nodeId"=>w.text(p.node_id.as_str())?,"componentIds"=>ids)?)
    }
    let pipelines = w.array(pipelines)?;
    let mut annotations = vec![];
    for a in &m.annotations {
        let mut points = vec![];
        for p in &a.coordinates {
            points.push(w.point(p)?)
        }
        let points = w.array(points)?;
        annotations.push(rec!(w;"number"=>Atom::Number(a.number),"coordinates"=>points,"text"=>w.opt(a.text.as_ref())?)?)
    }
    let annotations = w.array(annotations)?;
    let mut notes = vec![];
    for n in &m.notes {
        notes.push(
            rec!(w;"text"=>w.text(n.text.as_str())?,"x"=>Atom::Number(n.x),"y"=>Atom::Number(n.y))?,
        )
    }
    let notes = w.array(notes)?;
    let mut accelerators = vec![];
    for n in &m.accelerators {
        accelerators.push(w.force(n)?)
    }
    let accelerators = w.array(accelerators)?;
    let mut deaccelerators = vec![];
    for n in &m.deaccelerators {
        deaccelerators.push(w.force(n)?)
    }
    let deaccelerators = w.array(deaccelerators)?;
    let annotations_box = m
        .annotations_box
        .as_ref()
        .map_or(Ok(Atom::Undefined), |p| w.point(p))?;
    let mut axes = vec![];
    if let Some(v) = &m.axes.x_label {
        axes.push(("xLabel", w.text(v.as_str())?))
    }
    if let Some(v) = &m.axes.y_label {
        axes.push(("yLabel", w.text(v.as_str())?))
    }
    if let Some(v) = &m.axes.stages {
        let mut s = vec![];
        for v in v {
            s.push(w.text(v.as_str())?)
        }
        axes.push(("stages", w.array(s)?))
    }
    if let Some(v) = &m.axes.stage_boundaries {
        axes.push((
            "stageBoundaries",
            w.array(v.iter().copied().map(Atom::Number).collect())?,
        ))
    }
    let axes = w.record(axes)?;
    let size = if let Some(s) = &m.size {
        rec!(w;"width"=>Atom::Number(s.width),"height"=>Atom::Number(s.height))?
    } else {
        Atom::Undefined
    };
    let root = rec!(w;"type"=>w.text("wardley")?,"title"=>w.opt(m.title.as_ref())?,"accTitle"=>w.opt(m.acc_title.as_ref())?,"accDescr"=>w.opt(m.acc_descr.as_ref())?,"nodes"=>nodes,"links"=>links,"trends"=>trends,"pipelines"=>pipelines,"annotations"=>annotations,"notes"=>notes,"accelerators"=>accelerators,"deaccelerators"=>deaccelerators,"annotationsBox"=>annotations_box,"axes"=>axes,"size"=>size)?;
    w.out.finish(root)
}
struct Import<'a> {
    g: &'a ModelGraph,
    units: usize,
    work: usize,
}
impl<'a> Import<'a> {
    fn text(&mut self, a: &Atom) -> Result<String, PublicError> {
        let Atom::String(id) = a else {
            return Err(ty("Wardley render text must be a string"));
        };
        let value = &self.g.strings[*id as usize];
        self.units = self.units.saturating_add(value.0.len());
        if self.units > TEXT {
            return Err(limit().into());
        }
        value
            .well_formed()
            .ok_or_else(|| error("Wardley rendering requires well-formed UTF16 text").into())
    }
    fn optional(&mut self, a: &Atom, falsy: bool) -> Result<Option<String>, PublicError> {
        if matches!(a, Atom::Undefined) || falsy && !truth(self.g, a) {
            Ok(None)
        } else {
            self.text(a).map(Some)
        }
    }
    fn record(&self, a: &Atom) -> Result<u32, PublicError> {
        reference(a).ok_or_else(|| ty("Cannot read properties of null or undefined"))
    }
    fn array(&mut self, a: &Atom) -> Result<Vec<&'a Atom>, PublicError> {
        let id = self.record(a)?;
        let Node::Array { length, properties } = &self.g.nodes[id as usize] else {
            return Err(ty("Wardley render collection must be an Array"));
        };
        self.work = self.work.saturating_add(*length as usize);
        if *length as usize > ITEMS || self.work > 8_000_000 {
            return Err(limit().into());
        }
        let mut values = vec![&Atom::Undefined; *length as usize];
        for p in properties {
            let Some(key) = self.g.strings[p.key as usize].well_formed() else {
                continue;
            };
            if let Ok(at) = key.parse::<u32>() {
                if at < *length && at.to_string() == key {
                    values[at as usize] = &p.value
                }
            }
        }
        Ok(values)
    }
    fn strings(&mut self, a: &Atom) -> Result<Vec<String>, PublicError> {
        let mut out = vec![];
        for v in self.array(a)? {
            out.push(self.text(v)?)
        }
        Ok(out)
    }
    fn number(&self, a: &Atom) -> Result<f64, PublicError> {
        match a {
            Atom::Number(v) => Ok(*v),
            Atom::Undefined => Ok(f64::NAN),
            Atom::Null => Ok(0.),
            Atom::Bool(v) => Ok(if *v { 1. } else { 0. }),
            _ => Err(error("Wardley nonnumeric geometry coercion is not implemented").into()),
        }
    }
    fn n(&self, id: u32, key: &str) -> Result<f64, PublicError> {
        self.number(get(self.g, id, key))
    }
    fn opt_n(&self, id: u32, key: &str) -> Result<Option<f64>, PublicError> {
        let a = get(self.g, id, key);
        if matches!(a, Atom::Undefined) {
            Ok(None)
        } else {
            self.number(a).map(Some)
        }
    }
    fn point(&self, a: &Atom) -> Result<WardleyPoint, PublicError> {
        let id = self.record(a)?;
        Ok(WardleyPoint {
            x: self.n(id, "x")?,
            y: self.n(id, "y")?,
        })
    }
    fn force(&mut self, a: &Atom) -> Result<WardleyForce, PublicError> {
        let id = self.record(a)?;
        Ok(WardleyForce {
            name: self.text(get(self.g, id, "name"))?,
            x: self.n(id, "x")?,
            y: self.n(id, "y")?,
        })
    }
}
fn project(g: &ModelGraph) -> Result<WardleyModel, PublicError> {
    let mut i = Import {
        g,
        units: 0,
        work: 0,
    };
    let r = i.record(&g.root)?;
    let title = i.optional(get(g, r, "title"), false)?;
    let acc_title = i.optional(get(g, r, "accTitle"), true)?;
    let acc_descr = i.optional(get(g, r, "accDescr"), true)?;
    let mut nodes = vec![];
    for a in i.array(get(g, r, "nodes"))? {
        let id = i.record(a)?;
        let class_name = if matches!(get(g,id,"className"),Atom::String(s) if g.strings[*s as usize].is("anchor"))
        {
            Some("anchor".into())
        } else {
            None
        };
        let source_strategy = i.optional(get(g, id, "sourceStrategy"), true)?;
        // clamp(null) preserves null upstream, followed by nullish defaults.
        let anchor = class_name.is_some();
        let lift = 8.
            + if source_strategy.as_ref().is_some_and(|s| !s.is_empty()) {
                10.
            } else {
                0.
            };
        let label_offset_x = if matches!(get(g, id, "labelOffsetX"), Atom::Null) {
            Some(if anchor { 0. } else { lift })
        } else {
            i.opt_n(id, "labelOffsetX")?
        };
        let label_offset_y = if matches!(get(g, id, "labelOffsetY"), Atom::Null) {
            Some(if anchor { -3. } else { -lift })
        } else {
            i.opt_n(id, "labelOffsetY")?
        };
        nodes.push(WardleyNode {
            id: i.text(get(g, id, "id"))?,
            label: i.optional(get(g, id, "label"), false)?.unwrap_or_default(),
            x: i.n(id, "x")?,
            y: i.n(id, "y")?,
            class_name,
            label_offset_x,
            label_offset_y,
            in_pipeline: Some(truth(g, get(g, id, "inPipeline"))),
            is_pipeline_parent: Some(truth(g, get(g, id, "isPipelineParent"))),
            inertia: Some(matches!(get(g, id, "inertia"), Atom::Bool(true))),
            source_strategy,
            public_keys: vec![],
        })
    }
    let ids = nodes
        .iter()
        .map(|n| n.id.clone())
        .collect::<std::collections::HashSet<_>>();
    let mut links = vec![];
    for a in i.array(get(g, r, "links"))? {
        let id = i.record(a)?;
        let source = get(g, id, "source");
        let target = get(g, id, "target");
        let (Atom::String(s), Atom::String(t)) = (source, target) else {
            continue;
        };
        let (Some(s), Some(t)) = (
            g.strings[*s as usize].well_formed(),
            g.strings[*t as usize].well_formed(),
        ) else {
            continue;
        };
        if !ids.contains(&s) || !ids.contains(&t) || s == t {
            continue;
        }
        links.push(WardleyLink {
            source: s,
            target: t,
            dashed: truth(g, get(g, id, "dashed")),
            label: i.optional(get(g, id, "label"), true)?,
            flow: match get(g, id, "flow") {
                Atom::String(s) => ["forward", "backward", "bidirectional"]
                    .into_iter()
                    .find(|v| g.strings[*s as usize].is(v))
                    .map(str::to_owned),
                _ => None,
            },
        })
    }
    let mut trends = vec![];
    for a in i.array(get(g, r, "trends"))? {
        let id = i.record(a)?;
        let Atom::String(s) = get(g, id, "nodeId") else {
            continue;
        };
        let Some(node_id) = g.strings[*s as usize]
            .well_formed()
            .filter(|s| ids.contains(s))
        else {
            continue;
        };
        trends.push(WardleyTrend {
            node_id,
            target_x: i.n(id, "targetX")?,
            target_y: i.n(id, "targetY")?,
        })
    }
    let mut pipelines = vec![];
    for a in i.array(get(g, r, "pipelines"))? {
        let id = i.record(a)?;
        pipelines.push(WardleyPipeline {
            node_id: i.text(get(g, id, "nodeId"))?,
            component_ids: i.strings(get(g, id, "componentIds"))?,
        })
    }
    let mut annotations = vec![];
    for a in i.array(get(g, r, "annotations"))? {
        let id = i.record(a)?;
        let mut coordinates = vec![];
        for p in i.array(get(g, id, "coordinates"))? {
            coordinates.push(i.point(p)?)
        }
        annotations.push(WardleyAnnotation {
            number: i.n(id, "number")?,
            coordinates,
            text: i.optional(get(g, id, "text"), true)?,
        })
    }
    let mut notes = vec![];
    for a in i.array(get(g, r, "notes"))? {
        let id = i.record(a)?;
        notes.push(WardleyNote {
            text: i.optional(get(g, id, "text"), false)?.unwrap_or_default(),
            x: i.n(id, "x")?,
            y: i.n(id, "y")?,
        })
    }
    let mut accelerators = vec![];
    for a in i.array(get(g, r, "accelerators"))? {
        accelerators.push(i.force(a)?)
    }
    let mut deaccelerators = vec![];
    for a in i.array(get(g, r, "deaccelerators"))? {
        deaccelerators.push(i.force(a)?)
    }
    let annotations_box = if truth(g, get(g, r, "annotationsBox")) {
        Some(i.point(get(g, r, "annotationsBox"))?)
    } else {
        None
    };
    let axes = i.record(get(g, r, "axes"))?;
    let x_label = if matches!(get(g, axes, "xLabel"), Atom::Null) {
        None
    } else {
        i.optional(get(g, axes, "xLabel"), false)?
    };
    let y_label = if matches!(get(g, axes, "yLabel"), Atom::Null) {
        None
    } else {
        i.optional(get(g, axes, "yLabel"), false)?
    };
    let stages = if truth(g, get(g, axes, "stages")) {
        Some(i.strings(get(g, axes, "stages"))?)
    } else {
        None
    };
    let stage_boundaries = if matches!(get(g, axes, "stageBoundaries"), Atom::Undefined) {
        None
    } else {
        let mut out = vec![];
        for v in i.array(get(g, axes, "stageBoundaries"))? {
            out.push(i.number(v)?)
        }
        Some(out)
    };
    let size = if let Some(id) = reference(get(g, r, "size")) {
        let dimension = |key| {
            if let Atom::Number(v) = get(g, id, key) {
                *v
            } else {
                f64::NAN
            }
        };
        Some(WardleySize {
            width: dimension("width"),
            height: dimension("height"),
        })
    } else {
        None
    };
    Ok(WardleyModel {
        diagram_type: "wardley",
        title,
        acc_title,
        acc_descr,
        nodes,
        links,
        trends,
        pipelines,
        annotations,
        notes,
        accelerators,
        deaccelerators,
        annotations_box,
        axes: WardleyAxes {
            x_label,
            y_label,
            stages,
            stage_boundaries,
        },
        size,
    })
}
impl PublicDiagramModule for Wardley {
    type PublicState = WardleyPublicState;
    fn detached_public_render_capability(&self) -> Option<DetachedRenderCapability> {
        Some(DetachedRenderCapability::wardley())
    }
    fn detached_public_render(&self, state: &Self::PublicState) -> Option<DetachedPublicRender> {
        Some(DetachedPublicRender::wardley(
            super::WardleyDetachedInput::new(state.projected.as_ref().err()),
        ))
    }

    fn model_schema(&self) -> &'static ModelSchema {
        &SCHEMA
    }
    fn create_public_from_graph(&self, graph: ModelGraph) -> Result<Self::PublicState, PeleError> {
        graph.validate_schema(self.model_schema())?;
        let projected = project(&graph);
        Ok(WardleyPublicState { graph, projected })
    }
    fn create_public(&self, m: &WardleyModel) -> Result<Self::PublicState, PeleError> {
        let graph = new_graph(m)?;
        let projected = project(&graph);
        Ok(WardleyPublicState { graph, projected })
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
    fn apply_public(&self, s: &mut Self::PublicState, graph: ModelGraph) -> Result<(), PeleError> {
        s.projected = project(&graph);
        s.graph = graph;
        Ok(())
    }
    fn render_public_operation(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        self.render(s.projected.as_ref().map_err(Clone::clone)?, c, o, h)
            .map_err(Into::into)
    }
    fn render_public(
        &self,
        s: &mut Self::PublicState,
        c: &Object,
        o: &RenderSettings,
        h: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        self.render_public_operation(s, c, o, h).map_err(legacy)
    }
}
