import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
import {ORACLE_SHA256} from './support/pie-info-live-scenarios.js';
const url='/dist/evidence/pie-info-live-scenarios.js';
for(const family of ['pie','info'])test(`${family} resource callbacks preserve live phases, nested renders and original throws`,async({page},testInfo)=>{
 const body=await readFile(new URL('./support/pie-info-live-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,route=>route.fulfill({status:200,contentType:'text/javascript',body}));
 await page.goto('/examples/web/');
 const report=await page.evaluate(async({family,url,pin})=>{
  const hash=async path=>{const response=await fetch(path);if(!response.ok)throw Error('Missing '+path);return [...new Uint8Array(await crypto.subtle.digest('SHA-256',await response.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));
  if(before[paths[0]]!==pin)throw Error('Pinned Pele changed');
  const {loadRenderer}=await import('/dist/package-web/package/full.js');
  const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  // ESM core and entry share the same oracle registry and raw module identity.
  const oracle=await import('/dist/evidence/pele-esm/core.js'),expectedEntry=(await import('/dist/evidence/pele-esm/diagrams/'+family+'.js')).default;oracle.register(expectedEntry);
  const entry=(await import('/dist/package-web/package/diagrams/'+family+'.js')).default;
  const scenarios=await import(url),engine=await loadRenderer(),rows=[];
  const fixture=await(await fetch('/fixtures/public-'+family+'-live-render.json')).json();
  try{
   for(const raw of [true,false]){
    if(family==='pie')for(const responsive of [false,true])for(const mode of ['baseline','title','showData','map-edit','map-replace','accessibility','throw-object','throw-undefined','nested-render']){
     rows.push({raw,responsive,mode,oracle:scenarios.runPie(oracle,expectedEntry,()=>{},raw,responsive,mode),candidate:scenarios.runPie(engine,entry,model=>raw?disposeDiagramModel(model):engine.dispose(model),raw,responsive,mode),fixture:fixture.rows.find(row=>row.raw===raw&&row.responsive===responsive&&row.mode===mode)});
    }
    else for(const mode of ['normal','title-mutate','version-mutate','throw','throw-undefined','nested']){
     rows.push({raw,mode,oracle:scenarios.runInfo(oracle,expectedEntry,()=>{},raw,mode),candidate:scenarios.runInfo(engine,entry,model=>raw?disposeDiagramModel(model):engine.dispose(model),raw,mode),fixture:fixture.cases.find(row=>row.mode===mode)});
    }
   }
  }finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return{before,after,rows};
 },{family,url,pin:ORACLE_SHA256});
 const path=testInfo.outputPath(family+'-live-render.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('public-live-render',{path,contentType:'application/json'});
 expect(report.after).toEqual(report.before);expect(report.rows).toHaveLength(family==='pie'?36:12);
 for(const row of report.rows){const id=family+'/'+row.raw+'/'+row.responsive+'/'+row.mode;expect(row.candidate,id).toEqual(row.oracle);if(row.fixture)expect(row.oracle,id+'/fixture').toEqual(row.fixture);}
});
