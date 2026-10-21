export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='stateDiagram-v2\ndirection LR\nstate Group {\nstate "Alpha" as A\nA: Detail one\nA: Detail two\nstate "Beta" as B\nA --> B: transition\n}\n[*] --> Group';
export const MODES=['baseline','node-label','description-replace','description-append','styles-replace','styles-mutate','late-fields','graph-replace','edges-replace','edge-late','links','direction','title-accessibility','helper','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height),links:r.links??[]});const copy=v=>JSON.parse(JSON.stringify(v,(_k,v)=>v instanceof Map?{$map:[...v]}:v));
export function runState(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title="Initial";
 const graph=m.graph,nodes=graph.nodes,group=nodes.find(n=>n.id==='Group'),alpha=nodes.find(n=>n.id==='A'),beta=nodes.find(n=>n.id==='B'),edges=graph.edges,edge=edges[0],description=alpha.description,styles=beta.cssStyles;group.dir='LR';styles.push('fill:red');
 const trace=[],sentinel={mode};let fired=false,nested;const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const trigger={'description-replace':'Alpha','description-append':'Detail one','late-fields':'transition','edge-late':'transition'}[mode];
 const options={...(responsive?{maxWidth:160}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(!trigger||t===trigger)){fired=true;switch(mode){
 case'node-label':alpha.label='Changed Alpha';beta.label='Changed Beta';break;
 case'description-replace':alpha.description=['Replacement one','Replacement two'];break;
 case'description-append':description.push('Appended detail');alpha.description=['Replaced after capture'];break;
 case'styles-replace':beta.cssStyles=['fill:green'];break;
 case'styles-mutate':styles[0]='fill:green';break;
 case'late-fields':beta.cssClasses='late';beta.id='Changed B';break;
 case'graph-replace':m.graph={nodes:[],edges:[]};break;
 case'edges-replace':graph.edges=[{id:'new-edge',start:'B',end:'A',label:'Replacement edge',classes:'transition'}];break;
 case'edge-late':edge.id='Changed edge';edge.pattern='dashed';edge.label='After captured label';edges.push({id:'new-edge',start:'B',end:'A',label:'Appended edge',classes:'transition'});break;
 case'links':m.links.set('A',{url:'https://example.com/alpha',tooltip:'Alpha link'});break;
 case'direction':m.setDirection('TB');group.dir='TB';break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'helper':m.addState('Added');m.addDescription('Added','Saved helper');break;
 case'throw':m.title='Before throw';alpha.label='Saved Alpha';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(m),originalGraph=copy(graph),originalNodes=copy(nodes),originalAlpha=copy(alpha),originalBeta=copy(beta),originalEdges=copy(edges),originalEdge=copy(edge),originalDescription=copy(description),originalStyles=copy(styles),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:160}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first,trace,after,originalGraph,originalNodes,originalAlpha,originalBeta,originalEdges,originalEdge,originalDescription,originalStyles,graphCurrent:graph===m.graph,edgesCurrent:edges===graph.edges,descriptionCurrent:description===alpha.description,stylesCurrent:styles===beta.cssStyles,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
