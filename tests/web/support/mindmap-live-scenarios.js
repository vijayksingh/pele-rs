export const SOURCE='mindmap\nroot((Root))\n  alpha(Alpha)\n    leaf[Leaf]\n  beta[Beta]';
export const MODES=['baseline','title','current-descr','later-descr','type','decorate','replace-nodes','throw-object','throw-undefined','nested','icon-title','icon-current-descr','icon-throw-object','icon-nested'];
export const cases=()=>[...[true,false].flatMap(raw=>[false,true].flatMap(responsive=>MODES.slice(0,10).map(mode=>({raw,responsive,mode})))),...[false,true].flatMap(responsive=>MODES.slice(10).map(mode=>({raw:false,responsive,mode})))];
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const state=m=>({title:m.title??null,nodes:m.nodes.map(n=>({id:bits(n.id),nodeId:n.nodeId,descr:n.descr,type:bits(n.type),children:n.children.map(c=>bits(c.id)),class:n.class??null,icon:n.icon??null,section:n.section===undefined?null:bits(n.section)}))});
export function runMindmap(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);model.title='Initial title';model.nodes[0].icon='root-icon';model.nodes[1].icon='alpha-icon';
 const oldNodes=model.nodes,oldRoot=oldNodes[0],sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const pure={measurer:{width:(t,s)=>t.length*s*.6},icons:()=>'<path d="M0 0h1"/>'};
 const mutate=()=>{switch(mode.replace(/^icon-/,'')){
  case'title':model.title='Changed title';break;
  case'current-descr':oldRoot.descr='Changed Root';break;
  case'later-descr':oldNodes[1].descr='Changed Alpha';break;
  case'type':oldRoot.type=2;oldNodes[1].type=3;break;
  case'decorate':model.decorateNode({class:'decorated',icon:'decorated-icon'});break;
  case'replace-nodes':model.nodes=[{...oldRoot,descr:'Replacement root',children:[]}];break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldRoot.descr='Persisted root';throw mode.endsWith('undefined')?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(draw(pure));model.title='After nested';break;
 }};
 const invoke=kind=>{if(!fired&&((mode.startsWith('icon-')&&kind==='icon')||(!mode.startsWith('icon-')&&kind==='width'))){fired=true;mutate();}};
 const resources=(record,active)=>({measurer:{width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active)invoke('width');return t.length*s*.6;}},icons(name){record.push(['icon',name]);if(active)invoke('icon');return'<path d="M0 0h1"/>';}});
 try{
  let first;try{first={ok:true,...output(draw({...settings,...resources(trace,true)}))};}catch(e){first={ok:false,original:e===(mode.endsWith('undefined')?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after=state(model),identity={nodes:model.nodes===oldNodes,root:model.nodes[0]===oldRoot},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,...resources(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
