import * as oracle from 'pele';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {loadRenderer} from '../source-typed-columns-runtime/candidate-package/full.js';
const packageRoot=new URL('../source-typed-columns-runtime/candidate-package/',import.meta.url);
const hash=x=>createHash('sha256').update(x).digest('hex');
const wasm=readFileSync(new URL('pkg/pele_bg.wasm',packageRoot));
const engine=await loadRenderer(wasm);
const view=new DataView(new ArrayBuffer(8));
const bits=x=>{view.setFloat64(0,x);return view.getBigUint64(0).toString(16).padStart(16,'0');};
function graph(root){const ids=new Map,objects=[],nodes=[];function atom(x){if(x===null)return['null'];if(typeof x==='number')return['number',bits(x)];if(typeof x==='function')return['function',x.length];if(typeof x==='object'){if(!ids.has(x)){ids.set(x,objects.length);objects.push(x);}return['ref',ids.get(x)];}return[typeof x,typeof x==='symbol'?x.description:x];}const rootAtom=atom(root);for(let i=0;i<objects.length;i++){const x=objects[i],kind=Array.isArray(x)?'array':x instanceof Map?'map':x instanceof Set?'set':'object';nodes.push({kind,properties:Reflect.ownKeys(x).map(k=>{const d=Object.getOwnPropertyDescriptor(x,k);return[k,d.enumerable,d.configurable,'value'in d?['data',d.writable,atom(d.value)]:['accessor',atom(d.get),atom(d.set)]];}),entries:x instanceof Map?[...x].map(([k,v])=>[atom(k),atom(v)]):[]});}return{root:rootAtom,nodes};}
const source='---\ntitle: fa:fa-user Original\n---\nsankey\nA,B,5\nB,C,3\nA,C,1';
const mutations={currentId:m=>m.nodes[0].id='Changed A',laterId:m=>m.nodes[1].id='Changed B with a deliberately long label that expands drawing bounds',value:m=>m.links[0].value=99,endpoint:m=>m.links[0].target=m.nodes[2],title:m=>m.title='Changed title',lookup:m=>m.lookup.clear(),combined:m=>{m.nodes[0].id='Changed A';m.nodes[1].id='Changed B with a deliberately long label that expands drawing bounds';m.links[0].value=99;m.links[0].target=m.nodes[2];m.title='Changed title';m.lookup.clear();}};
const rows=[];
try{
for(const resource of ['width','icon'])for(const [mutation,mutate]of Object.entries(mutations))for(const failure of ['none']){
 const row={resource,mutation,failure};
 for(const [lane,api]of [['oracle',oracle],['native',engine]]){
  const model=api.parse(source);const initial=graph(model);const sentinel=failure==='object'?{original:'callback'}:failure==='symbol'?Symbol('callback'):undefined;let fired=false;
  function draw(mutating){const trace=[];let output,threw=false,original=false,error;function action(){if(!mutating||fired)return;fired=true;mutate(model);if(failure!=='none')throw sentinel;}try{output=api.render(model,{maxWidth:220,measurer:{width(text,size,style){trace.push(['width',text,bits(size),style]);if(resource==='width')action();return text.length*size*.6;}},icons(name){trace.push(['icon',name]);if(resource==='icon')action();return '<path d="M0 0L1 1"/>';}});output={...output,width:bits(output.width),height:bits(output.height)};}catch(e){threw=true;original=Object.is(e,sentinel);error=original?{sentinel:failure}:{name:e?.name,message:e?.message,code:e?.code};}return{trace,output,threw,original,error};}
  try{const first=draw(true),after=graph(model),next=draw(false);row[lane]={initial,fired,first,after,next,afterNext:graph(model),aliases:model.links.every(l=>model.nodes.includes(l.source)&&model.nodes.includes(l.target))};}finally{if(lane==='native')engine.dispose(model);}
 }
 row.equal=Object.fromEntries(['initial','first','after','next','afterNext','aliases'].map(k=>[k,JSON.stringify(row.oracle[k])===JSON.stringify(row.native[k])]));rows.push(row);
}
}finally{engine.close();}
const report={source,rootCommit:'bb0fc18055a175de4645129ea25a4917a6fa7d17',node:process.version,oracle:hash(readFileSync('node_modules/pele/dist/pele.js')),packageManifest:hash(readFileSync(new URL('../source-typed-columns-runtime/package.json',import.meta.url))),wasm:hash(wasm),rows};
writeFileSync(new URL('./responsive.json',import.meta.url),JSON.stringify(report,null,2)+'\n');
const summary={cases:rows.length,initialAllMatch:rows.every(r=>r.equal.initial),callbackReached:rows.every(r=>r.oracle.fired&&r.native.fired),throwIdentity:rows.filter(r=>r.failure!=='none').every(r=>r.oracle.first.original&&r.native.first.original),byMutation:Object.keys(mutations).map(m=>({mutation:m,rows:rows.filter(r=>r.mutation===m).map(r=>({resource:r.resource,failure:r.failure,equal:r.equal}))}))};
writeFileSync(new URL('./responsive-summary.json',import.meta.url),JSON.stringify(summary,null,2)+'\n');
console.log(JSON.stringify({cases:summary.cases,initialAllMatch:summary.initialAllMatch,callbackReached:summary.callbackReached,throwIdentity:summary.throwIdentity,outputMismatches:rows.filter(r=>!r.equal.first).length,stateMismatches:rows.filter(r=>!r.equal.after).length,recoveryMismatches:rows.filter(r=>!r.equal.next).length}));
