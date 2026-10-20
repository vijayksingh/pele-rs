import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('disposal retains caller values and borrowed helpers while releasing document ownership',async({page})=>{
 await page.goto('/examples/web/');
 const failures=await page.evaluate(async packageRoot=>{
  const {loadRenderer}=await import(packageRoot+'/full.js'),oracle=await import('/dist/evidence/pele-oracle.js');
  const engine=await loadRenderer(),failures=[];
  const check=(value,label)=>{if(!value)failures.push(label)},throws=(run,message)=>{try{run();return false}catch(error){return error instanceof TypeError&&error.message.includes(message)}};
  const source='flowchart LR\nA[Alpha]-->B[Beta]',model=engine.parse(source),reference=oracle.parse(source),live=engine.parse(source);
  const nodes=model.nodes,first=nodes.get('A'),edges=model.edges,edge=edges[0],prototype=Object.getPrototypeOf(model),method=model.addVertex;
  const date=new Date(NaN),extra=new Map([[first,date]]);Object.defineProperty(model,'retained',{value:extra,enumerable:false,writable:false,configurable:false});
  const before=Object.getOwnPropertyDescriptors(model),keys=Reflect.ownKeys(model);
  engine.dispose(model);engine.dispose(model);
  check(Object.getPrototypeOf(model)===prototype&&Reflect.ownKeys(model).every((key,i)=>key===keys[i])&&Reflect.ownKeys(model).length===keys.length,'root shape');
  for(const key of keys){const after=Object.getOwnPropertyDescriptor(model,key),old=before[key];check(after.enumerable===old.enumerable&&after.writable===old.writable&&after.configurable===old.configurable&&Object.is(after.value,old.value),'descriptor '+String(key))}
  check(model.nodes===nodes&&nodes.get('A')===first&&model.edges===edges&&edges[0]===edge,'retained aliases');
  check(model.retained===extra&&extra.get(first)===date&&Number.isNaN(+date),'Map and Date retained');
  check(throws(()=>engine.render(model),'takes diagram text'),'disposed core render');
  check(throws(()=>engine.renderRaw('flowchart',model,{}),'disposed'),'disposed raw render');
  const args=['C',{text:'Native',type:'text'},'square',[],[]];
  method.call(model,...args);reference.addVertex(...args);
  check(model.nodes===nodes&&nodes.get('C').text===reference.nodes.get('C').text,'borrowed mutating helper on disposed receiver');
  engine.dispose(live);
  check(throws(()=>method.call(model,...args),'no live document'),'helper without live owner');
  const kanban=engine.parse('kanban\n Todo'),mindmap=engine.parse('mindmap\n root((Core))'),helper=kanban.getType;
  engine.dispose(kanban);check(helper===mindmap.getType&&helper.call(kanban,'[',']')===2,'cross-schema shared helper');engine.dispose(mindmap);
  check(throws(()=>helper.call(null,'[',']'),'no live document'),'shared helper without live owner');
  const active=engine.parse(source);let blocked=false;
  engine.render(active,{measurer:{width(text,size){blocked=throws(()=>engine.dispose(active),'already in use')||blocked;return text.length*size*.6}}});
  check(blocked&&engine.render(active).svg.includes('Alpha'),'active disposal rejects and recovers');engine.dispose(active);
  return failures;
 },process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package');
 expect(failures).toEqual([]);
});

test('disposed public models release their roots while caller-held nodes survive',async({page,browserName},info)=>{
 test.skip(browserName!=='chromium','Explicit garbage collection requires Chromium CDP');
 await page.goto('/examples/web/');
 await page.evaluate(async packageRoot=>{
  const {loadRenderer}=await import(packageRoot+'/full.js'),engine=await loadRenderer();
  const rows=[];
  // A retained leaf keeps its owning bridge reachable. It has no edge back to
  // the public root, so disposal must release that root independently.
  for(const stage of ['parsed','rendered','helper']){
   const model=engine.parse('sankey\nA,B,5\nB,C,3');
   if(stage==='rendered')engine.render(model);
   if(stage==='helper')model.findOrCreateNode('Detached');
   const leaf=model.nodes[0],reference=new WeakRef(model),prototype=Object.getPrototypeOf(model);
   engine.dispose(model);
   rows.push({stage,leaf,reference,prototype});
  }
  globalThis.__disposalReachability={engine,rows};
 },process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package');
 // WeakRef construction pins its target for that JS job. Collect only after
 // returning to the browser, and never dereference the root before collection.
 const session=await page.context().newCDPSession(page);
 try{
  await session.send('HeapProfiler.collectGarbage');
  const report=await page.evaluate(()=>{
   const state=globalThis.__disposalReachability;
   return state.rows.map(({stage,leaf,reference,prototype})=>({stage,rootCollected:reference.deref()===undefined,leafId:leaf.id,leafIndex:leaf.index,helperRetained:typeof prototype.findOrCreateNode==='function'}));
  });
  const path=info.outputPath('disposed-root-reachability.json');
  await writeFile(path,JSON.stringify(report,null,2));
  await info.attach('disposed-root-reachability.json',{path,contentType:'application/json'});
  expect(report).toEqual(['parsed','rendered','helper'].map(stage=>({stage,rootCollected:true,leafId:'A',leafIndex:0,helperRetained:true})));
 }finally{
  await session.detach();
  await page.evaluate(()=>{globalThis.__disposalReachability?.engine.close();delete globalThis.__disposalReachability;});
 }
});
