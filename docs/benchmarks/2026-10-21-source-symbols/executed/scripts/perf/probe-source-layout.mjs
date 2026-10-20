// Untimed actual initializer proof. Only this isolated server instruments source;
// disk packages and the separate timing harness serve the unmodified module.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {chromium} from '@playwright/test';
import {cases,now} from './cases.mjs';
const arg=name=>{const i=process.argv.indexOf(name);return i<0?undefined:process.argv[i+1];};
const roots={control:resolve(arg('--control')),candidate:resolve(arg('--candidate'))},output=resolve(arg('--output'));
const selected=['sankey/small','sankey/medium','sankey/large','flowchart/medium'],workloads=cases().filter(value=>selected.includes(value.id));
assert.equal(workloads.length,4);assert.notEqual(roots.control,roots.candidate);
const original=await readFile(resolve(roots.candidate,'source-initializer.js'),'utf8'),signature='export function tryInitializeSourceLayout(';
assert.equal(original.split(signature).length,2,'Actual initializer export missing or ambiguous');
const instrumented=original.replace(signature,'function actualInitializeSourceLayout(')+'\nexport function tryInitializeSourceLayout(bridge,graph){const used=actualInitializeSourceLayout(bridge,graph);(globalThis.__initializerDecisions??=[]).push(used);return used;}\n';
const report={schemaVersion:1,startedAt:new Date().toISOString(),roots,checks:[],selectors:false,directParseSPI:false,probeInsideTimer:false,instrumentationOnlyProbe:true,actualInitializerSHA256:createHash('sha256').update(original).digest('hex')};
const server=createServer(async(req,res)=>{try{
  res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
  const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Untimed initializer activation</title>');return;}
  const runtime=Object.keys(roots).find(name=>path.startsWith('/'+name+'/'));assert(runtime);
  const file=resolve(roots[runtime],'.'+path.slice(runtime.length+1));assert(file.startsWith(roots[runtime]+'/'));
  res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');
  res.end(path==='/candidate/source-initializer.js'?instrumented:await readFile(file));
}catch{res.writeHead(404);res.end();}});
await new Promise(done=>server.listen(0,'127.0.0.1',done));let browser;
const deadline=setTimeout(()=>void browser?.close(),120000);
try{
  browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();
  const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const checks=await page.evaluate(async({workloads,now})=>{
    Date.now=()=>now;const checks=[];
    const controlAPI=await import('/control/index.js');
    for(const name of ['control','candidate']){
      const api=await import('/'+name+'/index.js'),renderer=await api.loadRenderer();
      try{for(const workload of workloads)for(const profile of ['fixed','canvas']){
        const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:controlAPI.canvasMeasurer('sans-serif','monospace');
        if(!measurer)throw Error('Canvas unavailable');
        const options={...workload.options,measurer},start=(globalThis.__initializerDecisions??=[]).length;
        let model,sibling;
        try{
          model=renderer.parse(workload.source,options);sibling=renderer.parse(workload.source,options);
          const initializerDecisions=globalThis.__initializerDecisions.slice(start),expected=name==='candidate'&&workload.family==='sankey'?[true,true]:[];
          if(JSON.stringify(initializerDecisions)!==JSON.stringify(expected))throw Error(name+'/'+workload.id+'/'+profile+': actual initializer decisions '+JSON.stringify(initializerDecisions));
          const rendered=renderer.render(model,options);
          if(typeof rendered.svg!=='string'||typeof rendered.width!=='number'||typeof rendered.height!=='number')throw Error('Public retained render did not complete');
          checks.push({runtime:name,case:workload.id,profile,modelType:model.type,initializerDecisions,expected,publicRetainedRender:true});
        }finally{if(model)renderer.dispose(model);if(sibling)renderer.dispose(sibling);}
      }}finally{renderer.close();}
    }
    return checks;
  },{workloads,now});
  report.checks=checks;report.succeeded=true;
}catch(error){report.error={name:error.name,message:error.message,stack:error.stack};process.exitCode=1;}
finally{clearTimeout(deadline);await browser?.close();report.browserClosedBeforeTiming=true;await new Promise(done=>server.close(done));report.completedAt=new Date().toISOString();await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');}
