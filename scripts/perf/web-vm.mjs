// Same-browser, same-workload public API measurements. No paint/GPU speedup claim.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {cpus,totalmem,freemem,loadavg,platform,arch} from 'node:os';
import {createHash} from 'node:crypto';
import {chromium} from '@playwright/test';
import {cases,revision,now} from './cases.mjs';
import {fileURLToPath} from 'node:url';
const args=process.argv.slice(2),arg=(key,fallback)=>args.includes(key)?args[args.indexOf(key)+1]:fallback;
const root=resolve(arg('--root','.')),packageDir=resolve(arg('--package','dist/package-web/package'));
const output=resolve(arg('--output','dist/evidence/perf-web-vm.json')),trials=Number(arg('--trials','9'));
assert(Number.isInteger(trials)&&trials>=5&&trials<=21);
const warmupMs=Number(arg('--warmup-ms','100'));assert(Number.isFinite(warmupMs)&&warmupMs>=100&&warmupMs<=2000);
const selectedIDs=arg('--cases','').split(',').filter(Boolean);
const selectedStages=arg('--stages','render-source,parse,render-model').split(',');
const selectedProfiles=arg('--profiles','fixed,canvas').split(',');
const selectedCases=cases().filter(c=>!selectedIDs.length||selectedIDs.includes(c.id));
assert(selectedCases.length&&selectedIDs.every(id=>selectedCases.some(c=>c.id===id)),'Unknown case');
assert(selectedStages.length&&new Set(selectedStages).size===selectedStages.length&&selectedStages.every(s=>['render-source','parse','render-model'].includes(s)),'Unknown stage');
assert(selectedProfiles.length&&new Set(selectedProfiles).size===selectedProfiles.length&&selectedProfiles.every(p=>['fixed','canvas'].includes(p)),'Unknown profile');

const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputs=async()=>Object.fromEntries(await Promise.all(['full.js','index.js','initialize.js','graph.js','graph-binary.js','resources.js','pkg/pele.js','pkg/pele_bg.wasm'].map(async name=>[name,sha(await readFile(resolve(packageDir,name)))])));
const harnessSHA256=sha(await readFile(fileURLToPath(import.meta.url)));
const casesSHA256=sha(await readFile(new URL('./cases.mjs',import.meta.url)));
const before=await inputs(),oracle=resolve(root,'node_modules/pele/dist/pele.js'),oracleSHA256=sha(await readFile(oracle));
assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const server=createServer(async(request,response)=>{
  try{
    const path=decodeURIComponent(new URL(request.url,'http://localhost').pathname);
    if(path==='/'){response.setHeader('Content-Type','text/html');response.end('<!doctype html><title>Pele benchmark</title>');return;}
    const file=path==='/oracle.js'?oracle:resolve(packageDir,'.'+path.replace(/^\/package/,''));
    if(path!=='/oracle.js'&&(!path.startsWith('/package/')||!file.startsWith(packageDir+'/'))){response.writeHead(404);response.end();return;}
    response.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');response.end(await readFile(file));
  }catch{response.writeHead(404);response.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
let browser;
const report={schemaVersion:1,startedAt:new Date().toISOString(),oracleRevision:revision,oracleSHA256,harnessSHA256,casesSHA256,runtimeHashes:before,
  platform:{os:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,totalMemory:totalmem()},
  hostBefore:{loadAverage:loadavg(),freeMemory:freemem()},settings:{trials,batchTargetMs:12,maxBatch:128,minimumWarmupOperations:6,minimumWarmupMs:warmupMs,alternatingPairOrder:true,freshRetainedModelPerBatch:true},
  scope:'Matched browser/hardware/source/options/measurer; full SVG and dimensions exact before/after. Source render, mutable parse/dispose, retained mutable render include JS/WASM ownership and graph publication. Warm CPU work excludes startup/download, DOM/SVG painting and GPU. Shared VM services remain untouched.',rows:[],cases:[]};
const deadline=setTimeout(()=>{void browser?.close();},600_000);
try{
  browser=await chromium.launch({headless:true,timeout:20_000});report.browser=browser.version();
  const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
  await page.evaluate(async fixedNow=>{
    Date.now=()=>fixedNow;
    const api=await import('/package/index.js'),oracle=await import('/oracle.js');
    window.bench={api,oracle,engine:await api.loadRenderer()};
  },now);
  for(const workload of selectedCases){
    const result=await page.evaluate(async({workload,trials,selectedStages,selectedProfiles,warmupMs})=>{
      const {api,oracle,engine}=window.bench,rows=[],checks=[];let checksum=0;
      const quantile=(values,p)=>[...values].sort((a,b)=>a-b)[Math.max(0,Math.ceil(values.length*p)-1)];
      for(const profile of selectedProfiles){
        const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?0.71:0.6)}:api.canvasMeasurer('sans-serif','monospace');
        if(!measurer)throw Error('Canvas unavailable');
        const options={...workload.options,measurer},wasmModel=engine.parse(workload.source,options),jsModel=oracle.parse(workload.source,options);
        const verify=()=>{
          const wasm=engine.render(wasmModel,options),js=oracle.render(jsModel,options);
          if(wasm.svg!==js.svg||!Object.is(wasm.width,js.width)||!Object.is(wasm.height,js.height))throw Error(`${workload.id}/${profile}: full output mismatch`);
          return {svgBytes:new TextEncoder().encode(wasm.svg).length,width:wasm.width,height:wasm.height};
        };
        try{
          const baseline=verify();
          for(const stage of selectedStages){
            const work={
              wasm:stage==='parse'?()=>{const model=engine.parse(workload.source,options);checksum+=model.type.length;engine.dispose(model);}:()=>{const value=engine.render(stage==='render-source'?workload.source:wasmModel,options);checksum+=value.svg.length+value.width+value.height;},
              js:stage==='parse'?()=>{checksum+=oracle.parse(workload.source,options).type.length;}:()=>{const value=oracle.render(stage==='render-source'?workload.source:jsModel,options);checksum+=value.svg.length+value.width+value.height;},
            };
            const batch=(runtime,count)=>{
              let model,run=work[runtime];
              if(stage==='render-model'){
                const renderer=runtime==='wasm'?engine:oracle;
                model=renderer.parse(workload.source,options);renderer.render(model,options);
                run=()=>{const value=renderer.render(model,options);checksum+=value.svg.length+value.width+value.height;};
              }
              const start=performance.now();
              try{for(let n=0;n<count;n++)run();return performance.now()-start;}
              finally{if(model&&runtime==='wasm')engine.dispose(model);}
            };
            const counts={},samples={js:[],wasm:[]};
            const warmups={};
            for(const runtime of ['js','wasm']){
              let operations=0;const start=performance.now();
              do{work[runtime]();operations++;}while(operations<6||performance.now()-start<warmupMs);
              warmups[runtime]={operations,elapsedMs:performance.now()-start};
              let count=1;while(count<128&&batch(runtime,count)<12)count*=2;counts[runtime]=count;
            }
            for(let trial=0;trial<trials;trial++)for(const runtime of trial%2?['wasm','js']:['js','wasm'])samples[runtime].push(batch(runtime,counts[runtime])/counts[runtime]);
            const jsMedianMs=quantile(samples.js,.5),wasmMedianMs=quantile(samples.wasm,.5);
            rows.push({case:workload.id,profile,stage,jsMedianMs,wasmMedianMs,wasmOverJS:wasmMedianMs/jsMedianMs,
              jsP95BatchMeanMs:quantile(samples.js,.95),wasmP95BatchMeanMs:quantile(samples.wasm,.95),counts,warmups,samples});
          }
          const after=verify();if(JSON.stringify(after)!==JSON.stringify(baseline))throw Error('Output changed after timing');checks.push({profile,...baseline});
        }finally{engine.dispose(wasmModel);}
      }
      return {rows,checks,checksum};
    },{workload,trials,selectedStages,selectedProfiles,warmupMs});
    report.rows.push(...result.rows);report.cases.push({id:workload.id,sourceSHA256:sha(Buffer.from(workload.source)),sourceBytes:Buffer.byteLength(workload.source),checks:result.checks,checksum:result.checksum});
    await mkdir(dirname(output),{recursive:true});await writeFile(output+'.partial',JSON.stringify(report,null,2)+'\n');console.log('measured '+workload.id);
  }
  assert.deepEqual(await inputs(),before,'Package changed during measurement');report.completedAt=new Date().toISOString();report.hostAfter={loadAverage:loadavg(),freeMemory:freemem()};report.succeeded=true;
  await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{clearTimeout(deadline);await browser?.close();await new Promise(resolve=>server.close(resolve));}
