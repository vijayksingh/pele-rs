import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const oracleSHA256=sha(fs.readFileSync('node_modules/pele/dist/pele.js'));
if(oracleSHA256!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw Error('Oracle changed');
const corpus=JSON.parse(fs.readFileSync('fixtures/pele/diagram-entries.json'));
const core=await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/core.js')));
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const drawing=r=>({svg:r.svg,width:bits(r.width),height:bits(r.height)}),rows=[];
for(const family of ['packet','radar','quadrantChart']){
 const source=corpus.cases.find(row=>row.family===family).source;
 const entry=(await import(pathToFileURL(path.resolve(`node_modules/pele/dist/esm/diagrams/${family}.js`)))).default;core.register(entry);
 for(const raw of [false,true])for(const responsive of [false,true])for(const mode of ['edit','throw','nested']){
  const model=raw?entry.parse(source,{},undefined,0):core.parse(source),sentinel={family,mode},trace=[];let fired=false,nested;
  const pure={measurer:{width:(t,s)=>t.length*s*.6}};
  const render=options=>raw?entry.render(model,{},options):core.render(model,options);
  const host={...(responsive?{maxWidth:200}:{}),measurer:{width(t,s,style){
   trace.push([t,bits(s),style??0]);
   if(!fired){fired=true;model.title='Changed title';model.accTitle='Changed access';model.accDescr='Changed description';
    if(mode==='throw')throw sentinel;
    if(mode==='nested'){nested=drawing(render(pure));model.title='After nested';}
   }
   return t.length*s*.6;
  }}};
  let first;try{first={ok:true,...drawing(render(host))};}catch(e){if(e!==sentinel)throw e;first={ok:false,original:true};}
  const recovery=drawing(render({...pure,...(responsive?{maxWidth:200}:{})}));
  rows.push({family,source,raw,responsive,mode,first,trace,title:model.title,accTitle:model.accTitle,accDescr:model.accDescr,...(nested?{nested}:{}),recovery});
 }
}
fs.writeFileSync(process.argv[2]??'fixtures/public-chart-callback-ownership.json',JSON.stringify({oracleSHA256,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),corpusSHA256:sha(fs.readFileSync('fixtures/pele/diagram-entries.json')),rows},null,2)+'\n');
console.log(JSON.stringify({rows:rows.length,originalThrows:rows.filter(row=>row.first.original).length}));
