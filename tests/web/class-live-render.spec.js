import{test,expect}from'@playwright/test';
import{readFile,writeFile}from'node:fs/promises';
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
test('Class resource callbacks preserve captured collection phases, late metadata, original errors and nested recovery',async({page},testInfo)=>{
 const url='/dist/evidence/class-live-scenarios.js',body=await readFile(new URL('./support/class-live-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,route=>route.fulfill({status:200,contentType:'text/javascript',body}));await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async path=>{const r=await fetch(path);if(!r.ok)throw Error('Missing '+path);return[...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));if(before[paths[0]]!==pin)throw Error('Oracle changed');
  const{loadRenderer}=await import('/dist/package-web/package/full.js');const{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const oracle=await import('/dist/evidence/pele-esm/core.js'),expectedEntry=(await import('/dist/evidence/pele-esm/diagrams/class.js')).default;oracle.register(expectedEntry);
  const entry=(await import('/dist/package-web/package/diagrams/class.js')).default,{SOURCE,MODES,runClass}=await import(url),engine=await loadRenderer(),rows=[];
  const fixture=await(await fetch('/fixtures/public-class-live-render.json')).json();
  try{for(const raw of[true,false])for(const responsive of[false,true])for(const mode of MODES)rows.push({raw,responsive,mode,oracle:runClass(oracle,expectedEntry,()=>{},raw,responsive,mode),candidate:runClass(engine,entry,m=>raw?disposeDiagramModel(m):engine.dispose(m),raw,responsive,mode),fixture:fixture.rows.find(r=>r.raw===raw&&r.responsive===responsive&&r.mode===mode)});}finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return{before,after,rows};
 },{url,pin});
 const path=testInfo.outputPath('class-live-render.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('paired-live-render',{path,contentType:'application/json'});expect(report.after).toEqual(report.before);expect(report.rows).toHaveLength(80);
 for(const row of report.rows){const id=`${row.raw}/${row.responsive}/${row.mode}`;expect(row.oracle,id+'/pinned').toEqual(row.fixture);expect(row.candidate,id).toEqual(row.oracle);}
});
