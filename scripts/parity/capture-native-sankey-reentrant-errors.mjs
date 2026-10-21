// Public pinned-Pele fixtures; never builds or executes the native package.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const fixturePath=resolve('fixtures/native-sankey-callbacks.json');
const fixtureBytes=await readFile(fixturePath),fixture=JSON.parse(fixtureBytes);
const oraclePath=resolve('node_modules/pele/dist/pele.js');
const oracleSHA256=sha(await readFile(oraclePath));
assert.equal(oracleSHA256,fixture.oracleSHA256);
assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const oracle=await import(pathToFileURL(oraclePath));
const pure=(text,size)=>text.length*size*.6;
const summary=value=>({svg:value.svg,svgSHA256:sha(Buffer.from(value.svg)),width:value.width,height:value.height,type:value.type,links:value.links});
const describe=model=>({nodes:model.nodes.map(n=>({id:n.id,index:n.index})),lookup:[...model.lookup].map(([key,n])=>({key,id:n.id,index:n.index,nodeIndex:model.nodes.indexOf(n)})),lengthDescriptor:Object.getOwnPropertyDescriptor(model.nodes,'length')});
const rows=[];
for(const mode of ['partial-helper-readonly-length','nested-render-original-throw']){
 const model=oracle.parse(fixture.source),row={mode,trace:[],measurements:[],initial:describe(model)};
 let entered=false,orphan;const originalNodes=model.nodes,sentinel={failure:'original nested measurer'};
 const measure=phase=>(text,size,style)=>{
  row.measurements.push({phase,text,size,style});row.trace.push({event:'width',phase,text,size,style});
  return pure(text,size);
 };
 const width=(text,size,style)=>{
  const answer=measure('outer')(text,size,style);
  if(!entered){entered=true;
   if(mode==='partial-helper-readonly-length'){
    Object.defineProperty(model.nodes,'length',{writable:false});
    row.trace.push({event:'helper-call',method:'findOrCreateNode',argument:'D'});
    try{model.findOrCreateNode('D');row.helperReturned=true;}
    catch(error){row.helperError={name:error.name,message:error.message};row.trace.push({event:'helper-caught',...row.helperError});}
    orphan=model.lookup.get('D');row.afterCaught=describe(model);
    row.partial={lookupContainsD:model.lookup.has('D'),nodesContainD:model.nodes.includes(orphan),originalArray:model.nodes===originalNodes,orphan:orphan&&{id:orphan.id,index:orphan.index}};
   }else{
    row.trace.push({event:'nested-render-call'});
    try{row.nestedResult=summary(oracle.render(model,{measurer:{width(innerText,innerSize,innerStyle){measure('inner')(innerText,innerSize,innerStyle);throw sentinel;}}}));}
    catch(error){row.innerOriginal=error===sentinel;row.innerThrownType=typeof error;row.trace.push({event:'nested-render-caught',original:row.innerOriginal});}
   }
  }
  return answer;
 };
 try{row.first=summary(oracle.render(model,{measurer:{width}}));row.trace.push({event:'outer-complete'});}
 catch(error){row.outerError={name:error?.name,message:error?.message,originalInner:error===sentinel};row.trace.push({event:'outer-threw',...row.outerError});}
 row.afterFirst=describe(model);
 if(mode==='partial-helper-readonly-length'){
  model.nodes=[...model.nodes,orphan];
  row.repair={newArray:model.nodes!==originalNodes,oldArrayLength:originalNodes.length,oldLengthWritable:Object.getOwnPropertyDescriptor(originalNodes,'length').writable,orphanAlias:model.nodes[3]===model.lookup.get('D')};
  row.afterRepair=describe(model);row.trace.push({event:'repair-nodes-array'});
 }
 try{row.recovery=summary(oracle.render(model,{measurer:{width:measure('recovery')}}));row.trace.push({event:'recovery-complete'});}
 catch(error){row.recoveryError={name:error?.name,message:error?.message};}
 row.afterRecovery=describe(model);rows.push(row);
}
const output=new URL('../../fixtures/native-sankey-reentrant-errors.json',import.meta.url);
const report={oracleRevision:fixture.oracleRevision,oracleSHA256,inputFixtureSHA256:sha(fixtureBytes),source:fixture.source,measurement:'text.length * size * 0.6',scriptSHA256:sha(await readFile(new URL(import.meta.url))),rows};
// Persist every observed outcome before non-vacuity assertions.
await writeFile(output,JSON.stringify(report,null,2)+'\n');
const partial=rows[0],nested=rows[1];
assert.equal(partial.helperError?.name,'TypeError');assert.equal(partial.helperReturned,undefined);
assert.deepEqual(partial.partial,{lookupContainsD:true,nodesContainD:false,originalArray:true,orphan:{id:'D',index:3}});
assert.equal(partial.afterFirst.nodes.length,3);assert.equal(partial.first.type,'sankey');
assert.equal(partial.repair.orphanAlias,true);assert.equal(partial.afterRecovery.nodes.length,4);assert.equal(partial.recovery.type,'sankey');
assert.equal(nested.innerOriginal,true);assert.equal(nested.nestedResult,undefined);assert.equal(nested.outerError,undefined);
assert.equal(nested.measurements.filter(x=>x.phase==='inner').length,1);assert(nested.measurements.filter(x=>x.phase==='outer').length>1);
assert.equal(nested.first.type,'sankey');assert.equal(nested.recovery.type,'sankey');assert.equal(nested.first.svg,nested.recovery.svg);
assert.equal(sha(await readFile(fixturePath)),report.inputFixtureSHA256);assert.equal(sha(await readFile(oraclePath)),oracleSHA256);
console.log(JSON.stringify({cases:rows.length,oracleSHA256,reportSHA256:sha(await readFile(output)),helperError:partial.helperError,outerWidths:rows.map(r=>r.measurements.filter(x=>x.phase==='outer').length),innerOriginal:nested.innerOriginal,recoveryWidths:rows.map(r=>r.measurements.filter(x=>x.phase==='recovery').length)}));
