import {guardQuiet} from '../../../../scripts/perf/swift/build.mjs';
import {spawn} from 'node:child_process';
import {open,writeFile} from 'node:fs/promises';
const base='dist/perf-patches/parse-architecture/owned-public-construction/runtime';
const logBase='dist/evidence/owned-public-construction';
const report={startedAt:new Date().toISOString(),strictGuardUnchanged:true,phases:[]};
const save=()=>writeFile(base+'/timing-schedule.json',JSON.stringify(report,null,2)+'\n');
const sleep=ms=>new Promise(r=>setTimeout(r,ms));
async function waitQuiet(phase){const deadline=Date.now()+240000;let clean=false;while(Date.now()<deadline){try{guardQuiet();if(clean)return;clean=true;await sleep(5000)}catch(error){clean=false;phase.deferrals.push({at:new Date().toISOString(),reason:String(error.message)});await save();await sleep(15000)}}throw Error('No coordinated timing window within four minutes')}
async function run(name,args){const phase={name,startedAt:new Date().toISOString(),deferrals:[]};report.phases.push(phase);await save();await waitQuiet(phase);phase.measurementStartedAt=new Date().toISOString();await save();console.log('Starting '+name+' with strict guard');const log=await open(logBase+'-'+name+'-timing-valid.log','w');try{await new Promise((resolve,reject)=>{const child=spawn(process.execPath,args,{stdio:['ignore',log.fd,log.fd]});child.on('error',reject);child.on('exit',code=>code===0?resolve():reject(Error(name+' timing rejected; exit '+code)))})}finally{await log.close()}phase.completedAt=new Date().toISOString();phase.succeeded=true;await save();console.log('Completed '+name)}
try{
await run('native',['--expose-gc','scripts/perf/compare-native.mjs','--quiet-run','--control','dist/perf-patches/parse-architecture/er-dense-array/candidate-native/pele-perf','--candidate','dist/perf-patches/parse-architecture/owned-public-construction/candidate-native/pele-perf','--cases','sankey/medium,sequence/medium','--stages','parse-public,parse,render-source,render-retained','--batch-count','16','--trials','15','--warmup-ms','1000','--output',base+'/native-timing.json']);
await run('swift',['scripts/perf/swift/compare.mjs','--quiet-run','--control-dir','dist/perf-patches/parse-architecture/er-dense-array/candidate-swift','--cases','sankey/medium,sequence/medium','--stages','parseMutable,lifetime,parse','--output',base+'/swift-timing.json']);
report.succeeded=true;report.completedAt=new Date().toISOString();await save();
}catch(error){report.failedAt=new Date().toISOString();report.error=String(error);await save();throw error}
