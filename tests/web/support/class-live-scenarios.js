export const SOURCE='classDiagram\ndirection LR\nclass A {\n+first: string\n+second: int\n+run() bool\n}\nclass B {\n+third: string\n}\nA --> B';
export const MODES=['baseline','title','accessibility','later-label','annotations','later-member','append-member','member-classifier','replace-classes','relation-label','late-tooltip','throw-object','throw-undefined','nested','replace-display','replace-classifier','display-throw','classifier-throw','display-reentry','width-classifier'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height),links:r.links});
const plain=v=>JSON.parse(JSON.stringify(v));
const state=m=>plain({title:m.title,accTitle:m.accTitle,accDescr:m.accDescr,direction:m.direction,classes:[...m.classes],relations:m.relations,notes:m.notes,namespaces:[...m.namespaces],interfaces:m.interfaces,styleClasses:[...m.styleClasses]});
export function runClass(core,entry,dispose,raw,responsive,mode){
 const m=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE);m.title='Initial';m.classes.get('A').label='Alpha';m.classes.get('B').label='Beta';
 const classes=m.classes,a=classes.get('A'),b=classes.get('B'),member=a.members[0],second=a.members[1],members=a.members,relations=m.relations;
 const execute=o=>raw?entry.render(m,{},o):core.render(m,o),base={...(responsive?{maxWidth:180}:{}),measurer:{width:(t,s)=>t.length*s*.6}},sentinel={mode},trace=[],calls=[];let fired=false,nested,inside=false;
 function mutate(){switch(mode){
 case'title':m.title='Changed';break;
 case'accessibility':m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'later-label':b.label='Changed beta';break;
 case'annotations':a.annotations.push('Changed annotation');break;
 case'later-member':second.id='changed: bool';break;
 case'append-member':members.push(new member.constructor('+appended: int','attribute'));break;
 case'member-classifier':member.classifier='*';break;
 case'replace-classes':m.classes=new Map([['B',b]]);break;
 case'relation-label':relations[0].title='Changed edge';break;
 case'late-tooltip':a.tooltip='Changed tooltip';a.link='https://example.test/class';a.linkTarget='_blank';break;
 case'throw-object':m.title='Saved';throw sentinel;
 case'throw-undefined':m.title='Saved';throw undefined;
 case'nested':m.title='Nested';nested=drawing(execute(base));m.title='After nested';break;
 case'replace-display':member.getDisplayDetails=function(){calls.push(['display',this===member]);return{displayText:'custom display',cssStyle:''};};break;
 case'replace-classifier':member.parseClassifier=function(){calls.push(['classifier',this===member]);this.id='changed in classifier';return'font-style:italic;';};break;
 case'display-throw':member.getDisplayDetails=function(){calls.push(['display',this===member]);m.title='Saved display';throw sentinel;};break;
 case'classifier-throw':member.parseClassifier=function(){calls.push(['classifier',this===member]);m.title='Saved classifier';throw undefined;};break;
 case'display-reentry':member.getDisplayDetails=function(){calls.push(['display',this===member]);if(!inside){inside=true;nested=drawing(execute(base));inside=false;}return{displayText:'custom nested',cssStyle:''};};break;
 case'width-classifier':member.classifier='*';break;
 }}
 let first;try{first={ok:true,...drawing(execute({...base,measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(mode==='width-classifier'?t.includes('first'):t==='Alpha')){fired=true;mutate();}return t.length*s*.6;}}}))};}catch(e){first={ok:false,original:mode==='throw-undefined'||mode==='classifier-throw'?e===undefined:e===sentinel,error:e===sentinel?'sentinel':e===undefined?'undefined':{name:e.name,message:e.message}};}
 const after=state(m),original=plain({a,b,member,second}),aliases={classes:classes===m.classes,a:a===m.classes.get('A'),members:members===a.members,member:member===a.members[0],relations:relations===m.relations};
 delete member.getDisplayDetails;delete member.parseClassifier;
 let recovery;try{recovery={ok:true,...drawing(execute(base))};}catch(e){recovery={ok:false,error:{name:e.name,message:e.message}};}
 const result={raw,responsive,mode,fired,first,trace,calls,after,original,aliases,...(nested?{nested}:{}),recovery};dispose(m);return result;
}
