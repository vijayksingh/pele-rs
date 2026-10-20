import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
const bytes=name=>readFile(new URL(name,import.meta.url));
const json=async name=>JSON.parse(await bytes(name));
const hash=b=>createHash('sha256').update(b).digest('hex');
const median=a=>{const s=[...a].sort((a,b)=>a-b),m=s.length>>1;return s.length%2?s[m]:(s[m-1]+s[m])/2;};
const near=(a,b,label)=>assert.ok(Math.abs(a-b)<=1e-10*Math.max(1,Math.abs(a),Math.abs(b)),`${label}: ${a} != ${b}`);
const key=r=>`${r.case}|${r.profile}|${r.stage}`;
function analyze(report,expectedRows,expectedTrials){
 assert.equal(report.succeeded,true);assert.equal(report.diagnostic,true);
 assert.equal(report.rows.length,expectedRows);assert.equal(report.settings.trials,expectedTrials);
 assert.equal(report.settings.matchedWasmBatchCounts,true);assert.equal(report.settings.balancedThreeWayOrder,true);
 assert.equal(report.settings.warmupMs,1000);assert.equal(new Set(report.rows.map(key)).size,expectedRows);
 const rows=report.rows.map(r=>{
  for(const runtime of ['js','control','candidate']){
   const values=r.samples[runtime];assert.equal(values.length,expectedTrials);
   assert.ok(values.every(x=>Number.isFinite(x)&&x>0));near(median(values),r.medianMs[runtime],`${key(r)} ${runtime} median`);
   near([...values].sort((a,b)=>a-b)[Math.max(0,Math.ceil(values.length*.95)-1)],r.p95BatchMeanMs[runtime],`${key(r)} ${runtime} p95 batch mean`);
  }
  assert.equal(r.counts.control,r.counts.candidate);
  near(r.medianMs.candidate/r.medianMs.control,r.candidateOverControl,`${key(r)} control ratio`);
  near(r.medianMs.candidate/r.medianMs.js,r.candidateOverJS,`${key(r)} JS ratio`);
  near(r.medianMs.control/r.medianMs.js,r.controlOverJS,`${key(r)} control/JS ratio`);
  const pairedPercent=r.samples.candidate.map((v,i)=>(v/r.samples.control[i]-1)*100);
  return {case:r.case,profile:r.profile,stage:r.stage,pairedMedianPercent:median(pairedPercent),lowerPairs:pairedPercent.filter(v=>v<0).length,totalPairs:expectedTrials,ratioOfMediansPercent:(r.candidateOverControl-1)*100,candidateOverJS:r.candidateOverJS,pairedMinimumPercent:Math.min(...pairedPercent),pairedMaximumPercent:Math.max(...pairedPercent)};
 });
 const stages={};for(const stage of [...new Set(rows.map(r=>r.stage))].sort()){
  const values=rows.filter(r=>r.stage===stage).map(r=>r.pairedMedianPercent);
  stages[stage]={rows:values.length,lowerMedianRows:values.filter(v=>v<0).length,medianPairedPercent:median(values),minimumPairedPercent:Math.min(...values),maximumPairedPercent:Math.max(...values)};
 }
 return {rows,stages,lowerMedianRows:rows.filter(r=>r.pairedMedianPercent<0).length,higherMedianRows:rows.filter(r=>r.pairedMedianPercent>0).length};
}
const comparison=await json('comparison.json'),first=await json('first-use.json');
const freeze=await json('package-freeze.json'),recheck=await json('package-local-recheck.json'),provenance=await json('provenance.json'),implementation=await json('implementation-manifest.json');
const stable=o=>JSON.stringify(Object.fromEntries(Object.entries(o).sort(([a],[b])=>a.localeCompare(b))));
for(const role of ['control','candidate']){
 const expected=stable(freeze.files[role]);
 for(const map of [comparison.runtimeHashes[role],first.runtimeHashes[role],recheck.hashes[role]])assert.equal(stable(map),expected,`${role} package identity`);
 for(const [path,digest] of Object.entries(provenance.sourcePins[role].files))assert.equal(freeze.files[role][path.replace('bindings/web/','')]??null,digest,`${role} source pin ${path}`);
}
const changed=[...new Set([...Object.keys(freeze.files.control),...Object.keys(freeze.files.candidate)])].filter(p=>freeze.files.control[p]!==freeze.files.candidate[p]).sort();
assert.deepEqual(changed,['graph-storage.js','graph.js','package.json']);assert.deepEqual([...freeze.changed].sort(),changed);
assert.deepEqual([...provenance.changedPackagePaths].sort(),changed);
assert.equal(freeze.files.control['pkg/pele_bg.wasm'],freeze.files.candidate['pkg/pele_bg.wasm']);assert.equal(provenance.wasmSha256,freeze.files.candidate['pkg/pele_bg.wasm']);
assert.equal(freeze.rustWasmSourceCommit,'365ec6d0df050199f9efb93d23d9ce7d352c60fa');assert.equal(recheck.measuredWasmSourceCommit,freeze.rustWasmSourceCommit);assert.equal(provenance.rustWasmSourceCommit,freeze.rustWasmSourceCommit);
assert.equal(provenance.sourcePins.candidate.commit,'e48ea33f65294f752fa5f3bee85c67611fb5db36');assert.equal(provenance.newNativeCoreIncluded,false);
assert.equal(freeze.patchSHA256,implementation.patchSha256);
for(const entry of implementation.files)assert.equal(entry.candidateSha256,freeze.files.candidate[entry.path.replace('bindings/web/','')]);
for(const [name,report] of [['compare-web-vm.mjs',comparison],['compare-first-use-web-vm.mjs',first]])assert.equal(hash(await bytes('harness/'+name)),report.harnessSHA256);
assert.equal(hash(await bytes('harness/cases.mjs')),comparison.casesSHA256);assert.equal(first.casesSHA256,comparison.casesSHA256);
assert.equal(first.oracleRevision,comparison.oracleRevision);assert.equal(first.oracleSHA256,comparison.oracleSHA256);
assert.equal(recheck.fullBrowserPassed,417);assert.equal(recheck.focusedBrowserPassed,54);
for(const entry of provenance.logs){const raw=gunzipSync(await bytes(entry.file));assert.equal(hash(raw),entry.uncompressedSha256);if(entry.passed!==null)assert.match(raw.toString(),new RegExp(`\\b${entry.passed} passed \\(`));}
const audit={schemaVersion:1,method:'Per row median of paired candidate/control percentages; conventional median across rows averages the two central values. Descriptive, no causal/noise/neutrality claim.',comparison:analyze(comparison,42,21),firstUse:analyze(first,40,15),sameWasmSha256:provenance.wasmSha256,changedPackagePaths:changed};
if(process.argv.includes('--write-audit')){await writeFile(new URL('audit.json',import.meta.url),JSON.stringify(audit,null,2)+'\n');console.log('Wrote audit; raw reports, package bindings, harness hashes and logs verified.');}
else{
 assert.deepEqual(await json('audit.json'),audit);
 const manifest=await json('bundle-manifest.json');
 for(const entry of manifest.files)assert.equal(hash(await bytes(entry.path)),entry.sha256,entry.path);
 console.log(JSON.stringify({verified:true,comparisonRows:42,firstUseRows:40,fullBrowserTests:417,focusedBrowserTests:54,wasmSha256:provenance.wasmSha256,comparison:audit.comparison.stages,firstUse:audit.firstUse.stages},null,2));
}
