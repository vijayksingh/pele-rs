import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const source=await readFile(new URL('./support/gantt-calendar-start.js',import.meta.url),'utf8');
const packageRoot=process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package';
test.use({timezoneId:'UTC'});
test('Gantt getStartDate retains warning phases, task identities and calendar replies',async({page},info)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async({source,packageRoot})=>{
  const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
  try{
   const {runGanttCalendarStart}=await import(url);
   const {loadDiagramEntries,disposeDiagramModel}=await import(location.origin+packageRoot+'/diagram-entries.js');
   await loadDiagramEntries();
   const candidate=(await import(location.origin+packageRoot+'/diagrams/gantt.js')).default;
   const oracle=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/gantt.js')).default;
   const fixture=await(await fetch('/fixtures/public-gantt-calendar-start.json')).json();
   return {oracle:runGanttCalendarStart(oracle),candidate:runGanttCalendarStart(candidate,disposeDiagramModel),fixture};
  }finally{URL.revokeObjectURL(url);}
 },{source,packageRoot});
 const path=info.outputPath('gantt-calendar-start.json');await writeFile(path,JSON.stringify(report,null,2));
 await info.attach('gantt-calendar-start',{path,contentType:'application/json'});
 expect(report.fixture.oracleSHA256).toBe('d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
 expect(report.oracle).toHaveLength(17);expect(report.candidate).toHaveLength(17);
 for(const row of report.oracle){
  const pinned=report.fixture.cases.find(item=>item.id===row.id);
  for(const phase of ['first','recovery']){
   const {error,...actual}=row[phase],{error:pinnedError,...expected}=pinned[phase];
   expect(actual,row.id+' '+phase+' pinned state/trace/aliases').toEqual(expected);
   if(pinnedError&&typeof pinnedError==='object'&&pinnedError.name!=='PeleError')expect(error?.name).toBe(pinnedError.name);
   else expect(error).toEqual(pinnedError);
  }
 }
 // Exact same-engine errors stay in the comparison; no wording normalization.
 expect(report.candidate).toEqual(report.oracle);
});
