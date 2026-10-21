#!/usr/bin/env python3
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--git',action='store_true');args=p.parse_args();root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((root/'manifest.json').read_bytes());packed=(root/'evidence.json.gz').read_bytes();raw=gzip.decompress(packed)
assert(sha(packed),len(packed),sha(raw),len(raw))==(m['sha256'],m['bytes'],m['decodedSHA256'],m['decodedBytes']);r=json.loads(raw);assert len(r)==m['records']
def data(n):return json.loads(r[n])
initial=data('flow-family-native-after-build.json');native=data('flow-family-native-after-build-repair.json');tested=data('flow-family-runtime-after-attempt2.json')
assert data('flow-family-native-before-build.json')['inputs']==initial['inputs']
assert data('flow-family-runtime-after-attempt1.json')['inputs']==initial['inputs']
assert data('flow-family-native-before-build-repair.json')['inputs']==native['inputs']==tested['inputs'] and len(native['inputs'])==404
assert native['sdk']==tested['sdk'] and len(native['sdk'])==13
assert [p for p in native['inputs'] if native['inputs'][p]!=initial['inputs'][p]]==['crates/pele-ffi/src/detached_render.rs']
assert all(c['exitCode']==0 for c in data('flow-family-validation-attempt2.json')['commands'])
assert 'Executed 36 tests, with 0 failures' in r['flow-family-swift-attempt2.log']
assert 'Native live iterable must be a Map' in r['flow-family-swift-attempt1.log']
for n in [15,17,18]:assert ('passed' if n==15 else 'PASS') in r[f'flow-family-repaired-c{n}.log']
names=set()
for n in ['flow-swimlane-xy-native-attempt2.log','flow-swimlane-expanded-style-limits-attempt2.log']:names.update(re.findall(r'^test (\S+) \.\.\. ok$',r[n],re.M))
assert len(names)==26,len(names)
web=data('flow-family-after-build.json');end=data('flow-family-after-tests.json');assert web['inputs']==end['inputs'] and len(web['inputs'])==480;assert web['package']==end['package'] and len(web['package'])==118;assert web['wasmSHA256']==m['wasmSHA256']
summary=data('flow-family-web-summary.json');assert summary['browser']['uniquePass']==30 and summary['browser']['attempt1']['harnessFailures']==6 and summary['browser']['attempt2']['pass']==6
for family,count in [('flowchart',60),('swimlane',60),('xychart',72)]:
 reports=[json.loads(v)for k,v in r.items()if k.startswith('reports/')and k.endswith('/'+family+'-live-render.json')];assert len(reports)==3,(family,len(reports))
 for report in reports:
  assert report['before']==report['after'] and len(report['rows'])==count
  for row in report['rows']:assert row['candidate']==row['oracle']==row['fixture']
audit=data('runtime-audit-flow-family.json');assert audit['package']==web['package'];assert audit['summary']['callbackMatches']==29 and len(audit['summary']['callbackMismatches'])==4
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
if args.git:
 for commit,files in[(m['nativeRuntimeCommit'],native['inputs']),(m['runtimeCommit'],web['inputs'])]:
  for path,h in files.items():assert sha(subprocess.check_output(['git','show',commit+':'+path]))==h,path
print(f'Verified {len(r)} records; native26, fresh Swift36/C3, web30/576 new paired rows; audit29/33.')
