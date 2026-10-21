//! Translated from Pele diagrams/quadrant/{index,db,lexer,parser,render}.ts.
//! Upstream 458322a5b8abb1ae57ddf127c32017ab9fb14a92, MIT, Copyright kepano.
mod parser;
mod public_model;
pub use public_model::QuadrantPublicState;
mod live_phase;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered, Value,
};
pub(crate) use live_phase::{render_detached, QuadrantDetachedInput};
use serde::{Serialize, Serializer};
#[derive(Clone, Debug, Serialize)]
pub struct QuadrantText {
    pub text: String,
    #[serde(rename = "type")]
    pub label_type: String,
}
#[derive(Clone, Debug, Default)]
pub struct PointStyle {
    pub radius: Option<f64>,
    pub color: Option<String>,
    pub stroke_color: Option<String>,
    pub stroke_width: Option<String>,
    pub(crate) order: Vec<&'static str>,
}
impl Serialize for PointStyle {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(None)?;
        let mut order = self.order.clone();
        for field in ["radius", "color", "strokeColor", "strokeWidth"] {
            if !order.contains(&field) {
                order.push(field)
            }
        }
        for field in order {
            match field {
                "radius" => {
                    if let Some(n) = self.radius {
                        map.serialize_entry(field, &Value::Number(n))?
                    }
                }
                "color" => {
                    if let Some(v) = &self.color {
                        map.serialize_entry(field, v)?
                    }
                }
                "strokeColor" => {
                    if let Some(v) = &self.stroke_color {
                        map.serialize_entry(field, v)?
                    }
                }
                "strokeWidth" => {
                    if let Some(v) = &self.stroke_width {
                        map.serialize_entry(field, v)?
                    }
                }
                _ => {}
            }
        }
        map.end()
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuadrantPoint {
    #[serde(serialize_with = "number")]
    pub x: f64,
    #[serde(serialize_with = "number")]
    pub y: f64,
    pub text: QuadrantText,
    pub class_name: String,
    #[serde(flatten)]
    pub style: PointStyle,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuadrantModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
    pub quadrants: [Option<QuadrantText>; 4],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_left: Option<QuadrantText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x_axis_right: Option<QuadrantText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_bottom: Option<QuadrantText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y_axis_top: Option<QuadrantText>,
    pub points: Vec<QuadrantPoint>,
    #[serde(serialize_with = "classes")]
    pub classes: Vec<(String, PointStyle)>,
}
fn number<S: Serializer>(n: &f64, s: S) -> Result<S::Ok, S::Error> {
    Value::Number(*n).serialize(s)
}
fn classes<S: Serializer>(entries: &[(String, PointStyle)], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeSeq;
    let mut out = s.serialize_seq(Some(entries.len()))?;
    for entry in entries {
        out.serialize_element(entry)?;
    }
    out.end()
}
#[derive(Default)]
pub struct QuadrantChart;
impl DiagramModule for QuadrantChart {
    type Model = QuadrantModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::QuadrantChart)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn render(
        &self,
        m: &Self::Model,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}

impl QuadrantModel {
    /// Empty class state shared by parser population and native construction.
    pub(super) fn empty(title: Option<&str>) -> Self {
        Self {
            diagram_type: DiagramType::QuadrantChart,
            title: title.map(str::to_owned),
            acc_title: None,
            acc_descr: None,
            quadrants: Default::default(),
            x_axis_left: None,
            x_axis_right: None,
            y_axis_bottom: None,
            y_axis_top: None,
            points: Vec::new(),
            classes: Vec::new(),
        }
    }
}
