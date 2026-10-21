import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const source=await readFile(new URL('./support/set-helper-publication.js',import.meta.url),'utf8');
test('Set iterators survive public helper publication without revisiting unchanged values',async({page},info)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async source=>{
  const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
  try{
   const {MODES,runSetHelper}=await import(url);
   const {loadRenderer}=await import('/dist/package-web/package/full.js');
   const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
   const nativeEntry=(await import('/dist/package-web/package/diagrams/sequence.js')).default;
   const oracle=await import('/dist/evidence/pele-esm/core.js'),entry=(await import('/dist/evidence/pele-esm/diagrams/sequence.js')).default;oracle.register(entry);
   const fixture=await(await fetch('/fixtures/public-set-helper-publication.json')).json(),engine=await loadRenderer(),rows=[];
   try{for(const raw of [false,true])for(const mode of MODES){
    const run=(core,entry,dispose)=>{try{return runSetHelper(core,entry,dispose,raw,mode);}catch(error){return {fatal:{name:error.name,message:error.message}};}};
    rows.push({raw,mode,oracle:run(oracle,entry,()=>{}),candidate:run(engine,nativeEntry,m=>raw?disposeDiagramModel(m):engine.dispose(m))});
   }}finally{engine.close();}
   return {fixture,rows};
  }finally{URL.revokeObjectURL(url);}
 },source);
 const path=info.outputPath('set-helper-publication.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('set-helper-publication',{path,contentType:'application/json'});
 expect(report.fixture.oracleSHA256).toBe('d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
 expect(report.rows).toHaveLength(8);
 for(const row of report.rows){
  const id=`${row.raw}/${row.mode}`;
  expect(row.oracle.fatal,id).toBeUndefined();expect(row.oracle.fired,id).toBe(true);expect(row.oracle.identity,id).toBe(true);expect(row.oracle.hasAddedActor,id).toBe(true);
  expect(row.oracle,id+' pinned').toEqual(report.fixture.rows.find(v=>v.raw===row.raw&&v.mode===row.mode));
  expect(row.candidate,id).toEqual(row.oracle);
 }
});
