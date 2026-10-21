export type {Diagram, ProviderRendered} from './full.js';
export {loadRegistry, type Registry} from './registry.js';
export type DiagramType = 'flowchart' | 'sequence' | 'class' | 'state' | 'er' | 'pie' | 'gantt' | 'gitGraph' | 'mindmap' | 'timeline' | 'journey' | 'xychart' | 'quadrantChart' | 'requirement' | 'c4' | 'sankey' | 'block' | 'packet' | 'kanban' | 'architecture' | 'radar' | 'treemap' | 'info' | 'agentflow' | 'usecase' | 'railroad' | 'treeView' | 'venn' | 'cynefin' | 'wardley' | 'swimlane' | 'ishikawa' | 'eventmodeling';
export type DiagramId = DiagramType | (string & {});
export type ConfigValue = null | boolean | number | string | ConfigValue[] | { [key: string]: ConfigValue };
export interface PrepareOptions { limit?: number; config?: { [key: string]: ConfigValue } }
export interface PreparedSource { text: string; title?: string; config: { [key: string]: ConfigValue }; withComments: string; lineOffset: number }
export interface Inspection { type: DiagramType | null; registered: boolean; prepared: PreparedSource; encodedText: string }
export interface Capability { type: DiagramId; known: boolean; registered: boolean; syntaxParity: boolean; semanticParity: boolean; renderParity: boolean; hostParity: boolean }
export type PeleErrorCode = 'unsupported-diagram' | 'syntax' | 'semantic' | 'limit';
export class PeleError extends Error {
 constructor(message:string,code?:PeleErrorCode,info?:{type?:string|null;line?:number;column?:number;snippet?:string});
 code:PeleErrorCode;type:string|null;line:number;column:number;snippet:string;
}
export {PeleError as PeleCompatibilityError};
export interface Core {
  inspect(source: string, options?: PrepareOptions): Inspection;
  detectType(source: string): DiagramType | null;
  supports(source: string): boolean;
  capabilities(): Capability[];
}
export function loadCore(moduleOrPath?: string | URL | Response | BufferSource | WebAssembly.Module): Promise<Core>;

export type {
  TextMeasurer, IconResolver, LinkInfo, RenderOptions, RenderResult, MountOptions, Mounted, Zoom, ZoomOptions, ParseOptions
} from './full.js';
