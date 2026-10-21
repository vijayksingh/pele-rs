// Public resource callbacks only. Same scenario executes against both engines.
export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
const bits=value=>{const bytes=new ArrayBuffer(8),view=new DataView(bytes);view.setFloat64(0,value);return view.getUint32(0).toString(16).padStart(8,'0')+view.getUint32(4).toString(16).padStart(8,'0');};
const output=value=>({svg:value.svg,width:bits(value.width),height:bits(value.height)});
const failure=(error,sentinel)=>({ok:false,original:error===sentinel||error===undefined,error:error===undefined?'undefined':error===sentinel?'sentinel':{name:error?.name,message:error?.message}});
export function runPie(core,pie,dispose,raw,responsive,mode){
 const source='pie\ntitle Initial title\n"Alpha" : 1\n"Beta" : 2\n"Gamma" : 3';
 const model=raw?pie.parse(source,{},undefined,0):core.parse(source),original=model.sections,trace=[],sentinel={mode};let fired=false,nested;
 const draw=options=>raw?pie.render(model,{},options):core.render(model,options);
 const settings=responsive?{maxWidth:200}:{};
 try{
  const options={...settings,measurer:{width(text,size,style){
   trace.push([text,bits(size),style??0]);
   if(!fired){fired=true;switch(mode){
    case'title':model.title='Changed title';break;
    case'showData':model.showData=true;break;
    case'map-edit':model.sections.set('Beta',20);model.sections.delete('Gamma');model.sections.set('Delta',4);break;
    case'map-replace':model.sections=new Map([['Replacement',8]]);break;
    case'accessibility':model.accTitle='Changed access';model.accDescr='Changed description';break;
    case'throw-object':case'throw-undefined':model.title='Persisted before throw';model.showData=true;model.sections.set('Added',7);throw mode==='throw-undefined'?undefined:sentinel;
    case'nested-render':model.title='Nested title';nested=output(pie.render(model,{}, {measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
   }}return text.length*size*.6;
  }}};
  let first;try{first={ok:true,...output(draw(options))};}catch(error){first=failure(error,sentinel);}
  const after={title:model.title??null,showData:model.showData,accTitle:model.accTitle??null,accDescr:model.accDescr??null,sections:[...model.sections].map(([k,v])=>[k,bits(v)])};
  const originalMapStillCurrent=original===model.sections,originalMap=[...original],recoveryTrace=[];
  let recovery;try{recovery=output(draw({...settings,measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}));}catch(error){recovery=failure(error,sentinel);}
  return{raw,responsive,mode,first,trace,after,originalMapStillCurrent,originalMap,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
export function runInfo(core,info,dispose,raw,mode){
 const source='info\ntitle Before\naccTitle: Before accessible\naccDescr: Before description';
 const model=raw?info.parse(source,{},undefined,0,{}):core.parse(source),sentinel={mode},trace=[];let fired=false,nested;
 const plain={measurer:{width:(t,s)=>t.length*s*.6}},draw=options=>raw?info.render(model,{},options):core.render(model,options);
 try{
  const width=(text,size)=>{
   trace.push([text,bits(size)]);
   if(!fired&&mode!=='normal'&&(mode!=='version-mutate'||text==='Pele v1.6.0')){
    fired=true;model.title='After';model.accTitle='After accessible';model.accDescr='After description';
    if(mode==='throw'||mode==='throw-undefined')throw mode==='throw-undefined'?undefined:sentinel;
    if(mode==='nested')nested=output(info.render(model,{},plain));
   }return text.length*size*.6;
  };
  let result;try{result={ok:true,...output(draw({measurer:{width}}))};}catch(error){result={ok:false,original:error===(mode==='throw-undefined'?undefined:sentinel)};if(!result.original)result.error={name:error?.name,message:error?.message};}
  const after=Object.fromEntries(Object.keys(model).map(key=>[key,model[key]===undefined?{undefined:true}:model[key]]));
  let recovery;try{recovery=output(draw(plain));}catch(error){recovery=failure(error,sentinel);}
  return{mode,trace,result,after,...(nested?{nested}:{}),recovery};
 }finally{dispose(model);}
}
