import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

// Construction behavior/ownership only: actual names and non-new error messages
// are diagnostics. Shared realm identities and minified names remain open.
test('seven captured classes preserve defaults, first helpers, aliases and raw outcomes',async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const fixture=await(await fetch('/fixtures/public-direct-constructors.json')).json();
    const corpus=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const view=new DataView(new ArrayBuffer(8)),bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    const error=e=>e?.name==='PeleError'?{name:e.name,message:e.message,code:e.code,type:e.type??null,line:e.line??0,column:e.column??0,snippet:e.snippet??''}:{name:e?.name,message:e?.message};
    const caught=run=>{try{run();return null;}catch(e){return error(e);}};
    const arg=v=>v&&typeof v==='object'?v.array?v.array.map(arg):Object.fromEntries(v.object.map(([k,x])=>[k,arg(x)])):v;
    function graph(root,prototype,result){
      const nodes=[],ids=new Map();
      function atom(v){
        if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];
        if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');
        const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===prototype?'instance':'object',properties:[]};nodes.push(n);
        if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);
        for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}
        return['ref',id];
      }
      const rootAtom=atom(root),returned=atom(result);return{root:rootAtom,nodes,result:returned};
    }
    const rows=[],metadata=[];
    try{
      for(const c of fixture.cases){
        const source=corpus.cases.find(x=>x.family===c.family),raw=(await import('/dist/package-web/package/diagrams/'+c.family+'.js')).default,refRaw=(await import('/dist/evidence/pele-esm/diagrams/'+c.family+'.js')).default;
        const parsed=engine.parse(source.source),prepared=raw.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});
        const refParsed=oracle.parse(source.source),refPrepared=refRaw.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});
        const classes={candidate:[parsed.constructor,prepared.constructor],oracle:[refParsed.constructor,refPrepared.constructor]};engine.dispose(parsed);disposeDiagramModel(prepared);
        for(const lane of ['oracle','candidate'])for(const [origin,C]of classes[lane].entries()){
          const flags=(o,k)=>{const d=Object.getOwnPropertyDescriptor(o,k);return{writable:d.writable,enumerable:d.enumerable,configurable:d.configurable};};
          metadata.push({family:c.family,lane,origin:origin?'prepared':'core',name:C.name,withoutNew:caught(()=>C()),arity:C.length,ownKeys:Reflect.ownKeys(C),length:flags(C,'length'),nameFlags:flags(C,'name'),prototype:flags(C,'prototype'),backlink:flags(C.prototype,'constructor'),backlinkIdentity:C.prototype.constructor===C});
        }
        for(const mode of ['plain','helperFirstSubclass','alternate']){
          const row={family:c.family,mode,fixture:c};
          for(const lane of ['oracle','candidate']){
            const C=classes[lane][mode==='helperFirstSubclass'?1:0],entry=lane==='candidate'?raw:refRaw;
            let reads=0,writes=0;const ignored=new Proxy({}, {get(){reads++;throw Error('Ignored getter');},ownKeys(){reads++;throw Error('Ignored keys');},getPrototypeOf(){reads++;throw Error('Ignored prototype');}});
            let model,prototype;
            if(mode==='helperFirstSubclass'){class Derived extends C{}model=new Derived(ignored,Symbol('ignored'));prototype=Derived.prototype;}
            else if(mode==='alternate'){function Alternate(){}Alternate.prototype=Object.create(C.prototype);for(const [key]of c.initial.graph.nodes[0].properties)Object.defineProperty(Alternate.prototype,key,{set(){writes++;throw Error('Inherited field setter');},configurable:true});model=Reflect.construct(C,[ignored,Symbol('ignored')],Alternate);prototype=Alternate.prototype;}
            else{model=new C(ignored,Symbol('ignored'));prototype=C.prototype;}
            try{
              const trace=[],resources={now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}};
              const rawRender=()=>{trace.length=0;try{const r=entry.render(model,{},resources);return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height),trace:trace.slice()};}catch(e){return{ok:false,error:error(e),trace:trace.slice()};}};
              const core=()=>caught(()=>lane==='candidate'?engine.render(model,resources):oracle.render(model,resources));
              const initial={graph:graph(model,prototype),...(mode==='helperFirstSubclass'?{}:{render:rawRender()})},before=core(),saved=new Map(),actions=[];
              for(const action of c.actions){
                const result=C.prototype[action.name].call(model,...action.args.map(arg));
                const selected=(action.rememberPath??action.samePath)?.reduce((v,k)=>v[k],model)??result;
                if(action.remember)saved.set(action.remember,selected);
                actions.push({graph:graph(model,prototype,result),...(action.sameAs?{sameIdentity:selected===saved.get(action.sameAs)}:{})});
              }
              const populated={graph:graph(model,prototype),render:rawRender()},after=core();
              row[lane]={initial,actions,populated,before,after,reads,writes,prototype:Object.getPrototypeOf(model)===prototype};
            }finally{if(lane==='candidate')disposeDiagramModel(model);}
          }
          rows.push(row);
        }
      }
      return{rows,metadata};
    }finally{engine.close();}
  });
  const path=info.outputPath('direct-constructors.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('direct-constructors.json',{path,contentType:'application/json'});
  expect(report.rows).toHaveLength(21);expect(report.metadata).toHaveLength(28);
  for(const m of report.metadata){
    // Intrinsic key order, like minified runtime names, remains diagnostic:
    // Firefox materializes these keys differently among the pinned classes.
    // Preserve full ordered observations above; this feature asserts presence
    // and flags rather than claiming complete constructor reflection parity.
    expect(m.arity,m.family).toBe(0);expect(m.withoutNew.name,m.family).toBe('TypeError');
    expect(m.ownKeys,m.family).toHaveLength(3);
    expect(m.ownKeys,m.family).toEqual(expect.arrayContaining(['length','name','prototype']));
    expect(m.length).toEqual({writable:false,enumerable:false,configurable:true});expect(m.nameFlags).toEqual(m.length);
    expect(m.prototype).toEqual({writable:false,enumerable:false,configurable:false});expect(m.backlink).toEqual({writable:true,enumerable:false,configurable:true});expect(m.backlinkIdentity).toBe(true);
  }
  for(const row of report.rows){
    const context=row.family+'/'+row.mode;expect(row.candidate,context).toEqual(row.oracle);
    expect(row.candidate,context).toMatchObject({reads:0,writes:0,prototype:true,before:row.fixture.coreError,after:row.fixture.coreError});
    expect(row.candidate.initial.graph,context).toEqual(row.fixture.initial.graph);
    if(row.mode!=='helperFirstSubclass')expect(row.candidate.initial.render,context).toEqual(row.fixture.initial.render);
    expect(row.candidate.populated,context).toEqual(row.fixture.populated);
    for(let i=0;i<row.fixture.actions.length;i++){expect(row.candidate.actions[i].graph,context+'/'+i).toEqual(row.fixture.actions[i].graph);if(row.fixture.actions[i].sameIdentity)expect(row.candidate.actions[i].sameIdentity,context+'/'+i).toBe(true);}
  }
});

test('seven captured classes survive registry replacement and independent close without shared state',async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const fixture=await(await fetch('/fixtures/public-direct-constructors.json')).json(),corpus=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
    const {loadRegistry}=await import('/dist/package-web/package/registry.js'),{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
    const a=await loadRegistry(),b=await loadRegistry(),factories=[],rows=[];
    const arg=v=>v&&typeof v==='object'?v.array?v.array.map(arg):Object.fromEntries(v.object.map(([k,x])=>[k,arg(x)])):v;
    const view=new DataView(new ArrayBuffer(8)),bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
    try{
      for(const c of fixture.cases){
        const source=corpus.cases.find(x=>x.family===c.family),raw=(await import('/dist/package-web/package/diagrams/'+c.family+'.js')).default,oracle=(await import('/dist/evidence/pele-esm/diagrams/'+c.family+'.js')).default;
        a.register(raw);b.register(raw);const x=a.parse(source.source),y=b.parse(source.source),ref=oracle.parse(source.source,source.config,undefined,source.lineOffset,{maxEdges:undefined});
        factories.push({c,raw,oracle,A:x.constructor,B:y.constructor,C:ref.constructor});a.dispose(x);b.dispose(y);
        a.register({type:c.family,parse(){throw Error('Replacement parse');},render(){throw Error('Replacement render');}});
      }
      a.close();b.close();
      for(const {c,raw,oracle,A,B,C}of factories){
        const x=new A(),y=new B(),expected=new C(),fresh=new A(),firstValues=Object.values(y),freshValues=Object.values(fresh);
        try{
          for(const action of c.actions){x[action.name](...action.args.map(arg));expected[action.name](...action.args.map(arg));}
          const independent=Object.keys(y).every((k,i)=>Object.is(y[k],firstValues[i]))&&Object.keys(fresh).every((k,i)=>Object.is(fresh[k],freshValues[i]))&&Object.keys(x).filter(k=>x[k]&&typeof x[k]==='object').every(k=>x[k]!==y[k]&&x[k]!==fresh[k]);
          const draw=(entry,model)=>{const trace=[],r=entry.render(model,{}, {now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}});return{svg:r.svg,width:bits(r.width),height:bits(r.height),trace};};
          const actual=draw(raw,x),wanted=draw(oracle,expected);disposeDiagramModel(x);
          let disposed;try{raw.render(x,{},{});}catch(e){disposed={name:e.name,message:e.message};}
          // The retained class constructs again after an independently owned
          // instance is disposed. First helper still uses the captured codec.
          const later=new A();try{const action=c.actions[0];later[action.name](...action.args.map(arg));rows.push({family:c.family,independent,actual,wanted,disposed,laterType:later.type});}finally{disposeDiagramModel(later);}
        }finally{for(const model of [x,y,fresh])disposeDiagramModel(model);}
      }
      return rows;
    }finally{a.close();b.close();}
  });
  const path=info.outputPath('direct-constructor-lifetimes.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('direct-constructor-lifetimes.json',{path,contentType:'application/json'});
  expect(report).toHaveLength(7);for(const row of report){expect(row.independent,row.family).toBe(true);expect(row.actual,row.family).toEqual(row.wanted);expect(row.disposed,row.family).toEqual({name:'TypeError',message:'Public model document is disposed'});expect(row.laterType).toBe(row.family);}
});
