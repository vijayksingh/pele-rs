export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='cynefin-beta\ncomplex\n "Alpha"\n "Beta"\ncomplicated\n "Gamma"\nclear\n "Delta"\nchaotic\n "Epsilon"\nconfusion\n "One"\n "Two"\n "Three"\n "Four"\ncomplex --> clear : "Move one"\nclear --> chaotic : "Move two"';
export const MODES=['baseline','title-accessibility','domain-replace','next-domain','items-replace','captured-label','append','confusion-shrink','transitions-replace','transition-append','transition-fields','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const state=m=>({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,domains:[...m.domains],transitions:m.transitions});
const copy=v=>JSON.parse(JSON.stringify(v));
export function runCynefin(core,entry,dispose,source,raw,responsive,mode){
 const model=raw?entry.parse(source,{},undefined,0):core.parse(source);model.title='Initial';
 const domains=model.domains,items=domains.get('complex').items,first=items[0],transitions=model.transitions,transition=transitions[0];
 const trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&t===(mode==='transition-fields'?'Move one':mode==='items-replace'||mode==='captured-label'||mode==='append'?'Alpha':mode==='confusion-shrink'?'One':'Complex')){fired=true;switch(mode){
 case'title-accessibility':model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';break;
 case'domain-replace':model.domains=new Map([['complex',{name:'complex',items:[{label:'Replacement'}]}]]);break;
 case'next-domain':domains.get('complicated').items[0].label='Changed Gamma';break;
 case'items-replace':domains.get('complex').items=[{label:'Replacement items'}];break;
 case'captured-label':first.label='After captured Alpha';items[1].label='Changed Beta';break;
 case'append':items.push({label:'Appended'});break;
 case'confusion-shrink':domains.get('confusion').items.length=3;break;
 case'transitions-replace':model.transitions=[{from:'complicated',to:'chaotic',label:'Replacement transition'}];break;
 case'transition-append':transitions.push({from:'complex',to:'chaotic',label:'Appended transition'});break;
 case'transition-fields':transition.from='chaotic';transition.to='complicated';transition.label='Changed after capture';break;
 case'throw':model.title='Before throw';first.label='Saved Alpha';throw sentinel;
 case'nested':model.title='Nested title';nested=drawing(raw?entry.render(model,{},pure):core.render(model,pure));model.title='After nested';break;
 }}return t.length*s*.6;}}};
 let result;try{result={ok:true,...drawing(raw?entry.render(model,{},options):core.render(model,options))};}catch(e){result={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(model)),originalItems=copy(items),originalFirst=copy(first),originalTransitions=copy(transitions),originalTransition=copy(transition),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(model,{},recoveryOptions):core.render(model,recoveryOptions));
 const record={raw,responsive,mode,fired,first:result,trace,after,originalItems,originalFirst,originalTransitions,originalTransition,domainsCurrent:domains===model.domains,itemsCurrent:items===model.domains.get('complex')?.items,transitionsCurrent:transitions===model.transitions,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(model);return record;
}
