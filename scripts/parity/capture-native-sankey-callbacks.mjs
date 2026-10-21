import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const file='node_modules/pele/dist/pele.js',sha=b=>createHash('sha256').update(b).digest('hex');
const oracleSHA256=sha(await readFile(file));assert.equal(oracleSHA256,'d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d');
const oracle=await import(new URL('../../node_modules/pele/dist/pele.js',import.meta.url));
const source='sankey\nA,B,5\nB,C,3',pure=(text,size)=>text.length*size*.6;
const summary=r=>({svg:r.svg,svgSHA256:sha(Buffer.from(r.svg)),width:r.width,height:r.height,type:r.type,links:r.links});
const rows=[];
for(const mode of ['helper','throw','nested']){
 const model=oracle.parse(source),sentinel={failure:'original callback'},row={mode,measurements:[]};let entered=false;
 const options={measurer:{width(text,size,style){row.measurements.push({text,size,style});if(!entered){entered=true;if(mode==='throw'){model.nodes[1].id='Changed B';model.title='Changed title';throw sentinel;}model.findOrCreateNode('D');if(mode==='nested')row.nested=summary(oracle.render(model,{measurer:{width:pure}}));}return pure(text,size);}}};
 try{row.first=summary(oracle.render(model,options));row.originalError=false;}catch(error){assert.equal(error,sentinel);row.originalError=true;}
 row.nodes=model.nodes.map(node=>({id:node.id,index:node.index}));
 row.recovery=summary(oracle.render(model,{measurer:{width:pure}}));rows.push(row);
}
await writeFile('fixtures/native-sankey-callbacks.json',JSON.stringify({oracleRevision:'458322a5b8abb1ae57ddf127c32017ab9fb14a92',oracleSHA256,source,rows},null,2)+'\n');
console.log(JSON.stringify({cases:rows.length,oracleSHA256,fixtureSHA256:sha(await readFile('fixtures/native-sankey-callbacks.json'))}));
