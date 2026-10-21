export const SOURCE='treeView-beta\ntitle Initial title\nfolder/ icon(folder)\n    Alpha.txt ## Alpha description\n    Beta.txt icon(file) ## Beta description\nTail.txt';
export const MODES=['baseline','title','current','later','ids','children-append','children-replace','root-replace','description','icon-current','icon-later','accessibility','throw-object','throw-undefined','icon-throw','nested-render'];
const bits=n=>{const v=new DataView(new ArrayBuffer(8));v.setFloat64(0,n);return v.getUint32(0).toString(16).padStart(8,'0')+v.getUint32(4).toString(16).padStart(8,'0');};
const out=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const value=v=>typeof v==='number'?{number:bits(v)}:v===undefined?{undefined:true}:Array.isArray(v)?v.map(value):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).map(k=>[k,value(v[k])])):v;
const fresh=name=>({id:9,level:4,name,nodeType:'file',children:[]});
export function runTreeView(core,entry,dispose,raw,responsive,mode){
 const model=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE),originalRoot=model.root,folder=model.root.children[0],alpha=folder.children[0],beta=folder.children[1],sentinel={mode},trace=[];
 const draw=options=>raw?entry.render(model,{},options):core.render(model,options),settings=responsive?{maxWidth:180}:{};
 let fired=false,nested;
 const mutate=()=>{fired=true;switch(mode){
  case'title':model.title='Changed title';break;
  case'current':alpha.name='Changed current';alpha.nodeType='directory';alpha.cssClass='highlight red';break;
  case'later':beta.name='Changed later';beta.icon='none';break;
  case'ids':alpha.id=beta.id;folder.id=beta.id;break;
  case'children-append':folder.children.push(fresh('Appended'));break;
  case'children-replace':folder.children=[fresh('Replacement')];break;
  case'root-replace':model.root={...model.root,children:[fresh('New root child')]};break;
  case'description':alpha.description='Changed description';beta.description='Later description';break;
  case'icon-current':folder.name='Icon current';folder.nodeType='file';folder.cssClass='highlight blue';folder.description='Icon description';break;
  case'icon-later':beta.name='Icon later';beta.nodeType='directory';beta.description='Icon later description';break;
  case'accessibility':model.accTitle='Changed access';model.accDescr='Changed description';break;
  case'throw-object':case'throw-undefined':case'icon-throw':model.title='Persisted title';beta.name='Persisted later';throw mode==='throw-undefined'?undefined:sentinel;
  case'nested-render':model.title='Nested title';nested=out(entry.render(model,{},plain([])));model.title='After nested';break;
 }};
 const plain=target=>({...settings,measurer:{width(t,s,style){target.push(['width',t,bits(s),style??0]);return t.length*s*.6;}},icons(name){target.push(['icon',name]);return '<path d="M0,0H2"/>';}});
 try{let first;try{first={ok:true,...out(draw({...settings,measurer:{width(t,s,style){trace.push(['width',t,bits(s),style??0]);const target=['children-append','children-replace'].includes(mode)?'folder':['current','later','ids','root-replace','description'].includes(mode)?'Alpha.txt':'Initial title';if(!fired&&!mode.startsWith('icon-')&&t===target)mutate();return t.length*s*.6;}},icons(name){trace.push(['icon',name]);if(!fired&&mode.startsWith('icon-'))mutate();return '<path d="M0,0H2"/>';}}))};}catch(error){first={ok:false,original:error===(mode==='throw-undefined'?undefined:sentinel),error:error===undefined?'undefined':error===sentinel?'sentinel':{name:error?.name,message:error?.message}};}
 const after=value(model),capturedRoot=value(originalRoot),rootStillCurrent=originalRoot===model.root,recoveryTrace=[];let recovery;try{recovery=out(draw(plain(recoveryTrace)));}catch(error){recovery={error:{name:error?.name,message:error?.message}};}
 return{raw,responsive,mode,first,trace,after,capturedRoot,rootStillCurrent,...(nested?{nested}:{}),recovery,recoveryTrace};
 }finally{dispose(model);}
}
