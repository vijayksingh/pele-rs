import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
const base=new URL('./',import.meta.url),read=async p=>readFile(new URL(p,base)),json=async p=>JSON.parse(await read(p));
const sha=b=>createHash('sha256').update(b).digest('hex');
const index=await json('evidence-index.json');
for(const r of index.records){const b=await read(r.path);assert.equal(sha(b),r.archiveSha256,r.path);assert.equal(sha(r.path.endsWith('.gz')?gunzipSync(b):b),r.sourceSha256,r.path);}
const raw=await json('raw-comparison.json'),ordinary=await json('ordinary-comparison.json'),freeze=await json('source-freeze.json'),inputs=await json('raw-inputs.json'),summary=await json('paired-summary.json');
assert(raw.succeeded&&ordinary.succeeded&&raw.completedAt&&ordinary.completedAt);assert.equal(raw.rows.length,18);assert.equal(ordinary.rows.length,12);
assert.deepEqual(raw.runtimeHashes.candidate,freeze.packageFiles);assert.deepEqual(ordinary.runtimeHashes.candidate,freeze.packageFiles);assert.deepEqual(raw.runtimeHashes.control,ordinary.runtimeHashes.control);assert.deepEqual(inputs.hashes.packages.candidate,freeze.packageFiles);
assert.equal(Object.keys(freeze.packageFiles).length,106);assert.equal(Object.keys(freeze.sourceFiles).length,456);
const median=a=>[...a].sort((a,b)=>a-b)[Math.ceil(a.length/2)-1];
for(const [report,group,count]of [[raw,'raw',15],[ordinary,'ordinary',9]])for(const row of report.rows){
 for(const name of ['candidate','control','js'])assert.equal(row.samples[name].length,count);
 const ratios=row.samples.candidate.map((v,i)=>v/row.samples.control[i]);
 const s=summary[group].find(x=>x.case===row.case&&x.profile===row.profile&&x.stage===row.stage);assert(s);
 assert(Math.abs(s.pairedMedianPercentChange-(median(ratios)-1)*100)<1e-9);assert.equal(s.candidateWins,ratios.filter(v=>v<1).length);
 if(group==='raw'){assert(row.correctness.beforeAndAfter&&row.correctness.completeOutput&&row.correctness.descriptorsAndAliases);assert.equal(row.raw.length,15);const first={js:0,control:0,candidate:0};for(const t of row.raw){assert.equal(new Set(t.order).size,3);first[t.order[0]]++;}assert.deepEqual(first,{js:5,control:5,candidate:5});}
}
const swift=await json('swift-api.json');assert(swift.completedAt);assert.equal(swift.rows.length,168);
console.log(`Verified ${index.records.length} archives, complete package identity, 18 raw + 12 ordinary conditions, balanced raw order and 168 Swift conditions.`);
