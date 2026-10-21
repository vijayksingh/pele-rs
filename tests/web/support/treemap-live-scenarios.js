export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='treemap\n"Group One"\n "Alpha":4\n "Beta":2\n"Group Two"\n "Gamma":3';
export const MODES=['baseline','name','next-name','values','children-replace','roots-replace','append','styles','late-class','late-styles','shape','title-accessibility','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,roots:m.roots});
export function runTreemap(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title='Initial';
 const roots=m.roots,children=roots[0].children,first=children[0],styles=['fill:red','color:blue'];first.cssCompiledStyles=styles;children[1].cssCompiledStyles=styles;
 const trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(!mode.startsWith('late-')||t==='Alpha')){fired=true;switch(mode){
 case'name':m.roots[0].name='Changed group';first.name='Changed Alpha';break;
 case'next-name':children[1].name='Changed Beta';break;
 case'values':first.value=40;children[1].value=20;break;
 case'children-replace':m.roots[0].children=[{name:'Replacement',value:9}];break;
 case'roots-replace':m.roots=[{name:'Replacement root',value:9}];break;
 case'append':children.push({name:'Appended',value:8});break;
 case'styles':first.cssCompiledStyles=['fill:green','color:white'];first.classSelector='changed';break;
 case'late-class':first.classSelector='late';first.name='After captured name';break;
 case'late-styles':styles[0]='fill:green';break;
 case'shape':delete roots[0].children;roots[0].value=8;break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'throw':m.title='Before throw';first.name='Saved Alpha';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let firstResult;try{firstResult={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){firstResult={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(m)),originalRoots=copy(roots),originalChildren=copy(children),originalFirst=copy(first),originalStyles=copy(styles),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first:firstResult,trace,after,originalRoots,originalChildren,originalFirst,originalStyles,rootsCurrent:roots===m.roots,childrenCurrent:children===m.roots[0].children,firstCurrent:first===m.roots[0].children?.[0],stylesCurrent:styles===first.cssCompiledStyles,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
