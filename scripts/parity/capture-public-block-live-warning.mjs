import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pin)throw Error('Oracle mismatch');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/block.js')))).default;
const C=entry.parse('block\na["Alpha"]',{},undefined,0,{}).constructor;
const statements=()=>[{type:'column-setting',columns:2},{id:'A',type:'square',widthInColumns:4},{id:'B',type:'square',widthInColumns:3},{id:'G',type:'composite',widthInColumns:3,children:[{type:'column-setting',columns:1},{id:'C',type:'square',widthInColumns:2}]}];
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
function state(d,s){return structuredClone({root:d.root,blocks:d.blocks,edges:d.edges,byId:[...d.byId],pending:[...d.pending],ids:d.ids,colors:d.colors,spaces:d.spaces,statements:s,blocksAliasRoot:d.blocks===d.root.children,byIdAlias:s.filter(n=>n.id&&d.byId.has(n.id)).map(n=>[n.id,d.byId.get(n.id)===n])});}
function raw(d){const trace=[];try{const r=entry.render(d,{}, {measurer:{width(t,s){trace.push([t,bits(s)]);return t.length*s*.6;}}});return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height),trace};}catch(e){return{ok:false,error:{name:e.name,message:e.message,code:e.code},trace};}}
const cases=[];
for(const mode of ['normal','throw','getter-throw','getter-mutate','mutate','nested','noncallable']){
 let db,list=statements(),calls=0,gets=0;const sentinel={mode},trace=[];
 function warn(message){calls++;trace.push(['warn',this===db,message,state(db,list)]);if(mode==='throw')throw sentinel;if(mode==='mutate'&&calls===1){list[1].type='column-setting';list[1].columns=4;}if(mode==='mutate'&&calls===2){list[2].label='Changed';db.named('Pending');}if(mode==='nested'&&calls===1)db.setHierarchy([{id:'Nested',type:'square',widthInColumns:1}]);return new Proxy({}, {get(){throw Error('Discarded result inspected');}});}
 db=new C({warn});
 if(mode.startsWith('getter-'))Object.defineProperty(db,'warn',{configurable:true,enumerable:true,get(){gets++;trace.push(['get',gets,state(db,list)]);if(mode==='getter-throw')throw sentinel;if(gets===1)list[1].widthInColumns=9;return warn;}});
 if(mode==='noncallable')db.warn=42;
 let result;try{const value=db.setHierarchy(list);result={ok:true,undefined:value===undefined};}catch(e){result={ok:false,original:e===sentinel,...(e===sentinel?{}:{name:e.name,message:e.message})};}
 const after=state(db,list),firstRaw=result.ok?raw(db):null;
 Object.defineProperty(db,'warn',{value:undefined,writable:true,configurable:true,enumerable:true});
 db.setHierarchy(list);
 cases.push({mode,result,trace,after,firstRaw,recovered:state(db,list),recoveryRaw:raw(db)});
}
const report={oracleSHA256:pin,chunkSHA256:sha(fs.readFileSync('node_modules/pele/dist/esm/chunks/XFYV3ZRT.js')),scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-block-live-warning.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(cases.map(x=>({mode:x.mode,ok:x.result.ok,trace:x.trace.length,recovery:x.recoveryRaw.ok}))));
