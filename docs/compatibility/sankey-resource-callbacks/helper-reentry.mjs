import * as oracle from 'pele';
import {readFileSync,writeFileSync}from'node:fs';
import {loadRenderer}from'../source-typed-columns-runtime/candidate-package/full.js';
const engine=await loadRenderer(readFileSync(new URL('../source-typed-columns-runtime/candidate-package/pkg/pele_bg.wasm',import.meta.url))),rows=[];
try{for(const[lane,api]of[['oracle',oracle],['native',engine]]){const m=api.parse('sankey\nA,B,1');let fired=false,returned,output,error;try{output=api.render(m,{measurer:{width(t,s){if(!fired){fired=true;returned=m.findOrCreateNode('X');}return t.length*s*.6;}}});}catch(e){error={name:e?.name,message:e?.message};}rows.push({lane,fired,error,success:!!output,nodeCount:m.nodes.length,hasX:m.lookup.has('X'),helperAlias:returned!==undefined&&returned===m.lookup.get('X'),svg:output?.svg,width:output?.width,height:output?.height});if(lane==='native')engine.dispose(m);}}finally{engine.close();}
writeFileSync(new URL('./helper-reentry.json',import.meta.url),JSON.stringify(rows,null,2)+'\n');console.log(JSON.stringify(rows.map(({svg,...row})=>row)));
