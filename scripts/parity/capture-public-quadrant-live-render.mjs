import fs from 'node:fs';import path from 'node:path';import{pathToFileURL}from'node:url';import{createHash}from'node:crypto';
import{SOURCE,MODES,runQuadrant}from'../../tests/web/support/quadrant-live-scenarios.js';
const hash=p=>createHash('sha256').update(fs.readFileSync(p)).digest('hex');
const oracleSHA256='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';if(hash('node_modules/pele/dist/pele.js')!==oracleSHA256)throw Error('Oracle changed');
const core=await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/core.js'))),entry=(await import(pathToFileURL(path.resolve('node_modules/pele/dist/esm/diagrams/quadrantChart.js')))).default;core.register(entry);
const rows=[];for(const raw of[true,false])for(const responsive of[false,true])for(const mode of MODES)rows.push(runQuadrant(core,entry,()=>{},raw,responsive,mode));
const out={oracleSHA256,chunkSHA256:hash('node_modules/pele/dist/esm/chunks/RBHRHHSK.js'),scriptSHA256:hash(new URL(import.meta.url)),driverSHA256:hash(new URL('../../tests/web/support/quadrant-live-scenarios.js',import.meta.url)),source:SOURCE,rows};
fs.writeFileSync(process.argv[2]??'fixtures/public-quadrant-live-render.json',JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({cases:rows.length,unexpected:rows.filter(r=>!r.first.ok&&!r.first.original).map(r=>({mode:r.mode,error:r.first.error})),recoveryErrors:rows.filter(r=>r.recovery.error).map(r=>r.mode),pointCurrent:rows.find(r=>r.raw&&!r.responsive&&r.mode==='point-current').trace}));
