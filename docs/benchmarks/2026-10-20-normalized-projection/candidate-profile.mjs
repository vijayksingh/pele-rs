import assert from 'node:assert/strict';import fs from 'node:fs/promises';import {spawn} from 'node:child_process';import {createInterface} from 'node:readline';import {createHash} from 'node:crypto';
import {guardQuiet,inputs} from '../../../../../scripts/perf/swift/build.mjs';import {cases} from '../../../../../scripts/perf/cases.mjs';
const root=process.cwd(),dir='dist/perf-patches/parse-architecture/normalized-projection/post-profile/',manifest=JSON.parse(await fs.readFile('scripts/perf/swift/.build/swift-build-manifest.json'));
const sha=b=>createHash('sha256').update(b).digest('hex');assert.equal((await inputs(root)).sha256,manifest.sourceInputSha256);assert.equal(sha(await fs.readFile(manifest.binary)),manifest.binarySha256);
const report={buildManifest:manifest,profiles:[]};
for(const id of ['gantt/medium','sequence/small']){
 const before=guardQuiet(),child=spawn(manifest.binary,[],{env:{...process.env,TZ:'UTC',PELE_PERF_ALLOW_TIMING:'1'},stdio:['pipe','pipe','inherit']}),pending=[];
 const lines=createInterface({input:child.stdout});lines.on('line',text=>{const p=pending.shift();if(!p)return;try{const value=JSON.parse(text);value.ok?p.resolve(value.value):p.reject(Error(value.error));}catch(e){p.reject(e);}});
 const send=value=>new Promise((resolve,reject)=>{pending.push({resolve,reject});child.stdin.write(JSON.stringify(value)+'\n');});
 try{
 const workload=cases().find(c=>c.id===id),prepared=await send({op:'prepare',workload});
 for(let i=0;i<3;i++)await send({op:'measure',caseID:id,stage:'parse',count:128});
 const filename=id.replace('/','-')+'.sample.txt',sample=spawn('sample',[String(child.pid),'8','1','-file',root+'/'+dir+filename],{stdio:'ignore'});
 let finished=false;const sampling=new Promise((resolve,reject)=>{sample.on('error',reject);sample.on('exit',code=>{finished=true;code===0?resolve():reject(Error('sample failed '+code));});});
 let batches=0;while(!finished){guardQuiet();const result=await send({op:'measure',caseID:id,stage:'parse',count:128});assert.equal(result.caseID,id);batches++;}
 await sampling;const verified=await send({op:'verify',caseID:id});assert.deepEqual(verified.outputs,prepared.outputs);
 report.profiles.push({case:id,stage:'parse',count:128,batches,pid:child.pid,filename,systemBefore:before,systemAfter:guardQuiet(),sampleSHA256:sha(await fs.readFile(dir+filename)),verified});
 }finally{child.stdin.end();child.kill();}
}
assert.equal((await inputs(root)).sha256,manifest.sourceInputSha256);await fs.writeFile(dir+'profile.json',JSON.stringify(report,null,2)+'\n');console.log('Swift profiles captured and complete outputs verified');
