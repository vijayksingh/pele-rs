import{test,expect}from'@playwright/test';
import{readFile,writeFile}from'node:fs/promises';
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
test('Swimlane resource callbacks preserve copied graph phases, late metadata, original errors and nested recovery',async({page},testInfo)=>{
 const url='/dist/evidence/swimlane-live-scenarios.js',body=await readFile(new URL('./support/swimlane-live-scenarios.js',import.meta.url),'utf8');
 await page.route(`**${url}`,route=>route.fulfill({status:200,contentType:'text/javascript',body}));await page.goto('/examples/web/');
 const report=await page.evaluate(async({url,pin})=>{
  const hash=async path=>{const r=await fetch(path);if(!r.ok)throw Error('Missing '+path);return[...new Uint8Array(await crypto.subtle.digest('SHA-256',await r.arrayBuffer()))].map(n=>n.toString(16).padStart(2,'0')).join('');};
  const paths=['/dist/evidence/pele-oracle.js','/dist/package-web/package/full.js','/dist/package-web/package/pkg/pele_bg.wasm',url],before=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));if(before[paths[0]]!==pin)throw Error('Oracle changed');
  const{loadRenderer}=await import('/dist/package-web/package/full.js');const{loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const oracle=await import('/dist/evidence/pele-esm/core.js'),expectedEntry=(await import('/dist/evidence/pele-esm/diagrams/swimlane.js')).default;oracle.register(expectedEntry);
  const entry=(await import('/dist/package-web/package/diagrams/swimlane.js')).default,{SOURCE,MODES,runSwimlane}=await import(url),engine=await loadRenderer(),rows=[];
  const fixture=await(await fetch('/fixtures/public-swimlane-live-render.json')).json();
  try{for(const raw of[true,false])for(const responsive of[false,true])for(const mode of MODES)rows.push({raw,responsive,mode,oracle:runSwimlane(oracle,expectedEntry,()=>{},SOURCE,raw,responsive,mode),candidate:runSwimlane(engine,entry,m=>raw?disposeDiagramModel(m):engine.dispose(m),SOURCE,raw,responsive,mode),fixture:fixture.rows.find(r=>r.raw===raw&&r.responsive===responsive&&r.mode===mode)});}finally{engine.close();}
  const after=Object.fromEntries(await Promise.all(paths.map(async p=>[p,await hash(p)])));return{before,after,rows};
 },{url,pin});
 const path=testInfo.outputPath('swimlane-live-render.json');await writeFile(path,JSON.stringify(report,null,2));await testInfo.attach('paired-live-render',{path,contentType:'application/json'});expect(report.after).toEqual(report.before);expect(report.rows).toHaveLength(60);
 for(const row of report.rows){const id=`${row.raw}/${row.responsive}/${row.mode}`;expect(row.oracle,id+'/pinned').toEqual(row.fixture);expect(row.candidate,id).toEqual(row.oracle);}
});

test('Swimlane expanded class and default-edge styles reject before drawing copies and recover',async({page})=>{
 await page.goto('/examples/web/');const reports=await page.evaluate(async()=>{
  const{loadRenderer}=await import('/dist/package-web/package/full.js');const engine=await loadRenderer(),source='swimlane-beta LR\nA[Alpha] -->|first edge| B[Beta]\nB --> C[Gamma]\nstyle B fill:#113355\nclassDef custom color:#224466\nclass C custom';const reports=[];
  try{for(const mode of ['compiled','edge']){
   const model=engine.parse(source),classDef=model.classes.get('custom'),originalStyles=classDef.styles,originalDefault=model.defaultEdgeStyle,originalClasses=[...model.nodes].map(([id,node])=>[id,node.classes]);
   const pure={maxWidth:200,measurer:{width:(t,s)=>t.length*s*.6}},before=engine.render(model,pure);let fired=false,error;
   try{engine.render(model,{maxWidth:200,measurer:{width(t,s){if(!fired&&t==='Alpha'){fired=true;if(mode==='compiled'){classDef.styles=['x'.repeat(6000000)];for(const node of model.nodes.values())node.classes=['custom'];}else model.defaultEdgeStyle=['x'.repeat(9000000)];}return t.length*s*.6;}}});}catch(e){error={name:e.name,code:e.code};}
   const retained=mode==='compiled'?classDef.styles[0].length===6000000:model.defaultEdgeStyle[0].length===9000000;
   classDef.styles=originalStyles;model.defaultEdgeStyle=originalDefault;for(const[id,classes]of originalClasses)model.nodes.get(id).classes=classes;
   const after=engine.render(model,pure);reports.push({mode,fired,error,retained,recovered:before.svg===after.svg&&Object.is(before.width,after.width)&&Object.is(before.height,after.height)});engine.dispose(model);
  }}finally{engine.close();}return reports;
 });expect(reports).toHaveLength(2);for(const row of reports){expect(row.fired,row.mode).toBe(true);expect(row.error?.code,row.mode).toBe('limit');expect(row.retained,row.mode).toBe(true);expect(row.recovered,row.mode).toBe(true);}
});
