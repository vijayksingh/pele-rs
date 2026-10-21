export const SOURCE='gitGraph\ncommit id:"a" msg:"Alpha" tag:"v1"\nbranch feature\ncommit id:"b" type:HIGHLIGHT\ncheckout main\ncommit id:"c"\nmerge feature id:"m"';
export const MODES=['initial-set','initial-string','initial-astral','initial-empty','later-set','later-string','set-append','set-delete-future','set-replace','string-replace','throw-object','throw-undefined','nested'];
export const cases=()=>[true,false].flatMap(raw=>[false,true].flatMap(responsive=>MODES.map(mode=>({raw,responsive,mode}))));
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const state=m=>({title:m.title??null,direction:m.direction,lanes:[...m.lanes],commits:[...m.commits].map(([key,c])=>({key,id:c.id,message:c.message,seq:bits(c.seq),type:bits(c.type),customType:c.customType==null?null:bits(c.customType),customId:c.customId??null,branch:c.branch,tags:[...c.tags],parents:[...c.parents]}))});
export function runGitGraph(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);model.title='Initial title';
 const oldMap=model.commits,oldFirst=oldMap.get('a'),oldLater=oldMap.get('b');
 oldFirst.tags=mode==='initial-string'||mode==='string-replace'?'XY':mode==='initial-astral'?'😀Z':mode==='initial-empty'?new Set():mode.startsWith('later-')?['v1']:new Set(['v1','v2','v3']);
 const oldTags=oldFirst.tags,oldLanes=model.lanes,sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const mutate=()=>{switch(mode){
  case'later-set':oldLater.tags=new Set(['Later','Tail']);break;
  case'later-string':oldLater.tags='😀Z';break;
  case'set-append':oldTags.add('late-tag');break;
  case'set-delete-future':oldTags.delete('v2');break;
  case'set-replace':oldFirst.tags=new Set(['Replacement']);break;
  case'string-replace':oldFirst.tags='Replacement';break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldTags.add('Persisted');throw mode.endsWith('undefined')?undefined:sentinel;
  case'nested':oldTags.add('Nested tag');model.title='Nested title';nested=output(draw({measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 const matches=t=>mode==='initial-empty'?true:mode==='initial-string'||mode==='string-replace'?t==='X':mode==='initial-astral'?t==='😀':t==='v1';
 const resources=(record,active)=>({measurer:{width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active&&!fired&&matches(t)){fired=true;mutate();}return t.length*s*.6;}}});
 try{
  let first;try{first={ok:true,...output(draw({...settings,...resources(trace,true)}))};}catch(e){first={ok:false,original:e===(mode.endsWith('undefined')?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after=state(model),identity={map:model.commits===oldMap,first:model.commits.get('a')===oldFirst,tags:oldFirst.tags===oldTags,lanes:model.lanes===oldLanes},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,...resources(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,originalTags:[...oldTags],...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
