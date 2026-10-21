// Pinned Pele public constructors: defaults, helpers, aliases, and raw render.
// node scripts/parity/capture-public-next-six-constructors.mjs [output-fixture]
// Argument objects use ordered entry lists, preserving public own-key order.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const pinned='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pinned)throw Error('Unexpected Pele oracle');
const esm=path.resolve('node_modules/pele/dist/esm');
function hashes(){const out={};function visit(dir){for(const e of fs.readdirSync(dir,{withFileTypes:true})){const p=path.join(dir,e.name);if(e.isDirectory())visit(p);else out[path.relative(esm,p)]=sha(fs.readFileSync(p));}}visit(esm);return out;}
const esmFiles=hashes(),core=await import(pathToFileURL(esm+'/core.js'));
const corpus=JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json'));
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
// Same alias/descriptor/IEEE normalization as capture-public-sankey-constructor,
// extended with Set and a helper result traversed after the model root.
function graph(value,C,result){const nodes=[],ids=new Map();function atom(v){if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}
const obj=(...object)=>({object}),arr=(...array)=>({array});
const xytext=t=>obj(['text',t],['type','text']);
const cases=[
 {family:'architecture',actions:[{name:'addService',args:[obj(['id','a'],['title','Alpha'])],rememberPath:['nodes'],remember:'nodes'},{name:'addService',args:[obj(['id','b'],['title','Beta'])],samePath:['nodes'],sameAs:'nodes'}]},
 {family:'kanban',actions:[{name:'addNode',args:[0,'todo','Todo',0],rememberPath:['sections'],remember:'sections'},{name:'addNode',args:[1,'card','Card',0],samePath:['sections'],sameAs:'sections'},{name:'getType',args:['[',']']}]},
 {family:'mindmap',actions:[{name:'addNode',args:[0,'root','Root',0],rememberPath:['nodes'],remember:'nodes'},{name:'addNode',args:[1,'child','Child',0],samePath:['nodes'],sameAs:'nodes'},{name:'getType',args:['[',']']}]},
 {family:'timeline',actions:[{name:'addSection',args:['Era']},{name:'addTask',args:['2026',1,'Start'],rememberPath:['periods'],remember:'periods'},{name:'addEvent',args:['Next'],samePath:['periods'],sameAs:'periods'}]},
 {family:'journey',actions:[{name:'addSection',args:['Day']},{name:'addTask',args:['Work',': 5: Alice, Bob'],rememberPath:['tasks'],remember:'tasks'},{name:'addTask',args:['Rest',': 3: Alice'],samePath:['tasks'],sameAs:'tasks'}]},
 {family:'quadrantChart',actions:[{name:'setQuadrant1Text',args:[xytext('High')],rememberPath:['quadrants'],remember:'quadrants'},{name:'setQuadrant2Text',args:[xytext('Low')],samePath:['quadrants'],sameAs:'quadrants'},{name:'addPoint',args:[xytext('A'),'',0.2,0.7,'']}]},
];
function arg(v){if(v&&typeof v==='object'){if(v.array)return v.array.map(arg);if(v.object)return Object.fromEntries(v.object.map(([k,x])=>[k,arg(x)]));throw Error('Bad fixture argument');}return v;}
const error=e=>e?.name==='PeleError'?{name:e.name,message:e.message,code:e.code,type:e.type??null,line:e.line??0,column:e.column??0,snippet:e.snippet??''}:{name:e?.name,message:e?.message};
const output=[],sharedFunctions=new Map();
for(const definition of cases){const {family}=definition,entry=(await import(pathToFileURL(esm+'/diagrams/'+family+'.js'))).default;core.register(entry);const source=corpus.cases.find(c=>c.family===family);const parsed=entry.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});const C=Object.getPrototypeOf(parsed).constructor;const ignored=new Proxy({},{get(){throw Error('Arguments must be ignored');},ownKeys(){throw Error('Arguments must not be enumerated');}});const model=new C(ignored,Symbol('extra'));
 const trace=[],resources={now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}};
 function raw(){trace.length=0;try{const r=entry.render(model,{},resources);return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height),trace:trace.slice()};}catch(e){return{ok:false,error:error(e),trace:trace.slice()};}}
 if(['mindmap','kanban'].includes(family))sharedFunctions.set(family,model.getType);
 const initial={graph:graph(model,C),render:raw()},saved=new Map(),actions=[];
 for(const action of definition.actions){const result=model[action.name](...action.args.map(arg));const selected=(action.rememberPath??action.samePath)?.reduce((v,k)=>v[k],model)??result;const row={...action,graph:graph(model,C,result)};if(action.remember){if(!selected||typeof selected!=='object')throw Error(`${family}: vacuous remembered alias`);saved.set(action.remember,selected);}if(action.sameAs){row.sameIdentity=selected===saved.get(action.sameAs);if(!row.sameIdentity)throw Error(`${family}: repeated alias differs`);}actions.push(row);}
 const populated={graph:graph(model,C),render:raw()};let coreError;try{core.render(model,resources);}catch(e){coreError=error(e);}if(coreError?.name!=='TypeError')throw Error(`${family}: constructor gained parse provenance`);
 if(!populated.render.ok)throw Error(`${family}: populated raw failed: ${JSON.stringify(populated.render.error)}`);
 output.push({family,prototype:1,initial,actions,populated,coreError});
}
if(JSON.stringify(esmFiles)!==JSON.stringify(hashes()))throw Error('Oracle changed while capturing');
if(sharedFunctions.get('mindmap')!==sharedFunctions.get('kanban'))throw Error('Shared getType identity differs');
const report={sharedGetType:true,revision:corpus.revision,oracleSHA256:pinned,esmFiles,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),measurement:'text.length * size * .6',cases:output};
fs.writeFileSync(process.argv[2]??'fixtures/public-next-six-constructors.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({families:output.length,actions:output.reduce((n,c)=>n+c.actions.length,0),aliases:output.reduce((n,c)=>n+c.actions.filter(a=>a.sameIdentity).length,0),rawSuccess:output.reduce((n,c)=>n+Number(c.initial.render.ok)+Number(c.populated.render.ok),0),rawErrors:output.filter(c=>!c.initial.render.ok).map(c=>[c.family,c.initial.render.error])}));
