import fs from 'node:fs';import path from 'node:path';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const oracle='node_modules/pele/dist/pele.js',oracleSHA256=sha(fs.readFileSync(oracle));
if(oracleSHA256!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw Error('Unexpected Pele');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/gantt.js')))).default;
const C=Object.getPrototypeOf(entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :t, 2026-01-01, 1d',{},undefined,0,{})).constructor;
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
function graph(value,C,result,options){const nodes=[],ids=new Map();function atom(v){if(v===options)return["host","options"];if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}

const cases=[];
for(const id of ['known-only','shadowed-known','shadowed-warn','warn-false','normal','first-get-throw','second-get-throw','second-not-callable','keyword-throw','keyword-reentry','warn-reentry','warn-throw']) {
 let model,reads=0;const trace=[],sentinel={id},options={};
 const taskState=()=>model.tasks.map(t=>[t.id,t.task]);
 function callback(message){trace.push(['call',this===options,Array.from({length:message.length},(_,i)=>message.charCodeAt(i))]);if(id==='warn-reentry'){model.addTask('Resolved',':missing,2024-01-01,1d');model.tasks[0].task='Changed';}if(id==='warn-throw')throw sentinel;}
 Object.defineProperty(options,'warn',{get(){trace.push(['get',++reads]);if((id==='first-get-throw'&&reads===1)||(id==='second-get-throw'&&reads===2))throw sentinel;return id==='warn-false'?false:id==='second-not-callable'&&reads===2?0:callback;},configurable:true});
 model=new C(options);model.addTask('Alpha',':a,2024-01-01,1d');model.addTask('Beta',':b,2024-01-02,1d');
 const originalA=model.tasks[0];
 const keyword={toString(){trace.push(['keyword']);if(id==='keyword-throw')throw sentinel;if(id==='keyword-reentry'){model.addSection('Nested');Object.defineProperty(options,'warn',{value:()=>{throw Error('replaced callback called')},configurable:true});}return 'after\ud800';}};
 if(id==='shadowed-known'||id==='shadowed-warn')model.warn=null;
 const list=id==='known-only'||id==='shadowed-known'?'a b a':' a missing b missing  a ';
 let result,error;try{result=model.references(keyword,list);}catch(e){error=e===sentinel?'original':{name:e.name,message:e.message};}
 const first={graph:graph(model,C,result,options),trace:trace.slice(),error:error??null,returnedAlias:result?result[0]===originalA:null};
 Object.defineProperty(options,'warn',{value:undefined,configurable:true});
 if(id==='shadowed-known'||id==='shadowed-warn')delete model.warn;
 const recovery=model.references('after',list);
 cases.push({id,list,first,recovery:{graph:graph(model,C,recovery,options),returnedAlias:recovery[0]===originalA}});
}
const report={oracleSHA256,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),referencesBody:C.prototype.references.toString(),warnBody:C.prototype.warn.toString(),cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-gantt-reference-warnings.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(cases.map(c=>({id:c.id,error:c.first.error,trace:c.first.trace}))));
