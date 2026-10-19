import {test, expect} from '@playwright/test';

test('acknowledged public binary graphs compare exact retained host state and reject edits', async ({page}) => {
  await page.goto('/examples/web/');
  const report = await page.evaluate(async () => {
    const {ModelBridge,ModelRegistry} = await import('/dist/package-web/package/graph.js');
    const {encodeGraphBinary,decodeGraphBinary} = await import('/dist/package-web/package/graph-binary.js');
    const schema = {id:'consumer/direct-compare@1',version:1,records:[],
      prototypes:[{id:1,name:'Published',parent:{kind:'object'},members:[]}],
      methods:[{id:1,name:'method',arity:0}]};
    const names = ['__proto__','label','zero','nan','array','map','set','date','error','method','1','length'];
    const property = (name,value,flags=7) => ({key:names.indexOf(name),value,flags});
    const ref = value => ({kind:'ref',value}), number = value => ({kind:'number',value});
    const graph = {version:1,root:ref(0),strings:[...names.map(s=>Array.from(s,c=>c.charCodeAt(0))),[0xd800,0x61]],nodes:[
      {kind:'instance',prototype:1,properties:[property('__proto__',ref(0)),property('label',{kind:'string',value:names.length}),
        property('zero',number('8000000000000000')),property('nan',number('7ff8000000000000')),
        property('array',ref(1)),property('map',ref(2)),property('set',ref(3)),property('date',ref(4)),
        property('error',ref(5)),property('method',{kind:'builtinFunction',value:1})]},
      {kind:'array',length:4,properties:[property('1',ref(0)),property('length',number('4010000000000000'),0)]},
      {kind:'map',entries:[[ref(0),ref(1)],[ref(1),ref(0)]],properties:[]},
      {kind:'set',values:[ref(0),ref(1)],properties:[]},
      {kind:'date',milliseconds:'3ff0000000000000',properties:[]},
      {kind:'error',errorType:'type',properties:[]},
      // Detached arrays remain part of the retained public arena.
      {kind:'array',length:2,properties:[]}
    ]};
    const fresh = () => {
      const decoded = decodeGraphBinary(encodeGraphBinary('0',graph)).graph;
      const bridge = new ModelBridge(decoded,schema,new ModelRegistry(),()=>{});
      // Host-generated Error diagnostics and any VM numeric boxing become the
      // same acknowledged arena that the ordinary snapshot lane would publish.
      bridge.reconcile(bridge.snapshot().graph);
      return bridge;
    };
    const failures = [], checks = [];
    for (const [name, edit] of [
      ['unchanged',()=>{}],
      ['negative zero',b=>{b.root.zero=0;}],
      ['lone surrogate',b=>{b.root.label='\ud801a';}],
      ['alias',b=>{b.root.array={};}],
      ['detached array',b=>{b.objects[6][0]='detached edit';}],
      ['readonly array entry',b=>{b.objects[1][1]=undefined;}],
      ['descriptor',b=>{Object.defineProperty(b.root,'label',{writable:false});}],
      ['property order',b=>{const value=b.root.label;delete b.root.label;b.root.label=value;}],
      ['map order',b=>{const m=b.root.map,value=m.get(b.root);m.delete(b.root);m.set(b.root,value);}],
      ['set order',b=>{const s=b.root.set;s.delete(b.root);s.add(b.root);}],
      ['date',b=>{b.root.date.setTime(2);}],
      ['error own field',b=>{b.root.error.message='changed';}],
      ['function',b=>{b.root.method=function replacement(){};}]
    ]) {
      const bridge=fresh(), before=bridge.graph;
      if(!bridge.matchesPublishedGraph()) failures.push(name+': acknowledged baseline did not match');
      edit(bridge);
      const matched=bridge.matchesPublishedGraph(), snapshot=bridge.snapshot([],true);
      const unchanged = snapshot.delta.nodes.length===0 && snapshot.delta.strings.length===0 &&
        snapshot.graph.nodes.length===before.nodes.length && snapshot.graph.strings.length===before.strings.length &&
        snapshot.graph.root.kind===before.root.kind && snapshot.graph.root.value===before.root.value;
      if(matched!==unchanged || matched!==(name==='unchanged'))failures.push(name+': comparator/snapshot disagreement');
      checks.push(name);
    }
    let getters=0;
    const accessor=fresh();Object.defineProperty(accessor.root,'label',{get(){getters++;throw Error('getter ran');},configurable:true});
    if(accessor.matchesPublishedGraph())failures.push('accessor acknowledged');
    let accessorError;try{accessor.snapshot();}catch(error){accessorError=error.message;}
    if(getters!==0||accessorError!=='Accessor properties require an explicit public model codec')failures.push('accessor ordering');
    const prototype=fresh();Object.getPrototypeOf(prototype.root).extra=true;
    let comparatorError,snapshotError;
    try{prototype.matchesPublishedGraph();}catch(error){comparatorError=error.message;}
    try{prototype.snapshot();}catch(error){snapshotError=error.message;}
    if(comparatorError!=='Prototype edits require an explicit native prototype mutation codec'||comparatorError!==snapshotError)failures.push('prototype ordering');
    return {failures,checks,getters};
  });
  expect(report.failures).toEqual([]);
  expect(report.checks).toHaveLength(13);
  expect(report.getters).toBe(0);
});
