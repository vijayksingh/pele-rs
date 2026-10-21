export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='wardley-beta\ntitle Initial\nanchor User [0.9,0.1]\ncomponent Service [0.7,0.3] (build)\ncomponent Data [0.4,0.6] (buy) inertia\npipeline Data {\ncomponent SQL [0.3]\ncomponent Cloud [0.8]\n}\nUser -> Service\nService +> Data\nevolve Service 0.8\nannotation 1,[0.6,0.4] "First annotation"\nnote "Build carefully" [0.8,0.6]\naccelerator Cloud [0.3,0.8]\ndeaccelerator Legacy [0.4,0.2]';
export const MODES=['baseline','stage-edit','stages-replace','axis-edit','nodes-replace','node-late','link-late','annotation-late','note-late','forces','title-accessibility','throw','nested','pipeline-edit','append'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const copy=v=>JSON.parse(JSON.stringify(v));
export function runWardley(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);
 m.axes.stages=['Genesis','Custom','Product','Commodity'];m.links[0].label='Edge caption';
 const nodes=m.nodes,first=nodes[0],links=m.links,link=links[0],annotations=m.annotations,annotation=annotations[0],notes=m.notes,note=notes[0],stages=m.axes.stages;
 const trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const trigger={'node-late':'User','link-late':'Edge caption','annotation-late':'1','note-late':'Build carefully'}[mode];
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(!trigger||t===trigger)){fired=true;switch(mode){
 case'stage-edit':stages[1]='Changed stage';stages.push('Extra');break;
 case'stages-replace':m.axes.stages=['Replacement'];break;
 case'axis-edit':m.axes.xLabel='Changed evolution';m.axes.yLabel='Changed visibility';m.size={width:700,height:400};break;
 case'nodes-replace':m.nodes=[{id:'Replacement',label:'Replacement node',x:50,y:50}];break;
 case'node-late':first.id='ChangedUser';first.label='After captured label';first.x=80;first.labelOffsetX=20;first.inPipeline=true;break;
 case'link-late':link.label='After captured edge';link.dashed=true;link.source='ChangedSource';break;
 case'annotation-late':annotation.number=3;annotation.text='Changed annotation';annotation.coordinates=[{x:20,y:30}];break;
 case'note-late':note.text='After captured note';note.x=15;note.y=20;break;
 case'forces':m.accelerators[0].name='Changed accelerator';m.deaccelerators=[{name:'Replacement force',x:10,y:10}];break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'throw':m.title='Before throw';first.label='Saved user';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
 case'pipeline-edit':m.pipelines[0].componentIds.reverse();nodes[3].x=5;break;
 case'append':nodes.push({id:'Extra',label:'Appended node',x:60,y:20});notes.push({text:'Appended note',x:20,y:80});break;
 }}return t.length*s*.6;}}};
 let firstResult;try{firstResult={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){firstResult={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(m),originalNodes=copy(nodes),originalFirst=copy(first),originalLinks=copy(links),originalLink=copy(link),originalAnnotations=copy(annotations),originalAnnotation=copy(annotation),originalNotes=copy(notes),originalNote=copy(note),originalStages=copy(stages),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first:firstResult,trace,after,originalNodes,originalFirst,originalLinks,originalLink,originalAnnotations,originalAnnotation,originalNotes,originalNote,originalStages,nodesCurrent:nodes===m.nodes,firstCurrent:first===m.nodes[0],stagesCurrent:stages===m.axes.stages,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
