import {test,expect} from '@playwright/test';
const source='gantt\ndateFormat YYYY-MM-DD\nsection S\nA:a,2024-01-01,1d\nB:b,after a,2d';

test('Gantt host-edited task records grow, shrink and recover during native delegation',async({page})=>{
 await page.goto('/examples/web/');
 const result=await page.evaluate(async(source)=>{
  const {loadRenderer}=await import('/dist/package-web/package/full.js');
  const oracle=await import('/dist/evidence/pele-oracle.js'),native=await loadRenderer();
  function exercise(api){
   const model=api.parse(source),original=model.getTasks,checks=[];
   model.getTasks=function(now){
    const tasks=original.call(this,now),first=tasks[0];
    for(let index=0;index<40;index++)first[`extension${index}`]=index;
    first.task='Expanded task';
    this.setClass('a','expanded');
    checks.push(original.call(this,now)===tasks,first.task==='Expanded task');
    for(let index=0;index<40;index++){
     checks.push(first[`extension${index}`]===index);
     delete first[`extension${index}`];
    }
    first.task='Restored task';
    this.setClass('a','restored');
    checks.push(original.call(this,now)===tasks,first.task==='Restored task');
    checks.push(first.endTime===tasks[1].startTime);
    return tasks;
   };
   try{
    const drawings=Array.from({length:2},()=>api.render(model,{
     now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6},
    }));
    return{drawings,checks,keys:Object.keys(model.tasks[0]),classes:[...model.tasks[0].classes]};
   }finally{api.dispose?.(model);api.close?.();}
  }
  return{native:exercise(native),oracle:exercise(oracle)};
 },source);
 expect(result.native).toEqual(result.oracle);
 expect(result.native.checks.every(Boolean)).toBe(true);
});

test('Gantt live getTasks preserves captured native calls, alternate arrays, Date aliases and mutable state',async({page})=>{
 await page.goto('/examples/web/');
 const result=await page.evaluate(async(source)=>{
  const {loadRenderer}=await import('/dist/package-web/package/full.js'), oracle=await import('/dist/evidence/pele-oracle.js'), native=await loadRenderer();
  function exercise(api){const all=[];for(const mode of ['wrapped','alternate','capturedFlags','beforeAndAfter','repeat']){
   const model=api.parse(source), original=model.getTasks, calls=[], checks=[];let savedTasks,savedDate;
   model.getTasks=function(now){
    checks.push(this===model,now===1704067200000);
    calls.push(['host',now]);
    if(mode==='beforeAndAfter'){this.tasks[0].task='Before native';this.extra={self:this,tasks:this.tasks,utf16:'\ud800'};this.extra.self=this.extra;}
    const tasks=original.call(this,now);
    checks.push(tasks===this.tasks,tasks[0].endTime===tasks[1].startTime,tasks[0].startTime instanceof Date);
    savedTasks=tasks;savedDate=tasks[0].startTime;tasks[0].task='Changed live';
    if(mode==='capturedFlags'){this.displayMode='compact';this.topAxis=true;}
    if(mode==='repeat'){const next=original.call(this,now+86400000);checks.push(next===tasks,next[0].endTime===next[1].startTime);}
    return mode==='alternate'?[tasks[0]]:tasks;
   };
   const drawing=api.render(model,{now:1704067200000,measurer:{width(text,size,style){calls.push(['width',text,size,style]);return text.length*size*.6;}}});
   checks.push(model.tasks===savedTasks,mode==='repeat'||model.tasks[0].startTime===savedDate,model.getTasks.name==='');
   if(mode==='beforeAndAfter')checks.push(model.extra.self===model.extra,model.extra.tasks===model.tasks,model.extra.utf16==='\ud800');
   all.push({mode,drawing,calls,checks,state:{work:model.work,now:model.now,tasks:model.tasks.map(t=>({task:t.task,start:+t.startTime,end:+t.endTime,processed:t.processed})),excludes:[...model.excludeSet],includes:[...model.includeSet]}});api.dispose?.(model);
  }api.close?.();return all;}
  return {native:exercise(native),oracle:exercise(oracle)};
 },source);
 expect(result.native).toEqual(result.oracle);for(const entry of result.native)expect(entry.checks.every(Boolean),entry.mode).toBe(true);
});

test('Gantt host throws publish partial edits, keep exact thrown values, recover and precede output policy',async({page})=>{
 await page.goto('/examples/web/');
 const result=await page.evaluate(async(source)=>{
  const {loadRenderer}=await import('/dist/package-web/package/full.js'),oracle=await import('/dist/evidence/pele-oracle.js'),native=await loadRenderer();
  function exercise(api){const model=api.parse(source),original=model.getTasks,checks=[],drawings=[],states=[];
   const markers=[undefined,null,false,-0,'original exception',new Error('original error'),{marker:true}];
   for(const marker of markers){let caught=false;model.getTasks=function(now){const tasks=original.call(this,now);tasks[0].task='Retained before throw';this.partial={tasks,date:tasks[0].endTime};throw marker;};
    try{api.render(model,{now:1704067200000,outputLimit:0});}catch(error){caught=true;checks.push(Object.is(error,marker));}
    checks.push(caught,model.tasks[0].task==='Retained before throw',model.partial.tasks===model.tasks,model.partial.date===model.tasks[1].startTime);
    states.push([model.tasks[0].task,+model.tasks[0].endTime,model.work]);model.getTasks=original;drawings.push(api.render(model,{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}}));
   }
   model.getTasks=function(now){const tasks=original.call(this,now);tasks[0].task='Retained after limit';return tasks;};
   try{api.render(model,{now:1704067200000,outputLimit:0,measurer:{width:(text,size)=>text.length*size*.6}});}catch(error){checks.push(error.code==='limit');}
   checks.push(model.tasks[0].task==='Retained after limit');model.getTasks=original;drawings.push(api.render(model,{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}}));
   api.dispose?.(model);api.close?.();return{checks,drawings,states};
  }return{native:exercise(native),oracle:exercise(oracle)};
 },source);
 expect(result.native).toEqual(result.oracle);expect(result.native.checks.every(Boolean)).toBe(true);
});

test('Gantt captured native calls can catch host clock failures and recover inside the override',async({page})=>{
 await page.goto('/examples/web/');
 const result=await page.evaluate(async(source)=>{
  const {loadRenderer}=await import('/dist/package-web/package/full.js'),oracle=await import('/dist/evidence/pele-oracle.js'),native=await loadRenderer();
  function exercise(api){const model=api.parse(source),original=model.getTasks,checks=[],trace=[],marker={message:'clock identity'},saved=Date.now;
   model.getTasks=function(now){
    Date.now=()=>{trace.push('clock');throw marker;};
    try{original.call(this,NaN);checks.push(false);}catch(error){checks.push(error===marker);}finally{Date.now=saved;}
    const tasks=original.call(this,now);checks.push(tasks===this.tasks,tasks[0].endTime===tasks[1].startTime);return tasks;
   };
   let drawing;try{drawing=api.render(model,{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}});}finally{Date.now=saved;api.dispose?.(model);api.close?.();}
   return{drawing,trace,checks};
  }return{native:exercise(native),oracle:exercise(oracle)};
 },source);
 expect(result.native).toEqual(result.oracle);expect(result.native.checks.every(Boolean)).toBe(true);
});

test('retained Gantt callback documents survive registry close and reject disposal or rendering during the active call',async({page})=>{
 await page.goto('/examples/web/');
 const result=await page.evaluate(async(source)=>{
  const {loadRegistry}=await import('/dist/package-web/package/registry.js');const api=await loadRegistry();api.registerBuiltin('gantt');
  const model=api.parse(source),original=model.getTasks,checks=[];api.close();
  model.getTasks=function(now){
   for(const operation of [()=>api.dispose(model),()=>api.render(model,{now})]){try{operation();checks.push(false);}catch(error){checks.push(error instanceof TypeError&&error.message.includes('already in use'));}}
   const tasks=original.call(this,now);checks.push(tasks===this.tasks);return tasks;
  };
  const drawing=api.render(model,{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}});model.getTasks=original;const next=api.render(model,{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}});api.dispose(model);
  try{original.call(model,1704067200000);checks.push(false);}catch(error){checks.push(error instanceof TypeError);}
  return{checks,same:drawing.svg===next.svg};
 },source);
 expect(result.same).toBe(true);expect(result.checks.every(Boolean)).toBe(true);
});

test('prepared Gantt raw render invokes the same live method while preserving raw limits and exception identity',async({page})=>{
 await page.goto('/examples/web/');
 const result=await page.evaluate(async(source)=>{
  const ready=await import('/dist/package-web/package/diagram-entries.js');await ready.loadDiagramEntries();
  const native=(await import('/dist/package-web/package/diagrams/gantt.js')).default,oracle=(await import('/dist/evidence/pele-esm/diagrams/gantt.js')).default;
  function exercise(api,dispose){const model=api.parse(source,{},undefined,0,{maxEdges:undefined}),original=model.getTasks,checks=[],trace=[];
   model.getTasks=function(now){trace.push(['host',now]);const tasks=original.call(this,now);checks.push(tasks===this.tasks,tasks[0].endTime===tasks[1].startTime);tasks[0].task='Raw override';return [tasks[0]];};
   const options={now:1704067200000,outputLimit:0,limit:-1,maxEdges:-1,measurer:{width(text,size,style){trace.push(['width',text,size,style]);return text.length*size*.6;}}};
   const drawing=api.render(model,{},options),marker=new RangeError('raw exception identity');
   model.getTasks=function(now){const tasks=original.call(this,now);tasks[0].task='Raw partial';throw marker;};
   try{api.render(model,{},options);checks.push(false);}catch(error){checks.push(error===marker,model.tasks[0].task==='Raw partial');}
   model.getTasks=original;const recovered=api.render(model,{},options);dispose?.(model);return{drawing,recovered,trace,checks};
  }return{native:exercise(native,ready.disposeDiagramModel),oracle:exercise(oracle)};
 },source);
 expect(result.native).toEqual(result.oracle);expect(result.native.checks.every(Boolean)).toBe(true);
});
