import {test,expect} from '@playwright/test';
import {readFile,writeFile} from 'node:fs/promises';
const source=(await readFile(new URL('./support/constructor-feature.js',import.meta.url),'utf8'))+'\n'+(await readFile(new URL('./support/ordered-constructor-body.js',import.meta.url),'utf8'));
async function run(page,name){await page.goto('/examples/web/');return page.evaluate(async({source,name})=>{const url=URL.createObjectURL(new Blob([source],{type:'text/javascript'}));try{return await(await import(url))[name]();}finally{URL.revokeObjectURL(url);}},{source,name});}
async function record(info,name,value){const path=info.outputPath(name);await writeFile(path,JSON.stringify(value,null,2));await info.attach(name,{path,contentType:'application/json'});}
test('ordered constructor bodies preserve live option reads, first helpers and subclass receivers',async({page},info)=>{
  const rows=await run(page,'orderedBodies');await record(info,'ordered-bodies.json',rows);expect(rows).toHaveLength(3);
  for(const row of rows){expect(row.candidate,row.family).toEqual(row.oracle);expect(row.candidate.initial,row.family).toEqual(row.fixture.initial);expect(row.candidate.populated,row.family).toEqual(row.fixture.populated);expect(row.candidate.constructorCases,row.family).toEqual(row.fixture.constructorCases);expect(row.candidate.arity).toBe(0);expect(row.candidate.subclass.instance).toBe(true);expect(row.candidate.subclass.prototype).toBe(true);}
});
test('ClassMember constructor evaluates native argument metadata and retains overridden parser receivers',async({page},info)=>{
  const row=await run(page,'memberBodies');await record(info,'class-member-bodies.json',row);expect(row.candidate).toEqual(row.oracle);expect(row.candidate.cases).toEqual(row.fixture.cases);expect(row.candidate.arity).toBe(2);expect(row.candidate.virtual).toHaveLength(6);for(const v of row.candidate.virtual)if(!v.ok)expect(v.original).toBe(true);
});
