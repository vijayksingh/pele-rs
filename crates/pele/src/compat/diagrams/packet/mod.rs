//! Pele diagrams/packet/{parser,model,index}.ts, 458322a5 (MIT, kepano).
mod live_phase;
pub(crate) use live_phase::{render_detached, PacketDetachedInput};
mod parser;
mod public_model;
pub use public_model::PacketPublicState;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered, Value,
};
use serde::{Serialize, Serializer};
use std::sync::Arc;
fn label<S: Serializer>(v: &Arc<str>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(v)
}
fn number<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
    Value::Number(*v).serialize(s)
}
#[derive(Clone, Debug, Serialize)]
pub struct PacketBlock {
    #[serde(serialize_with = "number")]
    pub start: f64,
    #[serde(serialize_with = "number")]
    pub end: f64,
    #[serde(serialize_with = "number")]
    pub bits: f64,
    #[serde(serialize_with = "label")]
    pub label: Arc<str>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PacketModel {
    #[serde(rename = "type")]
    pub diagram_type: DiagramType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acc_descr: Option<String>,
    #[serde(serialize_with = "number")]
    pub bits_per_row: f64,
    pub rows: Vec<Vec<PacketBlock>>,
}
pub(super) fn packet_option(config: &Object, key: &str, fallback: f64, min: f64, max: f64) -> f64 {
    match config
        .get("packet")
        .and_then(Value::as_object)
        .and_then(|c| c.get(key))
    {
        Some(Value::Number(n)) if *n >= min => n.min(max),
        _ => fallback,
    }
}
#[derive(Default)]
pub struct Packet;
impl DiagramModule for Packet {
    type Model = PacketModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Packet)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<PacketModel, PeleError> {
        parser::parse(c.source, c.title, c.config)
    }
    fn render(
        &self,
        m: &PacketModel,
        c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, c, o, r)
    }
}
