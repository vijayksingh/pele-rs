import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';
async function save(info,name,report){const path=info.outputPath(name);await writeFile(path,JSON.stringify(report,null,2));await info.attach(name,{path,contentType:'application/json'});}

test('arena packets preserve full graphs, revisions and retained/raw results across all arena families',async({page},info)=>{
  test.setTimeout(180000);await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const {decodeGraphBinary}=await import('/dist/package-web/package/graph-binary.js');
    const fixture=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
    const prototype=wasm.MutableDocument.prototype;
    const pairs=[['render_numbers_operation_packet_with_functions','render_numbers_operation_binary_with_functions'],['render_raw_operation_packet_with_functions','render_raw_operation_binary_with_functions']];
    const descriptors=new Map(pairs.flat().map(k=>[k,Object.getOwnPropertyDescriptor(prototype,k)]));
    if([...descriptors.values()].some(v=>!v))throw Error('Both arena transport capabilities required');
    const restore=()=>{for(const[k,d]of descriptors)Object.defineProperty(prototype,k,d);};
    const view=new DataView(new ArrayBuffer(8)),bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const result=v=>({keys:Object.keys(v),...v,width:bits(v.width),height:bits(v.height)});
    const rows=[],engine=await loadRenderer();let observed;
    const cases=fixture.cases.filter(c=>c.id.endsWith('/baseline')&&!c.id.startsWith('er/'));
    cases.push({id:'flowchart/links',source:'flowchart TD\nA["Note &amp; 漢😀"]:::internal-link --> B[Other]',options:{links:true}});
    try{for(const c of cases)for(const raw of [false,true]){
      const row={id:c.id,raw};
      for(const mode of ['legacy','packet']){
        restore();observed=undefined;
        for(const [packet,legacy]of pairs){
          Object.defineProperty(prototype,packet,{...descriptors.get(packet),value:mode==='legacy'?undefined:function(...args){const value=descriptors.get(packet).value.apply(this,args);const bytes=Array.isArray(value)?value[2]:value;const decoded=decodeGraphBinary(bytes);observed={revision:decoded.revision,graph:decoded.graph,bytes:Array.isArray(value)?value[1]:new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getUint32(20,true)};return value;}});
          Object.defineProperty(prototype,legacy,{...descriptors.get(legacy),value:function(...args){const bytes=descriptors.get(legacy).value.apply(this,args);const decoded=decodeGraphBinary(bytes);observed={revision:decoded.revision,graph:decoded.graph,bytes:new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getUint32(20,true)};return bytes;}});
        }
        const model=engine.parse(c.source),trace=[],options={...c.options,now:1704067200000,measurer:{width(text,size,style){trace.push([text,bits(size),style]);return text.length*size*.6;}}};
        try{const value=raw?engine.renderRaw(c.id.split('/')[0],model,{},options):engine.render(model,options);row[mode]={ok:true,value:result(value),trace,transport:observed};}
        catch(e){row[mode]={ok:false,error:{name:e?.name,message:e?.message,code:e?.code,type:e?.type},trace,transport:observed};}
        engine.dispose(model);
      }rows.push(row);
    }return rows;}finally{restore();engine.close();}
  });
  await save(info,'arena-packet-all-families.json',report);
  expect(report).toHaveLength(66);
  for(const row of report){expect(row.packet,row.id+'/'+row.raw).toEqual(row.legacy);expect(row.packet.transport,row.id).toBeDefined();if(!row.raw)expect(row.packet.ok,row.id).toBe(true);}
  expect(report.find(r=>r.id==='flowchart/links'&&!r.raw).packet.value.links.length).toBeGreaterThan(0);
});

test('arena packet callbacks, full restoration, malformed edits, dimension bits and recovery match legacy',async({page},info)=>{
  test.setTimeout(180000);await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const prototype=wasm.MutableDocument.prototype,keys=['render_numbers_operation_packet_with_functions','render_raw_operation_packet_with_functions'];
    const descriptors=new Map(keys.map(k=>[k,Object.getOwnPropertyDescriptor(prototype,k)]));const restore=()=>{for(const[k,d]of descriptors)Object.defineProperty(prototype,k,d);};
    const view=new DataView(new ArrayBuffer(8)),bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const value=v=>({keys:Object.keys(v),...v,width:bits(v.width),height:bits(v.height)}),rows=[],engine=await loadRenderer();
    try{for(const raw of [false,true])for(const variant of ['ordinary','edited','callback','throw-object','throw-undefined','throw-symbol','limit','nan','readonly','accessor']){
      const row={raw,variant};for(const mode of ['legacy','packet']){
        restore();let packets=0;for(const[k,d]of descriptors)Object.defineProperty(prototype,k,{...d,value:mode==='legacy'?undefined:function(...a){const v=d.value.apply(this,a);if(Array.isArray(v)&&v[3]===0)packets++;return v;}});
        const model=engine.parse('flowchart TD\nA[Alpha] --> B[Beta]');const extra={n:1};extra.self=extra;model.extra=extra;model.detached=[extra];const detached=model.detached;
        const seedOptions={measurer:{width:(text,size)=>text.length*size*.6}};
        if(raw)engine.renderRaw('flowchart',model,{},seedOptions);else engine.render(model,seedOptions);
        delete model.detached;
        if(variant==='edited')extra.n=2;
        if(variant==='readonly')Object.defineProperty(extra,'n',{value:1,writable:false,enumerable:true,configurable:true});
        if(variant==='accessor')Object.defineProperty(extra,'bad',{get(){throw Error('accessor executed');},configurable:true});
        const sentinel=variant==='throw-undefined'?undefined:variant==='throw-symbol'?Symbol('original'):{original:true};let fired=false,output,error,original=false;const trace=[];
        const draw=options=>raw?engine.renderRaw('flowchart',model,{},options):engine.render(model,options);
        try{output=value(draw({outputLimit:variant==='limit'?0:Infinity,padding:variant==='nan'?NaN:8,measurer:{width(text,size,style){trace.push([text,bits(size),style]);if(!fired){fired=true;if(variant==='callback'||variant.startsWith('throw')){extra.n=99;extra.added={};delete model.extra;}if(variant.startsWith('throw'))throw sentinel;}return text.length*size*.6;}}}));}
        catch(e){original=e===sentinel;error={name:e?.name,message:e?.message,code:e?.code,type:e?.type};}
        const state={root:model.extra===extra,cycle:extra.self===extra,n:extra.n,added:Object.hasOwn(extra,'added'),detached:detached[0]===extra,flags:Object.getOwnPropertyDescriptor(extra,'n')};
        delete extra.bad;model.extra=extra;const repaired=value(draw({measurer:{width:(text,size)=>text.length*size*.6}}));engine.dispose(model);
        row[mode]={output,error,original,trace,state,repaired};row[mode+'Packets']=packets;
      }rows.push(row);
    }return rows;}finally{restore();engine.close();}
  });
  await save(info,'arena-packet-callbacks.json',report);
  for(const row of report){expect(row.packet,`${row.raw}/${row.variant}`).toEqual(row.legacy);if(row.variant.startsWith('throw'))expect(row.packet.original).toBe(true);}
  const nan=report.find(r=>!r.raw&&r.variant==='nan');expect(nan.packet.error).toBeUndefined();expect(nan.packetPackets).toBeGreaterThan(0);
  const isNaNBits=hex=>{const n=BigInt('0x'+hex);return(n&0x7ff0000000000000n)===0x7ff0000000000000n&&(n&0xfffffffffffffn)!==0n;};
  expect(isNaNBits(nan.packet.output.width)||isNaNBits(nan.packet.output.height)).toBe(true);
});

test('arena packet factory throws retain committed graph publication and original exception identity',async({page},info)=>{
  await page.route('**/dist/package-web/package/host-er-store.js',async route=>{
    const response=await route.fetch(),text=await response.text(),declaration='export function hostErRenderResult(';
    if(!text.includes(declaration))throw Error('Expected shared result factory');
    await route.fulfill({response,body:text.replace(declaration,'function originalArenaResult(')+'\nexport function hostErRenderResult(...args){if(globalThis.__arenaFactoryThrow){throw globalThis.__arenaFactoryValue;}return originalArenaResult(...args);}\n'});
  });
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');const engine=await loadRenderer(),rows=[];
    try{for(const raw of [false,true])for(const kind of ['object','undefined','symbol']){
      const model=engine.parse('flowchart TD\nA --> B'),extra={n:1};model.extra=extra;
      // Establish the host edit in the native graph before the failing render.
      const draw=options=>raw?engine.renderRaw('flowchart',model,{},options):engine.render(model,options);
      draw({measurer:{width:(text,size)=>text.length*size*.6}});
      const sentinel=kind==='undefined'?undefined:kind==='symbol'?Symbol('factory'):{factory:true};globalThis.__arenaFactoryValue=sentinel;globalThis.__arenaFactoryThrow=true;
      let fired=false,threw=false,original=false;
      try{draw({measurer:{width(text,size){if(!fired){fired=true;extra.n=99;delete model.extra;}return text.length*size*.6;}}});}catch(error){threw=true;original=error===sentinel;}finally{globalThis.__arenaFactoryThrow=false;}
      const restored=model.extra===extra&&extra.n===1;const recovery=draw({measurer:{width:(text,size)=>text.length*size*.6}}).svg.length;engine.dispose(model);
      rows.push({raw,kind,fired,threw,original,restored,recovery});
    }return rows;}finally{globalThis.__arenaFactoryThrow=false;delete globalThis.__arenaFactoryValue;engine.close();}
  });
  await save(info,'arena-packet-factory-errors.json',report);
  for(const row of report){expect(row).toMatchObject({fired:true,threw:true,original:true,restored:true});expect(row.recovery).toBeGreaterThan(0);}
});

test('arena packets retain exact combined graph and metadata 64 MiB admission',async({page},info)=>{
  test.setTimeout(180000);await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const prototype=wasm.MutableDocument.prototype,pairs=[['render_numbers_operation_packet_with_functions','render_numbers_operation_binary_with_functions'],['render_raw_operation_packet_with_functions','render_raw_operation_binary_with_functions']];
    const descriptors=new Map(pairs.flat().map(k=>[k,Object.getOwnPropertyDescriptor(prototype,k)]));const restore=()=>{for(const[k,d]of descriptors)Object.defineProperty(prototype,k,d);};
    const rows=[],maximum=64*1024*1024;let observed;
    try{for(const raw of [false,true])for(const mode of ['legacy','packet']){
      restore();const engine=await loadRenderer(),model=engine.parse('flowchart TD\nA');
      for(const[packet,legacy]of pairs){
        Object.defineProperty(prototype,packet,{...descriptors.get(packet),value:mode==='legacy'?undefined:function(...args){const v=descriptors.get(packet).value.apply(this,args);if(Array.isArray(v))observed={metadata:v[1],frame:v[1]+v[2].length};return v;}});
        Object.defineProperty(prototype,legacy,{...descriptors.get(legacy),value:function(...args){const bytes=descriptors.get(legacy).value.apply(this,args);observed={metadata:new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getUint32(20,true),frame:bytes.length};return bytes;}});
      }
      let calls=0;const draw=()=>{const options={outputLimit:Infinity,maxWidth:10000,measurer:{width(text,size){calls++;return text.length*size*.6;}}};return raw?engine.renderRaw('flowchart',model,{},options):engine.render(model,options);};
      model.accDescr='"';draw();const first={...observed};model.accDescr='""';draw();const second={...observed};
      // Canonical string history retains both calibration strings. The second
      // frame therefore includes new string-table overhead in addition to SVG
      // growth. A third small sample isolates steady per-unit output growth.
      model.accDescr='"""';draw();const third={...observed};
      const metadataStep=third.metadata-second.metadata;
      if(!Number.isInteger(metadataStep)||metadataStep<2||metadataStep>16)throw Error('Expected bounded escaped accessibility growth');
      // New final string adds 4 + 2*n binary bytes; existing small calibration
      // strings stay in history. Metadata is affine in this one string.
      const count=1+Math.floor((maximum-third.frame+3*metadataStep-4)/(metadataStep+2));
      if(count<1||count>12*1024*1024)throw Error('Unexpected frame boundary fixture size');
      const expectedMetadata=third.metadata+(count-3)*metadataStep;
      const expectedFrame=third.frame+4+2*count+(count-3)*metadataStep;
      if(expectedMetadata>=maximum||expectedFrame<=maximum)throw Error('Fixture must isolate combined frame admission');
      model.accDescr='"'.repeat(count);observed=undefined;calls=0;let error;
      try{draw();}catch(e){error={name:e?.name,message:e?.message,code:e?.code,type:e?.type};}
      const attempted=observed,attemptedCalls=calls;model.accDescr='';let retry;
      // Encoding failure happens after native revision advance, but before the
      // facade receives a frame. Preserve the existing stale-revision outcome.
      try{draw();}catch(e){retry={name:e?.name,message:e?.message,code:e?.code};}
      engine.dispose(model);const fresh=engine.parse('flowchart TD\nA');
      const options={measurer:{width:(text,size)=>text.length*size*.6}};
      const recovery=(raw?engine.renderRaw('flowchart',fresh,{},options):engine.render(fresh,options)).svg.length;
      engine.dispose(fresh);engine.close();
      rows.push({raw,mode,first,second,third,count,metadataStep,expectedMetadata,expectedFrame,error,attempted,calls:attemptedCalls,retry,recovery});
    }return rows;}finally{restore();}
  });
  await save(info,'arena-packet-combined-budget.json',report);
  for(const raw of [false,true]){const old=report.find(r=>r.raw===raw&&r.mode==='legacy'),packet=report.find(r=>r.raw===raw&&r.mode==='packet');expect({...packet,mode:'legacy'}).toEqual(old);expect(packet.expectedMetadata).toBeLessThan(64*1024*1024);expect(packet.expectedFrame).toBeGreaterThan(64*1024*1024);expect(packet.error).toMatchObject({name:'PeleError',message:'Public binary graph exceeds transport budget',code:'limit'});expect(packet.calls).toBeGreaterThan(0);expect(packet.attempted).toBeUndefined();expect(packet.retry).toMatchObject({message:'Stale public model revision'});expect(packet.recovery).toBeGreaterThan(0);}
});
