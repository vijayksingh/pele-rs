// Separate untimed browser. Real native apply calls are counted; the comparison
// disables only optional capability discovery, retaining old canonical ingress.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {chromium} from '@playwright/test';
import {cases,now} from './cases.mjs';
const arg=name=>{const i=process.argv.indexOf(name);return i<0?undefined:process.argv[i+1];};
const root=resolve(arg('--candidate')),output=resolve(arg('--output'));
const workloads=cases().filter(value=>['sankey/small','sankey/medium','sankey/large'].includes(value.id));assert.equal(workloads.length,3);
const report={schemaVersion:1,startedAt:new Date().toISOString(),root,checks:[],probeInsideTimer:false,directParseSPI:false};
const server=createServer(async(req,res)=>{try{
  res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
  const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Untimed sparse ingress activation</title>');return;}
  assert(path.startsWith('/candidate/'));const file=resolve(root,'.'+path.slice('/candidate'.length));assert(file.startsWith(root+'/'));
  res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
}catch{res.writeHead(404);res.end();}});
await new Promise(done=>server.listen(0,'127.0.0.1',done));let browser;
const deadline=setTimeout(()=>void browser?.close(),120000);
try{
  browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();
  const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const observed=await page.evaluate(async({workloads,now})=>{
    Date.now=()=>now;
    const api=await import('/candidate/index.js'),wasm=await import('/candidate/pkg/pele.js');
    const prototype=wasm.MutableDocument.prototype,names=['source_ingress_identity','apply_source_input','apply_binary'];
    const saved=new Map(names.map(name=>[name,Object.getOwnPropertyDescriptor(prototype,name)]));
    for(const[name,descriptor]of saved)if(typeof descriptor?.value!=='function')throw Error('Missing ingress capability '+name);
    const restore=()=>{for(const[name,descriptor]of saved)Object.defineProperty(prototype,name,descriptor);};
    const bitsBuffer=new DataView(new ArrayBuffer(8));const bits=value=>{bitsBuffer.setFloat64(0,value);return bitsBuffer.getBigUint64(0).toString(16).padStart(16,'0');};
    const output=value=>({...value,width:bits(value.width),height:bits(value.height),keys:Object.keys(value)});
    const snapshot=root=>{
      const ids=new Map(),objects=[],nodes=[];
      const atom=value=>{if(value===undefined)return['undefined'];if(value===null)return['null'];if(typeof value==='number')return['number',bits(value)];if(typeof value!=='object')return[typeof value,value];if(!ids.has(value)){ids.set(value,objects.length);objects.push(value);}return['ref',ids.get(value)];};
      const first=atom(root);
      for(let id=0;id<objects.length;id++){
        const value=objects[id],properties=Reflect.ownKeys(value).map(key=>{const d=Object.getOwnPropertyDescriptor(value,key);if(!Object.hasOwn(d,'value'))throw Error('Unexpected accessor');return[key,d.enumerable,d.writable,d.configurable,atom(d.value)];});
        nodes.push({kind:Array.isArray(value)?'array':value instanceof Map?'map':value instanceof Set?'set':'object',properties,...(value instanceof Map?{entries:[...value].map(([k,v])=>[atom(k),atom(v)])}:{}),...(value instanceof Set?{values:[...value].map(atom)}:{})});
      }
      return{root:first,nodes};
    };
    const checks=[],pairs=new Map();
    try{
      for(const mode of ['sparse','canonical']){
        restore();let active;
        if(mode==='canonical')Object.defineProperty(prototype,'source_ingress_identity',{...saved.get('source_ingress_identity'),value(){return undefined;}});
        for(const name of ['apply_source_input','apply_binary']){const descriptor=saved.get(name);Object.defineProperty(prototype,name,{...descriptor,value:function(...args){if(active)active[name==='apply_source_input'?'sparse':'canonical']++;return Reflect.apply(descriptor.value,this,args);}});}
        const renderer=await api.loadRenderer();
        try{for(const workload of workloads)for(const profile of ['fixed','canvas']){
          const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:api.canvasMeasurer('sans-serif','monospace');if(!measurer)throw Error('Canvas unavailable');
          const options={...workload.options,measurer};let model;active={sparse:0,canonical:0};
          try{
            model=renderer.parse(workload.source,options);const first=model.nodes[0],rootPrototype=Object.getPrototypeOf(model);
            const extension={label:'probe 漢\ud800',negativeZero:-0,first};extension.self=extension;extension.map=new Map([[first,extension]]);Object.defineProperty(extension,'hidden',{value:'retained',enumerable:false,writable:false,configurable:true});model.extension=extension;
            const firstEdited=output(renderer.render(model,options)),afterEdited=snapshot(model);
            const helper=model.findOrCreateNode('probe helper node'),helperAlias=helper===model.lookup.get('probe helper node')&&helper===model.nodes.at(-1);
            const afterHelper=output(renderer.render(model,options)),finalGraph=snapshot(model);
            const identities=model.extension===extension&&extension.self===extension&&extension.first===first&&extension.map.get(first)===extension&&model.nodes[0]===first&&Object.getPrototypeOf(model)===rootPrototype;
            if(!identities||!helperAlias)throw Error('Public identity failure');
            if(mode==='sparse'?active.sparse===0||active.canonical!==0:active.canonical===0||active.sparse!==0)throw Error('Wrong actual ingress dispatch');
            const semantic={firstEdited,afterEdited,helperAlias,afterHelper,finalGraph,identities},key=workload.id+'/'+profile;
            if(mode==='sparse')pairs.set(key,JSON.stringify(semantic));else if(pairs.get(key)!==JSON.stringify(semantic))throw Error(key+': sparse/canonical public state or output differs');
            checks.push({mode,case:workload.id,profile,calls:{...active},...semantic});
          }finally{if(model)renderer.dispose(model);active=undefined;}
        }}finally{renderer.close();}
      }
    }finally{restore();}
    const methodsRestored=names.every(name=>{const a=Object.getOwnPropertyDescriptor(prototype,name),b=saved.get(name);return a.value===b.value&&a.writable===b.writable&&a.configurable===b.configurable&&a.enumerable===b.enumerable;});
    if(!methodsRestored)throw Error('Native originals not restored');return{checks,methodsRestored};
  },{workloads,now});
  Object.assign(report,observed);report.succeeded=true;
}catch(error){report.error={name:error.name,message:error.message,stack:error.stack};process.exitCode=1;}
finally{clearTimeout(deadline);await browser?.close();report.browserClosedBeforeTiming=true;await new Promise(done=>server.close(done));report.completedAt=new Date().toISOString();await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');}
