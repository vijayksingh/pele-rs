//! Complete sequence diagram family, Pele sequence/* 458322a5 (MIT, kepano).
mod actors;
mod colors;
mod json;
mod kinds;
pub use json::SeqValue;
mod lexer;
mod live_phase;
mod model;
mod parser;
mod phase;
mod render;
pub(crate) use live_phase::{render_detached, SequenceDetachedInput};
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{
    MessageText, NoteActors, SeqActor, SeqBox, SeqMessage, SeqNote, SeqNumbering, SequenceModel,
};
#[derive(Default)]
pub struct Sequence;
impl DiagramModule for Sequence {
    type Model = SequenceModel;
    fn validate_model_projection(&self, model: &SequenceModel) -> Result<(), PeleError> {
        let actor_depth = model.actors.iter().any(|(_, actor)| {
            actor
                .links
                .iter()
                .chain(&actor.properties)
                .any(|(_, value)| crate::compat::value::json_depth_exceeds(value.json(), 122))
        });
        let message_depth = model.messages.iter().any(|message| {
            message
                .answer
                .as_ref()
                .is_some_and(|value| crate::compat::value::json_depth_exceeds(value.json(), 125))
        });
        if actor_depth || message_depth {
            return Err(PeleError::new(
                "Sequence model exceeds safe JSON projection depth",
                crate::compat::ErrorCode::Limit,
            ));
        }
        Ok(())
    }
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Sequence)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<SequenceModel, PeleError> {
        parser::parse(context)
    }
    fn render(
        &self,
        model: &SequenceModel,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}

mod public_model;
pub use model::SequencePublicOrigin;
pub use public_model::SequencePublicState;
