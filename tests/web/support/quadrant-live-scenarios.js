export const SOURCE='quadrantChart\ntitle Initial title\nx-axis Low --> High\ny-axis Bottom --> Top\nquadrant-1 Q1\nquadrant-2 Q2\nquadrant-3 Q3\nquadrant-4 Q4\nAlpha: [0.2, 0.3]\nBeta: [0.7, 0.8]';
export const MODES=['baseline','title','axes','quadrants','point-current','point-later','points-replace','points-append','class-style','accessibility','throw-object','throw-undefined','nested-helper','nested-render'];
const bits=n=>{const v=new DataView(new ArrayBuffer(8));v.setFloat64(0,n);return v.getUint32(0).toString(16).padStart(8,'0')+v.getUint32(4).toString(16).padStart(8,'0');};
const out=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const value=v=>typeof v==='number'?{number:bits(v)}:v===undefined?{undefined:true}:v instanceof Map?{map:[...v].map(([k,v])=>[value(k),value(v)])}:Array.isArray(v)?v.map(value):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).map(k=>[k,value(v[k])])):v;
const state=m=>value(Object.fromEntries(Object.keys(m).map(k=>[k,m[k]])));
export function runQuadrant(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE);
 const originalPoints=model.points,originalPoint=model.points[0],sentinel={mode},trace=[];
 model.points[0].className='hot';model.points[1].className='hot';model.classes.set('hot',{radius:6,color:'ff0000',strokeColor:'00ff00',strokeWidth:'2'});
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options),settings=responsive?{maxWidth:320}:{};
 const pointMode=['point-current','point-later','points-replace','points-append','class-style'].includes(mode);
 let fired=false,nested;
 try{
  let first;
  try{first={ok:true,...out(draw({...settings,measurer:{width(text,size,style){
   trace.push([text,bits(size),style??0]);
   if(!fired&&(!pointMode||text==='Alpha')){
    fired=true;
    switch(mode){
     case'title':model.title='Changed title';break;
     case'axes':model.xAxisRight={text:'Changed right',type:'text'};model.yAxisTop={text:'Changed top',type:'text'};break;
     case'quadrants':model.quadrants[1]={text:'Changed Q2',type:'text'};break;
     case'point-current':originalPoint.x=.9;originalPoint.text={text:'Changed current',type:'text'};originalPoint.className='changed';break;
     case'point-later':model.points[1].x=.1;model.points[1].text={text:'Changed later',type:'text'};break;
     case'points-replace':model.points=[{x:.5,y:.5,text:{text:'Replacement',type:'text'},className:''}];break;
     case'points-append':model.points.push({x:.4,y:.6,text:{text:'Appended',type:'text'},className:''});break;
     case'class-style':model.classes.set('hot',{radius:17,color:'0000ff'});break;
     case'accessibility':model.accTitle='Changed access';model.accDescr='Changed description';break;
     case'throw-object':case'throw-undefined':model.title='Persisted title';model.points[1].text.text='Persisted later';throw mode==='throw-undefined'?undefined:sentinel;
     case'nested-helper':model.setQuadrant2Text({text:'Nested Q2',type:'text'});break;
     case'nested-render':model.title='Nested title';nested=out(entry.render(model,{}, {measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
    }
   }return text.length*size*.6;
  }}}))};}catch(error){first={ok:false,original:error===(mode==='throw-undefined'?undefined:sentinel),error:error===undefined?'undefined':error===sentinel?'sentinel':{name:error?.name,message:error?.message}};}
  const after=state(model),originalPointsStillCurrent=originalPoints===model.points,originalPointStillFirst=originalPoint===model.points[0],capturedPoints=value(originalPoints),recoveryTrace=[];
  let recovery;try{recovery=out(draw({...settings,measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}));}catch(error){recovery={error:{name:error?.name,message:error?.message}};}
  return{raw,responsive,mode,first,trace,after,originalPointsStillCurrent,originalPointStillFirst,capturedPoints,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
