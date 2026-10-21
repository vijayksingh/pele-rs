// Preserve the existing native arena representation before changing its builder.
// These hashes pin IDs/order/unused tables; Pele's public corpus pins semantics.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {cases} from '../perf/cases.mjs';
const [packagePath,output]=process.argv.slice(2);
assert(packagePath&&output,'Usage: node scripts/parity/freeze-gantt-construction.mjs FROZEN_PACKAGE OUTPUT');
const pkg=resolve(packagePath),sha=b=>createHash('sha256').update(b).digest('hex');
const wasm=await readFile(resolve(pkg,'pkg/pele_bg.wasm'));
assert.equal(sha(wasm),'6c3fca583babada8323ef7dca2f0dd1320749d2ed5bdb4e46e550e18268ecee3',
 'Use the frozen pre-builder SVG-root package; never regenerate expectations from a candidate');
const {initSync,NativeRegistry}=await import(pathToFileURL(resolve(pkg,'pkg/pele.js')));
const {decodeGraphBinary,encodeGraphBinary}=await import(pathToFileURL(resolve(pkg,'graph-binary.js')));
initSync({module:wasm});
const registry=new NativeRegistry(true);
const selected=cases().filter(c=>c.family==='gantt').map(c=>({id:c.id,source:c.source,options:c.options}));
for(const [id,source]of Object.entries({
 empty:'gantt',
 metadata:'gantt\ntitle **Build**\naccTitle: Accessible\naccDescr: Schedule\ndateFormat YYYY-MM-DD\naxisFormat %Y-%m-%d\nincludes 2024-01-06\nexcludes weekends\nsection Work\nTask:active,done,crit,milestone,a,2024-01-01,2d\nFollow:b,after a,1d',
 duplicates:'gantt\ndateFormat YYYY-MM-DD\nsection A\nFirst:same,2024-01-01,1d\nSecond:same,2024-01-02,2d\nsection A\nThird:t,after same,3d',
 unicode:'gantt\ndateFormat YYYY-MM-DD\nsection 漢字\nCafé 😀:a,2024-01-01,1d\nNext:b,after a,2d',
 dates:'gantt\ndateFormat YYYY-MM-DD\nA:a,2024-01-01,2024-01-03\nB:b,after a,2d\nC:c,after b,1d',
 internedKeys:'gantt\ntitle push\ndateFormat YYYY-MM-DD\nsection length\n0:push,2024-01-01,1d\nlength:0,after push,2d\npush:length,after 0,1d\nclick push call callback() href "https://example.com"',
}))selected.push({id,source,options:{}});
selected.push({id:'utf8-boundaries',options:{},source:'gantt\ndateFormat YYYY-MM-DD\nsection length\n'+
 ['a'.repeat(64),'a'.repeat(65),'é'.repeat(32),'é'.repeat(33),'🦀'.repeat(16),'🦀'.repeat(17)]
 .flatMap((label,i)=>[`${label}:id${i},2024-01-01,1d`,`${label}:again${i},2024-01-02,1d`]).join('\n')});
const rows=[];
try{for(const c of selected){
 const document=registry.parse(c.source,JSON.stringify(c.options),0,0,0,0);
 try{const {graph}=decodeGraphBinary(document.snapshot_binary()),binary=encodeGraphBinary('0',graph);
  rows.push({...c,graphSHA256:sha(binary),nodes:graph.nodes.length,strings:graph.strings.length,binaryBytes:binary.length});
 }finally{document.free();}
}}finally{registry.free();}
await writeFile(output,JSON.stringify({provenance:'Frozen pele-rs SVG-root package, production source 0aba8ef; storage regression, not upstream oracle',wasmSHA256:sha(wasm),graphBinarySHA256:sha(await readFile(resolve(pkg,'graph-binary.js'))),cases:rows},null,2)+'\n');
console.log(`Frozen ${rows.length} complete arena fingerprints`);
