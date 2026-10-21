export const SOURCE='stateDiagram-v2\ndirection LR\nstate "Alpha" as A\nstate "Beta" as B\nA --> B: transition';
export const MODES=['inherited','last-direction','own-undefined','nonenumerable-undefined','nonenumerable-TB','callback-undefined','getter-return','getter-throw','getter-reentry','callback-helper'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height),links:r.links??[]});
const atom=v=>v===undefined?{$undefined:true}:v;
const descriptor=m=>{const d=Object.getOwnPropertyDescriptor(m,'direction');return d?{enumerable:d.enumerable,configurable:d.configurable,...('value'in d?{value:atom(d.value),writable:d.writable}:{get:typeof d.get,set:typeof d.set})}:null;};
export function runStateDirection(core,entry,dispose,raw,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE);
 const prototype=Object.getPrototypeOf(model),originalDescriptor=Object.getOwnPropertyDescriptor(prototype,'direction'),inherited=originalDescriptor.get;
 if(mode==='last-direction')model.rootDoc.push({stmt:'dir',value:'TB'});
 if(mode.startsWith('own-')||mode.startsWith('nonenumerable-'))Object.defineProperty(model,'direction',{value:mode==='nonenumerable-TB'?'TB':undefined,enumerable:mode==='own-undefined',writable:true,configurable:true});
 const before={direction:atom(model.direction),descriptor:descriptor(model)},trace=[],getterTrace=[],sentinel={mode};let fired=false,busy=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const render=options=>raw?entry.render(model,{},options):core.render(model,options);
 const options={maxWidth:180,measurer:{width(t,s,style){
  trace.push([t,bits(s),style??0]);
  if(!fired){fired=true;
   if(mode==='callback-undefined')Object.defineProperty(model,'direction',{value:undefined,writable:true,configurable:true});
   if(mode==='callback-helper')model.setDirection('TB');

  }return t.length*s*.6;
 }}};
 if(mode.startsWith('getter-')||mode==='getter-return')Object.defineProperty(prototype,'direction',{...originalDescriptor,get(){
  getterTrace.push(mode);
  if(mode==='getter-throw')throw sentinel;
  if(mode==='getter-reentry'&&!busy){busy=true;try{nested=drawing(render(pure));}finally{busy=false;}}
  return inherited.call(this);
 }});
 let first;try{first={ok:true,...drawing(render(options))};}catch(error){first={ok:false,original:error===sentinel,error:error===sentinel?'sentinel':{name:error.name,message:error.message}};}
 Object.defineProperty(prototype,'direction',originalDescriptor);
 const after={descriptor:descriptor(model),rootDoc:JSON.parse(JSON.stringify(model.rootDoc)),title:atom(model.title)};
 delete model.direction;model.setDirection('LR');
 const recoveryTrace=[],recovery=drawing(render({maxWidth:180,measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}));
 const result={raw,mode,before,fired,first,trace,getterTrace,after,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(model);return result;
}
