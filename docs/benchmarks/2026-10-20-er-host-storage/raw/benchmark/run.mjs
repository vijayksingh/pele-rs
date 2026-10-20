// Private diagnostic; never replaces the full VM release gate.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir,readdir,realpath} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {cpus,totalmem,freemem,loadavg,platform,arch} from 'node:os';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {chromium} from '@playwright/test';
const args=process.argv.slice(2),arg=(name,fallback)=>{const i=args.indexOf(name);return i<0?fallback:args[i+1];};
const root=resolve(arg('--root','.')),here=dirname(fileURLToPath(import.meta.url)),sha=data=>createHash('sha256').update(data).digest('hex');
for(const key of ['--control','--candidate','--inputs','--control-proof'])assert(arg(key),key+' is required');
const roots={control:await realpath(resolve(arg('--control'))),candidate:await realpath(resolve(arg('--candidate'))),oracle:await realpath(resolve(root,'node_modules/pele/dist/esm'))};
assert.notEqual(roots.control,roots.candidate,'Independent packages required');
const inputFile=resolve(arg('--inputs')),output=resolve(arg('--output','dist/evidence/raw-first-vm.json')),proofFile=resolve(arg('--control-proof'));
const {cases,revision,now}=await import(pathToFileURL(resolve(root,'scripts/perf/cases.mjs')));
const {guardQuiet,systemProcesses}=await import(pathToFileURL(resolve(root,'scripts/perf/swift/build.mjs')));
const trials=Number(arg('--trials','15')),warmupMs=Number(arg('--warmup-ms','1000'));
assert.equal(trials,15,'This diagnostic fixes 15 balanced triples');assert(warmupMs>=500&&warmupMs<=2000);
const ids=arg('--cases','er/small,er/medium,er/large').split(','),profiles=arg('--profiles','fixed,canvas').split(','),stages=arg('--stages','clone-only,raw-first-inclusive,raw-repeat').split(',');
assert(ids.length&&new Set(ids).size===ids.length&&ids.every(id=>['er/small','er/medium','er/large'].includes(id)));
assert(profiles.length&&new Set(profiles).size===profiles.length&&profiles.every(p=>['fixed','canvas'].includes(p)));
assert(stages.length&&new Set(stages).size===stages.length&&stages.every(s=>['clone-only','raw-first-inclusive','raw-repeat'].includes(s)));
const workloads=cases().filter(c=>ids.includes(c.id));
async function hashes(directory,prefix=''){
 const result={};for(const entry of(await readdir(resolve(directory,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(result,await hashes(directory,name+'/'));else if(entry.isFile())result[name]=sha(await readFile(resolve(directory,name)));else throw Error('Non-regular input: '+name);}return result;
}
const stable=object=>Object.fromEntries(Object.entries(object).sort(([a],[b])=>a.localeCompare(b)));
const pinOracle=async()=>{const bytes=await readFile(resolve(root,'node_modules/pele/dist/pele.js'));assert.equal(sha(bytes),'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d','Pinned Pele bundle differs');return sha(bytes);};
const hashInputs=async()=>({packages:Object.fromEntries(await Promise.all(Object.entries(roots).map(async([key,path])=>[key,await hashes(path)]))),oracleBundleSHA256:await pinOracle(),harness:{'run.mjs':sha(await readFile(fileURLToPath(import.meta.url))),'browser.mjs':sha(await readFile(resolve(here,'browser.mjs'))),'oracle-pins.json':sha(await readFile(resolve(here,'oracle-pins.json')))},casesSHA256:sha(await readFile(resolve(root,'scripts/perf/cases.mjs'))),guardSHA256:sha(await readFile(resolve(root,'scripts/perf/swift/build.mjs'))),controlProofSHA256:sha(await readFile(proofFile))});
const oraclePins=JSON.parse(await readFile(resolve(here,'oracle-pins.json'),'utf8'));
const before=await hashInputs();assert.equal(oraclePins.sourceRevision,revision);assert.deepEqual(stable(before.packages.oracle),stable(oraclePins.files),'Pinned ESM oracle differs');
const proof=JSON.parse(await readFile(proofFile,'utf8'));
const proofSide=arg('--control-proof-side','candidate');assert(['candidate','control'].includes(proofSide));
const proofHashes=proof.runtimeHashes?.[proofSide];assert(proofHashes,'Control proof must be an existing comparison report with runtimeHashes.'+proofSide);
assert.deepEqual(stable(before.packages.control),stable(proofHashes),'Control differs from preceding accepted package report');
assert.notDeepEqual(before.packages.control,before.packages.candidate,'A/A packages are not this diagnostic');
if(args.includes('--freeze-inputs')){
 await mkdir(dirname(inputFile),{recursive:true});await writeFile(inputFile,JSON.stringify({schemaVersion:1,createdAt:new Date().toISOString(),oracleRevision:revision,roots,controlProof:{path:proofFile,side:proofSide},hashes:before},null,2)+'\n',{flag:'wx'});
 console.log(inputFile);
}else{
 assert(args.includes('--quiet-run'),'Explicit coordinated --quiet-run required');
 const freeze=JSON.parse(await readFile(inputFile,'utf8'));assert.deepEqual(before,freeze.hashes,'Frozen inputs differ');assert.equal(freeze.oracleRevision,revision);
 assert.equal(platform(),'linux','Run this diagnostic on the coordinated Linux VM');
 const processes=guardQuiet();
 const existingBrowsers=systemProcesses(root).filter(p=>/(chrome|chromium|firefox|webkit)/i.test(p.name));assert.equal(existingBrowsers.length,0,'Another browser is active: '+JSON.stringify(existingBrowsers));
 const host=()=>({loadAverage:loadavg(),freeMemory:freemem()});
 const report={schemaVersion:1,diagnostic:true,startedAt:new Date().toISOString(),oracleRevision:revision,inputManifestSHA256:sha(await readFile(inputFile)),inputHashes:before,runtimeHashes:{control:before.packages.control,candidate:before.packages.candidate},oracleHashes:before.packages.oracle,platform:{os:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,totalMemory:totalmem()},hostBefore:host(),guardBefore:processes,settings:{trials,warmupMs,profiles,stages,cases:ids,batchTargetMs:12,maxBatch:128,matchedThreeWayBatchCount:true,balancedThreeWayOrder:true,freshCallerRootEveryFirstOperation:true,cloneAndDisposeInsideFirstTimer:true,retainedInitialAdoptionOutsideTimer:true,quietGuard:'scripts/perf/swift/build.mjs guardQuiet; active-browser preflight; before/after every row'},scope:'Warm CPU public renderRaw diagnostic. First-inclusive includes fresh descriptor/alias preserving clone, all raw setup/admission, first full render and disposal. Clone-only is disclosed separately and must not be subtracted as causal native cost. Repeated render explicitly excludes its first adoption/clone/disposal. No source parse, download/startup, paint or GPU timing. One shared page/heap; GC and serial warmup remain limitations. No release/parity claim.',rows:[],guards:[]};
 const server=createServer(async(req,res)=>{try{
  res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
  const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Raw adoption diagnostic</title>');return;}
  let file;if(path==='/browser.mjs')file=resolve(here,'browser.mjs');else{const key=Object.keys(roots).find(key=>path.startsWith('/'+key+'/'));assert(key);file=resolve(roots[key],'.'+path.slice(key.length+1));assert(file.startsWith(roots[key]+'/'));}
  res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
 }catch{res.writeHead(404);res.end();}});
 await new Promise(done=>server.listen(0,'127.0.0.1',done));let browser;
 const deadline=setTimeout(()=>void browser?.close(),600000);
 const save=async suffix=>{await mkdir(dirname(output),{recursive:true});await writeFile(output+suffix,JSON.stringify(report,null,2)+'\n');};
 try{
  browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();
  const context=await browser.newContext({timezoneId:'UTC'}),page=await context.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const clock=await page.evaluate(async now=>{
   Date.now=()=>now;
   const api=await import('/control/index.js'),candidate=await import('/candidate/index.js'),oracle=(await import('/oracle/diagrams/er.js')).default;
   const control=await api.loadRenderer(),next=await candidate.loadRenderer();
   window.rawBenchmark={api,oracle,runtimes:{control,candidate:next}};
   let resolution=Infinity,last=performance.now();for(let i=0;i<4096;i++){const value=performance.now();if(value>last)resolution=Math.min(resolution,value-last);last=value;}
   return {crossOriginIsolated,resolutionMs:resolution};
  },now);
  assert(clock.crossOriginIsolated&&clock.resolutionMs>0&&clock.resolutionMs<=.01,'Isolated high-resolution clock required');report.clock=clock;
  for(const workload of workloads)for(const profile of profiles)for(const stage of stages){
   const guard={case:workload.id,profile,stage,before:guardQuiet()};
   const row=await page.evaluate(async settings=>(await import('/browser.mjs')).runRow(settings),{workload,profile,stage,trials,warmupMs});
   guard.after=guardQuiet();report.guards.push(guard);
   row.sourceSHA256=sha(Buffer.from(workload.source));row.sourceBytes=Buffer.byteLength(workload.source);report.rows.push(row);await save('.partial');console.log([workload.id,profile,stage].join(' '));
  }
  await page.evaluate(()=>{window.rawBenchmark.runtimes.control.close();window.rawBenchmark.runtimes.candidate.close();});
  assert.deepEqual(await hashInputs(),before,'Inputs changed during diagnostic');report.hostAfter=host();report.guardAfter=guardQuiet();report.completedAt=new Date().toISOString();report.succeeded=true;await save('');console.log(output);
 }catch(error){report.succeeded=false;report.failure={name:error.name,message:error.message};report.failedAt=new Date().toISOString();await save('.failed');throw error;}
 finally{clearTimeout(deadline);await browser?.close();await new Promise(done=>server.close(done));}
}
