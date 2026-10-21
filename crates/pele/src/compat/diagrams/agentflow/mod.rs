//! AgentFlow, complete Pele agentflow/* 458322a5 translation (MIT, kepano).
mod graph;
mod lexer;
mod model;
mod parser;
mod render;
mod shapes;
mod tokens;
use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub use model::{
    AgentClassDef, AgentEdge, AgentNode, AgentSubgraph, AgentflowModel, Diagnostic, DiagnosticId,
    ElementMapping, ElementPosition, LabelType, SemanticConnector, SemanticEdge, SemanticModel,
    SemanticSubgraph, SemanticVertex,
};
#[derive(Default)]
pub struct AgentFlow;
impl DiagramModule for AgentFlow {
    type Model = AgentflowModel;
    fn descriptor(&self) -> ModuleDescriptor {
        let mut d = ModuleDescriptor::new(DiagramType::Agentflow);
        d.keep_comments = true;
        d
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(c)
    }
    fn parse_with_edge_limit(
        &self,
        c: ParseContext<'_>,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        parser::parse_with_edge_limit(c, limit)
    }
    fn query(
        &self,
        model: &Self::Model,
        operation: &str,
        _arguments: &crate::compat::Value,
    ) -> Result<crate::compat::Value, PeleError> {
        if operation == "semanticModel" {
            crate::compat::engine::project_model(&model.semantic_model())
        } else {
            Err(PeleError::new(
                format!("Unknown model query: {operation}"),
                crate::compat::ErrorCode::Semantic,
            ))
        }
    }
    fn parse_prepared(
        &self,
        context: ParseContext<'_>,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        // The raw JavaScript count >= undefined comparison never rejects.
        let limit = match limit {
            crate::compat::EdgeLimit::Undefined => crate::compat::EdgeLimit::Number(f64::NAN),
            other => other,
        };
        self.parse_with_edge_limit(context, limit)
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
pub use public_model::{AgentflowPublicOrigin, AgentflowPublicState};

mod live_phase;
mod phase;
pub(crate) use live_phase::{render_detached, AgentflowDetachedInput};
