// Compile against the real package export map; no source-path aliases.
import type {IconResolver, LinkInfo, RenderResult} from '@unlocalhosted/pele-rs';
import type {IconResolver as FullIcons, RenderResult as FullResult} from '@unlocalhosted/pele-rs/full';
import type {LinkInfo as LazyLink, RenderResult as LazyResult} from '@unlocalhosted/pele-rs/lazy';
import type {IconResolver as RegistryIcons, LinkInfo as RegistryLink, RenderResult as RegistryResult} from '@unlocalhosted/pele-rs/registry';
import {loadRenderer} from '@unlocalhosted/pele-rs';
const icons: IconResolver = name => name === 'ok' ? '<svg/>' : null;
const fullIcons: FullIcons = icons;
const registryIcons: RegistryIcons = fullIcons;
const link: LinkInfo = {id:'A',href:'https://example.test',internal:false};
const lazyLink: LazyLink = link;
const registryLink: RegistryLink = lazyLink;
const output: RenderResult = {type:'pie',svg:'<svg/>',width:10,height:20,links:[registryLink]};
const full: FullResult = output;
const lazy: LazyResult = full;
const registry: RegistryResult = lazy;
async function consumer() {
  const renderer = await loadRenderer();
  const result: RenderResult = renderer.render('pie\n"A":1', {icons:registryIcons});
  renderer.close();
  return result;
}
void [registry, consumer];

// Complete pinned public type catalog, including all 33 diagram model names.
import type * as RootTypes from '@unlocalhosted/pele-rs';
type RootPublicTypes = [RootTypes.DiagramType, RootTypes.PeleErrorCode, RootTypes.TextMeasurer, RootTypes.IconResolver, RootTypes.LinkInfo, RootTypes.RenderOptions, RootTypes.RenderResult, RootTypes.Diagram<unknown>, RootTypes.MountOptions, RootTypes.Mounted, RootTypes.Zoom, RootTypes.ZoomOptions, RootTypes.ParseOptions];
type RootModels = [RootTypes.AgentflowModel, RootTypes.ArchitectureModel, RootTypes.BlockModel, RootTypes.C4Model, RootTypes.ClassModel, RootTypes.CynefinModel, RootTypes.ErModel, RootTypes.EventModelingModel, RootTypes.FlowchartModel, RootTypes.GanttModel, RootTypes.GitModel, RootTypes.InfoModel, RootTypes.IshikawaModel, RootTypes.JourneyModel, RootTypes.KanbanModel, RootTypes.MindmapModel, RootTypes.PacketModel, RootTypes.PieModel, RootTypes.QuadrantModel, RootTypes.RadarModel, RootTypes.RailroadModel, RootTypes.RequirementModel, RootTypes.SankeyModel, RootTypes.SequenceModel, RootTypes.StateModel, RootTypes.SwimlaneModel, RootTypes.TimelineModel, RootTypes.TreeViewModel, RootTypes.TreemapModel, RootTypes.UsecaseModel, RootTypes.VennModel, RootTypes.WardleyModel, RootTypes.XyChartModel];
import type * as FullTypes from '@unlocalhosted/pele-rs/full';
type FullPublicTypes = [FullTypes.DiagramType, FullTypes.PeleErrorCode, FullTypes.TextMeasurer, FullTypes.IconResolver, FullTypes.LinkInfo, FullTypes.RenderOptions, FullTypes.RenderResult, FullTypes.Diagram<unknown>, FullTypes.MountOptions, FullTypes.Mounted, FullTypes.Zoom, FullTypes.ZoomOptions, FullTypes.ParseOptions];
type FullModels = [FullTypes.AgentflowModel, FullTypes.ArchitectureModel, FullTypes.BlockModel, FullTypes.C4Model, FullTypes.ClassModel, FullTypes.CynefinModel, FullTypes.ErModel, FullTypes.EventModelingModel, FullTypes.FlowchartModel, FullTypes.GanttModel, FullTypes.GitModel, FullTypes.InfoModel, FullTypes.IshikawaModel, FullTypes.JourneyModel, FullTypes.KanbanModel, FullTypes.MindmapModel, FullTypes.PacketModel, FullTypes.PieModel, FullTypes.QuadrantModel, FullTypes.RadarModel, FullTypes.RailroadModel, FullTypes.RequirementModel, FullTypes.SankeyModel, FullTypes.SequenceModel, FullTypes.StateModel, FullTypes.SwimlaneModel, FullTypes.TimelineModel, FullTypes.TreeViewModel, FullTypes.TreemapModel, FullTypes.UsecaseModel, FullTypes.VennModel, FullTypes.WardleyModel, FullTypes.XyChartModel];
import type * as LazyTypes from '@unlocalhosted/pele-rs/lazy';
type LazyPublicTypes = [LazyTypes.DiagramType, LazyTypes.PeleErrorCode, LazyTypes.TextMeasurer, LazyTypes.IconResolver, LazyTypes.LinkInfo, LazyTypes.RenderOptions, LazyTypes.RenderResult, LazyTypes.Diagram<unknown>, LazyTypes.MountOptions, LazyTypes.Mounted, LazyTypes.Zoom, LazyTypes.ZoomOptions, LazyTypes.ParseOptions];
type LazyModels = [LazyTypes.AgentflowModel, LazyTypes.ArchitectureModel, LazyTypes.BlockModel, LazyTypes.C4Model, LazyTypes.ClassModel, LazyTypes.CynefinModel, LazyTypes.ErModel, LazyTypes.EventModelingModel, LazyTypes.FlowchartModel, LazyTypes.GanttModel, LazyTypes.GitModel, LazyTypes.InfoModel, LazyTypes.IshikawaModel, LazyTypes.JourneyModel, LazyTypes.KanbanModel, LazyTypes.MindmapModel, LazyTypes.PacketModel, LazyTypes.PieModel, LazyTypes.QuadrantModel, LazyTypes.RadarModel, LazyTypes.RailroadModel, LazyTypes.RequirementModel, LazyTypes.SankeyModel, LazyTypes.SequenceModel, LazyTypes.StateModel, LazyTypes.SwimlaneModel, LazyTypes.TimelineModel, LazyTypes.TreeViewModel, LazyTypes.TreemapModel, LazyTypes.UsecaseModel, LazyTypes.VennModel, LazyTypes.WardleyModel, LazyTypes.XyChartModel];
import type * as CoreTypes from '@unlocalhosted/pele-rs/core';
type CorePublicTypes = [CoreTypes.DiagramType, CoreTypes.PeleErrorCode, CoreTypes.TextMeasurer, CoreTypes.IconResolver, CoreTypes.LinkInfo, CoreTypes.RenderOptions, CoreTypes.RenderResult, CoreTypes.Diagram<unknown>, CoreTypes.MountOptions, CoreTypes.Mounted, CoreTypes.Zoom, CoreTypes.ZoomOptions, CoreTypes.ParseOptions];
import type * as RegistryTypes from '@unlocalhosted/pele-rs/registry';
type RegistryPublicTypes = [RegistryTypes.DiagramType, RegistryTypes.PeleErrorCode, RegistryTypes.TextMeasurer, RegistryTypes.IconResolver, RegistryTypes.LinkInfo, RegistryTypes.RenderOptions, RegistryTypes.RenderResult, RegistryTypes.Diagram<unknown>, RegistryTypes.MountOptions, RegistryTypes.Mounted, RegistryTypes.Zoom, RegistryTypes.ZoomOptions, RegistryTypes.ParseOptions];
type RegistryModels = [RegistryTypes.AgentflowModel, RegistryTypes.ArchitectureModel, RegistryTypes.BlockModel, RegistryTypes.C4Model, RegistryTypes.ClassModel, RegistryTypes.CynefinModel, RegistryTypes.ErModel, RegistryTypes.EventModelingModel, RegistryTypes.FlowchartModel, RegistryTypes.GanttModel, RegistryTypes.GitModel, RegistryTypes.InfoModel, RegistryTypes.IshikawaModel, RegistryTypes.JourneyModel, RegistryTypes.KanbanModel, RegistryTypes.MindmapModel, RegistryTypes.PacketModel, RegistryTypes.PieModel, RegistryTypes.QuadrantModel, RegistryTypes.RadarModel, RegistryTypes.RailroadModel, RegistryTypes.RequirementModel, RegistryTypes.SankeyModel, RegistryTypes.SequenceModel, RegistryTypes.StateModel, RegistryTypes.SwimlaneModel, RegistryTypes.TimelineModel, RegistryTypes.TreeViewModel, RegistryTypes.TreemapModel, RegistryTypes.UsecaseModel, RegistryTypes.VennModel, RegistryTypes.WardleyModel, RegistryTypes.XyChartModel];
function swimlane(model: RootTypes.SwimlaneModel): FullTypes.FlowchartModel { const type: 'flowchart' = model.type; void type; return model; }
void swimlane;
