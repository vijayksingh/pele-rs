// Complete parse + first-use API costs. The web-vm.mjs matrix remains the release gate.
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
const controlNativeCanonicalSankey=process.argv.includes('--control-native-canonical-sankey');
assert(!controlNativeCanonicalSankey||candidateNativeCanonicalSankey,'--control-native-canonical-sankey requires --candidate-native-canonical-sankey');
const candidateCapturedSankey=candidateNativeCanonicalSankey||process.argv.includes('--candidate-captured-sankey');
const capturedModule='captured-owner.js';
const roots={control:resolve(arg('--control','dist/package-web/package')),candidate:resolve(arg('--candidate','dist/package-web/package'))};
assert.notEqual(roots.control,roots.candidate,'Use independent immutable packages');
const trials=Number(arg('--trials','15')),warmupMs=Number(arg('--warmup-ms','500'));
assert(Number.isInteger(trials)&&trials>=9&&trials<=21&&trials%3===0,'Balanced three-way trials must be a multiple of three');
assert(warmupMs>=500&&warmupMs<=2000);
const stages=arg('--stages','render-source,parse-first-render,parse-first-edited-render,parse-first-invoke,parse-first-callback').split(','),profiles=arg('--profiles','canvas').split(','),ids=arg('--cases','sequence/small,flowchart/medium,er/medium,gantt/medium').split(',');
assert(stages.length&&new Set(stages).size===stages.length&&stages.every(s=>['render-source','parse-first-render','parse-first-edited-render','parse-first-invoke','parse-first-callback'].includes(s)));
assert(profiles.length&&new Set(profiles).size===profiles.length&&profiles.every(p=>['fixed','canvas'].includes(p)));
const workloads=cases().filter(c=>ids.includes(c.id));assert(workloads.length===new Set(ids).size,'Unknown case');
assert(workloads.every(workload=>stages.some(stage=>['gantt','er'].includes(workload.family)||!['parse-first-invoke','parse-first-callback'].includes(stage))),'No applicable first-use stage for a selected case');
const output=resolve(arg('--output','dist/evidence/compare-first-use-web-vm.json')),oracle=resolve('node_modules/pele/dist/pele.js'),sha=bytes=>createHash('sha256').update(bytes).digest('hex');
async function hashes(root,prefix=''){
 const files={};for(const entry of (await readdir(resolve(root,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(files,await hashes(root,name+'/'));else if(entry.isFile())files[name]=sha(await readFile(resolve(root,name)));else throw Error('Package contains a non-regular file');}return files;
}
const inputs=async()=>Object.fromEntries(await Promise.all(Object.entries(roots).map(async([key,root])=>[key,await hashes(root)])));
const before=await inputs(),oracleSHA256=sha(await readFile(oracle));assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
if(candidateCapturedSankey)assert(before.candidate[capturedModule],'Candidate package lacks captured-owner.js');
const capturedBackend={candidateEnabled:candidateCapturedSankey,controlEnabled:controlNativeCanonicalSankey,moduleSHA256:{control:before.control[capturedModule]??null,candidate:before.candidate[capturedModule]??null},probeInsideTimer:false};
if(candidateNativeCanonicalSankey)assert(before.candidate['native-storage.js'],'Candidate package lacks native-storage.js');
if(controlNativeCanonicalSankey)assert(before.control[capturedModule]&&before.control['native-storage.js'],'Control package lacks native storage modules');
const nativeCanonicalBackend={candidateEnabled:candidateNativeCanonicalSankey,controlEnabled:controlNativeCanonicalSankey,moduleSHA256:Object.fromEntries(['control','candidate'].map(name=>[name,{owner:before[name][capturedModule]??null,storage:before[name]['native-storage.js']??null}])),probeInsideTimer:false};
const report={schemaVersion:1,diagnostic:true,startedAt:new Date().toISOString(),oracleRevision:revision,oracleSHA256,harnessSHA256:sha(await readFile(fileURLToPath(import.meta.url))),casesSHA256:sha(await readFile(new URL('./cases.mjs',import.meta.url))),runtimeHashes:before,capturedBackend,nativeCanonicalBackend,platform:{os:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,totalMemory:totalmem()},hostBefore:{loadAverage:loadavg(),freeMemory:freemem()},settings:{controlNativeCanonicalSankey,candidateNativeCanonicalSankey,candidateCapturedSankey,stages,profiles,selectedCases:ids,trials,warmupMs,batchTargetMs:12,maxBatch:128,balancedThreeWayOrder:true,matchedWasmBatchCounts:true,calibrationMedianOfThree:true,freshDocumentPerOperation:true,disposeInsideTimer:true},scope:'One browser/page holds independent control/candidate WASM instances and pinned JS. Each operation includes parse, first API use and disposal. Invoke/callback stages cover Gantt getTasks and ER addEntity/first measuring callback; other families cover first unchanged/edited render. Source is a negative control. Complete SVG, dimensions and edited-object identity checked before/after. Shared heaps and serial warmup remain limitations. Diagnostic, never the release gate.',rows:[],cases:[]};
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
 const clock=await page.evaluate(async({now,candidateCapturedSankey,candidateNativeCanonicalSankey,controlNativeCanonicalSankey,controlHasCapturedModule})=>{
 Date.now=()=>now;const js=await import('/oracle.js'),controlAPI=await import('/control/index.js'),candidateAPI=await import('/candidate/index.js');
 let candidateOwner,controlOwner;
 if(candidateCapturedSankey){
  candidateOwner=await import('/candidate/captured-owner.js');
  if(candidateNativeCanonicalSankey&&typeof candidateOwner.capturedSankeyStorage!=='function')throw Error('Candidate package lacks native storage probe');
  if(controlHasCapturedModule){controlOwner=await import('/control/captured-owner.js');if(controlNativeCanonicalSankey&&typeof controlOwner.capturedSankeyStorage!=='function')throw Error('Control package lacks native storage probe');controlOwner.enableCapturedSankeyForTesting(controlNativeCanonicalSankey);}
 }
 const control=await controlAPI.loadRenderer();
 if(candidateCapturedSankey)candidateOwner.enableCapturedSankeyForTesting(true);
 const candidate=await candidateAPI.loadRenderer();
 const probeBackend=candidateCapturedSankey?(name,model,family,lane)=>{
  if(name==='js')return undefined;
  const actual=(name==='candidate'?candidateOwner:controlOwner)?.capturedSankeyBackend(model);
  const expected=(name==='candidate'||controlNativeCanonicalSankey)&&family==='sankey'?'source':undefined;
  if(actual!==expected)throw Error(name+'/'+family+'/'+lane+': expected backend '+String(expected)+', got '+String(actual));
  let storage;
  if(candidateNativeCanonicalSankey){
   const actualStorage=(name==='candidate'?candidateOwner:controlOwner)?.capturedSankeyStorage?.(model);
   const expectedStorage=(name==='candidate'||controlNativeCanonicalSankey)&&family==='sankey'?'native':undefined;
   if(actualStorage!==expectedStorage)throw Error(name+'/'+family+'/'+lane+': expected storage '+String(expectedStorage)+', got '+String(actualStorage));
   storage={expectedStorage:expectedStorage??null,actualStorage:actualStorage??null};
  }
  return {runtime:name,lane,expected:expected??null,actual:actual??null,...storage};
 }:undefined;
 window.comparison={runtimes:{js,control,candidate},controlAPI,probeBackend};
 let resolution=Infinity,last=performance.now();for(let i=0;i<4096;i++){const n=performance.now();if(n>last)resolution=Math.min(resolution,n-last);last=n;}return{crossOriginIsolated,observedClockResolutionMs:resolution};},{now,candidateCapturedSankey,candidateNativeCanonicalSankey,controlNativeCanonicalSankey,controlHasCapturedModule:!!before.control[capturedModule]});
 assert(clock.crossOriginIsolated&&clock.observedClockResolutionMs>0&&clock.observedClockResolutionMs<=.01,'High-resolution isolated timer required');Object.assign(report.settings,clock);
 for(const workload of workloads){
 const result=await page.evaluate(async({workload,stages,profiles,trials,warmupMs})=>{
 const {runtimes,controlAPI,probeBackend}=window.comparison,names=['js','control','candidate'],rows=[],checks=[],backendChecks=[];let checksum=0;
 const quantile=(samples,p)=>[...samples].sort((a,b)=>a-b)[Math.max(0,Math.ceil(samples.length*p)-1)];
 for(const profile of profiles){
 const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:controlAPI.canvasMeasurer('sans-serif','monospace');if(!measurer)throw Error('Canvas unavailable');
 const options={...workload.options,measurer};
 const applicable=stages.filter(stage=>['gantt','er'].includes(workload.family)||!['parse-first-invoke','parse-first-callback'].includes(stage));
 // Snapshot first method values before a verification render can reschedule.
 const taskSignature=value=>{
  const ids=new Map(),pending=[],nodes=[],bits=new DataView(new ArrayBuffer(8));
  const atom=value=>{
   if(value===undefined)return ['undefined'];
   if(value===null)return ['null'];
   if(typeof value==='number'){bits.setFloat64(0,value);return ['number',bits.getBigUint64(0).toString(16).padStart(16,'0')];}
   if(typeof value==='string'||typeof value==='boolean')return [typeof value,value];
   if(typeof value!=='object')throw Error('Unexpected first-method signature value');
   if(!ids.has(value)){ids.set(value,pending.length);pending.push(value);}
   return ['ref',ids.get(value)];
  };
  const root=atom(value);
  for(let index=0;index<pending.length;index++){
   const object=pending[index],prototype=Object.getPrototypeOf(object);
   const kind=Array.isArray(object)?'array':prototype===Date.prototype?'date':prototype===Map.prototype?'map':prototype===Set.prototype?'set':prototype===Object.prototype?'object':null;
   if(!kind)throw Error('Unexpected first-method prototype');
   const properties=Object.getOwnPropertyNames(object).map(key=>{
    const descriptor=Object.getOwnPropertyDescriptor(object,key);
    if(!Object.hasOwn(descriptor,'value'))throw Error('Unexpected first-method accessor');
    return [key,descriptor.enumerable,descriptor.writable,descriptor.configurable,atom(descriptor.value)];
   });
   nodes.push({kind,properties,...(kind==='date'?{milliseconds:atom(Date.prototype.getTime.call(object))}:{}),
    ...(kind==='map'?{entries:[...Map.prototype.entries.call(object)].map(([key,value])=>[atom(key),atom(value)])}:{}),
    ...(kind==='set'?{values:[...Set.prototype.values.call(object)].map(atom)}:{})});
  }
  return JSON.stringify({root,nodes});
 };
 const scenario=(name,stage,verify=false)=>{
  const renderer=runtimes[name];
  if(stage==='render-source')return renderer.render(workload.source,options);
  const model=renderer.parse(workload.source,options);
  try{
   // verify=true is used only by the untimed before/after correctness passes.
   if(verify&&probeBackend&&name!=='js')backendChecks.push({profile,stage,...probeBackend(name,model,workload.family,'fresh-first-use')});
   let extension,firstCallSignature,renderOptions=options,callbackCalls=0;
   if(stage==='parse-first-edited-render'){
    extension={label:'host extension',self:null};extension.self=extension;model.benchmarkExtension=extension;
   }
   if(stage==='parse-first-callback'){
    if(workload.family==='er'){
     const underlying=options.measurer;
     renderOptions={...options,measurer:{width(...argumentsValue){
      callbackCalls++;
      if(!extension){extension={label:'callback extension',self:null};extension.self=extension;model.benchmarkExtension=extension;}
      return underlying.width(...argumentsValue);
     }}};
    }else{
     const original=model.getTasks;
     model.getTasks=function(now){const tasks=original.call(this,now);tasks[0].task='First callback task';return tasks;};
    }
   }
   if(stage==='parse-first-invoke'){
    if(workload.family==='er'){
     const [key,saved]=model.entities.entries().next().value;
     const entity=model.addEntity(key);
     if(verify&&(entity!==saved||entity!==model.entities.get(key)))throw Error(name+': first method lost entity identity');
     if(!verify)return entity.attributes.length;
     firstCallSignature=taskSignature(entity);
    }else{
     const tasks=model.getTasks(options.now);
     if(verify&&tasks!==model.tasks)throw Error(name+': first method lost task-array identity');
     if(!verify)return tasks.length;
     firstCallSignature=taskSignature(tasks);
    }
   }
   const drawing=renderer.render(model,renderOptions);
   if(verify&&stage==='parse-first-callback'&&workload.family==='er'){
    if(!callbackCalls||!extension)throw Error(name+': first measuring callback did not run');
    firstCallSignature=taskSignature(extension);
   }
   if(verify&&extension&&(model.benchmarkExtension!==extension||extension.self!==extension))throw Error(name+': first edited render lost host-object identity');
   return firstCallSignature===undefined?drawing:{...drawing,firstCallSignature};
  }finally{if(name!=='js')renderer.dispose(model);}
 };
 const verify=()=>{
  const verified=[];
  for(const stage of applicable){
   const expected=scenario('js',stage,true);
   for(const name of ['control','candidate']){
    const actual=scenario(name,stage,true);
    if(actual.firstCallSignature!==expected.firstCallSignature)throw Error(workload.id+'/'+stage+'/'+name+': first method values/aliases differ');
    if(actual.svg!==expected.svg||!Object.is(actual.width,expected.width)||!Object.is(actual.height,expected.height))throw Error(workload.id+'/'+stage+'/'+name+': whole output mismatch');
   }
   verified.push({stage,svg:expected.svg,width:expected.width,height:expected.height,firstCallSignature:expected.firstCallSignature});
  }
  return verified;
 };
 const baseline=verify();
 for(const stage of applicable){
 const work=Object.fromEntries(names.map(name=>[name,()=>{const value=scenario(name,stage);checksum+=typeof value==='number'?value:value.svg.length+value.width+value.height;}]));
 const batch=(name,count)=>{const start=performance.now();for(let i=0;i<count;i++)work[name]();return performance.now()-start;};
 const counts={},warmups={},samples=Object.fromEntries(names.map(name=>[name,[]]));
 for(const name of names){let operations=0,start=performance.now();do{work[name]();operations++;}while(operations<6||performance.now()-start<warmupMs);warmups[name]={operations,elapsedMs:performance.now()-start};let count=1;while(count<128&&quantile(Array.from({length:3},()=>batch(name,count)),.5)<12)count*=2;counts[name]=count;}
 counts.control=counts.candidate=Math.min(counts.control,counts.candidate);
 for(let trial=0;trial<trials;trial++){let order=names.slice(trial%3).concat(names.slice(0,trial%3));if(Math.floor(trial/3)%2)order.reverse();for(const name of order)samples[name].push(batch(name,counts[name])/counts[name]);}
 const medianMs=Object.fromEntries(names.map(name=>[name,quantile(samples[name],.5)]));
 rows.push({case:workload.id,profile,stage,medianMs,candidateOverControl:medianMs.candidate/medianMs.control,candidateOverJS:medianMs.candidate/medianMs.js,controlOverJS:medianMs.control/medianMs.js,counts,warmups,samples,p95BatchMeanMs:Object.fromEntries(names.map(name=>[name,quantile(samples[name],.95)]))});
 }
 const after=verify();
 for(let index=0;index<baseline.length;index++){
  const before=baseline[index],value=after[index];
  if(before.firstCallSignature!==value.firstCallSignature||before.stage!==value.stage||before.svg!==value.svg||!Object.is(before.width,value.width)||!Object.is(before.height,value.height))throw Error('First-use output changed after timing');
  const bytes=new TextEncoder().encode(value.svg),digest=await crypto.subtle.digest('SHA-256',bytes);
  const signature=value.firstCallSignature===undefined?null:new TextEncoder().encode(value.firstCallSignature);
  const signatureDigest=signature?await crypto.subtle.digest('SHA-256',signature):null;
  checks.push({profile,stage:value.stage,svgBytes:bytes.length,svgSha256:Array.from(new Uint8Array(digest),byte=>byte.toString(16).padStart(2,'0')).join(''),
   ...(signature?{firstCallSignatureBytes:signature.length,firstCallSignatureSha256:Array.from(new Uint8Array(signatureDigest),byte=>byte.toString(16).padStart(2,'0')).join('')}:{})});
 }
 }
 return{rows,checks,backendChecks,checksum};},{workload,stages,profiles,trials,warmupMs});
 report.rows.push(...result.rows);report.cases.push({id:workload.id,sourceSHA256:sha(Buffer.from(workload.source)),sourceBytes:Buffer.byteLength(workload.source),checks:result.checks,backendChecks:result.backendChecks,checksum:result.checksum});
 await mkdir(dirname(output),{recursive:true});await writeFile(output+'.partial',JSON.stringify(report,null,2)+'\n');console.log('compared '+workload.id);
 }
 assert.deepEqual(await inputs(),before,'Package changed during comparison');report.completedAt=new Date().toISOString();report.hostAfter={loadAverage:loadavg(),freeMemory:freemem()};report.succeeded=true;await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{clearTimeout(deadline);await browser?.close();await new Promise(r=>server.close(r));}
