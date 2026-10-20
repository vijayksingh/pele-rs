import {spawn} from 'node:child_process';
import {writeFile} from 'node:fs/promises';
import {createWriteStream} from 'node:fs';
import {guardQuiet} from '../../../../scripts/perf/swift/build.mjs';
const directory='dist/perf-patches/parse-architecture/er-borrowed-tokens',events=[];
for(const [workload,stage] of [['er/medium','parse'],['er/medium','parse-public'],['er/medium','render-source'],['er/medium','render-retained'],['gantt/medium','parse-public']]){
 let complete=false;
 for(let attempt=1;attempt<=6&&!complete;attempt++){
  const deadline=Date.now()+180000;let quiet=false;
  while(Date.now()<deadline){try{guardQuiet();quiet=true;break}catch{}await new Promise(r=>setTimeout(r,2000))}
  if(!quiet)throw Error('No compiler-free native timing window in180s');
  const stem=`native-${stage}-${workload.replace('/','-')}-attempt${attempt}`,output=`${directory}/${stem}.json`,log=`${directory}/${stem}.log`;
  const stream=createWriteStream(log);let text='';
  const args=['--expose-gc','scripts/perf/compare-native.mjs','--quiet-run','--control','dist/perf-patches/parse-architecture/direct-er/self-control-native/pele-perf','--candidate','scripts/perf/native/target/release/pele-perf','--cases',workload,'--stages',stage,'--trials','15','--warmup-ms','1000','--batch-count','16','--output',output];
  const code=await new Promise((resolve,reject)=>{const child=spawn(process.execPath,args,{stdio:['ignore','pipe','pipe']});for(const pipe of [child.stdout,child.stderr])pipe.on('data',chunk=>{text+=chunk;stream.write(chunk)});child.on('error',reject);child.on('exit',resolve)});stream.end();
  complete=code===0;events.push({workload,stage,attempt,output,log,code,complete});await writeFile(`${directory}/native-windows.json`,JSON.stringify(events,null,2)+'\n');
  if(!complete&&!text.includes('Build/Pele test process active; defer this operation'))throw Error(`Non-guard native failure: ${log}`);
  console.log(`${complete?'completed':'guard-rejected'} ${workload} ${stage} attempt${attempt}`);
 }
 if(!complete)throw Error('Six interrupted native windows; stop');
}
