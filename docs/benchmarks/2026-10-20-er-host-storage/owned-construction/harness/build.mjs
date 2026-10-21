// Build consumer binaries and bind them to exact source inputs. Never runs timings.
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync,spawn} from 'node:child_process';
import {resolve,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
process.chdir(resolve(dirname(fileURLToPath(import.meta.url)),'../..'));
const sha=b=>createHash('sha256').update(b).digest('hex');
export async function inputs(){const files=execFileSync('rg',['--files','crates/pele/src','scripts/perf/native/src'],{encoding:'utf8'}).trim().split('\n').concat(['crates/pele/Cargo.toml','Cargo.toml','scripts/perf/native/Cargo.toml','scripts/perf/native/Cargo.lock']).sort();const h=createHash('sha256');for(const p of files){h.update(p+'\0');h.update(await readFile(p))}return h.digest('hex')}
const before=await inputs();
await new Promise((resolve,reject)=>{const p=spawn('/bin/bash',['-c','source scripts/env.sh && cargo build --locked --release --manifest-path scripts/perf/native/Cargo.toml'],{stdio:'inherit'});p.on('error',reject);p.on('exit',code=>code===0?resolve():reject(Error('cargo build failed: '+code)))});
const after=await inputs();if(before!==after)throw Error('Source changed during build; rerun after agents freeze');
const manifest={schemaVersion:1,sourceInputSha256:after,revision:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),builtAt:new Date().toISOString(),binaries:{}};
for(const name of ['pele-perf','pele-perf-alloc'])manifest.binaries[name]=sha(await readFile('scripts/perf/native/target/release/'+name));
await writeFile('scripts/perf/native/target/release/build-manifest.json',JSON.stringify(manifest,null,2)+'\n');
