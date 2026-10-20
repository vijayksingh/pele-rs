// New standalone consumer build. Never collects timing samples.
import {readFile,writeFile,mkdir} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync,spawn} from 'node:child_process';
import {resolve,dirname,relative,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
export const packageDir=dirname(fileURLToPath(import.meta.url));
export const sha=data=>createHash('sha256').update(data).digest('hex');
const arg=(name,fallback)=>{const i=process.argv.indexOf(name);return i<0?fallback:process.argv[i+1]};
export function repository(){return resolve(arg('--root',process.env.PELE_SWIFT_PERF_ROOT??resolve(packageDir,'../../..')))}
export function files(root,...paths){return execFileSync('rg',['--files','--hidden','--no-ignore',...paths],{cwd:root,encoding:'utf8'}).trim().split('\n').filter(Boolean).sort()}
export function systemProcesses(root=repository()) {
 return execFileSync('ps',['-ww','-Ao','pid=,pcpu=,comm='],{encoding:'utf8'}).trim().split('\n').map(line=>{
  const m=line.trim().match(/^(\d+)\s+([\d.]+)\s+(.*)$/);if(!m)return null;
  const name=m[3].split('/').pop();let testScope;
  // Wide command-only output preserves names containing spaces. Arguments are
  // inspected separately for test ownership and never enter reports.
  if(name==='xctest') {
   try {const args=execFileSync('ps',['-ww','-p',m[1],'-o','args='],{encoding:'utf8'});testScope=args.includes(root)||/Pele[^/\s]*\.xctest/.test(args)?'pele':'external'}
   catch {return null} // Process exited between the two snapshots.
  }
  return {pid:+m[1],cpuPercent:+m[2],name,...(testScope?{testScope}:{})};
 }).filter(Boolean);
}
export function guardQuiet(){const processes=systemProcesses();const active=processes.filter(p=>/^(cargo|rustc|swiftc|swift-build|swift-frontend|xcodebuild|ld)$/.test(p.name)||(p.name==='xctest'&&p.testScope==='pele'));assert.equal(active.length,0,'Build/Pele test process active; defer this operation: '+JSON.stringify(active));return processes.filter(p=>p.cpuPercent>=10||p.name==='xctest').sort((a,b)=>b.cpuPercent-a.cpuPercent).slice(0,20)}
export async function inputs(root){
 const sourceFiles=files(root,'crates/pele/src','crates/pele-ffi/src','crates/pele/assets','bindings/swift/Sources/Pele','include','third_party').concat(['Cargo.toml','Cargo.lock','crates/pele/Cargo.toml','crates/pele-ffi/Cargo.toml','rust-toolchain.toml','Package.swift','scripts/env.sh','scripts/build-native.sh','scripts/perf/cases.mjs']).sort();
 const hashes={};for(const p of sourceFiles)hashes['root/'+p]=sha(await readFile(join(root,p)));
 for(const p of ['Package.swift','Sources/PeleSwiftPerf/main.swift','build.mjs','run.mjs','README.md'])hashes['consumer/'+p]=sha(await readFile(join(packageDir,p)));
 const artifact=join(root,'bindings/swift/Artifacts/CPele.xcframework');
 const artifactFiles=files(root,artifact);
 assert(artifactFiles.some(p=>p.endsWith('.a')),'Source-built CPele archive required; old published binary is not selected');
 for(const p of artifactFiles)hashes['CPele/'+relative(artifact,p)]=sha(await readFile(p));
 const aggregate=createHash('sha256');for(const [p,h]of Object.entries(hashes).sort(([a],[b])=>a.localeCompare(b)))aggregate.update(p+'\0'+h+'\0');
 return {sha256:aggregate.digest('hex'),files:hashes,artifact,artifactFiles};
}
async function command(program,args,options={}){await new Promise((resolve,reject)=>{const child=spawn(program,args,{stdio:'inherit',...options});child.on('error',reject);child.on('exit',code=>code===0?resolve():reject(Error(program+' failed: '+code)))})}
async function main(){
 // Correctness builds may share the host. Before/after input hashes and the
 // linked SDK slice still establish freshness; only timed runs require quiet.
 const root=repository();
 const before=await inputs(root);
 const arch=process.arch==='arm64'?'arm64':process.arch==='x64'?'x86_64':null;assert(arch,'Unsupported Swift benchmark host architecture');
 const target=arch==='arm64'?'aarch64-apple-darwin':'x86_64-apple-darwin';
 // Cargo verifies freshness without replacing the SDK. A stale XCFramework fails.
 await command('/bin/bash',['-c','source scripts/env.sh && cargo build --locked --release -p pele-ffi --target "$1"','pele-swift-perf',target],{cwd:root,env:{...process.env,MACOSX_DEPLOYMENT_TARGET:'14.0'}});
 const archives=before.artifactFiles.filter(p=>p.endsWith('.a'));assert.equal(archives.length,1,'Expected one macOS CPele static archive');
 const archive=archives[0];
 const archs=execFileSync('xcrun',['lipo','-archs',archive],{encoding:'utf8'}).trim().split(/\s+/);assert(archs.includes(arch),'CPele does not contain host architecture');
 const scratch=join(packageDir,'.build');await mkdir(scratch,{recursive:true});let slice=archive;
 if(archs.length>1){slice=join(scratch,'CPele-host.a');execFileSync('xcrun',['lipo',archive,'-thin',arch,'-output',slice])}
 const nativeSha256=sha(await readFile(join(root,'target',target,'release/libpele_ffi.a')));assert.equal(sha(await readFile(slice)),nativeSha256,'SDK archive is stale: build native SDK before building this consumer');
 const env={...process.env,PELE_SWIFT_PERF_ROOT:root};
 await command('swift',['build','--package-path',packageDir,'--scratch-path',scratch,'-c','release'],{env});
 const binDir=execFileSync('swift',['build','--package-path',packageDir,'--scratch-path',scratch,'-c','release','--show-bin-path'],{env,encoding:'utf8'}).trim();
 const binary=join(binDir,'PeleSwiftPerf'),after=await inputs(root);assert.equal(before.sha256,after.sha256,'Sources/SDK changed during build; retry after freeze');
 const manifest={schemaVersion:1,builtAt:new Date().toISOString(),revision:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),root,sourceInputSha256:after.sha256,inputHashes:after.files,binary,binarySha256:sha(await readFile(binary)),nativeArchive:archive,nativeHostArchiveSha256:nativeSha256,nativeSourceFreshness:`cargo build --locked --release -p pele-ffi --target ${target}; host slice equals target/${target}/release/libpele_ffi.a`,rustTarget:target,swift:execFileSync('swift',['--version'],{encoding:'utf8'}).trim(),sdk:execFileSync('xcrun',['--show-sdk-version'],{encoding:'utf8'}).trim(),architecture:arch};
 await writeFile(join(scratch,'swift-build-manifest.json'),JSON.stringify(manifest,null,2)+'\n');console.log(join(scratch,'swift-build-manifest.json'));
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))await main();
