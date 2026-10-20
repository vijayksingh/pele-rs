// Portable data-only verifier: no Git, browser, binaries, SDK or subprocesses.
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import {dirname,join,relative} from 'node:path';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
import assert from 'node:assert/strict';
const root=dirname(fileURLToPath(import.meta.url));
const sha=b=>createHash('sha256').update(b).digest('hex');
const raw=p=>readFile(join(root,p));
const bytes=async p=>p.endsWith('.gz')?gunzipSync(await raw(p)):raw(p);
const text=async p=>(await bytes(p)).toString('utf8');
const json=async p=>JSON.parse(await text(p));
const commit='3166b3dd18569ad0bc20f173a8941dd628609ddc';
const source=await json('proof/source.json.gz'),packed=await json('proof/package.json.gz'),after=await json('proof/after.json.gz'),final=await json('proof/after-full-browser.json.gz');
assert.equal(final.commit,commit);assert.equal(after.commit,commit);
for(const record of[source,packed,after,final]){assert.equal(Object.keys(record.sources).length,586);assert.deepEqual(record.sources,source.sources);}
for(const record of[packed,after,final]){assert.equal(Object.keys(record.package).length,113);assert.deepEqual(record.package,packed.package);}
assert.equal(final.gitInputsVerified,586);assert.equal(after.gitInputsVerified,586);
const mapping=await json('proof/immutable-commit-mapping.json.gz');assert.equal(mapping.commit,commit);assert.deepEqual(Object.keys(mapping.files).sort(),Object.keys(final.sources).sort());
for(const[p,entry]of Object.entries(mapping.files)){assert.equal(entry.sha256,final.sources[p]);assert.match(entry.blob,/^[0-9a-f]{40}$/);assert(Number.isSafeInteger(entry.bytes)&&entry.bytes>=0);}
const rehash=await json('proof/frozen-package-rehash.json.gz');assert.deepEqual(rehash.files,final.package);
for(const name of['public_native_flat_store.rs','captured-native-memory.spec.js','captured-sankey.spec.js']) {
 const p=name.endsWith('.rs')?'crates/pele/tests/'+name:'tests/web/'+name;
 assert.equal(sha(await bytes('scope/'+name+'.gz')),final.sources[p]);
}
const full=await text('logs/native-canonical-full-web.log.gz');
assert.match(full,/Running 468 tests using 1 worker/);assert.match(full,/\n\s*468 passed \(13\.4m\)\s*$/);
assert(!/^\s*\d+\s+(?:failed|skipped|interrupted|did not run)\b/m.test(full));
const rows=[...full.matchAll(/^\s*✓\s+(\d+) \[(chromium|firefox|webkit)\] › (.+) \([^\n]*\)\s*$/gm)].map(m=>({ordinal:+m[1],engine:m[2],test:m[3]}));
assert.equal(rows.length,468);assert.deepEqual(rows.map(r=>r.ordinal).sort((a,b)=>a-b),Array.from({length:468},(_,i)=>i+1));
const browser={};let reference;
for(const engine of['chromium','firefox','webkit']) {
 const selected=rows.filter(r=>r.engine===engine);assert.equal(selected.length,156);
 const tests=selected.map(r=>r.test).sort();assert.equal(new Set(tests).size,156);
 if(reference)assert.deepEqual(tests,reference);else reference=tests;
 for(const row of selected){const file=row.test.match(/^(tests\/web\/[^:]+):/);assert(file);assert(final.sources[file[1]]);}
 const memory=selected.filter(r=>r.test.startsWith('tests/web/captured-native-memory.spec.js:'));
 const captured=selected.filter(r=>r.test.startsWith('tests/web/captured-sankey.spec.js:'));
 assert.equal(memory.length,6);assert.equal(captured.length,3);
 browser[engine]={passed:selected.length,failed:0,skipped:0,nativeMemoryCases:memory.length,capturedFeatureCases:captured.length};
}
async function nativeWindow(name,expected){
 const log=await text('logs/'+name+'.log.gz');let target;let pending=[];const rows=[],targets=[];
 for(const line of log.split('\n')) {
  const run=line.match(/Running tests\/([^\s]+)\.rs /);if(run){target=run[1];pending=[];}
  const test=line.match(/^test (.+) \.\.\. ok$/);if(test){assert(target);pending.push(target+'::'+test[1]);rows.push(target+'::'+test[1]);}
  const result=line.match(/test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out/);
  if(result){assert.equal(+result[1],pending.length);assert.deepEqual(result.slice(2).map(Number),[0,0,0,0]);targets.push({target,passed:pending.length});}
 }
 assert.equal(rows.length,expected);assert.equal(new Set(rows).size,rows.length);return {rows,targets};
}
const initial=await nativeWindow('native-canonical-core',26),followup=await nativeWindow('native-canonical-corpus',6),defaults=await nativeWindow('native-canonical-default',11);
const featureUnion=new Set([...initial.rows,...followup.rows]);assert.equal(featureUnion.size,27);
assert.equal(initial.rows.filter(x=>followup.rows.includes(x)).length,5);
assert(followup.rows.includes('public_native_flat_store::native_flat_frozen_source_corpus_preserves_complete_graphs_and_errors'));
const nativeSource=await text('scope/public_native_flat_store.rs.gz');assert.match(nativeSource,/assert_eq!\(cases\.len\(\), 11_616\)/);
assert.match(await text('scope/captured-native-memory.spec.js.gz'),/PELE_NATIVE_CANONICAL_SANKEY/);
assert.match(await text('scope/captured-sankey.spec.js.gz'),/PELE_CAPTURED_SANKEY/);
assert.match(await text('logs/native-canonical-wasm-build.log.gz'),/Finished `release` profile/);
assert.match(await text('logs/native-canonical-wasm-check.log.gz'),/Finished `dev` profile/);
const provenance=await json('provenance.json');assert.equal(provenance.sourceCommit,commit);
for(const[p,o]of Object.entries(provenance.origins)){const b=await bytes(p);assert.equal(sha(b),o.sourceSha256,p);assert.equal(b.length,o.sourceBytes);}
const summary={scope:'frozen native-canonical backend correctness; no performance or release claim',sourceCommit:commit,
 browser:{totalPassed:rows.length,totalFailed:0,totalSkipped:0,engines:browser,reportedWallTime:'13.4m'},
 native:{featureDistinctTargetTests:featureUnion.size,featureExecutions:initial.rows.length+followup.rows.length,initialWindow:initial.targets,followupWindow:followup.targets,defaultPassed:defaults.rows.length,defaultWindow:defaults.targets,sourceCorpusCases:11616},
 provenance:{sourceInputs:586,packageFiles:113,wasmSha256:final.package['pkg/pele_bg.wasm'],sourceFrozenBeforeBuildCommit:source.commit,sourceAndPackageUnchangedThroughFullBrowser:true,immutableGitMappings:Object.keys(mapping.files).length,frozenPackageRehashedFiles:Object.keys(rehash.files).length},
 limitations:['Feature-native windows overlap: five tests repeat; default/feature names also overlap and are not one unique total.','Package build was recorded before final commit; identical frozen source hashes bind it to 3166b3d, not a claim of a later clean rebuild.','The portable verifier checks captured logs and hash records; source/binary trees are omitted and it performs no new browser run or package rehash.','Explicit feature tests opt into the native owner; ordinary default selection remains a separate adoption decision.','The experimental backend performance/release decision is not established by passing correctness tests. Current cursor-layer sources are excluded.']};
if(process.argv.includes('--write-summary'))await writeFile(join(root,'summary.json'),JSON.stringify(summary,null,2)+'\n');else assert.deepEqual(await json('summary.json'),summary);
if(!process.argv.includes('--write-summary')){
 const index=await json('evidence-index.json');for(const[p,h]of Object.entries(index.files))assert.equal(sha(await raw(p)),h,p);
 const all=[];async function walk(dir){for(const e of await readdir(dir,{withFileTypes:true})){const p=join(dir,e.name);if(e.isDirectory())await walk(p);else all.push(relative(root,p));}}await walk(root);
 assert.deepEqual(all.filter(p=>p!=='evidence-index.json').sort(),Object.keys(index.files).sort());
}
console.log(JSON.stringify(summary,null,2));
