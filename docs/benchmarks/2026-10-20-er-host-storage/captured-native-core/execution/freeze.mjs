// Read-only source/SDK/consumer proof; only writes private evidence.
import {readFile,writeFile,mkdir,copyFile,chmod} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {inputs,sha} from '../../../../../scripts/perf/swift/build.mjs';
const [phase,commit]=process.argv.slice(2);assert(['before-sdk','after-sdk','before-correctness','after-correctness'].includes(phase));assert(/^[0-9a-f]{40}$/.test(commit));
const dir='dist/perf-patches/parse-architecture/captured-native-core',root=process.cwd(),current=await inputs(root);
const head=execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim();assert.equal(head,commit,'HEAD changed before/through consolidated check');
const record={phase,recordedAt:new Date().toISOString(),commit,current};
if(phase==='before-sdk') {
 const mapping=Object.fromEntries(Object.keys(current.files).filter(k=>k.startsWith('root/')||k.startsWith('consumer/')).map(k=>[k,k.startsWith('root/')?k.slice(5):'scripts/perf/swift/'+k.slice(9)]));
 const keys=Object.keys(mapping),bytes=execFileSync('git',['cat-file','--batch'],{input:keys.map(k=>commit+':'+mapping[k]+'\n').join(''),maxBuffer:100*1024*1024});let offset=0;const hashes={};
 for(const key of keys){const end=bytes.indexOf(10,offset);const header=bytes.subarray(offset,end).toString().split(' ');assert.equal(header[1],'blob');const size=Number(header[2]);hashes[key]=sha(bytes.subarray(end+1,end+1+size));assert.equal(hashes[key],current.files[key],key+' differs from committed source');offset=end+size+2;}
 const controlDir='dist/perf-patches/parse-architecture/arena-render-packet/candidate-swift';const controlManifest=JSON.parse(await readFile(controlDir+'/swift-build-manifest.json','utf8'));
 assert.equal(sha(await readFile(controlDir+'/PeleSwiftPerf')),'2e1491d5b80da5d64b7f042abe934f47d662901d9d8037eb1646a0d7a03cff9a');
 Object.assign(record,{gitInputHashes:hashes,controlDir,controlManifest,changedInputsBeforeSDK:[...new Set([...Object.keys(current.files),...Object.keys(controlManifest.inputHashes)])].filter(k=>current.files[k]!==controlManifest.inputHashes[k])});
} else {
 const before=JSON.parse(await readFile(dir+'/runtime/before-sdk.json','utf8'));assert.equal(before.commit,commit);
 const allKeys=[...new Set([...Object.keys(current.files),...Object.keys(before.current.files)])];const changed=allKeys.filter(k=>current.files[k]!==before.current.files[k]);assert(changed.every(k=>k.startsWith('CPele/')),'Source drift during SDK/consumer/correctness');
 record.changedDuringSDK=changed;record.nativeArchiveSha256=sha(await readFile('target/aarch64-apple-darwin/release/libpele_ffi.a'));assert.equal(current.files['CPele/macos-arm64/libpele_ffi.a'],record.nativeArchiveSha256);
 if(phase!=='after-sdk') {
  const manifest=JSON.parse(await readFile('scripts/perf/swift/.build/swift-build-manifest.json','utf8'));assert.equal(manifest.sourceInputSha256,current.sha256);record.manifest=manifest;record.binarySha256=sha(await readFile(manifest.binary));assert.equal(record.binarySha256,manifest.binarySha256);assert.equal(record.nativeArchiveSha256,manifest.nativeHostArchiveSha256);
  if(phase==='before-correctness'){await mkdir(dir+'/candidate-swift',{recursive:true});await copyFile(manifest.binary,dir+'/candidate-swift/PeleSwiftPerf');await chmod(dir+'/candidate-swift/PeleSwiftPerf',0o755);await writeFile(dir+'/candidate-swift/swift-build-manifest.json',JSON.stringify(manifest,null,2)+'\n');}
  else {const prior=JSON.parse(await readFile(dir+'/runtime/before-correctness.json','utf8'));assert.deepEqual(current,prior.current);assert.equal(record.binarySha256,prior.binarySha256);assert.equal(sha(await readFile(dir+'/candidate-swift/PeleSwiftPerf')),record.binarySha256);const api=JSON.parse(await readFile('dist/evidence/captured-native-core-swift-api.json','utf8'));assert.equal(api.mode,'swift-api-correctness-only');assert.equal(api.rows.length,168);assert.equal(api.cases.length,21);assert.deepEqual(api.buildManifest,manifest);const corpus=await readFile('dist/evidence/captured-native-core-swift-corpus.log','utf8');assert(corpus.includes('Executed 16 tests, with 0 failures'));assert(corpus.includes('12481 drawings, 16572 errors, 0 explicit budget differences'));record.correctness={swiftTests:16,drawings:12481,expectedErrors:16572,budgetDifferences:0,apiCases:21,apiConditions:168,cConsumers:3,timing:false};}
 }
 assert.equal(sha(await readFile(before.controlDir+'/PeleSwiftPerf')),before.controlManifest.binarySha256,'Frozen arena control changed');
}
await writeFile(dir+'/runtime/'+phase+'.json',JSON.stringify(record,null,2)+'\n');console.log(JSON.stringify({phase,commit,inputCount:Object.keys(current.files).length,sourceInputSha256:current.sha256,binarySha256:record.binarySha256,nativeArchiveSha256:record.nativeArchiveSha256,correctness:record.correctness}));
