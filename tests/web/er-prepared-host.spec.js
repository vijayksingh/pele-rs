import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

async function saveReport(testInfo,name,value){
  const path=testInfo.outputPath(name);
  await writeFile(path,JSON.stringify(value,null,2));
  await testInfo.attach(name,{path,contentType:'application/json'});
}

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
  await saveReport(testInfo,'er-prepared-live-baseline.json',rows);
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

test('prepared ER retains raw policy, detached references and independent captured providers',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const entries=await import('/dist/package-web/package/diagram-entries.js');await entries.loadDiagramEntries();
    const nativeEntry=(await import('/dist/package-web/package/diagrams/er.js')).default;
    const oracleEntry=(await import('/dist/evidence/pele-esm/diagrams/er.js')).default;
    const engine=await loadRenderer(),source='erDiagram\nA { int id PK }\nB { string value FK }\nA ||--o{ B : owns\nclassDef marked fill:red\nclass B marked';
    const config={er:{nodeSpacing:71}},width=(text,size)=>text.length*size*.6,rows=[],checks=[];
    const shape=model=>({title:model.title,accTitle:model.accTitle,entities:[...model.entities].map(([key,e])=>[key,e.alias,e.attributes.map(a=>({...a,keys:[...a.keys]})),[...e.cssStyles]]),extension:model.extension?.self===model.extension});
    for(const lane of ['entry','renderer']){
      const row={lane};
      for(const implementation of ['oracle','native']){
        const oracle=implementation==='oracle';
        const parse=()=>oracle?oracleEntry.parse(source,config,'Raw title',2,{maxEdges:null}):lane==='entry'?nativeEntry.parse(source,config,'Raw title',2,{maxEdges:null}):engine.parsePrepared('er',source,config,'Raw title',2,null);
        const render=(m,o)=>oracle?oracleEntry.render(m,config,o):lane==='entry'?nativeEntry.render(m,config,o):engine.renderRaw('er',m,config,o);
        const model=parse(),entity=model.entities.get('B'),oldAttrs=entity.attributes,oldStyles=entity.cssStyles;
        try{
          let fired=false;const trace=[];
          const first=render(model,{limit:-1,maxEdges:-1,outputLimit:0,nativeTheme:{ignored:true},measurer:{width(...args){
            trace.push(args);
            if(!fired){fired=true;entity.attributes=[{type:'int',name:'replacement',keys:[],comment:''}];entity.cssStyles=['fill:green'];oldAttrs[0].name='old detached mutation';oldStyles.push('stroke:blue');model.title='Late raw title';model.accTitle='Late accessible title';const cycle={};cycle.self=cycle;model.extension=cycle;}
            return width(...args);
          }}});
          row[implementation]={fired,first,next:render(model,{measurer:{width}}),trace,state:shape(model),detached:{attributes:oldAttrs.map(a=>({...a,keys:[...a.keys]})),styles:[...oldStyles],entitySame:model.entities.get('B')===entity,replacedAttrs:entity.attributes!==oldAttrs,replacedStyles:entity.cssStyles!==oldStyles}};
        }finally{if(!oracle){if(lane==='entry')entries.disposeDiagramModel(model);else engine.dispose(model);}}
      }
      rows.push(row);
    }
    const prepared=engine.parsePrepared('er',source,config,'Captured prepared',5,undefined);
    const second=engine.parsePrepared('er','erDiagram\nX',{},undefined,0,undefined),helper=prepared.addEntity;
    checks.push(['shared helper/prototype',helper===second.addEntity&&Object.getPrototypeOf(prepared)===Object.getPrototypeOf(second)]);
    checks.push(['borrowed helper',helper.call(second,'Y')===second.entities.get('Y')]);
    const retained=engine.render(prepared,{measurer:{width}});
    let foreignParses=0,foreignRenders=0;
    const provider={type:'er',parse(){foreignParses++;return {foreign:true};},render(){foreignRenders++;return {svg:'<svg data-provider="foreign"/>',width:1,height:2,links:[]};}};
    engine.register(provider);
    const raw=engine.renderRaw('er',prepared,config,{measurer:{width}});
    const reference=oracleEntry.parse(source,config,'Captured prepared',5,{maxEdges:undefined});
    checks.push(['raw oracle after replacement',JSON.stringify(raw)===JSON.stringify(oracleEntry.render(reference,config,{measurer:{width}}))]);
    checks.push(['normal prepared rendering has no foreign provider',JSON.stringify(engine.render(prepared,{measurer:{width}}))===JSON.stringify(retained)&&foreignRenders===0]);
    const freshPrepared=engine.parsePrepared('er','erDiagram\nQ',{},undefined,0,undefined);
    checks.push(['new prepared stays builtin',freshPrepared.entities.has('Q')&&foreignParses===0]);
    const custom=engine.parse('erDiagram\nZ');engine.render(custom);
    checks.push(['ordinary provider unchanged',foreignParses===1&&foreignRenders===1]);
    engine.dispose(custom);engine.dispose(freshPrepared);
    const saved=prepared.entities;
    engine.close();
    checks.push(['prepared survives close',JSON.stringify(engine.render(prepared,{measurer:{width}}))===JSON.stringify(retained)]);
    let closed=false;try{engine.parsePrepared('er','erDiagram\nA',{},undefined,0,undefined);}catch(error){closed=error.message==='Registry is closed';}checks.push(['new prepared closed',closed]);
    engine.dispose(prepared);
    checks.push(['disposed fields remain',prepared.entities===saved]);
    checks.push(['saved helper survives with live receiver',helper.call(second,'After')===second.entities.get('After')]);
    let disposed=false;try{engine.renderRaw('er',prepared,{});}catch(error){disposed=error instanceof TypeError&&error.message.includes('disposed');}checks.push(['raw disposed',disposed]);
    engine.dispose(second);
    return {rows,checks};
  });
  await saveReport(testInfo,'er-prepared-policy-controls.json',report);
  for(const row of report.rows){expect(row.native.fired).toBe(true);expect(row.native).toEqual(row.oracle);}
  expect(report.checks.filter(([,ok])=>!ok)).toEqual([]);
});

test('prepared host capability preserves low-level decoding errors and low-level arena APIs',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');const engine=await loadRenderer();
    const wasm=await import('/dist/package-web/package/pkg/pele.js'),registry=new wasm.NativeRegistry(true),rows=[];
    let factories=0,captures=0;
    const make=()=>{factories++;throw new Error('invalid prepared inputs reached host factory');};
    const capture=()=>{captures++;};
    const readError=run=>{try{const handle=run();handle?.free?.();return {unexpectedSuccess:true};}catch(error){if(typeof error==='string'){try{return JSON.parse(error);}catch{}}return {name:error?.name,message:error?.message};}};
    const base=['er','erDiagram\nA','{}',undefined,0,0,0];
    const cases=[
      ['negative offset',{4:-1}],['fraction offset',{4:.5}],['nan offset',{4:NaN}],['infinite offset',{4:Infinity}],
      ['position before config',{2:'{',4:-1}],['invalid config',{2:'{'}],['config before edge',{2:'{',5:99}],
      ['large config',{2:JSON.stringify({large:'x'.repeat(1024*1024)})}],['edge before type',{0:'not-a-type',5:99}],
      ['unknown type',{0:'not-a-type'}],['syntax',{1:'erDiagram\nA { int }'}],
      ['verbatim frontmatter',{1:'---\ntitle: do not preprocess\n---\nerDiagram\nA'}],
    ];
    for(const [name,edits]of cases){const args=[...base];for(const [key,value]of Object.entries(edits))args[+key]=value;
      rows.push({name,legacy:readError(()=>registry.parse_prepared(...args)),host:readError(()=>registry.parse_prepared_with_host_er(...args,make,capture))});
    }
    const handle=registry.parse_prepared(...base);let lowLevel;
    try{const first=Array.from(handle.snapshot_binary()),second=Array.from(handle.snapshot_binary());lowLevel={constructor:handle instanceof wasm.MutableDocument,entry:handle.entry_type(),same:JSON.stringify(first)===JSON.stringify(second),schema:JSON.parse(handle.schema()).result.id};}finally{handle.free();}
    registry.close();rows.push({name:'closed before position',legacy:readError(()=>registry.parse_prepared('er','erDiagram\nA','{',undefined,-1,99,0)),host:readError(()=>registry.parse_prepared_with_host_er('er','erDiagram\nA','{',undefined,-1,99,0,make,capture))});
    registry.free();engine.close();return {rows,factories,captures,lowLevel};
  });
  await saveReport(testInfo,'er-prepared-error-controls.json',report);
  expect(report.factories).toBe(0);expect(report.captures).toBe(0);
  for(const row of report.rows){expect(row.legacy.unexpectedSuccess,row.name).toBeUndefined();expect(row.host,row.name).toEqual(row.legacy);}
  expect(report.lowLevel.constructor).toBe(true);expect(report.lowLevel.entry).toBe('er');expect(report.lowLevel.same).toBe(true);
});

test('normal render of prepared models keeps arena config and wrapper policy',async({page},testInfo)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const host=await loadRenderer(),arena=await loadRenderer();
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const oracleCore=await import('/dist/evidence/pele-esm/core.js');
    const oracleEntry=(await import('/dist/evidence/pele-esm/diagrams/er.js')).default;
    oracleCore.register(oracleEntry);
    const source='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns';
    const width=(text,size)=>text.length*size*.6,models=[],rows=[],checks=[];
    const bits=value=>{const view=new DataView(new ArrayBuffer(8));view.setFloat64(0,value);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const outcome=run=>{try{const value=run();return {ok:true,value:{...value,width:bits(value.width),height:bits(value.height)}};}catch(error){return {ok:false,name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet};}};
    const oldPrepared=(...args)=>{
      // Public capability fallback chooses the unchanged parse_prepared API;
      // render below remains renderer.render(model), never entry.render.
      const prototype=wasm.NativeRegistry.prototype,key='parse_prepared_with_host_er';
      const descriptor=Object.getOwnPropertyDescriptor(prototype,key);
      if(!descriptor)throw new Error('Prepared host capability missing');
      Object.defineProperty(prototype,key,{...descriptor,value:undefined});
      try{return arena.parsePrepared(...args);}finally{Object.defineProperty(prototype,key,descriptor);}
    };
    try{
      for(const fixed of [true,false]){
        const config={er:{useMaxWidth:!fixed,nodeSpacing:71,rankSpacing:83}},title='Prepared retained title';
        const a=host.parsePrepared('er',source,config,title,11,undefined),b=oldPrepared('er',source,config,title,11,undefined);
        models.push([host,a],[arena,b]);
        const cases=[['default',{}],['responsive true',{responsive:true}],['responsive false',{responsive:false}],['max width',{maxWidth:320}],['passive parse options',{limit:-1,maxEdges:-1,config:{er:{useMaxWidth:fixed,nodeSpacing:999}}}],['output limit',{outputLimit:0}]];
        for(const [name,options]of cases){
          const draw=(engine,model)=>{const trace=[];const result=outcome(()=>engine.render(model,{...options,measurer:{width(...args){trace.push(args);return width(...args);}}}));return {result,trace};};
          rows.push({name,fixed,host:draw(host,a),arena:draw(arena,b)});
        }
        const beforeA=outcome(()=>host.render(a,{measurer:{width}})),beforeB=outcome(()=>arena.render(b,{measurer:{width}}));
        config.er.nodeSpacing=999;config.er.useMaxWidth=fixed;a.type='caller-edited-type';b.type='caller-edited-type';
        rows.push({name:'captured config and type',fixed,host:outcome(()=>host.render(a,{measurer:{width}})),arena:outcome(()=>arena.render(b,{measurer:{width}}))});
        checks.push(['host retains config/type '+fixed,JSON.stringify(beforeA)===JSON.stringify(rows.at(-1).host)]);
        checks.push(['arena retains config/type '+fixed,JSON.stringify(beforeB)===JSON.stringify(rows.at(-1).arena)]);
      }
      // Pinned core rejects standalone prepared roots because parse ownership
      // is separate. Assert that contract rather than substituting raw drawing.
      const oraclePrepared=oracleEntry.parse(source,{},'Standalone',0,{maxEdges:undefined});
      const unsupported=outcome(()=>oracleCore.render(oraclePrepared,{measurer:{width}}));
      checks.push(['pinned core rejects standalone root',!unsupported.ok&&unsupported.name==='TypeError'&&unsupported.message==='render() takes diagram text or a model that parse() returned.']);
      // Actual supported core ownership control uses core.parse on each side.
      const ordinaryConfig={er:{useMaxWidth:false,nodeSpacing:71}};
      const nativeOwned=host.parse(source,{config:ordinaryConfig}),oracleOwned=oracleCore.parse(source,{config:ordinaryConfig});models.push([host,nativeOwned]);
      const coreControl={native:outcome(()=>host.render(nativeOwned,{measurer:{width}})),oracle:outcome(()=>oracleCore.render(oracleOwned,{measurer:{width}}))};
      let providerCalls=0;const provider={type:'er',parse(){return {};},render(){providerCalls++;throw new Error('prepared model retargeted');}};
      host.register(provider);arena.register(provider);host.close();arena.close();
      for(let i=0;i<4;i+=2){const a=models[i][1],b=models[i+1][1];rows.push({name:'replacement and close',fixed:i===0,host:outcome(()=>host.render(a,{measurer:{width}})),arena:outcome(()=>arena.render(b,{measurer:{width}}))});}
      checks.push(['no prepared provider retarget',providerCalls===0]);
      return {rows,checks,unsupported,coreControl};
    }finally{for(const [engine,model]of models)engine.dispose(model);host.close();arena.close();}
  });
  await saveReport(testInfo,'er-prepared-normal-render-controls.json',report);
  for(const row of report.rows)expect(row.host,`${row.name}/${row.fixed}`).toEqual(row.arena);
  expect(report.checks.filter(([,ok])=>!ok)).toEqual([]);
  expect(report.coreControl.native).toEqual(report.coreControl.oracle);
});
