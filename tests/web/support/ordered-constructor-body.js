// Appended to constructor-feature.js by the test driver; uses the same public
// graph/render oracle normalization. No package SPI or diagram implementation.
export async function orderedBodies(){
  const {cases,corpus}=await inputs(['public-ordered-constructors.json']);
  const {loadRenderer}=await import(location.origin+'/dist/package-web/package/full.js');
  const {loadDiagramEntries,disposeDiagramModel}=await import(location.origin+'/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const engine=await loadRenderer(),report=[];
  try{for(const fixture of cases){
    const source=corpus.find(c=>c.family===fixture.family),row={family:fixture.family,fixture};
    for(const lane of ['oracle','candidate']){
      const entry=(await import(location.origin+(lane==='candidate'?'/dist/package-web/package':'/dist/evidence/pele-esm')+'/diagrams/'+fixture.family+'.js')).default;
      const parsed=entry.parse(source.source,source.config,source.title??undefined,source.lineOffset,{}),C=parsed.constructor;
      if(lane==='candidate')disposeDiagramModel(parsed);
      const model=new C();
      const initial={graph:graph(model,C.prototype),render:draw(entry,model)},steps=actions(model,fixture,entry),populated={graph:graph(model,C.prototype),render:draw(entry,model)};
      if(lane==='candidate')disposeDiagramModel(model);
      const constructorCases=fixture.constructorCases.map(({kind})=>{
        const reads=[],sentinel={kind};let instance;
        const options=new Proxy({},{ownKeys(){throw Error('Options enumerated');},get(_,key){reads.push(key);if(kind==='firstThrow'||kind==='undefinedThrow')throw kind==='undefinedThrow'?undefined:sentinel;if(kind==='secondThrow'&&reads.length===2)throw sentinel;if(kind==='values'){if(key==='wrap')return true;if(key==='sequenceWrap')return false;if(key==='lineOffset')return 9;if(key==='warn')return function warn(message){};}return undefined;}});
        const extra=new Proxy({},{get(){throw Error('Extra argument read');},ownKeys(){throw Error('Extra argument enumerated');}});
        try{const first=kind==='undefined'?undefined:options;instance=new C(first,extra);return{kind,reads,thrown:false,graph:graph(instance,C.prototype,undefined,fixture.family==='agentflow'&&first!==undefined?options:undefined)};}
        catch(e){return{kind,reads,thrown:true,error:e===sentinel?'original-object':e===undefined?'original-undefined':error(e)};}
        finally{if(instance&&lane==='candidate')disposeDiagramModel(instance);}
      });
      class Derived extends C{}
      const derived=new Derived();
      const subclass={instance:derived instanceof Derived,prototype:Object.getPrototypeOf(derived)===Derived.prototype,graph:graph(derived,Derived.prototype),first:actions(derived,{actions:fixture.actions.slice(0,1)},entry)};
      if(lane==='candidate')disposeDiagramModel(derived);
      row[lane]={initial,actions:steps,populated,constructorCases,subclass,arity:C.length};
    }report.push(row);
  }return report;}finally{engine.close();}
}
export async function memberBodies(){
  const fixture=await(await fetch('/fixtures/public-class-member-constructor.json')).json();
  const {loadDiagramEntries,disposeDiagramModel}=await import(location.origin+'/dist/package-web/package/diagram-entries.js');await loadDiagramEntries();
  const report={fixture};
  const fields=o=>Object.getOwnPropertyNames(o).map(k=>{const d=Object.getOwnPropertyDescriptor(o,k);return[k,d.value===undefined?{undefined:true}:d.value,(d.enumerable?1:0)|(d.writable?2:0)|(d.configurable?4:0)];});
  for(const lane of ['oracle','candidate']){
    const entry=(await import(location.origin+(lane==='candidate'?'/dist/package-web/package':'/dist/evidence/pele-esm')+'/diagrams/class.js')).default;
    const seed=entry.parse('classDiagram\nclass A {\n+method() String\n}',{},undefined,0),C=seed.classes.get('A').methods[0].constructor;
    if(lane==='candidate')disposeDiagramModel(seed);
    const cases=fixture.cases.map(({input,kind})=>{const member=new C(input,kind,new Proxy({},{get(){throw Error('Unused third argument');}}));try{return{input,kind,fields:fields(member),display:member.getDisplayDetails(),classifier:member.parseClassifier()};}finally{if(lane==='candidate')disposeDiagramModel(member);}});
    const virtual=[];
    for(const mode of ['method-true','method-false','attribute','method-throw','attribute-throw','undefined-throw']){
      const trace=[],sentinel={mode};let escaped,member;
      class Derived extends C{
        parseMethod(input){escaped=this;trace.push(['parseMethod',input,fields(this)]);if(mode==='method-throw'||mode==='undefined-throw')throw mode==='undefined-throw'?undefined:sentinel;this.id=' + custom ';return mode==='method-true';}
        parseAttribute(input){escaped=this;trace.push(['parseAttribute',input,fields(this)]);if(mode==='attribute-throw')throw sentinel;this.id=' attribute ';}
      }
      const kind=mode.startsWith('attribute')?'attribute':'method';
      try{member=new Derived('+input()',kind);virtual.push({mode,kind,trace,ok:true,fields:fields(member),instance:member instanceof Derived});}
      catch(e){virtual.push({mode,kind,trace,ok:false,original:e===(mode==='undefined-throw'?undefined:sentinel),fields:fields(escaped)});}
      if(escaped){virtual.at(-1).recovery=escaped.getDisplayDetails();if(lane==='candidate')disposeDiagramModel(escaped);}
      else if(member&&lane==='candidate')disposeDiagramModel(member);
    }
    report[lane]={cases,virtual,arity:C.length};
  }return report;
}
