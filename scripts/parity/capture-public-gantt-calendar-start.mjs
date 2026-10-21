import fs from 'node:fs';import path from 'node:path';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const oracle='node_modules/pele/dist/pele.js',oracleSHA256=sha(fs.readFileSync(oracle));
if(oracleSHA256!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw Error('Unexpected Pele');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/gantt.js')))).default;
const C=Object.getPrototypeOf(entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :t, 2026-01-01, 1d',{},undefined,0,{})).constructor;
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
function graph(value,C,result,options){const nodes=[],ids=new Map();function atom(v){if(v===options)return["host","options"];if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':v instanceof Date?'date':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Date)n.milliseconds=bits(+v);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}

process.env.TZ='UTC';
const cases=[];
const rows=[
 ['known-only','after a b a'],['shadowed-known','after a b'],['shadowed-warn','after a missing'],
 ['warn-false','after a missing'],['normal','after a missing b missing a'],
 ['first-get-throw','after a missing'],['second-get-throw','after a missing'],
 ['second-not-callable','after a missing'],['warn-throw','after a missing'],
 ['warn-reentry','after a missing b'],['getter-edit','after a missing b'],
 ['all-missing','after missing'],['strict-date','2024-02-29'],['loose-date','January 5 2024'],
 ['invalid-date','not-a-date'],['timestamp','1704153600000'],['undefined-end','after a']
];
for(const [id,raw] of rows){
 let model,reads=0;const trace=[],sentinel={id},options={};
 function callback(message){
  trace.push(['call',this===options,Array.from({length:message.length},(_,i)=>message.charCodeAt(i))]);
  if(id==='warn-reentry'){
   model.addSection('Nested');model.tasks[0].endTime=new Date('2024-02-01T00:00:00.000Z');
   model.byId.clear();model.addTask('Replacement',':a,2024-01-01,1d');
  }
  if(id==='warn-throw')throw sentinel;
 }
 Object.defineProperty(options,'warn',{get(){
  trace.push(['get',++reads]);
  if(id==='getter-edit'&&reads===1)model.tasks[0].endTime=new Date('2024-03-01T00:00:00.000Z');
  if((id==='first-get-throw'&&reads===1)||(id==='second-get-throw'&&reads===2))throw sentinel;
  return id==='warn-false'?false:id==='second-not-callable'&&reads===2?0:callback;
 },configurable:true});
 model=new C(options);model.dateFormat='YYYY-MM-DD';model.now=Date.UTC(2024,0,9,15,20,30);
 model.addTask('Alpha',':a,2024-01-01,1d');model.addTask('Beta',':b,2024-01-02,1d');
 model.tasks[0].endTime=new Date('2024-01-02T00:00:00.000Z');model.tasks[1].endTime=new Date('2024-01-03T00:00:00.000Z');
 if(id==='undefined-end')model.tasks[0].endTime=undefined;
 if(id==='timestamp')model.dateFormat='X';
 if(id==='shadowed-known'||id==='shadowed-warn')model.warn=null;
 const a=model.tasks[0],b=model.tasks[1];
 function execute(){let result,error;try{result=model.getStartDate(raw);}catch(e){error=e===sentinel?'original':{name:e.name,message:e.message,...(e.name==='PeleError'?{code:e.code,type:e.type,line:e.line,column:e.column,snippet:e.snippet}: {})};}
  return {graph:graph(model,C,result,options),trace:trace.slice(),error:error??null,aliases:[result!==undefined&&result===a.endTime,result!==undefined&&result===b.endTime]};
 }
 const first=execute();Object.defineProperty(options,'warn',{value:undefined,configurable:true});
 if(id==='shadowed-known'||id==='shadowed-warn')delete model.warn;
 trace.length=0;const recovery=execute();cases.push({id,raw,first,recovery});
}
const report={oracleSHA256,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),timezone:'UTC',body:C.prototype.getStartDate.toString(),cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-gantt-calendar-start.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(cases.map(c=>({id:c.id,error:c.first.error,trace:c.first.trace,aliases:c.first.aliases}))));
