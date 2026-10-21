// Fail closed against the complete measured matrix and the actual package.
// This is a release prerequisite, not a claim about unmeasured workloads.
import assert from 'node:assert/strict';
import {readFile,readdir} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {cases} from './cases.mjs';
const [reportFile,packageDir]=process.argv.slice(2);
assert(reportFile&&packageDir,'Usage: node scripts/perf/check-web-parity.mjs REPORT.json PACKAGE_DIR');
const hash=b=>createHash('sha256').update(b).digest('hex'),report=JSON.parse(await readFile(reportFile,'utf8'));
assert.equal(report.schemaVersion,1);assert.equal(report.succeeded,true,'Benchmark incomplete');
assert.equal(report.harnessSHA256,hash(await readFile(new URL('./web-vm.mjs',import.meta.url))),'Benchmark harness differs');
assert.equal(report.casesSHA256,hash(await readFile(new URL('./cases.mjs',import.meta.url))),'Workload definitions differ');
assert.equal(report.settings.crossOriginIsolated,true,'Benchmark timer was not isolated');
assert(Number.isFinite(report.settings.observedClockResolutionMs)&&report.settings.observedClockResolutionMs>0&&report.settings.observedClockResolutionMs<=0.01,'Benchmark clock is too coarse');
assert.equal(report.settings.alternatingPairOrder,true);assert.equal(report.settings.freshRetainedModelPerBatch,true);
assert(report.settings.trials>=9&&report.settings.minimumWarmupMs>=100&&report.settings.minimumWarmupOperations>=6);
assert.equal(report.oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
async function packageHashes(prefix=''){
 const files={};for(const entry of (await readdir(resolve(packageDir,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const name=prefix+entry.name;if(entry.isDirectory())Object.assign(files,await packageHashes(name+'/'));else if(entry.isFile())files[name]=hash(await readFile(resolve(packageDir,name)));else throw Error('Package contains a non-regular file');}return files;
}
assert.deepEqual(await packageHashes(),report.runtimeHashes,'Package differs from measured artifact');
const workloads=cases(),expected=new Set(workloads.flatMap(c=>['fixed','canvas'].flatMap(p=>['render-source','parse','render-model'].map(s=>`${c.id}|${p}|${s}`))));
assert.equal(report.rows.length,expected.size,'Incomplete benchmark matrix');assert.equal(report.cases.length,workloads.length);
for(const workload of workloads){const measured=report.cases.find(c=>c.id===workload.id);assert(measured);assert.equal(measured.sourceSHA256,hash(Buffer.from(workload.source)));assert.deepEqual(measured.checks.map(c=>c.profile).sort(),['canvas','fixed']);}
const slower=[];
for(const row of report.rows){
 const key=`${row.case}|${row.profile}|${row.stage}`;assert(expected.delete(key),'Duplicate or unexpected row: '+key);
 assert(Number.isFinite(row.jsMedianMs)&&row.jsMedianMs>0&&Number.isFinite(row.wasmMedianMs)&&row.wasmMedianMs>0,'Invalid timing');
 assert.equal(row.wasmOverJS,row.wasmMedianMs/row.jsMedianMs,'Ratio differs from measured medians');
 for(const runtime of ['js','wasm']){assert(row.warmups[runtime].operations>=6&&row.warmups[runtime].elapsedMs>=100,'Insufficient warmup');assert(Number.isInteger(row.counts[runtime])&&row.counts[runtime]>0&&row.counts[runtime]<=128);}
 assert(row.samples.js.length>=9&&row.samples.wasm.length>=9,'Insufficient paired trials');
 if(row.wasmMedianMs>row.jsMedianMs)slower.push({case:row.case,profile:row.profile,stage:row.stage,wasmOverJS:row.wasmOverJS});
}
assert.equal(expected.size,0);
console.log(JSON.stringify({passed:slower.length===0,checked:report.rows.length,slowerCount:slower.length,slower},null,2));
if(slower.length)process.exitCode=1;
