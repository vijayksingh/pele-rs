export {loadRegistry, type Registry} from './registry.js';
import type { ConfigValue, DiagramId, Capability } from './core.js';
export {PeleError, PeleCompatibilityError, type PeleErrorCode} from './core.js';
export interface ParseOptions { limit?: number; maxEdges?: number; config?: { [key: string]: ConfigValue } }
export interface RenderOptions extends ParseOptions {
  measurer?: { width(text: string, size: number, style: number): number };
  icons?: (name: string) => string | null | undefined;
  nativeTheme?: { background?: string; foreground?: string; muted?: string; line?: string; surface?: string; surfaceAlt?: string; border?: string; accent?: string; series?: [string,string,string,string,string,string,string,string] };
  fontFamily?: string; fontFamilyMono?: string; fontSize?: number; idPrefix?: string;
  responsive?: boolean; maxWidth?: number; autoDirection?: boolean; directionBreakpoint?: number;
  padding?: number; outputLimit?: number; linkSchemes?: string[]; imageSchemes?: string[];
  links?: boolean; images?: boolean; linkRel?: string; now?: number | Date;
}
/** Caller-authored extension. Config uses bounded finite JSON values; models
 * retain their original identity and methods without a graph codec. */
export interface Diagram<Model> {
  type: import('./core.js').DiagramType;
  section?: string;
  keepComments?: boolean;
  parse(source: string, config: { [key: string]: ConfigValue }, title: string | undefined,
    lineOffset: number, limits: {maxEdges: number}): Model;
  render(model: Model, config: { [key: string]: ConfigValue }, options: RenderOptions): ProviderRendered;
}
export interface ProviderRendered {
  svg: string; width: number; height: number;
  links: {id: string; href: string; internal: boolean}[];
}
export interface PieModel { type: 'pie'; title?: string; accTitle?: string; accDescr?: string; showData: boolean; sections: Map<string, number> }
export interface AccessibleModel { title?: string; accTitle?: string; accDescr?: string }
export interface PacketBlock { start: number; end: number; bits: number; label: string }
export interface PacketModel extends AccessibleModel { type: 'packet'; bitsPerRow: number; rows: PacketBlock[][] }
export interface RadarAxis { name: string; label: string }
export interface RadarCurve { name: string; label: string; entries: number[] }
export interface RadarModel extends AccessibleModel { type: 'radar'; axes: RadarAxis[]; curves: RadarCurve[]; options: { showLegend: boolean; ticks: number; max: number | null; min: number; graticule: 'circle' | 'polygon' } }
export interface QuadrantText { text: string; type: string }
export interface PointStyle { radius?: number; color?: string; strokeColor?: string; strokeWidth?: string }
export interface QuadrantPoint extends PointStyle { x: number; y: number; text: QuadrantText; className: string }
export interface QuadrantModel extends AccessibleModel {
 setQuadrant1Text(label:QuadrantText):void;setQuadrant2Text(label:QuadrantText):void;setQuadrant3Text(label:QuadrantText):void;setQuadrant4Text(label:QuadrantText):void;
 setXAxisLeftText(label:QuadrantText):void;setXAxisRightText(label:QuadrantText):void;setYAxisBottomText(label:QuadrantText):void;setYAxisTopText(label:QuadrantText):void;
 addPoint(label:QuadrantText,className:string,x:string,y:string,styles:string[]):void;addClass(className:string,styles:string[]):void;
 setDiagramTitle(text:string):void;setAccTitle(text:string):void;setAccDescription(text:string):void;
 type: 'quadrantChart'; quadrants: [QuadrantText | undefined, QuadrantText | undefined, QuadrantText | undefined, QuadrantText | undefined]; xAxisLeft?: QuadrantText; xAxisRight?: QuadrantText; yAxisBottom?: QuadrantText; yAxisTop?: QuadrantText; points: QuadrantPoint[]; classes: Map<string, PointStyle> }
export interface TimelinePeriod { text: string; events: string[]; section: string; sectionIndex: number }
export interface TimelineModel extends AccessibleModel { getCommonDb():this; setDiagramTitle(text:string):void; setAccTitle(text:string):void; setAccDescription(text:string):void; setDirection(direction:'LR'|'TD'):void; addSection(text:string):void; addTask(period:string,length?:number,event?:string):void; addEvent(event:string):void; type: 'timeline'; direction: string; sections: string[]; periods: TimelinePeriod[]; currentSection: string }
export interface JourneyTask { task: string; score: number; people: string[]; section: string }
export interface JourneyModel extends AccessibleModel {
 setDiagramTitle(text:string):void;setAccTitle(text:string):void;setAccDescription(text:string):void;
 addSection(text:string):void;addTask(task:string,scoreAndPeople:string):void;getActors():string[];
 type: 'journey'; sections: string[]; tasks: JourneyTask[]; currentSection: string }
export interface FlowNode { id: string; text?: string; labelType: string; type?: string; styles: string[]; classes: string[]; dir?: string; props?: { [key: string]: ConfigValue }; link?: string; linkTarget?: string; icon?: string; form?: string; pos?: string; img?: string; constraint?: string; assetWidth?: ConfigValue; assetHeight?: ConfigValue }
export interface FlowEdge { start: string; end: string; type?: string; text: string; labelType: string; stroke?: string; length?: ConfigValue; style?: string[]; classes: string[]; id?: string; isUserDefinedId: boolean; interpolate?: ConfigValue; animate?: ConfigValue; animation?: ConfigValue }
export interface FlowClassDef { id: string; styles: string[]; textStyles: string[] }
export interface FlowSubgraph { id: string; nodes: string[]; title: string; classes: string[]; dir?: string; labelType: string; metadata?: { [key: string]: ConfigValue } }
export interface FlowText { text:string;type:'text'|'string'|'markdown' }
export interface FlowLinkInfo { type:string;stroke:string;length?:number;text?:FlowText;id?:string }
export type FlowDocItem=string|string[]|{stmt:'dir';value:string};
export interface FlowchartModel extends AccessibleModel {
 addVertex(id:string,text?:FlowText,type?:string,styles?:string[]|null,classes?:string[]|null,direction?:string,props?:Record<string,string>,metadata?:string):void;
 addSingleLink(start:string,end:string,link:FlowLinkInfo|undefined,id?:string):void;addLink(starts:string[],ends:string[],link:FlowLinkInfo|undefined):void;
 updateLinkInterpolate(positions:(number|'default')[],interpolation:string):void;updateLink(positions:(number|'default')[],styles:string[]):void;edgeAt(index:number):FlowEdge;
 addClass(ids:string,styles:string[]):void;setDirection(direction:string):void;setClass(ids:string,className:string):void;setTooltip(ids:string,tooltip:string|undefined):void;
 setLink(ids:string,link:string,target?:string):void;setClickEvent(ids:string,functionName?:string,functionArgs?:string):void;setAccTitle(text:string):void;setAccDescription(text:string):void;
 addSubGraph(id:FlowText,list:FlowDocItem[],title:FlowText):string;exists(all:FlowSubgraph[],id:string):boolean;makeUniq(group:FlowSubgraph,all:FlowSubgraph[]):{nodes:string[]};
 destructLink(end:string,start?:string):{type:string;stroke:string;length?:number};subgraph(id:string):FlowSubgraph|undefined;
 subgraphLookup:Map<string,FlowSubgraph>;members:Set<string>;edgeById:Map<string,FlowEdge>;pairCount:Map<string,number>;subCount:number;options:{maxEdges?:number;inheritDir?:boolean;direction?:string};
 type: 'flowchart'; direction?: string; nodes: Map<string, FlowNode>; edges: FlowEdge[]; subgraphs: FlowSubgraph[]; classes: Map<string, FlowClassDef>; tooltips: Map<string, string>; defaultEdgeStyle?: string[]; defaultInterpolate?: ConfigValue }
export interface KanbanNode { id: string; level: number; label: string; icon?: string; assigned?: string; ticket?: string; priority?: string; cssClasses?: string }
export interface KanbanSection extends KanbanNode { items: KanbanNode[] }
export interface KanbanModel { getType(start:string,end?:string):number; addNode(level:number,id:string,description:string,type:number,shapeData?:string):void; decorateNode(decoration?:{class?:string;icon?:string}):void; type: 'kanban'; title?: string; sections: KanbanSection[]; last?: KanbanNode | KanbanSection; sectionLevel: number; misplaced?: KanbanNode }
export interface MindmapNode { id: number; nodeId: string; level: number; descr: string; type: number; children: MindmapNode[]; isRoot: boolean; section?: number; class?: string; icon?: string }
export interface MindmapModel { getType(start:string,end?:string):number; addNode(level:number,id:string,description:string,type:number):void; decorateNode(decoration?:{class?:string;icon?:string}):void; type: 'mindmap'; title?: string; nodes: MindmapNode[]; baseLevel: number; open: MindmapNode[] }
export interface ClassMember { parseMethod(input:string):void;parseAttribute(input:string):void;getDisplayDetails():{displayText:string;cssStyle:string};parseClassifier():string; id: string; memberType: string; visibility: string; classifier: string; parameters: string; returnType: string }
export interface ClassNode { id: string; type: string; label: string; cssClasses: string; methods: ClassMember[]; members: ClassMember[]; annotations: string[]; styles: string[]; parent?: string; link?: string; linkTarget?: string; haveCallback?: boolean; tooltip?: string }
export interface ClassRelation { id1: string; id2: string; relationTitle1: string; relationTitle2: string; title?: string; relation: { type1: number | 'none'; type2: number | 'none'; lineType: number } }
export interface ClassNote { id: string; class?: string; text: string; index: number; parent?: string }
export interface ClassNamespace { id: string; label: string; classes: Map<string, ClassNode>; notes: Map<string, ClassNote>; children: Map<string, ClassNamespace>; parent?: string; explicit: boolean }
export interface ClassModel extends AccessibleModel {
 addClass(id:string):void;tag(node:ClassNode,cssClass:string):void;ensure(id:string):ClassNode;setClassLabel(id:string,label:string):void;
 addRelation(relation:ClassRelation):void;addInterface(label:string,classId:string):string;addAnnotation(className:string,annotation:string):void;
 addMember(className:string,member:string):void;addMembers(className:string,members:string[]):void;addNote(text:string,className?:string):string;cleanupLabel(label:string):string;
 setCssClass(ids:string,className:string):void;defineClass(ids:string[],styles:string[]):void;setTooltip(ids:string,tooltip?:string):void;setLink(ids:string,link:string,target?:string):void;
 setClickEvent(ids:string,functionName?:string,functionArgs?:string):void;setDirection(direction:string):void;setAccTitle(text:string):void;setAccDescription(text:string):void;
 addNamespace(id:string,label?:string):string;popNamespace():void;addClassesToNamespace(id:string,classNames:string[],noteNames:string[]):void;setCssStyle(id:string,styles:string[]):void;
 type: 'class'; lineType: { LINE: 0; DOTTED_LINE: 1 }; relationType: { AGGREGATION: 0; EXTENSION: 1; COMPOSITION: 2; DEPENDENCY: 3; LOLLIPOP: 4 }; direction: string; classes: Map<string, ClassNode>; relations: ClassRelation[]; notes: Map<string, ClassNote>; namespaces: Map<string, ClassNamespace>; interfaces: { id: string; label: string; classId: string }[]; styleClasses: Map<string, FlowClassDef>; namespaceStack: string[]; withCssClass: Map<string, Set<ClassNode>> }
export interface ErAttribute { type: string; name: string; keys: string[]; comment: string }
export interface ErEntity { id: string; label: string; alias: string; attributes: ErAttribute[]; cssClasses: string; cssStyles: string[] }
export type ErCardinality = 'ZERO_OR_ONE' | 'ZERO_OR_MORE' | 'ONE_OR_MORE' | 'ONLY_ONE' | 'MD_PARENT';
export interface ErRelationship { entityA: string; roleA: string; entityB: string; relSpec: { cardA: ErCardinality; relType: 'NON_IDENTIFYING' | 'IDENTIFYING'; cardB: ErCardinality } }
export interface ErSubgraph extends FlowSubgraph { cssStyles: string[] }
export interface ErModel extends AccessibleModel {
 addEntity(name:string,alias?:string):ErEntity;addAttributes(name:string,attributes:{type:string;name:string;keys?:string[];comment?:string}[]):void;
 addRelationship(entityA:string,role:string,entityB:string,spec:ErRelationship['relSpec']):void;setDirection(direction:string):void;
 addCssStyles(ids:string[],styles:string[]|undefined):void;addClass(ids:string[],styles:string[]|undefined):void;setClass(ids:string[],classes:string[]):void;
 addSubGraph(id:{text:string},list:FlowDocItem[],title:{text:string;type?:string}):string;setAccTitle(text:string):void;setAccDescription(text:string):void;
 subgraphLookup:Map<string,ErSubgraph>;claimed:Set<string>;
 type: 'er'; direction: string; entities: Map<string, ErEntity>; relationships: ErRelationship[]; classes: Map<string, FlowClassDef>; subgraphs: ErSubgraph[] }
export interface XyLinearAxis { type: 'linear'; title: string; min: number | null; max: number | null }
export interface XyBandAxis { type: 'band'; title: string; categories: string[] }
export interface XyPlot { type: 'line' | 'bar'; title: string; data: [string, number | null][]; pointLabels?: string[] }
export interface XyChartModel extends AccessibleModel {
 setOrientation(orientation:string):void;setXAxisTitle(title:QuadrantText):void;setXAxisRangeData(min:number,max:number):void;setXAxisBand(categories:QuadrantText[]):void;
 setYAxisTitle(title:QuadrantText):void;setYAxisRangeData(min:number,max:number):void;
 plotData(values:number[]):[string,number|undefined][];addPlot(type:'line'|'bar',title:QuadrantText,data:{value:number;label:string}[]):void;
 setLineData(title:QuadrantText,data:{value:number;label:string}[]):void;setBarData(title:QuadrantText,data:{value:number;label:string}[]):void;
 setDiagramTitle(text:string):void;setAccTitle(text:string):void;setAccDescription(text:string):void;
 type: 'xychart'; orientation?: 'vertical' | 'horizontal'; xAxis: XyLinearAxis | XyBandAxis; yAxis: XyLinearAxis; hasSetXAxis: boolean; hasSetYAxis: boolean; plots: XyPlot[] }
export interface StateDeclaration { stmt: 'state'; id: string; type?: string; description?: string | string[]; descriptions?: string[]; doc?: StateStatement[]; note?: { position?: string; text: string }; start?: boolean; classes?: string[]; styles?: string[]; textStyles?: string[] }
export type StateStatement = string | StateDeclaration | { stmt: string; [key: string]: unknown };
export interface StateNode { id: string; shape?: string; label?: string; description: string[]; cssClasses: string; cssCompiledStyles: string[]; cssStyles: string[]; type?: string; isGroup?: boolean; parentId?: string; dir?: string; position?: string; noteFor?: string; colorIndex?: number }
export interface StateEdge { id: string; start: string; end: string; label: string; classes: string; pattern?: string }
export interface StateModel extends AccessibleModel {
 setRootDoc(doc:StateStatement[]):void;extract(statements:StateStatement[]|{doc:StateStatement[]}):void;translate():void;
 addState(id:string,type?:string,doc?:StateStatement[],description?:string|string[],note?:{position?:string;text:string},classes?:string[]):void;
 getState(id:string):StateDeclaration|undefined;addLink(stateId:string,url:string,tooltip:string):void;addRelation(from:StateDeclaration,to:StateDeclaration,title?:string):void;
 addDescription(id:string,description:string):void;getDividerId():string;addStyleClass(id:string,styles?:string):void;setCssClass(ids:string,className:string):void;
 getDirection():string;setDirection(direction:string):void;trimColon(text:string):string;setAccTitle(text:string):void;setAccDescription(text:string):void;named(ids:string):StateDeclaration[];
 dividerCount:number;type:'state';readonly direction:string; rootDoc: StateStatement[]; states: Map<string, StateDeclaration>; relations: { id1: string; id2: string; relationTitle?: string }[]; classes: Map<string, FlowClassDef>; links: Map<string, { url: string; tooltip: string }>; graph: { nodes: StateNode[]; edges: StateEdge[] } }
export interface GanttTask { id: string; task: string; section: string; startTime: Date | undefined; endTime: Date | undefined; renderEndTime: Date | null; manualEndTime: boolean; processed: boolean; active: boolean; done: boolean; crit: boolean; milestone: boolean; vert: boolean; order: number; classes: string[]; prevTaskId: string | undefined; raw: { data: string; start: string | undefined; end: string } }
export interface GanttModel extends AccessibleModel {
  type: 'gantt'; dateFormat: string; axisFormat: string; tickInterval?: string; todayMarker: string;
  includes: string[]; excludes: string[]; inclusiveEndDates: boolean; topAxis: boolean; displayMode: string;
  weekday?: string; weekend: string; sections: string[]; links: Map<string, string>; tasks: GanttTask[];
  byId: Map<string, GanttTask>; currentSection: string; taskCount: number; lastOrder: number; lastTaskId?: string;
  options: {warn?: (message: string) => void}; now: number; work: number; includeSet: Set<string>; excludeSet: Set<string>; excludedDays: boolean[]; datedTokens: boolean;
  setDateFormat(text: string): void; enableInclusiveEndDates(): void; TopAxis(): void;
  setAxisFormat(text: string): void; setTickInterval(text: string): void; setTodayMarker(text: string): void;
  setIncludes(text: string): void; setExcludes(text: string): void; setWeekday(text: string): void; setWeekend(day: string): void;
  setDiagramTitle(text: string): void; setAccTitle(text: string): void; setAccDescription(text: string): void;
  addSection(text: string): void; addTask(description: string, data: string): void;
  setLink(ids: string, link: string): void; setClickEvent(ids: string, functionName: string, functionArguments: string | null): void;
  setClass(ids: string, className: string): void; getTasks(now?: number | Date): GanttTask[];
  isInvalidDate(date: Date): boolean; spend(units: number): void; parse(text: string, template: string): Date | undefined; today(): Date;
  warn(message: () => string): void; compileTasks(): boolean; compileTask(task: GanttTask): void;
  references(keyword: 'after' | 'until', list: string): GanttTask[]; getStartDate(raw: string): Date | undefined;
  checked(date: Date, raw: string): Date; getEndDate(start: Date, raw: string): Date | undefined; checkTaskDates(task: GanttTask): void;
}
export interface SequenceBox { name?: string; wrap: boolean; fill: string; actorKeys: string[] }
export interface SequenceActor { box?: SequenceBox; name: string; description: string; wrap: boolean; prevActor?: string; nextActor?: string; links: Map<string, unknown>; properties: Map<string, unknown>; details?: string; type: string }
export interface SequenceNumbering { start?: number; step?: number; visible: boolean }
export interface SequenceMessage { id: string; from?: string; to?: string; message: string | SequenceNumbering; wrap: boolean; type?: number; activate?: boolean; placement?: number; centralConnection?: number; answer?: unknown }
export interface SequenceNote { actor: string | string[]; placement: number; message: string; wrap: boolean }
export interface SequenceText { text: string; wrap?: boolean }
export interface SequenceBoxData { text?: string; color: string; wrap?: boolean }
export interface SequenceStatement {
  type: string; actor?: string | string[]; draw?: string; description?: SequenceText; config?: string;
  signalType?: number; from?: string; to?: string; msg?: SequenceText; activate?: boolean; centralConnection?: number;
  placement?: number; text?: SequenceText; boxData?: SequenceBoxData; boxText?: string; loopText?: SequenceText | string;
  optText?: SequenceText; altText?: SequenceText; parText?: SequenceText; criticalText?: SequenceText;
  optionText?: SequenceText; breakText?: SequenceText; color?: SequenceText; sequenceIndex?: number; sequenceIndexStep?: number; sequenceVisible?: boolean;
}
export type SequenceItem = SequenceStatement | string | SequenceItem[];
export interface SequenceModel extends AccessibleModel {
  type: 'sequence'; actors: Map<string, SequenceActor>; createdActors: Map<string, number>; destroyedActors: Map<string, number>;
  boxes: SequenceBox[]; messages: SequenceMessage[]; notes: SequenceNote[]; prevActor?: string; currentBox?: SequenceBox;
  lastCreated?: string; lastDestroyed?: string; wrapEnabled?: boolean; sequenceWrap: boolean; active: Map<string, number>;
  autoWrap(): boolean; setAccTitle(text: string): void; setAccDescription(text: string): void; setDiagramTitle(text: string): void;
  parseMessage(text: string): SequenceText; parseBoxData(text: string): SequenceBoxData; addBox(data: SequenceBoxData): void;
  addActor(id: string, name: string, description: (SequenceText & {type?: string}) | undefined, type: string | undefined, metadata?: string): void;
  addMessage(from: string, to: string, message: SequenceText, answer: unknown): void;
  addSignal(from?: string, to?: string, message?: SequenceText, type?: number, activate?: boolean, centralConnection?: number): boolean;
  addNote(actor: string | string[], placement: number, message: SequenceText): void;
  addLinks(actorId: string, text: {text: string}): void; addALink(actorId: string, text: {text: string}): void;
  addProperties(actorId: string, text: {text: string}): void; addDetails(actorId: string, text: {text: string}): void;
  apply(root: SequenceItem): void; applyOne(statement: SequenceStatement): void;
}
export interface GitCommit { id: string; message: string; seq: number; type: number; tags: string[]; parents: string[]; branch: string; customType?: number; customId?: boolean }
export interface GitModel extends AccessibleModel { type: 'gitGraph'; direction: string; commits: Map<string, GitCommit>; branches: Map<string, string | null>; lanes: string[]; currentBranch: string; head: GitCommit | null; warnings: string[] }
export interface ArchitectureNode { id: string; type: string; icon?: string; iconText?: string; title?: string; in?: string }
export interface ArchitectureGroup { id: string; icon?: string; title?: string; in?: string }
export interface ArchitectureEdge { lhsId: string; lhsDir: string; lhsInto: boolean; lhsGroup: boolean; rhsId: string; rhsDir: string; rhsInto: boolean; rhsGroup: boolean; title?: string }
export interface ArchitectureModel extends AccessibleModel {
 register(kind:'service'|'junction'|'group',id:string,parent:string|undefined):void;addService(service:Omit<ArchitectureNode,'type'>&{type?:'service'}):void;
 addJunction(junction:{id:string;in?:string;type?:'junction'}):void;addGroup(group:ArchitectureGroup):void;addEdge(edge:ArchitectureEdge):void;addLayoutHint(hint:{direction:'row'|'column';members:string[]}):void;
 type: 'architecture'; nodes: Map<string, ArchitectureNode>; groups: Map<string, ArchitectureGroup>; edges: ArchitectureEdge[]; layoutHints: { direction: string; members: string[] }[]; registered: Map<string, string> }
export interface SankeyNode { id: string; index: number }
export interface SankeyModel extends AccessibleModel { findOrCreateNode(id:string):SankeyNode;addLink(source:SankeyNode,target:SankeyNode,value:number):void; type: 'sankey'; lookup: Map<string,SankeyNode>; nodes: SankeyNode[]; links: { source: SankeyNode; target: SankeyNode; value: number }[] }
export interface TreemapNode { name: string; children?: TreemapNode[]; value?: number; classSelector?: string; cssCompiledStyles?: string[] }
export interface TreemapModel extends AccessibleModel { type: 'treemap'; roots: TreemapNode[]; classes: Map<string, string[]> }
export interface InfoModel extends AccessibleModel { type: 'info' }
export interface C4Named { key: string; value: string }
export type C4Attr = string | C4Named;
export interface C4Element {
  alias: string; label: string; aliasAttr?: C4Named; labelAttr?: C4Named; parent: C4Boundary | undefined;
  type?: string; descr?: string; techn?: string; sprite?: string; tags?: string; link?: string;
  bgColor?: string; fontColor?: string; borderColor?: string; shadowing?: string; shape?: string;
  legendText?: string; legendSprite?: string; extra?: Map<string, string>;
}
export interface C4Shape extends C4Element { kind: string }
export interface C4Boundary extends C4Element { nodeType?: string }
export interface C4Relation {
  type: string; from: string; to: string; label: string; fromAttr?: C4Named; toAttr?: C4Named; labelAttr?: C4Named;
  techn?: string; descr?: string; sprite?: string; tags?: string; link?: string; textColor?: string; lineColor?: string;
  offsetX?: number; offsetY?: number; extra?: Map<string, string>;
}
export interface C4Model extends AccessibleModel {
  type: 'c4'; c4Type: string | undefined; shapes: C4Shape[]; boundaries: C4Boundary[]; rels: C4Relation[];
  shapeInRow: number; boundaryInRow: number;
  shapeIndex: Map<string, C4Shape>; boundaryIndex: Map<string, C4Boundary>;
  relIndex: Map<string, Map<string, C4Relation>>; current: C4Boundary; stack: C4Boundary[];
  setC4Type(type: string): void; setTitle(title: string): void; setAccDescription(description: string): void;
  element<E extends C4Element>(list: E[], index: Map<string, E>, alias: C4Attr, title: C4Attr | undefined): E;
  addPersonOrSystem(kind: string, attrs: C4Attr[]): void; addContainer(kind: string, attrs: C4Attr[]): void;
  addComponent(kind: string, attrs: C4Attr[]): void; finishShape(shape: C4Shape, kind: string, attrs: C4Attr[], at: number): void;
  boundary(attrs: C4Attr[], fallback: string): C4Boundary; open(boundary: C4Boundary): void;
  addPersonOrSystemBoundary(attrs: C4Attr[], fallback?: string): void; addContainerBoundary(attrs: C4Attr[]): void;
  addDeploymentNode(nodeType: string, attrs: C4Attr[]): void; popBoundaryParseStack(): void;
  addRel(type: string, attrs: C4Attr[]): void; getC4Shape(alias: string): C4Shape | C4Boundary | undefined;
  updateElStyle(type: string, attrs: C4Attr[]): void; updateRelStyle(type: string, attrs: C4Attr[]): void;
  updateLayoutConfig(type: string, attrs: C4Attr[]): void;
}
export interface RequirementNode { name: string; type: string; requirementId: string; text: string; risk: string; verifyMethod: string; cssStyles: string[]; classes: string[] }
export interface RequirementElement { name: string; type: string; docRef: string; cssStyles: string[]; classes: string[] }
export interface RequirementRelation { type: string; src: string; dst: string }
export interface RequirementClass { id: string; styles: string[]; textStyles: string[] }
export interface RequirementModel extends AccessibleModel {
  type: 'requirement'; direction: string; requirements: Map<string, RequirementNode>; elements: Map<string, RequirementElement>;
  relations: RequirementRelation[]; classes: Map<string, RequirementClass>; newId: string; newText: string; newRisk: string;
  newVerifyMethod: string; newElementType: string; newDocRef: string; styleCopies: number;
  copy(target: string[], styles: string[]): void; setDirection(direction: string): void;
  addRequirement(name: string, type: string): RequirementNode; setNewReqId(id: string): void; setNewReqText(text: string): void;
  setNewReqRisk(risk: string): void; setNewReqVerifyMethod(method: string): void;
  addElement(name: string): RequirementElement; setNewElementType(type: string): void; setNewElementDocRef(reference: string): void;
  addRelationship(type: string, source: string, target: string): void; node(id: string): RequirementNode | RequirementElement | undefined;
  setCssStyle(ids: string[], styles: string[]): void; setClass(ids: string[], classes: string[]): void;
  defineClass(ids: string[], styles: string[]): void; setAccTitle(text: string): void; setAccDescription(text: string): void;
}
export interface BlockNode { id?: string; type?: string; label?: string; children?: BlockNode[]; columns?: number; widthInColumns?: number; width?: number; directions?: string[]; classes?: string[]; styles?: string[]; colorIndex?: number; start?: string; end?: string; arrowTypeStart?: string; arrowTypeEnd?: string; thickness?: string; pattern?: string; css?: string; styleClass?: string; stylesStr?: string }
export interface BlockClass { id: string; styles: string[]; textStyles: string[] }
export type BlockStatement = BlockNode | BlockNode[];
export interface BlockModel extends AccessibleModel {
  type: 'block'; root: BlockNode; blocks: BlockNode[]; edges: BlockNode[]; classes: Map<string, BlockClass>; byId: Map<string, BlockNode>;
  warn: ((message: string) => void) | undefined; pending: Set<string>; edgeCount: Map<string, number>; ids: number; colors: number; spaces: number;
  typeStr2Type(type: string | undefined): string; edgeStrToEdgeData(type: string): string; edgeStrToEdgeStartData(type: string): string;
  edgeStrToThickness(type: string): string; edgeStrToPattern(type: string): string; generateId(): string;
  addStyleClass(id: string, styles?: string): void; addStyle2Node(id: string, styles?: string): void; setCssClass(ids: string, className: string): void;
  named(id: string): BlockNode; setHierarchy(statements: BlockStatement[]): void;
}
export type RailroadNode = {type:'terminal';value:string} | {type:'nonterminal';name:string} | {type:'special';text:string} | {type:'sequence';elements:RailroadNode[]} | {type:'choice';alternatives:RailroadNode[]} | {type:'optional';element:RailroadNode} | {type:'repetition';element:RailroadNode;min:number;max:number};
export interface RailroadRule { name: string; definition: RailroadNode }
export interface RailroadModel extends AccessibleModel {
  type: 'railroad'; notation: 'railroad' | 'ebnf' | 'abnf' | 'peg'; rules: RailroadRule[]; byName: Map<string, RailroadRule>;
  setTitle(text: string): void; setAccTitle(text: string): void; setAccDescription(text: string): void;
  addRule(rule: RailroadRule): void; getRule(name: string): RailroadRule | undefined;
}
export type CynefinDomainName = 'complex' | 'complicated' | 'clear' | 'chaotic' | 'confusion';
export interface CynefinItem { label: string }
export interface CynefinDomain { name: CynefinDomainName; items: CynefinItem[] }
export interface CynefinTransition { from: CynefinDomainName; to: CynefinDomainName; label?: string }
export interface CynefinModel extends AccessibleModel {
  type: 'cynefin'; title: string | undefined; accTitle: string | undefined; accDescr: string | undefined;
  domains: Map<CynefinDomainName, CynefinDomain>; transitions: CynefinTransition[];
}
export interface TreeViewNode { id:number;level:number;name:string;nodeType:'file'|'directory';icon?:string;cssClass?:string;description?:string;children:TreeViewNode[] }
export interface TreeViewModel extends AccessibleModel { type:'treeView';root:TreeViewNode;count:number }
export interface AgentNode extends FlowNode { metadata?:{[key:string]:ConfigValue} }
export interface AgentLoc { first_line:number;first_column:number;last_line:number;last_column:number }
export interface AgentPosition { startLine:number;startColumn:number;endLine:number;endColumn:number;startIndex:number;endIndex:number }
export interface AgentMapping { id:string;type:string;position:AgentPosition }
export interface AgentDiagnostic { id:string;severity:'warning'|'error';message:string;nodeId?:string;edgeId?:string;position?:AgentPosition }
export interface AgentSemanticModel { direction?:string;vertices:{id:string;label?:string;shape?:string;vertexKind?:string;metadata?:Record<string,ConfigValue>}[];edges:{start:string;end:string;id?:string;label?:string;type?:string;stroke?:string;edgeSemantic?:string;length?:number;metadata?:Record<string,ConfigValue>}[];subGraphs:{id:string;type?:string;title?:string;nodes:string[];metadata?:Record<string,ConfigValue>;direction?:string}[];connectors:{id:string;title?:string;metadata?:Record<string,ConfigValue>}[];diagnostics:AgentDiagnostic[] }
export interface AgentflowModel extends AccessibleModel {
 addVertex(id:string,text?:FlowText,type?:string,styles?:string[]|null,classes?:string[]|null,direction?:string,props?:Record<string,string>,metadata?:string,location?:AgentLoc):void;
 addSingleLink(start:string,end:string,link:FlowLinkInfo|undefined,id?:string):void;addLink(starts:string[],ends:string[],link:FlowLinkInfo|undefined):void;
 updateLinkInterpolate(positions:(string|number)[],interpolation:string):void;updateLink(positions:(string|number)[],styles:string[]):void;edgeAt(index:number):FlowEdge;
 addClass(ids:string,styles:string[]):void;setDirection(direction:string):void;setClass(ids:string,className:string):void;setTooltip(ids:string,tooltip:string|undefined):void;
 setLink(ids:string,link:string,target?:string):void;setClickEvent(ids:string,functionName?:string,functionArgs?:string):void;setAccTitle(text:string):void;setAccDescription(text:string):void;
 addSubGraph(id:{text:string}|undefined,list:FlowDocItem[],title:{text:string;type?:string}|undefined,type?:'flow'):string;addGlobal(list:FlowDocItem[]):void;addConnector(text:FlowText|undefined,title?:FlowText):string;
 destructLink(end:string,start?:string):FlowLinkInfo;subgraph(id:string):AgentflowModel['subgraphs'][number]|undefined;vertexKind(vertex:AgentNode):string;semanticModel():AgentSemanticModel;
 metadata(id:string,text:string,location?:AgentLoc):ConfigValue;position(location?:AgentLoc):AgentPosition;
 shapeError(message:string,location?:AgentLoc):import('./core.js').PeleError;
 pushMapping(id:string,type:string,position:AgentPosition):AgentMapping;addVertexMapping(id:string,text:unknown,shape:unknown,location?:AgentLoc):void;extendVertexMapping(id:string,location?:AgentLoc):void;
 addEdgeMapping(from:unknown,to:unknown,link:unknown,location?:AgentLoc):void;blockMapping(id:unknown,type:string,start?:AgentLoc,end?:AgentLoc):void;
 addSubgraphMapping(id:unknown,title:unknown,start?:AgentLoc,end?:AgentLoc):void;addConnectorMapping(text:FlowText|undefined,title:unknown,start?:AgentLoc,end?:AgentLoc):void;
 emit(severity:'warning'|'error',id:string,message:string,context?:{nodeId?:string;edgeId?:string}):void;emitWarning(id:string,message:string,context?:{nodeId?:string;edgeId?:string}):void;emitError(id:string,message:string,context?:{nodeId?:string;edgeId?:string}):void;
 subgraphLookup:Map<string,AgentflowModel['subgraphs'][number]>;globalNodes:Set<string>;members:Set<string>;edgeById:Map<string,FlowEdge>;pairCount:Map<string,number>;subCount:number;
 bareMappings:Set<AgentMapping>;firstMapping:Map<string,AgentMapping>;reported:Map<string,AgentDiagnostic[]>;options:{maxEdges?:number;inheritDir?:boolean;direction?:string;lineOffset?:number;log?:(severity:'warning'|'error',message:string)=>void};
 type:'agentflow';direction?:string;nodes:Map<string,AgentNode>;connectors:Map<string,AgentNode>;edges:FlowEdge[];subgraphs:{id:string;type:string;nodes:string[];title:string;classes:string[];dir?:string;labelType:string;metadata?:{[key:string]:ConfigValue}}[];classes:Map<string,{id:string;styles:string[];textStyles:string[]}>;tooltips:Map<string,string>;mappings:AgentMapping[];diagnostics:AgentDiagnostic[];lineOffset:number;defaultEdgeStyle?:string[];defaultInterpolate?:ConfigValue }
export type WardleySourceStrategy = 'build' | 'buy' | 'outsource' | 'market';
export type WardleyFlow = 'forward' | 'backward' | 'bidirectional';
export interface WardleyNode {
  id: string; label: string; x?: number; y?: number; className?: string; labelOffsetX?: number; labelOffsetY?: number;
  inPipeline?: boolean; isPipelineParent?: boolean; inertia?: boolean; sourceStrategy?: WardleySourceStrategy;
}
export interface WardleyPoint { x: number; y: number }
export interface WardleyLink { source: string; target: string; dashed?: boolean; label?: string; flow?: WardleyFlow }
export interface WardleyTrend { nodeId: string; targetX: number; targetY: number }
export interface WardleyPipeline { nodeId: string; componentIds: string[] }
export interface WardleyAnnotation { number: number; coordinates: WardleyPoint[]; text?: string }
export interface WardleyNote extends WardleyPoint { text: string }
export interface WardleyForce extends WardleyPoint { name: string }
export interface WardleyAxes { xLabel?: string; yLabel?: string; stages?: string[]; stageBoundaries?: number[] }
export interface WardleyModel extends AccessibleModel {
  type: 'wardley'; title: string | undefined; accTitle: string | undefined; accDescr: string | undefined;
  nodes: WardleyNode[]; links: WardleyLink[]; trends: WardleyTrend[]; pipelines: WardleyPipeline[];
  annotations: WardleyAnnotation[]; notes: WardleyNote[]; accelerators: WardleyForce[]; deaccelerators: WardleyForce[];
  annotationsBox?: WardleyPoint; axes: WardleyAxes; size?: {width: number; height: number};
}
export interface EventLane { index:number;label:string;group:'trigger'|'state'|'stream';namespace?:string }
export interface EventBox { id:string;kind:'ui'|'processor'|'command'|'readmodel'|'event';entity:string;name:string;namespace?:string;reset:boolean;lane:EventLane;data?:string;dataType?:string }
export interface EventModelingModel extends AccessibleModel { type:'eventmodeling';boxes:EventBox[];lanes:EventLane[];relations:{source:EventBox;target:EventBox}[];notes:{frame:string;text:string;dataType?:string}[];specifications:{frame:string;given:string[];when:string[];then:string[]}[];entities:string[] }
export type SourceSpan = [number,number];
export type UsecaseLabelType = 'text' | 'markdown';
export type UsecaseDirection = 'TB' | 'TD' | 'BT' | 'RL' | 'LR';
export interface UsecaseActor {
  id: string; label: string; labelType: UsecaseLabelType; type: 'normal' | 'hollow' | 'awesome' | 'icon';
  icon?: string; business: boolean; stereotype?: string; parentId?: string; classes: string[]; styles: string[];
}
export interface UseCase {
  id: string; label: string; labelType: UsecaseLabelType; shape: 'ellipse' | 'rect'; business: boolean;
  stereotype?: string; parentId?: string; classes: string[]; styles: string[];
}
export interface UsecaseBoundary {
  id: string; label: string; labelType: UsecaseLabelType; type: 'rect' | 'package';
  members: string[]; classes: string[]; styles: string[];
}
export interface UsecaseRelationship {
  id: string; explicitId: boolean; source: string; target: string;
  type: 'association' | 'include' | 'extend' | 'generalization'; arrowType: number;
  label?: string; labelType?: UsecaseLabelType; minlen: number; classes: string[]; styles: string[];
  animate: boolean; animation?: 'fast' | 'slow';
}
export interface UsecaseNote { id: string; target: string; label: string; labelType: UsecaseLabelType }
export interface UsecaseJsonNode {
  id: string; value: Record<string, unknown>; propertyOrder: Map<string, string[]>; classes: string[]; styles: string[];
}
export interface UsecaseClassDef { id: string; styles: string[] }
export interface UsecaseMetadataOccurrence { key: string; span: SourceSpan; keySpan: SourceSpan; valueSpan: SourceSpan }
export interface UsecaseNodeOccurrence {
  id: string; span: SourceSpan; idSpan: SourceSpan; labelSpan?: SourceSpan; defines?: boolean;
  stereotypeSpan?: SourceSpan; metadata?: UsecaseMetadataOccurrence[]; classSpans?: SourceSpan[];
}
export interface UsecaseEdgeOccurrence { id: string; span: SourceSpan; labelSpan?: SourceSpan; idSpan?: SourceSpan; metadata?: UsecaseMetadataOccurrence[] }
export interface UsecaseGraphStatement {
  kind: 'node' | 'edge' | 'group' | 'note' | 'json' | 'metadata' | 'edgeMetadata' | 'classDef' | 'classAssign' | 'style' | 'direction' | 'accTitle' | 'accDescr' | 'comment' | 'blank';
  span: SourceSpan; nodes?: UsecaseNodeOccurrence[]; edges?: UsecaseEdgeOccurrence[]; group?: string;
  idSpan?: SourceSpan; titleSpan?: SourceSpan; endSpan?: SourceSpan; ref?: string; refSpan?: SourceSpan;
  children?: UsecaseGraphStatement[]; metadata?: UsecaseMetadataOccurrence[]; classSpans?: SourceSpan[];
}
export interface UsecaseModel {
  type: 'usecase'; actors: Map<string, UsecaseActor>; useCases: Map<string, UseCase>; systemBoundaries: Map<string, UsecaseBoundary>;
  relationships: UsecaseRelationship[]; notes: Map<string, UsecaseNote>; jsonNodes: Map<string, UsecaseJsonNode>;
  classDefs: Map<string, UsecaseClassDef>; direction: UsecaseDirection; title?: string;
  accTitle: string; accDescr: string; accDescription: string; source: string; headerSpan: SourceSpan;
  statements: UsecaseGraphStatement[];
}
export interface VennSubset { sets: string[]; size: number; label: string | undefined }
export interface VennText { sets: string[]; id: string; label: string | undefined }
export interface VennStyle { targets: string[]; styles: Map<string, string> }
export interface VennModel {
  type: 'venn'; title: string | undefined; subsets: VennSubset[]; textNodes: VennText[]; styles: VennStyle[];
  known: Set<string>; current: string[] | undefined; indent: boolean;
  setDiagramTitle(title: string): void; addSubsetData(ids: string[], label: string | undefined, size: number | undefined): void;
  addTextData(ids: string[], id: string, label: string | undefined): void; addStyleData(ids: string[], data: [string, string][]): void;
  validateUnionIdentifiers(ids: string[]): void; getCurrentSets(): string[] | undefined;
  getIndentMode(): boolean; setIndentMode(enabled: boolean): void;
}
export interface IshikawaNode { text:string;children:IshikawaNode[] }
export interface IshikawaModel { addNode(rawLevel:number,text:string):void;type:'ishikawa';title?:string;root?:IshikawaNode;stack:{level:number;node:IshikawaNode}[];baseLevel?:number }
export type DiagramModel = VennModel | IshikawaModel | PieModel | PacketModel | RadarModel | QuadrantModel | TimelineModel | JourneyModel | FlowchartModel | KanbanModel | MindmapModel | ClassModel | ErModel | XyChartModel | StateModel | GanttModel | SequenceModel | GitModel | ArchitectureModel | SankeyModel | TreemapModel | C4Model | InfoModel | RequirementModel | BlockModel | RailroadModel | CynefinModel | TreeViewModel | AgentflowModel | WardleyModel | EventModelingModel | UsecaseModel;
export interface Rendered { type: DiagramId; svg: string; width: number; height: number; links: { id: string; href: string; internal: boolean }[] }
export interface ZoomOptions {
  always?: boolean; maxScale?: number; controls?: boolean;
  labels?: {zoomIn?: string; zoomOut?: string; reset?: string};
}
export interface Zoom {
  readonly scale: number; readonly fit: number;
  zoomBy(factor: number): void; zoomTo(scale: number): void; reset(): void; refresh(): void; destroy(): void;
}
export interface MountOptions extends RenderOptions {
  onRender?: (result: Rendered) => void;
  zoom?: boolean | 'auto' | ZoomOptions;
  sanitize?: (svg: string) => Node;
  onError?: (error: unknown) => void;
}
export interface Mounted {
  readonly result: Rendered | undefined; readonly source: string; readonly error: unknown; readonly zoom: Zoom | undefined;
  update(text: string, options?: MountOptions): void; refresh(): void; destroy(): void;
}
export function enableZoom(element: HTMLElement, options?: ZoomOptions): Zoom;
export interface Renderer {
  register(...providers: Diagram<any>[]): void;
  registered(type: DiagramId): boolean;
  registerBuiltin(type: import('./core.js').DiagramType): void;
  detectType(source: string): import('./core.js').DiagramType | null;
  supports(source: string): boolean;
  close(): void;
  mount(element: HTMLElement, text: string, options?: MountOptions): Mounted;
  capabilities(): Capability[];
  parse<Model = DiagramModel>(source: string, options?: ParseOptions): Model;
  render(input: string | object, options?: RenderOptions): Rendered;
  query(model: DiagramModel, operation: string, argumentsValue?: ConfigValue): ConfigValue;
  dispose(model: object): void;
}
export function loadRenderer(moduleOrPath?: string | URL | Response | BufferSource | WebAssembly.Module): Promise<Renderer>;

export interface TextMeasurer { width(text: string, size: number, style: number): number }
export const metricsMeasurer: TextMeasurer;
export function canvasMeasurer(fontFamily?: string, monoFamily?: string): TextMeasurer | null;
export function forgetTextWidths(): void;

/** Upstream public names; preserve the existing binding result and resource shapes. */
export type IconResolver = NonNullable<RenderOptions['icons']>;
export type LinkInfo = Rendered['links'][number];
export type RenderResult = Rendered;
export type {DiagramType} from './core.js';
export type SwimlaneModel = FlowchartModel;
