import type {DiagramId} from './core.js';
import type {MountOptions, Mounted, RenderOptions, Rendered} from './full.js';
import type {Registry} from './registry.js';
export {loadRegistry, type Registry} from './registry.js';
export {PeleError, PeleCompatibilityError, enableZoom, forgetTextWidths} from './full.js';
export type {
  ParseOptions, RenderOptions, PieModel, AccessibleModel, PacketBlock,
  PacketModel, RadarAxis, RadarCurve, RadarModel, QuadrantText,
  PointStyle, QuadrantPoint, QuadrantModel, TimelinePeriod, TimelineModel,
  JourneyTask, JourneyModel, FlowNode, FlowEdge, FlowClassDef,
  FlowSubgraph, FlowText, FlowLinkInfo, FlowDocItem, FlowchartModel,
  KanbanNode, KanbanSection, KanbanModel, MindmapNode, MindmapModel,
  ClassMember, ClassNode, ClassRelation, ClassNote, ClassNamespace,
  ClassModel, ErAttribute, ErEntity, ErCardinality, ErRelationship,
  ErSubgraph, ErModel, XyLinearAxis, XyBandAxis, XyPlot,
  XyChartModel, StateDeclaration, StateStatement, StateNode, StateEdge,
  StateModel, GanttTask, GanttModel, SequenceBox, SequenceActor,
  SequenceNumbering, SequenceMessage, SequenceNote, SequenceText, SequenceBoxData,
  SequenceStatement, SequenceItem, SequenceModel, GitCommit, GitModel,
  ArchitectureNode, ArchitectureGroup, ArchitectureEdge, ArchitectureModel, SankeyNode,
  SankeyModel, TreemapNode, TreemapModel, InfoModel, C4Named,
  C4Attr, C4Element, C4Shape, C4Boundary, C4Relation,
  C4Model, RequirementNode, RequirementElement, RequirementRelation, RequirementClass,
  RequirementModel, BlockNode, BlockClass, BlockStatement, BlockModel,
  RailroadNode, RailroadRule, RailroadModel, CynefinDomainName, CynefinItem,
  CynefinDomain, CynefinTransition, CynefinModel, TreeViewNode, TreeViewModel,
  AgentNode, AgentLoc, AgentPosition, AgentMapping, AgentDiagnostic,
  AgentSemanticModel, AgentflowModel, WardleySourceStrategy, WardleyFlow, WardleyNode,
  WardleyPoint, WardleyLink, WardleyTrend, WardleyPipeline, WardleyAnnotation,
  WardleyNote, WardleyForce, WardleyAxes, WardleyModel, EventLane,
  EventBox, EventModelingModel, SourceSpan, UsecaseLabelType, UsecaseDirection,
  UsecaseActor, UseCase, UsecaseBoundary, UsecaseRelationship, UsecaseNote,
  UsecaseJsonNode, UsecaseClassDef, UsecaseMetadataOccurrence, UsecaseNodeOccurrence, UsecaseEdgeOccurrence,
  UsecaseGraphStatement, UsecaseModel, VennSubset, VennText, VennStyle,
  VennModel, IshikawaNode, IshikawaModel, DiagramModel, Rendered,
  ZoomOptions, Zoom, MountOptions, Mounted, Renderer,
  TextMeasurer, PeleErrorCode,
} from './full.js';
export interface LazyRegistry extends Registry {
  load(type: DiagramId | null): Promise<boolean>;
  renderAsync(source: string, options?: RenderOptions): Promise<Rendered>;
  mountAsync(element: HTMLElement, source: string, options?: MountOptions): Promise<Mounted>;
}
export function withLazy(registry: Registry): LazyRegistry;
export function loadLazy(moduleOrPath?: string | URL | Response | BufferSource | WebAssembly.Module): Promise<LazyRegistry>;

export type {Diagram, DiagramType, SwimlaneModel, IconResolver, LinkInfo, RenderResult} from './full.js';
