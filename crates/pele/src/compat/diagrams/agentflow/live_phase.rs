//! Agentflow's graph is cached during parse; original style-array identities stay live.
//! Pele458322a5 (MIT,kepano). Generic retained graph references bind native coordinates.
use super::{
    graph::{AgentGraph, GraphNode},
    phase::PhaseInput,
};
use crate::compat::{
    public::{LiveRenderRead, LiveStringUse, LiveValueKind, PublicError},
    ErrorCode, Object, PeleError, RenderResources, RenderSettings, Rendered,
};
pub(crate) struct AgentflowDetachedInput {
    graph: Result<AgentGraph, PeleError>,
    styles: Vec<Option<u32>>,
    cache: Option<super::live_builder::Cache>,
}
impl AgentflowDetachedInput {
    pub(super) fn new(
        graph: &AgentGraph,
        styles: &[Option<u32>],
        error: Option<&PeleError>,
    ) -> Self {
        // Check expanded cached graph before cloning it into an independent job.
        let units = graph
            .nodes
            .iter()
            .flat_map(|n| n.css_compiled_styles.iter().chain(&n.css_styles))
            .chain(
                graph
                    .edges
                    .iter()
                    .flat_map(|e| e.css_compiled_styles.iter().chain(&e.style)),
            )
            .try_fold(0usize, |n, s| n.checked_add(s.encode_utf16().count()))
            .unwrap_or(usize::MAX);
        let graph = if let Some(error) = error.filter(|e| e.code == ErrorCode::Limit) {
            Err(error.clone())
        } else if units > 16_000_000 {
            Err(limit())
        } else {
            Ok(graph.clone())
        };
        Self {
            graph,
            styles: styles.to_vec(),
            cache: None,
        }
    }
    pub(super) fn constructed(
        cache: super::live_builder::Cache,
        error: Option<&PeleError>,
    ) -> Self {
        Self {
            graph: error
                .filter(|e| e.code == ErrorCode::Limit)
                .cloned()
                .map_or_else(
                    || {
                        Ok(AgentGraph {
                            nodes: vec![],
                            edges: vec![],
                        })
                    },
                    Err,
                ),
            styles: vec![],
            cache: Some(cache),
        }
    }
}
pub(super) fn error(message: &str) -> PeleError {
    let mut e = PeleError::new(message, ErrorCode::Semantic);
    e.diagram_type = Some("agentflow".into());
    e
}
pub(super) fn limit() -> PeleError {
    let mut e = error("AgentFlow public model expansion exceeds the work budget");
    e.code = ErrorCode::Limit;
    e
}
struct Host<'a, R: LiveRenderRead> {
    read: &'a mut R,
    styles: Vec<Option<R::Value>>,
    units: usize,
    work: usize,
}
impl<R: LiveRenderRead> Host<'_, R> {
    fn string(&mut self, value: &R::Value) -> Result<String, PublicError> {
        let s = self.read.require_string(
            value,
            LiveStringUse {
                method: "trim",
                expression_receiver: "t",
            },
        )?;
        self.units = self.units.saturating_add(s.0.len());
        if self.units > 16_000_000 {
            return Err(limit().into());
        }
        s.well_formed()
            .ok_or_else(|| error("Agentflow rendering requires well-formed UTF16 text").into())
    }
}
impl<R: LiveRenderRead> PhaseInput for Host<'_, R> {
    type Error = PublicError;
    fn resolve_styles(&mut self, index: usize, node: &mut GraphNode) -> Result<(), PublicError> {
        if let Some(Some(array)) = self.styles.get(index) {
            let array = array.clone();
            let values = self.read.snapshot_iterable(&array, 1_000_000)?;
            self.work = self.work.saturating_add(values.len());
            if self.work > 8_000_000 {
                return Err(limit().into());
            }
            node.css_styles = values
                .iter()
                .map(|v| self.string(v))
                .collect::<Result<_, _>>()?;
        }
        Ok(())
    }
    fn metadata(&mut self, key: &'static str) -> Result<Option<String>, PublicError> {
        let root = self.read.root();
        let v = self.read.get(&root, key)?;
        if matches!(
            self.read.kind(&v),
            LiveValueKind::Undefined | LiveValueKind::Null
        ) {
            Ok(None)
        } else {
            self.string(&v).map(Some)
        }
    }
}
pub(crate) fn render_detached<R: LiveRenderRead>(
    input: &AgentflowDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, PublicError> {
    let parsed_graph = input.graph.as_ref().map_err(Clone::clone)?;
    let built = if let Some(cache) = &input.cache {
        let existing = cache
            .lock()
            .map_err(|_| error("Agentflow graph cache unavailable"))?
            .clone();
        Some(if let Some(existing) = existing {
            existing
        } else {
            // No cache lock, owner borrow or placeholder is held while log can
            // reenter. A failed outer build leaves any completed nested cache.
            let built = std::sync::Arc::new(super::live_builder::build(read, resources, config)?);
            *cache
                .lock()
                .map_err(|_| error("Agentflow graph cache unavailable"))? = Some(built.clone());
            built
        })
    } else {
        None
    };
    let graph = built.as_ref().map_or(parsed_graph, |b| &b.graph);
    let styles = if let Some(built) = &built {
        built
            .styles
            .iter()
            .map(|id| id.map(|id| read.retained_value(id)).transpose())
            .collect::<Result<Vec<_>, _>>()?
    } else {
        input
            .styles
            .iter()
            .map(|id| id.map(|id| read.retained_graph_value(id)).transpose())
            .collect::<Result<Vec<_>, _>>()?
    };
    super::render::render_phases(
        &mut Host {
            read,
            styles,
            units: 0,
            work: 0,
        },
        graph,
        config,
        settings,
        resources,
    )
}
