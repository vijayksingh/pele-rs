// Browser-side public API diagnostic. Imported unchanged by all three lanes.
export async function runRow({workload,profile,stage,trials,warmupMs}) {
  const {runtimes,api,oracle}=window.rawBenchmark,names=['js','control','candidate'];
  const quantile=(values,p)=>[...values].sort((a,b)=>a-b)[Math.max(0,Math.ceil(values.length*p)-1)];
  const measurer=profile==='fixed'?{width:(text,size,style)=>text.length*size*((style&1)?.71:.6)}:api.canvasMeasurer('sans-serif','monospace');
  if(!measurer)throw Error('Canvas unavailable');
  const options={...workload.options,measurer},config={};
  // One untouched seed supplies identical fresh caller-owned data to every op.
  // No seed is rendered or handed to either native runtime.
  const seed=oracle.parse(workload.source,config,undefined,0,{});
  function clone(input){
    const copies=new Map();
    function copy(value){
      if(value===null||typeof value!=='object'){if(typeof value==='function'||typeof value==='symbol')throw Error('Fixture contains unsupported callable/symbol');return value;}
      if(copies.has(value))return copies.get(value);
      const output=Array.isArray(value)?[]:value instanceof Map?new Map():value instanceof Set?new Set():value instanceof Date?new Date(Date.prototype.getTime.call(value)):Object.create(Object.getPrototypeOf(value)===null?null:Object.prototype);
      copies.set(value,output);
      if(value instanceof Map)for(const [key,item]of Map.prototype.entries.call(value))output.set(copy(key),copy(item));
      if(value instanceof Set)for(const item of Set.prototype.values.call(value))output.add(copy(item));
      const keys=Reflect.ownKeys(value);if(Array.isArray(value))keys.sort((a,b)=>a==='length'?1:b==='length'?-1:0);
      for(const key of keys){if(typeof key!=='string')throw Error('Symbol fixture key');const d=Object.getOwnPropertyDescriptor(value,key);if(!Object.hasOwn(d,'value'))throw Error('Accessor fixture');Object.defineProperty(output,key,{...d,value:copy(d.value)});}
      return output;
    }
    return copy(input);
  }
  function signature(root){
    const ids=new Map(),pending=[],nodes=[],view=new DataView(new ArrayBuffer(8));
    const atom=value=>{
      if(value===undefined)return ['undefined'];if(value===null)return ['null'];
      if(typeof value==='number'){view.setFloat64(0,value);return ['number',view.getBigUint64(0).toString(16).padStart(16,'0')];}
      if(typeof value==='string'||typeof value==='boolean')return [typeof value,value];
      if(typeof value!=='object')throw Error('Unexpected signature primitive');
      if(!ids.has(value)){ids.set(value,pending.length);pending.push(value);}return ['ref',ids.get(value)];
    };
    const start=atom(root);
    for(let n=0;n<pending.length;n++){
      const object=pending[n],proto=Object.getPrototypeOf(object);
      const kind=Array.isArray(object)?'array':object instanceof Map?'map':object instanceof Set?'set':object instanceof Date?'date':proto===null?'null-object':proto===Object.prototype?'object':null;
      if(!kind)throw Error('Unexpected fixture prototype');
      const properties=Reflect.ownKeys(object).map(key=>{const d=Object.getOwnPropertyDescriptor(object,key);if(typeof key!=='string'||!Object.hasOwn(d,'value'))throw Error('Unsupported descriptor');return [key,d.enumerable,d.writable,d.configurable,atom(d.value)];});
      nodes.push({kind,properties,...(kind==='map'?{entries:[...object].map(([key,value])=>[atom(key),atom(value)])}:{}),...(kind==='set'?{values:[...object].map(atom)}:{}),...(kind==='date'?{time:atom(Date.prototype.getTime.call(object))}:{})});
    }
    return JSON.stringify({root:start,nodes});
  }
  const draw=(name,model)=>name==='js'?oracle.render(model,config,options):runtimes[name].renderRaw('er',model,config,options);
  const dispose=(name,model)=>{if(name!=='js')runtimes[name].dispose(model);};
  const exact=value=>({svg:value.svg,width:value.width,height:value.height,links:JSON.stringify(value.links)});
  const same=(a,b)=>a.svg===b.svg&&Object.is(a.width,b.width)&&Object.is(a.height,b.height)&&a.links===b.links;
  const fixtureSignature=signature(clone(seed));
  function verify(){
    const results={};
    for(const name of names){
      const model=clone(seed),entities=model.entities,firstEntity=[...entities.values()][0],attributes=firstEntity.attributes,row=attributes[0];
      if(signature(model)!==fixtureSignature)throw Error('Fresh fixture differs');
      try{
        const first=exact(draw(name,model)),afterFirst=signature(model),repeat=exact(draw(name,model)),afterRepeat=signature(model);
        if(model.entities!==entities||[...entities.values()][0]!==firstEntity||firstEntity.attributes!==attributes||attributes[0]!==row)throw Error(name+': caller identities changed');
        results[name]={first,repeat,afterFirst,afterRepeat};
      }finally{dispose(name,model);}
    }
    for(const name of ['control','candidate'])for(const pass of ['first','repeat']){
      if(!same(results.js[pass],results[name][pass]))throw Error(name+'/'+pass+': complete output mismatch');
      const key=pass==='first'?'afterFirst':'afterRepeat';if(results.js[key]!==results[name][key])throw Error(name+'/'+pass+': model/descriptors mismatch');
    }
    return results.js;
  }
  const before=verify();let checksum=0;
  const consume=value=>{checksum+=value.svg.length+value.width+value.height;};
  function batch(name,count){
    // Retained stage explicitly pays one clone/adoption outside its own timer.
    // First stage never performs any render/admission on its fresh root beforehand.
    let retained;
    if(stage==='raw-repeat'){retained=clone(seed);try{draw(name,retained);}catch(error){dispose(name,retained);throw error;}}
    const start=performance.now();let elapsed;
    try{
      for(let i=0;i<count;i++){
        if(stage==='clone-only'){const value=clone(seed);checksum+=value.entities.size;}
        else if(stage==='raw-first-inclusive'){
          const value=clone(seed);try{consume(draw(name,value));}finally{dispose(name,value);}
        }else consume(draw(name,retained));
      }
      elapsed=performance.now()-start;
    }finally{if(retained)dispose(name,retained);}
    return elapsed;
  }
  const warmups={},calibration={};
  for(const name of names){
    const start=performance.now();let operations=0;do{batch(name,1);operations++;}while(operations<6||performance.now()-start<warmupMs);
    warmups[name]={operations,elapsedMs:performance.now()-start};
    let count=1;const probes=[];
    while(true){const ms=Array.from({length:3},()=>batch(name,count));probes.push({count,ms});if(count===128||quantile(ms,.5)>=12)break;count*=2;}
    calibration[name]={count,probes};
  }
  // Same batch count across all lanes, not only the two WASM implementations.
  const count=Math.min(...names.map(name=>calibration[name].count));
  const samples=Object.fromEntries(names.map(name=>[name,[]])),raw=[];
  for(let trial=0;trial<trials;trial++){
    let order=names.slice(trial%3).concat(names.slice(0,trial%3));if(Math.floor(trial/3)%2)order.reverse();
    const elapsedMs={};for(const name of order){const ms=batch(name,count);if(!(ms>0))throw Error('Timer sample is zero');elapsedMs[name]=ms;samples[name].push(ms/count);}
    raw.push({trial,order,count,elapsedMs});
  }
  const after=verify();
  if(!same(before.first,after.first)||!same(before.repeat,after.repeat)||before.afterFirst!==after.afterFirst||before.afterRepeat!==after.afterRepeat)throw Error('Correctness changed after timing');
  const hash=async text=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');
  const ratios=samples.candidate.map((value,i)=>value/samples.control[i]);
  const medianMs=Object.fromEntries(names.map(name=>[name,quantile(samples[name],.5)]));
  return {case:workload.id,profile,stage,count,warmups,calibration,samples,raw,medianMs,p95BatchMeanMs:Object.fromEntries(names.map(name=>[name,quantile(samples[name],.95)])),candidateOverControl:medianMs.candidate/medianMs.control,candidateOverJS:medianMs.candidate/medianMs.js,controlOverJS:medianMs.control/medianMs.js,paired:{ratios,medianRatio:quantile(ratios,.5),medianPercentChange:(quantile(ratios,.5)-1)*100,candidateWins:ratios.filter(r=>r<1).length},correctness:{beforeAndAfter:true,completeOutput:true,descriptorsAndAliases:true,fixtureSha256:await hash(fixtureSignature),firstSvgSha256:await hash(after.first.svg),repeatSvgSha256:await hash(after.repeat.svg),firstStateSha256:await hash(after.afterFirst),repeatStateSha256:await hash(after.afterRepeat),width:after.first.width,height:after.first.height,links:after.first.links},checksum};
}
