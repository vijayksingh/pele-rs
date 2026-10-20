//! Full flowchart port of pinned Pele 458322a5 (MIT, kepano).
//! Source/model parsing and actual compound rendering share the same registered module.
pub(crate) mod graph;
mod lexer;
mod model;
mod parser;
pub(crate) mod render;
pub(crate) mod shapes;
mod tokens;
use crate::compat::{Engine, ParseContext, ParseOptions, PeleError};
pub use model::{FlowClassDef, FlowEdge, FlowNode, FlowSubgraph, FlowchartModel, LabelType};
/// Parse through the same bounded preparation, directive, encoding and configuration path as Engine.
/// Register `Flowchart` with `Engine` for retained document rendering; this helper leaves the registry unchanged.
pub fn parse(source: &str, options: &ParseOptions) -> Result<FlowchartModel, PeleError> {
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

use crate::compat::{
    DiagramModule, DiagramType, ModuleDescriptor, Object, RenderResources, RenderSettings, Rendered,
};
#[derive(Default)]
pub struct Flowchart;
impl DiagramModule for Flowchart {
    type Model = FlowchartModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Flowchart)
    }
    fn parse(&self, context: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        parser::parse(context)
    }
    fn parse_with_edge_limit(
        &self,
        context: ParseContext<'_>,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        parser::parse_with_edge_limit(context, limit)
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
        model: &Self::Model,
        config: &Object,
        options: &RenderSettings,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PeleError> {
        render::render(model, config, options, resources)
    }
}

mod public_model;
pub use public_model::{FlowchartPublicOrigin, FlowchartPublicState};

pub(crate) use public_model::{
    construct_with as construct_flow_with, detached_method as detached_flow_method,
    LIVE_FIELDS as FLOW_LIVE_FIELDS,
};
