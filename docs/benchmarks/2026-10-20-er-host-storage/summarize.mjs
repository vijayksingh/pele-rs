import {readFile} from 'node:fs/promises';
import assert from 'node:assert/strict';
const read=async path=>JSON.parse(await readFile(new URL(path,import.meta.url)));
const median=values=>[...values].sort((a,b)=>a-b)[Math.floor(values.length/2)];
const report=await read('inline/comparison.json');
const freeze=await read('inline/source-freeze.json');
assert.equal(report.succeeded,true);
assert.equal(report.rows.length,30);
assert.deepEqual(report.runtimeHashes.candidate,freeze.packageHashes);
assert.deepEqual((await read('inline/profile/summary.json')).runtimeHashes,freeze.packageHashes);
const rows=report.rows.map(row=>{
  const ratios=row.samples.candidate.map((value,index)=>value/row.samples.control[index]);
  assert.equal(ratios.length,21);
  return {case:row.case,stage:row.stage,profile:row.profile,pairedPercent:(median(ratios)-1)*100,
    faster:ratios.filter(value=>value<1).length,trials:ratios.length,
    ratioOfMedians:row.candidateOverControl,wasmOverJS:row.candidateOverJS};
});
assert.deepEqual((await read('inline/paired-summary.json')).rows,rows);
const swift=await read('inline/swift-comparison-attempt2.json');
assert.equal(swift.completed,true);
assert.equal(swift.builds.candidate.manifest.sourceInputSha256,freeze.sourceInputSha256);
assert.equal(swift.builds.candidate.binarySha256,freeze.swiftBinarySha256);
assert.deepEqual(swift.orderCounts,{control:45,candidate:45});
assert.equal((await read('inline/swift-comparison-attempt1.json')).completed,false);
for(const row of swift.rows){
  assert.equal(row.statistics.ratios.length,15);
  assert.equal(median(row.statistics.ratios),row.statistics.pairedCandidateOverControl.median);
}
const native=await read('inline/native-comparison-attempt2.json');
assert.equal(native.succeeded,true);
assert.equal(native.rows.length,8);
assert.equal(native.artifacts.candidate.binarySHA256,
  (await read('inline/native-build-manifest.json')).binaries['pele-perf']);
console.log(JSON.stringify({web:rows,native:native.rows.map(row=>({case:row.case,stage:row.stage,
  pairedPercent:(median(row.samples.candidate.map((value,index)=>value/row.samples.control[index]))-1)*100,
  faster:row.samples.candidate.filter((value,index)=>value<row.samples.control[index]).length})),
  swift:swift.rows.map(row=>({case:row.case,stage:row.stage,
  pairedPercent:(row.statistics.pairedCandidateOverControl.median-1)*100,
  faster:row.statistics.candidateFasterPairs}))},null,2));
