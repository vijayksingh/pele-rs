import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('fresh source publication preserves first helpers, bits, aliases and callback recovery',async({page},info)=>{
  test.setTimeout(120000);
  const reports={};
  for(const mode of ['generated','generic']){
    // Untimed feature instrumentation: run the real generated function, recording its
    // actual decision. The control bypasses it but records the same public parse sites.
    await page.route('**/dist/package-web/package/source-initializer.js',async route=>{
      const response=await route.fetch(),original=await response.text();
      const signature='export function tryInitializeSourceLayout(';
      if(original.split(signature).length!==2)throw new Error('Initializer export missing or ambiguous');
      const actual=original.replace(signature,'function actualInitializeSourceLayout(');
      const decision=mode==='generated'?'actualInitializeSourceLayout(bridge,graph)':'false';
      await route.fulfill({response,body:actual+`\nexport function tryInitializeSourceLayout(bridge,graph){const used=${decision};(globalThis.__initializerDecisions??=[]).push(used);return used;}\n`});
    });
    await page.goto('/examples/web/');
    reports[mode]=await page.evaluate(async()=>{
      const {loadRenderer}=await import('/dist/package-web/package/full.js');
      const oracle=await import('/dist/evidence/pele-oracle.js');
      const engine=await loadRenderer(),buffer=new DataView(new ArrayBuffer(8));
      const bits=value=>{buffer.setFloat64(0,value);return buffer.getBigUint64(0).toString(16).padStart(16,'0');};
      const output=value=>({...value,width:bits(value.width),height:bits(value.height),keys:Object.keys(value)});
      const failure=value=>({name:value?.name,message:value?.message,code:value?.code,type:value?.type,line:value?.line,column:value?.column});
      const snapshot=root=>{
        const ids=new Map(),objects=[],nodes=[];
        const atom=value=>{
          if(value===undefined)return['undefined'];if(value===null)return['null'];
          if(typeof value==='number')return['number',bits(value)];
          if(typeof value!=='object')return[typeof value,value];
          if(!ids.has(value)){ids.set(value,objects.length);objects.push(value);}return['ref',ids.get(value)];
        };
        const initial=atom(root);
        for(let id=0;id<objects.length;id++){
          const value=objects[id],properties=Reflect.ownKeys(value).map(key=>{const d=Object.getOwnPropertyDescriptor(value,key);return[key,d.enumerable,d.writable,d.configurable,atom(d.value)];});
          nodes.push({kind:Array.isArray(value)?'array':value instanceof Map?'map':value instanceof Set?'set':'object',properties,...(value instanceof Map?{entries:[...value].map(([k,v])=>[atom(k),atom(v)])}:{}),...(value instanceof Set?{values:[...value].map(atom)}:{})});
        }
        return{data:{root:initial,nodes},objects};
      };
      const width=(text,size)=>text.length*size*.6,options={measurer:{width}};
      const cases=[
        ['finite','sankey\nA,B,5\nB,C,3\nA,C,1'],
        ['first-render','sankey\nA,B,5\nB,C,3\nA,C,1'],
        ['key-collisions','sankey\nid,index,0.25\nindex,source,2\nsource,target,3'],
        ['prototype-label','sankey\n__proto__,type,1'],
        ['title','---\ntitle: "漢😀"\n---\nsankey\nA,B,1'],
        ['negative-zero','sankey\nA,B,-0'],
        ['nan','sankey\nA,B,NaN'],
        ['infinity','sankey\nA,B,Infinity'],
      ],rows=[];
      try{
        for(const[name,source]of cases){
          let model,sibling;const row={name};
          try{
            let reference;
            try{reference=oracle.parse(source);row.oracleParsed=true;}catch(error){row.oracleParsed=false;row.oracleParseError=failure(error);}
            const parse=()=>{const start=(globalThis.__initializerDecisions??=[]).length;try{return engine.parse(source);}finally{(row.activation??=[]).push(globalThis.__initializerDecisions.slice(start));}};
            try{model=parse();row.parsed=true;}catch(error){row.parsed=false;row.parseError=failure(error);}
            if(!row.parsed||!row.oracleParsed){rows.push(row);continue;}
            sibling=parse();
            row.numericKind=model.links.map(({value})=>Number.isNaN(value)?'nan':Object.is(value,-0)?'-0':value===Infinity?'infinity':Number.isFinite(value)?'finite':'other');
            row.oracleNumericKind=reference.links.map(({value})=>Number.isNaN(value)?'nan':Object.is(value,-0)?'-0':value===Infinity?'infinity':Number.isFinite(value)?'finite':'other');
            const initial=snapshot(model);row.initial=initial.data;row.numeric=model.links.map(link=>bits(link.value));
            const prototype=Object.getPrototypeOf(model);row.sharedPrototype=prototype===Object.getPrototypeOf(sibling);
            // Exercise both first-render and first-helper entry without native export.
            if(name==='first-render')row.unchangedFirstRender=output(engine.render(model,options));
            const first=model.nodes[0],found=model.findOrCreateNode(first.id);
            row.firstAlias=found===first&&found===model.lookup.get(first.id);
            row.firstRender=output(engine.render(model,options));row.oracle=output(oracle.render(reference,options));
            const added=model.findOrCreateNode('fresh extension 漢'),referenceAdded=reference.findOrCreateNode('fresh extension 漢');
            row.addedAlias=added===model.nodes.at(-1)&&added===model.lookup.get('fresh extension 漢');
            const extension={value:-0};extension.self=extension;Object.defineProperty(extension,'hidden',{value:'\ud800',writable:false,enumerable:false,configurable:true});
            model.extension={extension,map:new Map([[first,extension],[added,added]])};
            const referenceExtension={value:-0};referenceExtension.self=referenceExtension;
            Object.defineProperty(referenceExtension,'hidden',{value:'\ud800',writable:false,enumerable:false,configurable:true});
            reference.extension={extension:referenceExtension,map:new Map([[reference.nodes[0],referenceExtension],[referenceAdded,referenceAdded]])};
            row.accepted=output(engine.render(model,options));row.acceptedOracle=output(oracle.render(reference,options));
            const baseline=snapshot(model),sentinel={first:'callback throw'};let changed=false;
            try{engine.render(model,{measurer:{width(){if(!changed){changed=true;first.id='callback edit';extension.value=99;extension.extra=true;model.links[0].value=17;}throw sentinel;}}});row.callbackThrew=false;}
            catch(value){row.callbackThrew=true;row.originalThrow=value===sentinel;}
            const restored=snapshot(model);row.restored=restored.data;row.beforeThrow=baseline.data;
            row.identities=baseline.objects.length===restored.objects.length&&baseline.objects.every((value,i)=>value===restored.objects[i]);
            row.prototypeStable=Object.getPrototypeOf(model)===prototype;
            let referenceChanged=false;
            try{oracle.render(reference,{measurer:{width(){if(!referenceChanged){referenceChanged=true;reference.nodes[0].id='callback edit';referenceExtension.value=99;referenceExtension.extra=true;reference.links[0].value=17;}throw sentinel;}}});row.oracleCallbackThrew=false;}
            catch(value){row.oracleCallbackThrew=true;row.oracleOriginalThrow=value===sentinel;}
            row.callbackReached=changed;row.oracleCallbackReached=referenceChanged;
            row.oracleAfterThrow=snapshot(reference).data;
            row.recovery=output(engine.render(model,options));row.oracleRecovery=output(oracle.render(reference,options));
          }catch(error){row.fatal=failure(error);}
          finally{if(model)engine.dispose(model);if(sibling)engine.dispose(sibling);}
          rows.push(row);
        }
        let invalid,oracleInvalid;try{engine.parse('sankey\nA,B');}catch(error){invalid=failure(error);}
        try{oracle.parse('sankey\nA,B');}catch(error){oracleInvalid=failure(error);}
        return{rows,invalid,oracleInvalid};
      }finally{engine.close();}
    });
    await page.unroute('**/dist/package-web/package/source-initializer.js');
  }
  const path=info.outputPath('source-initializer-public.json');await writeFile(path,JSON.stringify(reports,null,2));await info.attach('source-initializer-public',{path,contentType:'application/json'});
  const semantic=report=>({...report,rows:report.rows.map(({activation,...row})=>row)});
  expect(semantic(reports.generated)).toEqual(semantic(reports.generic));
  expect(reports.generated.invalid).toBeDefined();
  expect(reports.generated.invalid).toEqual(reports.generated.oracleInvalid);
  for(const row of reports.generated.rows){
    expect(row.fatal,row.name).toBeUndefined();
    expect(row.parsed,row.name).toBe(row.oracleParsed);
    if(!['nan','negative-zero','infinity'].includes(row.name))expect(row.parsed,row.name).toBe(true);
    if(!row.parsed){expect(row.parseError,row.name).toEqual(row.oracleParseError);continue;}
    expect(row.numericKind,row.name).toEqual(row.oracleNumericKind);
    const fallback={nan:'nan','negative-zero':'-0',infinity:'infinity'}[row.name];
    if(fallback)expect(row.numericKind,row.name).toContain(fallback);
    expect(row.activation,row.name).toEqual([[!fallback],[!fallback]]);
    expect(reports.generic.rows.find(other=>other.name===row.name).activation,row.name).toEqual([[false],[false]]);
    expect(row,row.name).toMatchObject({sharedPrototype:true,firstAlias:true,addedAlias:true,callbackThrew:true,originalThrow:true,oracleCallbackThrew:true,oracleOriginalThrow:true,callbackReached:true,oracleCallbackReached:true,identities:true,prototypeStable:true});
    expect(row.firstRender,row.name).toEqual(row.oracle);expect(row.accepted,row.name).toEqual(row.acceptedOracle);
    // Pele retains callback edits even when the original thrown value escapes.
    expect(row.restored,row.name).toEqual(row.oracleAfterThrow);expect(row.restored,row.name).not.toEqual(row.beforeThrow);
    expect(row.recovery,row.name).toEqual(row.oracleRecovery);
  }
});

test('inherited fixed-field setter keeps generic fresh-publication behavior',async({page},info)=>{
  const reports={};
  for(const mode of ['generated','generic']){
    await page.route('**/dist/package-web/package/source-initializer.js',async route=>{
      const response=await route.fetch(),original=await response.text();
      const signature='export function tryInitializeSourceLayout(';
      if(original.split(signature).length!==2)throw new Error('Initializer export missing or ambiguous');
      const actual=original.replace(signature,'function actualInitializeSourceLayout(');
      const decision=mode==='generated'?'actualInitializeSourceLayout(bridge,graph)':'false';
      await route.fulfill({response,body:actual+`\nexport function tryInitializeSourceLayout(bridge,graph){const used=${decision};(globalThis.__initializerDecisions??=[]).push(used);return used;}\n`});
    });
    await page.goto('/examples/web/');
    reports[mode]=await page.evaluate(async()=>{
      const {loadRenderer}=await import('/dist/package-web/package/full.js');
      const engine=await loadRenderer(),previous=Object.getOwnPropertyDescriptor(Object.prototype,'id');
      const calls=[],receivers=[];let model;
      try{
        const start=(globalThis.__initializerDecisions??=[]).length;
        try{
          Object.defineProperty(Object.prototype,'id',{configurable:true,set(value){
            calls.push(value);receivers.push(this);
            Object.defineProperty(this,'id',{value,enumerable:true,writable:true,configurable:true});
          }});
          model=engine.parse('sankey\nA,B,5\nB,C,3');
        }finally{
          if(previous)Object.defineProperty(Object.prototype,'id',previous);
          else delete Object.prototype.id;
        }
        const restored=Object.getOwnPropertyDescriptor(Object.prototype,'id');
        const buffer=new DataView(new ArrayBuffer(8));
        const bits=value=>{buffer.setFloat64(0,value);return buffer.getBigUint64(0).toString(16).padStart(16,'0');};
        const rendered=engine.render(model,{measurer:{width:(text,size)=>text.length*size*.6}});
        return{
          activation:globalThis.__initializerDecisions.slice(start),calls,
          nodeCalls:calls.filter((_,i)=>model.nodes.includes(receivers[i])),
          setterReceivers:receivers.filter(value=>model.nodes.includes(value)).length===model.nodes.length&&receivers.filter(value=>model.nodes.includes(value)).every((value,i)=>value===model.nodes[i]),
          descriptors:model.nodes.map(node=>Object.getOwnPropertyDescriptor(node,'id')),
          nodeKeys:model.nodes.map(node=>Reflect.ownKeys(node)),
          aliases:model.nodes.every(node=>model.lookup.get(node.id)===node&&model.findOrCreateNode(node.id)===node),
          restored:previous?restored?.get===previous.get&&restored?.set===previous.set&&restored?.value===previous.value&&restored?.writable===previous.writable&&restored?.enumerable===previous.enumerable&&restored?.configurable===previous.configurable:restored===undefined,
          rendered:{...rendered,width:bits(rendered.width),height:bits(rendered.height),keys:Object.keys(rendered)},
        };
      }finally{if(model)engine.dispose(model);engine.close();}
    });
    await page.unroute('**/dist/package-web/package/source-initializer.js');
  }
  const path=info.outputPath('source-initializer-inherited-field.json');await writeFile(path,JSON.stringify(reports,null,2));await info.attach('source-initializer-inherited-field',{path,contentType:'application/json'});
  expect(reports.generated).toEqual(reports.generic);
  expect(reports.generated).toMatchObject({activation:[false],nodeCalls:['A','B','C'],setterReceivers:true,aliases:true,restored:true});
  expect(reports.generated.nodeKeys).toEqual([['id','index'],['id','index'],['id','index']]);
  expect(reports.generated.descriptors).toEqual(['A','B','C'].map(value=>({value,writable:true,enumerable:true,configurable:true})));
});
