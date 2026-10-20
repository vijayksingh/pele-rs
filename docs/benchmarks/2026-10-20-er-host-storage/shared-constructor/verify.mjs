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
const provenance=await json('provenance.json');
assert.equal(provenance.wasmSourceCommit,'365ec6d0df050199f9efb93d23d9ce7d352c60fa');
assert.equal(provenance.newNativeCoreIncluded,false);
const stable=o=>JSON.stringify(Object.fromEntries(Object.entries(o).sort(([a],[b])=>a.localeCompare(b))));
const analyses={},reports={};let firstControl;
for(const variant of ['v1','v2']){
 const report=await json(variant+'/comparison.json'),freeze=await json(variant+'/package-freeze.json');reports[variant]=report;
 analyses[variant]=analyze(report,18,21);
 for(const role of ['control','candidate'])assert.equal(stable(freeze.hashes[role]),stable(report.runtimeHashes[role]));
 const changed=[...new Set([...Object.keys(freeze.hashes.control),...Object.keys(freeze.hashes.candidate)])].filter(p=>freeze.hashes.control[p]!==freeze.hashes.candidate[p]).sort();
 assert.deepEqual(changed,['constructor-storage.js','flat-constructor.js','host-er-store.js','package.json']);
 assert.deepEqual([...freeze.changed].sort(),changed);
 assert.equal(freeze.hashes.control['pkg/pele_bg.wasm'],freeze.hashes.candidate['pkg/pele_bg.wasm']);
 assert.equal(provenance.variants[variant].wasmSha256,freeze.hashes.candidate['pkg/pele_bg.wasm']);
 if(firstControl)assert.equal(stable(freeze.hashes.control),firstControl);else firstControl=stable(freeze.hashes.control);
 assert.equal(hash(await bytes(variant+'/source.patch')),provenance.variants[variant].sourcePatchSha256);
 for(const check of provenance.variants[variant].syntax){assert.equal(check.exitCode,0);assert.equal(check.sha256,freeze.hashes.candidate[check.path]);}
 assert.equal(hash(await bytes('harness/compare-web-vm.mjs')),report.harnessSHA256);
 assert.equal(hash(await bytes('harness/cases.mjs')),report.casesSHA256);
}
assert.equal(reports.v1.oracleRevision,reports.v2.oracleRevision);assert.equal(reports.v1.oracleSHA256,reports.v2.oracleSHA256);
for(const entry of provenance.logs){const raw=gunzipSync(await bytes(entry.file));assert.equal(hash(raw),entry.uncompressedSha256);if(entry.passed!==null)assert.match(raw.toString(),new RegExp(`\\b${entry.passed} passed \\(`));}
const audit={schemaVersion:1,method:'Per row median paired percentage; conventional median across rows averages central two. Descriptive, not a direct V1/V2 causal comparison, neutrality or speedup claim.',...analyses};
if(process.argv.includes('--write-audit')){await writeFile(new URL('audit.json',import.meta.url),JSON.stringify(audit,null,2)+'\n');console.log('Verified and wrote audit.');}
else{assert.deepEqual(await json('audit.json'),audit);const manifest=await json('bundle-manifest.json');for(const entry of manifest.files)assert.equal(hash(await bytes(entry.path)),entry.sha256,entry.path);console.log(JSON.stringify({verified:true,rows:36,trialsPerRow:21,browserPassedEach:57,v1:audit.v1.stages,v2:audit.v2.stages},null,2));}
