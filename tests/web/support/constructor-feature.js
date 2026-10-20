// Browser-side public constructor feature driver. No native SPI or selectors.
const view=new DataView(new ArrayBuffer(8));
const bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
const error=e=>e?.name==='PeleError'?{name:e.name,message:e.message,code:e.code,type:e.type??null,line:e.line??0,column:e.column??0,snippet:e.snippet??''}:{name:e?.name,message:e?.message};
const caught=run=>{try{run();return null;}catch(e){return error(e);}};
const arg=v=>v&&typeof v==='object'?v.array?v.array.map(arg):Object.fromEntries(v.object.map(([k,x])=>[k,arg(x)])):v;
function graph(root,prototype,result){
  const nodes=[],ids=new Map();
  function atom(v){
    if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];
    if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');
    const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===prototype?'instance':'object',properties:[]};nodes.push(n);
    if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);
    for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}
    return['ref',id];
  }
  const rootAtom=atom(root),returned=atom(result);return{root:rootAtom,nodes,result:returned};
}
function draw(entry,model){
  const trace=[],resources={now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}};
  try{const r=entry.render(model,{},resources);return{ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height),trace};}catch(e){return{ok:false,error:error(e),trace};}
}
function actions(model,c,entry){
  const saved=new Map();
  return c.actions.map(action=>{
    // getType is an own function, while the other helpers are prototype methods.
    const result=model[action.name](...action.args.map(arg));
    const path=action.rememberPath??action.samePath,selected=path?path.reduce((v,k)=>v[k],model):result;
    const row={graph:graph(model,Object.getPrototypeOf(model),result)};
    if(action.draw)row.render=draw(entry,model);
    if(action.remember){if(!selected||typeof selected!=='object')throw Error('Vacuous remembered alias');saved.set(action.remember,selected);}
    if(action.sameAs)row.sameIdentity=selected===saved.get(action.sameAs);
    if(action.differentFrom)row.differentIdentity=selected!==saved.get(action.differentFrom);
    return row;
  });
}
function metadata(C){
  const flags=(o,k)=>{const d=Object.getOwnPropertyDescriptor(o,k);return{writable:d.writable,enumerable:d.enumerable,configurable:d.configurable};};
  return{name:C.name,withoutNew:caught(()=>C()),arity:C.length,ownKeys:Reflect.ownKeys(C),length:flags(C,'length'),nameFlags:flags(C,'name'),prototype:flags(C,'prototype'),backlink:flags(C.prototype,'constructor'),backlinkIdentity:C.prototype.constructor===C};
}
async function inputs(paths){
  const fixtures=await Promise.all(paths.map(async p=>(await fetch('/fixtures/'+p)).json()));
  return{cases:fixtures.flatMap(x=>x.cases),corpus:(await(await fetch('/fixtures/pele/diagram-entries.json')).json()).cases};
}
export async function constructionFeature(paths){
  const {cases,corpus}=await inputs(paths);
  const {loadRenderer}=await import(location.origin+'/dist/package-web/package/full.js');
  const {loadDiagramEntries,disposeDiagramModel}=await import(location.origin+'/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const oracle=await import(location.origin+'/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
  const rows=[],constructors=[],shared={candidate:new Map(),oracle:new Map()};
  try{
    for(const c of cases){
      const source=corpus.find(x=>x.family===c.family),raw=(await import(location.origin+'/dist/package-web/package/diagrams/'+c.family+'.js')).default,refRaw=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/'+c.family+'.js')).default;
      const parsed=engine.parse(source.source),prepared=raw.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});
      const refParsed=oracle.parse(source.source),refPrepared=refRaw.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});
      const classes={candidate:[parsed.constructor,prepared.constructor],oracle:[refParsed.constructor,refPrepared.constructor]};
      if(c.family==='mindmap'||c.family==='kanban'){
        shared.candidate.set(c.family,{parsed:parsed.getType,constructed:[]});shared.oracle.set(c.family,{parsed:refParsed.getType,constructed:[]});
      }
      engine.dispose(parsed);disposeDiagramModel(prepared);
      for(const lane of ['oracle','candidate'])for(const [origin,C]of classes[lane].entries())constructors.push({family:c.family,lane,origin:origin?'prepared':'core',...metadata(C)});
      for(const mode of ['plain','helperFirstSubclass','alternate']){
        const row={family:c.family,mode,fixture:c};
        for(const lane of ['oracle','candidate']){
          const C=classes[lane][mode==='helperFirstSubclass'?1:0],entry=lane==='candidate'?raw:refRaw;
          let reads=0,writes=0;const ignored=new Proxy({},{get(){reads++;throw Error('Ignored getter');},ownKeys(){reads++;throw Error('Ignored keys');},getPrototypeOf(){reads++;throw Error('Ignored prototype');}});
          let model,prototype;
          if(mode==='helperFirstSubclass'){class Derived extends C{}model=new Derived(ignored,Symbol('ignored'));prototype=Derived.prototype;}
          else if(mode==='alternate'){function Alternate(){}Alternate.prototype=Object.create(C.prototype);for(const [key]of c.initial.graph.nodes[0].properties)Object.defineProperty(Alternate.prototype,key,{set(){writes++;throw Error('Inherited field setter');},configurable:true});model=Reflect.construct(C,[ignored,Symbol('ignored')],Alternate);prototype=Alternate.prototype;}
          else{model=new C(ignored,Symbol('ignored'));prototype=C.prototype;}
          try{
            if(shared[lane].has(c.family)&&mode!=='helperFirstSubclass')shared[lane].get(c.family).constructed.push(model.getType);
            const core=()=>caught(()=>lane==='candidate'?engine.render(model,{}):oracle.render(model,{}));
            const initial={graph:graph(model,prototype)},before=core();
            if(mode!=='helperFirstSubclass'){
              // State raw render extracts rootDoc and replaces states. Its fixture
              // deliberately renders a separate empty instance before helpers.
              if(c.family==='state'){const blank=new C();try{initial.render=draw(entry,blank);}finally{if(lane==='candidate')disposeDiagramModel(blank);}}
              else initial.render=draw(entry,model);
            }
            const steps=actions(model,c,entry),populated={graph:graph(model,prototype),render:draw(entry,model)},after=core();
            row[lane]={initial,actions:steps,populated,before,after,reads,writes,prototype:Object.getPrototypeOf(model)===prototype};
          }finally{if(lane==='candidate')disposeDiagramModel(model);}
        }
        rows.push(row);
      }
    }
    const sharedIdentity={};
    for(const lane of ['candidate','oracle']){
      const k=shared[lane].get('kanban'),m=shared[lane].get('mindmap');
      sharedIdentity[lane]={present:typeof k?.parsed==='function'&&typeof m?.parsed==='function',parsedSame:k?.parsed===m?.parsed,constructedSame:[...(k?.constructed??[]),...(m?.constructed??[])].every(f=>f===k.parsed),constructedCount:(k?.constructed.length??0)+(m?.constructed.length??0)};
    }
    return{rows,metadata:constructors,sharedIdentity};
  }finally{engine.close();}
}
export async function constructorLifetimes(paths){
  const {cases,corpus}=await inputs(paths);
  const {loadRegistry}=await import(location.origin+'/dist/package-web/package/registry.js'),{loadDiagramEntries,disposeDiagramModel}=await import(location.origin+'/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const a=await loadRegistry(),b=await loadRegistry(),factories=[],rows=[];
  try{
    for(const c of cases){
      const source=corpus.find(x=>x.family===c.family),raw=(await import(location.origin+'/dist/package-web/package/diagrams/'+c.family+'.js')).default,oracle=(await import(location.origin+'/dist/evidence/pele-esm/diagrams/'+c.family+'.js')).default;
      a.register(raw);b.register(raw);const x=a.parse(source.source),y=b.parse(source.source),ref=oracle.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});
      factories.push({c,raw,oracle,A:x.constructor,B:y.constructor,C:ref.constructor});a.dispose(x);b.dispose(y);
      a.register({type:c.family,parse(){throw Error('Replacement parse');},render(){throw Error('Replacement render');}});
    }
    a.close();b.close();
    for(const {c,raw,oracle,A,B,C}of factories){
      const x=new A(),y=new B(),expected=new C(),fresh=new A();
      try{
        const yBefore=graph(y,B.prototype),freshBefore=graph(fresh,A.prototype);
        const actualActions=actions(x,c,raw),expectedActions=actions(expected,c,oracle);
        const independentObjects=Object.keys(x).filter(k=>x[k]&&typeof x[k]==='object').every(k=>x[k]!==y[k]&&x[k]!==fresh[k]);
        const actual={graph:graph(x,A.prototype),render:draw(raw,x)},wanted={graph:graph(expected,C.prototype),render:draw(oracle,expected)};
        const yAfter=graph(y,B.prototype),freshAfter=graph(fresh,A.prototype);disposeDiagramModel(x);
        const disposed=caught(()=>raw.render(x,{},{}));
        const later=new A(),refLater=new C();
        try{const action=c.actions[0],r=later[action.name](...action.args.map(arg)),rr=refLater[action.name](...action.args.map(arg));rows.push({family:c.family,actualActions,expectedActions,actual,wanted,yBefore,yAfter,freshBefore,freshAfter,independentObjects,disposed,later:graph(later,A.prototype,r),expectedLater:graph(refLater,C.prototype,rr)});}finally{disposeDiagramModel(later);}
      }finally{for(const model of [x,y,fresh])disposeDiagramModel(model);}
    }
    return rows;
  }finally{a.close();b.close();}
}
