// CPU profiles diagnose hot paths; web-vm.mjs remains the latency acceptance gate.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {cpus,platform,arch,loadavg} from 'node:os';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {cases,now} from './cases.mjs';
const args=process.argv.slice(2),arg=(k,d)=>args.includes(k)?args[args.indexOf(k)+1]:d;
const packageDir=resolve(arg('--package','dist/package-web/package')),output=resolve(arg('--output','dist/evidence/profiles'));
const selected=arg('--cases','flowchart/large,er/large,gantt/large').split(','),duration=Number(arg('--duration-ms','1500'));
assert(duration>=500&&duration<=10000);assert(selected.every(id=>cases().some(c=>c.id===id)));
const oracle=resolve('node_modules/pele/dist/pele.js'),hash=b=>createHash('sha256').update(b).digest('hex');
const hashes=async()=>Object.fromEntries(await Promise.all(['index.js','full.js','initialize.js','graph.js','graph-binary.js','resources.js','pkg/pele.js','pkg/pele_bg.wasm'].map(async name=>[name,hash(await readFile(resolve(packageDir,name)))])));
const before=await hashes();
assert.equal(hash(await readFile(oracle)),'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const server=createServer(async(req,res)=>{try{
 const path=new URL(req.url,'http://localhost').pathname;
 if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>CPU profile</title>');return;}
 const file=path==='/oracle.js'?oracle:resolve(packageDir,'.'+path.replace(/^\/package/,''));
 if(path!=='/oracle.js'&&(!path.startsWith('/package/')||!file.startsWith(packageDir+'/'))){res.writeHead(404);res.end();return;}
 res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));
}catch{res.writeHead(404);res.end();}});
await mkdir(output,{recursive:true});await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser;
const deadline=setTimeout(()=>void browser?.close(),180000);
try{
 browser=await chromium.launch({headless:true});const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 await page.evaluate(async now=>{Date.now=()=>now;const api=await import('/package/index.js'),oracle=await import('/oracle.js');window.bench={api,oracle,engine:await api.loadRenderer()};},now);
 const cdp=await page.context().newCDPSession(page);await cdp.send('Profiler.enable');await cdp.send('Profiler.setSamplingInterval',{interval:250});
 const summary={browser:browser.version(),wasmSHA256:before['pkg/pele_bg.wasm'],runtimeHashes:before,harnessSHA256:hash(await readFile(fileURLToPath(import.meta.url))),casesSHA256:hash(await readFile(new URL('./cases.mjs',import.meta.url))),host:{platform:platform(),arch:arch(),cpu:cpus()[0]?.model,logicalCPUs:cpus().length,loadBefore:loadavg()},samplingIntervalUs:250,durationMs:duration,profiles:[]};
 for(const workload of cases().filter(c=>selected.includes(c.id)))for(const stage of ['render-source','parse','render-model']){
  await page.evaluate(({workload,stage})=>{
   const {api,oracle,engine}=window.bench,measurer=api.canvasMeasurer('sans-serif','monospace'),options={...workload.options,measurer};
   const a=engine.render(workload.source,options),b=oracle.render(workload.source,options);
   if(a.svg!==b.svg||!Object.is(a.width,b.width)||!Object.is(a.height,b.height))throw Error('Output mismatch');
   const model={js:oracle.parse(workload.source,options),wasm:engine.parse(workload.source,options)};
   window.profileWork={checksum:0,run(runtime,ms){const renderer=runtime==='js'?oracle:engine,start=performance.now();let operations=0;
    do{if(stage==='parse'){const value=renderer.parse(workload.source,options);this.checksum+=value.type.length;if(runtime==='wasm')engine.dispose(value);}else{const value=renderer.render(stage==='render-source'?workload.source:model[runtime],options);this.checksum+=value.svg.length;}operations++;}while(performance.now()-start<ms);return{operations,checksum:this.checksum};},verify(){const a=engine.render(model.wasm,options),b=oracle.render(model.js,options);if(a.svg!==b.svg||!Object.is(a.width,b.width)||!Object.is(a.height,b.height))throw Error('Post-profile output mismatch');},close(){engine.dispose(model.wasm)}};
   window.profileWork.run('js',200);window.profileWork.run('wasm',200);
  },{workload,stage});
  for(const runtime of ['js','wasm']){
   await cdp.send('Profiler.start');const exercised=await page.evaluate(({runtime,duration})=>window.profileWork.run(runtime,duration),{runtime,duration});
   const {profile}=await cdp.send('Profiler.stop');const label=workload.id.replace('/','-')+'-'+stage+'-'+runtime;
   await writeFile(resolve(output,label+'.cpuprofile'),JSON.stringify(profile));
   const byId=new Map(profile.nodes.map(n=>[n.id,n])),self=new Map();
   for(let i=0;i<(profile.samples?.length??0);i++){const n=byId.get(profile.samples[i]),name=n.callFrame.functionName||n.callFrame.url||'(anonymous)';self.set(name,(self.get(name)||0)+(profile.timeDeltas?.[i]??250));}
   const total=[...self.values()].reduce((a,b)=>a+b,0),top=[...self].sort((a,b)=>b[1]-a[1]).slice(0,35).map(([name,us])=>({name,percent:100*us/total}));
   summary.profiles.push({case:workload.id,stage,runtime,...exercised,top});console.log(label,JSON.stringify(top.slice(0,10)));
  }
  await page.evaluate(()=>{window.profileWork.verify();window.profileWork.close();});
 }
 assert.deepEqual(await hashes(),before,'Package changed during profiling');summary.host.loadAfter=loadavg();
 await writeFile(resolve(output,'summary.json'),JSON.stringify(summary,null,2)+'\n');
}finally{clearTimeout(deadline);await browser?.close();await new Promise(r=>server.close(r));}
