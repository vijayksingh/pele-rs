#!/usr/bin/env python3
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--git',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((root/'manifest.json').read_bytes());packed=(root/'evidence.json.gz').read_bytes();raw=gzip.decompress(packed)
assert (sha(packed),len(packed),sha(raw),len(raw))==(m['sha256'],m['bytes'],m['decodedSHA256'],m['decodedBytes'])
r=json.loads(raw);assert len(r)==m['records']
def data(name):return json.loads(r[name])
before=data('chart-live-native-before-build.json');built=data('chart-live-native-after-build.json');tested=data('chart-live-native-after-tests.json')
assert before['inputs']==built['inputs']==tested['inputs'] and len(built['inputs'])==371
assert built['sdk']==tested['sdk'] and len(built['sdk'])==13
assert 'Executed 12 tests, with 0 failures' in r['chart-live-swift-attempt1.log']
assert 'Assertion failed: (!job)' in r['chart-live-c15.log']
assert 'native detached render lifecycle passed' in r['chart-live-c15-attempt2.log']
assert 'PASS' in r['chart-live-c17.log'] and 'PASS' in r['chart-live-c18.log']
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',r['chart-live-native-attempt2.log'])))==21
assert 'test result: ok. 1 passed' in r['quadrant-live-render-native-final.log']
web=data('chart-live-render-after-build.json');end=data('chart-live-render-after-tests.json')
assert web['inputs']==end['inputs'] and len(web['inputs'])==447
assert web['package']==end['package'] and len(web['package'])==118
assert web['wasmSHA256']==end['wasmSHA256']==m['wasmSHA256']
assert '24 passed' in r['chart-live-render-web-attempt1.log']
for family,count in [('packet',48),('radar',40),('quadrant',56)]:
 reports=[json.loads(value) for key,value in r.items() if key.startswith('reports/') and key.endswith('/'+family+'-live-render.json')]
 assert len(reports)==3,(family,len(reports))
 for report in reports:
  assert report['before']==report['after'] and len(report['rows'])==count
  for row in report['rows']:assert row['candidate']==row['oracle']==row['fixture']
audit=data('runtime-audit-charts.json');assert audit['package']==web['package']
assert audit['summary']['families']==33 and audit['summary']['callbackMatches']==7 and len(audit['summary']['callbackMismatches'])==26
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
if args.git:
 for files in [built['inputs'],web['inputs']]:
  for path,h in files.items():assert sha(subprocess.check_output(['git','show',m['runtimeCommit']+':'+path]))==h,path
print(f"Verified {len(r)} records; native21, Swift12, C3, browser24 with432 new paired scenarios; audit7/33 matches.")
