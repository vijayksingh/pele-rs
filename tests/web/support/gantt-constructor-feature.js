// Scenarios extracted from the pinned Gantt oracle probe, no family runtime code.
const view=new DataView(new ArrayBuffer(8));
const bits=n=>{view.setFloat64(0,n);return view.getBigUint64(0).toString(16).padStart(16,'0');};
function graph(value,C,result){const nodes=[],ids=new Map();function atom(v){if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}
export function runGanttCases(entry,dispose=()=>{}){
const seed=entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :t, 2026-01-01, 1d',{},undefined,0,{});
const C=Object.getPrototypeOf(seed).constructor;dispose(seed);
const defaults=new C(),widthTrace=[],resources={now:1704067200000,measurer:{width(text,size){widthTrace.push([text,size]);return text.length*size*.6;}}};
let drawing;try{const r=entry.render(defaults,{},resources);drawing={ok:true,svg:r.svg,width:r.width,height:r.height,trace:widthTrace};}catch(e){drawing={ok:false,error:{name:e.name,message:e.message},trace:widthTrace};}
const initialModel=new C(),initial=graph(initialModel,C);dispose(initialModel);
const cases=[];
const definitions=[
 ['lazy-undefined',undefined],['lazy-null',null],['lazy-false',false],['lazy-zero',0],['lazy-nan',NaN],['lazy-empty',''],['lazy-bigint-zero',0n],
 ['changing-options','function'],['producer-reentry','function'],['warn-reentry','function'],
 ['producer-throw-object','function'],['producer-throw-undefined','function'],['producer-throw-symbol','function'],
 ['second-get-throw','function'],['second-noncallable-false','function'],['second-noncallable-object','function'],
 ['producer-noncallable-null','function'],['producer-noncallable-number','function'],['warn-throw','function'],
];
for(const [id,first] of definitions){
 const trace=[],sentinel={id},symbol=Symbol(id);let optionReads=0,warnReads=0,readsInConstructor=0;
 let model;const names=new Map(), label=v=>names.get(v)??(v===undefined?'undefined':v===null?'null':typeof v==='object'?'object':typeof v==='symbol'?'symbol':v);
 const firstOptions={},secondOptions={};names.set(firstOptions,'options1');names.set(secondOptions,'options2');
 function firstWarn(){throw Error('First callback must never run');}names.set(firstWarn,'firstWarn');
 function warn(message){trace.push(['call','warn',label(this),label(message)]);if(id==='warn-reentry')model.addSection('nested warning');if(id==='warn-throw')throw sentinel;}
 names.set(warn,'warn');
 Object.defineProperty(firstOptions,'warn',{get(){warnReads++;trace.push(['get','options1','warn']);return first==='function'?firstWarn:first;},configurable:true});
 Object.defineProperty(secondOptions,'warn',{get(){warnReads++;trace.push(['get','options2','warn']);if(id==='second-get-throw')throw sentinel;return id==='second-noncallable-false'?false:id==='second-noncallable-object'?{}:warn;},configurable:true});
 const retained=new Proxy(firstOptions,{ownKeys(){throw Error('Options enumerated');},get(t,k,r){readsInConstructor++;return Reflect.get(t,k,r);}});
 model=new C(retained,new Proxy({},{get(){throw Error('Extra argument read');}}));
 if(readsInConstructor||model.options!==retained)throw Error('Constructor reads or copies options');
 Object.defineProperty(model,'options',{get(){trace.push(['get','model','options']);return ++optionReads===1?firstOptions:secondOptions;},configurable:true});
 function producer(){trace.push(['call','producer',label(this)]);if(id==='producer-reentry'){model.addSection('nested producer');Object.defineProperty(secondOptions,'warn',{value:()=>{throw Error('Stale callee replaced');}});}if(id==='producer-throw-object')throw sentinel;if(id==='producer-throw-undefined')throw undefined;if(id==='producer-throw-symbol')throw symbol;return 42;}
 let result,thrown=false,error;try{result=model.warn(id==='producer-noncallable-null'?null:id==='producer-noncallable-number'?3:producer);}catch(e){thrown=true;error=e===sentinel?'original-object':e===symbol?'original-symbol':e===undefined?'original-undefined':{name:e.name,message:e.message};}
 cases.push({id,trace,optionReads,warnReads,readsInConstructor,result:label(result),thrown,...(thrown?{error}:{}),sections:[...model.sections]});dispose(model);
}
dispose(defaults);return{initial,drawing,cases};
}

export function runGanttRetainedCases(entry,dispose=()=>{}){
  const seed=entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :t, 2026-01-01, 1d',{},undefined,0,{}),C=seed.constructor;dispose(seed);
  const rows=[];
  for(const name of ['negative-zero','nan','symbol','bigint','long-string','proxy']){
    let reads=0;
    const value=name==='negative-zero'?-0:name==='nan'?NaN:name==='symbol'?Symbol('message'):name==='bigint'?0n:name==='long-string'?'x'.repeat(16_000_001):new Proxy({}, {get(){reads++;throw Error('Returned value inspected');},ownKeys(){reads++;throw Error('Returned value enumerated');}});
    const trace=[];
    const model=new C({warn(message){trace.push(['warn',Object.is(message,value),typeof message,typeof message==='number'?bits(message):typeof message==='string'?message.length:null]);}});
    model.warn(()=>{trace.push(['producer']);return value;});
    rows.push({name,reads,trace});dispose(model);
  }
  return rows;
}
