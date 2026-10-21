import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const fixtures=['public-next-six-constructors.json','public-state-constructor.json'];
const packageRoot=process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package';
const driverSource=(await readFile(new URL('./support/constructor-feature.js',import.meta.url),'utf8'))
  .replaceAll('/dist/package-web/package',packageRoot);
async function runFeature(page,method){
  // Serve allowlist excludes tests. Load this test-owned module without changing
  // the public package/server or introducing a runtime capability.
  return page.evaluate(async({source,paths,method})=>{
    const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
    try{return await(await import(url))[method](paths);}finally{URL.revokeObjectURL(url);}
  },{source:driverSource,paths:fixtures,method});
}
async function record(info,name,value){const path=info.outputPath(name);await writeFile(path,JSON.stringify(value,null,2));await info.attach(name,{path,contentType:'application/json'});}

// Construction behavior and captured ownership. Runtime names, non-new error
// messages and intrinsic key order remain unmodified diagnostics, not parity claims.
test('next seven captured classes preserve defaults, helper-first aliases, State extraction and raw outcomes',async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const report=await runFeature(page,'constructionFeature');
  await record(info,'next-constructors.json',report);
  expect(report.rows).toHaveLength(21);expect(report.metadata).toHaveLength(28);
  for(const m of report.metadata){
    expect(m.arity,m.family).toBe(0);expect(m.withoutNew.name,m.family).toBe('TypeError');
    expect(m.ownKeys,m.family).toHaveLength(3);expect(m.ownKeys,m.family).toEqual(expect.arrayContaining(['length','name','prototype']));
    expect(m.length).toEqual({writable:false,enumerable:false,configurable:true});expect(m.nameFlags).toEqual(m.length);
    expect(m.prototype).toEqual({writable:false,enumerable:false,configurable:false});expect(m.backlink).toEqual({writable:true,enumerable:false,configurable:true});expect(m.backlinkIdentity).toBe(true);
  }
  expect(report.sharedIdentity.candidate).toEqual(report.sharedIdentity.oracle);
  expect(report.sharedIdentity.candidate).toEqual({present:true,parsedSame:true,constructedSame:true,constructedCount:4});
  for(const row of report.rows){
    const context=row.family+'/'+row.mode;expect(row.candidate,context).toEqual(row.oracle);
    expect(row.candidate,context).toMatchObject({reads:0,writes:0,prototype:true,before:row.fixture.coreError,after:row.fixture.coreError});
    expect(row.candidate.initial.graph,context).toEqual(row.fixture.initial.graph);
    if(row.mode!=='helperFirstSubclass')expect(row.candidate.initial.render,context).toEqual(row.fixture.initial.render);
    expect(row.candidate.populated,context).toEqual(row.fixture.populated);
    for(let i=0;i<row.fixture.actions.length;i++){
      const wanted=row.fixture.actions[i],actual=row.candidate.actions[i];
      expect(actual.graph,context+'/'+i).toEqual(wanted.graph);
      if(wanted.draw)expect(actual.render,context+'/'+i).toEqual(wanted.render);
      if(wanted.sameAs)expect(actual.sameIdentity,context+'/'+i).toBe(true);
      if(wanted.differentFrom)expect(actual.differentIdentity,context+'/'+i).toBe(true);
    }
    if(row.family==='state'){
      const beforeExtract=row.candidate.actions.find((_,i)=>row.fixture.actions[i].draw).render;
      expect(beforeExtract,context+'/blank before extract').toEqual(row.fixture.initial.render);
      expect(row.candidate.populated.render.svg,context+'/extracted contents').not.toBe(beforeExtract.svg);
    }
  }
});

test('next seven constructors retain independent ownership across registry replacement and close',async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const rows=await runFeature(page,'constructorLifetimes');
  await record(info,'next-constructor-lifetimes.json',rows);expect(rows).toHaveLength(7);
  for(const row of rows){
    expect(row.actualActions,row.family).toEqual(row.expectedActions);expect(row.actual,row.family).toEqual(row.wanted);
    expect(row.yAfter,row.family).toEqual(row.yBefore);expect(row.freshAfter,row.family).toEqual(row.freshBefore);expect(row.independentObjects,row.family).toBe(true);
    expect(row.disposed,row.family).toEqual({name:'TypeError',message:'Public model document is disposed'});
    expect(row.later,row.family).toEqual(row.expectedLater);
  }
});
