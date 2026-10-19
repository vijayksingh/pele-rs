import {test,expect} from '@playwright/test';

test('schema cache preserves public models, first-key conflicts, fallback and renderer isolation',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer,metricsMeasurer}=await import('/dist/package-web/package/full.js');
    const engine=await loadRenderer(),other=await loadRenderer(),wasm=await import('/dist/package-web/package/pkg/pele.js');
    const prototype=wasm.MutableDocument.prototype,originalSchema=prototype.schema,originalKey=Object.getOwnPropertyDescriptor(prototype,'schema_cache_key');
    let reads=0,epoch=0,conflict=false,budget=false;
    const failures=[],source='flowchart LR\nA[Cache] --> B';
    prototype.schema_cache_key=function(){return 'test-native-static-view:'+this.entry_type()+':'+epoch;};
    prototype.schema=function(){
      reads++;
      if(budget)return JSON.stringify({version:2,ok:false,error:{name:'PeleError',code:'limit',message:'Public result JSON exceeds 64 MiB transport budget',type:null,line:0,column:0,snippet:''}});
      const encoded=originalSchema.call(this);
      if(!conflict)return encoded;
      const changed=JSON.parse(encoded);changed.result.functionShapes=[...changed.result.functionShapes,{method:changed.result.methods[0].id,constructible:true}];return JSON.stringify(changed);
    };
    const parse=(renderer)=>{const model=renderer.parse(source);if(!renderer.render(model,{measurer:metricsMeasurer}).svg.includes('Cache'))failures.push('drawing');return model;};
    try {
      const a=parse(engine),b=parse(engine);if(reads!==1)failures.push('warm schema read');
      if(Object.getPrototypeOf(a)!==Object.getPrototypeOf(b)||a.addVertex!==b.addVertex)failures.push('registration identity');
      const c=parse(other);if(reads!==2)failures.push('renderer isolation');
      epoch=1;conflict=true;
      try{engine.parse(source);failures.push('conflict accepted');}catch(e){if(!(e instanceof TypeError)||e.message!=='Conflicting native model schema identity')failures.push('conflict error');}
      epoch=2;conflict=false;budget=true;
      try{engine.parse(source);failures.push('budget accepted');}catch(e){if(e.code!=='limit'||e.message!=='Public result JSON exceeds 64 MiB transport budget')failures.push('budget error');}
      epoch=0;budget=false;const d=parse(engine);if(reads!==4)failures.push('recovery cache');
      prototype.schema_cache_key=undefined;const e=parse(engine),f=parse(engine);if(reads!==6)failures.push('missing capability fallback');
      for(const model of [a,b,d,e,f])engine.dispose(model);other.dispose(c);
      return{failures,reads};
    }finally{prototype.schema=originalSchema;if(originalKey)Object.defineProperty(prototype,'schema_cache_key',originalKey);else delete prototype.schema_cache_key;engine.close();other.close();}
  });
  expect(report).toEqual({failures:[],reads:6});
});

test('caller-authored raw models always validate native metadata despite a primed cache',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer,metricsMeasurer}=await import('/dist/package-web/package/full.js'),engine=await loadRenderer();
    const wasm=await import('/dist/package-web/package/pkg/pele.js'),prototype=wasm.MutableDocument.prototype,originalSchema=prototype.schema,originalKey=Object.getOwnPropertyDescriptor(prototype,'schema_cache_key');
    let reads=0,keys=0;prototype.schema_cache_key=function(){keys++;return 'test-pie-schema';};prototype.schema=function(){reads++;return originalSchema.call(this);};
    const models=[];
    try{
      const parsed=engine.parse('pie\n"Primed":1');models.push(parsed);
      const drawings=[];
      for(const label of ['First raw','Second raw']){const model={type:'pie',sections:new Map([[label,3]]),showData:true};models.push(model);drawings.push(engine.renderRaw('pie',model,{}, {measurer:metricsMeasurer}).svg.includes(label));}
      return{reads,keys,drawings};
    }finally{for(const model of models)engine.dispose(model);prototype.schema=originalSchema;if(originalKey)Object.defineProperty(prototype,'schema_cache_key',originalKey);else delete prototype.schema_cache_key;engine.close();}
  });
  expect(report).toEqual({reads:3,keys:1,drawings:[true,true]});
});

test('native static schema identity stays stable across documents when the additive WASM getter is available',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');await loadRenderer();
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const handles=['flowchart LR\nA-->B','flowchart TD\nC-->D','sequenceDiagram\nA->>B: Hello'].map(source=>{const parsed=wasm.parse_v2(source,'{}');try{return parsed.to_public();}finally{parsed.free();}});
    try{
      const available=typeof handles[0].schema_cache_key==='function';if(!available)return{available};
      const keys=handles.map(h=>h.schema_cache_key());return{available,stable:keys[0]===keys[1],distinct:keys[0]!==keys[2],opaque:keys.every(k=>/^1:[0-9a-f]+:[0-9a-f]+:\d+:[0-9a-f]+:\d+$/.test(k))};
    }finally{for(const handle of handles)handle.free();}
  });
  await testInfo.attach('native-schema-key-status.json',{body:JSON.stringify(report),contentType:'application/json'});
  expect(report).toEqual({available:true,stable:true,distinct:true,opaque:true});
});
