// Evidence only. Importing build.mjs does not invoke its build entrypoint.
import {readFile,writeFile,mkdir,copyFile,chmod,constants} from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {inputs,sha,files} from '../../../../../scripts/perf/swift/build.mjs';
const [phase,commit]=process.argv.slice(2);
assert(['before-sdk','after-sdk','before-correctness','after-correctness'].includes(phase));
assert(/^[0-9a-f]{40}$/.test(commit));
const dir='dist/perf-patches/parse-architecture/native-canonical-correctness',prefix='native-canonical-correctness';
const root=process.cwd(),current=await inputs(root),head=execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim();
assert.equal(head,commit,'HEAD changed before/through consolidated check');
assert.equal(process.platform,'darwin');assert.equal(process.arch,'arm64');
const extraPaths=[...files(root,'bindings/swift/Tests'),'bindings/swift/Package.swift','tests/ffi.c','tests/ffi-public.c','tests/ffi-parse-limits.c'].sort();
const extraHashes=Object.fromEntries(await Promise.all(extraPaths.map(async p=>[p,sha(await readFile(p))])));
const driverPaths=['runtime/run-correctness.py','runtime/freeze.mjs'];
const driverHashes=Object.fromEntries(await Promise.all(driverPaths.map(async p=>[p,sha(await readFile(dir+'/'+p))])));
const record={phase,recordedAt:new Date().toISOString(),commit,current,extraHashes,driverHashes};
if(phase==='before-sdk') {
 const mapping=Object.fromEntries(Object.keys(current.files).filter(k=>k.startsWith('root/')||k.startsWith('consumer/')).map(k=>[k,k.startsWith('root/')?k.slice(5):'scripts/perf/swift/'+k.slice(9)]));
 for(const path of extraPaths)mapping['correctness/'+path]=path;
 const keys=Object.keys(mapping),bytes=execFileSync('git',['cat-file','--batch'],{input:keys.map(k=>commit+':'+mapping[k]+'\n').join(''),maxBuffer:128*1024*1024});
 let offset=0;const hashes={};
 for(const key of keys){const end=bytes.indexOf(10,offset);assert(end>=offset);const header=bytes.subarray(offset,end).toString().split(' ');assert.equal(header[1],'blob',mapping[key]+' not committed');const size=Number(header[2]);hashes[key]=sha(bytes.subarray(end+1,end+1+size));assert.equal(hashes[key],key.startsWith('correctness/')?extraHashes[mapping[key]]:current.files[key],key+' differs from committed source');offset=end+size+2;}
 const controlDir='dist/perf-patches/parse-architecture/captured-native-core/candidate-swift';
 const controlManifestBytes=await readFile(controlDir+'/swift-build-manifest.json');
 const controlManifest=JSON.parse(controlManifestBytes);
 const controlBinarySha256=sha(await readFile(controlDir+'/PeleSwiftPerf'));
 assert.equal(controlBinarySha256,'9c1d267539a7eaa1b94243dc64cc1dabfafbd0c795a73dd265d5b3f843d56888');
 assert.equal(controlBinarySha256,controlManifest.binarySha256);
 Object.assign(record,{gitInputHashes:hashes,gitInputPaths:mapping,controlDir,controlManifest,controlManifestSha256:sha(controlManifestBytes),controlBinarySha256,
 changedInputsBeforeSDK:[...new Set([...Object.keys(current.files),...Object.keys(controlManifest.inputHashes)])].filter(k=>current.files[k]!==controlManifest.inputHashes[k])});
} else {
 const before=JSON.parse(await readFile(dir+'/runtime/before-sdk.json','utf8'));assert.equal(before.commit,commit);
 assert.deepEqual(extraHashes,before.extraHashes,'Correctness fixtures changed');assert.deepEqual(driverHashes,before.driverHashes,'Driver changed during execution');
 const keys=[...new Set([...Object.keys(current.files),...Object.keys(before.current.files)])];
 const changed=keys.filter(k=>current.files[k]!==before.current.files[k]);assert(changed.every(k=>k.startsWith('CPele/')),'Source drift during SDK/consumer/correctness');
 record.changedDuringSDK=changed;
 record.nativeArchiveSha256=sha(await readFile('target/aarch64-apple-darwin/release/libpele_ffi.a'));
 assert.equal(current.files['CPele/macos-arm64/libpele_ffi.a'],record.nativeArchiveSha256);
 if(phase!=='after-sdk') {
  const manifestBytes=await readFile('scripts/perf/swift/.build/swift-build-manifest.json');const manifest=JSON.parse(manifestBytes);
  assert.equal(manifest.revision,commit);assert.equal(manifest.sourceInputSha256,current.sha256);assert.deepEqual(manifest.inputHashes,current.files);
  record.manifest=manifest;record.manifestSha256=sha(manifestBytes);record.binarySha256=sha(await readFile(manifest.binary));
  assert.equal(record.binarySha256,manifest.binarySha256);assert.equal(record.nativeArchiveSha256,manifest.nativeHostArchiveSha256);
  if(phase==='before-correctness') {
   const afterSDK=JSON.parse(await readFile(dir+'/runtime/after-sdk.json','utf8'));assert.deepEqual(current,afterSDK.current,'Consumer build changed SDK/source');
   await mkdir(dir+'/candidate-swift');
   await copyFile(manifest.binary,dir+'/candidate-swift/PeleSwiftPerf',constants.COPYFILE_EXCL);await chmod(dir+'/candidate-swift/PeleSwiftPerf',0o755);
   await writeFile(dir+'/candidate-swift/swift-build-manifest.json',manifestBytes,{flag:'wx'});
  } else {
   const prior=JSON.parse(await readFile(dir+'/runtime/before-correctness.json','utf8'));assert.deepEqual(current,prior.current);assert.equal(record.binarySha256,prior.binarySha256);assert.equal(record.manifestSha256,prior.manifestSha256);
   assert.equal(sha(await readFile(dir+'/candidate-swift/PeleSwiftPerf')),record.binarySha256);
   assert.equal(sha(await readFile(dir+'/candidate-swift/swift-build-manifest.json')),record.manifestSha256);
   const apiBytes=await readFile('dist/evidence/'+prefix+'-swift-api.json');const api=JSON.parse(apiBytes);
   assert.equal(api.mode,'swift-api-correctness-only');assert.equal(api.rows.length,168);assert.equal(api.cases.length,21);assert.deepEqual(api.buildManifest,manifest);
   const corpusBytes=await readFile('dist/evidence/'+prefix+'-swift-corpus.log');const corpus=corpusBytes.toString();
   assert(corpus.includes('Executed 16 tests, with 0 failures'));assert(corpus.includes('12481 drawings, 16572 errors, 0 explicit budget differences'));
   const commands=JSON.parse(await readFile(dir+'/runtime/correctness-commands.json','utf8'));
   const c=JSON.parse(await readFile(dir+'/runtime/c-consumers.json','utf8'));assert.equal(c.length,6);
   const cBinaryHashes={};
   for(const name of ['ffi','ffi-public','ffi-parse-limits']) {
    for(const kind of ['compile','run']){const row=c.find(v=>v.phase===name+'-'+kind);assert(row);assert.equal(row.exitCode,0);assert(commands.some(v=>v.phase===row.phase&&v.exitCode===0));}
    cBinaryHashes[name]=sha(await readFile('dist/evidence/'+prefix+'-'+name));
   }
   record.correctness={swiftTests:16,drawings:12481,expectedErrors:16572,budgetDifferences:0,apiCases:21,apiConditions:168,cConsumers:3,timing:false,apiReportSha256:sha(apiBytes),swiftLogSha256:sha(corpusBytes),cBinaryHashes};
  }
 }
 assert.equal(sha(await readFile(before.controlDir+'/PeleSwiftPerf')),before.controlBinarySha256,'Frozen prior consumer changed');
 assert.equal(sha(await readFile(before.controlDir+'/swift-build-manifest.json')),before.controlManifestSha256,'Frozen prior manifest changed');
}
await writeFile(dir+'/runtime/'+phase+'.json',JSON.stringify(record,null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({phase,commit,inputCount:Object.keys(current.files).length,sourceInputSha256:current.sha256,binarySha256:record.binarySha256,nativeArchiveSha256:record.nativeArchiveSha256,correctness:record.correctness}));
