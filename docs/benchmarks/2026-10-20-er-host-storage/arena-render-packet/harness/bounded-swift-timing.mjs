// Explicitly authorized one-shot scheduler. No builds, waits for quiet, retries or pooling.
import assert from 'node:assert/strict';
import {readFile,writeFile,mkdir,open} from 'node:fs/promises';
import {spawn} from 'node:child_process';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {performance} from 'node:perf_hooks';
import {guardQuiet,inputs,sha,systemProcesses} from '../../../../scripts/perf/swift/build.mjs';
assert.deepEqual(process.argv.slice(2),['--parent-go'],'Run only after explicit parent go; do not start from log completion alone');
const root=process.cwd(),base='dist/perf-patches/parse-architecture/arena-render-packet';
const output=base+'/runtime/swift-bounded-attempt1';
await mkdir(output); // Exclusive attempt directory: existing evidence must never be overwritten.
const started=performance.now(),deadline=started+180_000;
const report={schemaVersion:1,startedAt:new Date().toISOString(),budgetMs:180_000,strictGuardUnchanged:true,retries:0,pooling:false,case:'sankey/medium',stages:['render','lifetime','parseMutable','parse'],trialsPerCondition:15,warmupMsPerSide:1000,conditions:[],acceptedConditions:[],unmeasuredConditions:[]};
const save=()=>writeFile(output+'/schedule.json',JSON.stringify(report,null,2)+'\n');
const controlDir='dist/perf-patches/parse-architecture/owned-public-construction/candidate-swift';
const candidateDir=base+'/candidate-swift';
const candidateManifestPath=candidateDir+'/swift-build-manifest.json';
let currentCondition;
try {
 const fullLog=await readFile('dist/evidence/arena-render-packet-full-web.log','utf8');
 assert(/(?:^|\n)\s*417 passed\b/m.test(fullLog),'Full browser suite has not reported terminal417 passed');
 report.fullBrowserLogSha256=sha(Buffer.from(fullLog));
 report.schedulerSha256=sha(await readFile(fileURLToPath(import.meta.url)));
 report.comparatorSha256=sha(await readFile('scripts/perf/swift/compare.mjs'));
 report.guardModuleSha256=sha(await readFile('scripts/perf/swift/build.mjs'));
 report.hostProcessesBefore=systemProcesses();
 guardQuiet(); // Any compiler, even CPU0, or Pele xctest ends this attempt immediately.
 const manifest=JSON.parse(await readFile(candidateManifestPath,'utf8'));
 const control=JSON.parse(await readFile(controlDir+'/swift-build-manifest.json','utf8'));
 assert.equal(manifest.sourceInputSha256,'e632dc000ed18bd395fe971a7029faba664278cc58b4e2b525eaa6effecf9e58');
 assert.equal(manifest.binarySha256,'2e1491d5b80da5d64b7f042abe934f47d662901d9d8037eb1646a0d7a03cff9a');
 assert.equal(control.binarySha256,'368ab53bad3b3fdd6daa82cce6a53932a8eed8ea922d5c7683370b4e3acbc0cf');
 assert.equal(sha(await readFile(candidateDir+'/PeleSwiftPerf')),manifest.binarySha256);
 assert.equal(sha(await readFile(manifest.binary)),manifest.binarySha256);
 assert.equal(sha(await readFile(controlDir+'/PeleSwiftPerf')),control.binarySha256);
 const before=await inputs(root);assert.equal(before.sha256,manifest.sourceInputSha256);
 report.frozen={sourceInputSha256:before.sha256,candidateBinarySha256:manifest.binarySha256,controlBinarySha256:control.binarySha256,candidateManifestSha256:sha(await readFile(candidateManifestPath)),controlManifestSha256:sha(await readFile(controlDir+'/swift-build-manifest.json'))};
 await save();
 for(const [index,stage] of report.stages.entries()) {
  if(performance.now()>=deadline)throw Error('Overall180-second budget exhausted before next condition');
  guardQuiet();
  const target=output+'/'+stage+'.json',logPath=output+'/'+stage+'.log';
  currentCondition={stage,orderOffset:index%2,startedAt:new Date().toISOString(),output:target,log:logPath};
  report.conditions.push(currentCondition);
  const args=['scripts/perf/swift/compare.mjs','--quiet-run','--control-dir',controlDir,'--candidate-manifest',candidateManifestPath,'--cases','sankey/medium','--stages',stage,'--order-offset',String(index%2),'--output',target];
  currentCondition.command=[process.execPath,...args];await save();
  const log=await open(logPath,'wx');let child,timer,timedOut=false;
  try {
   const remaining=deadline-performance.now();assert(remaining>0,'Overall180-second budget exhausted');
   const result=await new Promise((resolve,reject)=>{
    child=spawn(process.execPath,args,{cwd:root,detached:true,stdio:['ignore',log.fd,log.fd]});
    timer=setTimeout(()=>{timedOut=true;try{process.kill(-child.pid,'SIGKILL')}catch(e){if(e.code!=='ESRCH')reject(e)}},remaining);
    child.on('error',reject);child.on('exit',(code,signal)=>resolve({code,signal}));
   });
   currentCondition.exitCode=result.code;currentCondition.signal=result.signal;
  } finally {clearTimeout(timer);await log.close()}
  currentCondition.completedAt=new Date().toISOString();currentCondition.timedOut=timedOut;
  if(timedOut)throw Error('Overall180-second budget exhausted; condition rejected');
  if(currentCondition.exitCode!==0)throw Error('Condition failed or strict guard rejected; no retry: '+stage);
  guardQuiet();
  const data=JSON.parse(await readFile(target,'utf8'));
  assert.equal(data.completed,true);assert.equal(data.rows.length,1);assert.equal(data.rows[0].raw.length,15);
  assert.equal(data.rows[0].case,'sankey/medium');assert.equal(data.rows[0].stage,stage);
  currentCondition.accepted=true;currentCondition.reportSha256=sha(await readFile(target));report.acceptedConditions.push(stage);
  await save();console.log('Accepted complete condition '+stage);
 }
 const after=await inputs(root);assert.equal(after.sha256,report.frozen.sourceInputSha256);
 assert.equal(sha(await readFile(manifest.binary)),report.frozen.candidateBinarySha256);
 assert.equal(sha(await readFile(candidateDir+'/PeleSwiftPerf')),report.frozen.candidateBinarySha256);
 assert.equal(sha(await readFile(controlDir+'/PeleSwiftPerf')),report.frozen.controlBinarySha256);
 guardQuiet();report.succeeded=true;report.completedAt=new Date().toISOString();
} catch(error) {
 report.succeeded=false;report.failedAt=new Date().toISOString();report.error=String(error);
 if(currentCondition&&!currentCondition.accepted)currentCondition.rejected=true;
 process.exitCode=1;
} finally {
 report.unmeasuredConditions=report.stages.filter(stage=>!report.acceptedConditions.includes(stage));
 report.elapsedWallMs=performance.now()-started;report.hostProcessesAfter=systemProcesses();await save();
 console.log(JSON.stringify({succeeded:report.succeeded,acceptedConditions:report.acceptedConditions,unmeasuredConditions:report.unmeasuredConditions,error:report.error,elapsedWallMs:report.elapsedWallMs}));
}
