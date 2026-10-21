import fs from 'node:fs';import path from 'node:path';import{pathToFileURL}from'node:url';import{createHash}from'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex'),pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pin)throw Error('Oracle mismatch');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/agentflow.js')))).default;
const row=JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json')).cases.find(c=>c.family==='agentflow');
const C=entry.parse(row.source,row.config,undefined,0,{maxEdges:undefined}).constructor;
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
function state(m){return structuredClone({diagnostics:m.diagnostics,mappings:m.mappings,reported:[...m.reported],mappingAlias:m.diagnostics.map(d=>!d.position||d.position===m.firstMapping.get(d.nodeId??d.edgeId)?.position),reportedAlias:m.diagnostics.map(d=>m.reported.get(d.nodeId??d.edgeId??'').includes(d))});}
function raw(m){const trace=[];try{const r=entry.render(m,{}, {measurer:{width(t,s){trace.push([t,bits(s)]);return t.length*s*.6;}}});return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height),trace};}catch(e){return{ok:false,name:e.name,message:e.message,code:e.code,trace};}}
const cases=[];
for(const mode of ['removed','unsupported','mixed','writable-length','readonly','shadowed','plain','copied-emitter','missing-endpoint']){
 const m=new C(),actions=[];function call(name,...args){actions.push({name,args});return m[name](...args);}
 if(mode==='missing-endpoint'){call('addSingleLink','Missing','Other',{type:'arrow_point',stroke:'normal',length:1});}
 else {call('addVertex','A',{text:'Alpha',type:'text'},mode==='unsupported'?'cloud':'circle');call('addVertexMapping','A',null,null,{first_line:2,first_column:3,last_line:2,last_column:9});if(mode==='mixed'){call('addVertex','B',{text:'Beta',type:'text'},'cloud');call('addSingleLink','A','B',{type:'arrow_point',stroke:'normal',length:1});}}
 if(mode==='readonly')Object.defineProperty(m.diagnostics,'length',{writable:false});
 if(mode==='shadowed')m.emitError=null;
 const receiver=mode==='plain'||mode==='copied-emitter'?{...m}:m;
 if(mode==='copied-emitter')receiver.emitError=m.emitError;
 const first=raw(receiver),after=state(m),second=first.ok?raw(receiver):null,afterSecond=state(m);
 if(mode==='readonly')m.diagnostics=[...m.diagnostics];
 if(mode==='shadowed')delete m.emitError;
 const recovered=raw(m),afterRecovery=state(m);
 cases.push({mode,actions,first,after,second,afterSecond,recovered,afterRecovery});
}
const report={oracleSHA256:pin,chunkSHA256:sha(fs.readFileSync('node_modules/pele/dist/esm/chunks/RLD435X7.js')),scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-agentflow-render-diagnostics.json',JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(cases.map(c=>({mode:c.mode,first:c.first.ok,error:c.first.message,count:c.after.diagnostics.length,recovery:c.recovered.ok}))));
