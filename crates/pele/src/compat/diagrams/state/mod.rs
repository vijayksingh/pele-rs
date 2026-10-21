//! Pele state diagram module, pinned 458322a5 (MIT, kepano).
pub(crate) mod graph;
mod lexer;
mod model;
mod parser;
mod tokens;
use crate::compat::{Engine, ParseContext, ParseOptions, PeleError};
pub use graph::{StateEdge, StateGraph, StateNode};
pub use model::*;
/// Public source/model slice; preparation is shared with Engine.
pub fn parse(source: &str, options: &ParseOptions) -> Result<StateModel, PeleError> {
    let inspection = Engine::empty().inspect(source, &options.preparation)?;
    let mut encoded = inspection.encoded_text;
    encoded.push('\n');
    parser::parse(ParseContext {
        source: &encoded,
        config: &inspection.prepared.config,
        title: inspection.prepared.title.as_deref(),
        line_offset: inspection.prepared.line_offset,
        max_edges: options.max_edges,
    })
}
mod live_phase;
mod phase;
mod render;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, RenderResources, RenderSettings, Rendered,
};
pub(crate) use live_phase::{render_detached, StateDetachedInput};
#[derive(Default)]
pub struct State;
impl DiagramModule for State {
    type Model = StateModel;
    fn validate_model_projection(&self, model: &StateModel) -> Result<(), PeleError> {
        // Bound the observable JSON projection, not typed state nesting. Root maps
        // and rootDoc may retain the same deep document tree. Leave both retained
        // document rendering and typed model access available beyond this boundary.
        enum Item<'a> {
            Stmt(&'a Stmt, usize),
            State(&'a StateStmt, usize),
        }
        let mut pending: Vec<_> = model.root_doc.iter().map(|s| Item::Stmt(s, 2)).collect();
        pending.extend(model.states.iter().map(|(_, s)| Item::State(s, 2)));
        let limit = || {
            PeleError::new(
                "State model exceeds safe JSON projection depth",
                crate::compat::ErrorCode::Limit,
            )
        };
        while let Some(item) = pending.pop() {
            let (state, depth) = match item {
                Item::State(s, d) => (s, d),
                Item::Stmt(stmt, d) => match stmt {
                    Stmt::State(s) => (s, d),
                    Stmt::Relation(r) => {
                        pending.push(Item::State(&r.state1, d + 1));
                        pending.push(Item::State(&r.state2, d + 1));
                        continue;
                    }
                    Stmt::Click(c) => {
                        if let ClickId::State(s) = &c.id {
                            pending.push(Item::State(s, d + 1))
                        }
                        continue;
                    }
                    _ => {
                        if d + 1 > 120 {
                            return Err(limit());
                        }
                        continue;
                    }
                },
            };
            if depth + 1 > 120 {
                return Err(limit());
            }
            let extra = state.note.is_some()
                || matches!(&state.description,Some(Description::Lines(v))if !v.is_empty())
                || [
                    &state.classes,
                    &state.styles,
                    &state.text_styles,
                    &state.descriptions,
                ]
                .iter()
                .any(|v| v.as_ref().is_some_and(|v| !v.is_empty()));
            if extra && depth + 2 > 120 {
                return Err(limit());
            }
            if let Some(doc) = &state.doc {
                pending.extend(doc.iter().map(|s| Item::Stmt(s, depth + 2)))
            }
        }
        Ok(())
    }
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::State)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(context)
    }
    fn render(
        &self,
        model: &Self::Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}

mod public_model;
pub use public_model::StatePublicState;
