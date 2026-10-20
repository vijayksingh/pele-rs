// Untimed public-parse activation proof. Runs in a separate browser which closes
// before the unchanged timing harness starts. No selector or direct parse SPI.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {chromium} from '@playwright/test';
import {cases,now} from './cases.mjs';
const arg=name=>{const i=process.argv.indexOf(name);return i<0?undefined:process.argv[i+1];};
const roots={control:resolve(arg('--control')),candidate:resolve(arg('--candidate'))},output=resolve(arg('--output'));
const selected=['sankey/small','sankey/medium','sankey/large','flowchart/medium'],workloads=cases().filter(value=>selected.includes(value.id));
assert.equal(workloads.length,4);assert.notEqual(roots.control,roots.candidate);
const report={schemaVersion:1,startedAt:new Date().toISOString(),roots,checks:[],selectors:false,directParseSPI:false,probeInsideTimer:false};
const server=createServer(async(req,res)=>{try{
  res.setHeader('Cross-Origin-Opener-Policy','same-origin');res.setHeader('Cross-Origin-Embedder-Policy','require-corp');
  const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Source layout activation preflight</title>');return;}
  const runtime=Object.keys(roots).find(name=>path.startsWith('/'+name+'/'));assert(runtime);
  const file=resolve(roots[runtime],'.'+path.slice(runtime.length+1));assert(file.startsWith(roots[runtime]+'/'));
  res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
}catch{res.writeHead(404);res.end();}});
await new Promise(done=>server.listen(0,'127.0.0.1',done));let browser;
const deadline=setTimeout(()=>void browser?.close(),120000);
try{
  browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();
  const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const result=await page.evaluate(async({workloads,now})=>{
    Date.now=()=>now;const checks=[];
    for(const name of ['control','candidate']){
      const api=await import('/'+name+'/index.js'),native=await import('/'+name+'/pkg/pele.js');
      const renderer=await api.loadRenderer(),Constructor=native.MutableDocument,prototype=Constructor.prototype;
      const parent=Object.getPrototypeOf(prototype),keys=Reflect.ownKeys(prototype);
      const names=['snapshot_source_layout','render_numbers_operation_source_layout_with_functions'];
      const originals=new Map(names.map(key=>[key,Object.getOwnPropertyDescriptor(prototype,key)]));
      for(const[key,original]of originals){if(name==='candidate'&&typeof original?.value!=='function')throw Error('Candidate lacks '+key);if(name==='control'&&original!==undefined)throw Error('Pinned control unexpectedly has '+key);}
      let snapshots=[],updates=[];
      const packetRecord=packet=>{
        if(packet===null)return{capability:null};
        if(!Array.isArray(packet)||(packet.length!==4&&packet.length!==6)||packet[0]!==1||!(packet[2] instanceof Uint8Array))throw Error('Unexpected source layout packet');
        const bytes=packet[2],magic=String.fromCharCode(...bytes.subarray(0,8));
        if(!['PLCSRC02','PLCSRU01'].includes(magic)||bytes.length<45)throw Error('Expected immutable full/update protocol');
        const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),layoutId=view.getBigUint64(8,true).toString(),revision=view.getBigUint64(16,true).toString();
        if(layoutId==='0')throw Error('Zero layout identity');
        return{capability:magic,layoutId,revision,canonicalBytes:packet[1],recipeBytes:bytes.length,...(magic==='PLCSRC02'?{type:JSON.parse(packet[3]).type}:{})};
      };
      try{
        for(const[key,original]of originals)if(original)Object.defineProperty(prototype,key,{...original,value:function(...args){const packet=Reflect.apply(original.value,this,args);(key==='snapshot_source_layout'?snapshots:updates).push(packetRecord(packet));return packet;}});
        for(const workload of workloads){
          snapshots=[];updates=[];let model;
          try{
            model=renderer.parse(workload.source,workload.options);
            const expected=name==='candidate'&&workload.family==='sankey'?'PLCSRC02':null;
            if(name==='candidate'){
              if(snapshots.length!==1||snapshots[0].capability!==expected)throw Error(name+'/'+workload.id+': wrong actual snapshot route');
              if(expected&&snapshots[0].type!=='sankey')throw Error('Wrong recipe type metadata');
            }else if(snapshots.length)throw Error('Control used source layout capability');
            const rendered=renderer.render(model,{measurer:{width(text,size){return text.length*size*.6;}}});
            if(typeof rendered.svg!=='string'||typeof rendered.width!=='number'||typeof rendered.height!=='number')throw Error('Public retained render did not finish');
            if(expected){
              if(updates.length!==1||updates[0].capability!=='PLCSRU01'||updates[0].layoutId!==snapshots[0].layoutId||BigInt(updates[0].revision)!==BigInt(snapshots[0].revision)+1n)throw Error('Wrong retained update identity/revision');
            }else if(updates.length)throw Error('Non-source-layout model used update capability');
            checks.push({runtime:name,case:workload.id,modelType:model.type,expected,actual:snapshots[0]?.capability??null,calls:snapshots.length,metadata:snapshots[0]??null,update:updates[0]??null,updateCalls:updates.length,publicRetainedRender:true});
          }finally{if(model)renderer.dispose(model);}
        }
      }finally{
        try{
          for(const[key,original]of originals)if(original)Object.defineProperty(prototype,key,original);
          if(Constructor.prototype!==prototype||Object.getPrototypeOf(prototype)!==parent||Reflect.ownKeys(prototype).length!==keys.length||Reflect.ownKeys(prototype).some((key,i)=>key!==keys[i]))throw Error('Native prototype identity changed');
          for(const[key,original]of originals){const restored=Object.getOwnPropertyDescriptor(prototype,key);if(original?Reflect.ownKeys(original).some(k=>original[k]!==restored?.[k])||Reflect.ownKeys(original).length!==Reflect.ownKeys(restored).length:restored!==undefined)throw Error('Native method descriptor not restored');}
        }finally{renderer.close();}
      }
    }
    return{checks,prototypesRestored:true,retainedUpdateIdentitiesVerified:true};
  },{workloads,now});
  Object.assign(report,result);report.succeeded=true;
}catch(error){report.error={name:error.name,message:error.message,stack:error.stack};process.exitCode=1;}
finally{clearTimeout(deadline);await browser?.close();report.browserClosedBeforeTiming=true;await new Promise(done=>server.close(done));report.completedAt=new Date().toISOString();await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');}
