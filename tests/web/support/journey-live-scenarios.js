export const SOURCE='journey\ntitle Journey title\nsection First\nAlpha: 5: Ada, Bob\nBeta with an exceptionally long task label: 2: Bob\nsection Last\nGamma: 3: Ada';
export const MODES=['baseline','title','current-task','later-task','score','people','section','replace-tasks','accessibility','throw-object','throw-undefined','nested'];
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const state=model=>({title:model.title??null,accTitle:model.accTitle??null,accDescr:model.accDescr??null,tasks:model.tasks.map(t=>({task:t.task,score:bits(t.score),people:[...t.people],section:t.section}))});
export function runJourney(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);
 const oldTasks=model.tasks,oldTask=oldTasks[0],sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const mutate=()=>{switch(mode){
  case'title':model.title='Changed title';break;
  case'current-task':oldTask.task='Changed Alpha';break;
  case'later-task':oldTasks[1].task='Changed Beta';break;
  case'score':oldTask.score=1;oldTasks[1].score=NaN;break;
  case'people':oldTask.people=['New actor','Ada'];break;
  case'section':oldTask.section='Changed section';break;
  case'replace-tasks':model.tasks=[{task:'Replacement task',score:4,people:['Fresh actor'],section:'Fresh section'}];break;
  case'accessibility':model.accTitle='Changed access';model.accDescr='Changed description';break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldTask.task='Persisted task';throw mode==='throw-undefined'?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(entry.render(model,{}, {measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 try{
  let first;try{first={ok:true,...output(draw({...settings,measurer:{width(text,size,style){trace.push([text,bits(size),style??0]);if(!fired){fired=true;mutate();}return text.length*size*.6;}}}))};}catch(e){first={ok:false,original:e===(mode==='throw-undefined'?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after=state(model),identity={tasks:model.tasks===oldTasks,first:model.tasks[0]===oldTask},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
