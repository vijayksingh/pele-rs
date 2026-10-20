// Portable, data-only verifier. No subprocesses, builds, SDK reads or test runs.
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
const aggregate=files=>{const h=createHash('sha256');for(const [p,s] of Object.entries(files).sort(([a],[b])=>a.localeCompare(b)))h.update(p+'\0'+s+'\0');return h.digest('hex')};
const records={};for(const phase of ['before-sdk','after-sdk','before-correctness','after-correctness'])records[phase]=await json('proof/'+phase+'.json.gz');
const before=records['before-sdk'],sdk=records['after-sdk'],pre=records['before-correctness'],post=records['after-correctness'];
for(const record of Object.values(records)){assert.equal(record.commit,before.commit);assert.equal(Object.keys(record.current.files).length,357);assert.equal(aggregate(record.current.files),record.current.sha256);}
assert.equal(Object.keys(before.gitInputHashes).length,344);
for(const [p,h] of Object.entries(before.gitInputHashes)){assert.equal(h,before.current.files[p]);for(const record of [sdk,pre,post])assert.equal(record.current.files[p],h);}
const changed=Object.keys(sdk.current.files).filter(k=>before.current.files[k]!==sdk.current.files[k]);assert.deepEqual(changed,['CPele/macos-arm64/libpele_ffi.a']);
assert.deepEqual(pre.current,sdk.current);assert.deepEqual(post.current,pre.current);
const manifest=await json('proof/swift-build-manifest.json.gz');assert.deepEqual(pre.manifest,manifest);assert.deepEqual(post.manifest,manifest);assert.deepEqual(manifest.inputHashes,post.current.files);assert.equal(manifest.sourceInputSha256,post.current.sha256);assert.equal(manifest.revision,before.commit);
for(const record of [pre,post]){assert.equal(record.binarySha256,manifest.binarySha256);assert.equal(record.nativeArchiveSha256,manifest.nativeHostArchiveSha256);assert.equal(record.current.files['CPele/macos-arm64/libpele_ffi.a'],record.nativeArchiveSha256);}
const api=await json('proof/swift-api.json.gz');assert.equal(api.mode,'swift-api-correctness-only');assert.equal(api.revision,before.commit);assert.deepEqual(api.buildManifest,manifest);assert.equal(api.cases.length,21);assert.equal(api.rows.length,168);
const cases=new Map(api.cases.map(c=>[c.id,c]));assert.equal(cases.size,21);const keys=new Set();
for(const c of cases.values()){assert.equal(sha(Buffer.from(c.source)),c.sourceSha256);assert.equal(Buffer.byteLength(c.source),c.sourceBytes);for(const output of c.prepared.outputs)assert.deepEqual(output,c.output);}
for(const row of api.rows){assert(!keys.has(row.case+'/'+row.stage));keys.add(row.case+'/'+row.stage);assert(api.settings.stages.includes(row.stage));assert.equal(row.correctness.caseID,row.case);assert.equal(row.correctness.stage,row.stage);assert.equal(row.correctness.count,1);assert.deepEqual(row.correctness.output,cases.get(row.case).output);assert(!('samples' in row));}
const corpus=await text('logs/captured-native-core-swift-corpus.log.gz');assert.match(corpus,/Executed 16 tests, with 0 failures/);const totals=corpus.match(/Swift mutable corpus: (\d+) drawings, (\d+) errors, (\d+) explicit budget differences/);assert(totals);assert.deepEqual(totals.slice(1).map(Number),[12481,16572,0]);
const commands=await json('proof/correctness-commands.json.gz');assert(commands.length>0);for(const c of commands)assert.equal(c.exitCode,0,c.phase);const cConsumers=await json('proof/c-consumers.json.gz');assert.equal(cConsumers.length,6);for(const c of cConsumers){assert.equal(c.exitCode,0);assert.deepEqual(c,commands.find(x=>x.phase===c.phase));}assert.equal(cConsumers.filter(c=>c.phase.endsWith('-run')).length,3);
const native={};for(const [name,expected] of [['sankey-graphless-core-native',61],['sankey-graphless-default-native',29],['combined-host-dispatch-native',22]]){const log=await text('logs/'+name+'.log.gz');const matches=[...log.matchAll(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/g)];assert(matches.length);for(const m of matches)assert.deepEqual(m.slice(2).map(Number),[0,0,0,0]);const passed=matches.reduce((s,m)=>s+Number(m[1]),0);assert.equal(passed,expected);native[name]={passed,testTargets:matches.length};}
const mappings=await json('proof/immutable-commit-mapping.json');for(const mapping of Object.values(mappings)){const m=await json('proof/'+mapping.manifest);const entries=Array.isArray(m.files)?m.files.map(x=>[x.path,x.candidateSha256]):Object.entries(m.files).map(([p,x])=>[p,x.candidateSha256]);assert.deepEqual(mapping.candidateFiles,Object.fromEntries(entries));}assert.equal(mappings['25d87cb'].commit,before.commit);
const origins=(await json('provenance.json')).origins;for(const [p,o] of Object.entries(origins))assert.equal(sha(await bytes(p)),o.sourceSha256,p+' recorded source copy mismatch');
assert.equal(sha(await raw('execution/swift-build.mjs')),manifest.inputHashes['consumer/build.mjs']);assert.equal(sha(await raw('execution/swift-run.mjs')),manifest.inputHashes['consumer/run.mjs']);
const summary={scope:'correctness-only; no latency or browser-backend proof',sdkCommit:before.commit,combinedCoreCommit:mappings['eb72419'].commit,native,swift:{tests:16,drawings:Number(totals[1]),expectedErrors:Number(totals[2]),explicitBudgetDifferences:Number(totals[3]),apiCases:api.cases.length,apiConditions:api.rows.length,cConsumers:3},freshness:{inputs:357,gitMappedSourceInputs:344,sourceInputSha256:manifest.sourceInputSha256,binarySha256:manifest.binarySha256,nativeHostArchiveSha256:manifest.nativeHostArchiveSha256,changedDuringSDK:changed,unchangedThroughCorrectness:true},limitations:['Native checks overlap between windows and are not a unique test count.','The SDK proof is 25d87cb, not later eb72419 or the optional captured WASM owner.','Binaries/packages are deliberately omitted; stored hash records establish captured provenance, not a portable rebuild.','No timings, browser validation, Intel SDK validation or complete JS parity claim.']};
if(process.argv.includes('--write-summary'))await writeFile(join(root,'summary.json'),JSON.stringify(summary,null,2)+'\n');else assert.deepEqual(await json('summary.json'),summary);
if(!process.argv.includes('--write-summary')){const index=await json('evidence-index.json');for(const [p,h]of Object.entries(index.files))assert.equal(sha(await raw(p)),h,p+' index mismatch');const all=[];async function walk(dir){for(const x of await readdir(dir,{withFileTypes:true})){const p=join(dir,x.name);if(x.isDirectory())await walk(p);else all.push(relative(root,p));}}await walk(root);assert.deepEqual(all.filter(x=>x!=='evidence-index.json').sort(),Object.keys(index.files).sort());}
console.log(JSON.stringify(summary,null,2));
