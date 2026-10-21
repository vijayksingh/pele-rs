import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=x=>createHash('sha256').update(x).digest('hex');
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pin)throw Error('Oracle pin mismatch');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/agentflow.js')))).default;
const source=JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json')).cases.find(c=>c.family==='agentflow');
const C=entry.parse(source.source,source.config,undefined,0,{maxEdges:undefined}).constructor;
const resources={measurer:{width:(t,s)=>t.length*s*.6}};
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const raw=m=>{try{const r=entry.render(m,{},resources);return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height)};}catch(e){return{ok:false,name:e.name,message:e.message,code:e.code};}};
const db=new C();db.addVertex('A',{text:'Alpha',type:'text'},'rect');const first=raw(db);db.addVertex('B',{text:'Beta',type:'text'},'rect');const second=raw(db);const fresh=new C();fresh.addVertex('A',{text:'Alpha',type:'text'},'rect');fresh.addVertex('B',{text:'Beta',type:'text'},'rect');const freshBoth=raw(fresh);
const cases=[];
for(const mode of ['normal','max-throw','inherit-throw','log-throw','log-reentry']){
 const trace=[],sentinel={mode};let model;
 const options={get lineOffset(){trace.push(['get','lineOffset']);return 0;},get maxEdges(){trace.push(['get','maxEdges',model.edges.length]);if(mode==='max-throw')throw sentinel;return 2;},get inheritDir(){trace.push(['get','inheritDir',model.subCount]);if(mode==='inherit-throw')throw sentinel;return true;},get direction(){trace.push(['get','direction']);return 'LR';},get log(){trace.push(['get','log',model.diagnostics.length]);return function(...args){trace.push(['call','log',this===options,...args]);if(mode==='log-throw')throw sentinel;if(mode==='log-reentry')model.addVertex('Nested');};}};
 model=new C(options);model.addVertex('A');model.addVertex('B');
 const operations=[];
 for(const [name,args]of [['addSingleLink',['A','B',{type:'arrow_point',stroke:'normal',length:1}]],['addSubGraph',[{text:'G'},['A','B'],{text:'Group',type:'text'}]],['emitWarning',['test','message',{nodeId:'A'}]]]){
  try{operations.push({name,value:model[name](...args),ok:true});}catch(e){operations.push({name,ok:false,original:e===sentinel,error:{name:e?.name,message:e?.message,code:e?.code}});}
 }
 cases.push({mode,trace,operations,state:{nodes:[...model.nodes.keys()],edges:model.edges.length,subCount:model.subCount,subgraphs:model.subgraphs,diagnostics:model.diagnostics}});
}
const rawGuards=[];
for(const kind of ['plain','shadowed']) {
 const model=new C();model.addVertex('A',{text:'Alpha',type:'text'},'rect');
 const receiver=kind==='plain'?{...model}:model;
 if(kind==='shadowed')receiver.vertexKind=null;
 rawGuards.push({kind,result:raw(receiver)});
}
const report={rawGuards,oracleSHA256:pin,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),cache:{first,second,freshBoth,sameAfterEdit:first.svg===second.svg,newOwnerDiffers:second.svg!==freshBoth.svg},cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-agentflow-live-options.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({cacheSame:report.cache.sameAfterEdit,freshDiffers:report.cache.newOwnerDiffers,cases:cases.length}));
