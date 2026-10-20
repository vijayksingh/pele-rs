// Portable data-only verification. No subprocess, Git, SDK, build or test access.
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {dirname,join,relative} from 'node:path';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
import assert from 'node:assert/strict';
const root=dirname(fileURLToPath(import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const raw=async p=>readFile(join(root,p));
const bytes=async p=>p.endsWith('.gz')?gunzipSync(await raw(p)):raw(p);
const text=async p=>(await bytes(p)).toString('utf8');
const json=async p=>JSON.parse(await text(p));
const aggregate=files=>{const h=createHash('sha256');for(const[p,s]of Object.entries(files).sort(([a],[b])=>a.localeCompare(b)))h.update(p+'\0'+s+'\0');return h.digest('hex')};
const phases=['before-sdk','after-sdk','before-correctness','after-correctness'];
const records=await Promise.all(phases.map(p=>json('proof/'+p+'.json.gz')));
const [before,sdk,pre,post]=records,commit='3166b3dd18569ad0bc20f173a8941dd628609ddc';
const sourceKeys=Object.keys(before.current.files).filter(k=>!k.startsWith('CPele/'));
for(const [i,record]of records.entries()) {
 assert.equal(record.phase,phases[i]);assert.equal(record.commit,commit);
 assert.equal(Object.keys(record.current.files).length,362);assert.equal(aggregate(record.current.files),record.current.sha256);
 assert.deepEqual(record.extraHashes,before.extraHashes);assert.deepEqual(record.driverHashes,before.driverHashes);
 for(const key of sourceKeys)assert.equal(record.current.files[key],before.current.files[key],key);
}
const mapping=await json('proof/immutable-commit-mapping.json.gz');assert.equal(mapping.commit,commit);
assert.equal(sourceKeys.length,349);assert.equal(Object.keys(before.extraHashes).length,34);assert.equal(Object.keys(mapping.files).length,383);
assert.deepEqual(Object.keys(mapping.files).sort(),Object.keys(before.gitInputHashes).sort());
for(const [key,entry]of Object.entries(mapping.files)) {
 assert.equal(entry.path,before.gitInputPaths[key]);assert.equal(entry.sha256,before.gitInputHashes[key]);assert.match(entry.blob,/^[0-9a-f]{40}$/);assert(Number.isSafeInteger(entry.bytes)&&entry.bytes>=0);
 assert.equal(entry.sha256,key.startsWith('correctness/')?before.extraHashes[entry.path]:before.current.files[key]);
}
const changed=Object.keys(sdk.current.files).filter(k=>before.current.files[k]!==sdk.current.files[k]);
assert.deepEqual(changed,['CPele/macos-arm64/libpele_ffi.a']);
for(const record of[sdk,pre,post])assert.deepEqual(record.changedDuringSDK,changed);
assert.deepEqual(pre.current,sdk.current);assert.deepEqual(post.current,pre.current);
const manifestBytes=await bytes('proof/swift-build-manifest.json.gz'),manifest=JSON.parse(manifestBytes);
assert.deepEqual(pre.manifest,manifest);assert.deepEqual(post.manifest,manifest);
assert.deepEqual(manifest.inputHashes,post.current.files);assert.equal(manifest.sourceInputSha256,post.current.sha256);assert.equal(manifest.revision,commit);
for(const record of[sdk,pre,post])assert.equal(record.nativeArchiveSha256,manifest.nativeHostArchiveSha256);
for(const record of[pre,post]) {
 assert.equal(record.manifestSha256,sha(manifestBytes));assert.equal(record.binarySha256,manifest.binarySha256);
 assert.equal(record.current.files['CPele/macos-arm64/libpele_ffi.a'],record.nativeArchiveSha256);
}
assert.equal(manifest.rustTarget,'aarch64-apple-darwin');assert.equal(manifest.architecture,'arm64');
assert.equal(before.controlBinarySha256,before.controlManifest.binarySha256);
for(const name of['freeze.mjs','run-correctness.py'])assert.equal(sha(await raw('execution/'+name)),before.driverHashes['runtime/'+name]);
for(const name of['build.mjs','run.mjs'])assert.equal(sha(await raw('execution/swift-'+name)),manifest.inputHashes['consumer/'+name]);
const apiBytes=await bytes('proof/swift-api.json.gz'),api=JSON.parse(apiBytes);
assert.equal(api.mode,'swift-api-correctness-only');assert.equal(api.revision,commit);assert.deepEqual(api.buildManifest,manifest);
assert.equal(api.cases.length,21);assert.equal(api.rows.length,168);assert.equal(api.settings.stages.length,8);
const cases=new Map(api.cases.map(c=>[c.id,c]));assert.equal(cases.size,21);const conditions=new Set();
for(const c of cases.values()) {
 assert.equal(sha(Buffer.from(c.source)),c.sourceSha256);assert.equal(Buffer.byteLength(c.source),c.sourceBytes);
 for(const output of c.prepared.outputs)assert.deepEqual(output,c.output);
}
for(const row of api.rows) {
 const key=row.case+'/'+row.stage;assert(!conditions.has(key));conditions.add(key);assert(api.settings.stages.includes(row.stage));
 assert.equal(row.correctness.caseID,row.case);assert.equal(row.correctness.stage,row.stage);assert.equal(row.correctness.count,1);
 assert.deepEqual(row.correctness.output,cases.get(row.case).output);
 for(const forbidden of['samples','samplesMs','raw','medianMs'])assert(!(forbidden in row));
}
for(const c of cases.values())for(const stage of api.settings.stages)assert(conditions.has(c.id+'/'+stage));
const logPrefix='native-canonical-correctness-',corpusBytes=await bytes('logs/'+logPrefix+'swift-corpus.log.gz'),corpus=corpusBytes.toString();
assert.match(corpus,/Executed 16 tests, with 0 failures/);
const totals=corpus.match(/Swift mutable corpus: (\d+) drawings, (\d+) errors, (\d+) explicit budget differences/);
assert(totals);assert.deepEqual(totals.slice(1).map(Number),[12481,16572,0]);
assert.equal([...corpus.matchAll(/Test Case .* passed \(/g)].length,16);
const commands=await json('proof/correctness-commands.json.gz');
const expected=['freeze-before-sdk','sdk-build','freeze-after-sdk','swift-build','freeze-before-correctness','swift-corpus','ffi-compile','ffi-run','ffi-public-compile','ffi-public-run','ffi-parse-limits-compile','ffi-parse-limits-run','swift-api','freeze-after-correctness'];
assert.deepEqual(commands.map(c=>c.phase),expected);
for(const row of commands) {
 assert.equal(row.exitCode,0,row.phase);assert(Date.parse(row.endedAt)>=Date.parse(row.startedAt));
 const log='logs/'+row.log.split('/').at(-1)+'.gz';await bytes(log);
}
assert.deepEqual(commands.find(c=>c.phase==='sdk-build').command,['bash','scripts/build-native.sh','--host']);
assert.equal(commands.find(c=>c.phase==='sdk-build').environmentOverrides.CARGO_BUILD_JOBS,'2');
assert.deepEqual(commands.find(c=>c.phase==='swift-corpus').environmentOverrides,{PELE_VERIFY_MUTABLE:'1',PELE_FAMILIES:'er,sankey'});
assert(!commands.find(c=>c.phase==='swift-api').command.includes('--timed'));
const c=await json('proof/c-consumers.json.gz');assert.equal(c.length,6);
for(const row of c)assert.deepEqual(row,commands.find(v=>v.phase===row.phase));
for(const name of['ffi','ffi-public','ffi-parse-limits']) {
 const compile=c.find(v=>v.phase===name+'-compile');assert(compile);assert.equal(compile.command[0],'cc');
 for(const flag of['-Wall','-Wextra','-Werror','-std=c11'])assert(compile.command.includes(flag));
 assert(compile.command.includes('bindings/swift/Artifacts/CPele.xcframework/macos-arm64/libpele_ffi.a'));
 assert(c.some(v=>v.phase===name+'-run'&&v.exitCode===0));assert.match(post.correctness.cBinaryHashes[name],/^[0-9a-f]{64}$/);
}
assert.match(await text('logs/'+logPrefix+'sdk-build.log.gz'),/xcframework successfully written out/);
assert.match(await text('logs/'+logPrefix+'swift-build.log.gz'),/Build complete!/);
assert.equal(post.correctness.apiReportSha256,sha(apiBytes));assert.equal(post.correctness.swiftLogSha256,sha(corpusBytes));
for(const[key,value]of Object.entries({swiftTests:16,drawings:12481,expectedErrors:16572,budgetDifferences:0,apiCases:21,apiConditions:168,cConsumers:3,timing:false}))assert.equal(post.correctness[key],value);
const attempt=await json('proof/attempt-started.json.gz');assert.equal(attempt.commit,commit);assert.equal(attempt.timing,false);
const provenance=await json('provenance.json');assert.equal(provenance.sourceCommit,commit);
for(const[p,o]of Object.entries(provenance.origins)){const b=await bytes(p);assert.equal(sha(b),o.sourceSha256,p);assert.equal(b.length,o.sourceBytes);}
const summary={scope:'default native/Swift/C correctness only; no latency or opt-in browser-backend proof',sourceCommit:commit,
 swift:{tests:16,drawings:Number(totals[1]),expectedErrors:Number(totals[2]),explicitBudgetDifferences:Number(totals[3]),apiCases:cases.size,apiConditions:conditions.size,cConsumers:3},
 freshness:{inputs:362,gitMappedBuildInputs:sourceKeys.length,additionalGitMappedCorrectnessInputs:Object.keys(before.extraHashes).length,sourceInputSha256:manifest.sourceInputSha256,binarySha256:manifest.binarySha256,nativeHostArchiveSha256:manifest.nativeHostArchiveSha256,changedDuringSDK:changed,unchangedThroughCorrectness:true,previousControlCommit:before.controlManifest.revision,previousControlBinarySha256:before.controlBinarySha256},
 limitations:['This proves default native/FFI/Swift consumers, not the opt-in native-canonical WASM/JS owner.','No browser-suite result, latency, Intel SDK, release or complete JS parity claim.','Binaries and source trees are omitted. Portable verification checks captured records, output checks and hashes; it is not a rebuild or fresh binary rehash.','Command durations are correctness execution metadata and are not performance measurements.']};
if(process.argv.includes('--write-summary'))await writeFile(join(root,'summary.json'),JSON.stringify(summary,null,2)+'\n');
else assert.deepEqual(await json('summary.json'),summary);
if(!process.argv.includes('--write-summary')) {
 const index=await json('evidence-index.json');for(const[p,h]of Object.entries(index.files))assert.equal(sha(await raw(p)),h,p);
 const all=[];async function walk(dir){for(const e of await readdir(dir,{withFileTypes:true})){const p=join(dir,e.name);if(e.isDirectory())await walk(p);else all.push(relative(root,p));}}await walk(root);
 assert.deepEqual(all.filter(p=>p!=='evidence-index.json').sort(),Object.keys(index.files).sort());
}
console.log(JSON.stringify(summary,null,2));
