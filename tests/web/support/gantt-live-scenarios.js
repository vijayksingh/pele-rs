export const SOURCE='gantt\n title Initial title\n dateFormat YYYY-MM-DD\n section Main\n Alpha :a,2024-01-01,2d\n Beta :b,after a,2d\n section Later\n Gamma :c,after b,1d';
export const MODES=['baseline','current-label','next-label','dates','render-end','flags','classes','links','tasks-replace','tasks-append','section','axis','title-accessibility','today','throw-object','throw-undefined','nested','exclusions'];
export const cases=()=>[false,true].flatMap(raw=>[false,true].flatMap(responsive=>MODES.map(mode=>({raw,responsive,mode}))));
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height),links:v.links}),copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>copy({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,displayMode:m.displayMode,topAxis:m.topAxis,axisFormat:m.axisFormat,dateFormat:m.dateFormat,tickInterval:m.tickInterval??null,todayMarker:m.todayMarker,tasks:m.tasks,sections:m.sections,links:[...m.links],includes:m.includes,excludes:m.excludes,work:m.work,now:m.now});
export function runGantt(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);
 if(responsive){model.displayMode='compact';model.topAxis=true;}
 const tasks=model.tasks,a=tasks[0],b=tasks[1],c=tasks[2],sentinel={mode},trace=[];let fired=false,nested;
 const settings={now:Date.UTC(2024,0,3,12),...(responsive?{maxWidth:320}:{})};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const mutate=()=>{switch(mode){
 case'current-label':a.task='Changed current';break;
 case'next-label':b.task='Changed later';break;
 case'dates':a.startTime=new Date('2024-01-02T00:00:00.000Z');a.endTime=new Date('2024-01-04T00:00:00.000Z');b.endTime=new Date('2024-01-07T00:00:00.000Z');break;
 case'render-end':a.renderEndTime=new Date('2024-01-02T00:00:00.000Z');break;
 case'flags':a.active=true;a.crit=true;a.milestone=true;b.done=true;break;
 case'classes':a.classes.push('clickable');break;
 case'links':model.links.set('a','https://example.com/alpha');break;
 case'tasks-replace':model.tasks=[{...b,id:'replacement',task:'Replacement',classes:[...b.classes]}];break;
 case'tasks-append':tasks.push({...c,id:'appended',task:'Appended',classes:[...c.classes],order:3});break;
 case'section':a.section='Changed current section';b.section='Changed next section';break;
 case'axis':model.axisFormat='%d';model.tickInterval='1day';break;
 case'title-accessibility':model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';break;
 case'today':model.todayMarker='off';break;
 case'exclusions':model.excludes=['2024-01-02'];model.excludeSet=new Set(model.excludes);model.datedTokens=true;break;
 case'throw-object':case'throw-undefined':model.title='Before failure';b.task='Before failure task';throw mode==='throw-undefined'?undefined:sentinel;
 case'nested':model.title='Nested title';nested=output(draw({...settings,measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 const measure=(record,active)=>({width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active&&!fired&&t==='Alpha'){fired=true;mutate();}return t.length*s*.6;}});
 try{
 let first;try{first={ok:true,...output(draw({...settings,measurer:measure(trace,true)}))};}catch(e){first={ok:false,original:e===(mode==='throw-undefined'?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
 const after=state(model),original={tasks:copy(tasks),a:copy(a),b:copy(b),c:copy(c)},identity={tasks:model.tasks===tasks,a:model.tasks.includes(a),b:model.tasks.includes(b),c:model.tasks.includes(c)},recoveryTrace=[];
 let recovery;try{recovery={ok:true,...output(draw({...settings,measurer:measure(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
 return{raw,responsive,mode,fired,first,trace,after,original,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
