import {test, expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

async function save(testInfo,name,report){
  const path=testInfo.outputPath(name);
  await writeFile(path,JSON.stringify(report,null,2));
  await testInfo.attach(name,{path,contentType:'application/json'});
}

test('ER success packet preserves complete public results, numeric bits, counts and legacy capability fallback',async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const engine=await loadRenderer(),prototype=wasm.HostErDocument.prototype;
    const packetKeys=['render_host_packet','render_raw_host_packet',...(typeof prototype.render_host_frame_packet==='function'?['render_host_frame_packet']:[])],oldKeys=['render_host','render_raw_host'];
    const descriptors=new Map([...packetKeys,...oldKeys].map(key=>[key,Object.getOwnPropertyDescriptor(prototype,key)]));
    if([...descriptors.values()].some(value=>!value))throw new Error('Both public transport capabilities required');
    const data=new DataView(new ArrayBuffer(8));
    const bits=value=>{data.setFloat64(0,value);return data.getBigUint64(0).toString(16).padStart(16,'0');};
    const number=hex=>{data.setBigUint64(0,BigInt('0x'+hex));return data.getFloat64(0);};
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
    const rows=[],counts=[];let observed=[];
    const restore=()=>{for(const [key,d]of descriptors)Object.defineProperty(prototype,key,d);};
    const select=legacy=>{
      restore();
      for(const key of packetKeys)Object.defineProperty(prototype,key,{...descriptors.get(key),value:legacy?undefined:function(...args){
        const value=descriptors.get(key).value.apply(this,args);
        observed.push(typeof value==='string'?{kind:'error',bytes:new TextEncoder().encode(value).length}:{kind:'packet',bytes:value[1]});
        return value;
      }});
      if(legacy)for(const key of oldKeys)Object.defineProperty(prototype,key,{...descriptors.get(key),value:function(...args){
        const value=descriptors.get(key).value.apply(this,args);
        observed.push({kind:Object.hasOwn(JSON.parse(value),'Ok')?'json':'error',bytes:new TextEncoder().encode(value).length});return value;
      }});
    };
    const snapshot=value=>({keys:Object.keys(value),svg:value.svg,width:bits(value.width),height:bits(value.height),links:value.links.map(link=>({keys:Object.keys(link),...link})),type:value.type});
    const options=[['missing',{}],['undefined',{outputLimit:undefined}],['null',{outputLimit:null}],['zero',{outputLimit:0}],['negative-zero',{outputLimit:-0}],['infinity',{outputLimit:Infinity}],['negative-infinity',{outputLimit:-Infinity}],['nan-limit',{outputLimit:NaN}],['zero-dimensions',{padding:-0,fontSize:0,maxWidth:-0}],['nan-dimensions',{padding:number('fff8000000000042'),outputLimit:Infinity}],['infinite-dimensions',{fontSize:Infinity,outputLimit:Infinity}],['unicode-icons',{icons:()=>'<g data-q="&quot;\\\u0000\ufeff漢😀">\n<path d="M0 0h2v2z"/></g>'}]];
    try{
      for(const lane of ['ordinary','prepared','raw','caller-raw'])for(const [name,option]of options){
        const row={lane,name};
        for(const [mode,legacy]of [['json',true],['packet',false]]){
          select(legacy);observed=[];
          let model=lane==='ordinary'?engine.parse(source):engine.parsePrepared('er',source,{},'Prepared',0,undefined);
          if(lane==='caller-raw'){const caller=structuredClone(model);engine.dispose(model);model=caller;}
          model.title='\ufeffUnicode " \\ \u0000 漢😀';model.accTitle='\ufeffAccessible';model.accDescr='Line\nTwo';
          model.entities.get('A').alias=name==='unicode-icons'?'Icon fa:fa-user':'Quoted " \\ 漢😀';
          const trace=[];
          try{
            const settings={...option,measurer:{width(text,size,style){trace.push([text,bits(size),style]);return name==='nan-dimensions'?number('fff8000000000042'):text.length*size*.6;}}};
            const value=lane.endsWith('raw')?engine.renderRaw('er',model,{},settings):engine.render(model,settings);
            row[mode]={ok:true,result:snapshot(value),trace};
          }catch(error){row[mode]={ok:false,error:{name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet},trace};}
          finally{engine.dispose(model);}
          row[mode].bytes=observed.map(value=>value.bytes);counts.push(...observed.map(value=>({lane,name,mode,...value})));
        }
        rows.push(row);
      }
      return {rows,counts};
    }finally{restore();engine.close();}
  });
  await save(testInfo,'er-result-packet-differential.json',report);
  for(const row of report.rows)expect(row.packet,`${row.lane}/${row.name}`).toEqual(row.json);
  expect(report.counts.some(row=>row.kind==='packet')).toBe(true);
  expect(report.counts.some(row=>row.kind==='error')).toBe(true);
  expect(report.rows.filter(row=>row.name==='unicode-icons').every(row=>row.packet.ok)).toBe(true);
  const nanBits=hex=>{const value=BigInt('0x'+hex);return (value&0x7ff0000000000000n)===0x7ff0000000000000n&&(value&0x000fffffffffffffn)!==0n;};
  for(const row of report.rows.filter(row=>row.name==='nan-dimensions')){
    expect(row.packet.ok,`${row.lane}: NaN dimensions must reach successful transport`).toBe(true);
    expect(nanBits(row.packet.result.width)||nanBits(row.packet.result.height)).toBe(true);
    expect(report.counts.some(value=>value.lane===row.lane&&value.name===row.name&&value.mode==='packet'&&value.kind==='packet')).toBe(true);
  }
});

test('ER result packet keeps nested render, original throws, final audit precedence and repair',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const engine=await loadRenderer(),prototype=wasm.HostErDocument.prototype;
    const packetKeys=['render_host_packet','render_raw_host_packet',...(typeof prototype.render_host_frame_packet==='function'?['render_host_frame_packet']:[])];
    const descriptors=new Map(packetKeys.map(key=>[key,Object.getOwnPropertyDescriptor(prototype,key)]));
    const rows=[],source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
    const view=new DataView(new ArrayBuffer(8)),bits=value=>{view.setFloat64(0,value);return view.getBigUint64(0).toString(16);};
    const rendered=value=>({...value,width:bits(value.width),height:bits(value.height)});
    const restore=()=>{for(const [key,d]of descriptors)Object.defineProperty(prototype,key,d);};
    try{
      for(const raw of [false,true])for(const variant of ['nested','throw-object','throw-undefined','throw-null','throw-symbol','audit-with-limit','throw-with-audit-and-limit']){
        const row={raw,variant};
        for(const [mode,legacy]of [['json',true],['packet',false]]){
          restore();if(legacy)for(const [key,d]of descriptors)Object.defineProperty(prototype,key,{...d,value:undefined});
          const model=engine.parse(source),saved=model.entities.get('B').attributes;
          const sentinel=variant==='throw-undefined'?undefined:variant==='throw-null'?null:variant==='throw-symbol'?Symbol('sentinel'):{sentinel:true};
          const trace=[];let fired=false,nested,first,error,threw=false;
          const call=options=>raw?engine.renderRaw('er',model,{},options):engine.render(model,options);
          try{
            first=rendered(call({outputLimit:variant.includes('limit')?0:Infinity,measurer:{width(text,size,style){
              trace.push([text,bits(size),style]);
              if(!fired){
                fired=true;const extra={};extra.self=extra;model.extra=extra;saved[0].name='changed';
                if(variant.includes('audit'))Object.defineProperty(model,'unreadable',{get(){throw new Error('accessor invoked');},configurable:true});
                if(variant==='nested')nested=rendered(call({measurer:{width:(text,size)=>text.length*size*.6}}));
                if(variant.startsWith('throw'))throw sentinel;
              }
              return text.length*size*.6;
            }}}));
          }catch(value){threw=true;error=value;}
          const state={cycle:model.extra?.self===model.extra,alias:model.entities.get('B').attributes===saved,name:saved[0].name};
          delete model.unreadable;
          let repaired;try{repaired=rendered(call({measurer:{width:(text,size)=>text.length*size*.6}}));}finally{engine.dispose(model);}
          row[mode]={first,nested,threw,original:threw&&error===sentinel,error:threw?{name:error?.name,message:error?.message,code:error?.code,type:error?.type}:null,state,trace,repaired};
        }
        rows.push(row);
      }
      return rows;
    }finally{restore();engine.close();}
  });
  await save(testInfo,'er-result-packet-errors.json',report);
  for(const row of report){
    expect(row.packet,`${row.raw}/${row.variant}`).toEqual(row.json);
    expect(row.packet.state).toEqual({cycle:true,alias:true,name:'changed'});
    if(row.variant.startsWith('throw'))expect(row.packet.original).toBe(true);
    if(row.variant==='audit-with-limit')expect(row.packet.error).toMatchObject({name:'TypeError',message:'Accessor properties require an explicit public model codec'});
  }
});

test('ER packet preserves both envelope and combined graph-plus-metadata byte gates',async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const engine=await loadRenderer(),prototype=wasm.HostErDocument.prototype;
    const packetKeys=['render_host_packet','render_raw_host_packet',...(typeof prototype.render_host_frame_packet==='function'?['render_host_frame_packet']:[])],keys=[...packetKeys,'render_host','render_raw_host'];
    const descriptors=new Map(keys.map(key=>[key,Object.getOwnPropertyDescriptor(prototype,key)]));
    const restore=()=>{for(const [key,d]of descriptors)Object.defineProperty(prototype,key,d);};
    const maximum=64*1024*1024,rows=[];let observed;
    try{
      // Normal render tests the serializer ceiling; raw render tests a successful
      // native envelope rejected only when final graph bytes are added. Both
      // use one bounded accessibility string instead of a million-property model.
      for(const raw of [false,true])for(const [mode,legacy]of [['json',true],['packet',false]]){
        restore();
        if(legacy)for(const key of packetKeys)Object.defineProperty(prototype,key,{...descriptors.get(key),value:undefined});
        const normalPacket=packetKeys.includes('render_host_frame_packet')?'render_host_frame_packet':'render_host_packet';
        const key=raw?(legacy?'render_raw_host':'render_raw_host_packet'):(legacy?'render_host':normalPacket);
        Object.defineProperty(prototype,key,{...descriptors.get(key),value:function(...args){
          const value=descriptors.get(key).value.apply(this,args);
          observed=typeof value==='string'?{success:value.startsWith('{"Ok":'),bytes:new TextEncoder().encode(value).length}:{success:true,bytes:value[1]};
          return value;
        }});
        const model=engine.parse('erDiagram\nA'),width=(text,size)=>text.length*size*.6;
        model.accDescr='"';let calls=0,error;
        const draw=()=>{
          const options={maxWidth:10000,outputLimit:Infinity,measurer:{width(...args){calls++;return width(...args);}}};
          return raw?engine.renderRaw('er',model,{},options):engine.render(model,options);
        };
        draw();const first=observed.bytes;model.accDescr='""';draw();const step=observed.bytes-first;
        if(!Number.isSafeInteger(step)||step<2||step>16)throw new Error('Unexpected escaped accessibility JSON growth');
        const count=raw?1+Math.floor((maximum-1-first)/step):2+Math.floor((maximum-first)/step);
        if(count<1||count>12*1024*1024)throw new Error('Invalid bounded accessibility fixture');
        model.accDescr='"'.repeat(count);observed=undefined;calls=0;
        try{draw();}catch(value){error={name:value?.name,message:value?.message,code:value?.code,type:value?.type};}
        const attempted=observed,expectedBytes=first+(count-1)*step;
        model.accDescr='';model.entities.get('A').alias='repaired';
        const repaired=(raw?engine.renderRaw('er',model,{}, {measurer:{width}}):engine.render(model,{measurer:{width}})).svg.length;
        engine.dispose(model);rows.push({raw,mode,calls,error,repaired,attempted,expectedBytes});
      }
      return rows;
    }finally{restore();engine.close();}
  });
  await save(testInfo,'er-result-packet-budget.json',report);
  for(const raw of [false,true]){
    const old=report.find(row=>row.raw===raw&&row.mode==='json'),packet=report.find(row=>row.raw===raw&&row.mode==='packet');
    expect({...packet,mode:'json'}).toEqual(old);
    expect(packet.calls).toBeGreaterThan(0);
    expect(packet.error).toMatchObject({name:'PeleError',message:raw?'Public binary graph exceeds transport budget':'Browser numeric transport exceeds byte budget',code:'limit'});
    expect(packet.repaired).toBeGreaterThan(0);
    if(raw){
      expect(packet.attempted).toEqual({success:true,bytes:packet.expectedBytes});
      expect(packet.attempted.bytes).toBeLessThanOrEqual(64*1024*1024);
      expect(packet.attempted.bytes).toBeGreaterThan(64*1024*1024-17);
    }else {expect(packet.expectedBytes).toBeGreaterThan(64*1024*1024);expect(packet.attempted).toBeUndefined();}
  }
});
