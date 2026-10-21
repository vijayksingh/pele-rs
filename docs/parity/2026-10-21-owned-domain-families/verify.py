#!/usr/bin/env python3
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--git',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((root/'manifest.json').read_bytes());packed=(root/'evidence.json.gz').read_bytes();raw=gzip.decompress(packed)
assert(sha(packed),len(packed),sha(raw),len(raw))==(m['sha256'],m['bytes'],m['decodedSHA256'],m['decodedBytes'])
r=json.loads(raw);assert len(r)==m['records']
def data(name):return json.loads(r[name])
before=data('domain-family-native-before-build.json');built=data('domain-family-native-after-build.json');tested=data('domain-family-native-after-tests.json')
assert before['inputs']==built['inputs']==tested['inputs'] and len(built['inputs'])==383
assert built['sdk']==tested['sdk'] and len(built['sdk'])==13
assert 'Executed 24 tests, with 1 failure' in r['domain-family-swift-attempt4.log']
assert 'Executed 1 test, with 0 failures' in r['domain-family-swift-attempt5.log']
assert r['domain-family-swift-attempt4.log'].count(" : XCTAssertEqual failed:")==1
assert 'testErLiveJoinedAttributeTextBudgetPreservesWritesAndRecovery' in r['domain-family-swift-attempt5.log']
assert data('domain-family-validation-attempt5.json')['commands'][0]['exitCode']==0
assert 'native detached render lifecycle passed' in r['domain-family-c15.log']
assert 'PASS' in r['domain-family-c17.log'] and 'PASS' in r['domain-family-c18.log']
names=set()
for key in ['kanban-treemap-mindmap-native-attempt4.log','treemap-live-native-repair.log','er-cynefin-native-attempt3.log','cynefin-live-native-attempt1.log','wardley-native-attempt1.log','wardley-native-repair.log']:
 names.update(re.findall(r'^test (\S+) \.\.\. ok$',r[key],re.M))
assert len(names)==46,len(names)
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',r['domain-family-native-final.log'])))==8
web=data('domain-family-after-build.json');end=data('domain-family-after-tests.json')
assert web['inputs']==end['inputs'] and len(web['inputs'])==459
assert web['package']==end['package'] and len(web['package'])==118
assert web['wasmSHA256']==end['wasmSHA256']==m['wasmSHA256']
assert '54 passed' in r['domain-family-web-attempt1.log']
for family,count in [('kanban',54),('treemap',56),('mindmap',48),('er-owned',48),('cynefin',52),('wardley',60)]:
 reports=[json.loads(value) for key,value in r.items() if key.startswith('reports/') and key.endswith('/'+family+'-live-render.json')]
 assert len(reports)==3,(family,len(reports))
 for report in reports:
  assert report['before']==report['after'] and len(report['rows'])==count
  for row in report['rows']:assert row['candidate']==row['oracle']==row['fixture']
reports=[json.loads(value) for key,value in r.items() if key.startswith('reports/') and key.endswith('/er-join-budget.json')]
assert len(reports)==3
for report in reports:assert report['fired'] and report['retained'] and report['recovered'] and report['error']['code']=='limit'
audit=data('runtime-audit-domain-family.json');assert audit['package']==web['package']
assert audit['summary']['families']==33 and audit['summary']['callbackMatches']==17 and len(audit['summary']['callbackMismatches'])==16
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
if args.git:
 for files in[built['inputs'],web['inputs']]:
  for path,h in files.items():assert sha(subprocess.check_output(['git','show',m['runtimeCommit']+':'+path]))==h,path
print(f"Verified {len(r)} records; native46 slices/final8, Swift24 distinct features across recorded repair, C3, web54/954 new paired scenarios; audit17/33 matches.")
