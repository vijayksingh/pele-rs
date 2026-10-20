// Portable captured-data verification only. No subprocess, build, benchmark, Git or network.
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {dirname,join,relative} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
import assert from 'node:assert/strict';
const root=dirname(fileURLToPath(import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const raw=p=>readFile(join(root,p));
const bytes=async p=>p.endsWith('.gz')?gunzipSync(await raw(p)):raw(p);
const json=async p=>JSON.parse((await bytes(p)).toString());
const aggregate=files=>sha(Object.entries(files).sort(([a],[b])=>a.localeCompare(b)).map(([p,h])=>p+'\0'+h+'\0').join(''));
const quantile=(xs,p)=>[...xs].sort((a,b)=>a-b)[Math.max(0,Math.ceil(xs.length*p)-1)];
const stats=xs=>{const median=quantile(xs,.5);return {median,mad:quantile(xs.map(x=>Math.abs(x-median)),.5),min:Math.min(...xs),max:Math.max(...xs),sampleP95:quantile(xs,.95)}};
const stages=['parseMutable','parse','render','lifetime'];
const schedule=await json('proof/schedule.json.gz'),prepared=await json('proof/prepared-manifest.json.gz'),continuity=await json('proof/swift-input-continuity.json.gz');
assert.equal(schedule.succeeded,true);assert.equal(schedule.budgetMs,180000);assert(schedule.elapsedWallMs<schedule.budgetMs);
assert.equal(schedule.strictGuardUnchanged,true);assert.equal(schedule.retries,0);assert.equal(schedule.pooling,false);
assert.deepEqual(schedule.stages,stages);assert.deepEqual(schedule.acceptedConditions,stages);assert.deepEqual(schedule.unmeasuredConditions,[]);assert.equal(schedule.conditions.length,4);
assert.deepEqual(schedule.acceptedOrderCounts,{control:30,candidate:30});
for(const[file,key]of [['execution/bounded-swift-timing.mjs.gz','schedulerSha256'],['execution/compare.mjs.gz','comparatorSha256'],['execution/build.mjs.gz','guardModuleSha256'],['proof/swift-input-continuity.json.gz','continuitySha256']]){
 assert.equal(sha(await bytes(file)),schedule[key]);assert.equal(schedule[key],prepared[key]);
}
const manifests={};for(const side of ['control','candidate']){
 const b=await bytes(`proof/${side}-build-manifest.json.gz`),m=JSON.parse(b);manifests[side]=m;
 assert.equal(sha(b),prepared[side].manifestSha256);assert.equal(aggregate(m.inputHashes),m.sourceInputSha256);
 for(const key of ['revision','binarySha256','sourceInputSha256','nativeHostArchiveSha256'])assert.equal(m[key],prepared[side][key]);
 assert.equal(m.inputHashes['CPele/macos-arm64/libpele_ffi.a'],m.nativeHostArchiveSha256);
 assert.equal(schedule.frozen[side+'BinarySha256'],m.binarySha256);assert.equal(schedule.frozen[side+'ManifestSha256'],sha(b));
}
assert.equal(manifests.control.revision,'25d87cb49bb451d3e7c1e70c2856ccb6fef4abd0');assert.equal(manifests.candidate.revision,'3166b3dd18569ad0bc20f173a8941dd628609ddc');
const before=await json('proof/source-before.json.gz'),after=await json('proof/source-after.json.gz');assert.deepEqual(before,after);assert.equal(Object.keys(before.files).length,362);assert.equal(aggregate(before.files),before.sha256);assert.deepEqual(before.files,manifests.candidate.inputHashes);
assert.equal(continuity.unchanged,true);assert.deepEqual(continuity.current,before);assert.equal(continuity.validatedSDKCommit,manifests.candidate.revision);assert.equal(schedule.frozen.sourceInputSha256,before.sha256);
const changed=[...new Set([...Object.keys(manifests.control.inputHashes),...Object.keys(manifests.candidate.inputHashes)])].sort().filter(p=>manifests.control.inputHashes[p]!==manifests.candidate.inputHashes[p]).map(p=>({path:p,control:manifests.control.inputHashes[p]??null,candidate:manifests.candidate.inputHashes[p]??null}));
let expected,caseSource;const rows=[],orders={control:0,candidate:0};
for(const[index,stage]of stages.entries()){
 const dataBytes=await bytes(`reports/${stage}.json.gz`),d=JSON.parse(dataBytes),c=schedule.conditions[index];
 assert.equal(sha(dataBytes),c.reportSha256);assert.equal(c.stage,stage);assert.equal(c.orderOffset,index%2);assert.equal(c.accepted,true);assert.equal(c.exitCode,0);assert.equal(c.signal,null);assert.equal(c.timedOut,false);assert(c.command.includes('--quiet-run'));
 assert.equal(d.completed,true);assert.equal(d.rows.length,1);assert.equal(d.cases.length,1);assert.equal(d.comparatorSha256,schedule.comparatorSha256);assert.deepEqual(d.changedInputs,changed);
 for(const side of ['control','candidate']){assert.deepEqual(d.builds[side].manifest,manifests[side]);assert.equal(d.builds[side].binarySha256,prepared[side].binarySha256);assert.equal(d.builds[side].manifestSha256,prepared[side].manifestSha256);}
 const s=d.settings;assert.equal(s.trials,15);assert.equal(s.orderOffset,index%2);assert.equal(s.warmupMsPerBinaryPerStage,1000);assert.equal(s.warmupMaximumBatch,128);assert.equal(s.matchedCounts,true);assert.equal(s.sequentialTiming,true);assert.equal(s.freshRetainedStatePerBatch,true);assert.deepEqual(s.stages,[stage]);assert.deepEqual(s.selectedCases,['sankey/medium']);
 const fixture=d.cases[0];assert.equal(sha(fixture.source),fixture.sourceSha256);assert.equal(Buffer.byteLength(fixture.source),fixture.sourceBytes);
 if(expected){assert.deepEqual(fixture.expected,expected);assert.equal(fixture.source,caseSource);}else{expected=fixture.expected;caseSource=fixture.source;}
 assert.deepEqual(fixture.before.control,fixture.before.candidate);assert.deepEqual(fixture.before,fixture.after);for(const output of fixture.before.control.outputs)assert.deepEqual(output,expected);
 const r=d.rows[0];assert.equal(r.case,'sankey/medium');assert.equal(r.stage,stage);assert.equal(r.raw.length,15);assert.equal(r.warmup.control.operations,r.warmup.candidate.operations);
 for(const side of ['control','candidate'])assert(r.warmup[side].elapsedNs>=1e9);
 function pair(p){const a={...p.values.control},b={...p.values.candidate};delete a.elapsedNs;delete b.elapsedNs;assert.deepEqual(a,b);assert.deepEqual(a.output,expected);assert.equal(a.caseID,'sankey/medium');assert.equal(a.stage,stage);}
 pair(r.before);pair(r.after);assert.equal(r.before.values.control.count,1);assert.equal(r.after.values.control.count,r.count);
 const control=[],candidate=[],local={control:0,candidate:0};
 for(const[t,p]of r.raw.entries()){
  assert.equal(p.trial,t);assert.deepEqual(p.order,(t+index)%2===0?['control','candidate']:['candidate','control']);orders[p.order[0]]++;local[p.order[0]]++;pair(p);
  for(const side of ['control','candidate']){const v=p.values[side];assert.equal(v.count,r.count);assert(Number.isSafeInteger(v.elapsedNs)&&v.elapsedNs>0);assert(p.guards[side].at);assert(Array.isArray(p.guards[side].externalLoad));}
  control.push(p.values.control.elapsedNs/r.count/1e6);candidate.push(p.values.candidate.elapsedNs/r.count/1e6);
 }
 assert.deepEqual(local,d.orderCounts);assert.deepEqual(local,c.orderCounts);
 const ratios=candidate.map((v,i)=>v/control[i]);const calculated={pairedCandidateOverControl:stats(ratios),ratioOfMedians:quantile(candidate,.5)/quantile(control,.5),candidateFasterPairs:ratios.filter(v=>v<1).length,pairedPercentChanges:ratios.map(v=>(v-1)*100),controlMs:control,candidateMs:candidate,ratios};assert.deepEqual(r.statistics,calculated);
 rows.push({case:r.case,stage,pairs:15,count:r.count,controlMedianMs:quantile(control,.5),candidateMedianMs:quantile(candidate,.5),pairedMedianPercent:(calculated.pairedCandidateOverControl.median-1)*100,ratioOfMedians:calculated.ratioOfMedians,candidateLowerPairs:calculated.candidateFasterPairs,pairedPercentRange:[Math.min(...calculated.pairedPercentChanges),Math.max(...calculated.pairedPercentChanges)],orderCounts:local});
}
assert.deepEqual(orders,schedule.acceptedOrderCounts);assert(schedule.hostProcessesBefore.length>0);assert(schedule.hostProcessesAfter.length>0);
const provenance=await json('provenance.json');for(const[p,o]of Object.entries(provenance.origins)){const b=await bytes(p);assert.equal(sha(b),o.sourceSha256);assert.equal(b.length,o.sourceBytes);}
const summary={scope:'One shared-host default-native Swift attempt; no speedup, neutrality, WASM or release claim',control:prepared.control,candidate:prepared.candidate,continuityCommit:continuity.currentCommit,sourceInputCount:362,changedInputs:changed,elapsedWallMs:schedule.elapsedWallMs,completeConditions:4,pairs:60,orderCounts:orders,warmupMsPerSidePerCondition:1000,retries:0,pooling:false,rows,limitations:['One Sankey medium fixture, four separate complete condition windows; no cross-window compounding.','Render paired median is adverse. Lifetime has large bidirectional outliers, all retained; these data do not establish neutrality or a causal speedup.','Strict global compiler/Pele-test guard was unchanged. Passing its checks does not imply an idle or load-free host; process and per-pair load records are retained.','Control25d87cb to candidate3166b3d includes the recorded native source and SDK delta; it does not isolate the opt-in WASM native-canonical backend. JS-only cursor5db9f35 has the same native inputs.','Archived executables, SDKs and source trees are omitted. Portable verification checks recorded identities and arithmetic, not a fresh rebuild or physical binary rehash.','No retries, sample exclusions, pooling, additional timings or statistical equivalence test.']};
if(process.argv.includes('--write-summary'))await writeFile(join(root,'summary.json'),JSON.stringify(summary,null,2)+'\n');else assert.deepEqual(await json('summary.json'),summary);
if(!process.argv.includes('--write-summary')){const index=await json('evidence-index.json');for(const[p,h]of Object.entries(index.files))assert.equal(sha(await raw(p)),h,p);const all=[];async function walk(dir){for(const e of await readdir(dir,{withFileTypes:true})){const p=join(dir,e.name);if(e.isDirectory())await walk(p);else all.push(relative(root,p));}}await walk(root);assert.deepEqual(all.filter(p=>p!=='evidence-index.json').sort(),Object.keys(index.files).sort());}
console.log(JSON.stringify({verified:true,conditions:rows.length,pairs:60,orderCounts:orders,rows},null,2));
