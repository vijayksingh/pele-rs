export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='kanban\nTodo\n  a[First task]\n  b[Second task]\nDone\n  c[Third task]';
export const MODES=['baseline','header-edit','item-edit','item-replace','items-replace','sections-replace','item-append','section-append','section-replace','late-fields','throw','nested'];
export const ICON_MODES=['icon-label','icon-paint','icon-throw'];
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(v=>v.toString(16).padStart(2,'0')).join('');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height),links:r.links??[]});
const copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>({title:m.title??null,sections:m.sections});
const item=(id,label)=>({id,level:1,label});
const markup='<path d="M0 0h4v4z"/>';
export function runKanban(core,entry,dispose,source,raw,responsive,mode){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source);
 m.title='Initial';m.sections[0].icon='header';m.sections[0].items[0].icon='item';m.sections[0].items[0].ticket='T1';m.sections[0].items[0].assigned='Ada';
 const sections=m.sections,first=sections[0],items=first.items,original=items[0],trace=[],sentinel={mode};let fired=false,nested,icons=0;
 const pure={measurer:{width:(t,s)=>t.length*s*.6},icons:()=>markup};
 function mutate(){const current=m.sections[0];switch(mode){
  case'header-edit':current.label='Changed header';current.id='changed-header';break;
  case'item-edit':current.items[0].label='Changed first';current.items[1].label='Changed second';break;
  case'item-replace':current.items[1]=item('replacement','Replacement');break;
  case'items-replace':current.items=[item('replacement','Replacement')];break;
  case'sections-replace':m.sections=[{...item('new-root','New root'),level:0,items:[item('new','New')]}];break;
  case'item-append':current.items.push(item('append','Appended'));break;
  case'section-append':m.sections.push({...item('append-section','Appended section'),level:0,items:[item('append','Appended')]});break;
  case'section-replace':m.sections[1]={...item('replacement-section','Replacement section'),level:0,items:[item('replacement','Replacement')]};break;
  case'late-fields':original.id='late-id';original.priority='high';original.cssClasses='late-card';original.ticket='Changed ticket';original.assigned='Grace';first.cssClasses='late-column';m.title='Late title';break;
  case'throw':m.title='Before throw';original.label='Saved label';throw sentinel;
  case'nested':m.title='Nested title';nested=drawing(raw?entry.render(m,{},pure):core.render(m,pure));m.title='After nested';break;
  case'icon-label':original.label='After icon';original.ticket='After icon ticket';break;
  case'icon-paint':original.id='paint-id';original.cssClasses='paint-card';original.ticket='Paint ticket';first.items=[];break;
  case'icon-throw':original.label='Saved icon label';throw sentinel;
 }}
 const options={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){trace.push(['width',t,bits(s),style??0]);let trigger=mode==='item-append'?t==='First task':mode==='late-fields'?t==='Initial':true;if(!fired&&!mode.startsWith('icon-')&&trigger){fired=true;mutate();}return t.length*s*.6;}},icons(name){trace.push(['icon',name]);icons++;if(!fired&&mode.startsWith('icon-')&&((mode==='icon-paint'&&icons===3)||(mode!=='icon-paint'&&name==='item'))){fired=true;mutate();}return markup;}};
 let firstResult;try{firstResult={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){firstResult={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(m)),originalSections=copy(sections),originalItems=copy(items),originalItem=copy(original),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:300}:{}),measurer:{width(t,s,style){recoveryTrace.push(['width',t,bits(s),style??0]);return t.length*s*.6;}},icons(name){recoveryTrace.push(['icon',name]);return markup;}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 const result={raw,responsive,mode,fired,first:firstResult,trace,after,originalSections,originalItems,originalItem,sectionsCurrent:sections===m.sections,itemsCurrent:items===m.sections[0].items,itemCurrent:original===m.sections[0].items[0],...(nested?{nested}:{}),recovery,recoveryTrace};dispose(m);return result;
}
