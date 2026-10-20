import {test,expect} from '@playwright/test';

test('Gantt parse preserves complete native identity tables',async({page})=>{
 await page.goto('/examples/web/');
 const report=await page.evaluate(async()=>{
  const {loadRenderer}=await import('/dist/package-web/package/full.js');await loadRenderer();
  const {NativeRegistry}=await import('/dist/package-web/package/pkg/pele.js');
  const {decodeGraphBinary,encodeGraphBinary}=await import('/dist/package-web/package/graph-binary.js');
  const fixture=await(await fetch('/fixtures/pele/gantt-construction.json')).json(),registry=new NativeRegistry(true),failures=[];
  try{for(const c of fixture.cases){
   const document=registry.parse(c.source,JSON.stringify(c.options),0,0,0,0);
   try{
    const {graph}=decodeGraphBinary(document.snapshot_binary()),bytes=encodeGraphBinary('0',graph);
    const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),n=>n.toString(16).padStart(2,'0')).join('');
    if(hash!==c.graphSHA256||graph.nodes.length!==c.nodes||graph.strings.length!==c.strings||bytes.length!==c.binaryBytes)failures.push(c.id);
   }finally{document.free();}
  }}finally{registry.free();}
  return{count:fixture.cases.length,failures};
 });
 expect(report).toEqual({count:10,failures:[]});
});
