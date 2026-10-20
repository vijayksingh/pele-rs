// Diagnostic attribution only. Never use CPU sampling as latency acceptance.
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
const roots={control:resolve(arg('--control','dist/package-web/control')),candidate:resolve(arg('--candidate','dist/package-web/candidate'))};
assert.notEqual(roots.control,roots.candidate,'Independent immutable package directories required');
const output=resolve(arg('--output','dist/evidence/native-pod-sankey-profile'));
const selected=arg('--cases','sankey/medium').split(','),stages=arg('--stages','parse,render-model,render-source').split(','),resources=arg('--profiles','fixed').split(',');
const duration=Number(arg('--duration-ms','2000')),warmup=Number(arg('--warmup-ms','300')),repeats=Number(arg('--repeats','1')),interval=Number(arg('--sampling-interval-us','250'));
assert(selected.length<=3&&new Set(selected).size===selected.length&&selected.every(id=>['sankey/small','sankey/medium','sankey/large'].includes(id)));
assert(stages.length&&new Set(stages).size===stages.length&&stages.every(stage=>['parse','render-model','render-source'].includes(stage)));
assert(resources.length&&new Set(resources).size===resources.length&&resources.every(profile=>['fixed','canvas'].includes(profile)));
assert(Number.isInteger(duration)&&duration>=500&&duration<=5000);assert(Number.isInteger(warmup)&&warmup>=200&&warmup<=1000);
assert(Number.isInteger(repeats)&&repeats>=1&&repeats<=3);assert(Number.isInteger(interval)&&interval>=100&&interval<=1000);
const plannedProfiles=selected.length*stages.length*resources.length*repeats*2;
assert(plannedProfiles*(duration+warmup)<=120000,'Bound diagnostic sampling and warmup to 120 seconds');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),oracle=resolve('node_modules/pele/dist/pele.js');
async function hashes(root,prefix=''){
 const out={};for(const entry of (await readdir(resolve(root,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(out,await hashes(root,name+'/'));else if(entry.isFile())out[name]=sha(await readFile(resolve(root,name)));else throw Error('Package contains a non-regular file');}return out;
}
const inputs=async()=>Object.fromEntries(await Promise.all(Object.entries(roots).map(async([name,root])=>[name,await hashes(root)])));
const before=await inputs(),oracleSHA256=sha(await readFile(oracle));
const allowedPackageDifferences=['native-storage.js'];
assert.deepEqual(Object.keys(before.control).sort(),Object.keys(before.candidate).sort(),'Package file sets must match');
const actualPackageDifferences=Object.keys(before.control).filter(path=>before.control[path]!==before.candidate[path]);
assert(actualPackageDifferences.every(path=>allowedPackageDifferences.includes(path)),'Only the native-storage read cursor may differ');
assert.equal(before.control['pkg/pele_bg.wasm'],before.candidate['pkg/pele_bg.wasm'],'WASM bytes must match');
assert(before.candidate['captured-owner.js']);assert(before.candidate['native-storage.js']);assert(before.candidate['pkg/pele_bg.wasm']);
assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const groupRules=[
 ['native-admission',/(captured_admission|captured_transport|admit_observed|admit_outbound|canonical_frame_bytes|validate_schema|validate_structure|preflight_encoding)/],
 ['native-flat-storage',/public::flat_store::/],
 ['native-flat-observation',/flat_store::observation::/],
 ['native-flat-write',/flat_store::write::|NativeSourceWriter/],
 ['native-flat-page-cow',/flat_store::pages::/],
 ['native-lease-directories',/native_canonical::(Directory|NativeCanonicalEpoch|Lease)/],
 ['native-owner-wrapper',/pele_wasm::native_canonical::/],
 ['sankey-import-projection',/sankey::public_model::(imported|imported_from|normalized_source|stored_field|root_field|optional_text|array|string|index)(?:\b|::)/],
 ['sankey-raw-parser',/sankey::parser|sankey::Sankey as .*DiagramModule>::parse/],
 ['sankey-render-layout',/sankey::(render|layout|measure)/],
 ['initial-native-construction',/sankey::public_model::(initial_into|initial_graph)|NativeSourceWriter/],
 ['native-svg-and-json',/(render_numbers|serde_json|compat::svg|svg_document|escape_xml)/],
 ['js-native-storage-view-or-emitter',/\/native-storage\.js/],
 ['js-observation-emitter',/(?:#record|#flush|#strings|#atom|beginArguments|argument|beginNode|beginItems|mapEntry|setItem|finishNode|finish|abort|property|root) .*\/native-storage\.js/],
 ['shared-publication',/\breconcileGraph\b/],
 ['shared-observation-and-match',/\b(snapshotGraph|matchesGraph)\b/],
 ['owner-transport-gates',/\b(capturedObservedFrame|outcomePacket|publication_started|exactEquals)\b|\/captured-owner\.js/],
 ['legacy-binary',/\b(encodeGraphBinary|decodeGraphBinary)\b|encode_binary_exchange|snapshot_binary/],
 ['allocator',/\b(malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc/],
];
// Ordered leaf-only roles form a disjoint partition. Inclusive groups below do
// not: they are each de-duplicated per sample and must never be added together.
const roleRules=[
 ['admission',/(captured_admission|captured_transport|admit_observed|admit_outbound|canonical_frame_bytes|validate_schema|validate_structure|preflight_encoding)/],
 ['allocator',/\b(malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc/],
 ['flat-native-storage',/public::flat_store::/],
 ['native-wrapper-or-leases',/pele_wasm::native_canonical::/],
 ['native-parse',/sankey::parser|sankey::Sankey as .*DiagramModule>::parse/],
 ['native-projection',/sankey::public_model::/],
 ['native-render-layout',/sankey::(render|layout|measure)/],
 ['native-svg-json',/(render_numbers|serde_json|compat::svg|svg_document|escape_xml)/],
 ['js-storage-adapter',/\/native-storage\.js/],
 ['js-observer-publisher',/\/graph-storage\.js/],
 ['js-owner-transport',/\/(captured-owner|full)\.js/],
 ['legacy-graph-boundary',/\/(graph|graph-binary)\.js|encode_binary_exchange|snapshot_binary/],
];
const unionRules=[
 ['native-admission-or-flat-or-wrapper',['native-admission','native-flat-storage','native-owner-wrapper']],
 ['all-canonical-storage-boundary',['native-admission','native-flat-storage','native-owner-wrapper','js-native-storage-view-or-emitter','shared-publication','shared-observation-and-match','owner-transport-gates']],
 ['sankey-parse-projection-layout',['sankey-raw-parser','sankey-import-projection','sankey-render-layout']],
];
const normalizedFrameText=frame=>frame.functionName.replace(/\[[0-9a-f]+\]/g,'')+' '+frame.url;
const languageFor=frame=>frame.functionName==='(garbage collector)'?'gc':(frame.url.startsWith('wasm://')||/\.wasm(?:$|[?#])/.test(frame.url))?'wasm':/^https?:/.test(frame.url)?'javascript':'engine-or-profiler';
function summarize(profile){
 const nodes=new Map(profile.nodes.map(node=>[node.id,node])),parents=new Map();
 for(const node of profile.nodes)for(const child of node.children??[]){assert(!parents.has(child),'CPU profile is not a tree');parents.set(child,node.id);}
 assert(profile.samples?.length&&profile.timeDeltas?.length===profile.samples.length,'Weighted CPU samples unavailable');
 const self=new Map(),inclusive=new Map(),frames=new Map(),categories=new Map(),roles=new Map(),unions=new Map(unionRules.map(([name])=>[name,0])),groups=new Map(groupRules.map(([name])=>[name,{selfUs:0,inclusiveUs:0}]));let totalUs=0;
 const keyFor=frame=>JSON.stringify([frame.functionName,frame.url,frame.lineNumber,frame.columnNumber,frame.scriptId]);
 for(let i=0;i<profile.samples.length;i++){
  const weight=profile.timeDeltas[i];assert(Number.isFinite(weight)&&weight>=0);totalUs+=weight;
  const leaf=nodes.get(profile.samples[i]);assert(leaf);const frame=leaf.callFrame,key=keyFor(frame);frames.set(key,frame);self.set(key,(self.get(key)??0)+weight);
  const category=languageFor(frame);categories.set(category,(categories.get(category)??0)+weight);
  const role=category==='gc'?'gc':(roleRules.find(([,pattern])=>pattern.test(normalizedFrameText(frame)))?.[0]??category+'-other');
  roles.set(role,(roles.get(role)??0)+weight);
  const seenFrames=new Set(),seenGroups=new Set(),seenNodes=new Set();let node=leaf;
  while(node){
   assert(!seenNodes.has(node.id),'CPU profile parent cycle');seenNodes.add(node.id);
   const f=node.callFrame,k=keyFor(f);frames.set(k,f);if(!seenFrames.has(k)){inclusive.set(k,(inclusive.get(k)??0)+weight);seenFrames.add(k);}
   const text=normalizedFrameText(f);
   for(const[name,pattern]of groupRules)if(pattern.test(text)){if(node===leaf)groups.get(name).selfUs+=weight;seenGroups.add(name);}
   const parent=parents.get(node.id);node=parent===undefined?undefined:nodes.get(parent);
  }
  for(const name of seenGroups)groups.get(name).inclusiveUs+=weight;
  for(const[name,members]of unionRules)if(members.some(member=>seenGroups.has(member)))unions.set(name,unions.get(name)+weight);
 }
 assert(totalUs>0);
 assert.equal([...categories.values()].reduce((a,b)=>a+b,0),totalUs);assert.equal([...roles.values()].reduce((a,b)=>a+b,0),totalUs);
 const rank=table=>[...table].sort((a,b)=>b[1]-a[1]).slice(0,60).map(([key,us])=>({frame:frames.get(key),weightedUs:us,percent:100*us/totalUs}));
 return {sampleCount:profile.samples.length,weightedUs:totalUs,profileSpanUs:profile.endTime-profile.startTime,selfCategories:Object.fromEntries([...categories].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),disjointSelfRoles:Object.fromEntries([...roles].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),deduplicatedAncestorUnions:Object.fromEntries([...unions].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs,members:unionRules.find(([key])=>key===name)[1]}])),topSelf:rank(self),topInclusive:rank(inclusive),architectureGroups:Object.fromEntries([...groups].map(([name,g])=>[name,{...g,selfPercent:100*g.selfUs/totalUs,inclusivePercent:100*g.inclusiveUs/totalUs}])),groupPatterns:Object.fromEntries(groupRules.map(([name,pattern])=>[name,pattern.source])),phaseCallCounts:null,phaseCallCountsReason:'CPU samples are not invocation counts. No production instrumentation or monkey-patching is installed.'};
}
const report={schemaVersion:1,diagnostic:true,startedAt:new Date().toISOString(),oracleRevision:revision,oracleSHA256,runtimeHashes:before,wasmSHA256:before.candidate['pkg/pele_bg.wasm'],ownerModuleSHA256:before.candidate['captured-owner.js'],moduleSHA256:Object.fromEntries(['control','candidate'].map(name=>[name,Object.fromEntries(['captured-owner.js','native-storage.js','graph-storage.js','full.js','pkg/pele.js'].map(path=>[path,before[name][path]]))])),harnessSHA256:sha(await readFile(fileURLToPath(import.meta.url))),casesSHA256:sha(await readFile(new URL('./cases.mjs',import.meta.url))),settings:{selected,stages,resources,durationMs:duration,warmupMs:warmup,repeats,samplingIntervalUs:interval,plannedProfiles,identicalPackageBytes:actualPackageDifferences.length===0,allowedPackageDifferences,actualPackageDifferences,identicalWasmBytes:true,candidateCapturedSankey:true,controlCapturedSankey:false,candidateNativeCanonicalSankey:true,controlNativeCanonicalSankey:false,probeInsideProfile:false,forcedGC:false},host:{platform:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,loadBefore:loadavg(),freeMemoryBefore:freemem()},scope:'Sample attribution only. Same page and process share heaps/GC; independent package modules with an explicitly allowed native-storage.js difference and identical WASM bytes. Leaf language and role categories are disjoint. Inclusive groups overlap and are not additive; union categories de-duplicate ancestors. HTTP .wasm frames count as WASM even when V8 uses the other package URL for shared compiled code. Rust crate hashes are normalized for symbol grouping; inlining and regex coverage still limit attribution. Samples do not prove allocation counts or call frequency. Profiler throughput is not the latency acceptance benchmark.',profiles:[],checks:[]};
const server=createServer(async(req,res)=>{try{
 res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
 const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Native canonical Sankey CPU diagnosis</title>');return;}
 let file;if(path==='/oracle.js')file=oracle;else{const name=Object.keys(roots).find(name=>path.startsWith('/'+name+'/'));assert(name);file=resolve(roots[name],'.'+path.slice(name.length+1));assert(file.startsWith(roots[name]+'/'));}
 res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
 }catch{res.writeHead(404);res.end();}});
await mkdir(output,{recursive:true});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));let browser;
const deadline=setTimeout(()=>void browser?.close(),240000);
try{
 browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 await page.evaluate(async now=>{
  Date.now=()=>now;const oracle=await import('/oracle.js'),controlAPI=await import('/control/index.js'),candidateAPI=await import('/candidate/index.js'),controlOwner=await import('/control/captured-owner.js'),candidateOwner=await import('/candidate/captured-owner.js');
  if(typeof candidateOwner.capturedSankeyStorage!=='function'||typeof controlOwner.capturedSankeyStorage!=='function')throw Error('Native storage probes unavailable');
  controlOwner.enableCapturedSankeyForTesting(false);const control=await controlAPI.loadRenderer();candidateOwner.enableCapturedSankeyForTesting(true);const candidate=await candidateAPI.loadRenderer();
  window.capturedProfile={oracle,controlAPI,runtimes:{control,candidate},owners:{control:controlOwner,candidate:candidateOwner}};
 },now);
 const cdp=await page.context().newCDPSession(page);await cdp.send('Profiler.enable');await cdp.send('Profiler.setSamplingInterval',{interval});await cdp.send('Performance.enable');
 let condition=0;
 for(const workload of cases().filter(c=>selected.includes(c.id)))for(const resource of resources)for(const stage of stages){
  await page.evaluate(({workload,resource,stage})=>{
   const {oracle,controlAPI,runtimes,owners}=window.capturedProfile;
   const measurer=resource==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:controlAPI.canvasMeasurer('sans-serif','monospace');if(!measurer)throw Error('Canvas unavailable');
   const options={...workload.options,measurer},models=Object.fromEntries(Object.entries(runtimes).map(([name,renderer])=>[name,renderer.parse(workload.source,options)])),expected=oracle.render(workload.source,options);
   const probe=(name,model,lane)=>{const actual=owners[name].capturedSankeyBackend(model),expected=name==='candidate'?'source':undefined,storage=owners[name].capturedSankeyStorage(model),expectedStorage=name==='candidate'?'native':undefined;if(actual!==expected||storage!==expectedStorage)throw Error(name+'/'+lane+': backend '+String(actual)+', storage '+String(storage));return{runtime:name,lane,expected:expected??null,actual:actual??null,expectedStorage:expectedStorage??null,storage:storage??null};};
   const equal=(value,name)=>{if(value.svg!==expected.svg||!Object.is(value.width,expected.width)||!Object.is(value.height,expected.height))throw Error(workload.id+'/'+name+': output mismatch');};
   window.profileWork={
    run(name,ms){const renderer=runtimes[name],start=performance.now();let operations=0,checksum=0;do{if(stage==='parse'){const model=renderer.parse(workload.source,options);try{checksum+=model.type.length;}finally{renderer.dispose(model);}}else{const value=renderer.render(stage==='render-source'?workload.source:models[name],options);checksum+=value.svg.length+value.width+value.height;}operations++;}while(performance.now()-start<ms);return{operations,checksum,elapsedMs:performance.now()-start};},
    verify(){const backends=[];for(const[name,renderer]of Object.entries(runtimes)){backends.push(probe(name,models[name],'retained'));equal(renderer.render(models[name],options),name+'/retained');equal(renderer.render(workload.source,options),name+'/source');const fresh=renderer.parse(workload.source,options);try{backends.push(probe(name,fresh,'fresh'));equal(renderer.render(fresh,options),name+'/fresh');}finally{renderer.dispose(fresh);}}return{backends,svg:expected.svg,width:expected.width,height:expected.height};},
    close(){for(const[name,renderer]of Object.entries(runtimes))renderer.dispose(models[name]);}
   };
  },{workload,resource,stage});
  const checks=[];
  try{
   const initial=await page.evaluate(()=>window.profileWork.verify());checks.push({phase:'before',...initial});
   for(let repeat=0;repeat<repeats;repeat++){
    const order=(condition+repeat)%2?['candidate','control']:['control','candidate'];
    for(const runtime of order){
     const warmupResult=await page.evaluate(({runtime,warmup})=>window.profileWork.run(runtime,warmup),{runtime,warmup});
     const heapBefore=(await cdp.send('Performance.getMetrics')).metrics.filter(m=>['JSHeapUsedSize','JSHeapTotalSize'].includes(m.name));
     let exercised,profile;await cdp.send('Profiler.start');
     try{exercised=await page.evaluate(({runtime,duration})=>window.profileWork.run(runtime,duration),{runtime,duration});}
     finally{({profile}=await cdp.send('Profiler.stop'));}
     const heapAfter=(await cdp.send('Performance.getMetrics')).metrics.filter(m=>['JSHeapUsedSize','JSHeapTotalSize'].includes(m.name));
     const label=`${workload.id.replace('/','-')}-${resource}-${stage}-${runtime}-${repeat+1}`,raw=JSON.stringify(profile);
     await writeFile(resolve(output,label+'.cpuprofile'),raw);
     report.profiles.push({case:workload.id,resource,stage,runtime,repeat:repeat+1,order,label,profileSHA256:sha(raw),sourceSHA256:sha(workload.source),sourceBytes:Buffer.byteLength(workload.source),warmup:warmupResult,...exercised,heapBefore,heapAfter,...summarize(profile)});
     await writeFile(resolve(output,'summary.partial.json'),JSON.stringify(report,null,2)+'\n');console.log(label);
    }
   }
   const final=await page.evaluate(()=>window.profileWork.verify());assert.equal(final.svg,initial.svg);assert(Object.is(final.width,initial.width)&&Object.is(final.height,initial.height));checks.push({phase:'after',...final});
   for(const check of checks){const{svg,...rest}=check;report.checks.push({case:workload.id,resource,stage,...rest,svgSHA256:sha(svg),svgBytes:Buffer.byteLength(svg)});}
  }finally{await page.evaluate(()=>{window.profileWork.close();delete window.profileWork;});}
  condition++;
 }
 assert.deepEqual(await inputs(),before,'Package changed during profiling');report.completedAt=new Date().toISOString();report.host.loadAfter=loadavg();report.host.freeMemoryAfter=freemem();report.succeeded=true;
 await writeFile(resolve(output,'summary.json'),JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{clearTimeout(deadline);await browser?.close();await new Promise(resolve=>server.close(resolve));}
