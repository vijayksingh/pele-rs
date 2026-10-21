// Public entry correctness probe; no benchmark, private renderer, or timing loop.
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {gunzipSync} from 'node:zlib';
import {execFileSync} from 'node:child_process';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const read=path=>fs.readFileSync(path);
if(!process.argv[2])throw new Error('Usage: node scripts/parity/audit-runtime.mjs PACKAGE [ORACLE_ESM] [FIXTURE]');
const pkg=resolve(process.argv[2]),oracle=resolve(process.argv[3]??'node_modules/pele/dist/esm');
const fixturePath=resolve(process.argv[4]??'fixtures/pele/diagram-entries.json');
const fixture=JSON.parse(read(fixturePath));
const pinned=JSON.parse(gunzipSync(read('docs/parity/2026-10-21-runtime-audit/web-entry-report.json.gz')));
const equal=(a,b)=>JSON.stringify(Object.entries(a).sort())===JSON.stringify(Object.entries(b).sort());
function hashes(root){const out={};function walk(path){for(const item of fs.readdirSync(path,{withFileTypes:true})){const file=path+'/'+item.name;if(item.isDirectory())walk(file);else if(item.isFile())out[file.slice(root.length+1)]=sha(read(file));}}walk(root);return out;}
const packageBefore=hashes(pkg),oracleBefore=hashes(oracle);
if(!equal(oracleBefore,pinned.oracle)||sha(read(fixturePath))!==pinned.fixtureSHA256)throw new Error('Unpinned oracle or fixture');
const sourceCommit=execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim();
const ready=await import(pathToFileURL(pkg+'/diagram-entries.js'));
await ready.loadDiagramEntries(read(pkg+'/pkg/pele_bg.wasm'));
function construction(model,lane){const C=Object.getPrototypeOf(model)?.constructor;let instance;try{instance=new C();return {classRoot:C!==Object,name:C?.name,arity:C?.length,ok:true,ownKeys:Reflect.ownKeys(instance),type:instance.type};}catch(error){return {classRoot:C!==Object,name:C?.name,arity:C?.length,ok:false,error:{name:error?.name,message:error?.message}};}finally{if(lane==='native'&&instance&&C!==Object)ready.disposeDiagramModel(instance);}}
const rows=[];
for(const metadata of fixture.entries){
 const input=fixture.cases.find(row=>row.family===metadata.type),row={type:metadata.type,fixture:input.id};
 for(const lane of ['native','oracle']){
  const entry=(await import(pathToFileURL((lane==='native'?pkg:oracle)+'/diagrams/'+metadata.type+'.js'))).default;let model;
  try{
   model=entry.parse(input.source,input.config,input.title??undefined,input.lineOffset,{maxEdges:undefined});
   const marker={};let calls=0,originalThrow=false,error;
   const created=construction(model,lane);
   try{entry.render(model,input.config,{...input.options,measurer:{width(){calls++;model.title='audit callback title';throw marker;}}});}
   catch(value){originalThrow=value===marker;if(!originalThrow)error={name:value?.name,message:value?.message};}
   row[lane]={construction:created,callback:{calls,titleRetained:model.title==='audit callback title',originalThrow,...(error?{error}:{})}};
  }finally{if(lane==='native'&&model)ready.disposeDiagramModel(model);}
 }
 row.callbackMatches=JSON.stringify(row.native.callback)===JSON.stringify(row.oracle.callback);rows.push(row);
}
if(!equal(packageBefore,hashes(pkg))||!equal(oracleBefore,hashes(oracle)))throw new Error('Probe inputs changed during execution');
console.log(JSON.stringify({kind:'public API correctness only; no timing',sourceCommit,package:packageBefore,oracle:oracleBefore,fixtureSHA256:sha(read(fixturePath)),scriptSHA256:sha(read(new URL(import.meta.url))),summary:{families:rows.length,callbackMatches:rows.filter(row=>row.callbackMatches).length,callbackMismatches:rows.filter(row=>!row.callbackMatches).map(row=>row.type),classRoots:rows.filter(row=>row.oracle.construction.classRoot).length,nativeClassConstructionFailures:rows.filter(row=>row.oracle.construction.classRoot&&!row.native.construction.ok).map(row=>row.type)},rows},null,2));
