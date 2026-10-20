import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('ER entry frame preserves admission, live callbacks, exact data and optional scratch fallback',async({page},testInfo)=>{
  test.setTimeout(180000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const engine=await loadRenderer(),prototype=wasm.HostErDocument.prototype,key='render_host_frame_packet';
    const descriptor=Object.getOwnPropertyDescriptor(prototype,key);
    if(!descriptor)throw new Error('Entry frame capability missing');
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
    const data=new DataView(new ArrayBuffer(8));
    const bits=value=>{data.setFloat64(0,value);return data.getBigUint64(0).toString(16).padStart(16,'0');};
    const fromBits=hex=>{data.setBigUint64(0,BigInt('0x'+hex));return data.getFloat64(0);};
    const normalize=value=>({...value,width:bits(value.width),height:bits(value.height)});
    const rows=[];
    const variants=['plain','ordinary-kinds','readonly-length','hole','invalid-used-string','invalid-unused-string','accessor','scratch-fallback','future-row','same-row','nested','throw','keys-coercion'];
    try{
      for(const variant of variants){
        const row={variant};
        for(const [mode,enabled]of [['live',false],['frame',true]]){
          let offered=[];
          Object.defineProperty(prototype,key,{...descriptor,value:enabled?function(...args){offered.push(args.at(-1)?.byteLength??0);return descriptor.value.apply(this,args);}:undefined});
          const model=engine.parse(source),attributes=model.entities.get('B').attributes,first=model.entities.get('A').attributes[0];
          const special={negativeZero:null,nan:null};special.negativeZero=-0;special.nan=fromBits('fff8000000000042');
          const beforeNaN=bits(special.nan);
          const extra={self:null,date:new Date(-0),map:new Map(),set:new Set(),array:[]};extra.self=extra;
          extra.map.set(extra,special);extra.set.add(extra);extra.array.length=7;extra.array[5]=extra;
          if(variant==='ordinary-kinds')model.extra=extra;
          if(variant==='readonly-length')Object.defineProperty(attributes,'length',{writable:false});
          if(variant==='hole')attributes.length=3;
          if(variant==='invalid-used-string')attributes[0].name='\ud800';
          if(variant==='invalid-unused-string')model.extra='\ud800';
          if(variant==='accessor')Object.defineProperty(model,'extra',{get(){throw new Error('should not invoke');},configurable:true});
          if(variant==='scratch-fallback')model.extra='x'.repeat(4*1024*1024+1);
          const trace=[];let fired=false,nested,result,error,threw=false,sentinel={sentinel:true},coercions=0;
          const state=()=>({same:attributes===model.entities.get('B').attributes,name:attributes[0].name,firstName:first.name,negativeZero:bits(special.negativeZero),nan:bits(special.nan),cycle:extra.self===extra,map:extra.map.get(extra)===special,set:extra.set.has(extra),hole:!(0 in extra.array),readonly:Object.getOwnPropertyDescriptor(attributes,'length').writable===false});
          try{
            result=normalize(engine.render(model,{measurer:{width(text,size,style){
              trace.push([text,bits(size),style]);
              if(!fired){
                fired=true;
                if(variant==='future-row'||variant==='nested')attributes[0].name='mutated';
                if(variant==='same-row')first.name='same-row-changed';
                if(variant==='keys-coercion')first.keys=[{toString(){coercions++;attributes[0].name='coerced';return 'PK';}}];
                if(variant==='nested')nested=normalize(engine.render(model,{measurer:{width:(text,size)=>text.length*size*.6}}));
                if(variant==='throw'){attributes[0].name='before-throw';throw sentinel;}
              }
              return text.length*size*.6;
            }}}));
          }catch(value){threw=true;error=value;}
          const after=state();
          if(variant==='accessor')delete model.extra;
          if(variant==='hole')attributes.length=1;
          if(variant==='invalid-used-string')attributes[0].name='repaired';
          if(variant==='keys-coercion')first.keys=['PK'];
          let repaired;try{repaired=normalize(engine.render(model,{measurer:{width:(text,size)=>text.length*size*.6}}));}finally{engine.dispose(model);}
          row[mode]={beforeNaN,result,nested,threw,error:threw?{name:error?.name,message:error?.message,code:error?.code,type:error?.type}:null,original:threw&&error===sentinel,trace,after,coercions,repaired};
          if(enabled)row.offered=offered;
        }
        rows.push(row);
      }
      // Source, methods and raw entry stay on their existing paths. Raw config
      // serialization may mutate data after entry audit and cannot use a frame.
      let calls=0;
      Object.defineProperty(prototype,key,{...descriptor,value:function(...args){calls++;return descriptor.value.apply(this,args);}});
      const model=engine.parse(source);model.addEntity('A');
      engine.render(source,{measurer:{width:(text,size)=>text.length*size*.6}});
      const config={toJSON(){model.entities.get('B').attributes[0].name='config-written';return {};}};
      const raw=engine.renderRaw('er',model,config,{measurer:{width:(text,size)=>text.length*size*.6}});
      const controls={calls,configWritten:raw.svg.includes('config-written')};engine.dispose(model);
      return {rows,controls};
    }finally{Object.defineProperty(prototype,key,descriptor);engine.close();}
  });
  const path=testInfo.outputPath('er-entry-read-frame.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('er-entry-read-frame.json',{path,contentType:'application/json'});
  for(const row of report.rows){
    expect(row.frame,row.variant).toEqual(row.live);
    const inputNaN=BigInt('0x'+row.frame.beforeNaN);
    expect((inputNaN&0x7ff0000000000000n)===0x7ff0000000000000n&&(inputNaN&0x000fffffffffffffn)!==0n).toBe(true);
    expect(row.frame.after.nan).toBe(row.frame.beforeNaN);
    if(row.variant==='scratch-fallback')expect(row.offered.every(value=>value===0)).toBe(true);
    else if(row.variant!=='accessor')expect(row.offered.some(value=>value>0)).toBe(true);
    if(row.variant==='nested'){expect(row.offered[0]).toBeGreaterThan(0);expect(row.offered[1]).toBe(0);}
    if(row.variant==='throw')expect(row.frame.original).toBe(true);
  }
  expect(report.controls).toEqual({calls:0,configWritten:true});
});
