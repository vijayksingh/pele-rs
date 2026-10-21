import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
process.env.TZ='UTC';
const sha=b=>createHash('sha256').update(b).digest('hex');
const oracleSHA256=sha(fs.readFileSync('node_modules/pele/dist/pele.js'));
if(oracleSHA256!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw Error('Unexpected oracle');
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/gantt.js')))).default;
const C=entry.parse('gantt\n dateFormat YYYY-MM-DD\n Task :a,2024-01-01,1d',{},undefined,0,{}).constructor;
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const inputs=[
 {id:'parse-date',method:22,args:['2024-01-09','YYYY-MM-DD'],now:1704813630000},
 {id:'parse-invalid',method:22,args:['2024-02-31','YYYY-MM-DD'],now:1704813630000},
 {id:'parse-month',method:22,args:['Jan 09','MMM DD'],now:1704813630000},
 {id:'parse-epoch',method:22,args:['0','x'],now:1704813630000},
 {id:'parse-empty',method:22,args:['',''],now:1704813630000},
 {id:'parse-limit',method:22,args:['2024-01-09','YYYY-MM-DD'],now:1704813630000,work:400000000},
 {id:'today',method:23,args:[],now:1704813630000},
 {id:'today-invalid',method:23,args:[],now:null}
];
const cases=inputs.map(input=>{
 const model=new C();model.now=input.now===null?NaN:input.now;model.work=input.work??0;
 let value,error;try{value=input.method===22?model.parse(...input.args):model.today();}catch(e){error={name:e.name,message:e.message,...(e.name==='PeleError'?{code:e.code,type:e.type,line:e.line,column:e.column,snippet:e.snippet}: {})};}
 return {...input,result:value===undefined?{kind:'undefined'}:{kind:'date',bits:bits(+value)},work:model.work,error:error??null};
});
const result={oracleSHA256,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),timezone:'UTC',bodies:{parse:C.prototype.parse.toString(),today:C.prototype.today.toString()},cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-gantt-direct-calendar.json',JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify(cases.map(({id,result,work,error})=>({id,result,work,error}))));
