#!/usr/bin/env python3
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--git',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((root/'manifest.json').read_bytes());packed=(root/'evidence.json.gz').read_bytes();raw=gzip.decompress(packed)
assert(sha(packed),len(packed),sha(raw),len(raw))==(m['sha256'],m['bytes'],m['decodedSHA256'],m['decodedBytes']);r=json.loads(raw);assert len(r)==m['records']
def data(name):return json.loads(r[name])
before=data('extended-domain-native-before-build.json');built=data('extended-domain-native-after-build.json');tested=data('extended-domain-native-after-tests.json')
assert before['inputs']==built['inputs']==tested['inputs'] and len(built['inputs'])==400
assert built['sdk']==tested['sdk'] and len(built['sdk'])==13
for phase in ['before','after-attempt1','before-attempt2','after-attempt2']:
 runtime=data('extended-domain-runtime-'+phase+'.json');assert runtime['inputs']==built['inputs'] and runtime['sdk']==built['sdk']
assert 'Executed 33 tests, with 228 failures' in r['extended-domain-swift-attempt1.log']
assert set(re.findall(r'error: -\[PeleTests\.([^ ]+)',r['extended-domain-swift-attempt1.log']))=={'NativeRailroadCallbackTests'}
assert 'Executed 1 test, with 0 failures' in r['extended-domain-swift-attempt2.log']
assert data('extended-domain-validation-attempt2.json')['commands'][0]['exitCode']==0
assert all(c['exitCode']==0 for c in data('extended-domain-validation-attempt1.json')['commands'][1:])
assert 'native detached render lifecycle passed' in r['extended-domain-c15.log']
assert 'PASS' in r['extended-domain-c17.log'] and 'PASS' in r['extended-domain-c18.log']
names=set()
for key in ['c4-requirement-native-attempt2.log','railroad-native-attempt2.log','graph-family-native-attempt1.log','usecase-live-native-repair.log','architecture-state-block-native-attempt3.log']:
 names.update(re.findall(r'^test (\S+) \.\.\. ok$',r[key],re.M))
assert len(names)==75,len(names)
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',r['extended-domain-native-final.log'])))==11
web=data('extended-domain-after-build.json');end=data('extended-domain-after-tests.json')
assert web['inputs']==end['inputs'] and len(web['inputs'])==476
assert web['package']==end['package'] and len(web['package'])==118
assert web['wasmSHA256']==end['wasmSHA256']==m['wasmSHA256']
assert '45 passed' in r['extended-domain-web-attempt1.log']
for family,count in [('c4',48),('requirement',56),('railroad',56),('gitgraph',48),('usecase',48),('eventmodeling',56),('architecture',64),('state',64),('block',56)]:
 reports=[json.loads(value) for key,value in r.items() if key.startswith('reports/') and key.endswith('/'+family+'-live-render.json')]
 assert len(reports)==3,(family,len(reports))
 for report in reports:
  assert report['before']==report['after'] and len(report['rows'])==count
  for row in report['rows']:assert row['candidate']==row['oracle']==row['fixture']
audit=data('runtime-audit-extended-domain.json');assert audit['package']==web['package']
assert audit['summary']['families']==33 and audit['summary']['callbackMatches']==26 and len(audit['summary']['callbackMismatches'])==7
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
if args.git:
 for files in[built['inputs'],web['inputs']]:
  for path,h in files.items():assert sha(subprocess.check_output(['git','show',m['runtimeCommit']+':'+path]))==h,path
print(f'Verified {len(r)} records; native75 scoped/final11, Swift33 distinct across recorded test repair, C3, web45/1488 new paired rows; audit26/33.')
