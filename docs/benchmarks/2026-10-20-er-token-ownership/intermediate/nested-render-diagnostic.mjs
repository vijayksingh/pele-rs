import {createServer} from 'node:http';
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {chromium} from '@playwright/test';
const root=resolve('parse-architecture/er-token-ownership/package'),oracle=resolve('node_modules/pele/dist/pele.js');
const sha=b=>createHash('sha256').update(b).digest('hex');
async function hashes(){const result={};async function walk(p=''){for(const e of await readdir(resolve(root,p),{withFileTypes:true})){const f=p+e.name;if(e.isDirectory())await walk(f+'/');else if(e.isFile())result[f]=sha(await readFile(resolve(root,f)));else throw Error('Unexpected package entry');}}await walk();return result;}
const before=await hashes(),report={harnessSHA256:sha(await readFile(new URL(import.meta.url))),runtimeHashes:before,oracleSHA256:sha(await readFile(oracle))};
const server=createServer(async(req,res)=>{try{const p=new URL(req.url,'http://localhost').pathname;if(p==='/'){res.end('<!doctype html>');return;}const f=p==='/oracle.js'?oracle:resolve(root,'.'+p);if(f!==oracle&&!f.startsWith(root+'/'))throw Error('Outside package');res.setHeader('Content-Type',extname(f)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(f));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser;const deadline=setTimeout(()=>void browser?.close(),60000);
try{
 browser=await chromium.launch({headless:true,timeout:15000});report.browser=browser.version();const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}`);
 report.rows=await page.evaluate(async()=>{
  const js=await import('/oracle.js'),api=await import('/index.js'),wasm=await api.loadRenderer();
  const source='erDiagram\nA { int id PK\n string first }\nB { int id PK\n string second }\nA ||--o{ B : owns';
  const width=(text,size,style)=>text.length*size*((style&1)?.71:.6),rows=[];
  for(const variant of ['none','nested-render','nested-render-edited-attribute','nested-render-throw','source-render-control','cross-document-control']){
   const row={variant};
   for(const [name,renderer]of [['js',js],['wasm',wasm]]){
    const model=renderer.parse(source),other=renderer.parse(source),saved=model.entities.get('B'),attributes=saved.attributes,sentinel=new Error('nested render sentinel');let fired=false,nested=null;
    const outcome=call=>{try{return{ok:true,...call()}}catch(error){return{ok:false,name:error?.name,message:error?.message,code:error?.code,original:error===sentinel}}};
    const render=(input,measurer)=>renderer.render(input,{measurer});
    try{
     const first=outcome(()=>render(model,{width(...args){
      if(!fired){fired=true;
       if(variant==='nested-render-edited-attribute')attributes[1].name='changed by callback';
       if(variant==='nested-render'||variant==='nested-render-edited-attribute')nested=outcome(()=>render(model,{width}));
       if(variant==='nested-render-throw')nested=outcome(()=>render(model,{width(){throw sentinel;}}));
       if(variant==='source-render-control')nested=outcome(()=>render(source,{width}));
       if(variant==='cross-document-control')nested=outcome(()=>render(other,{width}));
      }
      return width(...args);
     }}));
     row[name]={first,nested,next:outcome(()=>render(model,{width})),entityIdentity:model.entities.get('B')===saved,arrayIdentity:saved.attributes===attributes,attributeName:attributes[1].name};
    }finally{if(name==='wasm'){renderer.dispose(model);renderer.dispose(other);}}
   }
   row.comparison=Object.fromEntries(Object.keys(row.js).map(k=>[k,JSON.stringify(row.js[k])===JSON.stringify(row.wasm[k])]));rows.push(row);
  }
  return rows;
 });
 if(JSON.stringify(before)!==JSON.stringify(await hashes()))throw Error('Package changed');report.completed=true;
 await writeFile('parse-architecture/er-token-ownership/nested-render-diagnostic.json',JSON.stringify(report,null,2)+'\n');
 for(const row of report.rows)console.log(row.variant,JSON.stringify(row.comparison),JSON.stringify({jsNestedError:row.js.nested?.ok?null:row.js.nested,wasmNestedError:row.wasm.nested?.ok?null:row.wasm.nested}));
}finally{clearTimeout(deadline);await browser?.close();await new Promise(r=>server.close(r));}
