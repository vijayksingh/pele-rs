import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

async function save(testInfo,name,report){
  const path=testInfo.outputPath(name);
  await writeFile(path,JSON.stringify(report,null,2));
  await testInfo.attach(name,{path,contentType:'application/json'});
}

test('source packets preserve all 33 public source results, exact legacy envelope counts and resource order',async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const oracle=await import('/dist/evidence/pele-oracle.js');
    const fixture=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
    const engine=await loadRenderer(),prototype=wasm.NativeRegistry.prototype;
    const packet=Object.getOwnPropertyDescriptor(prototype,'render_numbers_packet_with_resources');
    const legacy=Object.getOwnPropertyDescriptor(prototype,'render_numbers_with_resources');
    if(!packet||!legacy)throw Error('Both source transport capabilities required');
    const bitsView=new DataView(new ArrayBuffer(8));
    const bits=value=>{bitsView.setFloat64(0,value);return bitsView.getBigUint64(0).toString(16).padStart(16,'0');};
    const snapshot=value=>({keys:Object.keys(value),type:value.type,svg:value.svg,width:bits(value.width),height:bits(value.height),links:value.links.map(link=>({keys:Object.keys(link),...link}))});
    const rows=[];let observed;
    const restore=()=>{Object.defineProperty(prototype,'render_numbers_packet_with_resources',packet);Object.defineProperty(prototype,'render_numbers_with_resources',legacy);};
    const cases=fixture.cases.filter(c=>c.id.endsWith('/baseline')).map(c=>({id:c.id,source:c.source,options:c.options}));
    cases.push({id:'flowchart/links',source:'flowchart TD\nA["Note &amp; 漢😀"]:::internal-link --> B[Other]',options:{links:true}});
    try{
      for(const c of cases){
        const row={id:c.id};
        for(const mode of ['oracle','legacy','packet']){
          restore();observed=undefined;
          Object.defineProperty(prototype,'render_numbers_packet_with_resources',{...packet,value:mode==='legacy'?undefined:function(...args){const value=packet.value.apply(this,args);observed=typeof value==='string'?{ok:false,json:value}:{ok:true,bytes:value[1]};return value;}});
          Object.defineProperty(prototype,'render_numbers_with_resources',{...legacy,value:function(...args){const value=legacy.value.apply(this,args);observed=JSON.parse(value).ok?{ok:true,bytes:new TextEncoder().encode(value).length}:{ok:false,json:value};return value;}});
          const trace=[],options={...c.options,now:1704067200000,measurer:{width(text,size,style){trace.push(['width',text,bits(size),style]);return text.length*size*.6;}},icons(name){trace.push(['icon',name]);return '<g data-q="&quot;\\漢😀"/>';}};
          try{row[mode]={ok:true,value:snapshot((mode==='oracle'?oracle:engine).render(c.source,options)),trace};}
          catch(error){row[mode]={ok:false,error:{name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet},trace};}
          if(mode!=='oracle')row[mode].transport=observed;
        }
        rows.push(row);
      }
      return rows;
    }finally{restore();engine.close();}
  });
  await save(testInfo,'source-packet-all-families.json',report);
  expect(report).toHaveLength(34);
  for(const row of report){
    expect(row.packet,`${row.id}: old/new transport`).toEqual(row.legacy);
    const {transport,...actual}=row.packet;
    expect(actual,`${row.id}: pinned public oracle`).toEqual(row.oracle);
    expect(actual.ok,`${row.id}: must reach success`).toBe(true);
    expect(transport.ok).toBe(true);
  }
  expect(report.find(row=>row.id==='flowchart/links').packet.value.links.length).toBeGreaterThan(0);
});

// Remove only the optional JS export for the fallback page. All legacy exports,
// WASM bytes and public facade code remain identical. No replay-on-error path.
for(const capability of [true,false])test(`registry-free public source renderer uses ${capability?'packet':'legacy capability fallback'} across all families`,async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.route('**/dist/package-web/package/pkg/pele.js',async route=>{
    const response=await route.fetch(),original=await response.text();
    const declaration='export function render_numbers_packet_with_resources(';
    if(!original.includes(declaration))throw Error('Expected optional top-level source capability');
    let body=original.replace(declaration,'function sourcePacketOriginal(');
    if(capability)body+='\nexport function render_numbers_packet_with_resources(...args){const value=sourcePacketOriginal(...args);globalThis.__sourcePackets=(globalThis.__sourcePackets??0)+1;return value;}\n';
    await route.fulfill({response,body});
  });
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer,createRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js');
    const initialized=await loadRenderer();initialized.close();
    const engine=createRenderer(undefined),fixture=await(await fetch('/fixtures/pele/diagram-entries.json')).json(),rows=[];
    const view=new DataView(new ArrayBuffer(8)),bits=value=>{view.setFloat64(0,value);return view.getBigUint64(0).toString(16);};
    for(const c of fixture.cases.filter(c=>c.id.endsWith('/baseline'))){
      const row={id:c.id};
      for(const [name,api]of [['oracle',oracle],['actual',engine]]){
        const trace=[],options={...c.options,now:1704067200000,measurer:{width(text,size,style){trace.push([text,bits(size),style]);return text.length*size*.6;}}};
        try{const value=api.render(c.source,options);row[name]={ok:true,keys:Object.keys(value),value:{...value,width:bits(value.width),height:bits(value.height)},trace};}
        catch(error){row[name]={ok:false,name:error?.name,message:error?.message,trace};}
      }
      rows.push(row);
    }
    return {rows,packets:globalThis.__sourcePackets??0};
  });
  await save(testInfo,`source-packet-free-${capability}.json`,report);
  expect(report.rows).toHaveLength(33);
  for(const row of report.rows){expect(row.actual,row.id).toEqual(row.oracle);expect(row.actual.ok,row.id).toBe(true);}
  expect(report.packets).toBe(capability?33:0);
});

test('source packet dimensions, failures, nested callbacks, provider isolation and recovery match legacy',async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer,loadRegistry}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const prototype=wasm.NativeRegistry.prototype,packet=Object.getOwnPropertyDescriptor(prototype,'render_numbers_packet_with_resources'),legacy=Object.getOwnPropertyDescriptor(prototype,'render_numbers_with_resources');
    const view=new DataView(new ArrayBuffer(8)),bits=value=>{view.setFloat64(0,value);return view.getBigUint64(0).toString(16).padStart(16,'0');},number=hex=>{view.setBigUint64(0,BigInt('0x'+hex));return view.getFloat64(0);};
    const result=value=>({keys:Object.keys(value),...value,width:bits(value.width),height:bits(value.height)});
    const error=value=>({name:value?.name,message:value?.message,code:value?.code,type:value?.type,line:value?.line,column:value?.column,snippet:value?.snippet});
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns',rows=[];
    const variants=['default','null-limit','zero-limit','nan-limit','negative-zero','nan-dimensions','infinite-dimensions','unicode','invalid-grammar','source-limit','edge-limit','bad-config','nested','throw-object','throw-string','throw-undefined','throw-null','throw-symbol','clock-throw','closed','provider'];
    const restore=()=>{Object.defineProperty(prototype,'render_numbers_packet_with_resources',packet);Object.defineProperty(prototype,'render_numbers_with_resources',legacy);};
    try{
      for(const variant of variants){
        const row={variant};
        for(const mode of ['legacy','packet']){
          restore();let packets=0,bytes,rawError;
          Object.defineProperty(prototype,'render_numbers_packet_with_resources',{...packet,value:mode==='legacy'?undefined:function(...args){const value=packet.value.apply(this,args);if(typeof value==='string'){rawError=value;}else{packets++;bytes=value[1];}return value;}});
          Object.defineProperty(prototype,'render_numbers_with_resources',{...legacy,value:function(...args){const value=legacy.value.apply(this,args);if(JSON.parse(value).ok)bytes=new TextEncoder().encode(value).length;else rawError=value;return value;}});
          const engine=await loadRenderer(),trace=[],sentinel=variant==='throw-undefined'?undefined:variant==='throw-null'?null:variant==='throw-symbol'?Symbol('sentinel'):variant==='throw-string'?'original string':{sentinel:true};
          let text=source,options={now:1704067200000},fired=false,nested,original=false,threw=false,output,failure;
          if(variant==='null-limit')options.outputLimit=null;
          if(variant==='zero-limit')options.outputLimit=0;
          if(variant==='nan-limit')options.outputLimit=NaN;
          if(variant==='negative-zero')Object.assign(options,{padding:-0,fontSize:0,maxWidth:-0});
          if(variant==='nan-dimensions')Object.assign(options,{padding:number('fff8000000000042'),outputLimit:Infinity});
          if(variant==='infinite-dimensions')Object.assign(options,{padding:Infinity,outputLimit:Infinity});
          if(variant==='unicode')text='erDiagram\naccTitle: BOM \ufeff 漢😀\naccDescr: quoted " \\ text\nA["fa:fa-user"]';
          if(variant==='invalid-grammar')text='erDiagram\nA { ??? }';
          if(variant==='source-limit')options.limit=0;
          if(variant==='edge-limit')options.maxEdges=0;
          if(variant==='bad-config')options.config={er:17};
          options.measurer={width(text,size,style){trace.push(['width',text,bits(size),style]);if(!fired){fired=true;if(variant==='nested')nested=result(engine.render('pie\n"Nested":1',{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}}));if(variant.startsWith('throw-'))throw sentinel;}return text.length*size*.6;}};
          options.icons=name=>{trace.push(['icon',name]);return '<g data-value="\ufeff漢😀&quot;\\"/>';};
          if(variant==='closed')engine.close();
          if(variant==='provider')engine.register({type:'er',parse(){trace.push(['parse-provider']);return {};},render(){trace.push(['render-provider']);return {svg:'<svg data-provider="true"/>',width:-0,height:3,links:[]};}});
          const oldNow=Date.now;if(variant==='clock-throw'){delete options.now;Date.now=()=>{throw sentinel;};text='gantt\ndateFormat YYYY-MM-DD\nTask :a, 2024-01-01, 1d';}
          try{output=result(engine.render(text,options));}catch(value){threw=true;original=value===sentinel;failure=error(value);}finally{Date.now=oldNow;}
          const observed={bytes,rawError};
          if(variant!=='closed')engine.close();
          const recovered=await loadRenderer();const recovery=result(recovered.render(source,{now:1704067200000,measurer:{width:(text,size)=>text.length*size*.6}}));recovered.close();
          row[mode]={threw,original,output,nested,failure,trace,recovery,observed};row[mode+'Packets']=packets;
        }
        rows.push(row);
      }
      const empty=await loadRegistry(),emptyError=(()=>{try{empty.render(source);return null;}catch(value){return error(value);}})();empty.close();
      return {rows,emptyError};
    }finally{restore();}
  });
  await save(testInfo,'source-packet-controls.json',report);
  for(const row of report.rows)expect(row.packet,row.variant).toEqual(row.legacy);
  for(const row of report.rows.filter(row=>row.variant.startsWith('throw-')))expect(row.packet.original,row.variant).toBe(true);
  expect(report.rows.find(row=>row.variant==='nested').packet.nested).toBeDefined();
  expect(report.rows.find(row=>row.variant==='clock-throw').packet.original).toBe(true);
  expect(report.rows.find(row=>row.variant==='provider').packetPackets).toBe(1); // Recovery only: provider bypasses native source render.
  const nan=report.rows.find(row=>row.variant==='nan-dimensions');
  expect(nan.packet.threw).toBe(false);expect(nan.packetPackets).toBeGreaterThan(0);
  const isNaNBits=hex=>{const n=BigInt('0x'+hex);return(n&0x7ff0000000000000n)===0x7ff0000000000000n&&(n&0x000fffffffffffffn)!==0n;};
  expect(isNaNBits(nan.packet.output.width)||isNaNBits(nan.packet.output.height)).toBe(true);
  expect(report.rows.find(row=>row.variant==='closed').packet.failure.message).toBe('Registry is closed');
  expect(report.emptyError).not.toBeNull();
});

test('source packet preserves exact 64 MiB v2 envelope gate after output generation',async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const prototype=wasm.NativeRegistry.prototype,packet=Object.getOwnPropertyDescriptor(prototype,'render_numbers_packet_with_resources'),legacy=Object.getOwnPropertyDescriptor(prototype,'render_numbers_with_resources');
    const restore=()=>{Object.defineProperty(prototype,'render_numbers_packet_with_resources',packet);Object.defineProperty(prototype,'render_numbers_with_resources',legacy);};
    const maximum=64*1024*1024,rows=[];
    try{
      for(const mode of ['legacy','packet']){
        restore();let observed;
        Object.defineProperty(prototype,'render_numbers_packet_with_resources',{...packet,value:mode==='legacy'?undefined:function(...args){const value=packet.value.apply(this,args);observed=typeof value==='string'?{ok:false,json:value}:{ok:true,bytes:value[1]};return value;}});
        Object.defineProperty(prototype,'render_numbers_with_resources',{...legacy,value:function(...args){const value=legacy.value.apply(this,args);observed=JSON.parse(value).ok?{ok:true,bytes:new TextEncoder().encode(value).length}:{ok:false,json:value};return value;}});
        const engine=await loadRenderer(),options={now:1704067200000,limit:Infinity,outputLimit:Infinity,maxWidth:10000,measurer:{width:(text,size)=>text.length*size*.6}};
        const source=n=>'erDiagram\naccDescr: '+ '"'.repeat(n)+'\nA';
        engine.render(source(1),options);const first=observed.bytes;engine.render(source(2),options);const step=observed.bytes-first;
        if(!Number.isSafeInteger(step)||step<2||step>16)throw Error('Unexpected accessibility envelope growth');
        const count=Math.floor((maximum-first)/step)+2;
        if(count>12*1024*1024)throw Error('Bounded source budget fixture exceeded');
        let failure;try{engine.render(source(count),options);}catch(value){failure={name:value?.name,message:value?.message,code:value?.code,type:value?.type};}
        const attempted=observed,repaired=engine.render('erDiagram\nA',options).svg.length;engine.close();
        rows.push({mode,first,step,count,expectedBytes:first+(count-1)*step,failure,attempted,repaired});
      }
      return rows;
    }finally{restore();}
  });
  await save(testInfo,'source-packet-envelope-limit.json',report);
  expect({...report[1],mode:'legacy'}).toEqual(report[0]);
  expect(report[1].expectedBytes).toBeGreaterThan(64*1024*1024);
  expect(report[1].failure).toMatchObject({name:'PeleError',message:'Browser numeric transport exceeds byte budget',code:'limit'});
  expect(report[1].attempted.ok).toBe(false);expect(report[1].repaired).toBeGreaterThan(0);
});
