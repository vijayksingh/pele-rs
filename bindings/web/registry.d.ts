import type {DiagramId, DiagramType} from './core.js';
import type {Renderer} from './full.js';
export {PeleError, PeleCompatibilityError} from './full.js';
/** Owns native built-in and caller-authored registrations; retained models survive close(). */
export interface Registry extends Renderer {
  registered(type: DiagramId): boolean;
  registerBuiltin(type: DiagramType): void;
  detectType(source: string): DiagramType | null;
  supports(source: string): boolean;
  close(): void;
}
export function loadRegistry(
  moduleOrPath?: string | URL | Response | BufferSource | WebAssembly.Module,
  options?: {allBuiltins?: boolean},
): Promise<Registry>;

export type {DiagramType} from './core.js';
export type {
  PeleErrorCode, TextMeasurer, IconResolver, LinkInfo, RenderOptions, RenderResult, Diagram, MountOptions, Mounted, Zoom, ZoomOptions, ParseOptions,
  DiagramModel, AgentflowModel, ArchitectureModel, BlockModel, C4Model, ClassModel, CynefinModel, ErModel, EventModelingModel, FlowchartModel, GanttModel, GitModel, InfoModel, IshikawaModel, JourneyModel, KanbanModel, MindmapModel, PacketModel, PieModel, QuadrantModel, RadarModel, RailroadModel, RequirementModel, SankeyModel, SequenceModel, StateModel, SwimlaneModel, TimelineModel, TreeViewModel, TreemapModel, UsecaseModel, VennModel, WardleyModel, XyChartModel
} from './full.js';
