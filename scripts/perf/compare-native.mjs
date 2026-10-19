// Diagnostic: same-host pinned JS plus independent frozen Rust consumer binaries.
// Every timer is inside the measured runtime; control pipes are outside it.
process.env.TZ='UTC';
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {createInterface} from 'node:readline';
import {dirname,resolve} from 'node:path';
import {cpus,loadavg,platform,arch} from 'node:os';
import * as js from 'pele';
import {cases,now} from './cases.mjs';
import {guardQuiet,systemProcesses} from './swift/build.mjs';
const args=process.argv.slice(2),arg=(n,d)=>{const i=args.indexOf(n);return i<0?d:args[i+1]};
assert(args.includes('--quiet-run'),'Explicit coordinated --quiet-run required');
assert(global.gc,'Start Node with --expose-gc');
const trials=Number(arg('--trials',15)),warmupMs=Number(arg('--warmup-ms',500));
assert(trials>=15&&trials<=30&&trials%3===0&&warmupMs>=500&&warmupMs<=1000);
const names=['js','control','candidate'],stages=arg('--stages','render-source,render-retained').split(',');
assert(stages.length&&new Set(stages).size===stages.length&&stages.every(s=>['render-source','render-retained'].includes(s)));
const ids=arg('--cases','').split(',').filter(Boolean),selected=cases().filter(c=>c.scale==='medium'&&(!ids.length||ids.includes(c.id)));
assert(selected.length&&ids.every(id=>selected.some(c=>c.id===id)),'Unknown medium workload');
const sha=b=>createHash('sha256').update(b).digest('hex');
const hashInputs=async()=>{
  const {execFileSync}=await import('node:child_process');
  const files=execFileSync('rg',['--files','crates/pele/src','scripts/perf/native/src'],{encoding:'utf8'}).trim().split('\n').concat(['crates/pele/Cargo.toml','Cargo.toml','scripts/perf/native/Cargo.toml','scripts/perf/native/Cargo.lock']).sort();
  const hash=createHash('sha256');for(const file of files){hash.update(file+'\0');hash.update(await readFile(file));}return hash.digest('hex');
};
Date.now=()=>now;
const host=()=>({measurer:{width:(t,z,s)=>t.length*z*(s&1?.71:.6)}});
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex')};
const summary=r=>({type:r.type,widthBits:bits(r.width),heightBits:bits(r.height),svgSha256:sha(Buffer.from(r.svg)),svgBytes:Buffer.byteLength(r.svg)});
class Native {
  constructor(binary){
    this.pending=[];this.dead=false;
    this.child=spawn(binary,[],{env:{...process.env,TZ:'UTC',PELE_PERF_ALLOW_TIMING:'1'},stdio:['pipe','pipe','inherit']});
    createInterface({input:this.child.stdout}).on('line',line=>{
      const item=this.pending.shift();if(!item){this.fail(Error('Unexpected native output'));return;}
      clearTimeout(item.timer);try{const reply=JSON.parse(line);reply.ok?item.resolve(reply.value):item.reject(Error(reply.error));}catch(error){item.reject(error);}
    });
    this.child.on('error',e=>this.fail(e));this.child.on('exit',(code,signal)=>this.fail(Error('Native exited '+(code??signal))));
  }
  fail(error){this.dead=true;for(const item of this.pending){clearTimeout(item.timer);item.reject(error);}this.pending=[];}
  send(request){if(this.dead)return Promise.reject(Error('Native closed'));return new Promise((resolve,reject)=>{
    const timer=setTimeout(()=>{this.child.kill();this.fail(Error('Native request exceeded 60s'));},60_000);
    this.pending.push({resolve,reject,timer});this.child.stdin.write(JSON.stringify(request)+'\n');
  });}
  close(){this.child.stdin.end();this.child.kill();}
}
const median=a=>{const b=[...a].sort((x,y)=>x-y);return b[Math.floor(b.length/2)]};
const paths=Object.fromEntries(names.slice(1).map(n=>[n,resolve(arg('--'+n,''))]));
assert(args.includes('--control')&&args.includes('--candidate'),'Both frozen binary paths required');
const inputHash=await hashInputs(),artifacts={};
for(const name of names.slice(1)){
  const manifest=JSON.parse(await readFile(resolve(dirname(paths[name]),'build-manifest.json'),'utf8'));
  const hash=sha(await readFile(paths[name]));assert.equal(hash,manifest.binaries['pele-perf'],name+' binary differs');
  if(name==='candidate')assert.equal(inputHash,manifest.sourceInputSha256,'Candidate source differs');
  artifacts[name]={binarySHA256:hash,manifest};
}
const oracleSHA256=sha(await readFile('node_modules/pele/dist/pele.js'));
assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const report={schemaVersion:1,startedAt:new Date().toISOString(),mode:'native-three-way-diagnostic',artifacts,oracleSHA256,harnessSHA256:sha(await readFile(new URL(import.meta.url))),settings:{trials,warmupMs,matchedNativeBatchCounts:true,balancedCyclicAndReverseOrder:true,freshRetainedStatePerBatch:true,stages,selectedCases:selected.map(c=>c.id)},host:{platform:platform(),arch:arch(),cpu:cpus()[0]?.model,loadBefore:loadavg(),processesBefore:systemProcesses().filter(p=>p.cpuPercent>=10)},cases:[],rows:[]};
let native={};
try{
  guardQuiet();for(const name of names.slice(1))native[name]=new Native(paths[name]);
  for(const c of selected){
    const expected=summary(js.render(c.source,{...c.options,...host()}));
    for(const name of names.slice(1)){
      const prepared=await native[name].send({op:'prepare',case:c});assert.deepEqual(prepared.output,expected,name+' prepare '+c.id);
      for(const output of(await native[name].send({op:'verify',case:c.id})).outputs)assert.deepEqual(output,expected,name+' verify '+c.id);
    }
    report.cases.push({id:c.id,sourceSHA256:sha(Buffer.from(c.source)),output:expected});
    for(const stage of stages){
      const batch=async(name,count)=>{
        if(name!=='js')return native[name].send({op:'measure',case:c.id,stage,count});
        global.gc();const options={...c.options,...host()},model=js.parse(c.source,c.options);js.render(model,options);
        let checksum=0;const start=process.hrtime.bigint();for(let i=0;i<count;i++){const r=js.render(stage==='render-source'?c.source:model,options);checksum+=r.svg.length+r.width+r.height;}
        return{elapsedNs:Number(process.hrtime.bigint()-start),count,checksum};
      };
      const warmups={},counts={};
      for(const name of names){
        guardQuiet();let elapsedNs=0,operations=0;
        for(let i=0;i<6||elapsedNs<warmupMs*1e6;i++){const r=await batch(name,i<6?1:32);elapsedNs+=r.elapsedNs;operations+=r.count;}
        warmups[name]={elapsedNs,operations};const calibrations=[];
        for(let j=0;j<3;j++){let count=1;while(count<128&&(await batch(name,count)).elapsedNs<12e6)count*=2;calibrations.push(count);}
        counts[name]=median(calibrations);
      }
      counts.control=counts.candidate=Math.min(counts.control,counts.candidate);
      const samples=Object.fromEntries(names.map(n=>[n,[]])),raw=Object.fromEntries(names.map(n=>[n,[]]));
      for(let trial=0;trial<trials;trial++){
        guardQuiet();let order=names.map((_,i)=>names[(i+trial%3)%3]);if(Math.floor(trial/3)%2)order.reverse();
        for(const name of order){const r=await batch(name,counts[name]);raw[name].push(r);samples[name].push(r.elapsedNs/r.count/1e6);}
      }
      const medians=Object.fromEntries(names.map(n=>[n,median(samples[n])]));
      report.rows.push({case:c.id,stage,counts,warmups,samples,raw,medians,candidateOverControl:medians.candidate/medians.control,candidateOverJS:medians.candidate/medians.js});
    }
    for(const name of names.slice(1))for(const output of(await native[name].send({op:'verify',case:c.id})).outputs)assert.deepEqual(output,expected,name+' post-run '+c.id);
    console.log('compared '+c.id);
  }
  assert.equal(await hashInputs(),inputHash,'Source changed during run');
  for(const name of names.slice(1))assert.equal(sha(await readFile(paths[name])),artifacts[name].binarySHA256,'Binary changed');
  report.host.loadAfter=loadavg();report.host.processesAfter=systemProcesses().filter(p=>p.cpuPercent>=10);report.completedAt=new Date().toISOString();report.succeeded=true;
  const output=resolve(arg('--output','dist/evidence/native-three-way.json'));await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{for(const instance of Object.values(native))instance.close();}
