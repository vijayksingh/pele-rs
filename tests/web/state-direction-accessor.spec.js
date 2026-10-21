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
