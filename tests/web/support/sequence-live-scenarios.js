export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='sequenceDiagram\nbox red Group\nparticipant A as Alpha\nparticipant B as Beta\nend\nautonumber\nloop Count\nA->>B: Hello\nNote over A,B: Detail\nB-->>A: Reply\nend';
export const MODES=['baseline','actor-label','actor-map-append','actor-map-delete','actor-map-reinsert','actor-map-replace','lifecycle','message-replace','message-append','message-fields','box-replace','box-late','actor-class','note-placement','frame-label','metadata','helper','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height),links:r.links??[]});
const copy=v=>JSON.parse(JSON.stringify(v,(_k,v)=>v instanceof Map?{$map:[...v]}:v));
export function runSequence(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title='Initial';
 const actors=m.actors,alpha=actors.get('A'),beta=actors.get('B'),box=alpha.box,messages=m.messages,hello=messages.find(v=>v.message==='Hello'),note=messages.find(v=>v.message==='Detail'),frame=messages.find(v=>v.message==='Count');
 const trace=[],sentinel={mode};let fired=false,nested;const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const trigger={'message-fields':'Hello','note-placement':'Detail','frame-label':'loop','box-late':'Group','actor-class':'Reply'}[mode];
 const options={...(responsive?{maxWidth:180}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(!trigger||t===trigger)){fired=true;switch(mode){
 case'actor-label':alpha.description='Changed Alpha';beta.description='Changed Beta';break;
 case'actor-map-append':actors.set('C',{name:'C',description:'Gamma',wrap:false,type:'participant',links:new Map(),properties:new Map()});break;
 case'actor-map-delete':actors.delete('B');break;
 case'actor-map-reinsert':actors.delete('A');actors.set('A',alpha);break;
 case'actor-map-replace':m.actors=new Map([['B',beta],['A',alpha]]);break;
 case'lifecycle':m.createdActors.set('B',messages.indexOf(hello));m.destroyedActors.set('B',messages.length-2);break;
 case'message-replace':m.messages=[];break;
 case'message-append':messages.push({...hello,id:'appended',message:'Appended'});break;
 case'message-fields':hello.id='changed-message';hello.centralConnection=61;hello.message='Changed Hello';note.message='Changed detail';break;
 case'box-replace':beta.box={name:'Replacement',fill:'green',wrap:false,actorKeys:['B']};break;
 case'box-late':box.name='Changed box';box.fill='green';break;
 case'actor-class':alpha.properties.set('class','late-alpha');beta.properties.set('class','late-beta');break;
 case'note-placement':note.placement=0;note.id='changed-note';break;
 case'frame-label':frame.message='Changed frame';break;
 case'metadata':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'helper':m.addActor('C','Gamma',{text:'Gamma',type:'participant'});break;
 case'throw':m.title='Before throw';beta.description='Saved Beta';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(m),originalActors=copy(actors),originalAlpha=copy(alpha),originalBeta=copy(beta),originalBox=copy(box),originalMessages=copy(messages),originalHello=copy(hello),originalNote=copy(note),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:180}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first,trace,after,originalActors,originalAlpha,originalBeta,originalBox,originalMessages,originalHello,originalNote,actorsCurrent:actors===m.actors,messagesCurrent:messages===m.messages,alphaCurrent:alpha===actors.get('A'),betaCurrent:beta===actors.get('B'),boxCurrent:box===beta.box,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
