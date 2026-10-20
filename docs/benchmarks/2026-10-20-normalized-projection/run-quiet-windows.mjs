import {spawn} from 'node:child_process';
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createWriteStream} from 'node:fs';
import {guardQuiet} from '../../../../scripts/perf/swift/build.mjs';
const directory='dist/perf-patches/parse-architecture/normalized-projection';
await mkdir(directory,{recursive:true});
const events=JSON.parse(await readFile(`${directory}/quiet-windows.json`));
for(const stage of ['parse','render-source','render-retained'])for(const workload of ['sequence/small','gantt/medium']){
 let complete=false;
 for(let attempt=2;attempt<=7&&!complete;attempt++){
  const deadline=Date.now()+180000;let quiet=false;
  while(Date.now()<deadline){try{guardQuiet();quiet=true;break}catch{}await new Promise(r=>setTimeout(r,2000))}
  if(!quiet)throw Error('No compiler-free window in180s');
  const stem=`${stage}-${workload.replace('/','-')}-attempt${attempt}`;
  const output=`${directory}/swift-${stem}.json`,log=`${directory}/swift-${stem}.log`;
  const stream=createWriteStream(log),args=['scripts/perf/swift/compare.mjs','--control-dir',`${directory}/control-swift`,'--quiet-run','--cases',workload,'--stages',stage,'--output',output];
  let text='';const code=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,args,{stdio:['ignore','pipe','pipe']});for(const pipe of [child.stdout,child.stderr])pipe.on('data',chunk=>{text+=chunk;stream.write(chunk)});child.on('error',reject);child.on('exit',resolve)});stream.end();
  complete=code===0;events.push({stage,workload,attempt,output,log,code,complete});await writeFile(`${directory}/quiet-windows.json`,JSON.stringify(events,null,2)+'\n');
  if(!complete&&!text.includes('Build/Pele test process active; defer this operation'))throw Error(`Non-guard failure: ${log}`);
  console.log(`${complete?'completed':'guard-rejected'} ${stage} ${workload} attempt${attempt}`);
 }
 if(!complete)throw Error('Six compiler-interrupted attempts; stop');
}
