//! Pele agentflow/graph.ts + colorSlots.ts, 458322a5 (MIT, kepano).
//! Resolves collapsed/global membership, containment diagnostics and stable kind palette slots.
use super::model::{AgentDb, LabelType};
use crate::compat::{Object, Value};
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(super) struct GraphNode {
    pub id: String,
    pub label: Option<String>,
    pub label_type: LabelType,
    pub shape: String,
    pub is_group: bool,
    pub parent_id: Option<String>,
    pub css_styles: Vec<String>,
    pub css_compiled_styles: Vec<String>,
    pub css_classes: String,
    pub dir: Option<String>,
    pub link: Option<String>,
    pub link_target: Option<String>,
    pub tooltip: Option<String>,
    pub color_index: Option<usize>,
    pub kind: Option<String>,
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(super) struct GraphEdge {
    pub id: String,
    pub is_user_defined_id: bool,
    pub start: String,
    pub end: String,
    pub edge_type: String,
    pub label: String,
    pub label_type: LabelType,
    pub thickness: Option<String>,
    pub pattern: Option<String>,
    pub minlen: Option<Value>,
    pub arrow_type_start: String,
    pub arrow_type_end: String,
    pub css_compiled_styles: Vec<String>,
    pub style: Vec<String>,
    pub classes: String,
    pub animate: Option<Value>,
    pub animation: Option<Value>,
    pub curve: Option<Value>,
    pub metadata: Option<Object>,
}
#[derive(Clone, Debug)]
pub(super) struct AgentGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}
pub(super) fn container_slot(n: usize, palette: usize) -> usize {
    let slots = palette.saturating_sub(7).max(1);
    let slot = 7 + n % slots;
    if palette > 0 {
        slot % palette
    } else {
        slot
    }
}
pub(super) fn build(db: &mut AgentDb, config: &Object) -> AgentGraph {
    super::graph_source::typed(db, config)
}
