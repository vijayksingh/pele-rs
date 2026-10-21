export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='railroad-beta\nentry = sequence(terminal("Alpha"), nonterminal("Other"), zeroOrMore(terminal("Beta")));\nOther = choice(terminal("x"), special("y"));';
export const MODES=['baseline','title-accessibility','rules-replace','rule-name','definition-replace','element-text','children-replace','children-append','type-change','repetition','rules-append','shared-identity','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const state=m=>({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,rules:m.rules});
const copy=v=>JSON.parse(JSON.stringify(v));
export function runRailroad(core,entry,dispose,source,raw,responsive,mode){
 const model=raw?entry.parse(source,{},undefined,0):core.parse(source);model.title='Initial';
 const rules=model.rules,rule=rules[0],definition=rule.definition,children=definition.elements,first=children[0],repetition=children[2];
 if(mode==='shared-identity')children[1]=first;
 const trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const options={...(responsive?{maxWidth:180}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);const target=mode==='repetition'?'Beta':mode==='rules-append'?'entry':mode==='title-accessibility'||mode==='nested'||mode==='baseline'?'Initial':'Alpha';if(!fired&&t===target){fired=true;switch(mode){
 case'title-accessibility':model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';break;
 case'rules-replace':model.rules=[{name:'Replacement',definition:{type:'terminal',value:'new'}}];break;
 case'rule-name':rule.name='Changed entry';break;
 case'definition-replace':rule.definition={type:'terminal',value:'Replacement definition'};break;
 case'element-text':first.value='After captured Alpha';children[1].name='Changed Other';break;
 case'children-replace':definition.elements=[{type:'terminal',value:'Replacement child'}];break;
 case'children-append':children.push({type:'terminal',value:'Appended child'});break;
 case'type-change':definition.type='choice';definition.alternatives=children;break;
 case'repetition':repetition.min=2;repetition.max=4;break;
 case'rules-append':rules.push({name:'Appended',definition:{type:'terminal',value:'new'}});break;
 case'shared-identity':first.value='Second occurrence';break;
 case'throw':model.title='Before throw';first.value='Saved Alpha';throw sentinel;
 case'nested':model.title='Nested title';nested=drawing(raw?entry.render(model,{},pure):core.render(model,pure));model.title='After nested';break;
 }}return t.length*s*.6;}}};
 let result;try{result={ok:true,...drawing(raw?entry.render(model,{},options):core.render(model,options))};}catch(e){result={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(model)),originalRules=copy(rules),originalRule=copy(rule),originalDefinition=copy(definition),originalChildren=copy(children),originalFirst=copy(first),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:180}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(model,{},recoveryOptions):core.render(model,recoveryOptions));
 const record={raw,responsive,mode,fired,first:result,trace,after,originalRules,originalRule,originalDefinition,originalChildren,originalFirst,rulesCurrent:rules===model.rules,ruleCurrent:rule===model.rules[0],definitionCurrent:definition===model.rules[0].definition,childrenCurrent:children===model.rules[0].definition.elements,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(model);return record;
}
