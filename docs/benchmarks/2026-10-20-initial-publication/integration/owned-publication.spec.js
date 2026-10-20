// Rejected candidate reproduction only: apply ../candidate.patch and rebuild first.
import {test,expect} from '@playwright/test';
test('packed publication matches legacy descriptors, numeric payloads and error brands and fallback behavior',async({page})=>{
  await page.goto('/examples/web/');
  const failures=await page.evaluate(async()=>{
    const {ModelBridge,ModelRegistry}=await import('/dist/package-web/package/graph.js');
    const {encodeGraphBinary,decodeGraphBinary,indexGraphBinary}=await import('/dist/package-web/package/graph-binary.js');
    const schema={id:'packed/equivalence',version:1,records:[],prototypes:[{id:4,name:'Item',parent:{kind:'object'},members:[]}],methods:[{id:0,name:'method',arity:0}]};
    const strings=['nan','zero','raw','child','__proto__','length','1','map','set','date','instance','method','message','text','\ud800A\udc00','same','host'];
    const ref=value=>({kind:'ref',value}),num=value=>({kind:'number',value}),str=value=>({kind:'string',value});
    const prop=(key,value,flags=7)=>({key,value,flags});
    const graph={version:1,root:ref(0),strings:strings.map(s=>Array.from({length:s.length},(_,i)=>s.charCodeAt(i))),nodes:[
      {kind:'object',properties:[prop(0,num('7ff8000000000000')),prop(1,num('8000000000000000')),prop(2,str(14)),prop(3,ref(1)),prop(4,ref(0)),prop(7,ref(2)),prop(8,ref(3)),prop(9,ref(4)),prop(10,ref(5)),prop(11,{kind:'builtinFunction',value:0})]},
      {kind:'array',length:3,properties:[prop(6,ref(0)),prop(5,num('4008000000000000'),0)]},
      {kind:'map',entries:[[str(13),ref(0)]],properties:[]},
      {kind:'set',values:[ref(0)],properties:[]},
      {kind:'date',milliseconds:'409f400000000000',properties:[]},
      {kind:'instance',prototype:4,properties:[prop(15,{kind:'bool',value:true})]}
    ]};
    const bytes=encodeGraphBinary('7',graph),index=indexGraphBinary(bytes),legacy=new ModelBridge(decodeGraphBinary(bytes).graph,schema,new ModelRegistry(),()=>{}),packed=new ModelBridge(index,schema,new ModelRegistry(),()=>{}),failures=[];
    const bits=new DataView(new ArrayBuffer(8));
    const atom=(bridge,v)=>{if(typeof v==='number'){bits.setFloat64(0,v);return ['number',bits.getBigUint64(0).toString(16)];}if(bridge.ids.has(v))return ['ref',bridge.ids.get(v)];if(typeof v==='function')return ['function',bridge.registration.functionIds.get(v)];return [typeof v,v];};
    const describe=bridge=>bridge.objects.map((o,id)=>({props:Object.getOwnPropertyNames(o).filter(k=>!['stack','fileName','lineNumber','columnNumber','sourceURL'].includes(k)).map(k=>{const d=Object.getOwnPropertyDescriptor(o,k);return[k,d.enumerable,d.writable,d.configurable,atom(bridge,d.value)];}),entries:o instanceof Map?[...o].map(([k,v])=>[atom(bridge,k),atom(bridge,v)]):o instanceof Set?[...o].map(v=>atom(bridge,v)):null,date:o instanceof Date?atom(bridge,o.getTime()):null,proto:bridge.registration.prototypeIds.get(Object.getPrototypeOf(o))??Object.getPrototypeOf(o)?.constructor?.name}));
    if(JSON.stringify(describe(legacy))!==JSON.stringify(describe(packed)))failures.push('initial descriptors');
    if(!packed.matchesPublishedGraph())failures.push('unchanged packed scan');
    for(const bridge of [legacy,packed]){bridge.root.nan=NaN;bridge.root.zero=0;bridge.root.raw='changed';bridge.root.transient=true;bridge.root.map.clear();bridge.root.set.clear();bridge.root.date.setTime(2);}
    if(packed.matchesPublishedGraph())failures.push('mutation missed');
    legacy.reconcile(decodeGraphBinary(bytes).graph);packed.reconcile(index);
    if(JSON.stringify(describe(legacy))!==JSON.stringify(describe(packed))||!packed.matchesPublishedGraph())failures.push('restored payloads');
    // Retained detached objects remain part of comparison and publication.
    for(const bridge of [legacy,packed])delete bridge.root.child;if(packed.matchesPublishedGraph())failures.push('removed alias missed');legacy.reconcile(decodeGraphBinary(bytes).graph);packed.reconcile(index);
    Object.defineProperty(packed.objects[5],'same',{get(){throw Error('getter invoked');},configurable:true});
    if(packed.matchesPublishedGraph())failures.push('accessor missed');packed.reconcile(index);
    for(const errorType of ['error','pele','type','range','syntax','reference','eval','uri'])graph.nodes.push({kind:'error',errorType,properties:[prop(12,str(13))]});
    graph.nodes[0].properties.push(prop(16,{kind:'hostReference',value:0}));
    for(const bridge of [legacy,packed]){bits.setBigUint64(0,0x7ff8000000000042n);bridge.hostValues[0]=bits.getFloat64(0);}
    const errorBytes=encodeGraphBinary('8',graph);legacy.reconcile(decodeGraphBinary(errorBytes).graph);packed.reconcile(indexGraphBinary(errorBytes));
    // A second publication exercises descriptor comparison for hostReference NaNs.
    legacy.reconcile(decodeGraphBinary(errorBytes).graph);packed.reconcile(indexGraphBinary(errorBytes));
    if(JSON.stringify(describe(legacy))!==JSON.stringify(describe(packed)))failures.push('error brands or host NaN');
    const special={version:1,root:{kind:'ref',value:0},strings:[[110]],nodes:[{kind:'object',properties:[prop(0,num('7ff8000000000042'))]}]};
    const specialBytes=encodeGraphBinary('9',special),specialIndex=indexGraphBinary(specialBytes),specialBridge=new ModelBridge(specialIndex,schema,new ModelRegistry(),()=>{});
    if(specialBridge.graph.nodes[0].properties[0].value.value!=='7ff8000000000042')failures.push('wire payload');
    const matched=specialBridge.matchesPublishedGraph();bits.setFloat64(0,Object.getOwnPropertyDescriptor(specialBridge.root,'n').value);
    if(matched&&bits.getBigUint64(0)!==0x7ff8000000000042n)failures.push('false NaN acknowledgement');
    // JS storage may canonicalize NaNs. Distinct observable bits must never be
    // acknowledged as unchanged, even when both values satisfy Number.isNaN.
    specialBridge.root.n=NaN;if(specialBridge.matchesPublishedGraph())failures.push('canonical NaN edit missed');
    const before=packed.root.nan;bytes.fill(0);if(!Object.is(packed.root.nan,before)||index.view.getUint8(0)!==80)failures.push('defensive general input');
    if(JSON.stringify(packed.graph)!==JSON.stringify(decodeGraphBinary(errorBytes).graph))failures.push('legacy fallback full graph');
    for(const readonly of [false,true]){
      const duplicate={version:1,root:ref(0),strings:[[100],[100]],nodes:[{kind:'object',properties:[prop(0,{kind:'bool',value:true},readonly?0:7),prop(1,{kind:'bool',value:false})]}]};
      const bytes=encodeGraphBinary('10',duplicate);
      const publish=decode=>{try{const bridge=new ModelBridge(decode(bytes),schema,new ModelRegistry(),()=>{});return{shape:describe(bridge),matches:bridge.matchesPublishedGraph(),graph:bridge.graph};}catch(error){return{name:error.name,message:error.message};}};
      if(JSON.stringify(publish(bytes=>decodeGraphBinary(bytes).graph))!==JSON.stringify(publish(indexGraphBinary)))failures.push('duplicate key/descriptor '+readonly);
    }
    return failures;
  });
  expect(failures).toEqual([]);
});

test('packed validation preserves malformed, truncation and budget error behavior before publication',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {encodeGraphBinary,decodeGraphBinary,indexGraphBinary}=await import('/dist/package-web/package/graph-binary.js');
    const g={version:1,root:{kind:'ref',value:0},strings:[[120],[0xd800,65,0xdc00]],nodes:[{kind:'object',properties:[{key:0,flags:7,value:{kind:'string',value:1}}]},{kind:'set',values:[{kind:'ref',value:0}],properties:[]}]};
    const bytes=encodeGraphBinary('0',g,new TextEncoder().encode('{"Ok":true}')),indexed=indexGraphBinary(bytes),cases=[];
    for(let n=0;n<bytes.length;n++)cases.push(bytes.slice(0,n));
    for(let offset=0;offset<bytes.length;offset++){const bad=bytes.slice();bad[offset]=255;cases.push(bad);}
    const mixed=bytes.slice(),v=new DataView(mixed.buffer);v.setUint32(indexed.propertyOffsets[0]+4,0xffffffff,true);v.setUint32(indexed.nodeOffsets[1]+1,8000001,true);cases.push(mixed);
    const error=decode=>input=>{try{decode(input);return null;}catch(e){return [e.name,e.code,e.message];}};
    const legacy=error(decodeGraphBinary),packed=error(indexGraphBinary),failures=[];
    for(let i=0;i<cases.length;i++){const a=legacy(cases[i]),b=packed(cases[i]);if(JSON.stringify(a)!==JSON.stringify(b))failures.push({i,a,b});}
    const copy=bytes.slice();indexGraphBinary(copy);copy.fill(0);
    const lengths=[0,1,127,128,129,255,256,257,8191,8192,8193,16385];
    for(const length of lengths){
      const graph={version:1,root:{kind:'string',value:0},strings:[Array.from({length},(_,i)=>i%3===0?0xd800:i%3===1?0xdc00:0)],nodes:[]};
      const bytes=encodeGraphBinary('1',graph),frame=indexGraphBinary(bytes),decoded=decodeGraphBinary(bytes);
      if(frame.strings[0].length!==length||!decoded.graph.strings[0].every((v,i)=>v===frame.strings[0].charCodeAt(i)))failures.push({length});
    }
    // Invalid flags must not hide a truncated atom read; references must not
    // hide later structural/budget errors. Every prefix tests both phases.
    const badFlags=bytes.slice();badFlags[indexed.propertyOffsets[0]+8]=255;
    for(let n=0;n<=badFlags.length;n++){const cut=badFlags.slice(0,n);const a=legacy(cut),b=packed(cut);if(JSON.stringify(a)!==JSON.stringify(b))failures.push({flagsPrefix:n,a,b});}

    return {count:cases.length,failures};
  });
  expect(report.count).toBeGreaterThan(100);expect(report.failures).toEqual([]);
});
