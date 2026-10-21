export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='eventmodeling\ntf 01 ui CartUI\ntf 02 cmd AddItem { productId: 7 }\ntf 03 evt ItemAdded\nnote 02 {\n First note\n}\ngwt 02 given cmd Requested when evt Confirmed then evt Saved';
export const MODES=['baseline','lane-label','lanes-replace','boxes-replace','box-name','box-data','box-late','box-append','relations','notes','specifications','title-accessibility','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});const copy=v=>JSON.parse(JSON.stringify(v));
export function runEventmodeling(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);m.title="Initial";
 const lanes=m.lanes,lane=lanes[0],boxes=m.boxes,first=boxes[0],second=boxes[1],relations=m.relations,relation=relations[0],notes=m.notes,note=notes[0],specifications=m.specifications,specification=specifications[0];
 const trace=[],sentinel={mode};let fired=false,nested;const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const trigger={'box-data':'AddItem','box-late':'Initial','notes':'Note · 02 AddItem','specifications':'Given / When / Then · 02 AddItem'}[mode];
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(!trigger||t===trigger)){fired=true;switch(mode){
 case'lane-label':lane.label='Changed lane';break;
 case'lanes-replace':m.lanes=[lanes[1],lanes[0],lanes[2]];break;
 case'boxes-replace':m.boxes=[second,boxes[2]];break;
 case'box-name':first.name='Changed first';second.name='Changed second';break;
 case'box-data':second.data='Changed data';second.name='After captured name';break;
 case'box-late':first.kind='processor';first.reset=true;first.id='Changed id';lane.label='Late lane';break;
 case'box-append':boxes.push({...boxes[2],id:'04',name:'Appended'});break;
 case'relations':relation.target=boxes[2];m.relations=[{source:first,target:boxes[2]}];break;
 case'notes':note.text='After captured note';notes.push({frame:'01',text:'Appended note'});break;
 case'specifications':specification.given.push('Added');specification.then=['Changed'];break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'throw':m.title='Before throw';first.name='Saved first';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let firstResult;try{firstResult={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){firstResult={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(m),originalLanes=copy(lanes),originalLane=copy(lane),originalBoxes=copy(boxes),originalFirst=copy(first),originalSecond=copy(second),originalRelations=copy(relations),originalRelation=copy(relation),originalNotes=copy(notes),originalNote=copy(note),originalSpecification=copy(specification),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first:firstResult,trace,after,originalLanes,originalLane,originalBoxes,originalFirst,originalSecond,originalRelations,originalRelation,originalNotes,originalNote,originalSpecification,lanesCurrent:lanes===m.lanes,boxesCurrent:boxes===m.boxes,relationsCurrent:relations===m.relations,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
