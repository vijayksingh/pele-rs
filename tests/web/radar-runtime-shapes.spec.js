import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
import {ORACLE_SHA256} from './support/radar-runtime-shapes.js';
test('Radar sparse axes and empty circle errors preserve geometry phases and callback state',async({page},testInfo)=>{
 const url='/dist/evidence/radar-runtime-shapes.js',body=await readFile(new URL('./support/radar-runtime-shapes.js',import.meta.url),'utf8');
 await page.route(`**${url}`,route=>route.fulfill({status:200,contentType:'text/javascript',body}));
 await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async p=>{const r=await fetch(p);if(!r.ok)throw Error('Missing '+p);return [...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));
  if(before[paths[0]]!==pin)throw Error('Pinned oracle changed');
  const {loadRenderer}=await import('/dist/package-web/package/full.js'),ready=await import('/dist/package-web/package/diagram-entries.js');await ready.loadDiagramEntries();
  const oracle=await import('/dist/evidence/pele-esm/core.js'),expectedEntry=(await import('/dist/evidence/pele-esm/diagrams/radar.js')).default;oracle.register(expectedEntry);
  const entry=(await import('/dist/package-web/package/diagrams/radar.js')).default,fixture=await(await fetch('/fixtures/public-radar-runtime-shapes.json')).json();
  const {runRadar}=await import(url),engine=await loadRenderer(),rows=[];
  try{for(const expected of fixture.rows){const {raw,mode}=expected;rows.push({raw,mode,oracle:runRadar(oracle,expectedEntry,()=>{},raw,mode),candidate:runRadar(engine,entry,m=>raw?ready.disposeDiagramModel(m):engine.dispose(m),raw,mode),fixture:expected});}}
  finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return{before,after,rows};
 },{url,pin:ORACLE_SHA256});
 const path=testInfo.outputPath('radar-runtime-shapes.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('radar-runtime-shapes',{path,contentType:'application/json'});
 expect(report.before).toEqual(report.after);expect(report.rows).toHaveLength(24);
 for(const row of report.rows){const id=`${row.raw}/${row.mode}`;const oracleFixture=structuredClone(row.oracle), pinned=structuredClone(row.fixture); if(!oracleFixture.first.ok&&!oracleFixture.first.original){expect(oracleFixture.first.error.name).toBe(pinned.first.error.name); oracleFixture.first.error.message=pinned.first.error.message;} expect(oracleFixture,id+'/fixture').toEqual(pinned);expect(row.candidate,id).toEqual(row.oracle);}
});
