import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const oracleSHA256=sha('node_modules/pele/dist/pele.js');
if(oracleSHA256!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw Error('Oracle changed');
const core=await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/core.js')));
const entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/radar.js')))).default;
core.register(entry);
const source='radar-beta\ntitle Initial\naxis a["Alpha"], b["Beta"], c["Gamma"]\ncurve one["One"] {1,2,3}\ncurve two["Two"] {3,2,1}';
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const state=m=>({title:m.title??null,accTitle:m.accTitle??null,accDescr:m.accDescr??null,axes:m.axes,curves:m.curves,options:m.options});
const copy=v=>JSON.parse(JSON.stringify(v));
const rows=[];
for(const raw of [false,true])for(const responsive of [false,true])for(const mode of ['baseline','axis-edit','axes-replace','curve-edit','curves-replace','options','legend-mutation','title-accessibility','throw','nested']){
 const m=raw?entry.parse(source,{},undefined,0):core.parse(source),axes=m.axes,curves=m.curves,trace=[],sentinel={mode};let fired=false,nested;
 const pure={measurer:{width:(t,s)=>t.length*s*.6}};
 const options={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){trace.push([t,bits(s),style??0]);if(!fired&&(mode!=='legend-mutation'||t==='One')){fired=true;switch(mode){
 case'axis-edit':m.axes[0].name='changed-a';m.axes[1].name='changed-b';m.axes[1].label='Changed Beta';break;
 case'axes-replace':m.axes=[{name:'x',label:'X'},{name:'y',label:'Y'},{name:'z',label:'Z'}];break;
 case'curve-edit':m.curves[0].name='changed-one';m.curves[0].label='Changed One';m.curves[0].entries=[3,1,2];break;
 case'curves-replace':m.curves=[{name:'replacement',label:'Replacement',entries:[2,3,1]}];break;
 case'options':Object.assign(m.options,{ticks:2,min:1,max:4,graticule:'polygon',showLegend:false});break;
 case'legend-mutation':m.curves[0].name='late-one';m.curves[1].label='Late Two';m.title='Late title';break;
 case'title-accessibility':m.title='Changed title';m.accTitle='Changed access';m.accDescr='Changed description';break;
 case'throw':m.title='Before throw';m.axes[1].label='Saved Beta';throw sentinel;
 case'nested':m.title='Nested title';nested=drawing(entry.render(m,{},pure));m.title='After nested';break;
 }}return t.length*s*.6;}}};
 let first;try{first={ok:true,...drawing(raw?entry.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel,error:e===sentinel?'sentinel':{name:e.name,message:e.message}};}
 const after=copy(state(m)),originalAxes=copy(axes),originalCurves=copy(curves),recoveryTrace=[];
 const recoveryOptions={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=drawing(raw?entry.render(m,{},recoveryOptions):core.render(m,recoveryOptions));
 rows.push({raw,responsive,mode,first,trace,after,originalAxes,originalCurves,axesCurrent:axes===m.axes,curvesCurrent:curves===m.curves,...(nested?{nested}:{}),recovery,recoveryTrace});
}
const result={oracleSHA256,scriptSHA256:sha(new URL(import.meta.url)),chunkSHA256:sha('node_modules/pele/dist/esm/chunks/3GQJQJCW.js'),source,rows};
fs.writeFileSync(process.argv[2]??'fixtures/public-radar-live-render.json',JSON.stringify(result,null,2)+'\n');
console.log(JSON.stringify({rows:rows.length,originalThrows:rows.filter(r=>r.first.original).length,unexpected:rows.filter(r=>!r.first.ok&&!r.first.original).length}));
