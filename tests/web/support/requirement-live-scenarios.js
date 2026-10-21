export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='requirementDiagram\nrequirement Alpha {\n id: A1\n text: First field\n}\nrequirement Beta {\n id: B1\n text: Second field\n}\nelement Service {\n type: Thing\n docRef: Manual\n}\nAlpha - contains -> Beta\nBeta - satisfies -> Service';
export const MODES=['baseline','name-kind','fields-before','fields-after','styles-replace','styles-mutate','classes','maps-replace','append','relations','direction','title-accessibility','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const copy=v=>v instanceof Map?{$map:[...v].map(([k,v])=>[copy(k),copy(v)])}:Array.isArray(v)?v.map(copy):v&&typeof v==='object'?Object.fromEntries(Object.entries(v).filter(([,v])=>v!==undefined).map(([k,v])=>[k,copy(v)])):v;
export function runRequirement(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title='Initial';m.direction='LR';
 const requirements=m.requirements,alpha=requirements.get('Alpha'),beta=requirements.get('Beta'),elements=m.elements,element=elements.get('Service'),relations=m.relations,relation=relations[0];
 beta.cssStyles=['fill:red'];beta.classes=['default','before'];const styles=beta.cssStyles,classes=beta.classes;
 const trace=[],sentinel={mode};let fired=false,nested;const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const trigger=mode==='fields-after'?'ID':undefined;
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(!trigger||t===trigger)){fired=true;switch(mode){
 case'name-kind':alpha.name='Changed Alpha';beta.name='Changed Beta';beta.type='Changed type';break;
 case'fields-before':alpha.requirementId='Changed id';alpha.text='Changed field';alpha.risk='Changed risk';element.docRef='Changed doc';break;
 case'fields-after':alpha.text='After capture';alpha.risk='After risk';beta.text='Second changed';break;
 case'styles-replace':beta.cssStyles=['fill:green'];break;
 case'styles-mutate':styles[0]='fill:green';break;
 case'classes':classes.push('after');beta.classes=['replacement'];break;
 case'maps-replace':m.requirements=new Map([['Beta',beta]]);m.elements=new Map();break;
 case'append':requirements.set('Extra',{...beta,name:'Extra',classes:[],cssStyles:[]});break;
 case'relations':relation.type='satisfies';relation.src='Beta';m.relations=[{type:'contains',src:'Service',dst:'Beta'}];break;
 case'direction':m.direction='TB';break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'throw':m.title='Before throw';alpha.text='Saved field';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(m),originalRequirements=copy(requirements),originalAlpha=copy(alpha),originalBeta=copy(beta),originalElements=copy(elements),originalElement=copy(element),originalRelations=copy(relations),originalRelation=copy(relation),originalStyles=copy(styles),originalClasses=copy(classes),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first,trace,after,originalRequirements,originalAlpha,originalBeta,originalElements,originalElement,originalRelations,originalRelation,originalStyles,originalClasses,requirementsCurrent:requirements===m.requirements,stylesCurrent:styles===beta.cssStyles,classesCurrent:classes===beta.classes,relationsCurrent:relations===m.relations,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
