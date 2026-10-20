import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('ER record publication preserves complete graphs, prepared entry, fallback and retained edits',async({page},testInfo)=>{
  test.setTimeout(120000);
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js');
    const entries=await import('/dist/package-web/package/diagram-entries.js');await entries.loadDiagramEntries();
    const nativeEntry=(await import('/dist/package-web/package/diagrams/er.js')).default;
    const oracleEntry=(await import('/dist/evidence/pele-esm/diagrams/er.js')).default;
    const corpus=await(await fetch('/fixtures/pele/er.json')).json();
    const wanted=['upstream/4','upstream/10','upstream/106','upstream/110','upstream/111','upstream/135','upstream/137','upstream/146'];
    const fixtures=wanted.map(id=>{const value=corpus.cases.find(c=>c.id===id);if(!value?.expected.model)throw new Error('Missing frozen model: '+id);return value;});
    const engine=await loadRenderer(),view=new DataView(new ArrayBuffer(8));
    const bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const rawNaN=()=>{view.setBigUint64(0,0xfff8000000000042n);return view.getFloat64(0);};
    const signature=roots=>{
      const ids=new Map(),queue=[],nodes=[];
      const atom=value=>{
        if(value===undefined)return ['undefined'];
        if(value===null)return ['null'];
        if(typeof value==='number')return ['number',bits(value)];
        if(typeof value==='function')return ['function',value.length];
        if(typeof value!=='object')return [typeof value,value];
        if(!ids.has(value)){ids.set(value,queue.length);queue.push(value);}
        return ['ref',ids.get(value)];
      };
      const root=roots.map(atom);
      for(let i=0;i<queue.length;i++){
        const value=queue[i],kind=Array.isArray(value)?'array':value instanceof Map?'map':value instanceof Set?'set':'object';
        const properties=Object.getOwnPropertyNames(value).map(key=>{
          const d=Object.getOwnPropertyDescriptor(value,key);
          return[key,d.enumerable,d.configurable,Object.hasOwn(d,'value')?['data',d.writable,atom(d.value)]:['accessor',atom(d.get),atom(d.set)]];
        });
        nodes.push({kind,properties,...(kind==='map'?{entries:[...value].map(([k,v])=>[atom(k),atom(v)])}:kind==='set'?{values:[...value].map(atom)}:{})});
      }
      return JSON.stringify({root,nodes});
    };
    const outcome=run=>{try{const value=run();return{ok:true,svg:value.svg,width:bits(value.width),height:bits(value.height)};}catch(e){return{ok:false,name:e?.name,message:e?.message,code:e?.code,type:e?.type,line:e?.line,column:e?.column,snippet:e?.snippet};}};
    const rows=[];
    try{
      for(const fixture of fixtures)for(const lane of ['ordinary','prepared']){
        const row={id:fixture.id,lane};
        for(const runtime of ['oracle','native']){
          const native=runtime==='native',config={},options={measurer:{width:(text,size)=>text.length*size*.6}};
          const api=lane==='ordinary'?{parse:()=>native?engine.parse(fixture.source):oracle.parse(fixture.source),render:model=>native?engine.render(model,options):oracle.render(model,options),dispose:model=>{if(native)engine.dispose(model);}}:{parse:()=>native?nativeEntry.parse(fixture.source,config,'Record layout',0,{}):oracleEntry.parse(fixture.source,config,'Record layout',0,{}),render:model=>native?nativeEntry.render(model,config,options):oracleEntry.render(model,config,options),dispose:model=>{if(native)entries.disposeDiagramModel(model);}};
          let model;
          try{
            model=api.parse();
            const before=signature([model]),[key,entity]=model.entities.entries().next().value;
            const attributes=entity.attributes,saved=attributes[0];
            const returned=model.addEntity(key);
            const extra={nan:null,zero:null};extra.nan=rawNaN();extra.zero=-0;extra.self=extra;
            Object.defineProperty(entity,'publicationExtra',{value:extra,enumerable:false,writable:false,configurable:true});
            const beforeBits=bits(extra.nan),first=outcome(()=>api.render(model));
            if(saved)saved.comment='Edited after publication';
            const next=outcome(()=>api.render(model));
            row[runtime]={before,first,next,after:signature([model,entity,attributes,saved,returned]),identity:model.entities.get(key)===entity&&entity.attributes===attributes&&(!saved||attributes[0]===saved),nanStable:bits(extra.nan)===beforeBits,negativeZero:Object.is(extra.zero,-0)};
          }finally{if(model)api.dispose(model);}
        }
        rows.push(row);
      }
    }finally{engine.close();}
    return rows;
  });
  const path=testInfo.outputPath('er-record-layout.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('er-record-layout.json',{path,contentType:'application/json'});
  expect(report).toHaveLength(16);
  for(const row of report){expect(row.native,`${row.id}/${row.lane}`).toEqual(row.oracle);expect(row.native.identity).toBe(true);expect(row.native.nanStable).toBe(true);expect(row.native.negativeZero).toBe(true);}
});
