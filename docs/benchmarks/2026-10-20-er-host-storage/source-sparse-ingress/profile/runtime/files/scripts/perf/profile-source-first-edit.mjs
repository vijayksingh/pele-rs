// Offline-prepared diagnostic. Run only after coordinator GO; no latency verdict.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir,readdir} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {cpus,platform,arch,loadavg,freemem} from 'node:os';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {cases,revision,now} from './cases.mjs';
const arg=(key,fallback)=>{const i=process.argv.indexOf(key);return i<0?fallback:process.argv[i+1];};
assert(process.argv.includes('--execute-approved'),'Explicit coordinator GO required: --execute-approved');
const roots={control:resolve(arg('--control','dist/perf-patches/parse-architecture/svg-append-runtime/candidate-package')),candidate:resolve(arg('--candidate','dist/perf-patches/parse-architecture/source-sparse-ingress-runtime/candidate-package'))};
assert.notEqual(roots.control,roots.candidate);
const output=resolve(arg('--output','dist/evidence/source-first-edit-profiles'));
const duration=2000,warmup=500,interval=250,stages=['parse-first-edited-render'];
const workload=cases().find(c=>c.id==='sankey/medium');assert(workload);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const oracle=resolve('node_modules/pele/dist/pele.js');
const harness=fileURLToPath(import.meta.url),caseFile=fileURLToPath(new URL('./cases.mjs',import.meta.url)),expectedFile=fileURLToPath(new URL('./source-canonical-inputs.json',import.meta.url));
const expected=JSON.parse(await readFile(expectedFile,'utf8'));
async function hashes(root,prefix=''){
 const out={};for(const entry of (await readdir(resolve(root,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(out,await hashes(root,name+'/'));else if(entry.isFile())out[name]=sha(await readFile(resolve(root,name)));else throw Error('Nonregular package file');}return out;
}
const inputs=async()=>({packages:Object.fromEntries(await Promise.all(Object.entries(roots).map(async([name,root])=>[name,await hashes(root)]))),oracle:sha(await readFile(oracle)),harness:sha(await readFile(harness)),cases:sha(await readFile(caseFile)),expected:sha(await readFile(expectedFile)),originalHarness:sha(await readFile(new URL('./compare-first-use-web-vm.mjs',import.meta.url)))});
const originalHarness=await readFile(new URL('./compare-first-use-web-vm.mjs',import.meta.url),'utf8');
assert.equal(sha(originalHarness),expected.originalHarnessSHA256);
const extracted=originalHarness.slice(expected.extraction.startByte,expected.extraction.endByte);assert.equal(sha(extracted),expected.extraction.sha256);assert((await readFile(harness,'utf8')).includes(extracted),'Scenario extraction drift');
const workExtracted=originalHarness.slice(expected.workExtraction.startByte,expected.workExtraction.endByte);assert.equal(sha(workExtracted),expected.workExtraction.sha256);assert((await readFile(harness,'utf8')).includes(workExtracted),'Operation closure drift');
const before=await inputs();assert.deepEqual(before.packages,expected.packages);assert.equal(before.oracle,expected.oracle);assert.equal(before.cases,expected.cases);
const groupRules=[
 ['native-construction',/sankey::source_store::SourceCanonical::(new|add_text|add_key)|sankey::public_model::initial_|public::graph::GraphWriter/],
 ['native-validation',/captured_admission|captured_transport|canonical_binary_size|validate_schema|validate_read|ModelGraph::validate/],
 ['native-recipe-encoding',/source_store::.*recipe|source_store::(group|expr|atom9)|source_layout::(prepare_graph|snapshot|outcome)/],
 ['native-logical-records',/sankey::source_store::|graph_read::/],
 ['native-batch-synthesis',/source_store.*::(properties|map_entries|property_at|base_property|base_header)(?:\b|::)/],
 ['native-canonical-encoding',/binary::(encode_|write_read_|Writer)|canonical_frame_bytes/],
 ['native-parser',/sankey::parser::|sankey::indexed::(?:parse|parse_bounded|parse_with|parse_source)(?:\b|::)/],
 ['native-source-ingress',/source_input|prepare_overlay|admit_observed|apply_source_input|validate_read_observed|DecoderBudget/],
 ['native-overlay-preflight',/SourceCanonical::prepare_overlay/],
 ['native-observed-admission',/captured_transport::admit_observed/],
 ['native-size-counter',/captured_transport::Counter::/],
 ['native-decoder-budget',/captured_transport::DecoderBudget::/],
 ['native-captured-logical',/source_input::.*CapturedLogical|source_input::.*::(?:own_fields|string_is)/],
 ['native-import-string',/sankey::public_model::.*string_owned|sankey::public_model::.*imported_from/],
 ['native-projection',/sankey::public_model::(imported|normalized_source|source_text_budget)/],
 ['native-render',/sankey::(render|layout|measure)|compat::svg/],
 ['native-json',/serde_json|render_numbers|render_packet/],
 ['js-recipe-decoder',/decodeSourceLayout|decodeLayoutOutcome|\/layout-packet\.js/],
 ['js-logical-storage',/\/layout-storage\.js/],
 ['js-publication',/\breconcileGraph\b/],
 ['js-observation-match',/\b(snapshotGraph|matchesGraph)\b/],
 ['js-binary',/\b(encodeGraphBinary|decodeGraphBinary)\b|\/graph-binary\.js/],
 ['allocator',/\b(malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc/],
 ['gc',/\(garbage collector\)/],
];
const roleRules=[
 ['allocator',groupRules.find(([n])=>n==='allocator')[1]],
 ['validation',groupRules.find(([n])=>n==='native-validation')[1]],
 ['source-ingress',groupRules.find(([n])=>n==='native-source-ingress')[1]],
 ['construction',groupRules.find(([n])=>n==='native-construction')[1]],
 ['recipe-encoding',groupRules.find(([n])=>n==='native-recipe-encoding')[1]],
 ['canonical-encoding',groupRules.find(([n])=>n==='native-canonical-encoding')[1]],
 ['logical-native-records',groupRules.find(([n])=>n==='native-logical-records')[1]],
 ['parser',groupRules.find(([n])=>n==='native-parser')[1]],
 ['projection',groupRules.find(([n])=>n==='native-projection')[1]],
 ['render',groupRules.find(([n])=>n==='native-render')[1]],
 ['json',groupRules.find(([n])=>n==='native-json')[1]],
 ['js-publication',groupRules.find(([n])=>n==='js-publication')[1]],
 ['js-observation-match',groupRules.find(([n])=>n==='js-observation-match')[1]],
 ['js-layout',/\/layout-(storage|packet)\.js/],
 ['js-graph',/\/(graph|graph-storage|graph-binary)\.js/],
 ['js-facade',/\/full\.js/],
];
const unionRules=[
 ['construction-validation-encoding',['native-construction','native-validation','native-recipe-encoding','native-canonical-encoding']],
 ['host-publication-observation',['js-publication','js-observation-match','js-logical-storage','js-recipe-decoder','js-binary']],
 ['graph-boundary-total',['native-construction','native-validation','native-recipe-encoding','native-canonical-encoding','js-publication','js-observation-match','js-logical-storage','js-recipe-decoder','js-binary']],
 ['gc',['gc']],
];
const normalizedFrameText=frame=>frame.functionName.replace(/\[[0-9a-f]+\]/g,'')+' '+frame.url;
const languageFor=frame=>frame.functionName==='(garbage collector)'?'gc':(frame.url.startsWith('wasm://')||/\.wasm(?:$|[?#])/.test(frame.url))?'wasm':/^https?:/.test(frame.url)?'javascript':'engine-or-profiler';
function summarize(profile){
 const nodes=new Map(profile.nodes.map(node=>[node.id,node])),parents=new Map();
 for(const node of profile.nodes)for(const child of node.children??[]){assert(!parents.has(child),'CPU profile is not a tree');parents.set(child,node.id);}
 assert(profile.samples?.length&&profile.timeDeltas?.length===profile.samples.length,'Weighted CPU samples unavailable');
 const ancestorLanguages=new Map(),self=new Map(),inclusive=new Map(),frames=new Map(),categories=new Map(),roles=new Map(),unions=new Map(unionRules.map(([name])=>[name,0])),groups=new Map(groupRules.map(([name])=>[name,{selfUs:0,inclusiveUs:0}]));let totalUs=0;
 const keyFor=frame=>JSON.stringify([frame.functionName,frame.url,frame.lineNumber,frame.columnNumber,frame.scriptId]);
 for(let i=0;i<profile.samples.length;i++){
  const weight=profile.timeDeltas[i];assert(Number.isFinite(weight)&&weight>=0);totalUs+=weight;
  const leaf=nodes.get(profile.samples[i]);assert(leaf);const frame=leaf.callFrame,key=keyFor(frame);frames.set(key,frame);self.set(key,(self.get(key)??0)+weight);
  const category=languageFor(frame);categories.set(category,(categories.get(category)??0)+weight);
  const role=category==='gc'?'gc':(roleRules.find(([,pattern])=>pattern.test(normalizedFrameText(frame)))?.[0]??category+'-other');
  roles.set(role,(roles.get(role)??0)+weight);
  const seenFrames=new Set(),seenGroups=new Set(),seenNodes=new Set(),seenLanguages=new Set();let node=leaf;
  while(node){
   assert(!seenNodes.has(node.id),'CPU profile parent cycle');seenNodes.add(node.id);
   const f=node.callFrame,k=keyFor(f);seenLanguages.add(languageFor(f));frames.set(k,f);if(!seenFrames.has(k)){inclusive.set(k,(inclusive.get(k)??0)+weight);seenFrames.add(k);}
   const text=normalizedFrameText(f);
   for(const[name,pattern]of groupRules)if(pattern.test(text)){if(node===leaf)groups.get(name).selfUs+=weight;seenGroups.add(name);}
   const parent=parents.get(node.id);node=parent===undefined?undefined:nodes.get(parent);
  }
  for(const name of seenLanguages)ancestorLanguages.set(name,(ancestorLanguages.get(name)??0)+weight);
  for(const name of seenGroups)groups.get(name).inclusiveUs+=weight;
  for(const[name,members]of unionRules)if(members.some(member=>seenGroups.has(member)))unions.set(name,unions.get(name)+weight);
 }
 assert(totalUs>0);
 assert.equal([...categories.values()].reduce((a,b)=>a+b,0),totalUs);assert.equal([...roles.values()].reduce((a,b)=>a+b,0),totalUs);
 const rank=table=>[...table].sort((a,b)=>b[1]-a[1]).slice(0,60).map(([key,us])=>({frame:frames.get(key),weightedUs:us,percent:100*us/totalUs}));
 return {deduplicatedAncestorLanguages:Object.fromEntries([...ancestorLanguages].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),sampleCount:profile.samples.length,weightedUs:totalUs,profileSpanUs:profile.endTime-profile.startTime,selfCategories:Object.fromEntries([...categories].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),disjointSelfRoles:Object.fromEntries([...roles].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),deduplicatedAncestorUnions:Object.fromEntries([...unions].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs,members:unionRules.find(([key])=>key===name)[1]}])),topSelf:rank(self),topInclusive:rank(inclusive),architectureGroups:Object.fromEntries([...groups].map(([name,g])=>[name,{...g,selfPercent:100*g.selfUs/totalUs,inclusivePercent:100*g.inclusiveUs/totalUs}])),groupPatterns:Object.fromEntries(groupRules.map(([name,pattern])=>[name,pattern.source])),phaseCallCounts:null,phaseCallCountsReason:'CPU samples are not invocation counts. No production instrumentation or monkey-patching is installed.'};
}
const report={schemaVersion:1,diagnostic:true,succeeded:false,startedAt:new Date().toISOString(),oracleRevision:revision,sourcePins:expected.sourcePins,scenarioExtraction:expected.extraction,originalHarnessSHA256:expected.originalHarnessSHA256,inputsBefore:before,settings:{durationMs:duration,warmupMs:warmup,samplingIntervalUs:interval,case:workload.id,resource:'fixed',stages,profiles:3,ordinaryDefaultAPI:true,capabilityProbeOutsideProfile:true,forcedGC:false},host:{platform:platform(),arch:arch(),cpu:cpus()[0]?.model,loadBefore:loadavg(),freeMemoryBefore:freemem()},profiles:[],checks:[],scope:'Sample attribution only, not latency acceptance. One browser/page shares heaps. Leaf language/role partitions are disjoint; ancestor groups overlap, unions deduplicate each sample. HTTP .wasm URLs are WASM. Regex/inlining coverage limits remain; raw native names and all samples retained.'};
const server=createServer(async(req,res)=>{try{
 res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
 const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Source canonical profile</title>');return;}
 let file;if(path==='/oracle.js')file=oracle;else{const name=Object.keys(roots).find(name=>path.startsWith('/'+name+'/'));assert(name);file=resolve(roots[name],'.'+path.slice(name.length+1));assert(file.startsWith(roots[name]+'/'));}
 res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
 }catch{res.writeHead(404);res.end();}});
await mkdir(output,{recursive:true});await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser,page;
const deadline=setTimeout(()=>void browser?.close(),180000);
try{
 browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 await page.evaluate(async({now,workload})=>{
  Date.now=()=>now;const js=await import('/oracle.js'),controlAPI=await import('/control/index.js'),candidateAPI=await import('/candidate/index.js');
  const control=await controlAPI.loadRenderer(),candidate=await candidateAPI.loadRenderer();
  const runtimes={js,control,candidate},probeBackend=undefined,backendChecks=[],profile='fixed',applicable=['parse-first-edited-render'];
  const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:controlAPI.canvasMeasurer('sans-serif','monospace');if(!measurer)throw Error('Canvas unavailable');
  const options={...workload.options,measurer};
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
 const names=['js','control','candidate'],stage='parse-first-edited-render';let checksum=0;
 const work=Object.fromEntries(names.map(name=>[name,()=>{const value=scenario(name,stage);checksum+=typeof value==='number'?value:value.svg.length+value.width+value.height;}]));
  window.profileWork={
   run(name,stage,ms){const initialChecksum=checksum,start=performance.now();let operations=0;do{work[name]();operations++;}while(performance.now()-start<ms);return{operations,checksum:checksum-initialChecksum,elapsedMs:performance.now()-start};},
   verify,
   close(){for(const name of ['control','candidate'])runtimes[name].close();}
  };
 },{now,workload});
 const cdp=await page.context().newCDPSession(page);await cdp.send('Profiler.enable');await cdp.send('Profiler.setSamplingInterval',{interval});
 for(let condition=0;condition<stages.length;condition++){
  const stage=stages[condition],initial=await page.evaluate(()=>window.profileWork.verify());report.checks.push({stage,phase:'before',...initial});
  const order=[['js','control','candidate'],['control','candidate','js'],['candidate','js','control']][condition];
  for(const runtime of order){
   const warmed=await page.evaluate(({runtime,stage,warmup})=>window.profileWork.run(runtime,stage,warmup),{runtime,stage,warmup});
   let exercised,profile;await cdp.send('Profiler.start');
   try{exercised=await page.evaluate(({runtime,stage,duration})=>window.profileWork.run(runtime,stage,duration),{runtime,stage,duration});}finally{({profile}=await cdp.send('Profiler.stop'));}
   const label=`sankey-medium-fixed-${stage}-${runtime}`,raw=JSON.stringify(profile);await writeFile(resolve(output,label+'.cpuprofile'),raw);
   report.profiles.push({label,case:workload.id,stage,runtime,order,profileSHA256:sha(raw),sourceSHA256:sha(workload.source),warmup:warmed,...exercised,...summarize(profile)});
   await writeFile(resolve(output,'summary.partial.json'),JSON.stringify(report,null,2)+'\n');console.log(label);
  }
  const final=await page.evaluate(()=>window.profileWork.verify());assert.deepEqual(final,initial,'First-edited whole output changed after profiling');report.checks.push({stage,phase:'after',...final});
 }
 report.succeeded=true;
}catch(error){report.error=String(error?.stack??error);throw error;}
finally{
 clearTimeout(deadline);
 try{if(page&&!page.isClosed()){await page.evaluate(()=>{window.profileWork?.close();delete window.profileWork;});report.modelsDisposed=true;}}catch(error){report.cleanupError=String(error);report.succeeded=false;}
 try{await browser?.close();report.browserClosed=!browser||!browser.isConnected();}finally{await new Promise(r=>server.close(r));}
 report.inputsAfter=await inputs();report.completedAt=new Date().toISOString();report.host.loadAfter=loadavg();report.host.freeMemoryAfter=freemem();
 try{assert.deepEqual(report.inputsAfter,before,'Frozen profiling inputs changed');report.inputsUnchanged=true;}catch(error){report.succeeded=false;report.inputError=String(error);}
 await writeFile(resolve(output,'summary.json'),JSON.stringify(report,null,2)+'\n');
 if(!report.succeeded)process.exitCode=1;
}
