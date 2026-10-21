export const SOURCE='ishikawa\nRoot\n  Category A\n    Cause A\n  Category B\n    Cause B';
export const MODES=['baseline','title','current-text','later-text','append-child','replace-child','replace-children','replace-root','helper','throw-object','throw-undefined','nested'];
const bits=n=>{const b=new DataView(new ArrayBuffer(8));b.setFloat64(0,n);return b.getBigUint64(0).toString(16).padStart(16,'0');};
const output=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height)});
const tree=node=>node?{text:node.text,children:node.children.map(tree)}:null;
export function runIshikawa(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0,{}):core.parse(SOURCE);model.title='Initial title';
 const oldRoot=model.root,oldCategory=model.root.children[0],sentinel={mode},trace=[];let fired=false,nested;
 const settings=responsive?{maxWidth:220}:{};
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options);
 const trigger=text=>!['current-text','later-text','append-child','replace-child','replace-children'].includes(mode)||text==='Category A';
 const mutate=()=>{switch(mode){
  case'title':model.title='Changed title';break;
  case'current-text':oldCategory.text='Changed A';break;
  case'later-text':oldRoot.children[1].text='Changed B';break;
  case'append-child':oldCategory.children.push({text:'Added cause',children:[]});break;
  case'replace-child':oldRoot.children[1]={text:'Replacement category',children:[]};break;
  case'replace-children':oldCategory.children=[{text:'Replacement cause',children:[]}];break;
  case'replace-root':model.root={text:'Replacement root',children:[{text:'Fresh category',children:[]}]};break;
  case'helper':model.addNode(4,'Helper cause');break;
  case'throw-object':case'throw-undefined':model.title='Before failure';oldCategory.text='Persisted category';throw mode==='throw-undefined'?undefined:sentinel;
  case'nested':model.title='Nested title';nested=output(entry.render(model,{}, {measurer:{width:(t,s)=>t.length*s*.6}}));model.title='After nested';break;
 }};
 try{
  let first;try{first={ok:true,...output(draw({...settings,measurer:{width(text,size,style){trace.push([text,bits(size),style??0]);if(!fired&&trigger(text)){fired=true;mutate();}return text.length*size*.6;}}}))};}catch(e){first={ok:false,original:e===(mode==='throw-undefined'?undefined:sentinel)};if(!first.original)first.error={name:e?.name,message:e?.message};}
  const after={title:model.title??null,root:tree(model.root)},identity={root:model.root===oldRoot,category:model.root.children[0]===oldCategory},recoveryTrace=[];
  let recovery;try{recovery={ok:true,...output(draw({...settings,measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}))};}catch(e){recovery={ok:false,error:{name:e?.name,message:e?.message}};}
  return{raw,responsive,mode,fired,first,trace,after,identity,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
