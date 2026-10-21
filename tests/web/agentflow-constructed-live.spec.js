import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
test('constructed Agentflow caches successful graph builds across live diagnostics, original throws and nested recovery',async({page},info)=>{
 const url='/dist/evidence/agentflow-constructed-scenarios.js';const body=await readFile(new URL('./support/agentflow-constructed-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,r=>r.fulfill({status:200,contentType:'text/javascript',body}));await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async p=>{const r=await fetch(p);if(!r.ok)throw Error('Missing '+p);return [...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(x=>x.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));if(before[paths[0]]!==pin)throw Error('Oracle changed');
  const {loadRenderer}=await import('/dist/package-web/package/full.js');const{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const candidate=(await import('/dist/package-web/package/diagrams/agentflow.js')).default,oracle=(await import('/dist/evidence/pele-esm/diagrams/agentflow.js')).default,engine=await loadRenderer();
  const {MODES,runConstructedAgentflow}=await import(url),source='agentflow-beta LR\nA[Alpha] --> B[Beta]';
  const captured=candidate.parse(source,{},undefined,0,{maxEdges:500}),C=captured.constructor;disposeDiagramModel(captured);
  const expectedC=oracle.parse(source,{},undefined,0,{maxEdges:500}).constructor;
  const fixture=await(await fetch('/fixtures/public-agentflow-constructed-live.json')).json(),rows=[];
  try{for(const trigger of ['shape','containment'])for(const mode of MODES)rows.push({trigger,mode,candidate:runConstructedAgentflow(candidate,C,disposeDiagramModel,trigger,mode),oracle:runConstructedAgentflow(oracle,expectedC,()=>{},trigger,mode),fixture:fixture.rows.find(r=>r.trigger===trigger&&r.mode===mode)});}finally{engine.close();}
  return{before,after:Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)]))),rows};
 },{url,pin});
 const path=info.outputPath('agentflow-constructed-live.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('actual-oracle',{path,contentType:'application/json'});expect(report.after).toEqual(report.before);expect(report.rows).toHaveLength(18);
 for(const r of report.rows){expect(r.oracle,r.trigger+'/'+r.mode+'/pin').toEqual(r.fixture);expect(r.candidate,r.trigger+'/'+r.mode).toEqual(r.oracle);}
});
