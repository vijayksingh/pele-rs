#!/usr/bin/env python3
"""Verify archived public feature evidence; does not execute the library."""
import argparse,gzip,hashlib,json,re,subprocess
from pathlib import Path
parser=argparse.ArgumentParser();parser.add_argument('--git',action='store_true');args=parser.parse_args()
root=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
m=json.loads((root/'manifest.json').read_bytes())
for name,e in m['files'].items():
 packed=(root/name).read_bytes();raw=gzip.decompress(packed)
 assert (sha(packed),len(packed))==(e['sha256'],e['bytes']),name
 assert (sha(raw),len(raw))==(e['decodedSHA256'],e['decodedBytes']),name
assert sha((root/'source-binding.json').read_bytes())==m['sourceBindingSHA256']
binding=json.loads((root/'source-binding.json').read_bytes())
def raw(name):return gzip.decompress((root/'raw'/(name+'.gz')).read_bytes()).decode()
def data(name):return json.loads(raw(name))
def git_map(commit,files):
 for path,h in files.items():assert sha(subprocess.check_output(['git','show',commit+':'+path]))==h,path
before=data('ordered-constructors-native-before-build.json');after=data('ordered-constructors-native-after-build.json')
assert before['inputs']==after['inputs'] and len(after['inputs'])==366
assert len(after['sdk'])==13
if args.git:git_map(binding['nativeCommit'],after['inputs'])
s=raw('ordered-constructors-native-final.log')
assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed',s)))==24 and 'FAILED' not in s and 'warning:' not in s
assert 'Executed 8 tests, with 0 failures' in raw('ordered-constructors-swift-attempt1.log')
assert 'Executed 1 test, with 0 failures' in raw('ordered-constructors-swift-33entries.log')
assert 'Executed 1 test, with 0 failures' in raw('ordered-constructors-swift-block-attempt2.log')
assert 'PASS' in raw('ordered-constructors-c17.log') and 'PASS' in raw('ordered-constructors-c18-attempt2.log')
assert 'Assertion failed' in raw('ordered-constructors-c18.log')
assert '"pie"' in raw('ordered-constructors-c18-attempt1-source.c')
assert 'capture(registry, "sankey", 1)' in gzip.decompress((root/'sources/tests/ffi_constructor_body_v18.c.gz').read_bytes()).decode()
runtime=data('ordered-constructors-runtime-after.json')
assert runtime['nativeInputsVerified']==366 and runtime['nativeInputMapUnchanged']
assert runtime['sdkFilesVerified']==13 and runtime['sdkUnchanged']
block=data('ordered-constructors-swift-block-proof.json');assert block['unchanged'] and block['nativeInputsVerified']==366 and block['sdkFilesVerified']==13
web_before=data('constructor-body-before-build.json');built=data('constructor-body-after-build.json');final=data('constructor-body-final-package.json');committed=data('constructor-body-committed.json');tested=data('constructor-body-after-tests.json')
assert web_before['inputs']==built['inputs'] and len(built['inputs'])==442 and built['sourceUnchanged']
assert {p for p in final['inputs'] if final['inputs'][p]!=built['inputs'][p]}=={'bindings/web/full.js','bindings/web/graph.js'}
assert final['inputs']==committed['gitInputs'] and len(committed['gitInputs'])==442
assert final['package']==committed['package'] and len(final['package'])==118
assert built['wasmSHA256']==final['wasmSHA256']==committed['wasmSHA256']==binding['wasmSHA256']
assert tested['sourceUnchanged'] and tested['packageUnchanged'] and tested['inputCount']==442 and tested['packageCount']==118
if args.git:git_map(binding['webCommit'],committed['gitInputs'])
assert '51 passed' in raw('constructor-body-web-final.log')
assert '1 failed' in raw('constructor-body-web-attempt1.log') and '2 passed' in raw('constructor-body-web-attempt2.log')
assert '37 passed' in raw('flow-constructors-web-final.log') and '8 failed' in raw('flow-constructors-web-final.log')
ordered=list((root/'reports/web').glob('*/ordered-bodies.json.gz'));members=list((root/'reports/web').glob('*/class-member-bodies.json.gz'))
assert len(ordered)==len(members)==3
for p in ordered:
 rows=json.loads(gzip.decompress(p.read_bytes()));assert len(rows)==3
 for row in rows:
  assert row['candidate']==row['oracle'],row['family']
  for key in ['initial','populated','constructorCases']:assert row['candidate'][key]==row['fixture'][key],(row['family'],key)
for p in members:
 row=json.loads(gzip.decompress(p.read_bytes()));assert row['candidate']==row['oracle']
 assert row['candidate']['cases']==row['fixture']['cases'] and row['candidate']['arity']==2 and len(row['candidate']['virtual'])==6
audit=json.loads(gzip.decompress((root/'runtime-audit.json.gz').read_bytes()))
assert audit['package']==final['package'] and audit['summary']['families']==33
assert audit['summary']['classRoots']==22 and not audit['summary']['nativeClassConstructionFailures']
assert audit['summary']['callbackMatches']==2 and len(audit['summary']['callbackMismatches'])==31
assert {row['type'] for row in audit['rows'] if row['callbackMatches']}=={'sankey','er'}
print(f"Verified {len(m['files'])} lossless records; native24, Swift8+1, 99 entry cases, C17/C18 and web51 checkpoint. Known failures preserved.")
