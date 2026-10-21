#!/usr/bin/env python3
"""Verify recorded integration evidence, its immutable inputs and remaining failures."""
import argparse, gzip, hashlib, json, re, subprocess
from pathlib import Path
parser = argparse.ArgumentParser()
parser.add_argument('--git', action='store_true')
args = parser.parse_args()
root = Path(__file__).resolve().parent
sha = lambda value: hashlib.sha256(value).hexdigest()
manifest = json.loads((root / 'manifest.json').read_bytes())
packed = (root / 'evidence.json.gz').read_bytes()
raw = gzip.decompress(packed)
assert (sha(packed), len(packed), sha(raw), len(raw)) == (manifest['sha256'], manifest['bytes'], manifest['decodedSHA256'], manifest['decodedBytes'])
records = json.loads(raw)
assert len(records) == manifest['records']
data = lambda name: json.loads(records[name])
initial = data('remaining-family-native-after-build.json')
native = data('remaining-family-native-after-build-repair.json')
assert data('remaining-family-native-before-build.json')['inputs'] == initial['inputs']
assert data('remaining-family-runtime-after-attempt2.json')['inputs'] == initial['inputs']
assert data('remaining-family-native-before-build-repair.json')['inputs'] == native['inputs']
for name in ['remaining-family-runtime-after-attempt3.json', 'remaining-family-runtime-after-observer-repair.json']:
    tested = data(name)
    assert tested['inputs'] == native['inputs'] and tested['sdk'] == native['sdk']
assert len(native['inputs']) == 420 and len(native['sdk']) == 13
assert all(c['exitCode'] == 0 for c in data('remaining-family-validation-attempt3.json')['commands'][1:])
assert 'Executed 44 tests, with 4 failures' in records['remaining-family-swift-attempt3.log']
assert 'testClassOwnedResourceAndMemberCallbacksMatch80PinnedOracleRows]'+"' passed" in records['remaining-family-swift-attempt3.log']
assert 'Executed 1 test, with 0 failures' in records['remaining-family-swift-revision-observer-repair.log']
assert 'CompatibilityError' in subprocess.check_output(['git', 'show', manifest['testObserverRepairCommit']+':bindings/swift/Tests/PeleTests/NativeClassCallbackTests.swift'], text=True)
for version in [15, 17, 18, 19]:
    assert re.search(r'PASS|passed', records[f'remaining-family-final-c{version}.log'])
names = set()
for name in ['remaining-owned-native-regressions-attempt1.log', 'remaining-family-native-final.log']:
    names.update(re.findall(r'^test (\S+) \.\.\. ok$', records[name], re.M))
assert len(names) == 55
web = data('remaining-family-after-build.json')
assert web['inputs'] == data('remaining-family-before-build.json')['inputs'] == data('remaining-family-after-tests.json')['inputs']
assert web['package'] == data('remaining-family-after-tests.json')['package']
repaired = data('remaining-family-after-package-repair.json')
end = data('remaining-family-after-tests-repair.json')
assert repaired['inputs'] == end['inputs'] and repaired['package'] == end['package']
assert [path for path in web['inputs'] if web['inputs'][path] != repaired['inputs'][path]] == ['bindings/web/graph-storage.js']
wasm = 'dist/package-web/package/pkg/pele_bg.wasm'
assert web['package'][wasm] == repaired['package'][wasm] == manifest['wasmSHA256']
assert '192 passed' in records['remaining-family-web-attempt1.log'] and '9 failed' in records['remaining-family-web-attempt1.log']
assert '24 passed' in records['remaining-family-web-attempt2.log']
exact = total = 0
for family, count, attempt in [('agentflow',64,1),('class',80,1),('gantt',72,1),('sequence',76,2)]:
    reports = [json.loads(v) for key,v in records.items() if key.startswith(f'reports/attempt{attempt}/') and key.endswith('/'+family+'-live-render.json')]
    assert len(reports) == 3
    for report in reports:
        assert report['before'] == report['after'] and len(report['rows']) == count
        for row in report['rows']:
            assert row['candidate'] == row['oracle'] == row['fixture']
            exact += 1; total += 1
for name, count in [('gantt-calendar-start',17),('gantt-owned-scheduler',20)]:
    reports = [json.loads(v) for key,v in records.items() if key.startswith('reports/attempt1/') and key.endswith('/'+name+'.json')]
    assert len(reports) == 3
    for report in reports:
        assert len(report['oracle']) == len(report['candidate']) == count
        for actual, expected in zip(report['candidate'], report['oracle']):
            total += 1
            exact += actual == expected
assert (total, exact) == (987,985), (total,exact)
audit = data('runtime-audit-remaining-family-repair.json')
assert audit['summary'] == {'families':33,'callbackMatches':33,'callbackMismatches':[],'classRoots':22,'nativeClassConstructionFailures':[]}
assert audit['package'] == {path.removeprefix('dist/package-web/package/'):value for path,value in repaired['package'].items()}
if args.git:
    for commit, inputs in [(manifest['nativeRuntimeCommit'], native['inputs']), (manifest['webRuntimeCommit'], repaired['inputs'])]:
        for path, expected in inputs.items():
            assert sha(subprocess.check_output(['git','show',commit+':'+path])) == expected, path
print(f'Verified {len(records)} records: native55/final8, Swift44 distinct, C4, browser201/207 distinct; audit33/33. Six exact error-message failures remain.')
