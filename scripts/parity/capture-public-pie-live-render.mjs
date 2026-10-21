import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
const hash=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const pin='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';
if(hash('node_modules/pele/dist/pele.js')!==pin)throw Error('Oracle changed');
const core=await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/core.js')));
const pie=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/pie.js')))).default;
core.register(pie);
const source='pie\ntitle Initial title\n"Alpha" : 1\n"Beta" : 2\n"Gamma" : 3';
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const state=m=>({title:m.title??null,showData:m.showData,accTitle:m.accTitle??null,accDescr:m.accDescr??null,sections:[...m.sections].map(([k,v])=>[k,bits(v)])});
const outcome=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)});
const rows=[];
for(const raw of [true,false])for(const responsive of [false,true])for(const mode of ['baseline','title','showData','map-edit','map-replace','accessibility','throw-object','throw-undefined','nested-render']){
 const m=raw?pie.parse(source,{},undefined,0):core.parse(source),old=m.sections,trace=[],sentinel={mode};let changed=false,nested;
 const options={...(responsive?{maxWidth:200}:{}),measurer:{width(text,size,style){trace.push([text,bits(size),style??0]);if(!changed){changed=true;switch(mode){case'title':m.title='Changed title';break;case'showData':m.showData=true;break;case'map-edit':m.sections.set('Beta',20);m.sections.delete('Gamma');m.sections.set('Delta',4);break;case'map-replace':m.sections=new Map([['Replacement',8]]);break;case'accessibility':m.accTitle='Changed access';m.accDescr='Changed description';break;case'throw-object':case'throw-undefined':m.title='Persisted before throw';m.showData=true;m.sections.set('Added',7);throw mode==='throw-undefined'?undefined:sentinel;case'nested-render':m.title='Nested title';nested=outcome(pie.render(m,{}, {measurer:{width:(t,s)=>t.length*s*.6}}));m.title='After nested';break;}}return text.length*size*.6;}}};
 let first;try{first={ok:true,...outcome(raw?pie.render(m,{},options):core.render(m,options))};}catch(e){first={ok:false,original:e===sentinel||e===undefined,error:e===undefined?'undefined':e===sentinel?'sentinel':{name:e?.name,message:e?.message}};}
 const after=state(m),recoveryTrace=[],recoverOptions={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){recoveryTrace.push([t,bits(s),style??0]);return t.length*s*.6;}}};
 const recovery=outcome(raw?pie.render(m,{},recoverOptions):core.render(m,recoverOptions));
 rows.push({raw,responsive,mode,first,trace,after,originalMapStillCurrent:old===m.sections,originalMap:[...old],nested,recovery,recoveryTrace});
}
const out={oracleSHA256:pin,pieChunkSHA256:hash('node_modules/pele/dist/esm/chunks/LAFB5P2O.js'),scriptSHA256:hash(new URL(import.meta.url)),source,rows};
fs.writeFileSync('dist/parity-audits/pie-live-render/oracle.json',JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({cases:rows.length,throws:rows.filter(x=>!x.first.ok).length,originalThrows:rows.filter(x=>x.first.original).length,showDataTrace:rows.find(x=>x.raw&&!x.responsive&&x.mode==='showData').trace,mapReplaceTrace:rows.find(x=>x.raw&&x.responsive&&x.mode==='map-replace').trace}));
