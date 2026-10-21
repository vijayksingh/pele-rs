import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
import {ORACLE_SHA256} from './support/treemap-live-scenarios.js';
test('Treemap captured arrays and live fields preserve callbacks, nested renders and original errors',async({page},testInfo)=>{
 const url='/dist/evidence/treemap-live-scenarios.js',body=await readFile(new URL('./support/treemap-live-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,route=>route.fulfill({status:200,contentType:'text/javascript',body}));
 await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async p=>{const r=await fetch(p);if(!r.ok)throw Error('Missing '+p);return [...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));
  if(before[paths[0]]!==pin)throw Error('Pinned oracle changed');
  const {loadRenderer}=await import('/dist/package-web/package/full.js'),ready=await import('/dist/package-web/package/diagram-entries.js');await ready.loadDiagramEntries();
  const oracle=await import('/dist/evidence/pele-esm/core.js'),expectedEntry=(await import('/dist/evidence/pele-esm/diagrams/treemap.js')).default;oracle.register(expectedEntry);
  const entry=(await import('/dist/package-web/package/diagrams/treemap.js')).default,fixture=await(await fetch('/fixtures/public-treemap-live-render.json')).json();
  const {runTreemap}=await import(url),engine=await loadRenderer(),rows=[];
  try{for(const expected of fixture.rows){const {raw,responsive,mode}=expected;rows.push({raw,responsive,mode,oracle:runTreemap(oracle,expectedEntry,()=>{},fixture.source,raw,responsive,mode),candidate:runTreemap(engine,entry,m=>raw?ready.disposeDiagramModel(m):engine.dispose(m),fixture.source,raw,responsive,mode),fixture:expected});}}
  finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return{before,after,rows};
 },{url,pin:ORACLE_SHA256});
 const path=testInfo.outputPath('treemap-live-render.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('treemap-live-render',{path,contentType:'application/json'});
 expect(report.before).toEqual(report.after);expect(report.rows).toHaveLength(56);
 for(const row of report.rows){const id=`${row.raw}/${row.responsive}/${row.mode}`;expect(row.oracle,id+'/fixture').toEqual(row.fixture);expect(row.candidate,id).toEqual(row.oracle);}
});
