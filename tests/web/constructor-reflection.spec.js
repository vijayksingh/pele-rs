import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('captured constructors preserve artifact names, descriptors and shared registry identities',async({page},info)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const corpus=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const {default:initialize}=await import('/dist/package-web/package/initialize.js');
    const {NativeRegistry}=await import('/dist/package-web/package/pkg/pele.js');
    const {ModelRegistry}=await import('/dist/package-web/package/graph.js');
    await initialize();
    const sourceRegistry=new NativeRegistry(true);
    const copiedSchema=JSON.parse(sourceRegistry.entry_schema('sankey')).result;
    sourceRegistry.free();
    // A caller can import graph.js and copy a builtin schema, but neither that
    // schema nor shape-valid reflection JSON can mint shared realm authority.
    const forwarded=[new ModelRegistry(),new ModelRegistry()];
    const forgedReflection=copiedSchema.prototypes.map(p=>({prototype:p.id,identity:'sankey',monolithName:'Poisoned',moduleName:'Poisoned'}));
    const forwardedRegistrations=forwarded.map(r=>r.registration(structuredClone(copiedSchema),forgedReflection));
    const forwardedConstructors=forwardedRegistrations.map(r=>r.prototypes.get(1).constructor);
    let hijacked=0;

    const {loadRegistry}=await import('/dist/package-web/package/registry.js');
    const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');
    await loadDiagramEntries();
    const mono=await import('/dist/evidence/pele-oracle.js');
    const core=await import('/dist/evidence/pele-esm/core.js');
    const core2=await import('/dist/evidence/pele-esm/core.js?constructor-reflection-second-registry');
    const fullA=await loadRenderer(),fullB=await loadRenderer(),esmA=await loadRegistry(undefined,{allBuiltins:true}),esmB=await loadRegistry(undefined,{allBuiltins:true});
    const atom=x=>typeof x==='function'?{kind:'function',name:x.name,length:x.length,keys:Reflect.ownKeys(x).map(String)}:x===undefined?{kind:'undefined'}:x===null?null:typeof x==='object'?{kind:'object'}:x;
    const descriptor=(o,k)=>{const d=Object.getOwnPropertyDescriptor(o,k);return {key:String(k),enumerable:d.enumerable,configurable:d.configurable,...('value'in d?{writable:d.writable,value:atom(d.value)}:{get:atom(d.get),set:atom(d.set)})};};
    const withoutNew=C=>{try{Reflect.apply(C,undefined,[]);return {threw:false};}catch(error){return {threw:true,name:error.name,message:error.message};}};
    const describe=C=>({name:C.name,length:C.length,constructor:Reflect.ownKeys(C).map(k=>descriptor(C,k)),prototype:Reflect.ownKeys(C.prototype).map(k=>descriptor(C.prototype,k)),parentObject:Object.getPrototypeOf(C.prototype)===Object.prototype,withoutNew:withoutNew(C)});
    const classes=[],rows=[],closed=[];
    const collect=(family,values)=>{const constructors=values.map(value=>Object.getPrototypeOf(value).constructor);classes.push({family,constructors});rows.push({family,actual:constructors.slice(0,5).map(describe),expected:[constructors[5],constructors[5],constructors[6],constructors[7],constructors[8]].map(describe),identity:{actual:constructors[0]===constructors[1]&&constructors[2]===constructors[3]&&constructors[3]===constructors[4]&&constructors[0]!==constructors[2],expected:constructors[6]===constructors[7]&&constructors[7]===constructors[8]&&constructors[5]!==constructors[6]}});};
    try{
      for(const meta of corpus.entries){
        const c=corpus.cases.find(c=>c.family===meta.type),entry=(await import('/dist/package-web/package/diagrams/'+meta.type+'.js')).default,ref=(await import('/dist/evidence/pele-esm/diagrams/'+meta.type+'.js')).default;
        core.register(ref);core2.register(ref);
        const values=[fullA.parse(c.source),fullB.parse(c.source),esmA.parse(c.source),esmB.parse(c.source),entry.parse(c.source,c.config,c.title??undefined,c.lineOffset,{maxEdges:undefined}),mono.parse(c.source),core.parse(c.source),core2.parse(c.source),ref.parse(c.source,c.config,c.title??undefined,c.lineOffset,{maxEdges:undefined})];
        if(Object.getPrototypeOf(values[5])!==Object.prototype)collect(meta.type,values);
        fullA.dispose(values[0]);fullB.dispose(values[1]);esmA.dispose(values[2]);esmB.dispose(values[3]);disposeDiagramModel(values[4]);
        if(meta.type==='class'){
          const source='classDiagram\nclass A {\n+method() String\n}';
          const members=[fullA.parse(source),fullB.parse(source),esmA.parse(source),esmB.parse(source),entry.parse(source,{},undefined,0),mono.parse(source),core.parse(source),core2.parse(source),ref.parse(source,{},undefined,0)];
          collect('ClassMember',members.map(m=>m.classes.get('A').methods[0]));
          fullA.dispose(members[0]);fullB.dispose(members[1]);esmA.dispose(members[2]);esmB.dispose(members[3]);disposeDiagramModel(members[4]);
        }
      }
      const pair=(family)=>classes.find(c=>c.family===family).constructors;
      const authentic=fullA.parse('sankey\nA,B,1');
      try{
        // A local owner's table cannot insert a receiver into builtin dispatch.
        forwarded.forEach((registry,index)=>registry.owners.set(authentic,{registry,registration:forwardedRegistrations[index],disposed:false,invoke(){hijacked++;throw new Error('hijacked');}}));
        authentic.findOrCreateNode('Still native');
      }finally{fullA.dispose(authentic);}
      const isolation={distinct:forwardedConstructors[0]!==forwardedConstructors[1],notShared:forwardedConstructors.every(C=>pair('sankey').every(real=>C!==real)),namesUnforged:forwardedConstructors.every(C=>C.name!=='Poisoned'),hijacked};
      const flow=pair('flowchart'),swim=pair('swimlane');
      const flowAliases=flow.map((C,i)=>C===swim[i]&&C.prototype===swim[i].prototype);
      // Retained factories survive source registry closure. Helpers still select
      // the newly constructed owner, rather than the first registry's old model.
      fullA.close();fullB.close();esmA.close();esmB.close();
      for(const [index,C]of pair('sankey').slice(0,5).entries()){
        const x=new C(),y=new C();try{const first=x.findOrCreateNode('A');closed.push({index,alias:first===x.findOrCreateNode('A'),independent:x.nodes!==y.nodes&&y.nodes.length===0});}finally{disposeDiagramModel(x);disposeDiagramModel(y);}
      }
      return{rows,flowAliases,closed,isolation};
    }finally{fullA.close();fullB.close();esmA.close();esmB.close();}
  });
  const file=info.outputPath('constructor-reflection.json');await writeFile(file,JSON.stringify(report,null,2));await info.attach('constructor-reflection',{path:file,contentType:'application/json'});
  expect(report.isolation).toEqual({distinct:true,notShared:true,namesUnforged:true,hijacked:0});
  expect(report.rows).toHaveLength(23);
  // Firefox lazily materializes intrinsic constructor name/length properties;
  // two identical pinned oracle copies can expose different insertion order.
  // Keep raw order in the report; compare their exact descriptor/key sets here.
  // Prototype member order and all method/getter reflection stay strict.
  const stableConstructor=description=>({...description,
    constructor:Object.fromEntries(description.constructor.map(d=>[d.key,d])),
    prototype:description.prototype.map(d=>d.key==='constructor'?{...d,value:{...d.value,keys:Object.fromEntries(d.value.keys.map(key=>[key,true]))}}:d),
  });
  const compare=info.project.name==='firefox'?stableConstructor:description=>description;
  for(const row of report.rows){expect(row.actual.map(compare),row.family).toEqual(row.expected.map(compare));expect(row.identity,row.family).toEqual({actual:true,expected:true});}
  expect(report.flowAliases).toEqual(Array(9).fill(true));
  for(const row of report.closed)expect(row).toMatchObject({alias:true,independent:true});
});


test('captured constructors retain native class receiver, subclass and newTarget behavior',async({page},info)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const corpus=await(await fetch('/fixtures/pele/diagram-entries.json')).json();
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const {disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');
    const mono=await import('/dist/evidence/pele-oracle.js');
    const renderer=await loadRenderer(),rows=[];
    const check=(C,args,dispose)=>{
      const values=[];
      try{
        class Derived extends C {}
        const derived=new Derived(...args);values.push(derived);
        const custom=Object.create(C.prototype),reads=[];
        const target=new Proxy(function(){},{get(target,key,receiver){if(key==='prototype'){reads.push(key);return custom;}return Reflect.get(target,key,receiver);}});
        const allocated=Reflect.construct(C,args,target);values.push(allocated);
        const sentinel={original:true};let original=false;
        const throwing=new Proxy(function(){},{get(target,key,receiver){if(key==='prototype')throw sentinel;return Reflect.get(target,key,receiver);}});
        try{Reflect.construct(C,args,throwing);}catch(error){original=error===sentinel;}
        return{derivedPrototype:Object.getPrototypeOf(derived)===Derived.prototype,derivedInstance:derived instanceof C,
          derivedConstructor:derived.constructor===Derived,customPrototype:Object.getPrototypeOf(allocated)===custom,
          inheritedConstructor:allocated.constructor===C,ownConstructor:Object.hasOwn(allocated,'constructor'),
          constructorBackref:C.prototype.constructor===C,reads,original};
      }finally{for(const value of values)dispose?.(value);}
    };
    try{
      for(const meta of corpus.entries){
        const fixture=corpus.cases.find(row=>row.family===meta.type),reference=(await import('/dist/evidence/pele-esm/diagrams/'+meta.type+'.js')).default;
        const pairs=[['monolith',renderer.parse(fixture.source),mono.parse(fixture.source)],['module',renderer.parsePrepared(meta.type,fixture.source,fixture.config,fixture.title??undefined,fixture.lineOffset??0),reference.parse(fixture.source,fixture.config,fixture.title??undefined,fixture.lineOffset,{maxEdges:undefined})]];
        for(const [lane,actual,expected]of pairs){
          try{
            if(Object.getPrototypeOf(expected)!==Object.prototype)rows.push({family:meta.type,lane,actual:check(actual.constructor,[],disposeDiagramModel),expected:check(expected.constructor,[])});
          }finally{renderer.dispose(actual);}
        }
      }
      const source='classDiagram\nclass A {\n+method() String\n}';
      for(const lane of ['monolith','module']){
        const reference=(await import('/dist/evidence/pele-esm/diagrams/class.js')).default;
        const actual=lane==='monolith'?renderer.parse(source):renderer.parsePrepared('class',source,{},undefined,0),expected=lane==='monolith'?mono.parse(source):reference.parse(source,{},undefined,0);
        try{rows.push({family:'ClassMember',lane,actual:check(actual.classes.get('A').methods[0].constructor,['method() String','method'],disposeDiagramModel),expected:check(expected.classes.get('A').methods[0].constructor,['method() String','method'])});}
        finally{renderer.dispose(actual);}
      }
      return rows;
    }finally{renderer.close();}
  });
  await writeFile(info.outputPath('constructor-new-target.json'),JSON.stringify(report,null,2));
  expect(report).toHaveLength(46);
  for(const row of report){expect(row.actual,row.family+'/'+row.lane).toEqual(row.expected);expect(row.actual.original).toBe(true);expect(row.actual.customPrototype).toBe(true);expect(row.actual.derivedInstance).toBe(true);}
});
