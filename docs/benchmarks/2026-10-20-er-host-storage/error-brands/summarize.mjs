// Data-only verification. Never launches a consumer or rebuilds a package.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
const base=new URL('./',import.meta.url),read=p=>readFile(new URL(p,base)),json=async p=>JSON.parse(await read(p)),sha=b=>createHash('sha256').update(b).digest('hex');
const index=await json('evidence-index.json');
for(const r of index.records){const bytes=await read(r.path);assert.equal(sha(bytes),r.archiveSha256,r.path);assert.equal(sha(r.path.endsWith('.gz')?gunzipSync(bytes):bytes),r.sourceSha256,r.path);}
const report=await json('comparison.json'),identity=await json('identity.json'),summary=await json('paired-summary.json');
assert(report.succeeded&&report.completedAt);assert.equal(report.rows.length,24);assert.equal(summary.length,24);assert.equal(report.settings.trials,21);assert.equal(report.settings.warmupMs,1000);
assert.equal(Object.keys(report.runtimeHashes.control).length,106);assert.equal(Object.keys(report.runtimeHashes.candidate).length,106);assert.deepEqual(report.runtimeHashes.control,identity.controlPackageFiles);
assert.deepEqual(Object.keys(report.runtimeHashes.candidate).filter(k=>report.runtimeHashes.candidate[k]!==report.runtimeHashes.control[k]),['host-er-store.js']);
assert.equal(report.runtimeHashes.candidate['host-er-store.js'],identity.sourceFiles['bindings/web/host-er-store.js']);assert.equal(sha(await read('fixture.spec.js')),identity.sourceFiles['tests/web/er-error-brands.spec.js']);
assert.equal(report.harnessSHA256,identity.sourceFiles['scripts/perf/compare-web-vm.mjs']);assert.equal(report.casesSHA256,identity.sourceFiles['scripts/perf/cases.mjs']);
const median=a=>[...a].sort((a,b)=>a-b)[10],seen=new Set();
for(const row of report.rows){
 const key=JSON.stringify([row.case,row.profile,row.stage]);assert(!seen.has(key));seen.add(key);
 assert.equal(row.counts.control,row.counts.candidate);
 for(const lane of ['js','control','candidate']){assert.equal(row.samples[lane].length,21);assert.equal(row.medianMs[lane],median(row.samples[lane]));assert(row.warmups[lane].elapsedMs>=1000);}
 const ratios=row.samples.candidate.map((v,i)=>v/row.samples.control[i]);const s=summary.find(x=>x.case===row.case&&x.profile===row.profile&&x.stage===row.stage);assert(s);
 assert(Math.abs(s.pairedMedianPercentChange-(median(ratios)-1)*100)<1e-9);assert.equal(s.candidateWins,ratios.filter(v=>v<1).length);assert.equal(s.candidateOverJS,row.medianMs.candidate/row.medianMs.js);
}
for(const c of ['sequence/small','er/small','er/medium','er/large'])for(const p of ['fixed','canvas'])for(const s of ['parse','render-model','render-source'])assert(seen.has(JSON.stringify([c,p,s])));
const brands=['Error','PeleError','TypeError','RangeError','SyntaxError','ReferenceError','EvalError','URIError'];
for(const engine of ['chromium','firefox','webkit']){
 const rows=JSON.parse(gunzipSync(await read(`final/${engine}.json.gz`)));assert.equal(rows.length,16);
 for(const mode of ['control','candidate'])assert.deepEqual(rows.filter(r=>r.mode===mode).map(r=>r.brand),brands);
 for(const row of rows){assert.equal(row.diagnostics,0);for(const key of ['sameDescriptors','samePrototype','aliases'])assert.equal(row[key],true);assert.equal(row.original,row.brand!=='RangeError');assert.deepEqual(row.repaired,row.second);if(row.brand==='RangeError')assert.deepEqual(row.caught,{name:'PeleError',message:'Diagram is nested too deeply.',code:'limit'});}
 assert.deepEqual(rows.filter(r=>r.mode==='candidate').map(({mode,...r})=>r),rows.filter(r=>r.mode==='control').map(({mode,...r})=>r));
}
const first=gunzipSync(await read('logs/attempt1.log.gz')).toString(),final=gunzipSync(await read('logs/final.log.gz')).toString();assert(first.includes('39 passed')&&first.includes('3 failed'));assert(final.includes('3 passed'));
console.log('Verified complete 106-file identity, 24 conditions × 21 trials, all 24 paired summaries, 48 brand observations and retained failed-attempt archives. No new runtime execution.');
