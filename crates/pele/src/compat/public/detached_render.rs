//! Sealed, detached builtin rendering. No document borrow survives a host call.
use super::{engine::Registration, GraphReadView, PublicError};
use crate::compat::diagrams::radar::{Radar, RadarDetachedInput};
use crate::compat::{
    diagrams::info::{Info, InfoDetachedInput},
    diagrams::ishikawa::{Ishikawa, IshikawaDetachedInput},
    diagrams::packet::{Packet, PacketDetachedInput},
    diagrams::pie::{Pie, PieDetachedInput},
    diagrams::quadrant_chart::{QuadrantChart, QuadrantDetachedInput},
    diagrams::sankey::{render_detached, Sankey, SankeyDetachedInput},
    ErrorCode, Object, OutputLimit, PeleError, PresentationCapture, PresentationOutcome,
    RenderOptions, RenderResources, RenderResult, RenderSettings, Rendered,
};
use std::{
    any::TypeId,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

pub use crate::compat::diagrams::sankey::{
    LiveCallArgument, LiveRenderRead, LiveStringUse, LiveValueKind,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Agentflow,
    Sequence,
    Class,
    XYChart,
    Swimlane,
    Flowchart,
    Block,
    State,
    Architecture,
    EventModeling,
    Usecase,
    GitGraph,
    Railroad,
    Requirement,
    C4,
    Wardley,
    Cynefin,
    Er,
    Mindmap,
    Treemap,
    Kanban,
    TreeView,
    Journey,
    Timeline,
    Ishikawa,
    Venn,
    Quadrant,
    Packet,
    Radar,
    Info,
    Sankey,
    Pie,
}
/// Metadata-only capability. Only this crate can mint a concrete creator token.
#[doc(hidden)]
pub struct DetachedRenderCapability {
    creator: TypeId,
    kind: Kind,
}
impl DetachedRenderCapability {
    pub(crate) fn agentflow() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::agentflow::AgentFlow>(),
            kind: Kind::Agentflow,
        }
    }

    pub(crate) fn sequence() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::sequence::Sequence>(),
            kind: Kind::Sequence,
        }
    }

    pub(crate) fn class() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::class_diagram::Class>(),
            kind: Kind::Class,
        }
    }

    pub(crate) fn xychart() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::xychart::XyChart>(),
            kind: Kind::XYChart,
        }
    }

    pub(crate) fn swimlane() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::swimlane::Swimlane>(),
            kind: Kind::Swimlane,
        }
    }

    pub(crate) fn flowchart() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::flowchart::Flowchart>(),
            kind: Kind::Flowchart,
        }
    }

    pub(crate) fn block() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::block::Block>(),
            kind: Kind::Block,
        }
    }

    pub(crate) fn state() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::state::State>(),
            kind: Kind::State,
        }
    }

    pub(crate) fn architecture() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::architecture::Architecture>(),
            kind: Kind::Architecture,
        }
    }

    pub(crate) fn eventmodeling() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::event_modeling::EventModeling>(),
            kind: Kind::EventModeling,
        }
    }

    pub(crate) fn usecase() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::usecase::Usecase>(),
            kind: Kind::Usecase,
        }
    }

    pub(crate) fn git_graph() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::git_graph::GitGraph>(),
            kind: Kind::GitGraph,
        }
    }

    pub(crate) fn railroad() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::railroad::Railroad>(),
            kind: Kind::Railroad,
        }
    }

    pub(crate) fn requirement() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::requirement::RequirementDiagram>(),
            kind: Kind::Requirement,
        }
    }

    pub(crate) fn c4() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::c4::C4>(),
            kind: Kind::C4,
        }
    }

    pub(crate) fn wardley() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::wardley::Wardley>(),
            kind: Kind::Wardley,
        }
    }

    pub(crate) fn cynefin() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::cynefin::Cynefin>(),
            kind: Kind::Cynefin,
        }
    }

    pub(crate) fn er() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::er::Er>(),
            kind: Kind::Er,
        }
    }

    pub(crate) fn mindmap() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::mindmap::Mindmap>(),
            kind: Kind::Mindmap,
        }
    }

    pub(crate) fn treemap() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::treemap::Treemap>(),
            kind: Kind::Treemap,
        }
    }

    pub(crate) fn kanban() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::kanban::Kanban>(),
            kind: Kind::Kanban,
        }
    }

    pub(crate) fn tree_view() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::tree_view::TreeView>(),
            kind: Kind::TreeView,
        }
    }

    pub(crate) fn journey() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::journey::Journey>(),
            kind: Kind::Journey,
        }
    }
    pub(crate) fn timeline() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::timeline::Timeline>(),
            kind: Kind::Timeline,
        }
    }

    pub(crate) fn ishikawa() -> Self {
        Self {
            creator: TypeId::of::<Ishikawa>(),
            kind: Kind::Ishikawa,
        }
    }

    pub(crate) fn venn() -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::venn::Venn>(),
            kind: Kind::Venn,
        }
    }

    pub(crate) fn quadrant() -> Self {
        Self {
            creator: TypeId::of::<QuadrantChart>(),
            kind: Kind::Quadrant,
        }
    }

    pub(crate) fn packet() -> Self {
        Self {
            creator: TypeId::of::<Packet>(),
            kind: Kind::Packet,
        }
    }
    pub(crate) fn radar() -> Self {
        Self {
            creator: TypeId::of::<Radar>(),
            kind: Kind::Radar,
        }
    }

    pub(crate) fn pie() -> Self {
        Self {
            creator: TypeId::of::<Pie>(),
            kind: Kind::Pie,
        }
    }
    pub(crate) fn info() -> Self {
        Self {
            creator: TypeId::of::<Info>(),
            kind: Kind::Info,
        }
    }

    pub(crate) fn sankey() -> Self {
        Self {
            creator: TypeId::of::<Sankey>(),
            kind: Kind::Sankey,
        }
    }
    pub(super) fn for_creator<D: 'static>(self) -> Option<Self> {
        (self.creator == TypeId::of::<D>()).then_some(self)
    }
}
/// Owned builtin input. Public providers cannot construct or replace its body.
#[doc(hidden)]
pub struct DetachedPublicRender {
    creator: TypeId,
    body: Body,
}
enum Body {
    Agentflow(crate::compat::diagrams::agentflow::AgentflowDetachedInput),
    Sequence(crate::compat::diagrams::sequence::SequenceDetachedInput),
    Class(crate::compat::diagrams::class_diagram::ClassDetachedInput),
    XYChart(crate::compat::diagrams::xychart::XYChartDetachedInput),
    Swimlane(crate::compat::diagrams::swimlane::SwimlaneDetachedInput),
    Flowchart(crate::compat::diagrams::flowchart::FlowchartDetachedInput),
    Block(crate::compat::diagrams::block::BlockDetachedInput),
    State(crate::compat::diagrams::state::StateDetachedInput),
    Architecture(crate::compat::diagrams::architecture::ArchitectureDetachedInput),
    EventModeling(crate::compat::diagrams::event_modeling::EventModelingDetachedInput),
    Usecase(crate::compat::diagrams::usecase::UsecaseDetachedInput),
    GitGraph(crate::compat::diagrams::git_graph::GitGraphDetachedInput),
    Railroad(crate::compat::diagrams::railroad::RailroadDetachedInput),
    Requirement(crate::compat::diagrams::requirement::RequirementDetachedInput),
    C4(crate::compat::diagrams::c4::C4DetachedInput),
    Wardley(crate::compat::diagrams::wardley::WardleyDetachedInput),
    Cynefin(crate::compat::diagrams::cynefin::CynefinDetachedInput),
    Er(crate::compat::diagrams::er::ErDetachedInput),
    Mindmap(crate::compat::diagrams::mindmap::MindmapDetachedInput),
    Treemap(crate::compat::diagrams::treemap::TreemapDetachedInput),
    Kanban(crate::compat::diagrams::kanban::KanbanDetachedInput),
    TreeView(crate::compat::diagrams::tree_view::TreeViewDetachedInput),
    Journey(crate::compat::diagrams::journey::JourneyDetachedInput),
    Timeline(crate::compat::diagrams::timeline::TimelineDetachedInput),
    Ishikawa(IshikawaDetachedInput),
    Venn(crate::compat::diagrams::venn::VennDetachedInput),
    Quadrant(QuadrantDetachedInput),
    Packet(PacketDetachedInput),
    Radar(RadarDetachedInput),
    Info(InfoDetachedInput),
    Sankey(SankeyDetachedInput),
    Pie(PieDetachedInput),
}
impl DetachedPublicRender {
    pub(crate) fn agentflow(
        input: crate::compat::diagrams::agentflow::AgentflowDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::agentflow::AgentFlow>(),
            body: Body::Agentflow(input),
        }
    }

    pub(crate) fn sequence(
        input: crate::compat::diagrams::sequence::SequenceDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::sequence::Sequence>(),
            body: Body::Sequence(input),
        }
    }

    pub(crate) fn class(input: crate::compat::diagrams::class_diagram::ClassDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::class_diagram::Class>(),
            body: Body::Class(input),
        }
    }

    pub(crate) fn xychart(input: crate::compat::diagrams::xychart::XYChartDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::xychart::XyChart>(),
            body: Body::XYChart(input),
        }
    }

    pub(crate) fn swimlane(
        input: crate::compat::diagrams::swimlane::SwimlaneDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::swimlane::Swimlane>(),
            body: Body::Swimlane(input),
        }
    }

    pub(crate) fn flowchart(
        input: crate::compat::diagrams::flowchart::FlowchartDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::flowchart::Flowchart>(),
            body: Body::Flowchart(input),
        }
    }

    pub(crate) fn block(input: crate::compat::diagrams::block::BlockDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::block::Block>(),
            body: Body::Block(input),
        }
    }

    pub(crate) fn state(input: crate::compat::diagrams::state::StateDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::state::State>(),
            body: Body::State(input),
        }
    }

    pub(crate) fn architecture(
        input: crate::compat::diagrams::architecture::ArchitectureDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::architecture::Architecture>(),
            body: Body::Architecture(input),
        }
    }

    pub(crate) fn eventmodeling(
        input: crate::compat::diagrams::event_modeling::EventModelingDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::event_modeling::EventModeling>(),
            body: Body::EventModeling(input),
        }
    }

    pub(crate) fn usecase(input: crate::compat::diagrams::usecase::UsecaseDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::usecase::Usecase>(),
            body: Body::Usecase(input),
        }
    }

    pub(crate) fn git_graph(
        input: crate::compat::diagrams::git_graph::GitGraphDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::git_graph::GitGraph>(),
            body: Body::GitGraph(input),
        }
    }

    pub(crate) fn railroad(
        input: crate::compat::diagrams::railroad::RailroadDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::railroad::Railroad>(),
            body: Body::Railroad(input),
        }
    }

    pub(crate) fn requirement(
        input: crate::compat::diagrams::requirement::RequirementDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::requirement::RequirementDiagram>(),
            body: Body::Requirement(input),
        }
    }

    pub(crate) fn c4(input: crate::compat::diagrams::c4::C4DetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::c4::C4>(),
            body: Body::C4(input),
        }
    }

    pub(crate) fn wardley(input: crate::compat::diagrams::wardley::WardleyDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::wardley::Wardley>(),
            body: Body::Wardley(input),
        }
    }

    pub(crate) fn cynefin(input: crate::compat::diagrams::cynefin::CynefinDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::cynefin::Cynefin>(),
            body: Body::Cynefin(input),
        }
    }

    pub(crate) fn er(input: crate::compat::diagrams::er::ErDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::er::Er>(),
            body: Body::Er(input),
        }
    }

    pub(crate) fn mindmap(input: crate::compat::diagrams::mindmap::MindmapDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::mindmap::Mindmap>(),
            body: Body::Mindmap(input),
        }
    }

    pub(crate) fn treemap(input: crate::compat::diagrams::treemap::TreemapDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::treemap::Treemap>(),
            body: Body::Treemap(input),
        }
    }

    pub(crate) fn kanban(input: crate::compat::diagrams::kanban::KanbanDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::kanban::Kanban>(),
            body: Body::Kanban(input),
        }
    }

    pub(crate) fn tree_view(
        input: crate::compat::diagrams::tree_view::TreeViewDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::tree_view::TreeView>(),
            body: Body::TreeView(input),
        }
    }

    pub(crate) fn journey(input: crate::compat::diagrams::journey::JourneyDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::journey::Journey>(),
            body: Body::Journey(input),
        }
    }
    pub(crate) fn timeline(
        input: crate::compat::diagrams::timeline::TimelineDetachedInput,
    ) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::timeline::Timeline>(),
            body: Body::Timeline(input),
        }
    }

    pub(crate) fn ishikawa(input: IshikawaDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Ishikawa>(),
            body: Body::Ishikawa(input),
        }
    }

    pub(crate) fn venn(input: crate::compat::diagrams::venn::VennDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<crate::compat::diagrams::venn::Venn>(),
            body: Body::Venn(input),
        }
    }

    pub(crate) fn quadrant(input: QuadrantDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<QuadrantChart>(),
            body: Body::Quadrant(input),
        }
    }

    pub(crate) fn packet(input: PacketDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Packet>(),
            body: Body::Packet(input),
        }
    }
    pub(crate) fn radar(input: RadarDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Radar>(),
            body: Body::Radar(input),
        }
    }

    pub(crate) fn pie(input: PieDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Pie>(),
            body: Body::Pie(input),
        }
    }
    pub(crate) fn info(input: InfoDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Info>(),
            body: Body::Info(input),
        }
    }

    pub(crate) fn sankey(input: SankeyDetachedInput) -> Self {
        Self {
            creator: TypeId::of::<Sankey>(),
            body: Body::Sankey(input),
        }
    }
    pub(super) fn for_creator<D: 'static>(
        self,
        capability: &DetachedRenderCapability,
    ) -> Option<Self> {
        let kind = match &self.body {
            Body::Agentflow(_) => Kind::Agentflow,
            Body::Sequence(_) => Kind::Sequence,
            Body::XYChart(_) => Kind::XYChart,
            Body::Swimlane(_) => Kind::Swimlane,
            Body::Flowchart(_) => Kind::Flowchart,
            Body::Block(_) => Kind::Block,
            Body::State(_) => Kind::State,
            Body::Class(_) => Kind::Class,
            Body::Architecture(_) => Kind::Architecture,
            Body::EventModeling(_) => Kind::EventModeling,
            Body::Usecase(_) => Kind::Usecase,
            Body::GitGraph(_) => Kind::GitGraph,
            Body::Railroad(_) => Kind::Railroad,
            Body::Requirement(_) => Kind::Requirement,
            Body::C4(_) => Kind::C4,
            Body::Wardley(_) => Kind::Wardley,
            Body::Cynefin(_) => Kind::Cynefin,
            Body::Er(_) => Kind::Er,
            Body::Mindmap(_) => Kind::Mindmap,
            Body::Treemap(_) => Kind::Treemap,
            Body::Kanban(_) => Kind::Kanban,
            Body::TreeView(_) => Kind::TreeView,
            Body::Timeline(_) => Kind::Timeline,
            Body::Ishikawa(_) => Kind::Ishikawa,
            Body::Journey(_) => Kind::Journey,
            Body::Venn(_) => Kind::Venn,
            Body::Quadrant(_) => Kind::Quadrant,
            Body::Radar(_) => Kind::Radar,
            Body::Packet(_) => Kind::Packet,
            Body::Info(_) => Kind::Info,
            Body::Sankey(_) => Kind::Sankey,
            Body::Pie(_) => Kind::Pie,
        };
        (self.creator == TypeId::of::<D>()
            && self.creator == capability.creator
            && kind == capability.kind)
            .then_some(self)
    }
    fn render<R: LiveRenderRead>(
        &self,
        config: &Object,
        settings: &RenderSettings,
        read: &mut R,
        resources: &mut dyn RenderResources,
    ) -> Result<Rendered, PublicError> {
        match &self.body {
            Body::Agentflow(input) => crate::compat::diagrams::agentflow::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Sequence(input) => crate::compat::diagrams::sequence::render_detached(
                input, config, settings, read, resources,
            ),
            Body::XYChart(input) => crate::compat::diagrams::xychart::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Swimlane(input) => crate::compat::diagrams::swimlane::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Flowchart(input) => crate::compat::diagrams::flowchart::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Block(input) => crate::compat::diagrams::block::render_detached(
                input, config, settings, read, resources,
            ),
            Body::State(input) => crate::compat::diagrams::state::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Class(input) => crate::compat::diagrams::class_diagram::render_detached(
                input, read, config, settings, resources,
            ),
            Body::Architecture(input) => crate::compat::diagrams::architecture::render_detached(
                input, read, config, settings, resources,
            ),
            Body::EventModeling(input) => crate::compat::diagrams::event_modeling::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Usecase(input) => crate::compat::diagrams::usecase::render_detached(
                input, config, settings, read, resources,
            ),
            Body::GitGraph(input) => crate::compat::diagrams::git_graph::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Railroad(input) => crate::compat::diagrams::railroad::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Requirement(input) => crate::compat::diagrams::requirement::render_detached(
                input, config, settings, read, resources,
            ),
            Body::C4(input) => crate::compat::diagrams::c4::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Wardley(input) => crate::compat::diagrams::wardley::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Cynefin(input) => crate::compat::diagrams::cynefin::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Er(input) => crate::compat::diagrams::er::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Mindmap(input) => crate::compat::diagrams::mindmap::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Treemap(input) => crate::compat::diagrams::treemap::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Kanban(input) => crate::compat::diagrams::kanban::render_detached(
                input, config, settings, read, resources,
            ),
            Body::TreeView(input) => crate::compat::diagrams::tree_view::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Timeline(input) => crate::compat::diagrams::timeline::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Journey(input) => crate::compat::diagrams::journey::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Ishikawa(input) => crate::compat::diagrams::ishikawa::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Venn(input) => crate::compat::diagrams::venn::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Quadrant(input) => crate::compat::diagrams::quadrant_chart::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Radar(input) => crate::compat::diagrams::radar::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Packet(input) => crate::compat::diagrams::packet::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Info(input) => crate::compat::diagrams::info::render_detached(
                input, config, settings, read, resources,
            ),
            Body::Sankey(input) => render_detached(input, config, settings, read, resources),
            Body::Pie(input) => crate::compat::diagrams::pie::render_detached(
                input, config, settings, read, resources,
            ),
        }
    }
}

/// Already-decoded options. Raw config is the caller's explicit low-level config.
#[doc(hidden)]
pub enum DetachedRenderRequest {
    Normal {
        options: RenderOptions,
        limit: Option<OutputLimit>,
        enforce_native_bytes: bool,
    },
    Raw {
        config: Object,
        settings: RenderSettings,
    },
}
#[doc(hidden)]
pub enum DetachedRenderValue {
    Normal(RenderResult),
    Raw(Rendered),
}

pub(super) struct RenderBook {
    next_frame: u64,
    outstanding: u64,
    frames: Vec<Frame>,
}
struct Frame {
    id: u64,
    active: bool,
}
impl RenderBook {
    pub(super) fn new() -> Self {
        Self {
            next_frame: 0,
            outstanding: 0,
            frames: Vec::new(),
        }
    }
    fn trim(&mut self) {
        while self.frames.last().is_some_and(|frame| !frame.active) {
            self.frames.pop();
        }
    }
    pub(super) fn active(&self) -> bool {
        self.outstanding != 0
    }
}
fn book_lock(book: &Mutex<RenderBook>) -> MutexGuard<'_, RenderBook> {
    // No user callback executes under this lock. Recover only bookkeeping if a
    // native unwind interrupted a caller; never invent a document mutation.
    book.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
fn invalid(message: &str) -> PeleError {
    PeleError::new(message, ErrorCode::Semantic)
}
fn exhausted() -> PeleError {
    PeleError::new("Public model revision exhausted", ErrorCode::Limit)
}
pub(super) fn next_revision(
    revision: u64,
    book: Option<&Arc<Mutex<RenderBook>>>,
) -> Result<u64, PeleError> {
    let reserved = book.map_or(0, |book| book_lock(book).outstanding);
    revision
        .checked_add(reserved)
        .and_then(|v| v.checked_add(1))
        .ok_or_else(exhausted)?;
    Ok(revision + 1)
}
pub(super) fn has_active(book: Option<&Arc<Mutex<RenderBook>>>) -> bool {
    book.is_some_and(|book| book_lock(book).active())
}

pub(super) struct RenderLease {
    owner: Weak<Mutex<RenderBook>>,
    id: u64,
    active: bool,
}
impl RenderLease {
    pub(super) fn reserve(
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<Self, PeleError> {
        let mut book = book_lock(owner);
        revision
            .checked_add(book.outstanding)
            .and_then(|v| v.checked_add(1))
            .ok_or_else(exhausted)?;
        let id = book.next_frame.checked_add(1).ok_or_else(|| {
            PeleError::new("Public render frame identity exhausted", ErrorCode::Limit)
        })?;
        book.frames.push(Frame { id, active: true });
        book.next_frame = id;
        book.outstanding += 1;
        Ok(Self {
            owner: Arc::downgrade(owner),
            id,
            active: true,
        })
    }
    pub(super) fn check_owner(&self, expected: &Arc<Mutex<RenderBook>>) -> Result<(), PeleError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| invalid("Public render document is disposed"))?;
        if !Arc::ptr_eq(&owner, expected) {
            return Err(invalid("Public render job belongs to another document"));
        }
        self.check_top()
    }
    fn check_top(&self) -> Result<(), PeleError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| invalid("Public render document is disposed"))?;
        let book = book_lock(&owner);
        if !self.active
            || !book
                .frames
                .last()
                .is_some_and(|frame| frame.active && frame.id == self.id)
        {
            return Err(invalid("Public render jobs must finish in nesting order"));
        }
        Ok(())
    }
    pub(super) fn finish(
        &mut self,
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<u64, PeleError> {
        self.check_owner(owner)?;
        let next = revision.checked_add(1).ok_or_else(exhausted)?;
        self.release(owner)?;
        Ok(next)
    }
    pub(super) fn release(
        &mut self,
        expected_owner: &Arc<Mutex<RenderBook>>,
    ) -> Result<(), PeleError> {
        let owner = self
            .owner
            .upgrade()
            .ok_or_else(|| invalid("Public render document is disposed"))?;
        if !Arc::ptr_eq(&owner, expected_owner) {
            return Err(invalid("Public render job belongs to another document"));
        }
        let mut book = book_lock(&owner);
        if !self.active
            || !book
                .frames
                .last()
                .is_some_and(|frame| frame.active && frame.id == self.id)
        {
            return Err(invalid("Public render jobs must finish in nesting order"));
        }
        book.frames.pop();
        book.outstanding -= 1;
        book.trim();
        self.active = false;
        Ok(())
    }
}
impl Drop for RenderLease {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        if let Some(owner) = self.owner.upgrade() {
            let mut book = book_lock(&owner);
            if let Some(frame) = book
                .frames
                .iter_mut()
                .find(|frame| frame.id == self.id && frame.active)
            {
                frame.active = false;
                book.outstanding -= 1;
                book.trim();
            }
        }
    }
}

/// An owned render stack frame. It never holds PublicDocument, its graph or state.
#[doc(hidden)]
pub struct DetachedRenderJob {
    lease: RenderLease,
    registration: Arc<Registration>,
    config: Object,
    request: DetachedRenderRequest,
    input: DetachedPublicRender,
}
/// The same one-use frame after execution, including dispatched operation errors.
#[doc(hidden)]
pub struct DetachedRenderCompletion {
    lease: RenderLease,
    result: Result<DetachedRenderValue, PublicError>,
    presentation: Option<PresentationOutcome>,
}
/// Only created after resource execution. Drop its graph borrow before any host
/// presentation or result-factory callback. It is not a reconciliation request.
#[doc(hidden)]
pub struct DetachedRenderOutcome<'a> {
    pub revision: u64,
    pub graph: GraphReadView<'a>,
    pub result: Result<DetachedRenderValue, PublicError>,
    pub presentation: Option<PresentationOutcome>,
}
impl DetachedRenderJob {
    pub(super) fn new(
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
        registration: Arc<Registration>,
        config: Object,
        request: DetachedRenderRequest,
        input: DetachedPublicRender,
    ) -> Result<Self, PeleError> {
        Ok(Self {
            lease: RenderLease::reserve(owner, revision)?,
            registration,
            config,
            request,
            input,
        })
    }
    /// Consume the job without borrowing its document. Original JavaScript
    /// exceptions are retained by the caller's per-job channel, not this value.
    pub fn run<R: LiveRenderRead>(
        self,
        read: &mut R,
        resources: &mut dyn RenderResources,
    ) -> DetachedRenderCompletion {
        let Self {
            lease,
            registration,
            config,
            request,
            input,
        } = self;
        if let Err(error) = lease.check_top() {
            return DetachedRenderCompletion {
                lease,
                result: Err(error.into()),
                presentation: None,
            };
        }
        let mut operation_error = None;
        let (result, presentation) = match request {
            DetachedRenderRequest::Normal {
                options,
                limit,
                enforce_native_bytes,
            } => {
                let mut presentation = PresentationCapture::deferred(resources);
                let result = crate::compat::engine::render_registered_policy(
                    &registration.descriptor,
                    &config,
                    &options,
                    &mut presentation,
                    limit,
                    enforce_native_bytes,
                    |settings, host| {
                        input
                            .render(&config, settings, read, host)
                            .map_err(|error| {
                                operation_error = Some(error);
                                invalid("Public render operation failed")
                            })
                    },
                )
                .map(DetachedRenderValue::Normal)
                .map_err(|error| operation_error.unwrap_or_else(|| error.into()));
                (result, presentation.take())
            }
            DetachedRenderRequest::Raw { config, settings } => {
                let result = crate::compat::engine::render_raw_registered(
                    &registration.descriptor,
                    &settings,
                    resources,
                    |host| {
                        input
                            .render(&config, &settings, read, host)
                            .map_err(|error| {
                                operation_error = Some(error);
                                invalid("Public raw render operation failed")
                            })
                    },
                )
                .map(DetachedRenderValue::Raw)
                .map_err(|error| operation_error.unwrap_or_else(|| error.into()));
                (result, None)
            }
        };
        DetachedRenderCompletion {
            lease,
            result,
            presentation,
        }
    }
}
impl DetachedRenderCompletion {
    pub(super) fn finish(
        mut self,
        owner: &Arc<Mutex<RenderBook>>,
        revision: u64,
    ) -> Result<
        (
            u64,
            Result<DetachedRenderValue, PublicError>,
            Option<PresentationOutcome>,
        ),
        PeleError,
    > {
        let next = self.lease.finish(owner, revision)?;
        Ok((next, self.result, self.presentation))
    }
}

/// Native live readers share the canonical JavaScript number formatter.
#[doc(hidden)]
pub fn live_number_string(value: f64) -> String {
    crate::compat::svg::js_num(value)
}
