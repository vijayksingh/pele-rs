export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const MODES=['baseline','label','captured-groups','append-member','false-parent','collapsed-return','throw-object','throw-undefined','nested','redraw-replace'];
export const source=family=>`${family==='flowchart'?'flowchart':'swimlane-beta'} LR\nsubgraph Outer[Outer group]\nsubgraph Inner[Inner group]\nA[Alpha] --> B[Beta]\nend\nC[Gamma]\nend\nB --> C`;
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height),links:r.links??[]});
const state=m=>JSON.parse(JSON.stringify({title:m.title,accTitle:m.accTitle,accDescr:m.accDescr,direction:m.direction,nodes:[...m.nodes],edges:m.edges,subgraphs:m.subgraphs,lookup:[...m.subgraphLookup]}));
export function runSubgraph(core,entry,dispose,family,raw,responsive,mode){
 const m=raw?entry.parse(source(family),{},undefined,0,{maxEdges:500}):core.parse(source(family));
 const original=m.subgraph,groups=m.subgraphs,later=m.nodes.get('B'),inner=m.subgraph('Inner'),outer=m.subgraph('Outer');
 // Trigger both containment and collapsed-chain calls in every case.
 inner.metadata={view:'collapsed'};
 let fired=false,redrawn=false,nested;const trace=[],sentinel={family,mode};
 const pure={measurer:{width:(t,s)=>t.length*s*.6}},execute=options=>raw?entry.render(m,{},options):core.render(m,options);
 function replacement(id){trace.push(['replacement',id,this===m]);return original.call(this,id);}
 function wrapped(id){
  trace.push(['subgraph',id,this===m]);
  if(!fired){fired=true;switch(mode){
   case'label':later.text='Changed Beta';m.title='Changed title';break;
   case'captured-groups':m.subgraphs=[];outer.title='Captured outer';break;
   case'append-member':outer.nodes.push('Late');break;
   case'false-parent':break;
   case'collapsed-return':break;
   case'throw-object':m.title='Saved';throw sentinel;
   case'throw-undefined':m.title='Saved';throw undefined;
   case'nested':nested=drawing(execute(pure));m.title='After nested';break;
  }}
  if(mode==='false-parent')return undefined;
  if(mode==='collapsed-return')return id==='Outer'?{metadata:{view:'collapsed'}}:original.call(this,id);
  return original.call(this,id);
 }
 m.subgraph=wrapped;
 const options={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){trace.push(['width',t,bits(s),style??0]);if(mode==='redraw-replace'&&!redrawn){redrawn=true;m.subgraph=replacement;later.text='After width';}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(execute(options))};}catch(e){first={ok:false,original:mode==='throw-undefined'?e===undefined:e===sentinel,undefined:e===undefined,...(e!==sentinel&&e!==undefined?{error:{name:e.name,message:e.message}}:{})};}
 const firstTrace=trace.splice(0),after=state(m);m.subgraph=original;
 const recovery=drawing(execute({...pure,...(responsive?{maxWidth:200}:{})}));
 const out={family,raw,responsive,mode,fired,redrawn,first,trace:firstTrace,after,groupsCurrent:groups===m.subgraphs,laterCurrent:later===m.nodes.get('B'),...(nested?{nested}:{}),recovery,afterRecovery:state(m)};
 dispose(m);return out;
}
