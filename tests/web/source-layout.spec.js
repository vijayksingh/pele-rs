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
    // Untimed real API instrumentation. Returned jobs/ACKs keep their identities;
    // wrappers call the actual methods and restore every descriptor afterward.
    const installJobs=prototype=>{
      const saved=[],wrapped=new WeakMap(),jobs=new WeakMap(),acks=new WeakMap();
      const counts={prepareNumbers:0,prepareRaw:0,prepareErrors:0,created:0,runs:0,finishes:0,materialized:0,records:[]};
      const wrap=(target,name,make)=>{
        let keys=wrapped.get(target);if(!keys){keys=new Set();wrapped.set(target,keys);}if(keys.has(name))return;
        const descriptor=Object.getOwnPropertyDescriptor(target,name);if(typeof descriptor?.value!=='function')throw Error('Missing detached render method '+name);
        saved.push([target,name,descriptor]);keys.add(name);Object.defineProperty(target,name,{...descriptor,value:make(descriptor.value)});
      };
      for(const name of ['prepare_detached_render_numbers','prepare_detached_render_raw'])wrap(prototype,name,original=>function(...args){
        counts[name.endsWith('_raw')?'prepareRaw':'prepareNumbers']++;
        let job;try{job=Reflect.apply(original,this,args);}catch(value){counts.prepareErrors++;throw value;}
        if(!job)return job;
        const record={owner:this,runs:0,finishes:0,materialized:0};jobs.set(job,record);counts.created++;counts.records.push(record);
        wrap(Object.getPrototypeOf(job),'run',run=>function(...args){const row=jobs.get(this);if(!row)throw Error('Untracked detached job');row.runs++;counts.runs++;return Reflect.apply(run,this,args);});
        return job;
      });
      wrap(prototype,'finish_detached_render',original=>function(job,...args){
        const row=jobs.get(job);if(!row||row.owner!==this)throw Error('Detached job finished by wrong owner');
        const ack=Reflect.apply(original,this,[job,...args]);row.finishes++;counts.finishes++;row.revision=String(ack.revision);acks.set(ack,row);
        wrap(Object.getPrototypeOf(ack),'materialize',materialize=>function(...args){
          const record=acks.get(this);if(!record)throw Error('Untracked render ACK');record.materialized++;counts.materialized++;
          const packet=Reflect.apply(materialize,this,args);
          if(!Array.isArray(packet)||packet.length!==5||packet[0]!==1||packet[1]!==record.revision||!Number.isSafeInteger(packet[2])||packet[2]<32||packet[2]>64*1024*1024||![0,1].includes(packet[3]))throw Error('Invalid readonly render ACK');
          if(packet[3]===0){if(typeof packet[4]!=='string')throw Error('Expected ACK JSON');JSON.parse(packet[4]);if(packet[2]+new TextEncoder().encode(packet[4]).length>64*1024*1024)throw Error('Combined ACK exceeds canonical frame budget');}
          else if(!Array.isArray(packet[4])||packet[4].length!==3||packet[4][0]!==1||!Number.isSafeInteger(packet[4][1])||packet[4][1]<0||packet[2]+packet[4][1]>64*1024*1024||typeof packet[4][2]?.svg!=='string')throw Error('Expected direct rendered result packet');
          record.graphBytes=packet[2];record.status=packet[3];return packet;
        });return ack;
      });
      return{counts,report:()=>({...counts,records:counts.records.map(({owner,...row})=>row)}),restore(){
        for(const[target,name,descriptor]of saved.reverse())Object.defineProperty(target,name,descriptor);
        for(const[target,name,descriptor]of saved){const actual=Object.getOwnPropertyDescriptor(target,name);if(Reflect.ownKeys(actual).length!==Reflect.ownKeys(descriptor).length||Reflect.ownKeys(descriptor).some(key=>actual[key]!==descriptor[key]))throw Error('Detached descriptor restore failed');}
      }};
    };
    window.__sourceLayoutFeature={bits,number,snapshot,output,stable,error,installJobs,source:'sankey\nA,B,5\nB,C,3\nA,C,1'};
  });
}

test('Sankey readonly completion retains callback state through original result-factory throws',async({page},info)=>{
  test.setTimeout(120000);
  await page.route('**/dist/package-web/package/host-er-store.js',async route=>{
    const response=await route.fetch(),text=await response.text(),declaration='export function hostErRenderResult(';
    if(!text.includes(declaration))throw Error('Expected shared result factory');
    await route.fulfill({response,body:text.replace(declaration,'function originalSourceLayoutResult(')+'\nexport function hostErRenderResult(...args){if(globalThis.__sourceLayoutFactoryThrow)throw globalThis.__sourceLayoutFactoryValue;return originalSourceLayoutResult(...args);}\n'});
  });
  await prepare(page);
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),engine=await loadRenderer(),oracle=await import('/dist/evidence/pele-oracle.js');
    const {bits,snapshot,output,stable,error,source}=window.__sourceLayoutFeature,rows=[];
    try{
      for(const raw of [false,true])for(const kind of ['object','symbol','undefined']){
        const row={raw,kind};let model;
        try{
          model=engine.parse(source);const nodes=model.nodes,links=model.links,lookup=model.lookup,first=nodes[0],extra={n:1};extra.self=extra;
          model.extra=extra;model.detached=[extra,first];const detached=model.detached;
          let lastTrace;const draw=mutate=>{const trace=[];lastTrace=trace;let fired=false;const options={measurer:{width(text,size,style){trace.push([text,bits(size),style]);if(mutate&&!fired){fired=true;mutate();}return text.length*size*.6;}}};const value=raw?engine.renderRaw('sankey',model,{},options):engine.render(model,options);return{value:output(value),trace};};
          draw();delete model.detached;
          const expected=draw(),before=snapshot([model,detached]);
          const sentinel=kind==='undefined'?undefined:kind==='symbol'?Symbol('source layout factory'):{factory:'source layout'};
          globalThis.__sourceLayoutFactoryValue=sentinel;globalThis.__sourceLayoutFactoryThrow=true;
          let fired=false,threw=false,original=false;
          try{draw(()=>{fired=true;extra.n=99;extra.added={};delete model.extra;first.id='callback edit';lookup.clear();detached[0]={unaccepted:true};});}
          catch(value){threw=true;original=Object.is(value,sentinel);}
          finally{globalThis.__sourceLayoutFactoryThrow=false;delete globalThis.__sourceLayoutFactoryValue;}
          const failedTrace=lastTrace,accepted=snapshot([model,detached,extra]),recovery=draw(),after=snapshot([model,detached,extra]);
          const reference=oracle.parse(source),referenceFirst=reference.nodes[0],referenceExtra={n:1};referenceExtra.self=referenceExtra;
          reference.extra=referenceExtra;reference.detached=[referenceExtra,referenceFirst];const referenceDetached=reference.detached;delete reference.detached;
          let oracleFired=false;const oracleDraw=mutate=>{const trace=[];const value=oracle.render(reference,{measurer:{width(text,size,style){trace.push([text,bits(size),style]);if(mutate&&!oracleFired){oracleFired=true;referenceExtra.n=99;referenceExtra.added={};delete reference.extra;referenceFirst.id='callback edit';reference.lookup.clear();referenceDetached[0]={unaccepted:true};}return text.length*size*.6;}}});return{value:output(value),trace};};
          const oracleFirst=oracleDraw(true);const oracleAccepted=snapshot([reference,referenceDetached,referenceExtra]).signature,oracleRecovery=oracleDraw(false);
          row.result={fired,threw,original,failedTrace,oracleFirstTrace:oracleFirst.trace,oracleFired,oracleAccepted,oracleRecovery,before:before.signature,accepted:accepted.signature,after:after.signature,expected,recovery,stable:[model.nodes===nodes,model.links===links,model.lookup===lookup],aliases:model.nodes===nodes&&model.links===links&&model.lookup===lookup&&links[0].source===first&&lookup.size===0&&!Object.hasOwn(model,'extra')&&extra.self===extra&&detached[0].unaccepted===true&&detached[1]===first};
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
    expect(row.result).toMatchObject({fired:true,threw:true,original:true,oracleFired:true,stable:[true,true,true],aliases:true});
    expect(row.result.accepted).toEqual(row.result.oracleAccepted);expect(row.result.after).toEqual(row.result.oracleAccepted);
    expect(row.result.failedTrace).toEqual(row.result.oracleFirstTrace);expect(row.result.recovery).toEqual(row.result.oracleRecovery);expect(row.result.expected.trace.length).toBeGreaterThan(0);
  }
});

test('typed source columns own packet bytes across legacy and canonical lifecycle recovery',async({page},info)=>{
  test.setTimeout(120000);await prepare(page);
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),wasm=await import('/dist/package-web/package/pkg/pele.js');
    const {bits,number,snapshot,output,stable,error,installJobs,source}=window.__sourceLayoutFeature;
    const oracle=await import('/dist/evidence/pele-oracle.js');
    const prototype=wasm.MutableDocument.prototype,parent=Object.getPrototypeOf(prototype),prototypeKeys=Reflect.ownKeys(prototype);
    const names=['snapshot_source_layout','invoke_source_layout_with_functions','render_numbers_operation_source_layout_with_functions','render_raw_operation_source_layout_with_functions'];
    const descriptors=new Map(names.map(name=>[name,Object.getOwnPropertyDescriptor(prototype,name)]));
    // Rewrite the real native03 packet into the established02 format. All
    // non-column fields stay byte-identical; no second parse or model fixture.
    const legacyColumns=bytes=>{
      const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);let at=45;
      const need=n=>{if(!Number.isSafeInteger(n)||n<0||n>bytes.length-at)throw Error('Truncated source column fixture');};
      const word=()=>{need(4);const n=view.getUint32(at,true);at+=4;return n;};
      for(let n=word();n;n--){const size=word()*2;need(size);at+=size;}
      const columnCount=word(),prefix=bytes.slice(0,at),parts=[prefix];prefix[7]=50;
      for(let c=0;c<columnCount;c++){
        need(1);const tag=view.getUint8(at++),count=word();if(![4,5,6].includes(tag))throw Error('Unexpected typed source column');
        while(at%4){need(1);if(bytes[at++]!==0)throw Error('Nonzero source padding');}
        const stride=tag===4?8:4;need(count*stride);
        const old=new Uint8Array(4+count*9),out=new DataView(old.buffer);out.setUint32(0,count,true);
        for(let i=0;i<count;i++){const dst=4+i*9,src=at+i*stride;out.setUint8(dst,tag);out.setUint32(dst+1,view.getUint32(src,true),true);out.setUint32(dst+5,tag===4?view.getUint32(src+4,true):0,true);}
        at+=count*stride;parts.push(old);
      }
      parts.push(bytes.subarray(at));const out=new Uint8Array(parts.reduce((n,p)=>n+p.length,0));let offset=0;
      for(const part of parts){out.set(part,offset);offset+=part.length;}return out;
    };
    const restore=()=>{for(const[name,descriptor]of descriptors)Object.defineProperty(prototype,name,descriptor);};
    const report={rows:[],restored:false};
    if([...descriptors.values()].some(value=>typeof value?.value!=='function'))return{...report,fatal:{message:'Expected all source layout capabilities'}};
    try{
      for(const raw of [false,true]){
        const row={raw};
        for(const mode of ['recipe','legacy02','canonical','full']){
          restore();const engine=await loadRenderer();let model;const packets={snapshot:[],methods:0,renders:0,canonical:0,full:0,updates:0,legacy:0,copied:0,poisoned:0};const borrowed=[];const jobs=installJobs(prototype);
          const poison=()=>{for(const bytes of borrowed){bytes.fill(0xa5);packets.poisoned++;}borrowed.length=0;};
          const ownedInput=packet=>{const bytes=packet[2],magic=String.fromCharCode(...bytes.subarray(0,8));const transformed=mode==='legacy02'&&magic==='PLCSRC03'?legacyColumns(bytes):bytes;if(transformed!==bytes)packets.legacy++;const unaligned=new Uint8Array(transformed.length+1).subarray(1);unaligned.set(transformed);packet[2]=unaligned;borrowed.push(unaligned);packets.copied++;return packet;};
          try{
            for(const[name,descriptor]of descriptors)Object.defineProperty(prototype,name,{...descriptor,value:function(...args){
              const packet=Reflect.apply(descriptor.value,this,args);
              if(!Array.isArray(packet)||(packet.length!==4&&packet.length!==6)||packet[0]!==1)throw Error('Expected source layout outcome');
              const magic=String.fromCharCode(...packet[2].subarray(0,8));if(!['PLCSRC01','PLCSRC02','PLCSRC03','PLCSRU01'].includes(magic))throw Error('Small source fixture must retain real source recipe');
              if(name==='snapshot_source_layout'){packets.snapshot.push({magic,type:JSON.parse(packet[3]).type});return ownedInput(packet);}
              if(magic==='PLCSRU01')packets.updates++;if(name==='invoke_source_layout_with_functions')packets.methods++;else packets.renders++;
              if(mode==='full'){const full=Reflect.apply(descriptors.get('snapshot_source_layout').value,this,[]);if(!Array.isArray(full)||full[1]!==packet[1]||String.fromCharCode(...full[2].subarray(0,8))!=='PLCSRC03')throw Error('Expected full snapshot retaining original source catalog');packet[2]=full[2];packets.full++;}
              if(mode==='canonical'){
                // This forces only the physical fallback decoder. It does not
                // claim to execute the native 64 MiB recipe-overflow branch.
                const full=this.snapshot_binary(),view=new DataView(full.buffer,full.byteOffset,full.byteLength),metadataLength=view.getUint32(20,true);
                const graph=full.slice(0,full.length-metadataLength);new DataView(graph.buffer,graph.byteOffset,graph.byteLength).setUint32(20,0,true);
                if(graph.byteLength!==packet[1])throw Error('Canonical frame count mismatch');
                packet[2]=graph;packets.canonical++;
              }
              return ownedInput(packet);
            }});
            model=engine.parse(source);poison();const nodes=model.nodes,links=model.links,lookup=model.lookup,first=nodes[0];
            const firstHelper=model.findOrCreateNode('A');poison();
            const extra={text:'漢😀\ud800',nan:number('fff8000000000042'),zero:-0,infinity:Infinity};extra.self=extra;
            const initialNumeric={nan:bits(extra.nan),zero:bits(extra.zero),infinity:bits(extra.infinity)},numericHistory=[];
            const trackNumeric=stage=>{poison();numericHistory.push({stage,nan:bits(extra.nan),zero:bits(extra.zero),infinity:bits(extra.infinity)});};
            extra.map=new Map([[NaN,extra],[-0,first]]);extra.set=new Set([extra,first]);
            Object.defineProperty(extra,'hidden',{value:'non-enumerable 漢😀',writable:false,enumerable:false,configurable:true});
            model.extra=extra;model.detached=[extra,first];const detached=model.detached;
            const draw=mutate=>{const trace=[];let fired=false;const options={measurer:{width(text,size,style){trace.push([text,bits(size),style]);if(mutate&&!fired){fired=true;mutate();}return text.length*size*.6;}}};try{const value=raw?engine.renderRaw('sankey',model,{},options):engine.render(model,options);return{ok:true,value:output(value),trace};}catch(value){return{ok:false,error:error(value),trace};}finally{poison();}};
            const seed=draw();trackNumeric('seed');delete model.detached;const detachedSeed=draw();trackNumeric('detached seed');
            first.id='Accepted replacement 漢😀';const stringOverride=draw();trackNumeric('accepted label replacement');
            first.id='A';const stringReset=draw();trackNumeric('reset original label');
            const hit=model.findOrCreateNode('A');trackNumeric('existing helper');
            const added=model.findOrCreateNode('漢😀');trackNumeric('append helper');model.addLink(nodes[2],added,2);trackNumeric('link helper');
            model.title='Title 漢😀 &amp;';links[0].value=7;
            const edited=draw(),before=snapshot([model,detached,extra]);trackNumeric('edited');let fired=false;
            const callback=draw(()=>{fired=true;extra.text='callback';extra.zero=123;delete model.extra;detached[0]={unaccepted:true};});
            const accepted=snapshot([model,detached,extra]);trackNumeric('callback retained');
            const ref=oracle.parse(source),refExtra={text:'before',zero:-0};refExtra.self=refExtra;ref.extra=refExtra;const refDetached=[refExtra,ref.nodes[0]];let oracleFired=false;
            oracle.render(ref,{measurer:{width(text,size){if(!oracleFired){oracleFired=true;refExtra.text='callback';refExtra.zero=123;delete ref.extra;refDetached[0]={unaccepted:true};}return text.length*size*.6;}}});
            const mutationState=(root,value,detached)=>({rootAbsent:!Object.hasOwn(root,'extra'),text:value.text,zero:bits(value.zero),cycle:value.self===value,detachedReplacement:detached[0].unaccepted===true,firstAlias:detached[1]===root.nodes[0]});
            const oracleMutation=mutationState(ref,refExtra,refDetached),actualMutation=mutationState(model,extra,detached);
            first.id=42;const invalid=draw(),invalidState=snapshot([model,detached,extra]).signature;trackNumeric('invalid edit');
            first.id='A';const recovery=draw();trackNumeric('repair');const next=draw(),after=snapshot([model,detached,extra]);trackNumeric('next');
            row[mode]={seed,detachedSeed,stringOverride,stringReset,edited,before:before.signature,callback,accepted:accepted.signature,invalid,invalidState,recovery,next,after:after.signature,fired,oracleFired,oracleMutation,actualMutation,identity:firstHelper===first&&model.nodes===nodes&&model.links===links&&model.lookup===lookup&&hit===first&&lookup.get('A')===first&&added===nodes.at(-1)&&lookup.get('漢😀')===added&&links.at(-1).target===added&&!Object.hasOwn(model,'extra')&&extra.self===extra&&detached[0].unaccepted===true&&detached[1]===first,initialNumeric,numericHistory,numeric:{nan:bits(extra.nan),zero:bits(extra.zero),infinity:bits(extra.infinity)}};
          }catch(value){row[mode]={fatal:error(value)};}
          finally{poison();if(model)engine.dispose(model);engine.close();jobs.restore();restore();row[mode+'Packets']={...packets,jobs:jobs.report()};}
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
    expect(row.recipe.fatal,`${row.raw}/recipe`).toBeUndefined();expect(row.legacy02.fatal,`${row.raw}/legacy02`).toBeUndefined();expect(row.canonical.fatal,`${row.raw}/canonical`).toBeUndefined();expect(row.full.fatal,`${row.raw}/full`).toBeUndefined();
    // Engines can box the freshly created fixture NaN differently between
    // iterations. Prove exact original bits in EACH lane before comparing
    // that one input field symbolically across independently created models.
    const comparable=value=>{
      expect(value.numeric).toEqual({...value.initialNumeric,zero:'405ec00000000000'});expect(value.numericHistory).toHaveLength(12);
      for(const [index,{stage,...numeric}]of value.numericHistory.entries())expect(numeric,stage).toEqual({...value.initialNumeric,...(index>=8?{zero:'405ec00000000000'}:{})});
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
    expect(comparable(row.legacy02),row.raw+'/legacy02').toEqual(comparable(row.recipe));
    expect(comparable(row.full),row.raw+'/full').toEqual(comparable(row.recipe));
    expect(row.canonical).toMatchObject({fired:true,oracleFired:true,identity:true,numeric:{zero:'405ec00000000000',infinity:'7ff0000000000000'}});
    expect(row.canonical.numeric).toEqual({...row.canonical.initialNumeric,zero:'405ec00000000000'});expect(row.canonical.actualMutation).toEqual(row.canonical.oracleMutation);
    const nan=BigInt('0x'+row.canonical.numeric.nan);expect((nan&0x7ff0000000000000n)===0x7ff0000000000000n&&(nan&0xfffffffffffffn)!==0n).toBe(true);
    expect(row.canonical.accepted).not.toEqual(row.canonical.before);expect(row.canonical.callback).toEqual(row.canonical.edited);
    expect(row.canonical.invalid.ok).toBe(false);expect(row.canonical.recovery.ok).toBe(true);expect(row.canonical.next).toEqual(row.canonical.recovery);
    for(const mode of ['recipe','legacy02','canonical','full']){expect(row[mode+'Packets'].snapshot).toEqual([{magic:'PLCSRC03',type:'sankey'}]);expect(row[mode+'Packets'].methods).toBe(4);expect(row[mode+'Packets'].renders).toBe(0);const j=row[mode+'Packets'].jobs;expect(j.created).toBeGreaterThanOrEqual(6);expect(j.runs).toBe(j.created);expect(j.finishes).toBe(j.created);expect(j.materialized).toBe(j.created);expect(j[row.raw?'prepareNumbers':'prepareRaw']).toBe(0);for(const record of j.records)expect(record).toMatchObject({runs:1,finishes:1,materialized:1});}
    expect(row.canonicalPackets.methods).toBe(row.recipePackets.methods);expect(row.canonicalPackets.renders).toBe(row.recipePackets.renders);
    expect(row.canonicalPackets.canonical).toBe(row.canonicalPackets.methods+row.canonicalPackets.renders);expect(row.recipePackets.canonical).toBe(0);
    for(const mode of ['recipe','legacy02','canonical','full'])expect(row[mode+'Packets'].updates).toBe(row[mode+'Packets'].methods+row[mode+'Packets'].renders);
    for(const mode of ['recipe','legacy02','canonical','full']){const p=row[mode+'Packets'];expect(p.copied).toBe(1+p.methods+p.renders);expect(p.poisoned).toBe(p.copied);}
    expect(row.legacy02Packets.legacy).toBe(1);expect(row.recipePackets.legacy).toBe(0);
    expect(row.fullPackets.full).toBe(row.fullPackets.updates);expect(row.recipe.stringOverride.ok).toBe(true);expect(row.recipe.stringReset).toEqual(row.recipe.detachedSeed);
  }
});


test('source layout callbacks retain edits while helper publication isolates nested cursors',async({page},info)=>{
  test.setTimeout(120000);await prepare(page);
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const {bits,number,snapshot,output,stable,error,source}=window.__sourceLayoutFeature;
    const model=engine.parse(source),nested=engine.parse(source),reference=oracle.parse(source);
    const width=(text,size)=>text.length*size*.6,options={measurer:{width}};
    const expected=output(oracle.render(reference,options)),nestedExpected=expected;
    let observeArmed=false,publishArmed=false,inside=false;const events=[],nestedResults=[];
    const nestedRead=phase=>{
      if(inside)return;inside=true;
      try{events.push(phase);const alias=nested.findOrCreateNode('A');nestedResults.push({alias:alias===nested.nodes[0]&&alias===nested.lookup.get('A'),value:output(engine.render(nested,options))});}
      finally{inside=false;}
    };
    const target={scalar:number('fff8000000000042'),zero:-0,text:'cursor \ud800 title'};
    Object.defineProperty(target,'__proto__',{value:model.nodes[0],enumerable:true,writable:false,configurable:false});
    const proxy=new Proxy(target,{
      getOwnPropertyDescriptor(object,key){if(observeArmed){observeArmed=false;nestedRead('descriptor');}return Reflect.getOwnPropertyDescriptor(object,key);},
      defineProperty(object,key,descriptor){if(publishArmed){publishArmed=false;nestedRead('define');}return Reflect.defineProperty(object,key,descriptor);}
    });
    const list=[proxy,model.nodes[0],-0];list.length=5;
    const map=new Map([[proxy,model.nodes[0]],[-0,proxy],[NaN,list]]),set=new Set([proxy,list,-0,NaN]);
    target.self=proxy;model.extension={proxy,list,map,set,date:new Date(1234)};
    const referenceTarget={scalar:target.scalar,zero:-0,text:'cursor \ud800 title'};
    Object.defineProperty(referenceTarget,'__proto__',{value:reference.nodes[0],enumerable:true,writable:false,configurable:false});
    const referenceProxy=new Proxy(referenceTarget,{}),referenceList=[referenceProxy,reference.nodes[0],-0];referenceList.length=5;
    const referenceMap=new Map([[referenceProxy,reference.nodes[0]],[-0,referenceProxy],[NaN,referenceList]]),referenceSet=new Set([referenceProxy,referenceList,-0,NaN]);
    referenceTarget.self=referenceProxy;reference.extension={proxy:referenceProxy,list:referenceList,map:referenceMap,set:referenceSet,date:new Date(1234)};
    let row={};
    try{
      const acceptedOutput=output(engine.render(model,options)),initial=snapshot([model,proxy,list,map,set]);
      const originalBits=bits(target.scalar),initialProto=Object.getPrototypeOf(target),fixed=Object.getOwnPropertyDescriptor(target,'__proto__');
      observeArmed=true;const matched=output(engine.render(model,options));observeArmed=false;
      const afterMatch=snapshot([model,proxy,list,map,set]);
      const sentinel={cursor:'original callback throw'};let original=false,threw=false,mutated=false,oracleOriginal=false,oracleMutated=false;
      publishArmed=true;
      try{engine.render(model,{measurer:{width(){if(!mutated){mutated=true;target.scalar=7;target.zero=1;target.text='callback';target.extra='remove';list[0]={wrong:true};map.clear();set.clear();}throw sentinel;}}});}
      catch(value){threw=true;original=Object.is(value,sentinel);}
      finally{publishArmed=false;observeArmed=false;}
      try{oracle.render(reference,{measurer:{width(){if(!oracleMutated){oracleMutated=true;referenceTarget.scalar=7;referenceTarget.zero=1;referenceTarget.text='callback';referenceTarget.extra='remove';referenceList[0]={wrong:true};referenceMap.clear();referenceSet.clear();}throw sentinel;}}});}
      catch(value){oracleOriginal=Object.is(value,sentinel);}
      const retained=snapshot([model,proxy,list,map,set]),oracleRetained=snapshot([reference,referenceProxy,referenceList,referenceMap,referenceSet]);
      const renderEvents=[...events],retainedBits=bits(target.scalar),recovery=output(engine.render(model,options)),oracleRecovery=output(oracle.render(reference,options));
      // User repair is explicit. The readonly render ACK must not do this.
      target.scalar=number(originalBits);target.zero=-0;target.text='cursor \ud800 title';delete target.extra;list[0]=proxy;
      map.clear();map.set(proxy,model.nodes[0]);map.set(-0,proxy);map.set(NaN,list);
      set.clear();set.add(proxy);set.add(list);set.add(-0);set.add(NaN);
      // Keep the real retained-publication reentry gate: NaN property publication
      // defines the proxy descriptor during a helper outcome, not render exit.
      publishArmed=true;let alias;try{alias=model.findOrCreateNode('A');}finally{publishArmed=false;}
      const afterHelper=snapshot([model,proxy,list,map,set]);
      row={acceptedOutput,expected,matched,recovery,oracleRecovery,initial:initial.signature,afterMatch:afterMatch.signature,retained:retained.signature,oracleRetained:oracleRetained.signature,afterHelper:afterHelper.signature,originalBits,retainedBits,repairedBits:bits(target.scalar),mutated,threw,original,oracleOriginal,oracleMutated,renderEvents,events,nestedResults,nestedExpected,stable:[afterMatch,afterHelper].map(value=>stable(initial.objects,value.objects)),helperAlias:alias===model.nodes[0]&&alias===model.lookup.get('A'),prototypeStable:Object.getPrototypeOf(target)===initialProto,fixedDescriptor:Object.getOwnPropertyDescriptor(target,'__proto__').value===fixed.value&&Object.getOwnPropertyDescriptor(target,'__proto__').writable===false&&Object.getOwnPropertyDescriptor(target,'__proto__').configurable===false};
    }catch(value){row.fatal=error(value);row.events=events;}
    finally{observeArmed=false;publishArmed=false;engine.dispose(model);engine.dispose(nested);engine.close();}
    return row;
  });
  await save(info,'source-layout-cursor-reentry.json',report);
  expect(report.fatal).toBeUndefined();expect(report).toMatchObject({mutated:true,threw:true,original:true,oracleMutated:true,oracleOriginal:true,helperAlias:true,prototypeStable:true,fixedDescriptor:true,stable:[true,true]});
  expect(report.renderEvents).toEqual(['descriptor']);expect(report.events).toEqual(['descriptor','define']);expect(report.nestedResults).toHaveLength(2);
  for(const nested of report.nestedResults){expect(nested.alias).toBe(true);expect(nested.value).toEqual(report.nestedExpected);}
  expect(report.retained).toEqual(report.oracleRetained);expect(report.retained).not.toEqual(report.initial);expect(report.retainedBits).toBe('401c000000000000');
  expect(report.repairedBits).toBe(report.originalBits);
  for(const stage of ['afterMatch','afterHelper'])expect(report[stage],stage).toEqual(report.initial);
  for(const stage of ['acceptedOutput','matched','recovery'])expect(report[stage],stage).toEqual(report.expected);
  expect(report.recovery).toEqual(report.oracleRecovery);
});

test('source ingress preserves edited lifecycle, rejected observation repair and helper partial history',async({page},info)=>{
  test.setTimeout(120000);await prepare(page);
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),wasm=await import('/dist/package-web/package/pkg/pele.js');
    const oracle=await import('/dist/evidence/pele-oracle.js');
    const {snapshot,output,error,source}=window.__sourceLayoutFeature,p=wasm.MutableDocument.prototype;
    const names=['source_ingress_identity','apply_source_input','apply_binary'],saved=new Map(names.map(n=>[n,Object.getOwnPropertyDescriptor(p,n)]));
    if([...saved.values()].some(d=>typeof d?.value!=='function'))throw Error('Expected source ingress capabilities');
    const restore=()=>{for(const[n,d]of saved)Object.defineProperty(p,n,d);};const rows=[];
    try{
      for(const mode of ['sparse','canonical']){
        restore();const counts={sparse:0,canonical:0};
        if(mode==='canonical')Object.defineProperty(p,'source_ingress_identity',{...saved.get('source_ingress_identity'),value(){return undefined;}});
        for(const name of ['apply_source_input','apply_binary']){const d=saved.get(name);Object.defineProperty(p,name,{...d,value:function(...args){counts[name==='apply_source_input'?'sparse':'canonical']++;return Reflect.apply(d.value,this,args);}});}
        const engine=await loadRenderer();let model;const row={mode,steps:[],counts};
        try{
          model=engine.parse(source);const first=model.nodes[0],lookup=model.lookup,detached=[];
          const draw=mutate=>{const trace=[];let fired=false;let value;try{value=output(engine.render(model,{measurer:{width(text,size,style){trace.push([text,size,style]);if(mutate&&!fired){fired=true;mutate();}return text.length*size*.6;}}}));return{value,trace};}catch(e){return{error:error(e),trace,thrown:e};}};
          const record=name=>{const rendered=draw();row.steps.push({name,graph:snapshot([model,detached]).signature,...(rendered.error?{error:rendered.error}:{value:rendered.value}),trace:rendered.trace,aliases:model.links[0].source===first&&lookup.get('A')===first&&model.lookup===lookup});};
          const extra={label:'\ud800\u0000\udc00',negativeZero:-0,infinity:Infinity};extra.self=extra;model.extension=extra;detached.push(extra);record('edited');
          const returned=model.findOrCreateNode('A');row.helperAlias=returned===first&&lookup.get('A')===first;
          // Accessor failure follows retained-history additions. Repair must not
          // replay an observed-baseline delta against stale accepted native data.
          model.pending={label:'history before rejected accessor'};Object.defineProperty(model.pending,'bad',{get(){return 1;},configurable:true});
          const callsBefore=counts.sparse+counts.canonical;const rejected=draw();row.rejected={error:rejected.error,noApply:counts.sparse+counts.canonical===callsBefore};delete model.pending.bad;delete model.pending;record('repaired-observation');
          first.id=123;const invalid=draw();row.latent=invalid.error;first.id='A';record('repaired-import');
          const before=snapshot([model,detached]),sentinel={sourceIngress:'callback'};
          const thrown=draw(()=>{extra.label='unaccepted';first.id='callback';lookup.clear();delete model.extension;throw sentinel;});
          row.originalThrow=thrown.thrown===sentinel;row.callbackState=snapshot([model,detached]).signature;
          const ref=oracle.parse(source),refExtra={label:'\ud800\u0000\udc00',negativeZero:-0,infinity:Infinity};refExtra.self=refExtra;ref.extension=refExtra;const refDetached=[refExtra];let oracleFired=false;
          try{oracle.render(ref,{measurer:{width(){oracleFired=true;refExtra.label='unaccepted';ref.nodes[0].id='callback';ref.lookup.clear();delete ref.extension;throw sentinel;}}});}catch(e){row.oracleOriginalThrow=e===sentinel;}
          row.oracleFired=oracleFired;row.oracleCallbackState=snapshot([ref,refDetached]).signature;record('callback-recovery');
          // Scalar format rejection stays before native ingress; the Proxy also
          // remains in detached observation history after removing its root edge.
          let invalidLength=true;const proxy=new Proxy([],{get(target,key,receiver){return key==='length'&&invalidLength ? .5 : Reflect.get(target,key,receiver);}});
          model.invalidArray=proxy;const beforeScalar=counts.sparse+counts.canonical;const scalar=draw();row.scalar={error:scalar.error,noApply:counts.sparse+counts.canonical===beforeScalar};invalidLength=false;delete model.invalidArray;record('scalar-recovery');
          const locked=model.nodes;Object.defineProperty(locked,'length',{writable:false});record('locked');
          try{model.findOrCreateNode('partial-node');row.partial={missingThrow:true};}catch(e){row.partial=error(e);}
          detached.push(locked);model.nodes=[...locked];record('partial-helper-repair');
          row.finalAliases=extra.self===extra&&!Object.hasOwn(model,'extension')&&model.links[0].source===first;
        }catch(e){row.fatal=error(e);}finally{if(model)engine.dispose(model);engine.close();}
        rows.push(row);
      }
    }finally{restore();}
    return{rows,restored:names.every(n=>{const a=Object.getOwnPropertyDescriptor(p,n),b=saved.get(n);return a.value===b.value&&a.writable===b.writable&&a.configurable===b.configurable&&a.enumerable===b.enumerable;})};
  });
  await save(info,'source-ingress-edited-lifecycle.json',report);
  expect(report.restored).toBe(true);expect(report.rows).toHaveLength(2);
  const [actual,control]=report.rows;for(const row of report.rows){expect(row.fatal).toBeUndefined();expect(row.helperAlias).toBe(true);expect(row.originalThrow).toBe(true);expect(row.oracleFired).toBe(true);expect(row.oracleOriginalThrow).toBe(true);expect(row.callbackState).toEqual(row.oracleCallbackState);expect(row.finalAliases).toBe(true);expect(row.rejected.noApply).toBe(true);expect(row.scalar.noApply).toBe(true);expect(row.rejected.error).toBeDefined();expect(row.scalar.error).toBeDefined();expect(row.latent).toBeDefined();expect(row.partial.missingThrow).toBeUndefined();}
  expect(actual.counts.sparse).toBeGreaterThan(0);expect(actual.counts.canonical).toBe(0);expect(control.counts.sparse).toBe(0);expect(control.counts.canonical).toBeGreaterThan(0);
  expect(actual.steps).toEqual(control.steps);expect(actual.rejected).toEqual(control.rejected);expect(actual.scalar).toEqual(control.scalar);expect(actual.latent).toEqual(control.latent);expect(actual.partial).toEqual(control.partial);
});
