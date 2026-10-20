"""Read-only audit of completed correctness; writes private evidence records only."""
from pathlib import Path
import json,hashlib,re,datetime,subprocess
root=Path.cwd();base=root/'dist/perf-patches/parse-architecture/source-symbols-correctness';runtime=base/'runtime'
def read(p):return json.loads(Path(p).read_text())
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
records={p:read(runtime/(p+'.json'))for p in ['before-sdk','after-sdk','before-correctness','after-correctness']};b=records['before-sdk'];a=records['after-correctness'];commit='11f27516d98c3d89799a7fe2065eec0bd4e147cf';assert all(x['commit']==commit for x in records.values())
for x in records.values():
 for key in ['extraHashes','driverHashes','packageHashes']:assert x[key]==b[key],key
assert records['after-sdk']['current']==records['before-correctness']['current']==a['current']
changes=[k for k in b['current']['files']if b['current']['files'][k]!=a['current']['files'][k]];assert changes==['CPele/macos-arm64/libpele_ffi.a']
for key,h in a['current']['files'].items():
 if key.startswith('root/'):p=root/key[5:]
 elif key.startswith('consumer/'):p=root/'scripts/perf/swift'/key[9:]
 elif key.startswith('CPele/'):p=root/'bindings/swift/Artifacts/CPele.xcframework'/key[6:]
 else:raise AssertionError(key)
 assert sha(p)==h,key
for p,h in a['extraHashes'].items():assert sha(root/p)==h,p
for p,h in a['driverHashes'].items():assert sha(base/p)==h,p
keys=list(b['gitInputPaths']);blob=subprocess.check_output(['git','cat-file','--batch'],input=''.join(commit+':'+b['gitInputPaths'][k]+'\n'for k in keys).encode());offset=0
for key in keys:
 end=blob.index(b'\n',offset);header=blob[offset:end].split();assert header[1]==b'blob';size=int(header[2]);assert hashlib.sha256(blob[end+1:end+1+size]).hexdigest()==b['gitInputHashes'][key];offset=end+size+2
backup={str(p.relative_to(base/'prior-sdk')):sha(p)for p in(base/'prior-sdk').rglob('*')if p.is_file()};assert backup=={k[6:]:v for k,v in b['current']['files'].items()if k.startswith('CPele/')}
for directory,binary,manifest in [(base/'candidate-swift',a['binarySha256'],a['manifestSha256']),(root/b['controlDir'],b['controlBinarySha256'],b['controlManifestSha256']),(root/a['priorTypedControl']['directory'],a['priorTypedControl']['binarySha256'],a['priorTypedControl']['manifestSha256'])]:
 assert sha(directory/'PeleSwiftPerf')==binary and sha(directory/'swift-build-manifest.json')==manifest
saved=root/'dist/perf-patches/parse-architecture/source-sparse-ingress-correctness';savedRecord=read(saved/'control-sdk.json')
for p,h in savedRecord['files'].items():assert sha(saved/'control-sdk/CPele.xcframework'/p)==h
assert sha(root/'bindings/swift/Artifacts/CPele.xcframework/macos-arm64/libpele_ffi.a')==a['nativeArchiveSha256']
manifest=read(base/'candidate-swift/swift-build-manifest.json');assert manifest==a['manifest'] and manifest['inputHashes']==a['current']['files'] and manifest['sourceInputSha256']==a['current']['sha256']
commands=read(runtime/'correctness-commands.json');assert len(commands)==15 and all(x['exitCode']==0 for x in commands)
logs={x['log']:sha(root/x['log'])for x in commands};prefix='dist/evidence/source-symbols-correctness-'
native=(root/(prefix+'native-features.log')).read_text();counts=[int(x)for x in re.findall(r'test result: ok\. (\d+) passed; 0 failed;',native)];assert len(counts)==11 and sum(counts)==57
corpus=(root/(prefix+'swift-corpus.log')).read_text();assert 'Executed 16 tests, with 0 failures' in corpus and '12481 drawings, 16572 errors, 0 explicit budget differences' in corpus
api=read(root/(prefix+'swift-api.json'));assert api['mode']=='swift-api-correctness-only' and len(api['rows'])==168 and len(api['cases'])==21 and api['buildManifest']==manifest
for name,h in a['correctness']['cBinaryHashes'].items():assert sha(root/(prefix+name))==h
assert logs[prefix+'native-features.log']==a['correctness']['nativeLogSha256'];assert logs[prefix+'swift-corpus.log']==a['correctness']['swiftLogSha256'];assert sha(root/(prefix+'swift-api.json'))==a['correctness']['apiReportSha256']
proof=dict(verifiedAt=datetime.datetime.now(datetime.timezone.utc).isoformat(),sourceCommit=commit,nativeInputCount=len(a['current']['files']),gitInputMappings=len(keys),sourceInputSHA256=a['current']['sha256'],sdkSHA256=a['nativeArchiveSha256'],consumerSHA256=a['binarySha256'],consumerManifestSHA256=a['manifestSha256'],priorSDKFiles=len(backup),priorSDKArchiveSHA256=backup['macos-arm64/libpele_ffi.a'],sourceAndCorrectnessHashesVerified=True,protectedControlsVerified=True,correctness=a['correctness'],commands=len(commands),packageScope='Recorded 117-file preexisting a4 package stayed unchanged during correctness; not a fresh 11f WASM proof. Later packaging is outside this audit.',timing=False)
(runtime/'native-evidence-summary.json').write_text(json.dumps(proof,indent=2)+'\n')
paths=[runtime/(p+'.json')for p in records]+[runtime/n for n in ['run-correctness.py','freeze.mjs','correctness-commands.json','c-consumers.json','prior-sdk-verification.json','native-evidence-summary.json','collect-evidence.py']]+[root/p for p in logs]+[root/(prefix+'swift-api.json'),base/'candidate-swift/swift-build-manifest.json']
(runtime/'native-evidence-manifest.json').write_text(json.dumps(dict(sourceCommit=commit,files={str(p.relative_to(root)):dict(sha256=sha(p),bytes=p.stat().st_size)for p in paths}),indent=2)+'\n')
print(json.dumps(proof,indent=2))
