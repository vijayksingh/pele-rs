import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

async function save(info,name,report){const path=info.outputPath(name);await writeFile(path,JSON.stringify(report,null,2));await info.attach(name,{path,contentType:'application/json'});}
async function prepare(page){
  await page.goto('/examples/web/');
  await page.evaluate(()=>{
    const view=new DataView(new ArrayBuffer(8));
    const bits=value=>{view.setFloat64(0,value);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const number=hex=>{view.setBigUint64(0,BigInt('0x'+hex));return view.getFloat64(0);};
    const snapshot=roots=>{
      const ids=new Map(),objects=[],nodes=[];
      const atom=value=>{
        if(value===undefined)return['undefined'];if(value===null)return['null'];
        if(typeof value==='number')return['number',bits(value)];
        if(typeof value==='function')return['function',value.length];
        if(typeof value==='symbol')return['symbol',Symbol.keyFor(value),value.description];
        if(typeof value!=='object')return[typeof value,value];
        if(!ids.has(value)){ids.set(value,objects.length);objects.push(value);}return['ref',ids.get(value)];
      };
      const root=roots.map(atom);
      for(let i=0;i<objects.length;i++){
        const value=objects[i],kind=Array.isArray(value)?'array':value instanceof Map?'map':value instanceof Set?'set':value instanceof Date?'date':'object';
        const properties=Reflect.ownKeys(value).map(key=>{const d=Object.getOwnPropertyDescriptor(value,key);return[typeof key==='symbol'?['symbol',Symbol.keyFor(key),key.description]:key,d.enumerable,d.configurable,Object.hasOwn(d,'value')?['data',d.writable,atom(d.value)]:['accessor',atom(d.get),atom(d.set)]];});
        nodes.push({kind,properties,...(kind==='map'?{entries:[...value].map(([k,v])=>[atom(k),atom(v)])}:kind==='set'?{values:[...value].map(atom)}:kind==='date'?{milliseconds:bits(value.getTime())}:{})});
      }
      return{signature:{root,nodes},objects};
    };
    const output=value=>({keys:Object.keys(value),...value,width:bits(value.width),height:bits(value.height)});
    const stable=(a,b)=>a.length===b.length&&a.every((value,i)=>value===b[i]);
    const error=value=>({name:value?.name,message:value?.message,code:value?.code,type:value?.type,line:value?.line,column:value?.column,snippet:value?.snippet});
    window.__sourceLayoutFeature={bits,number,snapshot,output,stable,error,source:'sankey\nA,B,5\nB,C,3\nA,C,1'};
  });
}

test('source layouts publish complete accepted Sankey state before original result-factory throws',async({page},info)=>{
  test.setTimeout(120000);
  await page.route('**/dist/package-web/package/host-er-store.js',async route=>{
    const response=await route.fetch(),text=await response.text(),declaration='export function hostErRenderResult(';
    if(!text.includes(declaration))throw Error('Expected shared result factory');
    await route.fulfill({response,body:text.replace(declaration,'function originalSourceLayoutResult(')+'\nexport function hostErRenderResult(...args){if(globalThis.__sourceLayoutFactoryThrow)throw globalThis.__sourceLayoutFactoryValue;return originalSourceLayoutResult(...args);}\n'});
  });
  await prepare(page);
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),engine=await loadRenderer();
    const {bits,snapshot,output,stable,error,source}=window.__sourceLayoutFeature,rows=[];
    try{
      for(const raw of [false,true])for(const kind of ['object','symbol','undefined']){
        const row={raw,kind};let model;
        try{
          model=engine.parse(source);const nodes=model.nodes,links=model.links,lookup=model.lookup,first=nodes[0],extra={n:1};extra.self=extra;
          model.extra=extra;model.detached=[extra,first];const detached=model.detached;
          const draw=mutate=>{const trace=[];let fired=false;const options={measurer:{width(text,size,style){trace.push([text,bits(size),style]);if(mutate&&!fired){fired=true;mutate();}return text.length*size*.6;}}};const value=raw?engine.renderRaw('sankey',model,{},options):engine.render(model,options);return{value:output(value),trace};};
          draw();delete model.detached;
          const expected=draw(),before=snapshot([model,detached]);
          const sentinel=kind==='undefined'?undefined:kind==='symbol'?Symbol('source layout factory'):{factory:'source layout'};
          globalThis.__sourceLayoutFactoryValue=sentinel;globalThis.__sourceLayoutFactoryThrow=true;
          let fired=false,threw=false,original=false;
          try{draw(()=>{fired=true;extra.n=99;extra.added={};delete model.extra;first.id='callback edit';lookup.clear();detached[0]={unaccepted:true};});}
          catch(value){threw=true;original=Object.is(value,sentinel);}
          finally{globalThis.__sourceLayoutFactoryThrow=false;delete globalThis.__sourceLayoutFactoryValue;}
          const accepted=snapshot([model,detached]),recovery=draw(),after=snapshot([model,detached]);
          row.result={fired,threw,original,before:before.signature,accepted:accepted.signature,after:after.signature,expected,recovery,stable:[stable(before.objects,accepted.objects),stable(before.objects,after.objects)],aliases:model.nodes===nodes&&model.links===links&&model.lookup===lookup&&links[0].source===first&&lookup.get('A')===first&&model.extra===extra&&extra.self===extra&&detached[0]===extra&&detached[1]===first};
        }catch(value){row.fatal=error(value);}finally{globalThis.__sourceLayoutFactoryThrow=false;delete globalThis.__sourceLayoutFactoryValue;if(model)engine.dispose(model);}
        rows.push(row);
      }
      return rows;
    }finally{globalThis.__sourceLayoutFactoryThrow=false;delete globalThis.__sourceLayoutFactoryValue;engine.close();}
  });
  await save(info,'source-layout-factory-throws.json',report);
  expect(report).toHaveLength(6);
  for(const row of report){
    expect(row.fatal,`${row.raw}/${row.kind}`).toBeUndefined();
    expect(row.result).toMatchObject({fired:true,threw:true,original:true,stable:[true,true],aliases:true});
    expect(row.result.accepted).toEqual(row.result.before);expect(row.result.after).toEqual(row.result.before);
    expect(row.result.recovery).toEqual(row.result.expected);expect(row.result.expected.trace.length).toBeGreaterThan(0);
  }
});

test('source layouts consume canonical operation frames without losing sparse state, helpers or recovery',async({page},info)=>{
  test.setTimeout(120000);await prepare(page);
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),wasm=await import('/dist/package-web/package/pkg/pele.js');
    const {bits,number,snapshot,output,stable,error,source}=window.__sourceLayoutFeature;
    const prototype=wasm.MutableDocument.prototype,parent=Object.getPrototypeOf(prototype),prototypeKeys=Reflect.ownKeys(prototype);
    const names=['snapshot_source_layout','invoke_source_layout_with_functions','render_numbers_operation_source_layout_with_functions','render_raw_operation_source_layout_with_functions'];
    const descriptors=new Map(names.map(name=>[name,Object.getOwnPropertyDescriptor(prototype,name)]));
    const restore=()=>{for(const[name,descriptor]of descriptors)Object.defineProperty(prototype,name,descriptor);};
    const report={rows:[],restored:false};
    if([...descriptors.values()].some(value=>typeof value?.value!=='function'))return{...report,fatal:{message:'Expected all source layout capabilities'}};
    try{
      for(const raw of [false,true]){
        const row={raw};
        for(const mode of ['recipe','canonical']){
          restore();const engine=await loadRenderer();let model;const packets={snapshot:[],methods:0,renders:0,canonical:0};
          try{
            for(const[name,descriptor]of descriptors)Object.defineProperty(prototype,name,{...descriptor,value:function(...args){
              const packet=Reflect.apply(descriptor.value,this,args);
              if(!Array.isArray(packet)||(packet.length!==4&&packet.length!==6)||packet[0]!==1)throw Error('Expected source layout outcome');
              const magic=String.fromCharCode(...packet[2].subarray(0,8));if(magic!=='PLCSRC01')throw Error('Small source fixture must retain real source recipe');
              if(name==='snapshot_source_layout'){packets.snapshot.push({magic,type:JSON.parse(packet[3]).type});return packet;}
              if(name==='invoke_source_layout_with_functions')packets.methods++;else packets.renders++;
              if(mode==='canonical'){
                // This forces only the physical fallback decoder. It does not
                // claim to execute the native 64 MiB recipe-overflow branch.
                const full=this.snapshot_binary(),view=new DataView(full.buffer,full.byteOffset,full.byteLength),metadataLength=view.getUint32(20,true);
                const graph=full.slice(0,full.length-metadataLength);new DataView(graph.buffer,graph.byteOffset,graph.byteLength).setUint32(20,0,true);
                if(graph.byteLength!==packet[1])throw Error('Canonical frame count mismatch');
                packet[2]=graph;packets.canonical++;
              }
              return packet;
            }});
            model=engine.parse(source);const nodes=model.nodes,links=model.links,lookup=model.lookup,first=nodes[0];
            const extra={text:'漢😀\ud800',nan:number('fff8000000000042'),zero:-0,infinity:Infinity};extra.self=extra;
            const initialNumeric={nan:bits(extra.nan),zero:bits(extra.zero),infinity:bits(extra.infinity)},numericHistory=[];
            const trackNumeric=stage=>numericHistory.push({stage,nan:bits(extra.nan),zero:bits(extra.zero),infinity:bits(extra.infinity)});
            extra.map=new Map([[NaN,extra],[-0,first]]);extra.set=new Set([extra,first]);
            Object.defineProperty(extra,'hidden',{value:'non-enumerable 漢😀',writable:false,enumerable:false,configurable:true});
            model.extra=extra;model.detached=[extra,first];const detached=model.detached;
            const draw=mutate=>{const trace=[];let fired=false;const options={measurer:{width(text,size,style){trace.push([text,bits(size),style]);if(mutate&&!fired){fired=true;mutate();}return text.length*size*.6;}}};try{const value=raw?engine.renderRaw('sankey',model,{},options):engine.render(model,options);return{ok:true,value:output(value),trace};}catch(value){return{ok:false,error:error(value),trace};}};
            const seed=draw();trackNumeric('seed');delete model.detached;const detachedSeed=draw();trackNumeric('detached seed');
            const hit=model.findOrCreateNode('A');trackNumeric('existing helper');
            const added=model.findOrCreateNode('漢😀');trackNumeric('append helper');model.addLink(nodes[2],added,2);trackNumeric('link helper');
            model.title='Title 漢😀 &amp;';links[0].value=7;
            const edited=draw(),before=snapshot([model,detached]);trackNumeric('edited');let fired=false;
            const callback=draw(()=>{fired=true;extra.text='callback';extra.zero=123;delete model.extra;detached[0]={unaccepted:true};});
            const accepted=snapshot([model,detached]);trackNumeric('callback restoration');
            first.id=42;const invalid=draw(),invalidState=snapshot([model,detached]).signature;trackNumeric('invalid edit');
            first.id='A';const recovery=draw();trackNumeric('repair');const next=draw(),after=snapshot([model,detached]);trackNumeric('next');
            row[mode]={seed,detachedSeed,edited,before:before.signature,callback,accepted:accepted.signature,invalid,invalidState,recovery,next,after:after.signature,fired,restored:stable(before.objects,accepted.objects),identity:model.nodes===nodes&&model.links===links&&model.lookup===lookup&&hit===first&&lookup.get('A')===first&&added===nodes.at(-1)&&lookup.get('漢😀')===added&&links.at(-1).target===added&&model.extra===extra&&extra.self===extra&&detached[0]===extra&&detached[1]===first,initialNumeric,numericHistory,numeric:{nan:bits(extra.nan),zero:bits(extra.zero),infinity:bits(extra.infinity)}};
          }catch(value){row[mode]={fatal:error(value)};}
          finally{if(model)engine.dispose(model);engine.close();restore();row[mode+'Packets']=packets;}
        }
        report.rows.push(row);
      }
    }finally{
      restore();report.restored=wasm.MutableDocument.prototype===prototype&&Object.getPrototypeOf(prototype)===parent&&Reflect.ownKeys(prototype).length===prototypeKeys.length&&Reflect.ownKeys(prototype).every((key,i)=>key===prototypeKeys[i])&&[...descriptors].every(([name,original])=>{const actual=Object.getOwnPropertyDescriptor(prototype,name);return Reflect.ownKeys(original).length===Reflect.ownKeys(actual).length&&Reflect.ownKeys(original).every(key=>original[key]===actual[key]);});
    }
    return report;
  });
  await save(info,'source-layout-canonical-fallback.json',report);
  expect(report.fatal).toBeUndefined();expect(report.restored).toBe(true);expect(report.rows).toHaveLength(2);
  for(const row of report.rows){
    expect(row.recipe.fatal,`${row.raw}/recipe`).toBeUndefined();expect(row.canonical.fatal,`${row.raw}/canonical`).toBeUndefined();
    // Engines can box the freshly created fixture NaN differently between
    // iterations. Prove exact original bits in EACH lane before comparing
    // that one input field symbolically across independently created models.
    const comparable=value=>{
      expect(value.numeric).toEqual(value.initialNumeric);expect(value.numericHistory).toHaveLength(10);
      for(const {stage,...numeric}of value.numericHistory)expect(numeric,stage).toEqual(value.initialNumeric);
      const nan=BigInt('0x'+value.initialNumeric.nan);expect((nan&0x7ff0000000000000n)===0x7ff0000000000000n&&(nan&0xfffffffffffffn)!==0n).toBe(true);
      const copy=structuredClone(value);
      for(const stage of ['before','accepted','invalidState','after']){
        const properties=copy[stage].nodes.flatMap(node=>node.properties).filter(property=>property[0]==='nan');
        expect(properties,stage).toHaveLength(1);expect(properties[0][3][2],stage).toEqual(['number',value.initialNumeric.nan]);
        properties[0][3][2][1]='original fixture NaN';
      }
      copy.initialNumeric.nan=copy.numeric.nan='original fixture NaN';for(const numeric of copy.numericHistory)numeric.nan='original fixture NaN';return copy;
    };
    expect(comparable(row.canonical),row.raw+'').toEqual(comparable(row.recipe));
    expect(row.canonical).toMatchObject({fired:true,restored:true,identity:true,numeric:{zero:'8000000000000000',infinity:'7ff0000000000000'}});
    expect(row.canonical.numeric).toEqual(row.canonical.initialNumeric);
    const nan=BigInt('0x'+row.canonical.numeric.nan);expect((nan&0x7ff0000000000000n)===0x7ff0000000000000n&&(nan&0xfffffffffffffn)!==0n).toBe(true);
    expect(row.canonical.accepted).toEqual(row.canonical.before);expect(row.canonical.callback).toEqual(row.canonical.edited);
    expect(row.canonical.invalid.ok).toBe(false);expect(row.canonical.recovery.ok).toBe(true);expect(row.canonical.next).toEqual(row.canonical.recovery);
    for(const mode of ['recipe','canonical']){expect(row[mode+'Packets'].snapshot).toEqual([{magic:'PLCSRC01',type:'sankey'}]);expect(row[mode+'Packets'].methods).toBe(3);expect(row[mode+'Packets'].renders).toBeGreaterThanOrEqual(6);}
    expect(row.canonicalPackets.methods).toBe(row.recipePackets.methods);expect(row.canonicalPackets.renders).toBe(row.recipePackets.renders);
    expect(row.canonicalPackets.canonical).toBe(row.canonicalPackets.methods+row.canonicalPackets.renders);expect(row.recipePackets.canonical).toBe(0);
  }
});
