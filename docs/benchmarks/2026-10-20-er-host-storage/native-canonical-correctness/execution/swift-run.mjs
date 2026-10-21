// Real Swift public APIs. No timed work without --quiet-run and a frozen build.
process.env.TZ='UTC';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {spawn,execFileSync} from 'node:child_process';
import {createInterface} from 'node:readline';
import {cpus,loadavg,platform,arch,totalmem,freemem} from 'node:os';
import {join,dirname,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
import {packageDir,repository,inputs,sha,guardQuiet,systemProcesses} from './build.mjs';
const args=process.argv.slice(2),arg=(name,fallback)=>{const i=args.indexOf(name);return i<0?fallback:args[i+1]};
const root=repository(),timed=args.includes('--quiet-run');
const {cases,revision,now}=await import(pathToFileURL(join(root,'scripts/perf/cases.mjs')));
const bundle=join(root,'node_modules/pele/dist/pele.js'),oracleSha256=sha(await readFile(bundle));
assert.equal(oracleSha256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d','Pinned development oracle differs');
const upstream=await import(pathToFileURL(bundle));Date.now=()=>now;
const knownStages=['parse','render-source','render-retained','parseMutable','snapshot','apply','render','lifetime'];
const stages=arg('--stages',knownStages.join(',')).split(',');assert(stages.length&&stages.every(s=>knownStages.includes(s))&&new Set(stages).size===stages.length,'Invalid stages');
const ids=arg('--cases','').split(',').filter(Boolean),filter=arg('--filter','');
const selected=cases().filter(c=>(!ids.length||ids.includes(c.id))&&(!filter||c.id.includes(filter)));
assert(selected.length,'No cases selected');assert(ids.every(id=>selected.some(c=>c.id===id)),'Unknown/filtered explicit case');
const trials=Number(arg('--trials',7)),targetMs=Number(arg('--batch-ms',12)),warmupMs=Number(arg('--warmup-ms',100));
assert(Number.isInteger(trials)&&trials>=3&&trials<=31&&targetMs>=2&&targetMs<=100&&warmupMs>=0&&warmupMs<=1000,'Invalid bounded timing settings');
const manifest=JSON.parse(await readFile(join(packageDir,'.build/swift-build-manifest.json'),'utf8'));
assert.equal(manifest.schemaVersion,1);assert.equal((await inputs(root)).sha256,manifest.sourceInputSha256,'Source/SDK changed since consumer build');
assert.equal(sha(await readFile(manifest.binary)),manifest.binarySha256,'Swift executable differs from build manifest');
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex')};
const summary=r=>({type:r.type,widthBits:bits(r.width),heightBits:bits(r.height),svgSha256:sha(Buffer.from(r.svg)),svgBytes:Buffer.byteLength(r.svg)});
const resources=()=>({measurer:{width:(text,size,style)=>text.length*size*((style&1)?0.71:0.6)}});
const maxBatch=c=>Math.max(1,Math.min(128,Math.floor(25_000/c.units)));
const quantile=(values,p)=>{const v=[...values].sort((a,b)=>a-b);return v[Math.max(0,Math.ceil(v.length*p)-1)]};
function statistics(samples){const median=quantile(samples,.5);return{medianMs:median,madMs:quantile(samples.map(n=>Math.abs(n-median)),.5),minMs:Math.min(...samples),maxMs:Math.max(...samples),sampleP95BatchMeanMs:quantile(samples,.95)}}
const system=()=>({loadAverage:loadavg(),freeMemory:freemem(),processes:systemProcesses().filter(p=>p.cpuPercent>=10||/^(cargo|rustc|swiftc|swift-build|swift-frontend|xcodebuild|xctest|ld)$/.test(p.name)).sort((a,b)=>b.cpuPercent-a.cpuPercent).slice(0,20)});
class SwiftConsumer {
 constructor(){this.pending=[];this.dead=false;this.child=spawn(manifest.binary,[],{cwd:root,env:{...process.env,TZ:'UTC',PELE_PERF_ALLOW_TIMING:timed?'1':'0'},stdio:['pipe','pipe','inherit']});createInterface({input:this.child.stdout}).on('line',line=>{const p=this.pending.shift();if(!p){this.fail(Error('Unexpected Swift protocol line'));return}clearTimeout(p.timer);try{const r=JSON.parse(line);r.ok?p.resolve(r.value):p.reject(Error(r.error))}catch(e){p.reject(e)}});this.child.on('error',e=>this.fail(e));this.child.on('exit',(code,signal)=>this.fail(Error('Swift consumer exited '+(code??signal))))}
 fail(error){this.dead=true;for(const p of this.pending){clearTimeout(p.timer);p.reject(error)}this.pending=[]}
 send(request){if(this.dead)return Promise.reject(Error('Swift consumer closed'));return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{this.child.kill();this.fail(Error('Swift request exceeded 60 seconds'))},60_000);this.pending.push({resolve,reject,timer});this.child.stdin.write(JSON.stringify(request)+'\n')})}
 close(){this.child.stdin.end();this.child.kill()}
}
const report={schemaVersion:1,mode:timed?'local-swift-api-timing':'swift-api-correctness-only',startedAt:new Date().toISOString(),revision:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),worktree:execFileSync('git',['status','--porcelain'],{cwd:root,encoding:'utf8'}).trim().split('\n').filter(Boolean),buildManifest:manifest,oracleRevision:revision,oracleSha256,node:process.version,platform:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,totalMemory:totalmem(),settings:{trials,targetBatchMs:targetMs,warmupMs,fixedClock:now,timeZone:'UTC',resources:'UTF16 length × size × (bold?.71:.6)',stages,selectedCases:selected.map(c=>c.id),freshRetainedStatePerBatch:true,ownership:'Per-operation autorelease pool drain is inside the API timer; retained document setup/warm render and verification are outside'},scope:{apiCosts:'Actual Swift APIs include C FFI, binary graph crossings, intrinsic JSON encoding/decoding, native validation/import, fresh C registry on parse, and mutable publication on render.',lifetime:'parseMutable -> snapshot -> no-op apply -> render -> close, including owned values disposal',excluded:'Process/module startup, NDJSON control, output hashing/verification, AppKit/DOM decoding/painting and GPU work',comparisons:'No JS ratio, allocator comparison or subtraction from native component measurements',externalLoad:'Unrelated xctest processes are recorded, not stopped or used to block the run. Results describe a shared host, not a globally idle machine.'},systemBefore:timed?(guardQuiet(),system()):system(),cases:[],rows:[]};
let swift;
function check(output,expected,label){assert.deepEqual(output,expected,label)}
try{
 swift=new SwiftConsumer();
 for(const workload of selected){
  const expected=summary(upstream.render(workload.source,{...workload.options,...resources()}));
  const prepared=await swift.send({op:'prepare',workload});for(const output of prepared.outputs)check(output,expected,workload.id+' prepared');
  report.cases.push({...workload,sourceSha256:sha(Buffer.from(workload.source)),sourceBytes:Buffer.byteLength(workload.source),output:expected,prepared});
  // All selected executable API paths run once even in correctness-only mode.
  for(const stage of stages){
   const exercised=await swift.send({op:'exercise',caseID:workload.id,stage,count:1});check(exercised.output,expected,workload.id+' '+stage+' correctness');
   if(!timed){report.rows.push({case:workload.id,stage,correctness:exercised});continue}
   guardQuiet();let warmOps=0,warmNs=0;
   for(let i=0;i<6||warmNs<warmupMs*1e6;i++){const count=i<6?1:Math.min(32,maxBatch(workload));const sample=await swift.send({op:'measure',caseID:workload.id,stage,count});check(sample.output,expected,workload.id+' warmup '+stage);warmOps+=count;warmNs+=sample.elapsedNs}
   let count=1;
   while(count<maxBatch(workload)){const sample=await swift.send({op:'measure',caseID:workload.id,stage,count});check(sample.output,expected,workload.id+' calibration '+stage);if(sample.elapsedNs>=targetMs*1e6)break;count=Math.min(maxBatch(workload),count*2)}
   const raw=[];
   for(let trial=0;trial<trials;trial++){guardQuiet();const sample=await swift.send({op:'measure',caseID:workload.id,stage,count});check(sample.output,expected,workload.id+' trial '+stage);raw.push(sample)}
   const samplesMs=raw.map(s=>s.elapsedNs/s.count/1e6);report.rows.push({case:workload.id,stage,count,warmup:{operations:warmOps,elapsedNs:warmNs},...statistics(samplesMs),samplesMs,raw});
  }
  const final=await swift.send({op:'verify',caseID:workload.id});for(const output of final.outputs)check(output,expected,workload.id+' post-run');
  console.log((timed?'measured ':'verified ')+workload.id);
 }
 assert.equal((await inputs(root)).sha256,manifest.sourceInputSha256,'Source/SDK changed during run');assert.equal(sha(await readFile(manifest.binary)),manifest.binarySha256,'Swift executable changed during run');
 report.completedAt=new Date().toISOString();report.systemAfter=system();
 const output=resolve(arg('--output',join(packageDir,'.build',timed?'swift-api-timing.json':'swift-api-verified.json')));await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{swift?.close()}
