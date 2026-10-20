// Parent supplies immutable packages/hashes and authorizes each phase separately.
import assert from 'node:assert/strict';
import {readFile,readdir,mkdir,writeFile} from 'node:fs/promises';
import {resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawn} from 'node:child_process';
const here=dirname(fileURLToPath(import.meta.url)),sha=data=>createHash('sha256').update(data).digest('hex');
const arg=(key,fallback)=>{const i=process.argv.indexOf(key);return i<0?fallback:process.argv[i+1];};
assert(process.argv.includes('--go')||process.argv.includes('--preflight-only'),'Explicit GO required');
const phase=arg('--phase');assert(['warm','first-use'].includes(phase));
const roots={control:resolve(arg('--control','parse-architecture/source-symbols-runtime/candidate-package')),candidate:resolve(arg('--candidate','parse-architecture/owned-ids-runtime/candidate-package'))};assert.notEqual(roots.control,roots.candidate);
const expected={control:arg('--control-wasm'),candidate:arg('--candidate-wasm')};for(const hash of Object.values(expected))assert(/^[a-f0-9]{64}$/.test(hash??''),'Supply parent-verified default-build WASM hashes');
const candidateCommit=arg('--candidate-source');assert(/^[a-f0-9]{40}$/.test(candidateCommit??''),'Supply frozen candidate source commit');
const pins=JSON.parse(await readFile(resolve(here,'source-layout-pins.json'),'utf8'));
for(const[name,hash]of Object.entries(pins.files))assert.equal(sha(await readFile(resolve(here,name))),hash);
async function hashes(root,prefix=''){const out={};for(const e of(await readdir(resolve(root,prefix),{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){const path=prefix+e.name;if(e.isDirectory())Object.assign(out,await hashes(root,path+'/'));else{assert(e.isFile());out[path]=sha(await readFile(resolve(root,path)));}}return out;}
const before={};for(const[name,root]of Object.entries(roots)){before[name]=await hashes(root);assert.equal(before[name]['pkg/pele_bg.wasm'],expected[name]);const glue=await readFile(resolve(root,'pkg/pele.js'),'utf8');assert(!/native_canonical_memory|parse_with_native_storage|parse_with_host_storage|parse_with_captured_sankey|parse_prepared_with_captured_sankey/.test(glue),'Experimental exports present: default-feature build required');}
assert.equal(expected.control,pins.controlWasm);
assert.deepEqual(before.control,pins.controlPackageHashes,'Control differs from frozen immediate11f package');
const candidateManifestPath=resolve(arg('--candidate-manifest',''));assert(arg('--candidate-manifest'),'Supply frozen candidate package manifest');
const candidateManifestBytes=await readFile(candidateManifestPath),candidateManifest=JSON.parse(candidateManifestBytes);
assert.equal(candidateCommit,pins.candidateSourceCommit);assert.equal(candidateManifest.commit,candidateCommit);assert.deepEqual(before.candidate,candidateManifest.package,'Candidate differs from frozen package manifest');
const allPaths=[...new Set([...Object.keys(before.control),...Object.keys(before.candidate)])].sort();
const differences=allPaths.filter(path=>before.control[path]!==before.candidate[path]);
for(const[path,hash]of Object.entries(pins.candidateHandwrittenHashes))assert.equal(before.candidate[path],hash,'Candidate handwritten drift: '+path);
assert(!differences.some(path=>!path.startsWith('pkg/')&&!Object.hasOwn(pins.candidateHandwrittenHashes,path)),'Unapproved handwritten runtime or static package drift');
assert.equal(Object.keys(before.candidate).length,117);assert.equal(Object.keys(before.control).length,117);
const probeControl=resolve(arg('--probe-control',pins.probeLegacyControl.remotePackage));assert.notEqual(probeControl,roots.control);const probeBefore=await hashes(probeControl);assert.deepEqual(probeBefore,pins.probeLegacyControl.package);
const verifyProbeControl=async()=>assert.deepEqual(await hashes(probeControl),probeBefore,'Probe-only e148 drift');
const output=resolve(arg('--output','parse-architecture/owned-ids-runtime'));await mkdir(output,{recursive:true});
const stages=phase==='warm'?['parse','render-model','render-source']:['render-source','parse-first-render','parse-first-edited-render'];
const selected=['sankey/small','sankey/medium','sankey/large','flowchart/medium'];
const harnessName=phase==='warm'?'compare-web-vm.mjs':'compare-first-use-web-vm.mjs',reportPath=resolve(output,phase+'.json');
const args=[resolve(here,harnessName),'--control',roots.control,'--candidate',roots.candidate,'--trials','21','--warmup-ms','1000','--cases',selected.join(','),'--profiles','fixed,canvas','--stages',stages.join(','),'--output',reportPath];
assert(!args.some(value=>value.startsWith('--')&&/captured|canonical/.test(value)),'No experimental selector flags');
const record={schemaVersion:1,phase,source:{control:pins.controlSourceCommit,candidate:candidateCommit},expectedBuild:pins.build,candidateManifestSHA256:sha(candidateManifestBytes),roots,probeOnlyControl:{root:probeControl,hashes:probeBefore,source:pins.probeLegacyControl.source},reusedTimedControlProof:pins.reusedControlProof,runtimeHashes:before,harnessSHA256:pins.files[harnessName],casesSHA256:pins.files['cases.mjs'],runnerSHA256:sha(await readFile(fileURLToPath(import.meta.url))),experimentalExportsAbsent:true,experimentalSelectorFlags:false,differences:differences.map(path=>({path,control:before.control[path],candidate:before.candidate[path],generated:path.startsWith('pkg/'),change:!before.control[path]?'added':!before.candidate[path]?'removed':'modified'})),args,startedAt:new Date().toISOString()};
const save=()=>writeFile(resolve(output,phase+'-provenance.json'),JSON.stringify(record,null,2)+'\n');await save();
if(process.argv.includes('--preflight-only')){console.log('Preflight passed; no browser launched');process.exit(0);}
assert(process.env.PLAYWRIGHT_BROWSERS_PATH,'Use existing browser install');
const continuityPath=resolve(output,phase+'-layout-continuity.json');
const continuityArgs=[resolve(here,'probe-layout-continuity.mjs'),'--control',probeControl,'--candidate',roots.candidate,'--output',continuityPath];
const continuityExit=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,continuityArgs,{stdio:'inherit',env:process.env});child.on('error',reject);child.on('exit',(code,signal)=>resolve({code,signal}));});
record.continuity={...continuityExit,probeSHA256:pins.files['probe-layout-continuity.mjs'],path:continuityPath};await save();assert.equal(continuityExit.code,0);
const continuity=JSON.parse(await readFile(continuityPath));assert(continuity.succeeded&&continuity.checks.length===8&&continuity.prototypesRestored&&continuity.readonlyJobsVerified&&continuity.browserClosedBeforeTiming);
record.continuity.reportSHA256=sha(await readFile(continuityPath));await verifyProbeControl();for(const[name,root]of Object.entries(roots))assert.deepEqual(await hashes(root),before[name]);await save();
const activationPath=resolve(output,phase+'-activation.json');
const activationArgs=[resolve(here,'probe-source-layout.mjs'),'--control',probeControl,'--candidate',roots.candidate,'--output',activationPath];
const activationExit=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,activationArgs,{stdio:'inherit',env:process.env});child.on('error',reject);child.on('exit',(code,signal)=>resolve({code,signal}));});
record.activation={...activationExit,probeSHA256:pins.files['probe-source-layout.mjs'],path:activationPath};await save();assert.equal(activationExit.code,0);
const activation=JSON.parse(await readFile(activationPath));assert(activation.succeeded&&activation.checks.length===16&&activation.instrumentationOnlyProbe&&activation.browserClosedBeforeTiming);
for(const check of activation.checks){const expected=check.runtime==='candidate'&&check.case.startsWith('sankey/')?[true,true]:[];assert.deepEqual(check.initializerDecisions,expected);}
record.activation.reportSHA256=sha(await readFile(activationPath));await verifyProbeControl();for(const[name,root]of Object.entries(roots))assert.deepEqual(await hashes(root),before[name]);await save();
const ingressPath=resolve(output,phase+'-ingress-activation.json');
const ingressArgs=[resolve(here,'probe-source-ingress.mjs'),'--candidate',roots.candidate,'--output',ingressPath];
const ingressExit=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,ingressArgs,{stdio:'inherit',env:process.env});child.on('error',reject);child.on('exit',(code,signal)=>resolve({code,signal}));});
record.ingressActivation={...ingressExit,probeSHA256:pins.files['probe-source-ingress.mjs'],path:ingressPath};await save();assert.equal(ingressExit.code,0);
const ingress=JSON.parse(await readFile(ingressPath));assert(ingress.succeeded&&ingress.checks.length===12&&ingress.methodsRestored&&ingress.browserClosedBeforeTiming);
for(const check of ingress.checks){assert(check.mode==='sparse'?check.calls.sparse>0&&check.calls.canonical===0:check.calls.canonical>0&&check.calls.sparse===0);}
record.ingressActivation.reportSHA256=sha(await readFile(ingressPath));await verifyProbeControl();for(const[name,root]of Object.entries(roots))assert.deepEqual(await hashes(root),before[name]);await save();
const exit=await new Promise((resolve,reject)=>{const p=spawn(process.execPath,args,{stdio:'inherit',env:process.env});p.on('error',reject);p.on('exit',(code,signal)=>resolve({code,signal}));});Object.assign(record,exit);await save();assert.equal(exit.code,0);
const report=JSON.parse(await readFile(reportPath,'utf8'));assert(report.succeeded&&report.rows.length===24);assert.equal(report.harnessSHA256,pins.files[harnessName]);assert.equal(report.casesSHA256,pins.files['cases.mjs']);assert.deepEqual(report.runtimeHashes,before);assert.equal(report.settings.trials,21);assert.equal(report.settings.warmupMs,1000);assert(!report.nativeCanonicalBackend.candidateEnabled&&!report.nativeCanonicalBackend.controlEnabled&&!report.capturedBackend.candidateEnabled);
const keys=new Set();for(const row of report.rows){assert(selected.includes(row.case)&&['fixed','canvas'].includes(row.profile)&&stages.includes(row.stage));const key=[row.case,row.profile,row.stage].join('/');assert(!keys.has(key));keys.add(key);assert.equal(row.counts.control,row.counts.candidate);for(const values of Object.values(row.samples))assert.equal(values.length,21);}
await verifyProbeControl();for(const[name,root]of Object.entries(roots))assert.deepEqual(await hashes(root),before[name]);
record.succeeded=true;record.verifiedRows=24;record.reportSHA256=sha(await readFile(reportPath));record.completedAt=new Date().toISOString();await save();console.log(reportPath);
