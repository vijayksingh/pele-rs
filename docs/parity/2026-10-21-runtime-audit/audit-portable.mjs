import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {gunzipSync} from 'node:zlib';
const sha=b=>createHash('sha256').update(b).digest('hex');
const pkg=resolve(process.argv[2]??'dist/package-web/package'), oracle=resolve(process.argv[3]??'node_modules/pele/dist/esm');
const read=p=>fs.readFileSync(p);
const fixturePath=resolve(process.argv[4]??'fixtures/pele/diagram-entries.json'),fixture=JSON.parse(read(fixturePath));
const expected=JSON.parse(gunzipSync(read(new URL('./web-entry-report.json.gz',import.meta.url))));
const equalHashes=(a,b)=>JSON.stringify(Object.entries(a).sort())===JSON.stringify(Object.entries(b).sort());
if(!equalHashes(hashes(pkg),expected.package)||!equalHashes(hashes(oracle),expected.oracle)||sha(read(fixturePath))!==expected.fixtureSHA256)throw Error('Inputs do not match recorded d7caf/oracle/fixture');
const ready=await import(pathToFileURL(pkg+'/diagram-entries.js'));await ready.loadDiagramEntries(read(pkg+'/pkg/pele_bg.wasm'));
const rows=[];
const constructor=m=>{const C=Object.getPrototypeOf(m)?.constructor;try{const x=new C();return{name:C?.name,arity:C?.length,ok:true,ownKeys:Reflect.ownKeys(x),type:x.type};}catch(e){return{name:C?.name,arity:C?.length,ok:false,error:{name:e.name,message:e.message}};}};
for(const metadata of fixture.entries){const c=fixture.cases.find(c=>c.family===metadata.type),row={type:metadata.type,fixture:c.id};
 for(const lane of ['native','oracle']){const entry=(await import(pathToFileURL((lane==='native'?pkg:oracle)+'/diagrams/'+metadata.type+'.js'))).default;let model;
  try{model=entry.parse(c.source,c.config,c.title??undefined,c.lineOffset,{maxEdges:undefined});const marker={};let calls=0,originalThrow=false,error;
   const construction=constructor(model);
   try{entry.render(model,c.config,{...c.options,measurer:{width(){calls++;model.title='audit callback title';throw marker;}}});}
   catch(e){originalThrow=e===marker;if(!originalThrow)error={name:e?.name,message:e?.message};}
   row[lane]={construction,callback:{calls,titleRetained:model.title==='audit callback title',originalThrow,...(error?{error}:{})}};
  }finally{if(lane==='native'&&model)ready.disposeDiagramModel(model);}
 }rows.push(row);
}
function hashes(root){const out={};function walk(p){for(const e of fs.readdirSync(p,{withFileTypes:true})){const f=p+'/'+e.name;if(e.isDirectory())walk(f);else if(e.isFile())out[f.slice(root.length+1)]=sha(read(f));}}walk(root);return out;}
const actual=hashes(pkg);if(!equalHashes(actual,expected.package)||!equalHashes(hashes(oracle),expected.oracle))throw Error('Inputs changed during probe');
console.log(JSON.stringify({kind:'functional correctness only; no timing',source:expected.source,package:actual,oracle:hashes(oracle),fixtureSHA256:sha(read(fixturePath)),scriptSHA256:sha(read(new URL(import.meta.url))),rows},null,2));
