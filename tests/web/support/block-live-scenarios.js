export const SOURCE='block\ncolumns 2\nblock:g["Group"]:2\ncolumns 2\na["Alpha"] b(("Beta"))\nend\nc<["Arrow"]>(right) space:2 d{{"Delta"}}\na -- "Flow" --> d\nclassDef hot fill:#f96,color:blue\nclass a hot';
export const MODES=['baseline','title-accessibility','children-replace','children-append','current-node','later-node','grid','arrow','edge-label','edges-replace','style','throw-object','throw-undefined','nested'];
export const cases=()=>[false,true].flatMap(raw=>[false,true].flatMap(responsive=>MODES.map(mode=>({raw,responsive,mode}))));
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)}),copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>copy({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,root:m.root,edges:m.edges,classes:[...m.classes]});
export function runBlock(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);model.title='Initial title';
 const root=model.root,rootChildren=root.children,group=rootChildren.find(n=>n.id==='g'),children=group.children,a=children[0],b=children[1],arrow=rootChildren.find(n=>n.id==='c'),edges=model.edges,edge=edges[0],sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const mutate=()=>{switch(mode){
  case'title-accessibility':model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';break;
  case'children-replace':group.children=[{id:'replacement',type:'square',label:'Replacement'}];break;
  case'children-append':children.push({id:'appended',type:'square',label:'Appended'});break;
  case'current-node':a.label='Changed current';a.id='renamed-a';a.type='diamond';break;
  case'later-node':b.label='Changed later';break;
  case'grid':group.columns=1;group.widthInColumns=1;break;
  case'arrow':arrow.directions=['down'];break;
  case'edge-label':edge.label='Changed edge';edge.id='Changed edge ID';edge.arrowTypeEnd='circle';break;
  case'edges-replace':model.edges=[];break;
  case'style':model.classes.get('hot').styles=['fill:#123456','color:#654321'];b.classes=['hot'];break;
  case'throw-object':case'throw-undefined':model.title='Before failure';a.label='Saved Alpha';throw mode.endsWith('undefined')?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(draw({measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 const target=mode==='title-accessibility'?'Initial title':mode==='arrow'?'Arrow':mode==='edge-label'?'Flow':['baseline','children-replace','grid','nested'].includes(mode)?'Group':'Alpha';
 const resources=(record,active)=>({measurer:{width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active&&!fired&&t===target){fired=true;mutate();}return t.length*s*.6;}}});
 try{
  let first;try{first={ok:true,...output(draw({...settings,...resources(trace,true)}))};}catch(e){first={ok:false,original:e===(mode.endsWith('undefined')?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after=state(model),original={root:copy(root),rootChildren:copy(rootChildren),group:copy(group),children:copy(children),a:copy(a),b:copy(b),arrow:copy(arrow),edges:copy(edges),edge:copy(edge)},identity={root:model.root===root,rootChildren:model.root.children===rootChildren,group:model.root.children.includes(group),children:group.children===children,a:group.children.includes(a),edges:model.edges===edges,edge:model.edges.includes(edge)},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,...resources(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,original,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
