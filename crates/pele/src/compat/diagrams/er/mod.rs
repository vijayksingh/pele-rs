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

mod live_read;
#[cfg(feature = "web-host-er")]
#[doc(hidden)]
pub use live_read::{render_store, RenderStore};
#[cfg(feature = "web-host-er")]
#[doc(hidden)]
pub use public_model::MethodStore;

#[cfg(feature = "web-host-er")]
pub(crate) use direct_parse::parse_into as parse_host_into;
#[cfg(feature = "web-host-er")]
#[doc(hidden)]
pub use public_model::InitialWriter;

#[cfg(feature = "web-host-er")]
mod host_initial;
#[cfg(feature = "web-host-er")]
pub(crate) use host_initial::admit as admit_host_initial;
#[cfg(feature = "web-host-er")]
pub(crate) fn publish_host_initial<W: InitialWriter>(
    writer: W,
    model: &ErModel,
) -> Result<W::Output, PeleError> {
    public_model::initial_into(writer, model)
}
