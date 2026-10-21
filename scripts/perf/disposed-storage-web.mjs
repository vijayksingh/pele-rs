// Instrument retained transport references after a same-job public parse/dispose batch.
// Counts graph units and references, never requested heap bytes, RSS or timing.
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {resolve,dirname,extname} from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {chromium} from '@playwright/test';
import {cases,revision,now} from './cases.mjs';
const arg=(name,fallback)=>{const at=process.argv.indexOf(name);return at<0?fallback:process.argv[at+1]};
const roots={control:resolve(arg('--control','dist/package-web/disposal-control')),candidate:resolve(arg('--candidate','dist/package-web/package'))};
assert.notEqual(roots.control,roots.candidate);
const count=Number(arg('--count','200'));assert(Number.isInteger(count)&&count>0&&count<=512);
const workloads=cases().filter(c=>['flowchart/medium','er/medium','railroad/medium'].includes(c.id));assert(workloads.every(c=>c.units*count<=25000));
const output=resolve(arg('--output','dist/evidence/disposed-storage-web.json')),oracle=resolve('node_modules/pele/dist/pele.js'),sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const inputs=async()=>Object.fromEntries(await Promise.all(Object.entries(roots).map(async([runtime,root])=>[runtime,Object.fromEntries(await Promise.all(['index.js','full.js','initialize.js','graph.js','graph-binary.js','resources.js','pkg/pele.js','pkg/pele_bg.wasm'].map(async name=>[name,sha(await readFile(resolve(root,name)))])))])));
const before=await inputs(),oracleSHA256=sha(await readFile(oracle));assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const report={schemaVersion:1,startedAt:new Date().toISOString(),oracleRevision:revision,oracleSHA256,harnessSHA256:sha(await readFile(fileURLToPath(import.meta.url))),runtimeHashes:before,count,scope:'One synchronous public parse/dispose job per package/workload. Instrument ModelBridge publication using weak references; count disposed bridges and transport records still attached immediately afterward. WeakRef construction already keeps its target alive until job end. No forced GC, timing or heap/RSS claim. Caller-owned model liveness is separate.',rows:[]};
const server=createServer(async(req,res)=>{try{const path=decodeURIComponent(new URL(req.url,'http://localhost').pathname);if(path==='/'){res.setHeader('Content-Type','text/html');res.end('<!doctype html><title>Disposed storage census</title>');return;}let file;if(path==='/oracle.js')file=oracle;else{const runtime=Object.keys(roots).find(name=>path.startsWith('/'+name+'/'));assert(runtime);file=resolve(roots[runtime],'.'+path.slice(runtime.length+1));assert(file.startsWith(roots[runtime]+'/'));}res.setHeader('Content-Type',extname(file)==='.wasm'?'application/wasm':'text/javascript');res.end(await readFile(file));}catch{res.writeHead(404);res.end();}});
await new Promise(r=>server.listen(0,'127.0.0.1',r));let browser;const deadline=setTimeout(()=>void browser?.close(),180000);
try{
 browser=await chromium.launch({headless:true,timeout:20000});report.browser=browser.version();const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.address().port}/`);
 for(const workload of workloads)for(const runtime of ['control','candidate']){
 const result=await page.evaluate(async({runtime,workload,count,now})=>{
  Date.now=()=>now;const api=await import('/'+runtime+'/index.js'),oracle=await import('/oracle.js'),{ModelBridge}=await import('/'+runtime+'/graph.js'),engine=await api.loadRenderer();
  const options={...workload.options,measurer:{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}};
  const actual=engine.render(workload.source,options),expected=oracle.render(workload.source,options);
  if(actual.svg!==expected.svg||!Object.is(actual.width,expected.width)||!Object.is(actual.height,expected.height))throw Error('Whole output mismatch');
  const refs=[],seen=new WeakSet(),original=ModelBridge.prototype.reconcile;
  ModelBridge.prototype.reconcile=function(...args){if(!seen.has(this)){seen.add(this);refs.push(new WeakRef(this))}return original.apply(this,args)};
  let checksum=0;
  try{for(let i=0;i<count;i++){const model=engine.parse(workload.source,options);checksum+=model.type.length;engine.dispose(model)}}finally{ModelBridge.prototype.reconcile=original}
  let bridges=0,disposedBridges=0,graphs=0,nodes=0,properties=0,stringRecords=0,utf16Units=0,hostObjects=0;const invokeTargets=new Set();
  for(const ref of refs){const bridge=ref.deref();if(!bridge)continue;bridges++;if(bridge.disposed)disposedBridges++;if(bridge.invoke)invokeTargets.add(bridge.invoke);hostObjects+=bridge.objects?.length??0;const graph=bridge.graph;if(!graph)continue;graphs++;nodes+=graph.nodes.length;stringRecords+=graph.strings.length;for(const units of graph.strings)utf16Units+=units.length;for(const node of graph.nodes)properties+=node.properties.length;}
  if(bridges!==count||disposedBridges!==count)throw Error('Incomplete bridge census');
  return{case:workload.id,runtime,bridges,disposedBridges,retainedGraphs:graphs,retainedNodes:nodes,retainedProperties:properties,retainedStringRecords:stringRecords,retainedUTF16Units:utf16Units,retainedHostObjects:hostObjects,uniqueInvokeTargets:invokeTargets.size,checksum,svg:actual.svg};
 },{runtime,workload,count,now});
 report.rows.push({...result,svg:undefined,svgSHA256:sha(Buffer.from(result.svg))});console.log(runtime+' '+workload.id+' retained graphs: '+result.retainedGraphs);
 }
 assert.deepEqual(await inputs(),before,'Package changed during census');report.completedAt=new Date().toISOString();report.succeeded=true;await mkdir(dirname(output),{recursive:true});await writeFile(output,JSON.stringify(report,null,2)+'\n');console.log(output);
}finally{clearTimeout(deadline);await browser?.close();await new Promise(r=>server.close(r));}
