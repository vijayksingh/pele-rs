// Public API scenarios; opaque options are deliberately never enumerated here.
const bitsView=new DataView(new ArrayBuffer(8));
const bits=n=>{bitsView.setFloat64(0,n);return bitsView.getBigUint64(0).toString(16).padStart(16,'0');};
function modelGraph(model,C){
  const nodes=[],ids=new Map();
  function atom(v){
    if(v===undefined)return['undefined'];if(v===null)return['null'];
    if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];
    if(typeof v==='function')return['function',v.length];
    if(ids.has(v))return['ref',ids.get(v)];
    const id=nodes.length;ids.set(v,id);
    const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);
    if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);
    if(v instanceof Set)n.values=[...v].map(atom);
    for(const k of Object.getOwnPropertyNames(v)){
      if(v===model&&k==='options'){n.properties.push([k,['options'],7]);continue;}
      const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected unmasked accessor');
      n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);
    }
    return['ref',id];
  }
  return{root:atom(model),nodes};
}
const errorValue=(error,sentinel)=>error===sentinel?{original:true}:{name:error?.name,message:error?.message,code:error?.code??null};
function draw(entry,model){
  const trace=[];
  try{const r=entry.render(model,{}, {now:1704067200000,measurer:{width(text,size){trace.push([text,bits(size)]);return text.length*size*.6;}}});return{svg:r.svg,width:bits(r.width),height:bits(r.height),trace};}
  catch(e){return{error:errorValue(e,Symbol()),trace};}
}
export const fixtureCaseIds=['warn-null','warn-call','warn-get-throw','warn-call-throw','warn-reentry','warn-replacement','max-undefined','max-zero','max-one','max-get-throw','max-reentry','max-coerce-reentry','max-coerce-throw','max-string-throw','sub-false','sub-true','sub-root-direction','sub-inherit-throw','sub-direction-throw','sub-bigint-zero'];
const extraIds=['warn-throw-undefined','warn-throw-symbol','warn-false','warn-object','warn-return-opaque','warn-return-long','options-replace','options-delete','default-options-accessor','max-bigint','max-symbol','max-replace-edges','warn-caught-inner'];
export function runFlowCases(entry,family,source,dispose=()=>{}){
  const parsed=entry.parse(source.source,source.config,source.title??undefined,source.lineOffset,{maxEdges:undefined});
  const C=Object.getPrototypeOf(parsed).constructor;dispose(parsed);
  const rows=[];
  for(const id of [...fixtureCaseIds,...extraIds]){
    const trace=[];const sentinel=id==='warn-throw-undefined'?undefined:id==='warn-throw-symbol'?Symbol('original'):{original:true};
    let constructorReads=0,active=false,model,entered=false,returnReads=0;
    const ignored=new Proxy({}, {get(){returnReads++;throw Error('Ignored return read');},ownKeys(){returnReads++;throw Error('Ignored return enumerated');}});
    const options=new Proxy({}, {
      get(t,k){
        if(!active)constructorReads++;trace.push(['get','options',String(k)]);
        if(id===`${k==='warn'?'warn':'max'}-get-throw`)throw sentinel;
        if(k==='inheritDir'){if(id==='sub-inherit-throw')throw sentinel;if(id==='sub-root-direction')model.setDirection('TB');return id==='sub-false'?false:id==='sub-bigint-zero'?0n:true;}
        if(k==='direction'){if(id==='sub-direction-throw')throw sentinel;return 'RL';}
        if(k==='warn'){
          if(id==='warn-null')return null;if(id==='warn-false')return false;if(id==='warn-object')return {};
          return function(message){
            trace.push(['call','warn',this===options,message]);
            if(id==='warn-reentry')model.addVertex('B');
            if(id==='warn-call-throw'||id==='warn-throw-undefined'||id==='warn-throw-symbol'){model.addVertex('B');throw sentinel;}
            if(id==='warn-caught-inner'){
              const prior=model.options;model.options={warn(){throw sentinel;}};
              try{model.addVertex('inner',undefined,undefined,[]);}catch(e){trace.push(['caughtInner',e===sentinel]);}
              model.options=prior;
            }
            if(id==='warn-return-opaque')return ignored;
            if(id==='warn-return-long')return 'x'.repeat(16_000_001);
          };
        }
        if(k==='maxEdges'){
          if(id==='max-reentry'&&!entered){entered=true;model.addSingleLink('C','D');}
          if(id==='max-zero')return 0;if(id==='max-one'||id==='max-reentry')return 1;
          if(id==='max-bigint')return 0n;if(id==='max-symbol')return Symbol('limit');
          if(id.startsWith('max-coerce')||id==='max-string-throw'||id==='max-replace-edges')return {
            valueOf(){trace.push(['valueOf']);if(id==='max-coerce-throw')throw sentinel;if(id==='max-coerce-reentry')model.addVertex('B');if(id==='max-replace-edges'){model.edges=[];return 9;}return 0;},
            toString(){trace.push(['toString']);if(id==='max-string-throw')throw sentinel;return '0';}
          };
        }
      },ownKeys(){throw Error('Options enumerated');},getPrototypeOf(){throw Error('Options classified');}
    });
    model=id==='default-options-accessor'?new C():new C(options);
    const defaultOptions=id==='default-options-accessor'?model.options:null;
    const sameOptions=id==='default-options-accessor'?typeof defaultOptions==='object':model.options===options;
    const initial=modelGraph(model,C);active=true;trace.length=0;
    let getter;
    if(id==='default-options-accessor'){
      Object.defineProperty(defaultOptions,'poison',{get(){throw Error('Default options history enumerated');},enumerable:true,configurable:true});
      getter=()=>{trace.push(['get','model','options']);return options;};
      Object.defineProperty(model,'options',{enumerable:true,configurable:true,get:getter});
    }else if(id==='options-replace')model.options={warn(message){trace.push(['replacement',message]);}};
    else if(id==='options-delete')delete model.options;
    else if(id!=='warn-caught-inner'){
      getter=()=>{trace.push(['get','model','options']);return options;};
      Object.defineProperty(model,'options',{enumerable:true,configurable:true,get:getter});
    }
    let error=null;
    try{if(id.startsWith('sub-'))model.addSubGraph(undefined,[],undefined);else if(id.startsWith('max'))model.addSingleLink('A','B');else model.addVertex('A',undefined,undefined,[]);}
    catch(e){error=errorValue(e,sentinel);}
    const state=modelGraph(model,C),afterDescriptor=Object.getOwnPropertyDescriptor(model,'options');
    const descriptorPreserved=getter?afterDescriptor?.get===getter:id==='options-delete'?!afterDescriptor:true;
    const base={family,id,constructorReads,sameOptions,initial,trace:[...trace],error,state};
    const render=draw(entry,model);
    rows.push({...base,returnReads,descriptorPreserved,render});dispose(model);
  }
  return rows;
}

export function runFlowLifetimes(entry,family,source,C,dispose=()=>{}){
  const defaults=new C(),other=new C(),explicit=new C(undefined);
  const independent=defaults.options!==other.options&&other.options!==explicit.options;
  const options=defaults.options;
  defaults.addVertex('before');Object.defineProperty(options,'poison',{enumerable:true,get(){throw Error('Retained default options traversed');}});
  defaults.options={warn(){}};defaults.addVertex('after',undefined,undefined,[]);
  const rows={family,independent,defaultHistory:modelGraph(defaults,C)};
  class Child extends C{marker='child';}
  const child=new Child({warn(){}});child.addVertex('child');rows.subclass={prototype:Object.getPrototypeOf(child)===Child.prototype,marker:child.marker,node:child.nodes.get('child')===child.nodes.values().next().value};
  // A borrowed helper must protect the actual receiver's live field, not only
  // the owner root. Preserve a throwing options Proxy until the native Get.
  const borrowed={...other},trace=[];
  Object.defineProperty(borrowed,'options',{get(){trace.push('options');return {warn(message){trace.push(message);}};},configurable:true,enumerable:true});
  C.prototype.addVertex.call(borrowed,'borrowed',undefined,undefined,[]);
  rows.borrowed={trace,graph:modelGraph(borrowed,Object)};
  rows.arguments=[];
  for(const [name,argument] of [['null',null],['false',false],['zero',0],['negative-zero',-0],['number',7],['string','options'],['symbol',Symbol('options')],['bigint',0n]]){
    let reads=0;const ignored=new Proxy({}, {get(){reads++;throw Error('Extra argument read');}});
    const value=new C(argument,ignored);let error=null;
    try{value.addVertex('argument',undefined,undefined,[]);}catch(e){error=errorValue(e,Symbol());}
    rows.arguments.push({name,same:Object.is(value.options,argument),reads,error,graph:modelGraph(value,C)});dispose(value);
  }
  rows.draw=draw(entry,defaults);
  for(const value of [defaults,other,explicit,child])dispose(value);
  return rows;
}
