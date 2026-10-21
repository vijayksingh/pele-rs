import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
test('Flowchart and Swimlane call live subgraph helpers before graph projection and on responsive redraw',async({page},info)=>{
 const url='/dist/evidence/flow-subgraph-scenarios.js',body=await readFile(new URL('./support/flow-subgraph-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,r=>r.fulfill({status:200,contentType:'text/javascript',body}));await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async p=>{const r=await fetch(p);if(!r.ok)throw Error('Missing '+p);return [...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));if(before[paths[0]]!==pin)throw Error('Oracle changed');
  const {loadRenderer}=await import('/dist/package-web/package/full.js'),{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const oracle=await import('/dist/evidence/pele-esm/core.js'),engine=await loadRenderer(),{MODES,runSubgraph}=await import(url),fixture=await(await fetch('/fixtures/public-flow-subgraph-live.json')).json(),rows=[];
  try{for(const family of ['flowchart','swimlane']){const expected=(await import('/dist/evidence/pele-esm/diagrams/'+family+'.js')).default,entry=(await import('/dist/package-web/package/diagrams/'+family+'.js')).default;oracle.register(expected);
   for(const raw of [false,true])for(const responsive of [false,true])for(const mode of MODES)rows.push({family,raw,responsive,mode,oracle:runSubgraph(oracle,expected,()=>{},family,raw,responsive,mode),candidate:runSubgraph(engine,entry,m=>raw?disposeDiagramModel(m):engine.dispose(m),family,raw,responsive,mode),fixture:fixture.rows.find(r=>r.family===family&&r.raw===raw&&r.responsive===responsive&&r.mode===mode)});
  }}finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return {before,after,rows};
 },{url,pin});
 const path=info.outputPath('flow-subgraph-live.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('paired-subgraph-callbacks',{path,contentType:'application/json'});expect(report.after).toEqual(report.before);expect(report.rows).toHaveLength(80);
 for(const r of report.rows){const name=`${r.family}/${r.mode}/${r.raw}/${r.responsive}`;expect(r.oracle,name+'/pinned').toEqual(r.fixture);expect(r.candidate,name).toEqual(r.oracle);}
});
