import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const source=await readFile(new URL('./support/gantt-owned-scheduler.js',import.meta.url),'utf8');
const packageRoot=process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package';
test.use({timezoneId:'UTC'});
test('Gantt owned scheduler retains warning reentry, task iteration, Date replies and original errors',async({page},info)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async({source,packageRoot})=>{
  const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
  try{
   const {runGanttOwnedScheduler}=await import(url);
   const {loadDiagramEntries,disposeDiagramModel}=await import(location.origin+packageRoot+'/diagram-entries.js');
   await loadDiagramEntries();
   const candidate=(await import(location.origin+packageRoot+'/diagrams/gantt.js')).default;
   const oracle=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/gantt.js')).default;
   const fixture=await(await fetch('/fixtures/public-gantt-owned-scheduler.json')).json();
   return {oracle:runGanttOwnedScheduler(oracle),candidate:runGanttOwnedScheduler(candidate,disposeDiagramModel),fixture};
  }finally{URL.revokeObjectURL(url);}
 },{source,packageRoot});
 const path=info.outputPath('gantt-owned-scheduler.json');await writeFile(path,JSON.stringify(report,null,2));
 await info.attach('gantt-owned-scheduler',{path,contentType:'application/json'});
 expect(report.fixture.oracleSHA256).toBe('d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
 expect(report.oracle).toHaveLength(20);expect(report.candidate).toHaveLength(20);
 expect(report.oracle).toEqual(report.fixture.cases);
 for(const row of report.oracle){
  expect(row.recovery.error,row.id+' recovery').toBeNull();
  if(row.mode==='warn-throw')expect(row.first.error,row.id+' original').toBe('original');
  if(row.mode==='warn-reentry')expect(row.first.trace.some(item=>item[0]==='call'),row.id+' warning callback').toBe(true);
 }
 // Exact same-engine errors stay in the comparison; no wording normalization.
 expect(report.candidate).toEqual(report.oracle);
});
