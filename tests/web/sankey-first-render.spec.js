import {test,expect} from '@playwright/test';
import {writeFile} from 'node:fs/promises';

test('fresh public Sankey first render preserves thrown values, recovery and helper aliases',async({page},info)=>{
  await page.goto('/examples/web/');
  const report=await page.evaluate(async()=>{
    const {loadRenderer}=await import('/dist/package-web/package/full.js');
    const oracle=await import('/dist/evidence/pele-oracle.js'),engine=await loadRenderer();
    const source='sankey\nA,B,5\nB,C,3\nA,C,1';
    const bitsView=new DataView(new ArrayBuffer(8));
    const bits=value=>{bitsView.setFloat64(0,value);return bitsView.getBigUint64(0).toString(16).padStart(16,'0');};
    // Public own descriptors and collection entries, with ordered alias IDs.
    // Keep actual references separately to verify retained identity within a lane.
    function snapshot(root){
      const ids=new Map(),objects=[],nodes=[];
      const key=value=>typeof value==='symbol'?['symbol',Symbol.keyFor(value),value.description]:['string',value];
      const atom=value=>{
        if(value===null)return ['null'];
        if(typeof value==='number')return ['number',bits(value)];
        if(typeof value==='undefined')return ['undefined'];
        if(typeof value==='symbol')return key(value);
        if(typeof value==='function')return ['function',value.length];
        if(typeof value!=='object')return [typeof value,value];
        if(!ids.has(value)){ids.set(value,objects.length);objects.push(value);}
        return ['ref',ids.get(value)];
      };
      const rootAtom=atom(root);
      for(let i=0;i<objects.length;i++){
        const object=objects[i],proto=Object.getPrototypeOf(object);
        const kind=Array.isArray(object)?'Array':object instanceof Map?'Map':object instanceof Set?'Set':'Object';
        const prototype=proto===null?'null':proto===Object.prototype?'Object':kind==='Object'?'model':kind;
        const properties=Reflect.ownKeys(object).map(k=>{
          const d=Object.getOwnPropertyDescriptor(object,k);
          return [key(k),d.enumerable,d.configurable,'value'in d?['data',d.writable,atom(d.value)]:['accessor',atom(d.get),atom(d.set)]];
        });
        const entries=object instanceof Map?[...object].map(([k,v])=>[atom(k),atom(v)]):object instanceof Set?[...object].map(atom):[];
        nodes.push({kind,prototype,properties,entries});
      }
      return {signature:{root:rootAtom,nodes},objects};
    }
    const stable=(before,after)=>before.length===after.length&&before.every((value,index)=>value===after[index]);
    const rows=[];
    try{
      for(const kind of ['object','symbol','undefined']){
        const sentinel=kind==='object'?{original:'first Sankey width'}:kind==='symbol'?Symbol('first Sankey width'):undefined;
        const row={kind};
        for(const lane of ['oracle','candidate']){
          const api=lane==='candidate'?engine:oracle,model=api.parse(source);
          try{
            const initial=snapshot(model),aliases=()=>model.links[0].source===model.nodes[0]&&model.links[0].target===model.nodes[1]&&model.links[1].source===model.nodes[1]&&model.links[1].target===model.nodes[2]&&model.links[2].source===model.nodes[0]&&model.links[2].target===model.nodes[2]&&['A','B','C'].every((name,index)=>model.lookup.get(name)===model.nodes[index]);
            const firstNode=model.nodes[0],initialAliases=aliases(),failureTrace=[];
            let threw=false,original=false;
            // First render, before any helper or model mutation.
            try{api.render(model,{measurer:{width(text,size,style){failureTrace.push([text,bits(size),style]);throw sentinel;}}});}
            catch(error){threw=true;original=Object.is(error,sentinel);}
            const failed=snapshot(model);
            const draw=()=>{
              const trace=[];
              const output=api.render(model,{measurer:{width(text,size,style){trace.push([text,bits(size),style]);return text.length*size*.6;}}});
              return {output:snapshot(output).signature,trace};
            };
            const recovery=draw(),recovered=snapshot(model);
            const helper=model.findOrCreateNode('A'),afterHelper=snapshot(model);
            const helperAlias=helper===firstNode&&helper===model.nodes[0]&&helper===model.lookup.get('A');
            const next=draw(),afterNext=snapshot(model);
            row[lane]={initial:initial.signature,failed:failed.signature,recovered:recovered.signature,afterHelper:afterHelper.signature,afterNext:afterNext.signature,initialAliases,finalAliases:aliases(),threw,original,failureTrace,recovery,next,helperAlias,stable:[failed,recovered,afterHelper,afterNext].map(value=>stable(initial.objects,value.objects))};
          }finally{if(lane==='candidate')engine.dispose(model);}
        }
        rows.push(row);
      }
      return rows;
    }finally{engine.close();}
  });
  const path=info.outputPath('sankey-first-render-recovery.json');
  await writeFile(path,JSON.stringify(report,null,2));
  await info.attach('sankey-first-render-recovery.json',{path,contentType:'application/json'});
  expect(report).toHaveLength(3);
  for(const row of report){
    expect(row.candidate,row.kind).toEqual(row.oracle);
    expect(row.candidate,row.kind).toMatchObject({threw:true,original:true,initialAliases:true,finalAliases:true,helperAlias:true,stable:[true,true,true,true]});
    expect(row.candidate.failureTrace,row.kind).toHaveLength(1);
    expect(row.candidate.recovery.trace.length,row.kind).toBeGreaterThan(0);
    for(const stage of ['failed','recovered','afterHelper','afterNext'])expect(row.candidate[stage],row.kind+'/'+stage).toEqual(row.candidate.initial);
    expect(row.candidate.next,row.kind).toEqual(row.candidate.recovery);
  }
});
