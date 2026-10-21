// Read-only native/SDK/consumer verification. No build or consumer execution.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {inputs,sha} from '../../../../scripts/perf/swift/build.mjs';
const phase=process.argv[2];assert(['before','after'].includes(phase));
const dir='dist/perf-patches/parse-architecture/disposal-receipt-runtime';
const original='dist/perf-patches/parse-architecture/source-symbols-correctness';
const proof=JSON.parse(await readFile(original+'/runtime/after-correctness.json'));
assert.equal(proof.commit,'11f27516d98c3d89799a7fe2065eec0bd4e147cf');
const current=await inputs(process.cwd());assert.deepEqual(current,proof.current,'Native/SDK inputs changed: fresh build would be required');
assert.equal(Object.keys(current.files).length,370);
const manifestBytes=await readFile('scripts/perf/swift/.build/swift-build-manifest.json');
const manifest=JSON.parse(manifestBytes);assert.deepEqual(manifest,proof.manifest);
assert.equal(sha(manifestBytes),proof.manifestSha256);
const binarySHA256=sha(await readFile(manifest.binary));assert.equal(binarySHA256,proof.binarySha256);
const sdkSHA256=sha(await readFile('bindings/swift/Artifacts/CPele.xcframework/macos-arm64/libpele_ffi.a'));assert.equal(sdkSHA256,proof.nativeArchiveSha256);
assert.equal(sha(await readFile(original+'/candidate-swift/PeleSwiftPerf')),binarySHA256);
assert.equal(sha(await readFile(original+'/candidate-swift/swift-build-manifest.json')),proof.manifestSha256);
const record={recordedAt:new Date().toISOString(),phase,revision:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),nativeProofSource:proof.commit,nativeInputCount:Object.keys(current.files).length,nativeInputSHA256:current.sha256,unchangedNativeInputs:true,sdkSHA256,consumerSHA256:binarySHA256,consumerManifestSHA256:proof.manifestSha256,originalProofSHA256:sha(await readFile(original+'/runtime/after-correctness.json')),scope:'Native/SDK/consumer identities reuse fresh11f correctness. No build or local timing. API correctness is separate.'};
if(phase==='after'){
 const before=JSON.parse(await readFile(dir+'/native-proof-before.json'));
 for(const key of ['revision','nativeProofSource','nativeInputCount','nativeInputSHA256','sdkSHA256','consumerSHA256','consumerManifestSHA256'])assert.deepEqual(record[key],before[key]);
 const apiBytes=await readFile('dist/evidence/disposal-receipt-swift-api.json');const api=JSON.parse(apiBytes);
 assert.equal(api.mode,'swift-api-correctness-only');assert.equal(api.rows.length,168);assert.equal(api.cases.length,21);assert.deepEqual(api.buildManifest,manifest);
 record.freshSwiftAPICorrectness={conditions:168,cases:21,reportSHA256:sha(apiBytes)};
}
await writeFile(dir+'/native-proof-'+phase+'.json',JSON.stringify(record,null,2)+'\n',{flag:'wx'});
if(phase==='after')await writeFile(dir+'/native-proof-reuse.json',JSON.stringify(record,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify(record));
