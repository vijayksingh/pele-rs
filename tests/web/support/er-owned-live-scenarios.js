export const SOURCE='erDiagram\ndirection LR\nA { int aid PK\n string first }\nB { int bid PK\n string second }\nC { int cid PK\n string future }\nA ||--o{ B : owns\nB ||--o{ C : uses';
export const MODES=['baseline','current','later','replace-array','detached-array','styles','direction','metadata','throw-object','throw-undefined','nested-render','helper'];
const bits=n=>{const v=new DataView(new ArrayBuffer(8));v.setFloat64(0,n);return v.getUint32(0).toString(16).padStart(8,'0')+v.getUint32(4).toString(16).padStart(8,'0');};
const out=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const value=v=>typeof v==='number'?{number:bits(v)}:v===undefined?{undefined:true}:Array.isArray(v)?v.map(value):v instanceof Map?{map:[...v].map(([k,x])=>[value(k),value(x)])}:v instanceof Set?{set:[...v].map(value)}:v&&typeof v==='object'?Object.fromEntries(Object.keys(v).map(k=>[k,value(v[k])])):v;
export function runEr(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE),saved=model.entities.get('C'),attributes=saved.attributes,styles=saved.cssStyles,first=model.entities.get('A').attributes[0],sentinel={mode},trace=[];
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options),settings=responsive?{maxWidth:180}:{};
 let fired=false,nested,helperIdentity;
 const plain=target=>({...settings,measurer:{width(t,s,style){target.push(['width',t,bits(s),style??0]);return t.length*s*.6;}},icons(name){target.push(['icon',name]);return '<path d="M0,0H2"/>';}});
 const mutate=()=>{fired=true;switch(mode){
  case'current':first.name='Changed current';break;
  case'later':attributes[1].name='Changed later';saved.alias='Changed alias';break;
  case'replace-array':saved.attributes=[{type:'string',name:'Replacement',keys:[],comment:''}];break;
  case'detached-array':saved.attributes=[];attributes[1].name='Changed detached';break;
  case'styles':styles.push('fill:red');break;
  case'direction':model.direction='TB';break;
  case'metadata':model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';break;
  case'throw-object':case'throw-undefined':model.title='Persisted title';attributes[1].name='Persisted later';throw mode==='throw-undefined'?undefined:sentinel;
  case'nested-render':attributes[1].name='Nested later';nested=out(entry.render(model,{},plain([])));model.title='After nested';break;
  case'helper':helperIdentity=model.addEntity('C')===saved;break;
 }};
 try{let firstResult;try{firstResult={ok:true,...out(draw({...settings,measurer:{width(t,s,style){trace.push(['width',t,bits(s),style??0]);if(!fired&&(mode!=='current'||t==='int'))mutate();return t.length*s*.6;}},icons(name){trace.push(['icon',name]);return '<path d="M0,0H2"/>';}}))};}catch(error){firstResult={ok:false,original:error===(mode==='throw-undefined'?undefined:sentinel),error:error===undefined?'undefined':error===sentinel?'sentinel':{name:error?.name,message:error?.message}};}
 const after=value(model),captured=value({saved,attributes,styles,first}),recoveryTrace=[];let recovery;try{recovery=out(draw(plain(recoveryTrace)));}catch(error){recovery={error:{name:error?.name,message:error?.message}};}
 return{raw,responsive,mode,fired,first:firstResult,trace,after,captured,...(nested?{nested}:{}),...(helperIdentity===undefined?{}:{helperIdentity}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
