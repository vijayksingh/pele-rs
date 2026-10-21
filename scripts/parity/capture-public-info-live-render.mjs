import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(sha(fs.readFileSync('node_modules/pele/dist/pele.js'))!==pin)throw Error('Oracle mismatch');
const info=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/info.js')))).default;
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const state=m=>Object.fromEntries(Object.keys(m).map(k=>[k,m[k]===undefined?{undefined:true}:m[k]]));
const cases=[];
for(const mode of ['normal','title-mutate','version-mutate','throw','nested']){
 const model=info.parse('info\ntitle Before\naccTitle: Before accessible\naccDescr: Before description',{},undefined,0,{});
 const sentinel={mode},trace=[];let fired=false,nested;
 const plain={measurer:{width(t,s){return t.length*s*.6;}}};
 const width=(text,size)=>{
  trace.push([text,bits(size)]);
  if(!fired&&((mode==='version-mutate'&&text==='Pele v1.6.0')||(mode!=='version-mutate'&&mode!=='normal'))){
   fired=true;model.title='After';model.accTitle='After accessible';model.accDescr='After description';
   if(mode==='throw')throw sentinel;
   if(mode==='nested'){const r=info.render(model,{},plain);nested={svg:r.svg,width:bits(r.width),height:bits(r.height)};}
  }
  return text.length*size*.6;
 };
 let result;try{const r=info.render(model,{}, {measurer:{width}});result={ok:true,svg:r.svg,width:bits(r.width),height:bits(r.height)};}catch(e){result={ok:false,original:e===sentinel};}
 const after=state(model),r=info.render(model,{},plain);
 cases.push({mode,trace,result,after,...(nested?{nested}:{}),recovery:{svg:r.svg,width:bits(r.width),height:bits(r.height)}});
}
const model=info.parse('info',{},undefined,0,{}),reads=[];
for(const key of ['title','accTitle','accDescr'])Object.defineProperty(model,key,{configurable:true,get(){reads.push(key);return key==='title'?'Title':key;}});
info.render(model,{}, {measurer:{width(t,s){reads.push(['width',t]);return t.length*s*.6;}}});
const report={oracleSHA256:pin,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),infoChunkSHA256:sha(fs.readFileSync('node_modules/pele/dist/esm/chunks/QZEBSHRJ.js')),textChunkSHA256:sha(fs.readFileSync('node_modules/pele/dist/esm/chunks/RZSVICHU.js')),readOrder:reads,cases};
fs.writeFileSync(process.argv[2]??'fixtures/public-info-live-render.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify({cases:cases.map(c=>({mode:c.mode,ok:c.result.ok,widths:c.trace.length})),readOrder:reads}));
