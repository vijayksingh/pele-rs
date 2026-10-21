// Public constructor and method sequence from the pinned scheduler capture.
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
function graph(value,C,result,options){const nodes=[],ids=new Map();function atom(v){if(v===options)return["host","options"];if(v===undefined)return['undefined'];if(v===null)return['null'];if(typeof v==='string')return['string',v];if(typeof v==='number')return['number',bits(v)];if(typeof v==='boolean')return['bool',v];if(typeof v==='function')return['function',v.length];if(ids.has(v))return['ref',ids.get(v)];if(typeof v!=='object')throw Error('Unexpected constructor graph atom');const id=nodes.length;ids.set(v,id);const n={kind:Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':v instanceof Date?'date':Object.getPrototypeOf(v)===C.prototype?'instance':'object',properties:[]};nodes.push(n);if(v instanceof Date)n.milliseconds=bits(+v);if(v instanceof Map)n.entries=[...v].map(([k,x])=>[atom(k),atom(x)]);if(v instanceof Set)n.values=[...v].map(atom);for(const k of Object.getOwnPropertyNames(v)){const d=Object.getOwnPropertyDescriptor(v,k);if(!('value'in d))throw Error('Unexpected accessor');n.properties.push([k,atom(d.value),(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)]);}return['ref',id];}const root=atom(value),returned=atom(result);return{root,nodes,result:returned};}

export function runGanttOwnedScheduler(entry,dispose){
 const seed=entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :t,2026-01-01,1d',{},undefined,0,{});
 const C=Object.getPrototypeOf(seed).constructor;dispose?.(seed);
const cases=[];
for(const method of [19,25,26,30])for(const mode of ['warn-false','normal','warn-throw','warn-reentry','excludes']){
 const id=`${method}-${mode}`,trace=[],sentinel={id};let model,reads=0;
 const options={};Object.defineProperty(options,'warn',{get(){trace.push(['get',++reads]);return mode==='warn-false'?false:function(message){trace.push(['call',this===options,[...message].map(c=>c.charCodeAt(0))]);
  if(mode==='warn-reentry'){model.addSection('Nested');model.tasks[0].raw.end='2d';if(model.tasks.length===2)model.addTask('Appended',':c,2024-01-05,1d');}
  if(mode==='warn-throw')throw sentinel;
 };},configurable:true});
 model=new C(options);model.dateFormat='YYYY-MM-DD';model.now=Date.UTC(2024,0,9,15,20,30);
 model.addTask('Alpha',`:a,2024-01-01,${mode==='excludes'?'3d':'bad-duration'}`);model.addTask('Beta',':b,after a,1d');
 if(mode==='excludes'){model.excludes=['2024-01-02'];model.excludeSet=new Set(model.excludes);model.excludedDays=[false,false,false,false,false,false,false];model.datedTokens=true;}
 const originalTasks=model.tasks,originalA=model.tasks[0];
 function execute(){let result,error;try{
  if(method===19)result=model.getTasks(Date.UTC(2024,0,9,15,20,30));
  else if(method===25)result=model.compileTasks();else if(method===26)result=model.compileTask(originalA);
  else result=model.getEndDate(new Date('2024-01-01T00:00:00.000Z'),mode==='excludes'?'3d':'bad-duration');
 }catch(e){error=e===sentinel?'original':{name:e.name,message:e.message,...(e.name==='PeleError'?{code:e.code,type:e.type,line:e.line,column:e.column,snippet:e.snippet}: {})};}
 return{graph:graph(model,C,result,options),trace:trace.slice(),error:error??null,returnedTasks:result===originalTasks};}
 const first=execute();Object.defineProperty(options,'warn',{value:undefined,configurable:true});trace.length=0;
 const recovery=execute();cases.push({id,method,mode,first,recovery});dispose?.(model);
}
return cases;
}
