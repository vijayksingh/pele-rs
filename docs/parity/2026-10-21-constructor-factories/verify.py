#!/usr/bin/env python3
"""Offline constructor evidence integrity and result-count checks; no execution."""
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--git',action='store_true');args=p.parse_args()
root=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest=json.loads((root/'manifest.json').read_bytes())
for name,entry in manifest['files'].items():
 packed=(root/name).read_bytes();raw=gzip.decompress(packed)
 assert (sha(packed),len(packed))==(entry['sha256'],entry['bytes']),name
 assert (sha(raw),len(raw))==(entry['decodedSHA256'],entry['decodedBytes']),name
def raw(name):return gzip.decompress((root/'raw'/name).read_bytes()).decode()
def data(name):return json.loads(raw(name))
assert sha((root/'source-binding.json').read_bytes())==manifest['sourceBindingSHA256']
binding=json.loads((root/'source-binding.json').read_bytes())
for lane,count in [('native',355),('web',434)]:
 stem='direct-constructors-'+lane
 a=data(stem+'-before-build.json.gz')['inputs'];b=data(stem+'-after-build.json.gz')['inputs']
 assert a==b and len(a)==count
 if args.git:
  for path,h in a.items():assert sha(subprocess.check_output(['git','show',binding['eight'][lane]['gitCommit']+':'+path]))==h,path
assert len(data('direct-constructors-native-after-build.json.gz')['sdk'])==13
assert len(data('direct-constructors-web-after-build.json.gz')['package'])==117
for phase,count in [('direct',4),('next',9)]:
 s=raw(phase+'-constructors-native.log.gz')
 assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',s)))==count
 assert not re.search(r'test result: FAILED',s)
 s=raw(phase+'-constructors-swift-green.log.gz');assert 'Executed 10 tests, with 0 failures' in s
 assert 'C16 constructor lifecycle passed' in raw(phase+'-constructors-c.log.gz')
assert '6 passed' in raw('direct-constructors-web-accepted.log.gz')
assert '62 passed' in raw('direct-constructors-web-final.log.gz') and '1 failed' in raw('direct-constructors-web-final.log.gz')
assert '5 passed' in raw('direct-constructors-web-green.log.gz') and '1 failed' in raw('direct-constructors-web-green.log.gz')
assert 'Failed to fetch dynamically imported module' in raw('next-constructors-web-red.log.gz')
assert 'Failed to resolve module specifier' in raw('next-constructors-web-functional-red.log.gz')
assert 'Public model construction is unavailable' in raw('next-constructors-web-control.log.gz')
reports=list((root/'reports/eight').glob('*/*.json.gz'));assert len(reports)==6
for path in reports:
 d=json.loads(gzip.decompress(path.read_bytes()))
 if isinstance(d,list):
  assert len(d)==7
  for row in d:assert row['independent'] and row['actual']==row['wanted'],row['family']
 else:
  assert len(d['rows'])==21 and len(d['metadata'])==28
  for row in d['rows']:assert row['candidate']==row['oracle'],row['family']
  for m in d['metadata']:assert m['arity']==0 and set(m['ownKeys'])=={'length','name','prototype'} and m['backlinkIdentity']
assert '6 passed' in raw('next-constructors-web-green.log.gz')
assert '36 passed' in raw('next-constructors-regression.log.gz')
assert len(data('next-constructors-native-after-build.json.gz')['sdk'])==13
assert len(data('next-constructors-final-test-inputs.json.gz'))==237
for lane,count in [('native',355),('web',434)]:
 a=data('next-constructors-'+lane+'-before-build.json.gz')['inputs'];b=data('next-constructors-'+lane+'-after-build.json.gz')['inputs']
 assert a==b and len(a)==count
 if args.git:
  for path,h in a.items():assert sha(subprocess.check_output(['git','show',binding['fifteen'][lane]['gitCommit']+':'+path]))==h,path
if args.git:
 for phase,commit in [('direct',binding['eight']['native']['gitCommit']),('next',binding['fifteen']['native']['gitCommit'])]:
  for path,h in data(phase+'-constructors-final-test-inputs.json.gz').items():assert sha(subprocess.check_output(['git','show',commit+':'+path]))==h,path
assert data('next-constructors-web-before-build.json.gz')['inputs']==data('next-constructors-web-after-build.json.gz')['inputs']
assert len(data('next-constructors-web-after-build.json.gz')['inputs'])==434
assert len(data('next-constructors-web-after-build.json.gz')['package'])==117
reports=list((root/'reports/fifteen').glob('*/*.json.gz'));assert len(reports)==6
for path in reports:
 d=json.loads(gzip.decompress(path.read_bytes()))
 if isinstance(d,list):
  assert len(d)==7
  for row in d:
   assert row['actual']==row['wanted'] and row['actualActions']==row['expectedActions'] and row['independentObjects'],row['family']
   assert row['yBefore']==row['yAfter'] and row['freshBefore']==row['freshAfter'] and row['later']==row['expectedLater']
 else:
  assert len(d['rows'])==21 and len(d['metadata'])==28
  assert d['sharedIdentity']['candidate']==d['sharedIdentity']['oracle']=={'present':True,'parsedSame':True,'constructedSame':True,'constructedCount':4}
  for row in d['rows']:assert row['candidate']==row['oracle'],row['family']
print(f"Verified {len(manifest['files'])} lossless records; eight and fifteen factory checkpoints bound; 4+9 native, 10+10 Swift, 6+6 feature browser, 36 scoped regression passes.")
