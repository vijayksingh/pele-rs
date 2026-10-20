"""One authorized consolidated correctness sequence; no elapsed benchmark collection."""
import subprocess,json,pathlib,datetime,os,sys
assert len(sys.argv)==3 and sys.argv[1]=='--parent-go','Wait for parent GO and committed source SHA'
commit=subprocess.check_output(['git','rev-parse',sys.argv[2]],text=True).strip()
base=pathlib.Path('dist/perf-patches/parse-architecture/shared-graph-traversal/runtime');sentinel=base/'attempt-started.json'
with sentinel.open('x') as f:json.dump({'startedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'commit':commit,'timing':False},f)
prefix='shared-graph-traversal';results=[]
def run(name,args,env=None):
 log=f'dist/evidence/{prefix}-{name}.log';start=datetime.datetime.now(datetime.timezone.utc).isoformat()
 with open(log,'x') as f:result=subprocess.run(args,stdout=f,stderr=subprocess.STDOUT,env={**os.environ,**(env or {})})
 item={'phase':name,'command':args,'environmentOverrides':env or {},'log':log,'startedAt':start,'endedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exitCode':result.returncode};results.append(item);(base/'correctness-commands.json').write_text(json.dumps(results,indent=2)+'\n');print(json.dumps(item),flush=True)
 if result.returncode:raise SystemExit(result.returncode)
def freeze(phase):run('freeze-'+phase,['node',str(base/'freeze.mjs'),phase,commit])
freeze('before-sdk')
run('sdk-build',['bash','scripts/build-native.sh','--host'],{'CARGO_BUILD_JOBS':'2'})
freeze('after-sdk')
run('swift-build',['node','scripts/perf/swift/build.mjs'],{'CARGO_BUILD_JOBS':'2'})
freeze('before-correctness')
run('swift-corpus',['swift','test','-c','release','--package-path','bindings/swift','--filter','AdditionalDiagramTests/testPinnedDiagramsThroughNativeDocuments|EagerProjectionIntegrationTests|PreparedDocumentTests|NonfiniteDiagramTests/testSankeyOwnedRenderAndCodableKeepNonfiniteHeight'],{'PELE_VERIFY_MUTABLE':'1','PELE_FAMILIES':'er,sankey'})
c=[]
for name in ['ffi','ffi-public','ffi-parse-limits']:
 binary=f'dist/evidence/{prefix}-{name}'
 run(name+'-compile',['cc','-Wall','-Wextra','-Werror','-std=c11','-I','include',f'tests/{name}.c','bindings/swift/Artifacts/CPele.xcframework/macos-arm64/libpele_ffi.a','-o',binary])
 run(name+'-run',[binary]);c.extend(results[-2:])
(base/'c-consumers.json').write_text(json.dumps(c,indent=2)+'\n')
run('swift-api',['node','scripts/perf/swift/run.mjs','--output',f'dist/evidence/{prefix}-swift-api.json'])
freeze('after-correctness')
