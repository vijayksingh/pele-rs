export const SOURCE='sequenceDiagram\nparticipant A as Alpha\nparticipant B as Beta\nA->>B: Hello';
export const MODES=['unchanged','append','delete-future','numeric'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const scalar=v=>typeof v==='number'?{number:bits(v)}:v;
const copy=v=>JSON.parse(JSON.stringify(v,(_k,v)=>v instanceof Map?{$map:[...v]}:v instanceof Set?{$set:[...v].map(scalar)}:typeof v==='number'?scalar(v):v));
const drawing=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height),links:v.links??[]});
export function runSetHelper(core,entry,dispose,raw,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE);
 const values=new Set(mode==='numeric'?[NaN,-0,7]:['a','b','c']);model.extension=values;
 const iterator=values.values(),first=scalar(iterator.next().value),trace=[];let fired=false,remaining;
 const options={measurer:{width(text,size,style){
  trace.push([text,bits(size),style??0]);
  if(!fired){
   fired=true;
   if(mode==='append')values.add('d');
   if(mode==='delete-future')values.delete('b');
   model.addActor('C','Gamma',{text:'Gamma',type:'participant'});
   remaining=[...iterator].map(scalar);
  }
  return text.length*size*.6;
 }}};
 try{
  const firstDraw=drawing(raw?entry.render(model,{},options):core.render(model,options));
  const state=copy(model),recoveryTrace=[];
  const recoveryOptions={measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
  const recovery=drawing(raw?entry.render(model,{},recoveryOptions):core.render(model,recoveryOptions));
  return {raw,mode,fired,first,remaining,values:[...values].map(scalar),identity:model.extension===values,hasAddedActor:model.actors.has('C'),firstDraw,trace,state,recovery,recoveryTrace};
 }finally{dispose(model);}
}
