import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

// Public feature differential. Intentionally fails on a remaining arena/live
// callback mismatch; the attached rows preserve baseline evidence on failure.
test('prepared ER entry callbacks preserve live rows, original throws and nested raw rendering',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const rows=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const entries=await import('/dist/package-web/package/diagram-entries.js');
    await entries.loadDiagramEntries();
    const nativeEntry=(await import('/dist/package-web/package/diagrams/er.js')).default;
    const oracleEntry=(await import('/dist/evidence/pele-esm/diagrams/er.js')).default;
    const engine=await loadRenderer();
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns\n';
    const config={},title='Prepared title',lineOffset=7,limits={maxEdges:undefined};
    const width=(text,size)=>text.length*size*.6;
    const numberBits=value=>{
      const bytes=new DataView(new ArrayBuffer(8));bytes.setFloat64(0,value);
      return bytes.getBigUint64(0).toString(16).padStart(16,'0');
    };
    const signature=value=>{
      const ids=new Map(),pending=[],nodes=[];
      const atom=v=>{
        if(v===undefined)return ['undefined'];
        if(typeof v==='number')return ['number',numberBits(v)];
        if(typeof v==='function')return ['function',v.name,v.length];
        if(typeof v==='bigint')return ['bigint',String(v)];
        if(typeof v==='symbol')return ['symbol',String(v)];
        if(v===null||typeof v!=='object')return v;
        if(!ids.has(v)){ids.set(v,pending.length);pending.push(v);}
        return ['ref',ids.get(v)];
      };
      const root=atom(value);
      for(let i=0;i<pending.length;i++){
        const v=pending[i],kind=Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':'object';
        const node={kind,properties:Object.getOwnPropertyNames(v).map(key=>{
          const d=Object.getOwnPropertyDescriptor(v,key);
          return[key,d.enumerable,!!d.writable,d.configurable,'value'in d?atom(d.value):['accessor',atom(d.get),atom(d.set)]];
        })};
        if(kind==='map')node.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);
        if(kind==='set')node.values=[...v].map(atom);
        nodes.push(node);
      }
      return JSON.stringify({root,nodes});
    };
    const rows=[];
    try{
      for(const lane of ['standalone-prepared','renderer-prepared'])for(const variant of ['live-row','original-throw','nested-raw']){
        const row={lane,variant};
        for(const implementation of ['oracle','native']){
          const oracle=implementation==='oracle';
          const api=oracle?{
            parse:()=>oracleEntry.parse(source,config,title,lineOffset,limits),
            render:(model,options)=>oracleEntry.render(model,config,options),
            dispose:()=>{},
          }:lane==='standalone-prepared'?{
            parse:()=>nativeEntry.parse(source,config,title,lineOffset,limits),
            render:(model,options)=>nativeEntry.render(model,config,options),
            dispose:model=>entries.disposeDiagramModel(model),
          }:{
            parse:()=>engine.parsePrepared('er',source,config,title,lineOffset,limits.maxEdges),
            render:(model,options)=>engine.renderRaw('er',model,config,options),
            dispose:model=>engine.dispose(model),
          };
          let model;
          try{
            model=api.parse();
            const entity=model.entities.get('B'),attributes=entity.attributes,savedRow=attributes[0];
            const sentinel={marker:'prepared-original'},cycle={};cycle.self=cycle;
            let fired=false,nested=null;const trace=[];
            const outcome=run=>{
              try{const result=run();return {ok:true,svg:result.svg,widthBits:numberBits(result.width),heightBits:numberBits(result.height)};}
              catch(error){return {ok:false,original:error===sentinel,kind:typeof error,name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet};}
            };
            const first=outcome(()=>api.render(model,{measurer:{width(...args){
              trace.push(['outer',signature(args)]);
              if(!fired){
                fired=true;savedRow.name='Callback changed identifier';savedRow.comment='Callback changed comment';model.preparedCallbackExtension=cycle;
                if(variant==='nested-raw')nested=outcome(()=>api.render(model,{measurer:{width(...inner){trace.push(['nested',signature(inner)]);return width(...inner);}}}));
                if(variant==='original-throw')throw sentinel;
              }
              return width(...args);
            }}}));
            const after=signature({model,entity,attributes,savedRow});
            const identity=model.entities.get('B')===entity&&entity.attributes===attributes&&attributes[0]===savedRow;
            const next=outcome(()=>api.render(model,{measurer:{width}}));
            row[implementation]={fired,first,nested,trace,after,next,afterNext:signature({model,entity,attributes,savedRow}),identity};
          }catch(error){row[implementation]={setupFailure:true,name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet};}
          finally{if(model)api.dispose(model);}
        }
        const keys=new Set([...Object.keys(row.oracle),...Object.keys(row.native)]);
        row.mismatches=[...keys].filter(key=>JSON.stringify(row.oracle[key])!==JSON.stringify(row.native[key]));
        rows.push(row);
      }
    }finally{engine.close();}
    return rows;
  });
  const reportPath=testInfo.outputPath('er-prepared-live-baseline.json');
  await writeFile(reportPath,JSON.stringify(rows,null,2)+'\n');
  await testInfo.attach('er-prepared-live-baseline.json',{path:reportPath,contentType:'application/json'});
  expect(rows).toHaveLength(6);
  for(const row of rows){
    expect(row.oracle.setupFailure,`${row.lane}/${row.variant}: oracle setup`).toBeUndefined();
    expect(row.native.setupFailure,`${row.lane}/${row.variant}: native setup`).toBeUndefined();
    expect(row.oracle.fired,`${row.lane}/${row.variant}: oracle callback`).toBe(true);
    expect(row.native.fired,`${row.lane}/${row.variant}: native callback`).toBe(true);
    expect(row.oracle.identity,`${row.lane}/${row.variant}: oracle identity`).toBe(true);
    expect(row.native.identity,`${row.lane}/${row.variant}: native identity`).toBe(true);
    if(row.variant==='original-throw'){
      expect(row.oracle.first.original).toBe(true);
      expect(row.native.first.original).toBe(true);
    }
    if(row.variant==='nested-raw')expect(row.oracle.nested.ok).toBe(true);
  }
  expect(rows.filter(row=>row.mismatches.length).map(({lane,variant,mismatches})=>({lane,variant,mismatches}))).toEqual([]);
});
