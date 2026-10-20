// Frozen independent native consumers: allocation counters only, never elapsed timing.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
import {resolve} from 'node:path';
import {cases} from '../../../../scripts/perf/cases.mjs';
const base=resolve('dist/perf-patches/parse-architecture/er-borrowed-interner');
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const workloads=cases().filter(c=>['er/medium','flowchart/medium'].includes(c.id));
assert.equal(workloads.length,2);
const commands=workloads.flatMap(workload=>[
  {op:'prepare',case:workload},{op:'verify',case:workload.id},
  ...(workload.id==='er/medium'?['parse-public','parse','render-source','render-retained']:['parse-public']).flatMap(stage=>[1,16].map(count=>({op:'allocations',case:workload.id,stage,count}))),
  {op:'verify',case:workload.id},
]);
const runs={};
for(const name of ['control','candidate']){
  const directory=base+'/'+name+'-native',binary=directory+'/pele-perf-alloc';
  const manifest=JSON.parse(await readFile(directory+'/build-manifest.json','utf8'));
  const binarySHA256=sha(await readFile(binary));
  assert.equal(binarySHA256,manifest.binaries['pele-perf-alloc']);
  // Existing protocol gates allocation requests with ALLOW_TIMING too. This
  // script sends no measure request; allocation binary rejects such requests.
  const child=spawn(binary,[],{env:{...process.env,TZ:'UTC',PELE_PERF_ALLOW_TIMING:'1'},stdio:['pipe','pipe','pipe']});
  let stdout='',stderr='';child.stdout.setEncoding('utf8');child.stderr.setEncoding('utf8');
  child.stdout.on('data',s=>stdout+=s);child.stderr.on('data',s=>stderr+=s);
  const timeout=setTimeout(()=>child.kill('SIGTERM'),60000);
  const done=new Promise((resolve,reject)=>{child.on('error',reject);child.on('exit',(code,signal)=>code===0?resolve():reject(Error(`consumer failed ${code}/${signal}: ${stderr}`)));});
  child.stdin.end(commands.map(c=>JSON.stringify(c)).join('\n')+'\n');
  try{await done;}finally{clearTimeout(timeout);}
  const responses=stdout.trim().split('\n').map(line=>JSON.parse(line));
  assert.equal(responses.length,commands.length);const prepared=new Map();
  responses.forEach((r,i)=>{
    assert(r.ok,JSON.stringify(r));const c=commands[i];
    if(c.op==='prepare')prepared.set(c.case.id,r.value.output);
    if(c.op==='verify')for(const output of r.value.outputs)assert.deepEqual(output,prepared.get(c.case));
  });
  assert.equal(sha(await readFile(binary)),binarySHA256);
  runs[name]={binarySHA256,manifest,responses,stderr};
}
const rows=[];
commands.forEach((c,i)=>{
  const control=runs.control.responses[i].value,candidate=runs.candidate.responses[i].value;
  if(c.op==='allocations'){
    assert.equal(candidate.checksum,control.checksum);
    const delta=Object.fromEntries(Object.keys(control).filter(k=>k!=='checksum').map(k=>[k,candidate[k]-control[k]]));
    rows.push({case:c.case,stage:c.stage,count:c.count,control,candidate,delta});
  }else assert.deepEqual(candidate,control);
});
await writeFile(base+'/allocation-comparison.json',JSON.stringify({kind:'allocation-only',timingSamples:false,commands,runs,rows,checks:{beforeAfterOutputEquality:true,binaryUnchanged:true,controlCandidateOutputEquality:true}},null,2)+'\n');
console.log(JSON.stringify(rows,null,2));
