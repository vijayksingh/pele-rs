//! Pele swimlane/index.ts, 458322a5 (MIT, kepano). Shared flowchart grammar,
//! graph, shape and edge rendering with the actual lane-layout variant.
mod layout;
use super::flowchart::{
    self,
    graph::{FlowGraph, GraphNode},
    render::{render_flowchart, FlowVariant},
    LabelType,
};
use crate::compat::{
    layout::{
        compound::{CEdge, CNode, CompoundResult, Dir},
        layered::LayeredOptions,
    },
    svg::{edges::theme::ResolvedStyle, label_svg_with_resources, num},
    text::Label,
    DiagramModule, DiagramType, ModuleDescriptor, Object, ParseContext, PeleError, RenderResources,
    RenderSettings, Rendered,
};
pub type SwimlaneModel = flowchart::FlowchartModel;
#[derive(Default)]
pub struct Swimlane;
const DEFAULT_LANE: &str = "__swimlane_default__";
impl FlowVariant for Swimlane {
    fn diagram_type(&self) -> &str {
        "swimlane"
    }
    fn prepare(&self, g: &mut FlowGraph) {
        let mut loose = false;
        for n in &mut g.nodes {
            if n.is_group {
                n.dir = None
            } else if n.parent_id.is_none() {
                loose = true
            }
        }
        if !loose {
            return;
        }
        for n in &mut g.nodes {
            if !n.is_group && n.parent_id.is_none() {
                n.parent_id = Some(DEFAULT_LANE.into())
            }
        }
        g.nodes.push(GraphNode {
            id: DEFAULT_LANE.into(),
            label: None,
            label_type: LabelType::Text,
            shape: "rect".into(),
            is_group: true,
            parent_id: None,
            css_styles: vec![],
            css_styles_origin: None,
            css_compiled_styles: vec![],
            css_classes: String::new(),
            dir: None,
            link: None,
            link_target: None,
            tooltip: None,
            icon: None,
            pos: None,
            img: None,
            asset_width: None,
            asset_height: None,
            constraint: None,
            color_index: None,
            metadata: None,
        })
    }
    fn layout(
        &self,
        nodes: &mut [CNode],
        edges: &mut [CEdge],
        dir: Dir,
        o: &LayeredOptions,
        tight: bool,
    ) -> Result<CompoundResult, PeleError> {
        Ok(layout::lane_layout(
            nodes,
            edges,
            dir,
            o,
            if tight { 12. } else { 24. },
        ))
    }
    fn draw_group(
        &self,
        c: &CNode,
        label: &Label,
        style: &ResolvedStyle,
        classes: &str,
        id: &str,
        dir: Dir,
        r: &mut dyn RenderResources,
    ) -> Result<Option<String>, PeleError> {
        let left = c.x - c.w / 2.;
        let top = c.y - c.h / 2.;
        let side = dir.sideways();
        let cx = if side { left + c.pad_top / 2. } else { c.x };
        let cy = if side { c.y } else { top + c.pad_top / 2. };
        let text = label_svg_with_resources(
            label,
            cx,
            cy,
            &format!(
                " class=\"pele-cluster-label\" fill=\"var(--_m)\"{}",
                style.text
            ),
            r,
        )?;
        let title = if side && !text.is_empty() {
            format!(
                "<g transform=\"rotate(-90 {} {})\">{text}</g>",
                num(cx),
                num(cy)
            )
        } else {
            text
        };
        Ok(Some(format!("<g class=\"pele-cluster pele-lane{classes}\" data-id=\"{id}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"var(--_b)\"/><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"var(--_a)\" stroke=\"var(--_b)\"{}/>{title}</g>",num(left),num(top),num(c.w),num(c.h),num(left),num(top),num(if side{c.pad_top}else{c.w}),num(if side{c.h}else{c.pad_top}),style.shape)))
    }
}
impl DiagramModule for Swimlane {
    type Model = SwimlaneModel;
    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DiagramType::Swimlane)
    }
    fn parse(&self, c: ParseContext<'_>) -> Result<Self::Model, PeleError> {
        flowchart::Flowchart.parse(c)
    }
    fn parse_with_edge_limit(
        &self,
        c: ParseContext<'_>,
        limit: crate::compat::EdgeLimit,
    ) -> Result<Self::Model, PeleError> {
        flowchart::Flowchart.parse_with_edge_limit(c, limit)
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
        render_flowchart(m, c, o, r, Some(self))
    }
}

mod public_model;
pub use public_model::SwimlanePublicState;

pub(crate) type SwimlaneDetachedInput = flowchart::FlowchartDetachedInput;
pub(crate) fn render_detached<R: crate::compat::public::LiveRenderRead>(
    input: &SwimlaneDetachedInput,
    config: &Object,
    settings: &RenderSettings,
    read: &mut R,
    resources: &mut dyn RenderResources,
) -> Result<Rendered, crate::compat::public::PublicError> {
    flowchart::render_live_variant(input, config, settings, read, resources, Some(&Swimlane))
}
