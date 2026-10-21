const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
function graph(value,C,result,options){const nodes=[],ids=new Map();function atom(v){if(v===options)return["host","options"];if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':v instanceof Date?'date':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Date)n.milliseconds=bits(+v);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}


export function runGanttWarningErrors(entry,dispose=()=>{}){
 const seed=entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :a,2024-01-01,1d',{},undefined,0,{}),C=Object.getPrototypeOf(seed).constructor;dispose(seed);
 const rows=[];
 for(const method of [28,30])for(const kind of ['undefined','null','false','object']){
  const trace=[],options={get warn(){trace.push(['get']);return function(message){trace.push(['call',this===options,message]);};}},model=new C(options);
  model.dateFormat='YYYY-MM-DD';model.now=Date.UTC(2024,0,9,15,20,30);
  model.warn=kind==='undefined'?undefined:kind==='null'?null:kind==='false'?false:{};
  const execute=()=>{let result,error;try{result=method===28?model.getStartDate('after missing'):model.getEndDate(new Date('2024-01-01T00:00:00Z'),'bad-duration');}catch(e){error={name:e.name,message:e.message};}
   return{graph:graph(model,C,result,options),trace:trace.slice(),error:error??null};};
  try{const first=execute();delete model.warn;trace.length=0;const recovery=execute();rows.push({method,kind,first,recovery});}finally{dispose(model);}
 }
 return rows;
}
