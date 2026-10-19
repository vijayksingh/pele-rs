import {test,expect} from '@playwright/test';

test('SVG root output preserves pinned icons, radius, aliases, fonts and accessibility',async({page})=>{
  await page.goto('/examples/web/');
  const failures=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const engine=await loadRenderer(),fixture=await(await fetch('/fixtures/pele/svg-root.json')).json(),failures=[];
    for(const c of fixture.cases){
      const options={...c.options,now:1704067200000,measurer:{width:(t,z,s)=>t.length*z*(s&1?.71:.6)},icons:()=>c.markup};
      const model=engine.parse(c.source,options);
      try{for(const input of [c.source,model]){
        const result=engine.render(input,options);
        if(result.svg!==c.result.svg||result.type!==c.result.type||!Object.is(result.width,c.result.width)||!Object.is(result.height,c.result.height)||JSON.stringify(result.links)!==JSON.stringify(c.result.links)||Object.keys(result).sort().join()!==Object.keys(c.result).sort().join())failures.push(c.id+(typeof input==='string'?'/source':'/mutable'));
      }}finally{engine.dispose(model)}
    }
    return failures;
  });
  expect(failures).toEqual([]);
});
