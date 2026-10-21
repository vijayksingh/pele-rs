export const SOURCE='C4Context\nPerson(user,"Customer")\nSystem_Boundary(bank,"Bank") {\nSystem(app,"Banking")\nContainerDb(store,"Ledger","SQL")\n}\nRel(user,app,"Uses","HTTPS")\nRel(app,store,"Reads")';
export const MODES=['baseline','title','boundary-label','current-shape','later-shape','style','relation-label','relation-offset','replace-shapes','throw-object','throw-undefined','nested'];
export const cases=()=>[true,false].flatMap(raw=>[false,true].flatMap(responsive=>MODES.map(mode=>({raw,responsive,mode}))));
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const record=(v,fields)=>Object.fromEntries(fields.map(k=>[k,typeof v[k]==='number'?{bits:bits(v[k])}:v[k]??null]));
const state=m=>({title:m.title??null,shapeInRow:bits(m.shapeInRow),boundaryInRow:bits(m.boundaryInRow),shapes:m.shapes.map(n=>({...record(n,['alias','label','descr','techn','kind','shape','bgColor','fontColor','borderColor']),parent:n.parent?.alias??null})),boundaries:m.boundaries.map(n=>({...record(n,['alias','label','type','descr','bgColor','fontColor','borderColor']),parent:n.parent?.alias??null})),rels:m.rels.map(n=>record(n,['from','to','label','techn','descr','type','lineColor','textColor','offsetX','offsetY']))});
export function runC4(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);model.title='Initial title';
 const oldShapes=model.shapes,oldFirst=oldShapes[0],oldBoundary=model.boundaries[1],oldRel=model.rels[0],sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:300}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const mutate=()=>{switch(mode){
  case'helper-shadow':model.getC4Shape=0;break;
  case'title':model.title='Changed title';break;
  case'boundary-label':oldBoundary.label='Changed Bank';oldBoundary.descr='Late boundary description';break;
  case'current-shape':oldFirst.label='Changed customer';oldFirst.descr='Late current description';oldFirst.kind='System';break;
  case'later-shape':oldShapes[1].label='Changed Banking';oldShapes[1].techn='Late technology';break;
  case'style':model.updateElStyle('UpdateElementStyle',['user','#123456','#654321','#abcdef']);break;
  case'relation-label':oldRel.label='Changed relation';oldRel.techn='Late protocol';oldRel.descr='Late relation description';break;
  case'relation-offset':oldRel.offsetX=13;oldRel.offsetY=17;oldRel.textColor='#234567';break;
  case'replace-shapes':model.shapes=[...oldShapes].reverse();break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldFirst.label='Persisted customer';throw mode.endsWith('undefined')?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(draw({measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 const matches=t=>mode.startsWith('relation-')?t==='Uses':['current-shape','later-shape'].includes(mode)?t==='Customer':true;
 const resources=(record,active)=>({measurer:{width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active&&!fired&&matches(t)){fired=true;mutate();}return t.length*s*.6;}}});
 try{
  let first;try{first={ok:true,...output(draw({...settings,...resources(trace,true)}))};}catch(e){first={ok:false,original:e===(mode.endsWith('undefined')?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  if(mode==='helper-shadow')delete model.getC4Shape;
  const after=state(model),identity={shapes:model.shapes===oldShapes,first:model.shapes[0]===oldFirst,boundary:model.boundaries[1]===oldBoundary,relation:model.rels[0]===oldRel,lookup:model.getC4Shape('user')===oldFirst},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,...resources(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
