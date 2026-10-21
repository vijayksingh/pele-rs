import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const source=await readFile(new URL('./support/state-direction-accessor.js',import.meta.url),'utf8');
test('State direction distinguishes inherited getter and own undefined through callbacks and reentry',async({page},info)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async source=>{
  const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
  try{
   const {MODES,runStateDirection}=await import(url);
   const {loadRenderer}=await import('/dist/package-web/package/full.js');
   const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
   const entry=(await import('/dist/package-web/package/diagrams/state.js')).default,oracle=await import('/dist/evidence/pele-esm/core.js'),original=(await import('/dist/evidence/pele-esm/diagrams/state.js')).default;oracle.register(original);
   const fixture=await(await fetch('/fixtures/public-state-direction-accessor.json')).json(),engine=await loadRenderer(),rows=[];
   try{for(const raw of [false,true])for(const mode of MODES){const run=(core,entry,dispose)=>{try{return runStateDirection(core,entry,dispose,raw,mode);}catch(error){return {fatal:{name:error.name,message:error.message}};}};rows.push({raw,mode,oracle:run(oracle,original,()=>{}),candidate:run(engine,entry,m=>raw?disposeDiagramModel(m):engine.dispose(m))});}}finally{engine.close();}
   return {fixture,rows};
  }finally{URL.revokeObjectURL(url);}
 },source);
 const path=info.outputPath('state-direction-accessor.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('state-direction-accessor',{path,contentType:'application/json'});
 expect(report.fixture.oracleSHA256).toBe('d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');expect(report.rows).toHaveLength(20);
 for(const row of report.rows){const id=`${row.raw}/${row.mode}`;expect(row.oracle.fatal,id).toBeUndefined();expect(row.oracle.fired,id).toBe(row.mode!=='getter-throw');expect(row.oracle,id+' pinned').toEqual(report.fixture.rows.find(v=>v.raw===row.raw&&v.mode===row.mode));if(row.mode.startsWith('getter-')||row.mode==='getter-return')expect(row.oracle.getterTrace.length,id).toBeGreaterThan(0);if(row.mode==='getter-reentry')expect(row.oracle.nested?.svg,id).toBeTruthy();if(row.mode==='getter-throw')expect(row.oracle.first.original,id).toBe(true);expect(row.candidate,id).toEqual(row.oracle);}
});


test('State live getter capability stays captured and scoped through failure, helpers and fallback',async({page},info)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async()=>{
  const {loadRenderer}=await import('/dist/package-web/package/full.js');
  const {MutableDocument,NativeRegistry}=await import('/dist/package-web/package/pkg/pele.js');
  const {ModelBridge,ModelRegistry}=await import('/dist/package-web/package/graph.js');
  const {withDeclaredGetterScope}=await import('/dist/package-web/package/graph-storage.js');
  const oracle=await import('/dist/evidence/pele-esm/core.js');
  oracle.register((await import('/dist/evidence/pele-esm/diagrams/state.js')).default);
  const engine=await loadRenderer(),source='stateDiagram-v2\ndirection LR\nstate "Alpha" as A\nstate "Beta" as B\nA --> B: transition';
  const shape=r=>({svg:r.svg,width:r.width,height:r.height,links:r.links});
  const error=run=>{try{run();return null;}catch(e){return {name:e.name,message:e.message};}};
  function nested(core,dispose){
   const model=core.parse(source),proto=Object.getPrototypeOf(model),desc=Object.getOwnPropertyDescriptor(proto,'direction');
   const trace=[],sentinel={throw:true};let armed=true,busy=false,nested;
   Object.defineProperty(proto,'direction',{...desc,get(){
    trace.push('get');
    if(armed){armed=false;throw sentinel;}
    if(!busy){busy=true;try{this.setDirection('TB');nested=desc.get.call(this);}finally{busy=false;}}
    return desc.get.call(this);
   }});
   const options={measurer:{width:(t,s)=>t.length*s*.6}};
   try{
    let original=false;try{core.render(model,options);}catch(e){original=e===sentinel;}
    const recovered=shape(core.render(model,options));
    return {original,trace,nested,rootDoc:JSON.parse(JSON.stringify(model.rootDoc)),recovered};
   }finally{Object.defineProperty(proto,'direction',desc);dispose(model);}
  }
  const expected=nested(oracle,()=>{}),actual=nested(engine,m=>engine.dispose(m));
  const meta=Object.getOwnPropertyDescriptor(MutableDocument.prototype,'detached_public_accessors');
  const prepare=Object.getOwnPropertyDescriptor(MutableDocument.prototype,'prepare_detached_render_numbers');
  let metadataCalls=0;
  Object.defineProperty(MutableDocument.prototype,'detached_public_accessors',{...meta,value(){metadataCalls++;return new Uint32Array([1,1]);}});
  let model,proto,desc,method;const checks={};
  try{
   model=engine.parse(source);proto=Object.getPrototypeOf(model);desc=Object.getOwnPropertyDescriptor(proto,'direction');method=Object.getOwnPropertyDescriptor(proto,'setDirection');
   let reads=0;Object.defineProperty(proto,'direction',{...desc,get(){reads++;return desc.get.call(this);}});
   checks.authentic=!!engine.render(model,{measurer:{width:(t,s)=>t.length*s*.6}}).svg;
   checks.actualReads=reads;
   Object.defineProperty(MutableDocument.prototype,'prepare_detached_render_numbers',{...prepare,value:undefined});
   const priorReads=reads;checks.legacy=error(()=>engine.render(model));checks.legacyReadCount=reads-priorReads;
   Object.defineProperty(MutableDocument.prototype,'prepare_detached_render_numbers',prepare);
   Object.defineProperty(proto,'setDirection',{...method,value:function(){throw Error('Unexpected helper');}});
   checks.unrelated=error(()=>engine.render(model));
   Object.defineProperty(proto,'setDirection',method);
   Object.defineProperty(proto,'direction',{...desc,enumerable:!desc.enumerable});
   checks.flags=error(()=>engine.render(model));
   Object.defineProperty(proto,'direction',desc);
   checks.recovered=!!engine.render(model).svg;
   const registry=new NativeRegistry(true);let schema;
   try{schema=JSON.parse(registry.entry_schema('state')).result;}finally{registry.free();}
   const bridge=new ModelBridge({version:1,root:{kind:'undefined'},strings:[],nodes:[]},schema,new ModelRegistry(),()=>{});
   const copied=bridge.registration.prototypes.get(1),copiedDesc=Object.getOwnPropertyDescriptor(copied,'direction');
   Object.defineProperty(copied,'direction',{...copiedDesc,get(){throw Error('Getter must not run during admission');}});
   bridge.liveAccessors={scope:new Map([[copied,new Set(['direction'])]]),methods:new Set([1])};
   checks.forged=error(()=>bridge.snapshot());
   checks.directForged=error(()=>withDeclaredGetterScope(bridge,new Map([[copied,new Set(['direction'])]]),()=>bridge.snapshot()));
  }finally{
   Object.defineProperty(MutableDocument.prototype,'detached_public_accessors',meta);
   Object.defineProperty(MutableDocument.prototype,'prepare_detached_render_numbers',prepare);
   if(proto){Object.defineProperty(proto,'direction',desc);Object.defineProperty(proto,'setDirection',method);}
   if(model)engine.dispose(model);engine.close();
  }
  return {expected,actual,checks,metadataCalls};
 });
 const path=info.outputPath('state-live-getter-scope.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('state-live-getter-scope',{path,contentType:'application/json'});
 expect(report.actual).toEqual(report.expected);expect(report.actual.original).toBe(true);expect(report.actual.nested).toBe('TB');
 expect(report.metadataCalls).toBe(0);expect(report.checks.authentic).toBe(true);expect(report.checks.actualReads).toBeGreaterThan(0);expect(report.checks.legacyReadCount).toBe(0);expect(report.checks.recovered).toBe(true);
 for(const name of ['legacy','unrelated','flags','forged','directForged'])expect(report.checks[name],name).toEqual({name:'TypeError',message:'Prototype edits require an explicit native prototype mutation codec'});
});
