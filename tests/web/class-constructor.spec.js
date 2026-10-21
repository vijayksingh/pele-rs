import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const packageRoot=process.env.PELE_TEST_PACKAGE_ROOT??'/dist/package-web/package';
const driver=(await readFile(new URL('./support/constructor-feature.js',import.meta.url),'utf8')).replaceAll('/dist/package-web/package',packageRoot);
// Declared ClassDb construction only. Shared enum realm identity/mutation and
// direct ClassMember construction remain explicit gaps, not asserted parity.
test('declared Class constructor preserves defaults, thirteen helpers, aliases and raw render',async({page},info)=>{
  test.setTimeout(120000);await page.goto('/examples/web/');
  const report=await page.evaluate(async source=>{
    const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));
    try{return await(await import(url)).constructionFeature(['public-class-constructor.json']);}finally{URL.revokeObjectURL(url);}
  },driver);
  const path=info.outputPath('class-constructor.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('class-constructor.json',{path,contentType:'application/json'});
  expect(report.rows).toHaveLength(3);expect(report.metadata).toHaveLength(4);
  for(const m of report.metadata){expect(m.arity).toBe(0);expect(m.backlinkIdentity).toBe(true);expect(m.ownKeys).toEqual(expect.arrayContaining(['length','name','prototype']));}
  for(const row of report.rows){
    expect(row.candidate,row.mode).toEqual(row.oracle);expect(row.candidate.reads).toBe(0);expect(row.candidate.writes).toBe(0);
    expect(row.candidate.initial.graph).toEqual(row.fixture.initial.graph);
    if(row.mode!=='helperFirstSubclass')expect(row.candidate.initial.render).toEqual(row.fixture.initial.render);
    expect(row.candidate.populated).toEqual(row.fixture.populated);
    expect(row.candidate.before).toEqual(row.fixture.coreError);expect(row.candidate.after).toEqual(row.fixture.coreError);
    for(let i=0;i<row.fixture.actions.length;i++){expect(row.candidate.actions[i].graph,row.mode+'/'+i).toEqual(row.fixture.actions[i].graph);if(row.fixture.actions[i].sameAs)expect(row.candidate.actions[i].sameIdentity).toBe(true);}
  }
});
