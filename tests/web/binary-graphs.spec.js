import {test,expect} from '@playwright/test';

test('binary WASM consumers retain edited graph identities and reject malformed exchanges',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer,metricsMeasurer}=await import('/dist/package-web/package/full.js');await loadRenderer();
    const wasm=await import('/dist/package-web/package/pkg/pele.js');
    const {decodeGraphBinary,encodeGraphBinary}=await import('/dist/package-web/package/graph-binary.js');
    const parsed=wasm.parse_v2('sankey\nA,B,5','{}'),document=parsed.to_public();parsed.free();
    const envelope=json=>JSON.parse(json),read=()=>decodeGraphBinary(document.snapshot_binary());
    try{
      const first=read(),graph=first.graph,root=graph.nodes[graph.root.value];
      const intern=units=>{graph.strings.push(units);return graph.strings.length-1;};
      const key=intern([101,120,116,114,97]),raw=intern([0xd800,65,0xdc00]),date=graph.nodes.length;
      graph.nodes.push({kind:'date',milliseconds:'7ff8000000000042',properties:[]});
      const object=graph.nodes.length,next=intern([110,101,120,116]);
      graph.nodes.push({kind:'object',properties:[{key:next,flags:7,value:{kind:'ref',value:object}},{key:raw,flags:5,value:{kind:'ref',value:date}}]});
      root.properties.push({key,flags:7,value:{kind:'ref',value:object}});
      const upload=encodeGraphBinary(first.revision,graph);
      const malformed=envelope(document.apply_binary(upload.subarray(0,20)));
      const before=read().revision,applied=envelope(document.apply_binary(upload)),stale=envelope(document.apply_binary(upload));
      const after=read(),result=decodeGraphBinary(document.render_operation_binary(after.revision,'{}',metricsMeasurer.width,undefined));
      const rendered=result.metadata.Ok,failed=decodeGraphBinary(document.render_operation_binary(result.revision,'{"outputLimit":1}',metricsMeasurer.width,undefined));
      return{type:first.metadata.type,before,applied:applied.result,stale:stale.error.message,malformed:malformed.error.message,revision:result.revision,raw:after.graph.strings[raw],cycle:after.graph.nodes[object].properties[0].value.value===object,date:after.graph.nodes[date].milliseconds,rendered:rendered.type,failed:failed.metadata.Err.value.code};
    }finally{document.free();}
  });
  expect(report).toEqual({type:'sankey',before:'0',applied:'1',stale:'Stale public model revision',malformed:'Invalid public binary graph transport',revision:'2',raw:[0xd800,65,0xdc00],cycle:true,date:'7ff8000000000042',rendered:'sankey',failed:'limit'});
});

test('binary graph publication preserves descriptors, special keys and cyclic references',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {ModelBridge,ModelRegistry}=await import('/dist/package-web/package/graph.js');
    const {encodeGraphBinary,decodeGraphBinary}=await import('/dist/package-web/package/graph-binary.js');
    const schema={id:'consumer/publication@1',version:1,records:[],prototypes:[],methods:[]};
    const strings=['__proto__','same','length','child','code'].map(s=>Array.from(s,c=>c.charCodeAt(0)));
    const property=(key,value,flags=7)=>({key,value,flags}),ref=value=>({kind:'ref',value});
    const graph={version:1,root:ref(0),strings,nodes:[
      {kind:'object',properties:[property(0,ref(0)),property(1,{kind:'bool',value:false},4),property(1,{kind:'bool',value:true}),property(3,ref(1))]},
      {kind:'array',length:2,properties:[property(2,{kind:'number',value:'4000000000000000'},0)]},
      {kind:'error',errorType:'pele',properties:[]}
    ]};
    const exchange=decodeGraphBinary(encodeGraphBinary('0',graph));
    const bridge=new ModelBridge(exchange.graph,schema,new ModelRegistry(),()=>{}),root=bridge.root;
    const initial={ownProto:Object.hasOwn(root,'__proto__'),proto:Object.getPrototypeOf(root)===Object.prototype,cycle:root.__proto__===root,duplicate:root.same,readonly:!Object.getOwnPropertyDescriptor(root.child,'length').writable,holes:!(0 in root.child),errorCode:Object.hasOwn(bridge.objects[2],'code')};
    root.same=false;Object.defineProperty(root,'child',{get(){throw Error('must not invoke getter');},configurable:true});
    bridge.reconcile(exchange.graph);
    const restored=root.same===true&&root.child===bridge.objects[1];
    const invalidLength=structuredClone(graph);invalidLength.nodes[1].properties[0].flags=7;
    let rejectedLength=false;
    try{new ModelBridge(decodeGraphBinary(encodeGraphBinary('0',invalidLength)).graph,schema,new ModelRegistry(),()=>{});}catch(error){rejectedLength=error instanceof TypeError;}
    return {initial,restored,rejectedLength};
  });
  expect(report).toEqual({initial:{ownProto:true,proto:true,cycle:true,duplicate:true,readonly:true,holes:true,errorCode:false},restored:true,rejectedLength:true});
});
