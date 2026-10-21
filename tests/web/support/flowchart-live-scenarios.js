export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='flowchart LR\nA[Alpha] -->|first edge| B[Beta]\nB --> C[Gamma]\nstyle B fill:#113355\nclassDef custom color:#224466\nclass C custom';
export const MODES=['baseline','late-title','late-accessibility','copied-label','copied-edge','style-mutate','style-replace','compiled-class','direction','replace-maps','append-edge','default-style','throw-object','throw-undefined','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const state=m=>JSON.parse(JSON.stringify({title:m.title,accTitle:m.accTitle,accDescr:m.accDescr,direction:m.direction,nodes:[...m.nodes],edges:m.edges,subgraphs:m.subgraphs,classes:[...m.classes],tooltips:[...m.tooltips],defaultEdgeStyle:m.defaultEdgeStyle}));
export function runFlowchart(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0,{maxEdges:500}):core.parse(source);m.title='Initial';
 const nodes=m.nodes,later=nodes.get('B'),styles=later.styles,edges=m.edges,classStyles=m.classes.get('custom').styles;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}},trace=[],sentinel={mode};let fired=false,nested;
 const execute=options=>raw?entry.render(m,{},options):core.render(m,options);
 const options={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&t==='Alpha'){fired=true;switch(mode){
 case'late-title':m.title='Changed title';break;
 case'late-accessibility':m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'copied-label':later.text='Changed later';break;
 case'copied-edge':edges[0].text='Changed edge';edges[0].end='C';break;
 case'style-mutate':styles.push('fill:#ff0000','font-size:22px');break;
 case'style-replace':later.styles=['fill:#00ff00','font-size:24px'];break;
 case'compiled-class':classStyles.push('fill:#ff00ff','font-size:25px');break;
 case'direction':m.direction='TB';break;
 case'replace-maps':m.nodes=new Map([['B',later]]);break;
 case'append-edge':edges.push({...edges[0],start:'C',end:'A',id:'new-edge'});break;
 case'default-style':m.defaultEdgeStyle=['stroke:#ff0000'];break;
 case'throw-object':m.title='Saved';later.text='Saved later';throw sentinel;
 case'throw-undefined':m.title='Saved';later.text='Saved later';throw undefined;
 case'nested':m.title='Nested';nested=drawing(execute(pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(execute(options))};}catch(e){first={ok:false,original:mode==='throw-undefined'?e===undefined:e===sentinel,error:e===sentinel?'sentinel':e===undefined?'undefined':{name:e.name,message:e.message}};}
 const after=state(m),recoveryTrace=[];const recovery=drawing(execute({...pure,...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}));
 const record={raw,responsive,mode,fired,first,trace,after,nodesCurrent:nodes===m.nodes,laterCurrent:later===m.nodes.get('B'),stylesCurrent:styles===later.styles,edgesCurrent:edges===m.edges,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return record;
}
