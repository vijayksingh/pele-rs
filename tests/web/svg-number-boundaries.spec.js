import {test,expect} from '@playwright/test';

test('railroad complete SVG preserves coordinate rounding boundaries',async({page})=>{
  await page.goto('/examples/web/');
  const failures=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const fixture=await(await fetch('/fixtures/pele/svg-number-boundaries.json')).json();
    const engine=await loadRenderer(),data=new DataView(new ArrayBuffer(8)),failures=[];
    const number=hex=>{data.setBigUint64(0,BigInt('0x'+hex));return data.getFloat64(0)};
    for(const c of fixture.cases){
      const options={fontSize:13,maxWidth:220,padding:number(c.paddingBits),now:1704067200000,outputLimit:undefined,measurer:{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}};
      const model=engine.parse(fixture.source,options);
      try{for(const input of [fixture.source,model]){
        const result=engine.render(input,options);
        if(result.svg!==c.result.svg||result.type!==c.result.type||JSON.stringify(result.links)!==JSON.stringify(c.result.links)||Object.keys(result).sort().join()!==Object.keys(c.result).sort().join())failures.push(c.id+'/complete-output');
        for(const key of ['width','height']){const expected=number(c[key+'Bits']);if(!(Number.isNaN(result[key])&&Number.isNaN(expected))&&!Object.is(result[key],expected))failures.push(c.id+'/'+key)}
      }}finally{engine.dispose(model)}
    }
    return failures;
  });
  expect(failures).toEqual([]);
});
