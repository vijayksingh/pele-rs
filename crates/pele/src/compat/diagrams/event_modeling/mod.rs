//! Event-modeling grammar, shared models, horizontal lanes and vertical rails.
//! Translated from Pele eventmodeling/*, 458322a5 (MIT, Copyright 2026 kepano).
mod model;
mod parser;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{
    EmBox, EmLane, EmNote, EmRelation, EmSpecification, EntityKind, EventModelingModel, LaneGroup,
};
#[derive(Default)]
pub struct EventModeling;
impl DiagramModule for EventModeling {
    type Model = EventModelingModel;
    fn descriptor(&self) -> ModuleDescriptor {
        let mut d = ModuleDescriptor::new(DiagramType::Eventmodeling);
        d.keep_comments = true;
        d
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<EventModelingModel, PeleError> {
        parser::parse(c.source, c.title)
    }
    fn render(
        &self,
        m: &EventModelingModel,
        _c: &Object,
        o: &RenderSettings,
        r: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(m, o, r)
    }
}

mod public_model;
pub use public_model::EventModelingPublicState;

mod live_phase;
mod phase;
pub(crate) use live_phase::{render_detached, EventModelingDetachedInput};
