import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {gunzipSync} from 'node:zlib';
const base=new URL('./',import.meta.url),read=p=>readFile(new URL(p,base)),json=async p=>JSON.parse(await read(p)),sha=b=>createHash('sha256').update(b).digest('hex');
const index=await json('evidence-index.json');
for(const r of index.records){const b=await read(r.path);assert.equal(sha(b),r.archiveSha256,r.path);assert.equal(sha(r.path.endsWith('.gz')?gunzipSync(b):b),r.sourceSha256,r.path);}
const report=await json('comparison.json'),freeze=await json('source-freeze.json'),raw=await json('../raw/raw-comparison.json'),sdk=await json('../raw/swift-build-manifest.json'),summary=await json('paired-summary.json');
assert(report.succeeded&&report.completedAt);assert.equal(report.rows.length,24);assert.equal(report.settings.trials,21);assert.equal(report.settings.warmupMs,1000);
assert.deepEqual(report.runtimeHashes.candidate,freeze.packageFiles);assert.deepEqual(report.runtimeHashes.control,raw.runtimeHashes.candidate);assert.equal(Object.keys(freeze.packageFiles).length,106);assert.equal(freeze.nativeSourceInputSha256,sdk.sourceInputSha256);
const median=a=>[...a].sort((a,b)=>a-b)[10];
for(const row of report.rows){
 for(const n of ['js','control','candidate']){assert.equal(row.samples[n].length,21);assert.equal(row.medianMs[n],median(row.samples[n]));}
 const ratios=row.samples.candidate.map((v,i)=>v/row.samples.control[i]);const s=summary.find(x=>x.case===row.case&&x.profile===row.profile&&x.stage===row.stage);assert(s);
 assert(Math.abs(s.pairedMedianPercentChange-(median(ratios)-1)*100)<1e-9);assert.equal(s.candidateWins,ratios.filter(v=>v<1).length);
}
const nan=hex=>{const n=BigInt('0x'+hex);return(n&0x7ff0000000000000n)===0x7ff0000000000000n&&(n&0xfffffffffffffn)!==0n;};
const artifacts=index.records.filter(r=>r.path.startsWith('final/'));let differential=0,budget=0;
for(const r of artifacts){const d=JSON.parse(gunzipSync(await read(r.path)));
 if(r.path.endsWith('differential.json.gz')){differential++;assert.equal(d.rows.length,48);for(const row of d.rows){assert.deepEqual(row.packet,row.json);if(row.name==='nan-dimensions'){assert(row.packet.ok);assert(nan(row.packet.result.width)||nan(row.packet.result.height));}}}
 if(r.path.endsWith('budget.json.gz')){budget++;assert.equal(d.length,4);for(const raw of [false,true]){const old=d.find(x=>x.raw===raw&&x.mode==='json'),packet=d.find(x=>x.raw===raw&&x.mode==='packet');assert.deepEqual({...packet,mode:'json'},old);if(raw){assert(packet.attempted.success);assert(packet.attempted.bytes<=64*1024*1024&&packet.attempted.bytes>64*1024*1024-17);}else assert(packet.expectedBytes>64*1024*1024);}}
}
assert.equal(differential,3);assert.equal(budget,3);console.log(`Verified ${index.records.length} archives, 24 conditions, complete control/candidate hashes, unchanged native/Swift inputs, 144 complete result pairs and all six byte-gate comparisons.`);
