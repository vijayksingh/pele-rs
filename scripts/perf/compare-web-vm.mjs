// Diagnostic three-way comparison. The complete web-vm.mjs matrix remains the release gate.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir,readdir} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {cpus,totalmem,freemem,loadavg,platform,arch} from 'node:os';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {cases,revision,now} from './cases.mjs';
const arg=(name,fallback)=>{const i=process.argv.indexOf(name);return i<0?fallback:process.argv[i+1]};
const candidateNativeCanonicalSankey=process.argv.includes('--candidate-native-canonical-sankey');
const candidateCapturedSankey=candidateNativeCanonicalSankey||process.argv.includes('--candidate-captured-sankey');
const capturedModule='captured-owner.js';
const roots={control:resolve(arg('--control','dist/package-web/package')),candidate:resolve(arg('--candidate','dist/package-web/package'))};
assert.notEqual(roots.control,roots.candidate,'Use independent immutable packages');
const trials=Number(arg('--trials','15')),warmupMs=Number(arg('--warmup-ms','500'));
assert(Number.isInteger(trials)&&trials>=9&&trials<=21&&trials%3===0,'Balanced three-way trials must be a multiple of three');
assert(warmupMs>=500&&warmupMs<=2000);
const stages=arg('--stages','render-source,parse,render-model').split(','),profiles=arg('--profiles','canvas').split(','),ids=arg('--cases','flowchart/small,flowchart/medium,flowchart/large,sequence/small,er/small').split(',');
assert(stages.length&&new Set(stages).size===stages.length&&stages.every(s=>['render-source','parse','render-model'].includes(s)));
assert(profiles.length&&new Set(profiles).size===profiles.length&&profiles.every(p=>['fixed','canvas'].includes(p)));
const workloads=cases().filter(c=>ids.includes(c.id));assert(workloads.length===new Set(ids).size,'Unknown case');
const output=resolve(arg('--output','dist/evidence/compare-web-vm.json')),oracle=resolve('node_modules/pele/dist/pele.js'),sha=bytes=>createHash('sha256').update(bytes).digest('hex');
async function hashes(root,prefix=''){
 const files={};for(const entry of (await readdir(resolve(root,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(files,await hashes(root,name+'/'));else if(entry.isFile())files[name]=sha(await readFile(resolve(root,name)));else throw Error('Package contains a non-regular file');}return files;
}
const inputs=async()=>Object.fromEntries(await Promise.all(Object.entries(roots).map(async([key,root])=>[key,await hashes(root)])));
const before=await inputs(),oracleSHA256=sha(await readFile(oracle));assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
if(candidateCapturedSankey)assert(before.candidate[capturedModule],'Candidate package lacks captured-owner.js');
const capturedBackend={candidateEnabled:candidateCapturedSankey,controlEnabled:false,moduleSHA256:{control:before.control[capturedModule]??null,candidate:before.candidate[capturedModule]??null},probeInsideTimer:false};
if(candidateNativeCanonicalSankey)assert(before.candidate['native-storage.js'],'Candidate package lacks native-storage.js');
const nativeCanonicalBackend={candidateEnabled:candidateNativeCanonicalSankey,controlEnabled:false,moduleSHA256:Object.fromEntries(['control','candidate'].map(name=>[name,{owner:before[name][capturedModule]??null,storage:before[name]['native-storage.js']??null}])),probeInsideTimer:false};
const report={schemaVersion:1,diagnostic:true,startedAt:new Date().toISOString(),oracleRevision:revision,oracleSHA256,harnessSHA256:sha(await readFile(fileURLToPath(import.meta.url))),casesSHA256:sha(await readFile(new URL('./cases.mjs',import.meta.url))),runtimeHashes:before,capturedBackend,nativeCanonicalBackend,platform:{os:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,totalMemory:totalmem()},hostBefore:{loadAverage:loadavg(),freeMemory:freemem()},settings:{candidateNativeCanonicalSankey,candidateCapturedSankey,trials,warmupMs,batchTargetMs:12,maxBatch:128,balancedThreeWayOrder:true,matchedWasmBatchCounts:true,calibrationMedianOfThree:true,freshRetainedModelPerBatch:true},scope:'One browser/page holds independent control and candidate WASM instances and pinned JS. Balanced cyclic/reverse order; whole SVG and exact dimensions before/after. Shared heaps/GC/cache pressure differ from the release harness. Diagnostic, never a replacement release gate.',rows:[],cases:[]};
const server=createServer(async(req,res)=>{try{
 res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
 const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Controlled Pele comparison</title>');return;}
 let file;if(path==='/oracle.js')file=oracle;else{const runtime=Object.keys(roots).find(key=>path.startsWith('/'+key+'/'));assert(runtime);file=resolve(roots[runtime],'.'+path.slice(runtime.length+1));assert(file.startsWith(roots[runtime]+'/'));}
 res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
 }catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser;const deadline=setTimeout(()=>void browser?.close(),600000);
try{
 browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 const clock=await page.evaluate(async({now,candidateCapturedSankey,candidateNativeCanonicalSankey,controlHasCapturedModule})=>{
 Date.now=()=>now;const js=await import('/oracle.js'),controlAPI=await import('/control/index.js'),candidateAPI=await import('/candidate/index.js');
 let candidateOwner,controlOwner;
 if(candidateCapturedSankey){
  candidateOwner=await import('/candidate/captured-owner.js');
  if(candidateNativeCanonicalSankey&&typeof candidateOwner.capturedSankeyStorage!=='function')throw Error('Candidate package lacks native storage probe');
  if(controlHasCapturedModule){controlOwner=await import('/control/captured-owner.js');controlOwner.enableCapturedSankeyForTesting(false);}
 }
 const control=await controlAPI.loadRenderer();
 if(candidateCapturedSankey)candidateOwner.enableCapturedSankeyForTesting(true);
 const candidate=await candidateAPI.loadRenderer();
 const probeBackend=candidateCapturedSankey?(name,model,family,lane)=>{
  if(name==='js')return undefined;
  const actual=(name==='candidate'?candidateOwner:controlOwner)?.capturedSankeyBackend(model);
  const expected=name==='candidate'&&family==='sankey'?'source':undefined;
  if(actual!==expected)throw Error(name+'/'+family+'/'+lane+': expected backend '+String(expected)+', got '+String(actual));
  let storage;
  if(candidateNativeCanonicalSankey){
   const actualStorage=(name==='candidate'?candidateOwner:controlOwner)?.capturedSankeyStorage?.(model);
   const expectedStorage=name==='candidate'&&family==='sankey'?'native':undefined;
   if(actualStorage!==expectedStorage)throw Error(name+'/'+family+'/'+lane+': expected storage '+String(expectedStorage)+', got '+String(actualStorage));
   storage={expectedStorage:expectedStorage??null,actualStorage:actualStorage??null};
  }
  return {runtime:name,lane,expected:expected??null,actual:actual??null,...storage};
 }:undefined;
 window.comparison={runtimes:{js,control,candidate},controlAPI,probeBackend};
 let resolution=Infinity,last=performance.now();for(let i=0;i<4096;i++){const n=performance.now();if(n>last)resolution=Math.min(resolution,n-last);last=n;}return{crossOriginIsolated,observedClockResolutionMs:resolution};},{now,candidateCapturedSankey,candidateNativeCanonicalSankey,controlHasCapturedModule:!!before.control[capturedModule]});
 assert(clock.crossOriginIsolated&&clock.observedClockResolutionMs>0&&clock.observedClockResolutionMs<=.01,'High-resolution isolated timer required');Object.assign(report.settings,clock);
 for(const workload of workloads){
 const result=await page.evaluate(({workload,stages,profiles,trials,warmupMs})=>{
 const {runtimes,controlAPI,probeBackend}=window.comparison,names=['js','control','candidate'],rows=[],checks=[],backendChecks=[];let checksum=0;
 const quantile=(samples,p)=>[...samples].sort((a,b)=>a-b)[Math.max(0,Math.ceil(samples.length*p)-1)];
 for(const profile of profiles){
 const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:controlAPI.canvasMeasurer('sans-serif','monospace');if(!measurer)throw Error('Canvas unavailable');
 const options={...workload.options,measurer},models=Object.fromEntries(names.map(name=>[name,runtimes[name].parse(workload.source,options)]));
 const verify=()=>{const expected=runtimes.js.render(models.js,options);
 const equal=(actual,lane)=>{if(actual.svg!==expected.svg||!Object.is(actual.width,expected.width)||!Object.is(actual.height,expected.height))throw Error(workload.id+': '+lane+' output mismatch')};
 for(const name of names){const renderer=runtimes[name];
 if(probeBackend&&name!=='js')backendChecks.push({profile,...probeBackend(name,models[name],workload.family,'retained')});
 equal(renderer.render(models[name],options),name+'/retained');equal(renderer.render(workload.source,options),name+'/source');const fresh=renderer.parse(workload.source,options);
 try{if(probeBackend&&name!=='js')backendChecks.push({profile,...probeBackend(name,fresh,workload.family,'fresh-parse')});equal(renderer.render(fresh,options),name+'/fresh-parse')}finally{if(name!=='js')renderer.dispose(fresh)}}
 return{svg:expected.svg,width:expected.width,height:expected.height}};
 try{
 const baseline=verify();
 for(const stage of stages){
 const work=Object.fromEntries(names.map(name=>[name,()=>{const renderer=runtimes[name];if(stage==='parse'){const model=renderer.parse(workload.source,options);checksum+=model.type.length;if(name!=='js')renderer.dispose(model);}else{const r=renderer.render(stage==='render-source'?workload.source:models[name],options);checksum+=r.svg.length+r.width+r.height}}]));
 const batch=(name,count)=>{const renderer=runtimes[name];let model,run=work[name];if(stage==='render-model'){model=renderer.parse(workload.source,options);renderer.render(model,options);run=()=>{const r=renderer.render(model,options);checksum+=r.svg.length+r.width+r.height};}const start=performance.now();try{for(let i=0;i<count;i++)run();return performance.now()-start;}finally{if(model&&name!=='js')renderer.dispose(model)}};
 const counts={},warmups={},samples=Object.fromEntries(names.map(name=>[name,[]]));
 for(const name of names){let operations=0,start=performance.now();do{work[name]();operations++;}while(operations<6||performance.now()-start<warmupMs);warmups[name]={operations,elapsedMs:performance.now()-start};let count=1;while(count<128&&quantile(Array.from({length:3},()=>batch(name,count)),.5)<12)count*=2;counts[name]=count;}
 counts.control=counts.candidate=Math.min(counts.control,counts.candidate);
 for(let trial=0;trial<trials;trial++){let order=names.slice(trial%3).concat(names.slice(0,trial%3));if(Math.floor(trial/3)%2)order.reverse();for(const name of order)samples[name].push(batch(name,counts[name])/counts[name]);}
 const medianMs=Object.fromEntries(names.map(name=>[name,quantile(samples[name],.5)]));
 rows.push({case:workload.id,profile,stage,medianMs,candidateOverControl:medianMs.candidate/medianMs.control,candidateOverJS:medianMs.candidate/medianMs.js,controlOverJS:medianMs.control/medianMs.js,counts,warmups,samples,p95BatchMeanMs:Object.fromEntries(names.map(name=>[name,quantile(samples[name],.95)]))});
 }
 const after=verify();if(after.svg!==baseline.svg||!Object.is(after.width,baseline.width)||!Object.is(after.height,baseline.height))throw Error('Output changed after timing');checks.push({profile,svgBytes:new TextEncoder().encode(after.svg).length});
 }finally{for(const name of ['control','candidate'])runtimes[name].dispose(models[name]);}
 }
 return{rows,checks,backendChecks,checksum};},{workload,stages,profiles,trials,warmupMs});
 report.rows.push(...result.rows);report.cases.push({id:workload.id,sourceSHA256:sha(Buffer.from(workload.source)),sourceBytes:Buffer.byteLength(workload.source),checks:result.checks,backendChecks:result.backendChecks,checksum:result.checksum});
 await mkdir(dirname(output),{recursive:true});await writeFile(output+'.partial',JSON.stringify(report,null,2)+'\n');console.log('compared '+workload.id);
 }
 assert.deepEqual(await inputs(),before,'Package changed during comparison');report.completedAt=new Date().toISOString();report.hostAfter={loadAverage:loadavg(),freeMemory:freemem()};report.succeeded=true;await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{clearTimeout(deadline);await browser?.close();await new Promise(r=>server.close(r));}
