import {test,expect} from '@playwright/test';

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
