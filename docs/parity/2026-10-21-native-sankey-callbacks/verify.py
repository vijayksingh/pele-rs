#!/usr/bin/env python3
"""Verify this data-only bundle; requires only Python's standard library."""
from pathlib import Path
import gzip, hashlib, json, re
ROOT = Path(__file__).resolve().parent

def sha(data):
    return hashlib.sha256(data).hexdigest()

def read(name):
    return gzip.decompress((ROOT / 'raw' / (name + '.gz')).read_bytes())

def record(name):
    return json.loads(read(name))

manifest = json.loads((ROOT / 'raw-manifest.json').read_text())
for entry in manifest['files']:
    stored = (ROOT / entry['stored']).read_bytes()
    assert len(stored) == entry['storedBytes'] and sha(stored) == entry['storedSha256']
    original = gzip.decompress(stored)
    assert len(original) == entry['originalBytes'] and sha(original) == entry['originalSha256']
before = record('native-sankey-callback-before-build.json')
after = record('native-sankey-callback-after-build.json')
assert before['sources'] == after['sources'] and len(after['sources']) == 515
binding = json.loads((ROOT / 'source-binding.json').read_text())
assert binding['sdkHashes'] == after['sdk']
assert set(binding['unchangedBuildPaths']) | {x['path'] for x in binding['postBuildChanges']} == set(after['sources'])
assert len(binding['unchangedBuildPaths']) == 514
assert [x['path'] for x in binding['postBuildChanges']] == ['bindings/swift/Tests/PeleTests/NativeSankeyCallbackTests.swift']
assert binding['postBuildChanges'][0]['buildSha256'] == after['sources'][binding['postBuildChanges'][0]['path']]
api = record('native-sankey-callback-swift-api.json')
assert api['mode'] == 'swift-api-correctness-only'
assert len(api['rows']) == 168 and len(api['cases']) == 21
assert len({(r['case'], r['stage']) for r in api['rows']}) == 168
assert all(r['correctness']['count'] == 1 for r in api['rows'])
assert api['buildManifest']['binarySha256'] == binding['consumerBinarySha256']
assert api['buildManifest']['nativeHostArchiveSha256'] == binding['consumerNativeArchiveSha256']
passed = []
for name, expected in [('green', 22), ('recovery', 9)]:
    text = read('native-sankey-callback-' + name + '.log').decode()
    tests = re.findall(r"Test Case '-\[([^]]+)\]' passed", text)
    assert len(tests) == expected
    assert f'Executed {expected} tests, with 0 failures' in text
    passed.extend(tests)
assert len(set(passed)) == 31
new = [name for name in passed if name.startswith('PeleTests.NativeSankeyCallbackTests ')]
assert len(new) == 5
red = read('native-sankey-callback-red.log').decode()
assert len(re.findall(r"Test Case '-\[PeleTests.NativeSankeyCallbackTests [^]]+\]' failed", red)) == 3
legacy = record('native-sankey-callback-legacy-c.json')
assert len(legacy) == 5 and all(x['compileExit'] == x['runExit'] == 0 for x in legacy)
assert b'native detached render lifecycle passed' in read('native-sankey-callback-c.log')
web_before = record('port-completion-identity.json')
web_after = record('port-completion-web-identity-after.json')
assert web_before['package'] == web_after['package'] and len(web_after['package']) == 117
assert web_after['completion']['passed'] == 469 and web_after['completion']['skipped'] == 59
assert web_after['completion']['nativeFFIAndSwiftEditedDuringRun'] is True
web_log = read('port-completion-web.log')
assert b'469 passed' in web_log and b'59 skipped' in web_log
index = json.loads((ROOT / 'SHA256SUMS.json').read_text())
for name, digest in index.items():
    assert sha((ROOT / name).read_bytes()) == digest, name
print('Verified: 31 Swift tests (5 callback features), 168 API comparisons/21 cases, 6 C clients; 515 build inputs, explicit later test delta; fixed web package 469 passed/59 skipped. No performance or full-parity claim.')
