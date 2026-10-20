import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

for(const semantic of [false,true]){
test((semantic?'semantic construction parity: ':'')+'captured Sankey construction behavior preserves receiver, defaults, helpers and raw ownership',async({page},info)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');
    await loadDiagramEntries();
    const raw=(await import('/dist/package-web/package/diagrams/sankey.js')).default;
    const oracle=await import('/dist/evidence/pele-oracle.js');
    const oracleRaw=(await import('/dist/evidence/pele-esm/diagrams/sankey.js')).default;
    const engine=await loadRenderer(),seed=engine.parse('sankey\nA,B,1'),referenceSeed=oracle.parse('sankey\nA,B,1');
    const constructors=[referenceSeed.constructor,seed.constructor],rows=[];
    const bitsView=new DataView(new ArrayBuffer(8));
    const bits=value=>{bitsView.setFloat64(0,value);return bitsView.getBigUint64(0).toString(16).padStart(16,'0');};
    function snapshot(root){
      const ids=new Map(),objects=[],nodes=[];
      const atom=value=>{
        if(value===undefined)return ['undefined'];if(value===null)return ['null'];
        if(typeof value==='number')return ['number',bits(value)];
        if(typeof value!=='object')return [typeof value,typeof value==='symbol'?value.description:value];
        if(!ids.has(value)){ids.set(value,objects.length);objects.push(value);}return ['ref',ids.get(value)];
      };
      const rootAtom=atom(root);
      for(let i=0;i<objects.length;i++){
        const value=objects[i],properties=Reflect.ownKeys(value).map(key=>{
          const d=Object.getOwnPropertyDescriptor(value,key);
          return [key,d.enumerable,d.writable,d.configurable,atom(d.value)];
        });
        nodes.push({kind:Array.isArray(value)?'array':value instanceof Map?'map':'object',properties,...(value instanceof Map?{entries:[...value].map(([k,v])=>[atom(k),atom(v)])}:{})});
      }
      return {root:rootAtom,nodes};
    }
    const error=run=>{try{run();return null;}catch(value){return {name:value.name,message:value.message};}};
    function constructorMetadata(C){
      const flags=(object,key)=>{const d=Object.getOwnPropertyDescriptor(object,key);return {data:Object.hasOwn(d,'value'),writable:d.writable,enumerable:d.enumerable,configurable:d.configurable};};
      return {
        // Names and engine error text remain unmodified evidence. This feature
        // does not claim the pending constructor-identity/name catalog.
        name:C.name,callWithoutNew:error(()=>C()),arity:C.length,
        intrinsic:{ownKeys:Reflect.ownKeys(C),length:flags(C,'length'),name:flags(C,'name'),prototype:flags(C,'prototype'),prototypeKeys:Reflect.ownKeys(C.prototype),backlink:flags(C.prototype,'constructor'),backlinkIdentity:C.prototype.constructor===C,prototypeIdentity:Object.getOwnPropertyDescriptor(C,'prototype').value===C.prototype,functionParent:Object.getPrototypeOf(C)===Function.prototype,prototypeParent:Object.getPrototypeOf(C.prototype)===Object.prototype}
      };
    }
    const metadata={candidate:constructorMetadata(constructors[1]),oracleCore:constructorMetadata(constructors[0]),oracleEsm:constructorMetadata(oracleRaw.parse('sankey\nA,B,1',{},undefined).constructor)};
    // A captured constructor must not borrow its original parsed document.
    engine.dispose(seed);
    try{
      for(const mode of ['plain','ignored','subclass','alternate','nullPrototype']){
        const row={mode};
        for(const [index,C]of constructors.entries()){
          const candidate=index===1,trace=[],sentinel={ignored:true};let ignoredReads=0,inheritedWrites=0;
          const ignored=new Proxy({}, {get(){ignoredReads++;throw sentinel;},ownKeys(){ignoredReads++;throw sentinel;},getPrototypeOf(){ignoredReads++;throw sentinel;}});
          const args=mode==='ignored'?[ignored,Symbol('ignored'),null,undefined]:[];
          let model,expectedPrototype;
          if(mode==='subclass'){
            class Derived extends C { extra='subclass'; }
            model=new Derived(...args);expectedPrototype=Derived.prototype;
          }else if(mode==='alternate'||mode==='nullPrototype'){
            function Alternate(){}
            Alternate.prototype=Object.create(mode==='nullPrototype'?null:C.prototype);
            for(const key of ['type','nodes','links','title','accTitle','accDescr','lookup'])Object.defineProperty(Alternate.prototype,key,{set(){inheritedWrites++;throw sentinel;},configurable:true});
            model=Reflect.construct(C,args,Alternate);expectedPrototype=Alternate.prototype;
          }else{model=new C(...args);expectedPrototype=C.prototype;}
          try{
            const initial=snapshot(model),roots=[model.nodes,model.links,model.lookup];
            const draw=()=>{trace.length=0;const options={measurer:{width(text,size,style){trace.push([text,bits(size),style]);return text.length*size*.6;}}};const output=(candidate?raw:oracleRaw).render(model,{},options);return {output,width:bits(output.width),height:bits(output.height),trace:trace.slice()};};
            const before=error(()=>candidate?engine.render(model):oracle.render(model));
            const empty=draw();
            // Borrow from the actual constructor prototype, including an
            // alternate newTarget whose prototype has no helper inheritance.
            const a=C.prototype.findOrCreateNode.call(model,'__proto__'),again=C.prototype.findOrCreateNode.call(model,'__proto__');
            const b=C.prototype.findOrCreateNode.call(model,'B\u{1f642}');
            const add=C.prototype.addLink.call(model,a,b,5);
            model.title='Constructed';
            const populated=draw(),after=error(()=>candidate?engine.render(model):oracle.render(model));
            const thrown={callback:true};let original=false;
            try{(candidate?raw:oracleRaw).render(model,{}, {measurer:{width(){model.title='Kept callback title';throw thrown;}}});}catch(value){original=value===thrown;}
            const recovery=draw();
            row[candidate?'candidate':'oracle']={initial,empty,populated,recovery,state:snapshot(model),prototype:Object.getPrototypeOf(model)===expectedPrototype,rootsStable:roots[0]===model.nodes&&roots[1]===model.links&&roots[2]===model.lookup,alias:a===again&&a===model.nodes[0]&&model.lookup.get('__proto__')===a&&model.links[0].source===a&&model.links[0].target===b,addUndefined:add===undefined,ignoredReads,inheritedWrites,before,after,original,arity:C.length};
          }finally{if(candidate)disposeDiagramModel(model);}
        }
        rows.push(row);
      }
      const callWithoutNew=constructors.map(C=>error(()=>C()).name);
      // Flow now has a real captured constructor; verify ready defaults and helpers.
      const other=engine.parse('flowchart TD\nA-->B'),reference=oracle.parse('flowchart TD\nA-->B');let flow;
      const made=new other.constructor(),expected=new reference.constructor();
      try{made.addVertex('Fresh');expected.addVertex('Fresh');flow={actual:snapshot(made),expected:snapshot(expected),prototype:Object.getPrototypeOf(made)===other.constructor.prototype};}
      finally{engine.dispose(made);engine.dispose(other);}
      return {rows,callWithoutNew,metadata,flow};
    }finally{engine.close();}
  });
  const path=info.outputPath('sankey-constructor.json');await writeFile(path,JSON.stringify(report,null,2));await info.attach('sankey-constructor.json',{path,contentType:'application/json'});
  expect(report.callWithoutNew).toEqual(['TypeError','TypeError']);
  for(const [lane,metadata]of Object.entries(report.metadata)){
    expect(metadata.arity,lane).toBe(0);
    expect(metadata.callWithoutNew.name,lane).toBe('TypeError');
    // Exact companion retains key-order equality; semantic companion checks
    // presence/flags while raw metadata preserves the known Firefox order gap.
    if(semantic)expect([...metadata.intrinsic.ownKeys].sort(),lane).toEqual([...report.metadata.oracleEsm.intrinsic.ownKeys].sort());
    expect(metadata.intrinsic,lane).toEqual({
      ownKeys:semantic?metadata.intrinsic.ownKeys:report.metadata.oracleEsm.intrinsic.ownKeys,
      length:{data:true,writable:false,enumerable:false,configurable:true},
      name:{data:true,writable:false,enumerable:false,configurable:true},
      prototype:{data:true,writable:false,enumerable:false,configurable:false},
      prototypeKeys:['constructor','findOrCreateNode','addLink'],
      backlink:{data:true,writable:true,enumerable:false,configurable:true},
      backlinkIdentity:true,prototypeIdentity:true,functionParent:true,prototypeParent:true
    });
  }
  // Known name and non-new message gaps are retained in the attached raw report,
  // not normalized or asserted equal to the oracle.
  expect(report.flow.actual).toEqual(report.flow.expected);
  expect(report.flow.prototype).toBe(true);
  expect(report.rows).toHaveLength(5);
  for(const row of report.rows){
    expect(row.candidate,row.mode).toEqual(row.oracle);
    expect(row.candidate,row.mode).toMatchObject({prototype:true,rootsStable:true,alias:true,addUndefined:true,ignoredReads:0,inheritedWrites:0,original:true,arity:0,before:{name:'TypeError',message:'render() takes diagram text or a model that parse() returned.'},after:{name:'TypeError',message:'render() takes diagram text or a model that parse() returned.'}});
  }
});

}

test('Sankey constructors retain captured registrations across replacement, close and disposal',async({page})=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRegistry}=await import('/dist/package-web/package/registry.js');
    const {loadDiagramEntries,disposeDiagramModel}=await import('/dist/package-web/package/diagram-entries.js');
    await loadDiagramEntries();
    const raw=(await import('/dist/package-web/package/diagrams/sankey.js')).default,oracleRaw=(await import('/dist/evidence/pele-esm/diagrams/sankey.js')).default;
    const oracle=await import('/dist/evidence/pele-oracle.js'),reference=oracle.parse('sankey\nA,B,1'),Expected=reference.constructor;
    const a=await loadRegistry(),b=await loadRegistry();a.register(raw);b.register(raw);
    const seedA=a.parse('sankey\nA,B,1'),seedB=b.parse('sankey\nX,Y,2'),A=seedA.constructor,B=seedB.constructor;
    a.dispose(seedA);b.dispose(seedB);
    // A foreign replacement cannot become the previously captured factory.
    a.register({type:'sankey',parse(){throw new Error('Replacement parse');},render(){throw new Error('Replacement render');}});
    a.close();b.close();
    const checks=[],rows=[],models=[];
    const options={measurer:{width(text,size){return text.length*size*.6;}}};
    try{
      for(const C of [A,B]){
        const model=new C(Symbol('ignored')),expected=new Expected();models.push(model);
        const first=model.findOrCreateNode('First'),target=model.findOrCreateNode('Last');model.addLink(first,target,3);
        const x=expected.findOrCreateNode('First'),y=expected.findOrCreateNode('Last');expected.addLink(x,y,3);
        rows.push({actual:raw.render(model,{},options),expected:oracleRaw.render(expected,{},options)});
        checks.push(first===model.nodes[0]&&model.lookup.get('First')===first&&model.links[0].source===first);
      }
      // Plain foreign receivers use the live captured constructor owner.
      const plain={type:'sankey',nodes:[],links:[],lookup:new Map()},expectedPlain={type:'sankey',nodes:[],links:[],lookup:new Map()};
      const added=A.prototype.findOrCreateNode.call(plain,'Borrowed'),wanted=Expected.prototype.findOrCreateNode.call(expectedPlain,'Borrowed');
      checks.push(added===plain.nodes[0]&&plain.lookup.get('Borrowed')===added&&JSON.stringify(added)===JSON.stringify(wanted));
      const kept=models[0],helper=kept.findOrCreateNode;
      disposeDiagramModel(kept);
      let disposed;try{raw.render(kept,{},options);}catch(error){disposed={name:error.name,message:error.message};}
      // A new factory invocation stays independent of that disposed owner.
      const fresh=new A();models.push(fresh);checks.push(fresh.findOrCreateNode('Fresh')===fresh.nodes[0]);
      const borrowed=helper.call(fresh,'After disposal');checks.push(borrowed===fresh.nodes[1]&&fresh.lookup.get('After disposal')===borrowed);
      return {rows,checks,disposed};
    }finally{for(const model of models)disposeDiagramModel(model);a.close();b.close();}
  });
  expect(report.checks).toEqual(Array(5).fill(true));
  expect(report.disposed).toEqual({name:'TypeError',message:'Public model document is disposed'});
  for(const row of report.rows)expect(row.actual).toEqual(row.expected);
});
