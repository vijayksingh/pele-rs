import {test,expect} from '@playwright/test';

test('ER compatibility grammar keeps live callbacks and retained host identity',async({page})=>{
  await page.goto('/examples/web/');
  const rows=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const base='erDiagram\nA { int aid PK }\nB { string bid FK }\nA ||--o{ B : owns\n';
    const sources=[
      ['class',base+'classDef marked fill:red\nclass B marked'],
      ['style',base+'style B fill:blue'],
      ['duplicate-alias',base+'B["Replacement alias"]'],
      ['subgraph','erDiagram\nsubgraph box\nA { int aid PK }\nB { string bid FK }\nend\nA ||--o{ B : owns'],
      ['late-entity',base+'C'],
      ['large-source',base+'A\n'.repeat(33000)],
      ['large-title','---\ntitle: '+'T'.repeat(65537)+'\n---\n'+base],
    ];
    const width=(text,size)=>text.length*size*.6;
    const signature=value=>{
      const ids=new Map(),pending=[],nodes=[];
      const atom=v=>{
        if(v===undefined)return ['undefined'];
        if(typeof v==='function')return ['function',v.name,v.length];
        if(v===null||typeof v!=='object')return v;
        if(!ids.has(v)){ids.set(v,pending.length);pending.push(v);}
        return ['ref',ids.get(v)];
      };
      const root=atom(value);
      for(let i=0;i<pending.length;i++){
        const v=pending[i],kind=Array.isArray(v)?'array':v instanceof Map?'map':v instanceof Set?'set':'object';
        nodes.push({kind,properties:Object.getOwnPropertyNames(v).map(key=>{
          const d=Object.getOwnPropertyDescriptor(v,key);
          return[key,d.enumerable,!!d.writable,d.configurable,'value'in d?atom(d.value):['accessor']];
        }),...(kind==='map'?{entries:[...v].map(([k,x])=>[atom(k),atom(x)])}:{}),...(kind==='set'?{values:[...v].map(atom)}:{})});
      }
      return JSON.stringify({root,nodes});
    };
    const rows=[];
    for(const [fixture,source] of sources)for(const variant of ['edit','nested','throw']){
      const row={fixture,variant};
      for(const [lane,renderer] of [['js',oracle],['wasm',engine]]){
        const model=renderer.parse(source,{limit:1_000_000}),saved=model.entities.get('B'),attributes=saved.attributes,styles=saved.cssStyles;
        const sentinel={marker:'original'},cycle={};cycle.self=cycle;
        let fired=false,nested=null,methodIdentity=null;const widths=[];
        const outcome=fn=>{try{const v=fn();return {ok:true,svg:v.svg,width:String(v.width),height:String(v.height)};}catch(error){return {ok:false,kind:typeof error,name:error?.name,message:error?.message,code:error?.code,original:error===sentinel};}};
        const render=measurer=>renderer.render(model,{measurer});
        try{
          const first=outcome(()=>render({width(...args){
            widths.push(args);
            if(!fired){
              fired=true;attributes[0].name='changed in callback';styles.push('fill:green');
              model.title='callback title';model.accTitle='callback accessible title';model.callbackExtension=cycle;
              if(variant==='nested'){
                methodIdentity=model.addEntity('B')===saved;
                nested=outcome(()=>render({width}));
              }
              if(variant==='throw')throw sentinel;
            }
            return width(...args);
          }}));
          const after=signature({model,saved,attributes,styles});
          const next=outcome(()=>render({width}));
          row[lane]={fired,first,nested,methodIdentity,widths,after,next,afterNext:signature({model,saved,attributes,styles}),identity:model.entities.get('B')===saved&&saved.attributes===attributes&&saved.cssStyles===styles};
        }finally{if(lane==='wasm')renderer.dispose(model);}
      }
      row.mismatches=Object.keys(row.js).filter(key=>JSON.stringify(row.js[key])!==JSON.stringify(row.wasm[key]));rows.push(row);
    }
    return rows;
  });
  expect(rows).toHaveLength(21);
  for(const row of rows){expect(row.js.fired,row.fixture).toBe(true);expect(row.wasm.fired,row.fixture).toBe(true);}
  expect(rows.filter(row=>row.mismatches.length).map(({fixture,variant,mismatches})=>({fixture,variant,mismatches}))).toEqual([]);
});

test('ER streamed public construction preserves large Unicode text and discards partial publications',async({page})=>{
  await page.goto('/examples/web/');
  const rows=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js'),oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const declarations=Array.from({length:300},(_,i)=>`A${i} { string id PK\n string value }`).join('\n');
    const sources=[...['T'.repeat(32762),'T'.repeat(32763),'T'.repeat(32763)+'😀'+'Z'.repeat(100),'T'.repeat(65537)].map((title,i)=>['text-'+i,'---\ntitle: '+title+'\n---\nerDiagram\nA { string id PK }']),['flushed-fallback','erDiagram\n'+declarations+'\nstyle A0 fill:red'],['flushed-syntax','erDiagram\n'+declarations+'\nB { int }']];
    const options={limit:1_000_000,measurer:{width:(text,size)=>text.length*size*.6}},rows=[];
    for(const [name,source]of sources){const row={name};
      for(const [lane,renderer]of [['js',oracle],['wasm',engine]]){let model;
        try{model=renderer.parse(source,options);const title=model.title,first=renderer.render(model,options);const entity=model.entities.values().next().value;row[lane]={ok:true,title,source:renderer.render(source,options),first,next:renderer.render(model,options),sameEntity:model.entities.values().next().value===entity};}
        catch(error){row[lane]={ok:false,name:error?.name,message:error?.message,code:error?.code,type:error?.type,line:error?.line,column:error?.column,snippet:error?.snippet};}
        finally{if(lane==='wasm'&&model)renderer.dispose(model);}
      }
      row.mismatches=Object.keys(row.js).filter(key=>JSON.stringify(row.js[key])!==JSON.stringify(row.wasm[key]));rows.push(row);
    }return rows;
  });
  expect(rows).toHaveLength(6);
  expect(rows.filter(row=>row.mismatches.length).map(({name,mismatches})=>({name,mismatches}))).toEqual([]);
});
