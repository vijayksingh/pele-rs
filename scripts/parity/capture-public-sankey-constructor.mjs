// Exact public constructor/default/helper oracle. Pele 458322a5 (MIT, kepano).
// Run from repository root; regenerates only fixtures/public-sankey-constructor.json.
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
const sha=b=>createHash('sha256').update(b).digest('hex');
const full=fs.readFileSync('node_modules/pele/dist/pele.js');
const pinned='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(full)!==pinned)throw Error('Unexpected pinned Pele bundle');
const root=path.resolve('node_modules/pele/dist/esm');
const core=await import(pathToFileURL(root+'/core.js'));
const entry=(await import(pathToFileURL(root+'/diagrams/sankey.js'))).default;
core.register(entry);
const C=Object.getPrototypeOf(entry.parse('sankey\nA,B,1',{},undefined,0)).constructor;
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
function graph(value){const nodes=[],ids=new Map();function atom(v){if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(ids.has(v))return['ref',ids.get(v)];const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}return{root:atom(value),nodes};}
const trace=[];const resources={now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}};
const draw=m=>{trace.length=0;const r=entry.render(m,{},resources);return{svg:r.svg,width:bits(r.width),height:bits(r.height),trace:trace.slice()};};
const ignored=new Proxy({},{get(){throw Error('Constructor must ignore arguments');},ownKeys(){throw Error('Constructor must not enumerate arguments');}});
const m=new C(ignored,Symbol('extra'));
const initial={graph:graph(m),render:draw(m)};
const a=m.findOrCreateNode('A'),b=m.findOrCreateNode('B');m.addLink(a,b,2);
const populated={graph:graph(m),render:draw(m),sameNode:m.findOrCreateNode('A')===a};
Object.defineProperty(m.nodes,'length',{writable:false});let failure;
try{m.findOrCreateNode('D');throw Error('Expected readonly length failure');}catch(e){failure={name:e.name,message:e.message};}
const partial={graph:graph(m),failure,orphan:m.lookup.get('D')!==undefined&&!m.nodes.includes(m.lookup.get('D'))};
m.nodes=[...m.nodes,m.lookup.get('D')];
const repaired={graph:graph(m),render:draw(m)};
let coreError;try{core.render(m,resources);}catch(e){coreError={name:e.name,message:e.message};}
const output={revision:'458322a5b8abb1ae57ddf127c32017ab9fb14a92',oracleSHA256:pinned,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),measurement:'text.length * size * .6',initial,populated,partial,repaired,coreError};
fs.writeFileSync('fixtures/public-sankey-constructor.json',JSON.stringify(output,null,2)+'\n');
if(!populated.sameNode||!partial.orphan||coreError?.name!=='TypeError')throw Error('Oracle feature was vacuous');
console.log('Captured public constructor, helper partial failure, raw render and repair.');
