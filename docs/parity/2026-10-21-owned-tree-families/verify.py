#!/usr/bin/env python3
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--git',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((root/'manifest.json').read_bytes());packed=(root/'evidence.json.gz').read_bytes();raw=gzip.decompress(packed)
assert (sha(packed),len(packed),sha(raw),len(raw))==(m['sha256'],m['bytes'],m['decodedSHA256'],m['decodedBytes'])
r=json.loads(raw);assert len(r)==m['records']
def data(name):return json.loads(r[name])
before=data('tree-family-native-before-build.json');built=data('tree-family-native-after-build.json');tested=data('tree-family-native-after-tests.json')
assert before['inputs']==built['inputs']==tested['inputs'] and len(built['inputs'])==376
assert built['sdk']==tested['sdk'] and len(built['sdk'])==13
assert 'Executed 17 tests, with 0 failures' in r['tree-family-swift-attempt1.log']
assert 'native detached render lifecycle passed' in r['tree-family-c15.log']
assert 'PASS' in r['tree-family-c17.log'] and 'PASS' in r['tree-family-c18.log']
names=set()
for key in ['venn-ishikawa-native-attempt1.log','timeline-live-native-attempt1.log','journey-live-native-attempt1.log','tree-journey-native-attempt1.log','owned-tree-native-attempt2.log']:names.update(re.findall(r'^test (\S+) \.\.\. ok$',r[key],re.M))
assert len(names)==45
assert 'FAILED' in r['tree-journey-native-attempt1.log']
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',r['owned-tree-native-attempt2.log'])))==6
web=data('tree-family-after-build.json');end=data('tree-family-after-tests.json')
assert web['inputs']==end['inputs'] and len(web['inputs'])==452
assert web['package']==end['package'] and len(web['package'])==118
assert web['wasmSHA256']==end['wasmSHA256']==m['wasmSHA256']
assert '39 passed' in r['tree-family-web-attempt1.log']
for family,count in [('venn',22),('ishikawa',48),('timeline',52),('journey',48),('treeview',64)]:
 reports=[json.loads(value) for key,value in r.items() if key.startswith('reports/') and key.endswith('/'+family+'-live-render.json')]
 assert len(reports)==3,(family,len(reports))
 for report in reports:
  assert report['before']==report['after'] and len(report['rows'])==count
  for row in report['rows']:assert row['candidate']==row['oracle']==row['fixture']
audit=data('runtime-audit-tree-family.json');assert audit['package']==web['package']
assert audit['summary']['families']==33 and audit['summary']['callbackMatches']==12 and len(audit['summary']['callbackMismatches'])==21
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
if args.git:
 for files in [built['inputs'],web['inputs']]:
  for path,h in files.items():assert sha(subprocess.check_output(['git','show',m['runtimeCommit']+':'+path]))==h,path
print(f"Verified {len(r)} records; native45 slices/final6, Swift17, C3, web39 with702 new paired scenarios; webaudit12/33 matches.")
