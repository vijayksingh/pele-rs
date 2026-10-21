export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='timeline\ntitle Initial\nsection Before\nPeriod One : Event One\nPeriod Two : Event Two\nsection Later\nPeriod Three : Event Three';
export const MODES=['baseline','period-edit','period-replace','periods-replace','period-append','events-edit','events-replace','events-append','sections','title-accessibility','refit','throw','nested'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,direction:m.direction,sections:m.sections,periods:m.periods});
export function runTimeline(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);
 if(mode==='refit')m.periods[0].text='A'.repeat(80);
 const periods=m.periods,sections=m.sections,events=m.periods[0].events,trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(mode!=='events-append'||t==='Event One')){fired=true;switch(mode){
 case'period-edit':m.periods[0].text='Changed first';m.periods[1].text='Changed second';m.periods[0].sectionIndex=1;break;
 case'period-replace':m.periods[1]={text:'Replacement',events:['Replacement event'],section:'Before',sectionIndex:0};break;
 case'periods-replace':m.periods=[{text:'New root period',events:['New root event'],section:'Before',sectionIndex:0}];break;
 case'period-append':m.periods.push({text:'Appended',events:['Appended event'],section:'Later',sectionIndex:1});break;
 case'events-edit':m.periods[1].events[0]='Changed event';break;
 case'events-replace':m.periods[0].events=['New events','Second event'];break;
 case'events-append':m.periods[0].events.push('Appended event');break;
 case'sections':m.sections=['Changed section','Changed later'];break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';m.direction='TD';break;
 case'refit':m.periods[0].text='After refit';m.periods[1].text='Changed second';break;
 case'throw':m.title='Before throw';m.periods[1].text='Saved period';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(entry.render(m,{},pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(m)),originalPeriods=copy(periods),originalSections=copy(sections),originalEvents=copy(events),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,first,trace,after,originalPeriods,originalSections,originalEvents,periodsCurrent:periods===m.periods,sectionsCurrent:sections===m.sections,eventsCurrent:events===m.periods[0].events,...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
