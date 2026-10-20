import * as pele from 'pele';
import {writeFileSync,readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
const source='sankey\nA,B,5\nB,C,3\nA,C,1';
const rows=[];
const hash=x=>createHash('sha256').update(x).digest('hex');
for(const resource of ['width','icon']) for(const kind of ['success','object','symbol','undefined']){
  const model=pele.parse(resource==='icon'?'---\ntitle: fa:fa-user Energy\n---\n'+source:source);
  const first=model.nodes[0],second=model.nodes[1],lookup=model.lookup;
  const extra={n:1};extra.self=extra;model.extra=extra;
  const detached=[first];model.detached=detached;
  const sentinel=kind==='object'?{original:'sentinel'}:kind==='symbol'?Symbol('sentinel'):undefined;
  const trace=[];let calls=0,threw=false,original=false,output;
  function mutate(){calls++;if(calls!==1)return;extra.n=99;extra.added=first;delete model.extra;delete model.detached;detached[0]=second;first.id='Changed A';second.id='Changed B';lookup.clear();Object.defineProperty(first,'callbackField',{value:extra,writable:false,enumerable:false,configurable:true});if(kind!=='success')throw sentinel;}
  try{output=pele.render(model,{measurer:{width(text,size,style){trace.push(['width',text,size,style]);if(resource==='width')mutate();return text.length*size*.6;}},icons(name){trace.push(['icon',name]);if(resource==='icon')mutate();return '<path d="M0 0L1 1"/>';}});}catch(error){threw=true;original=Object.is(error,sentinel);if(!original)trace.push(['unexpected',String(error)]);}
  const descriptor=Object.getOwnPropertyDescriptor(first,'callbackField');
  const state={firstId:first.id,secondId:second.id,lookupSize:lookup.size,extraAbsent:!Object.hasOwn(model,'extra'),detachedAbsent:!Object.hasOwn(model,'detached'),extraValue:extra.n,extraAlias:extra.added===first,detachedAlias:detached[0]===second,cycle:extra.self===extra,modelAliases:model.nodes[0]===first&&model.links[0].source===first&&model.links[0].target===second,descriptor:descriptor?{writable:descriptor.writable,enumerable:descriptor.enumerable,configurable:descriptor.configurable,valueAlias:descriptor.value===extra}:null};
  rows.push({resource,kind,calls,threw,original,trace,state,output:output?{width:output.width,height:output.height,svgSha256:hash(output.svg),changedA:output.svg.includes('Changed A'),changedB:output.svg.includes('Changed B')}:null});
}
const helperModel=pele.parse(source),nodes=helperModel.nodes,lookup=helperModel.lookup;
Object.defineProperty(nodes,'length',{writable:false});let helperError;
try{helperModel.findOrCreateNode('X');}catch(e){helperError={name:e.name,message:e.message};}
const helper={error:helperError,lookupHasX:lookup.has('X'),index:lookup.get('X')?.index,nodeCount:nodes.length,absentFromArray:!nodes.includes(lookup.get('X')),retrySameAlias:helperModel.findOrCreateNode('X')===lookup.get('X')};
const paths=['node_modules/pele/package.json','node_modules/pele/dist/pele.js','dist/evidence/pele-oracle.js','node_modules/pele/dist/esm/chunks/CTRBU7NA.js','node_modules/pele/dist/esm/chunks/ESCH476E.js','node_modules/pele/dist/esm/chunks/ND66TM4A.js'];
const evidence={node:process.version,package:JSON.parse(readFileSync(paths[0])).version,source,files:Object.fromEntries(paths.map(p=>[p,hash(readFileSync(p))])),rows,helper};
writeFileSync(new URL('./report.json',import.meta.url),JSON.stringify(evidence,null,2)+'\n');
// Assertions follow the persisted raw report; no timing or benchmark work.
for(const row of rows){assert.ok(row.calls>0,row.resource+' callback reached');assert.equal(row.threw,row.kind!=='success');if(row.threw)assert.equal(row.original,true);assert.deepEqual(row.state,{firstId:'Changed A',secondId:'Changed B',lookupSize:0,extraAbsent:true,detachedAbsent:true,extraValue:99,extraAlias:true,detachedAlias:true,cycle:true,modelAliases:true,descriptor:{writable:false,enumerable:false,configurable:true,valueAlias:true}});}
assert.equal(helper.error.name,'TypeError');assert.equal(helper.lookupHasX,true);assert.equal(helper.absentFromArray,true);assert.equal(helper.retrySameAlias,true);
console.log(JSON.stringify({rows:rows.length,allCallbackWritesPersist:true,originalThrows:rows.filter(x=>x.threw).length,partialHelper:helper}));
