export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='usecase-beta\ndirection LR\nactor A("Person")\nU("Action")\nV("Later")\nA --> U\nU --> V\njson J@{"first":"One","second":"Two"}\nV --> J\nclassDef blue fill:#247acd,color:#ffffff\nclass U blue';
export const MODES=['baseline','late-title','late-accessibility','later-label','replace-maps','relationship','class-style','json-value','direction','throw-object','throw-undefined','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const state=m=>JSON.parse(JSON.stringify({title:m.title,accTitle:m.accTitle,accDescr:m.accDescr,direction:m.direction,actors:[...m.actors],useCases:[...m.useCases],relationships:m.relationships,jsonNodes:[...m.jsonNodes],classDefs:[...m.classDefs]}));
export function runUsecase(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title='Initial';
 const actors=m.actors,cases=m.useCases,later=cases.get('V'),relations=m.relationships,json=m.jsonNodes.get('J'),styles=m.classDefs.get('blue').styles;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}}, trace=[],sentinel={mode};let fired=false,nested;
 const execute=options=>raw?entry.render(m,{},options):core.render(m,options);
 const options={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&t==='Person'){fired=true;switch(mode){
 case'late-title':m.title='Changed title';break;
 case'late-accessibility':m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'later-label':later.label='Changed later';break;
 case'replace-maps':m.actors=new Map();m.useCases=new Map([['V',later]]);break;
 case'relationship':relations[0].label='Changed relation';relations[0].target='V';break;
 case'class-style':styles.push('fill:#ff0000');break;
 case'json-value':json.value.first='Changed JSON';break;
 case'direction':m.direction='TB';break;
 case'throw-object':m.title='Saved';later.label='Saved later';throw sentinel;
 case'throw-undefined':m.title='Saved';later.label='Saved later';throw undefined;
 case'nested':m.title='Nested';nested=drawing(execute(pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(execute(options))};}catch(e){first={ok:false,original:mode==='throw-undefined'?e===undefined:e===sentinel,error:e===sentinel?'sentinel':e===undefined?'undefined':{name:e.name,message:e.message}};}
 const after=state(m),recoveryTrace=[];const recovery=drawing(execute({...pure,...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}));
 const record={raw,responsive,mode,fired,first,trace,after,actorsCurrent:actors===m.actors,casesCurrent:cases===m.useCases,laterCurrent:later===m.useCases.get('V'),relationsCurrent:relations===m.relationships,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return record;
}
