export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='architecture-beta\ngroup g(cloud)[Cluster]\nservice a(server)[First] in g\nservice b(database)[Second] in g\nservice c(disk)[Outside]\na:R -- L:b\nb:B -- T:c';
export const MODES=['baseline','late-title','late-accessibility','later-label','replace-maps','append-node','edge-title','replace-edges','group-title','group-icon','node-icon','icon-group-id','icon-later-node','throw-object','throw-undefined','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const state=m=>JSON.parse(JSON.stringify({title:m.title,accTitle:m.accTitle,accDescr:m.accDescr,nodes:[...m.nodes],groups:[...m.groups],edges:m.edges,layoutHints:m.layoutHints}));
const markup='<path d="M0,0H2"/>';
export function runArchitecture(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title='Initial';
 const nodes=m.nodes,groups=m.groups,first=nodes.get('a'),later=nodes.get('b'),group=groups.get('g'),edges=m.edges;
 const pure={measurer:{width:(t,s)=>t.length*s*.6},icons:()=>markup},trace=[],sentinel={mode};let fired=false,nested;
 const execute=options=>raw?entry.render(m,{},options):core.render(m,options);
 function mutate(){switch(mode){
 case'late-title':m.title='Changed title';break;
 case'late-accessibility':m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'later-label':later.title='Changed second';break;
 case'replace-maps':m.nodes=new Map([['b',later],['c',nodes.get('c')]]);m.groups=new Map();break;
 case'append-node':nodes.set('d',{id:'d',type:'service',icon:'server',title:'Appended'});break;
 case'edge-title':edges[0].title='Changed edge';break;
 case'replace-edges':m.edges=[{...edges[1],title:'Replacement edge'}];break;
 case'group-title':group.title='Changed cluster';break;
 case'group-icon':group.icon='disk';break;
 case'node-icon':later.icon='internet';break;
 case'icon-group-id':group.id='changed-group';m.title='After icon';break;
 case'icon-later-node':later.title='Changed after icon';later.icon='internet';later.id='changed-node';break;
 case'throw-object':m.title='Saved';later.title='Saved later';throw sentinel;
 case'throw-undefined':m.title='Saved';later.title='Saved later';throw undefined;
 case'nested':m.title='Nested';nested=drawing(execute(pure));m.title='After nested';break;
 }}
 const options={...(responsive?{maxWidth:220}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&!mode.startsWith('icon-')&&t==='First'){fired=true;mutate();}return t.length*s*.6;}},icons(name){trace.push(['icon',name]);if(!fired&&mode.startsWith('icon-')&&name==='cloud'){fired=true;mutate();}return markup;}};
 let result;try{result={ok:true,...drawing(execute(options))};}catch(e){result={ok:false,original:mode==='throw-undefined'?e===undefined:e===sentinel,error:e===sentinel?'sentinel':e===undefined?'undefined':{name:e.name,message:e.message}};}
 const after=state(m),originalFirst=JSON.parse(JSON.stringify(first)),originalLater=JSON.parse(JSON.stringify(later)),originalGroup=JSON.parse(JSON.stringify(group)),recoveryTrace=[];
 const recovery=drawing(execute({...pure,...(responsive?{maxWidth:220}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}},icons(name){recoveryTrace.push(['icon',name]);return markup;}}));
 const record={raw,responsive,mode,fired,first:result,trace,after,originalFirst,originalLater,originalGroup,nodesCurrent:nodes===m.nodes,groupsCurrent:groups===m.groups,laterCurrent:later===m.nodes.get('b'),edgesCurrent:edges===m.edges,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return record;
}
