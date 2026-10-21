import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const pinned='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pinned)throw Error('Unexpected Pele oracle');
const esm=path.resolve('node_modules/pele/dist/esm'),core=await import(pathToFileURL(esm+'/core.js'));
const corpus=JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json'));
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
function graph(value,C,result,opaque){const nodes=[],ids=new Map();function atom(v){if(opaque!==undefined&&v===opaque)return["host","options"];if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}
const obj=(...object)=>({object}),arr=(...array)=>({array});
const text=t=>obj(['text',t],['type','text']);
const definitions=[
 {family:'sequence',actions:[{name:'parseMessage',args:['hello']},{name:'addActor',args:['a','a',obj(['text','Alice']),undefined]},{name:'addActor',args:['b','b',obj(['text','Bob']),undefined]},{name:'addSignal',args:['a','b',obj(['text','Hi']),0]}]},
 {family:'block',actions:[{name:'generateId',args:[]},{name:'addStyleClass',args:['wide','fill:red,color:blue']},{name:'setHierarchy',args:[arr(obj(['id','a'],['type','square'],['label','Alpha']),obj(['id','b'],['type','square'],['label','Beta']))]}]},
 {family:'agentflow',actions:[{name:'addVertex',args:['a',text('Alpha'),'rect']},{name:'addVertex',args:['b',text('Beta'),'rect']},{name:'addLink',args:[arr('a'),arr('b'),obj(['type','arrow_point'],['stroke','normal'],['length',1])]}]},
];
function argument(value){if(value&&typeof value==='object'){if(value.object)return Object.fromEntries(value.object.map(([k,v])=>[k,argument(v)]));if(value.array)return value.array.map(argument);throw Error('Unknown argument');}return value;}
function error(e){return {name:e?.name,message:e?.message,...(e?.name==='PeleError'?{code:e.code,type:e.type??null,line:e.line??0,column:e.column??0,snippet:e.snippet??''}:{})};}
const cases=[];
for(const definition of definitions){
 const {family}=definition,entry=(await import(pathToFileURL(esm+'/diagrams/'+family+'.js'))).default;core.register(entry);
 const row=corpus.cases.find(c=>c.family===family),parsed=entry.parse(row.source,row.config,row.title??undefined,row.lineOffset,{}),C=Object.getPrototypeOf(parsed).constructor;
 const model=new C(),trace=[],resources={now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}};
 function raw(m){trace.length=0;try{const r=entry.render(m,{},resources);return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height),trace:trace.slice()};}catch(e){return{ok:false,error:error(e),trace:trace.slice()};}}
 const initial={graph:graph(model,C),render:raw(model)},actions=[];
 for(const action of definition.actions){const args=action.args.map(argument),result=model[action.name](...args);actions.push({...action,args:action.args.map(v=>v===undefined?{undefined:true}:v),graph:graph(model,C,result)});}
 const populated={graph:graph(model,C),render:raw(model)},fresh=new C();for(const action of definition.actions)fresh[action.name](...action.args.map(argument));const freshPopulated={graph:graph(fresh,C),render:raw(fresh)};
 const constructorCases=[];
 for(const kind of ['undefined','empty','values','firstThrow','undefinedThrow','secondThrow']){
  const reads=[],sentinel={family,kind},options=new Proxy({}, {ownKeys(){throw Error('Options enumerated');},get(_,key){reads.push(key);if(kind==='firstThrow'||kind==='undefinedThrow')throw kind==='undefinedThrow'?undefined:sentinel;if(kind==='secondThrow'&&reads.length===2)throw sentinel;
    if(kind==='values'){if(key==='wrap')return true;if(key==='sequenceWrap')return false;if(key==='lineOffset')return 9;if(key==='warn')return function warn(message){};}return undefined;}});
  const extra=new Proxy({},{get(){throw Error('Extra argument read');},ownKeys(){throw Error('Extra argument enumerated');}});
  let result,thrown=false,caught;try{const first=kind==='undefined'?undefined:options;const value=new C(first,extra);result=graph(value,C,undefined,family==='agentflow'&&first!==undefined?options:undefined);}catch(e){thrown=true;caught=e===sentinel?'original-object':e===undefined?'original-undefined':error(e);}
  constructorCases.push({kind,reads,thrown,...(thrown?{error:caught}:{graph:result})});
 }
 let coreError;try{core.render(model,resources);}catch(e){coreError=error(e);}
 cases.push({family,prototype:1,initial,actions,populated,freshPopulated,constructorCases,coreError});
}
const report={oracleSHA256:pinned,revision:corpus.revision,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-ordered-constructors.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(cases.map(c=>({family:c.family,actions:c.actions.length,initial:c.initial.render.ok,populated:c.populated.render.ok,fresh:c.freshPopulated.render.ok,constructorCases:c.constructorCases.length}))));
