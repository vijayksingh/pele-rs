//! Complete ER diagram module, Pele er/* 458322a5 (MIT, kepano).
mod direct_parse;
mod graph;
mod lexer;
mod markers;
mod model;
mod parser;
mod render;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{
    Cardinality, ErAttribute, ErClassDef, ErEntity, ErModel, ErRelationship, ErSubgraph,
    Identification, RelSpec,
};
#[derive(Default)]
pub struct Er;
impl DiagramModule for Er {
    type Model = ErModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Er)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<ErModel, PeleError> {
        parser::parse(context)
    }
    fn render(
        &self,
        model: &ErModel,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}

mod public_model;
pub use public_model::ErPublicState;
