import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const sha=b=>createHash('sha256').update(b).digest('hex');
const oracleSHA256=sha(fs.readFileSync('node_modules/pele/dist/pele.js'));
if(oracleSHA256!=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d')throw Error('Oracle changed');
const core=await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/core.js'))),entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/xychart.js')))).default;core.register(entry);
const source='xychart\ntitle "Initial title"\nx-axis Month [Jan, Feb, Mar]\ny-axis Revenue -10 --> 20\nline Forecast [5 "PointA", 10 "PointB", 15 "PointC"]\nbar Sales [-5, 10, 15]';
const bits=n=>{const b=Buffer.alloc(8);b.writeDoubleBE(n);return b.toString('hex');};
const rows=[];
for(const raw of [false,true]){
 const model=raw?entry.parse(source,{},undefined,0,{}):core.parse(source),trace=[];
 const settings={padding:NaN,measurer:{width(t,s,style){trace.push(['width',t,bits(s),style??0]);return t.length*s*.6;}}};
 const config={xyChart:{showDataLabel:true}};
 const out=raw?entry.render(model,config,settings):core.render(model,{config,...settings});
 rows.push({raw,svg:out.svg,width:bits(out.width),height:bits(out.height),trace});
}
const upstream=fs.readFileSync('node_modules/pele/dist/esm/chunks/YO6PB664.js','utf8');
const loop='for(let N=0;N<m.categories.length;N+=R)';
if(!upstream.includes(loop))throw Error('Upstream loop changed');
const report={oracleSHA256,scriptSHA256:sha(fs.readFileSync(new URL(import.meta.url))),chunkSHA256:sha(upstream),loop,source,option:'padding: NaN',rows};
fs.writeFileSync(process.argv[2]??'fixtures/public-xychart-nan-category.json',JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(rows.map(({raw,svg,width,height,trace})=>({raw,svgBytes:svg.length,width,height,calls:trace.length}))));
