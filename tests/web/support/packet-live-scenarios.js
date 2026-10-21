export const SOURCE='packet\ntitle Packet title\n0-7: "Alpha"\n8-15: "Beta"\n16-31: "Gamma"\n32-63: "Delta"';
export const MODES=['baseline','title','current-label','later-label','current-geometry','append-block','replace-row','replace-rows','accessibility','throw-object','throw-undefined','nested'];
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const rows=value=>value.map(row=>row.map(b=>({start:bits(b.start),end:bits(b.end),bits:bits(b.bits),label:b.label})));
export function runPacket(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE),oldRows=model.rows,oldFirst=model.rows[0][0],sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const state=()=>({title:model.title??null,accTitle:model.accTitle??null,accDescr:model.accDescr??null,bitsPerRow:bits(model.bitsPerRow),rows:rows(model.rows)});
 const mutate=()=>{switch(mode){
  case'title':model.title='Changed title';break;
  case'current-label':oldFirst.label='Changed Alpha';break;
  case'later-label':model.rows[0][1].label='Changed Beta';break;
  case'current-geometry':oldFirst.start=1;oldFirst.end=6;break;
  case'append-block':model.rows[0].push({start:64,end:71,bits:8,label:'Added'});break;
  case'replace-row':model.rows[0]=[{start:0,end:31,bits:32,label:'Replacement row'}];break;
  case'replace-rows':model.rows=[[{start:0,end:15,bits:16,label:'Replacement rows'}]];break;
  case'accessibility':model.accTitle='Changed access';model.accDescr='Changed description';break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldFirst.label='Persisted label';throw mode==='throw-undefined'?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(entry.render(model,{}, {measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 try{
  let first;
  try{first={ok:true,...output(draw({...settings,measurer:{width(text,size,style){trace.push([text,bits(size),style??0]);if(!fired&&(!['current-label','current-geometry','append-block'].includes(mode)||text==='Alpha')){fired=true;mutate();}return text.length*size*.6;}}}))};}
  catch(e){first={ok:false,original:e===(mode==='throw-undefined'?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after=state(),identity={rows:model.rows===oldRows,first:model.rows[0]?.[0]===oldFirst},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
