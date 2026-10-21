export const ORACLE_SHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
export const SOURCE='radar-beta\naxis a["Alpha"], b["Beta"], c["Gamma"]\ncurve one["One"] {1,2,3}\ncurve two["Two"] {3,2,1}';
const bits=n=>{const b=new ArrayBuffer(8);new DataView(b).setFloat64(0,n,false);return [...new Uint8Array(b)].map(x=>x.toString(16).padStart(2,'0')).join('');};
const drawing=v=>({svg:v.svg,width:bits(v.width),height:bits(v.height),links:v.links});
const copy=v=>JSON.parse(JSON.stringify(v));
const state=m=>({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,axes:m.axes,curves:m.curves,options:m.options});
export function cases(){return [false,true].flatMap(raw=>['hole-circle','hole-polygon','hole-polygon-no-grid','hole-skipped-curves','hole-no-curves','all-holes','empty-circle','empty-circle-no-legend','empty-circle-skipped','empty-polygon','callback-hole','callback-throw'].map(mode=>({raw,mode})));}
export function runRadar(core,entry,dispose,raw,mode){
 const m=raw?entry.parse(SOURCE,{},undefined,0):core.parse(SOURCE),axes=m.axes,curves=m.curves,original=copy(m),trace=[],sentinel={mode};let fired=false;
 if(mode.startsWith('hole-'))delete axes[1];
 if(mode==='all-holes'){delete axes[0];delete axes[1];delete axes[2];}
 if(mode.includes('polygon'))m.options.graticule='polygon';
 if(mode==='hole-polygon-no-grid')m.options.ticks=0;
 if(mode==='hole-skipped-curves')for(const c of curves)c.entries=[1,2];
 if(mode==='hole-no-curves')curves.length=0;
 if(mode.startsWith('empty-')){axes.length=0;curves.length=1;if(mode!=='empty-circle-skipped')curves[0].entries=[];}
 if(mode==='empty-circle-no-legend')m.options.showLegend=false;
 const invoke=options=>raw?entry.render(m,{},options):core.render(m,options);
 const options={measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired){fired=true;if(mode.startsWith('callback-')){delete axes[1];m.title='Kept mutation';if(mode==='callback-throw')throw sentinel;}}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(invoke(options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(m)),oldAxes=copy(axes),oldCurves=copy(curves),aliases={axes:m.axes===axes,curves:m.curves===curves};
 m.axes=copy(original.axes);m.curves=copy(original.curves);m.options=copy(original.options);m.title=original.title;
 const recoveryTrace=[],recovery=drawing(invoke({measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}}));
 const result={raw,mode,first,trace,fired,after,oldAxes,oldCurves,aliases,recovery,recoveryTrace};dispose(m);return result;
}
