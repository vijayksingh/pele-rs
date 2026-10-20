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
const output=resolve(arg('--output','dist/evidence/captured-sankey-profile'));
const selected=arg('--cases','sankey/medium').split(','),stages=arg('--stages','parse,render-model').split(','),resources=arg('--profiles','fixed').split(',');
const duration=Number(arg('--duration-ms','1500')),warmup=Number(arg('--warmup-ms','300')),repeats=Number(arg('--repeats','1')),interval=Number(arg('--sampling-interval-us','250'));
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
assert.deepEqual(before.control,before.candidate,'Use identical complete package bytes; selection must be the only runtime difference');
assert(before.candidate['captured-owner.js']);assert(before.candidate['pkg/pele_bg.wasm']);
assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const groupRules=[
 ['native-admission',/(captured_admission|captured_transport|admit_observed|admit_outbound|canonical_frame_bytes|validate_schema|validate_structure)/],
 ['captured-native-adapter',/pele_wasm::captured_store|CapturedStore/],
 ['sankey-import-projection',/sankey::public_model::(imported|imported_from|normalized_source|stored_field|root_field|optional_text|array|string|index)(?:\b|::)/],
 ['sankey-raw-parser',/sankey::parser/],
 ['initial-native-construction',/sankey::public_model::(initial_into|initial_graph)|host_construction::/],
 ['flat-columns-and-indexes',/\/captured-graph\.js/],
 ['flat-page-fork-copy-compact',/(?:#page|fork|compact|copy) .*\/captured-graph\.js/],
 ['captured-js-imports',/\/captured-store\.js/],
 ['construction-js',/\/(flat-constructor|constructor-storage)\.js/],
 ['shared-publication',/\breconcileGraph\b/],
 ['shared-observation-and-match',/\b(snapshotGraph|matchesGraph)\b/],
 ['owner-transport-gates',/\b(capturedObservedFrame|sameGraph|outcomePacket)\b/],
 ['legacy-binary',/\b(encodeGraphBinary|decodeGraphBinary)\b|encode_binary_exchange|snapshot_binary/],
 ['allocator',/\b(malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc/],
];
function summarize(profile){
 const nodes=new Map(profile.nodes.map(node=>[node.id,node])),parents=new Map();
 for(const node of profile.nodes)for(const child of node.children??[]){assert(!parents.has(child),'CPU profile is not a tree');parents.set(child,node.id);}
 assert(profile.samples?.length&&profile.timeDeltas?.length===profile.samples.length,'Weighted CPU samples unavailable');
 const self=new Map(),inclusive=new Map(),frames=new Map(),categories=new Map(),groups=new Map(groupRules.map(([name])=>[name,{selfUs:0,inclusiveUs:0}]));let totalUs=0;
 const keyFor=frame=>JSON.stringify([frame.functionName,frame.url,frame.lineNumber,frame.columnNumber,frame.scriptId]);
 for(let i=0;i<profile.samples.length;i++){
  const weight=profile.timeDeltas[i];assert(Number.isFinite(weight)&&weight>=0);totalUs+=weight;
  const leaf=nodes.get(profile.samples[i]);assert(leaf);const frame=leaf.callFrame,key=keyFor(frame);frames.set(key,frame);self.set(key,(self.get(key)??0)+weight);
  const category=frame.functionName==='(garbage collector)'?'gc':frame.url.startsWith('wasm://')?'wasm':/^https?:/.test(frame.url)?'javascript':'engine-or-profiler';categories.set(category,(categories.get(category)??0)+weight);
  const seenFrames=new Set(),seenGroups=new Set(),seenNodes=new Set();let node=leaf;
  while(node){
   assert(!seenNodes.has(node.id),'CPU profile parent cycle');seenNodes.add(node.id);
   const f=node.callFrame,k=keyFor(f);frames.set(k,f);if(!seenFrames.has(k)){inclusive.set(k,(inclusive.get(k)??0)+weight);seenFrames.add(k);}
   const text=f.functionName+' '+f.url;
   for(const[name,pattern]of groupRules)if(pattern.test(text)){if(node===leaf)groups.get(name).selfUs+=weight;seenGroups.add(name);}
   const parent=parents.get(node.id);node=parent===undefined?undefined:nodes.get(parent);
  }
  for(const name of seenGroups)groups.get(name).inclusiveUs+=weight;
 }
 assert(totalUs>0);
 const rank=table=>[...table].sort((a,b)=>b[1]-a[1]).slice(0,60).map(([key,us])=>({frame:frames.get(key),weightedUs:us,percent:100*us/totalUs}));
 return {sampleCount:profile.samples.length,weightedUs:totalUs,profileSpanUs:profile.endTime-profile.startTime,selfCategories:Object.fromEntries([...categories].map(([name,us])=>[name,{weightedUs:us,percent:100*us/totalUs}])),topSelf:rank(self),topInclusive:rank(inclusive),architectureGroups:Object.fromEntries([...groups].map(([name,g])=>[name,{...g,selfPercent:100*g.selfUs/totalUs,inclusivePercent:100*g.inclusiveUs/totalUs}])),groupPatterns:Object.fromEntries(groupRules.map(([name,pattern])=>[name,pattern.source])),phaseCallCounts:null,phaseCallCountsReason:'CPU samples are not invocation counts. No production instrumentation or monkey-patching is installed.'};
}
const report={schemaVersion:1,diagnostic:true,startedAt:new Date().toISOString(),oracleRevision:revision,oracleSHA256,runtimeHashes:before,wasmSHA256:before.candidate['pkg/pele_bg.wasm'],ownerModuleSHA256:before.candidate['captured-owner.js'],harnessSHA256:sha(await readFile(fileURLToPath(import.meta.url))),casesSHA256:sha(await readFile(new URL('./cases.mjs',import.meta.url))),settings:{selected,stages,resources,durationMs:duration,warmupMs:warmup,repeats,samplingIntervalUs:interval,plannedProfiles,identicalPackageBytes:true,candidateCapturedSankey:true,controlCapturedSankey:false,probeInsideProfile:false,forcedGC:false},host:{platform:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,loadBefore:loadavg(),freeMemoryBefore:freemem()},scope:'Sample attribution only. Same page and process share heaps/GC; independent same-byte package module and WASM instances. Inclusive groups overlap and are not additive. Samples do not prove allocation counts or call frequency. Profiler throughput is not the latency acceptance benchmark.',profiles:[],checks:[]};
const server=createServer(async(req,res)=>{try{
 res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
 const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Captured Sankey CPU diagnosis</title>');return;}
 let file;if(path==='/oracle.js')file=oracle;else{const name=Object.keys(roots).find(name=>path.startsWith('/'+name+'/'));assert(name);file=resolve(roots[name],'.'+path.slice(name.length+1));assert(file.startsWith(roots[name]+'/'));}
 res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
 }catch{res.writeHead(404);res.end();}});
await mkdir(output,{recursive:true});await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));let browser;
const deadline=setTimeout(()=>void browser?.close(),240000);
try{
 browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 await page.evaluate(async now=>{
  Date.now=()=>now;const oracle=await import('/oracle.js'),controlAPI=await import('/control/index.js'),candidateAPI=await import('/candidate/index.js'),controlOwner=await import('/control/captured-owner.js'),candidateOwner=await import('/candidate/captured-owner.js');
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
   const probe=(name,model,lane)=>{const actual=owners[name].capturedSankeyBackend(model),expected=name==='candidate'?'source':undefined;if(actual!==expected)throw Error(name+'/'+lane+': backend '+String(actual));return{runtime:name,lane,actual:actual??null};};
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
