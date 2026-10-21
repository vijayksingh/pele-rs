export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='venn-beta\ntitle Initial\nset A[Alpha]:10\nset B[Beta]:10\nunion A,B[Both]:3\ntext A note[Note]\nstyle A fill:red';
export const MODES=['baseline','subset-edit','subsets-replace','subset-append','texts-edit','texts-replace','texts-append','styles','title','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>({title:m.title??null,subsets:m.subsets,textNodes:m.textNodes,styles:m.styles.map(s=>({targets:s.targets,styles:[...s.styles]}))});
export function runVenn(core,entry,dispose,source,raw,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source),subsets=m.subsets,texts=m.textNodes,trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const options={measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(mode!=='texts-append'||t==='Note')){fired=true;switch(mode){
 case'subset-edit':m.subsets[1].label='Changed Beta';m.subsets[1].size=20;break;
 case'subsets-replace':m.subsets=[{sets:['A'],size:20,label:'New Alpha'},{sets:['B'],size:10,label:'New Beta'},{sets:['A','B'],size:3,label:'New Both'}];break;
 case'subset-append':m.subsets.push({sets:['A'],size:10,label:'Last Alpha'});break;
 case'texts-edit':m.textNodes[0].label='Changed Note';m.textNodes[0].id='changed-note';break;
 case'texts-replace':m.textNodes=[{sets:['B'],id:'other',label:'Other'}];break;
 case'texts-append':m.textNodes.push({sets:['B'],id:'appended',label:'Appended'});break;
 case'styles':m.styles[0].styles.set('fill','blue');m.styles.push({targets:['B'],styles:new Map([['font-weight','bold']])});break;
 case'title':m.title='Changed title';break;
 case'throw':m.title='Before throw';m.subsets[1].label='Saved Beta';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(entry.render(m,{},pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(m)),originalSubsets=copy(subsets),originalTexts=copy(texts),recoveryTrace=[];
 const recoveryOptions={measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,mode,first,trace,after,originalSubsets,originalTexts,subsetsCurrent:subsets===m.subsets,textsCurrent:texts===m.textNodes,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
