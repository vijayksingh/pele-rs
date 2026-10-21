export const SOURCE='gitGraph\ncommit id:"a" msg:"Alpha" tag:"v1"\nbranch feature\ncommit id:"b" type:HIGHLIGHT\ncheckout main\ncommit id:"c"\nmerge feature id:"m"';
export const MODES=['baseline','title','lanes','current-tag','current-id','later-id','parents','replace-map','type','throw-object','throw-undefined','nested'];
export const cases=()=>[true,false].flatMap(raw=>[false,true].flatMap(responsive=>MODES.map(mode=>({raw,responsive,mode}))));
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const state=m=>({title:m.title??null,direction:m.direction,lanes:[...m.lanes],commits:[...m.commits].map(([key,c])=>({key,id:c.id,message:c.message,seq:bits(c.seq),type:bits(c.type),customType:c.customType==null?null:bits(c.customType),customId:c.customId??null,branch:c.branch,tags:[...c.tags],parents:[...c.parents]}))});
export function runGitGraph(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);model.title='Initial title';
 const oldMap=model.commits,oldFirst=oldMap.get('a'),oldLater=oldMap.get('b'),oldTags=oldFirst.tags,oldLanes=model.lanes,sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const mutate=()=>{switch(mode){
  case'title':model.title='Changed title';break;
  case'lanes':model.lanes=[...oldLanes].reverse();break;
  case'current-tag':oldTags[0]='Changed tag';oldTags.push('late-tag');break;
  case'current-id':oldFirst.id='renamed-a';break;
  case'later-id':oldLater.id='renamed-b';break;
  case'parents':oldLater.parents=[];break;
  case'replace-map':model.commits=new Map();break;
  case'type':oldFirst.type=2;oldLater.customType=1;break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldFirst.message='Persisted message';throw mode.endsWith('undefined')?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(draw({measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 const matches=t=>['current-tag','replace-map','type'].includes(mode)?t==='v1':['current-id','later-id','parents'].includes(mode)?t==='a':true;
 const resources=(record,active)=>({measurer:{width(t,s,style){record.push(['width',t,bits(s),style??0]);if(active&&!fired&&matches(t)){fired=true;mutate();}return t.length*s*.6;}}});
 try{
  let first;try{first={ok:true,...output(draw({...settings,...resources(trace,true)}))};}catch(e){first={ok:false,original:e===(mode.endsWith('undefined')?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after=state(model),identity={map:model.commits===oldMap,first:model.commits.get('a')===oldFirst,tags:oldFirst.tags===oldTags,lanes:model.lanes===oldLanes},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,...resources(recoveryTrace,false)}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
