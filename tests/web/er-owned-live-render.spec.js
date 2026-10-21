import{test,expect}from'@playwright/test';
import{readFile,writeFile}from'node:fs/promises';
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
test('Er resource callbacks preserve captured attributes, late fields, nested helpers and original throws',async({page},testInfo)=>{
 const url='/dist/evidence/er-owned-live-scenarios.js',body=await readFile(new URL('./support/er-owned-live-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,route=>route.fulfill({status:200,contentType:'text/javascript',body}));await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async path=>{const r=await fetch(path);if(!r.ok)throw Error('Missing '+path);return[...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));if(before[paths[0]]!==pin)throw Error('Oracle changed');
  const{loadRenderer}=await import('/dist/package-web/package/full.js');const{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const oracle=await import('/dist/evidence/pele-esm/core.js'),expectedEntry=(await import('/dist/evidence/pele-esm/diagrams/er.js')).default;oracle.register(expectedEntry);
  const entry=(await import('/dist/package-web/package/diagrams/er.js')).default,{MODES,runEr}=await import(url),engine=await loadRenderer(),rows=[];
  const fixture=await(await fetch('/fixtures/public-er-owned-live-render.json')).json();
  try{for(const raw of[true,false])for(const responsive of[false,true])for(const mode of MODES)rows.push({raw,responsive,mode,oracle:runEr(oracle,expectedEntry,()=>{},raw,responsive,mode),candidate:runEr(engine,entry,m=>raw?disposeDiagramModel(m):engine.dispose(m),raw,responsive,mode),fixture:fixture.rows.find(r=>r.raw===raw&&r.responsive===responsive&&r.mode===mode)});}finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return{before,after,rows};
 },{url,pin});
 const path=testInfo.outputPath('er-owned-live-render.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('paired-live-render',{path,contentType:'application/json'});expect(report.after).toEqual(report.before);expect(report.rows).toHaveLength(48);
 for(const row of report.rows){const id=`${row.raw}/${row.responsive}/${row.mode}`;expect(row.oracle,id+'/pinned').toEqual(row.fixture);expect(row.candidate,id).toEqual(row.oracle);}
});

// Native bounded-resource extension, not an oracle limit. The captured ER
// constructor uses the generic owned-job route rather than the existing web
// host-ER parser route. Repeated aliases must be counted before join allocates.
test('constructed ER bounds live joined attribute text before allocation and recovers',async({page},testInfo)=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async()=>{
  const{loadRenderer}=await import('/dist/package-web/package/full.js');
  const{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const raw=(await import('/dist/package-web/package/diagrams/er.js')).default,engine=await loadRenderer(),parsed=engine.parse('erDiagram\nA'),Ctor=parsed.constructor;engine.dispose(parsed);
  const model=new Ctor(),entity=model.addEntity('A'),row={type:'int',name:'id',keys:['PK'],comment:''};entity.attributes.push(row);
  let fired=false,error;
  const plain={measurer:{width:(text,size)=>text.length*size*.6}};
  try{
   const before=raw.render(model,{},plain),shared='x'.repeat(8192);
   try{raw.render(model,{}, {measurer:{width(text,size){if(!fired){fired=true;row.keys=Array(2049).fill(shared);}return text.length*size*.6;}}});}catch(e){error={name:e?.name,code:e?.code,message:e?.message};}
   const retained=row.keys.length===2049&&row.keys[0]===shared;
   row.keys=['PK'];const recovered=raw.render(model,{},plain);
   return{fired,error,retained,recovered:recovered.svg===before.svg&&Object.is(recovered.width,before.width)&&Object.is(recovered.height,before.height)};
  }finally{disposeDiagramModel(model);engine.close();}
 });
 const path=testInfo.outputPath('er-join-budget.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('er-join-budget',{path,contentType:'application/json'});
 expect(report.fired).toBe(true);expect(report.error?.code).toBe('limit');expect(report.retained).toBe(true);expect(report.recovered).toBe(true);
});
