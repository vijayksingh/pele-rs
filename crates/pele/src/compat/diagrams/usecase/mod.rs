//! Complete Pele Usecase translation, pinned 458322a5 (MIT, kepano).
mod db;
mod graph;
mod json;
mod json_budget;
mod live_phase;
pub(crate) use live_phase::{render_detached, UsecaseDetachedInput};
mod lexer;
mod model;
mod parser;
mod render;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ErrorCode, ModuleDescriptor, Object, ParseContext, PeleError,
    RenderResources, RenderSettings, Rendered,
};
pub use graph::{flatten_json_rows, JsonRow};
pub use json::{JsonAtom, JsonValue};
pub use model::{
    Actor, ClassDef, EdgeOccurrence, GraphStatement, MetadataOccurrence, NodeOccurrence,
    Relationship, Span, SystemBoundary, UseCase, UsecaseJsonNode, UsecaseModel, UsecaseNote,
};
#[derive(Default)]
pub struct Usecase;
impl DiagramModule for Usecase {
    type Model = UsecaseModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Usecase)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c)
    }
    fn validate_model_projection(&self, m: &Self::Model) -> Result<(), PeleError> {
        if m.json_nodes.iter().any(|(_, j)| j.value.depth + 4 > 128) {
            Err(PeleError::new(
                "Usecase JSON exceeds safe model projection depth",
                ErrorCode::Limit,
            ))
        } else {
            Ok(())
        }
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

mod public_model;
pub use public_model::UsecasePublicState;
