import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
const root='dist/perf-patches/parse-architecture/disposal-receipt-runtime';
const sha=data=>createHash('sha256').update(data).digest('hex');
const phase=process.argv[2];
assert(['source','package','after'].includes(phase));
const paths=execFileSync('git',['ls-files','--cached','--others','--exclude-standard','--','Cargo.toml','Cargo.lock','rust-toolchain.toml','crates','bindings/web','scripts/build-wasm.sh','scripts/generate-source-initializer.mjs','scripts/perf/compare-web-vm.mjs','scripts/perf/compare-first-use-web-vm.mjs'],{encoding:'utf8'}).trim().split('\n').sort();
const sources=Object.fromEntries(await Promise.all(paths.map(async path=>[path,sha(await readFile(path))])));
async function hashes(dir,prefix='') {
  const result={};
  for(const entry of (await readdir(dir+'/'+prefix,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))) {
    const path=prefix+entry.name;
    if(entry.isDirectory())Object.assign(result,await hashes(dir,path+'/'));
    else {assert(entry.isFile());result[path]=sha(await readFile(dir+'/'+path));}
  }
  return result;
}
const record={recordedAt:new Date().toISOString(),phase,commit:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),sourceStatus:execFileSync('git',['status','--short'],{encoding:'utf8'}),sources};
if(phase!=='source') {
  const before=JSON.parse(await readFile(root+'/source.json'));
  assert.deepEqual(sources,before.sources,'Source inputs changed after freeze');
  record.package=await hashes('dist/package-web/package');
  if(phase==='after') {
    const packed=JSON.parse(await readFile(root+'/package.json'));
    assert.deepEqual(record.package,packed.package,'Package changed during runtime checks');
    // Bind every source input to the final immutable commit after integration.
    const bytes=execFileSync('git',['cat-file','--batch'],{input:paths.map(p=>record.commit+':'+p+'\n').join(''),maxBuffer:100*1024*1024});
    let offset=0;
    for(const path of paths) {
      const end=bytes.indexOf(10,offset),header=bytes.subarray(offset,end).toString().split(' ');
      assert.equal(header[1],'blob',path);
      const size=Number(header[2]);assert.equal(sha(bytes.subarray(end+1,end+1+size)),sources[path],path+' differs from commit');
      offset=end+size+2;
    }
    record.gitInputsVerified=paths.length;
  }
}
await writeFile(root+'/'+phase+'.json',JSON.stringify(record,null,2)+'\n');
console.log(JSON.stringify({phase,commit:record.commit,sourceInputs:paths.length,packageFiles:record.package&&Object.keys(record.package).length,wasm:record.package?.['pkg/pele_bg.wasm'],gitInputsVerified:record.gitInputsVerified}));
