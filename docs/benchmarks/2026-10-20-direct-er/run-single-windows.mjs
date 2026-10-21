import {spawn} from 'node:child_process';
import {writeFile,mkdir} from 'node:fs/promises';
import {createWriteStream} from 'node:fs';
import {guardQuiet} from '../../../../scripts/perf/swift/build.mjs';
const directory='dist/perf-patches/parse-architecture/direct-er';
await mkdir(directory,{recursive:true});
const events=[];
for(const stage of ['parseMutable','lifetime','parse'])for(const [caseIndex,workload] of ['sequence/small','er/medium'].entries()){
 let complete=false;
 for(let attempt=1;attempt<=6&&!complete;attempt++){
  const deadline=Date.now()+180000;let quiet=false;
  while(Date.now()<deadline){try{guardQuiet();quiet=true;break}catch{}await new Promise(r=>setTimeout(r,2000))}
  if(!quiet)throw Error('No compiler-free window in180s');
  const stem=`single-${stage}-${workload.replace('/','-')}-attempt${attempt}`;
  const output=`${directory}/swift-${stem}.json`,log=`${directory}/swift-${stem}.log`;
  const stream=createWriteStream(log),args=['scripts/perf/swift/compare.mjs','--control-dir',`${directory}/control-swift`,'--quiet-run','--cases',workload,'--stages',stage,'--order-offset',String(caseIndex),'--output',output];
  let text='';const code=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,args,{stdio:['ignore','pipe','pipe']});for(const pipe of [child.stdout,child.stderr])pipe.on('data',chunk=>{text+=chunk;stream.write(chunk)});child.on('error',reject);child.on('exit',resolve)});stream.end();
  complete=code===0;events.push({stage,workload,attempt,output,log,code,complete});await writeFile(`${directory}/single-windows.json`,JSON.stringify(events,null,2)+'\n');
  if(!complete&&!text.includes('Build/Pele test process active; defer this operation'))throw Error(`Non-guard failure: ${log}`);
  console.log(`${complete?'completed':'guard-rejected'} ${stage} ${workload} attempt${attempt}`);
 }
 if(!complete)throw Error('Six compiler-interrupted attempts; stop');
}
