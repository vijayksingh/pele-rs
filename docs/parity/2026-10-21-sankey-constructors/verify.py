#!/usr/bin/env python3
"""Data-only constructor evidence checks; no SDK, browser or build required."""
from pathlib import Path
import gzip, hashlib, json, re
ROOT = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()

def raw(name):
    return gzip.decompress((ROOT / 'raw' / (name + '.gz')).read_bytes())

def record(name):
    return json.loads(raw(name))

manifest = json.loads((ROOT / 'raw-manifest.json').read_text())
for row in manifest['files']:
    b = (ROOT / row['stored']).read_bytes()
    assert sha(b) == row['storedSha256'] and len(b) == row['storedBytes']
    b = gzip.decompress(b)
    assert sha(b) == row['originalSha256'] and len(b) == row['originalBytes']
binding = json.loads((ROOT / 'source-binding.json').read_text())
assert binding['sourceCommit'] == '2be54917938f880d09d2b34cafe1d32bb3633e8f'
for lane, count in [('native',468),('web',547)]:
    a = record(f'sankey-constructor-{lane}-before-build.json')
    b = record(f'sankey-constructor-{lane}-after-build.json')
    assert a['inputs'] == b['inputs'] and len(a['inputs']) == count == b['inputsVerified']
    assert binding['lanes'][lane]['count'] == count
assert len(record('sankey-constructor-web-after-build.json')['package']) == 117
assert record('sankey-constructor-test-inputs.json') == binding['finalTestHashes']
initial = raw('sankey-constructor-swift.log').decode()
assert "requires that 'Pele.PublicGraph.Atom' conform to 'Equatable'" in initial
combined = raw('sankey-constructor-swift-green.log').decode()
passed = re.findall(r"Test Case '-\[([^]]+)\]' passed",combined)
failed = re.findall(r"Test Case '-\[([^]]+)\]' failed",combined)
assert len(passed) == 32 and len(failed) == 1
assert len([x for x in passed if 'PublicConstructorTests' not in x]) == 31
assert 'Executed 33 tests, with 6 failures' in combined
final = raw('sankey-constructor-swift-constructor-green.log').decode()
assert len(re.findall(r"Test Case '-\[([^]]+)\]' passed",final)) == 2
assert 'Executed 2 tests, with 0 failures' in final
assert b'C16 constructor lifecycle passed' in raw('sankey-constructor-c.log')
assert b'native detached render lifecycle passed' in raw('sankey-constructor-c-callback-regression.log')
for name, expected in [('web-red','2 failed'),('web-green','5 passed'),('web-final','6 passed'),('regression','73 passed')]:
    assert expected.encode() in raw('sankey-constructor-'+name+'.log')
assert b'1 failed' in raw('sankey-constructor-web-green.log')
assert b'2 skipped' in raw('sankey-constructor-regression.log')
reports = list((ROOT/'raw/web-final-artifacts').glob('*/sankey-constructor.json.gz'))
assert len(reports) == 3
for p in reports:
    d = json.loads(gzip.decompress(p.read_bytes()))
    assert len(d['rows']) == 5
    for row in d['rows']:
        assert row['candidate'] == row['oracle'], (p,row['mode'])
    for metadata in d['metadata'].values():
        assert metadata['arity'] == 0 and metadata['callWithoutNew']['name'] == 'TypeError'
        assert metadata['intrinsic'] == d['metadata']['oracleEsm']['intrinsic']
    assert d['metadata']['candidate']['name'] != d['metadata']['oracleEsm']['name']
for name,digest in json.loads((ROOT/'SHA256SUMS.json').read_text()).items():
    assert sha((ROOT/name).read_bytes()) == digest,name
print('Verified: source maps 468/547, package117; Swift31 regression +2 final constructor features, C16/C15 logs, web6 +73/2 scoped regression. Native2 remains reported-only. Original failures and metadata gaps retained.')
