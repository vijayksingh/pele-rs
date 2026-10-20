import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('Sankey accepted baselines preserve live edits, failed observations and nested helper history',async({page},info)=>{
  test.setTimeout(120000);
  await page.route('**/dist/package-web/package/host-er-store.js',async route=>{
    const response=await route.fetch(),text=await response.text(),declaration='export function hostErRenderResult(';
    if(!text.includes(declaration))throw Error('Expected actual result factory');
    await route.fulfill({response,body:text.replace(declaration,'function baselineOriginalResult(')+'\nexport function hostErRenderResult(...args){const hook=globalThis.__baselineFactoryHook;if(hook){globalThis.__baselineFactoryHook=undefined;hook();}return baselineOriginalResult(...args);}\n'});
  });
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),native=await import('/dist/package-web/package/pkg/pele.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),rawOracle=(await import('/dist/evidence/pele-esm/diagrams/sankey.js')).default;
    const prototype=native.MutableDocument.prototype,names=['apply_source_input','apply_binary','prepare_detached_render_numbers','prepare_detached_render_raw','finish_detached_render'];
    const saved=new Map(names.map(name=>[name,Object.getOwnPropertyDescriptor(prototype,name)])),counts=Object.fromEntries(names.map(name=>[name,0])),rows=[];
    for(const[name,descriptor]of saved){if(typeof descriptor?.value!=='function')throw Error('Missing actual method '+name);Object.defineProperty(prototype,name,{...descriptor,value:function(...args){counts[name]++;return Reflect.apply(descriptor.value,this,args);}});}
    const bitView=new DataView(new ArrayBuffer(8)),bits=value=>{bitView.setFloat64(0,value);return bitView.getBigUint64(0).toString(16);};
    const graph=roots=>{
      const objects=[],ids=new Map(),nodes=[];
      const atom=value=>{if(value===null)return['null'];if(typeof value==='number')return['number',bits(value)];if(typeof value==='function')return['function',value.length];if(typeof value!=='object')return[typeof value,value];if(!ids.has(value)){ids.set(value,objects.length);objects.push(value);}return['ref',ids.get(value)];};
      const root=roots.map(atom);
      for(const value of objects){const properties=Reflect.ownKeys(value).map(key=>{const d=Object.getOwnPropertyDescriptor(value,key);return[key,d.enumerable,d.configurable,Object.hasOwn(d,'value')?['data',d.writable,atom(d.value)]:['accessor',atom(d.get),atom(d.set)]];});nodes.push({kind:Array.isArray(value)?'array':value instanceof Map?'map':'object',properties,...(value instanceof Map?{entries:[...value].map(([k,v])=>[atom(k),atom(v)])}:{})});}
      return{root,nodes};
    };
    const error=value=>({name:value?.name,message:value?.message,code:value?.code});
    const output=value=>({...value,width:bits(value.width),height:bits(value.height)});
    const applyCount=()=>counts.apply_source_input+counts.apply_binary,source='sankey\nA,B,5\nB,C,3\nA,C,1';
    let restored=false;
    try{
      for(const raw of [false,true]){
        const engine=await loadRenderer(),model=engine.parse(source),reference=oracle.parse(source),nodes=model.nodes,refNodes=reference.nodes,row={raw,stages:[]};
        const draw=(value,isOracle=false,callback)=>{const options={measurer:{width(text,size){callback?.();return text.length*size*.6;}}};return output(isOracle?(raw?rawOracle.render(value,{},options):oracle.render(value,options)):(raw?engine.renderRaw('sankey',value,{},options):engine.render(value,options)));};
        const pair=stage=>{const before=applyCount(),value=draw(model),expected=draw(reference,true);row.stages.push({stage,value,expected,graph:graph([model,nodes]),oracleGraph:graph([reference,refNodes]),applies:applyCount()-before});};
        try{
          const before={...counts};pair('first');pair('unchanged');pair('unchanged again');row.unchangedCalls=Object.fromEntries(names.map(name=>[name,counts[name]-before[name]]));
          const sentinel={failure:'live callback'};let mutated=false;try{draw(model,false,()=>{if(!mutated){mutated=true;model.nodes[1].id='Changed B';throw sentinel;}});}catch(value){row.callbackOriginal=value===sentinel;}
          reference.nodes[1].id='Changed B';pair('live edit accepted');pair('live edit unchanged');
          // Failed capture allocates history before a later ownKeys trap throws.
          // Removing the root edge does not remove the retained observed object.
          let blocked=true;const pending=new Proxy({label:'unaccepted history'}, {ownKeys(value){if(blocked)throw sentinel;return Reflect.ownKeys(value);}});
          model.pending=pending;const failedBefore=applyCount();try{draw(model);}catch(value){row.observationOriginal=value===sentinel;}row.failedApplyCalls=applyCount()-failedBefore;
          blocked=false;delete model.pending;pair('failed observation repaired');pair('history unchanged');
          // Registered prototype mutation still fails its existing host schema
          // gate even with a previously accepted baseline; restoration is user work.
          const proto=Object.getPrototypeOf(model),descriptor=Object.getOwnPropertyDescriptor(proto,'findOrCreateNode');
          Object.defineProperty(proto,'findOrCreateNode',{...descriptor,enumerable:!descriptor.enumerable});
          try{draw(model);}catch(value){row.prototypeError=error(value);}finally{Object.defineProperty(proto,'findOrCreateNode',descriptor);}
          pair('prototype repaired');
          // Native partial helper failure publishes the prior Map insertion.
          Object.defineProperty(nodes,'length',{writable:false});Object.defineProperty(refNodes,'length',{writable:false});
          try{model.findOrCreateNode('X');}catch(value){row.partialError=error(value);}try{reference.findOrCreateNode('X');}catch(value){row.oraclePartialError=error(value);}
          row.partialGraph=graph([model,nodes]);row.oraclePartialGraph=graph([reference,refNodes]);
          const orphan=model.findOrCreateNode('X'),refOrphan=reference.findOrCreateNode('X');row.orphanAlias=orphan===model.lookup.get('X')&&!nodes.includes(orphan);row.oracleOrphanAlias=refOrphan===reference.lookup.get('X')&&!refNodes.includes(refOrphan);
          model.nodes=[...nodes,orphan];reference.nodes=[...refNodes,refOrphan];pair('partial helper repaired');pair('partial helper unchanged');
          // Real factory reentry publishes a newer native revision. Outer ACK
          // must not overwrite that receipt, even when the factory then throws.
          globalThis.__baselineFactoryHook=()=>{const added=model.findOrCreateNode('Y');row.factoryAlias=added===model.nodes.at(-1)&&added===model.lookup.get('Y');reference.findOrCreateNode('Y');throw sentinel;};
          try{draw(model);}catch(value){row.factoryOriginal=value===sentinel;}
          pair('nested factory recovery');pair('nested factory unchanged');
          row.finalGraph=graph([model,nodes,pending]);row.oracleFinalGraph=graph([reference,refNodes,{label:'unaccepted history'}]);
        }catch(value){row.fatal=error(value);}finally{globalThis.__baselineFactoryHook=undefined;engine.dispose(model);engine.close();}
        rows.push(row);
      }
    }finally{for(const[name,descriptor]of saved)Object.defineProperty(prototype,name,descriptor);restored=[...saved].every(([name,descriptor])=>Reflect.ownKeys(descriptor).every(key=>Object.getOwnPropertyDescriptor(prototype,name)[key]===descriptor[key]));}
    return{rows,counts,restored};
  });
  const path=info.outputPath('sankey-accepted-baseline.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('sankey-accepted-baseline.json',{path,contentType:'application/json'});
  expect(report.restored).toBe(true);expect(report.rows).toHaveLength(2);
  for(const row of report.rows){
    expect(row.fatal).toBeUndefined();expect(row).toMatchObject({callbackOriginal:true,observationOriginal:true,failedApplyCalls:0,orphanAlias:true,oracleOrphanAlias:true,factoryAlias:true,factoryOriginal:true});
    expect(row.unchangedCalls).toMatchObject({apply_source_input:0,apply_binary:0,finish_detached_render:3,[row.raw?'prepare_detached_render_raw':'prepare_detached_render_numbers']:3});
    expect(row.prototypeError?.message).toBe('Prototype edits require an explicit native prototype mutation codec');expect(row.partialError).toEqual(row.oraclePartialError);expect(row.partialError).toBeDefined();
    expect(row.partialGraph).toEqual(row.oraclePartialGraph);expect(row.finalGraph).toEqual(row.oracleFinalGraph);
    for(const stage of row.stages){expect(stage.value,stage.stage).toEqual(stage.expected);expect(stage.graph,stage.stage).toEqual(stage.oracleGraph);}
    const stages=new Map(row.stages.map(stage=>[stage.stage,stage]));
    for(const name of ['first','unchanged','unchanged again','live edit unchanged','history unchanged','partial helper unchanged','nested factory recovery','nested factory unchanged'])expect(stages.get(name).applies,name).toBe(0);
    for(const name of ['live edit accepted','failed observation repaired','partial helper repaired'])expect(stages.get(name).applies,name).toBe(1);
  }
});
