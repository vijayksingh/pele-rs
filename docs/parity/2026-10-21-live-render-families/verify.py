#!/usr/bin/env python3
"""Check archived evidence integrity and, optionally, source commit bindings."""
import argparse, gzip, hashlib, json, subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--git',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((root/'manifest.json').read_bytes())
for name,e in m['files'].items():
 packed=(root/name).read_bytes();raw=gzip.decompress(packed)
 assert (sha(packed),len(packed),sha(raw),len(raw))==(e['sha256'],e['bytes'],e['decodedSHA256'],e['decodedBytes']),name
assert sha((root/'source-binding.json').read_bytes())==m['sourceBindingSHA256']
binding=json.loads((root/'source-binding.json').read_bytes())
def raw(name):return gzip.decompress((root/'raw'/(name+'.gz')).read_bytes()).decode()
def data(name):return json.loads(raw(name))
def git_map(commit,files):
 for path,h in files.items():assert sha(subprocess.check_output(['git','show',commit+':'+path]))==h,path
before=data('live-render-native-before-build.json');after=data('live-render-native-after-build.json')
assert before['inputs']==after['inputs'] and len(after['inputs'])==368 and len(after['sdk'])==13
summary=data('live-render-native-validation-summary.json')
assert summary['nativeInputsVerified']==368 and summary['sdkFilesVerified']==13
assert 'Executed 9 tests, with 0 failures' in raw('live-render-swift-attempt2.log')
assert 'error:' in raw('live-render-swift-attempt1.log')
assert 'FAILED' in raw('pie-live-render-native-attempt1.log')
assert 'test result: ok. 1 passed' in raw('pie-live-render-native-attempt2.log')
assert 'PASS' in raw('live-render-c17.log') and 'PASS' in raw('live-render-c18.log')
web=data('pie-info-live-render-after-build.json');end=data('pie-info-live-render-after-semantic.json')
assert web['inputs']==end['inputs'] and len(web['inputs'])==444
assert web['package']==end['package'] and len(web['package'])==118
assert web['wasmSHA256']==end['wasmSHA256']==binding['wasmSHA256']
assert '59 passed' in raw('pie-info-live-render-web-attempt1.log') and '1 failed' in raw('pie-info-live-render-web-attempt1.log')
assert '3 passed' in raw('gantt-reference-warnings-semantic.log')
audit=data('runtime-audit-live-render.json')
assert audit['package']==web['package'] and audit['summary']['families']==33
assert audit['summary']['callbackMatches']==4 and len(audit['summary']['callbackMismatches'])==29
assert {r['type'] for r in audit['rows'] if r['callbackMatches']}=={'sankey','er','pie','info'}
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
for family,count in [('pie',36),('info',12)]:
 reports=list((root/'reports/strict').glob('*/'+family+'-live-render.json.gz'))
 assert len(reports)==3
 for path in reports:
  report=json.loads(gzip.decompress(path.read_bytes()))
  assert report['before']==report['after'] and len(report['rows'])==count
  for row in report['rows']:assert row['candidate']==row['oracle']
if args.git:git_map(binding['nativeCommit'],after['inputs']);git_map(binding['webCommit'],web['inputs'])
print(f"Verified {len(m['files'])} lossless records; Swift9, C17/C18, browser59/60 plus semantic3/3; audit4/33 callback case matches.")
