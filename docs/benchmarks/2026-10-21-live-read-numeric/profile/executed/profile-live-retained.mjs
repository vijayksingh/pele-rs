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
const roots={control:resolve(arg('--control')),candidate:resolve(arg('--candidate'))};
const output=resolve(arg('--output'));const duration=5000,warmup=500,interval=250;
const workloads=cases().filter(c=>['sankey/medium','sankey/large'].includes(c.id));assert.equal(workloads.length,2);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const oracle=resolve('node_modules/pele/dist/pele.js');const harness=fileURLToPath(import.meta.url),caseFile=fileURLToPath(new URL('./cases.mjs',import.meta.url)),expectedFile=fileURLToPath(new URL('./inputs.json',import.meta.url));
const expected=JSON.parse(await readFile(expectedFile,'utf8'));
async function hashes(root,prefix=''){const out={};for(const e of(await readdir(resolve(root,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const n=prefix+e.name;if(e.isDirectory())Object.assign(out,await hashes(root,n+'/'));else{assert(e.isFile());out[n]=sha(await readFile(resolve(root,n)));}}return out;}
const inputs=async()=>({packages:Object.fromEntries(await Promise.all(Object.entries(roots).map(async([n,r])=>[n,await hashes(r)]))),oracle:sha(await readFile(oracle)),harness:sha(await readFile(harness)),cases:sha(await readFile(caseFile)),expected:sha(await readFile(expectedFile))});
const before=await inputs();assert.deepEqual(before.packages,expected.packages);assert.equal(before.oracle,expected.oracle);assert.equal(before.cases,expected.cases);
const groupRules=[
 ['js-live-read',/\/live-render\.js/],
 ['js-prepare-input',/prepareMutableInput|prepareInput|snapshotGraph|matchesGraph/],
 ['native-live-phase',/sankey::live_phase|sankey::render::(HostPhase|Phase)/],
 ['native-detached',/detached_render/],
 ['native-construction',/sankey::source_store::SourceCanonical::(new|add_text|add_key)|sankey::public_model::initial_|public::graph::GraphWriter/],
 ['native-validation',/captured_admission|captured_transport|canonical_binary_size|validate_schema|validate_read|ModelGraph::validate/],
 ['native-recipe-encoding',/source_store::.*recipe|source_store::(group|expr|atom9)|source_layout::(prepare_graph|snapshot|outcome)/],
 ['native-logical-records',/sankey::source_store::|graph_read::/],
 ['native-batch-synthesis',/source_store.*::(properties|map_entries|property_at|base_property|base_header)(?:\b|::)/],
 ['native-canonical-encoding',/binary::(encode_|write_read_|Writer)|canonical_frame_bytes/],
 ['native-parser',/sankey::parser|sankey::indexed::/],
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
 ['js-live-read',/\/live-render\.js/],
 ['js-prepare-input',/prepareMutableInput|prepareInput|snapshotGraph|matchesGraph/],
 ['native-live-phase',/sankey::live_phase/],
 ['native-detached',/detached_render/],
 ['allocator',groupRules.find(([n])=>n==='allocator')[1]],
 ['validation',groupRules.find(([n])=>n==='native-validation')[1]],
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
 ['live-read-and-phase',['js-live-read','native-live-phase']],
 ['observation-and-prepare',['js-prepare-input','js-observation-match']],
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
const report={schemaVersion:1,succeeded:false,sourcePins:expected.sourcePins,inputsBefore:before,settings:{durationMs:duration,warmupMs:warmup,samplingIntervalUs:interval,stage:'render-model',resource:'fixed',profiles:4,parseOutsideProfile:true,prototypeInterception:false},startedAt:new Date().toISOString(),host:{cpu:cpus()[0]?.model,platform:platform(),arch:arch(),loadBefore:loadavg()},profiles:[],checks:[]};
const server=createServer(async(req,res)=>{try{res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(path==='/'){res.end('<!doctype html><title>Retained Sankey profile</title>');return;}let file;if(path==='/oracle.js')file=oracle;else{const n=Object.keys(roots).find(n=>path.startsWith('/'+n+'/'));assert(n);file=resolve(roots[n],'.'+path.slice(n.length+1));assert(file.startsWith(roots[n]+'/'));}res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));}catch{res.writeHead(404);res.end();}});
await mkdir(output,{recursive:true});await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser,page;const deadline=setTimeout(()=>void browser?.close(),150000);
try{
 browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 await page.evaluate(async({now,workloads})=>{Date.now=()=>now;const js=await import('/oracle.js');const control=await(await import('/control/index.js')).loadRenderer(),candidate=await(await import('/candidate/index.js')).loadRenderer();const runtimes={js,control,candidate};const entries=workloads.map(workload=>{const options={...workload.options,measurer:{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}};return{workload,options,models:Object.fromEntries(Object.entries(runtimes).map(([n,r])=>[n,r.parse(workload.source,options)]))};});
 window.work={run(name,index,ms){const e=entries[index],r=runtimes[name],start=performance.now();let operations=0,checksum=0;do{const value=r.render(e.models[name],e.options);checksum+=value.svg.length+value.width+value.height;operations++;}while(performance.now()-start<ms);return{operations,checksum,elapsedMs:performance.now()-start};},verify(index){const e=entries[index],results={};for(const[name,r]of Object.entries(runtimes)){const trace=[];const o={...e.options,measurer:{width:(text,size,style)=>{trace.push([text,size,style]);return text.length*size*((style&1)?.71:.6);}}};const result=r.render(e.models[name],o);results[name]={svg:result.svg,width:result.width,height:result.height,trace,callbackCalls:trace.length};}for(const name of ['control','candidate']){const a=results[name],b=results.js;if(a.svg!==b.svg||!Object.is(a.width,b.width)||!Object.is(a.height,b.height)||JSON.stringify(a.trace)!==JSON.stringify(b.trace))throw Error(name+': public retained output/trace mismatch');}return results;},close(){for(const name of ['control','candidate']){for(const e of entries)runtimes[name].dispose(e.models[name]);runtimes[name].close();}}};
 },{now,workloads});
 const cdp=await page.context().newCDPSession(page);await cdp.send('Profiler.enable');await cdp.send('Profiler.setSamplingInterval',{interval});
 for(let index=0;index<workloads.length;index++){const initial=await page.evaluate(i=>window.work.verify(i),index);report.checks.push({case:workloads[index].id,phase:'before',results:initial});for(const runtime of (index===0?['control','candidate']:['candidate','control'])){const warmed=await page.evaluate(a=>window.work.run(a.runtime,a.index,a.ms),{runtime,index,ms:warmup});await cdp.send('Profiler.start');let exercised,profile;try{exercised=await page.evaluate(a=>window.work.run(a.runtime,a.index,a.ms),{runtime,index,ms:duration});}finally{({profile}=await cdp.send('Profiler.stop'));}const label=workloads[index].id.replace('/','-')+'-fixed-render-model-'+runtime;const raw=JSON.stringify(profile);await writeFile(resolve(output,label+'.cpuprofile'),raw);report.profiles.push({label,runtime,case:workloads[index].id,profileSHA256:sha(raw),sourceSHA256:sha(workloads[index].source),warmup:warmed,...exercised,...summarize(profile)});await writeFile(resolve(output,'summary.partial.json'),JSON.stringify(report,null,2)+'\n');console.log(label);}const final=await page.evaluate(i=>window.work.verify(i),index);assert.deepEqual(final,initial);report.checks.push({case:workloads[index].id,phase:'after',results:final});}
 report.succeeded=true;
}catch(error){report.error=String(error?.stack??error);throw error;}
finally{clearTimeout(deadline);try{if(page&&!page.isClosed()){await page.evaluate(()=>window.work?.close());report.modelsDisposed=true;}}catch(e){report.cleanupError=String(e);report.succeeded=false;}try{await browser?.close();report.browserClosed=!browser||!browser.isConnected();}finally{await new Promise(r=>server.close(r));}report.inputsAfter=await inputs();try{assert.deepEqual(report.inputsAfter,before);report.inputsUnchanged=true;}catch(e){report.inputError=String(e);report.succeeded=false;}report.completedAt=new Date().toISOString();await writeFile(resolve(output,'summary.json'),JSON.stringify(report,null,2)+'\n');if(!report.succeeded)process.exitCode=1;}
