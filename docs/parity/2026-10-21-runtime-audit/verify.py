#!/usr/bin/env python3
"""Portable evidence verification; no packages, browser or native runtime needed."""
from pathlib import Path
import gzip, hashlib, json
root = Path(__file__).resolve().parent
sha = lambda b: hashlib.sha256(b).hexdigest()
manifest = json.loads((root / 'manifest.json').read_text())
for name, expected in manifest['files'].items():
    assert sha((root / name).read_bytes()) == expected, name
for name, expected in manifest['uncompressed'].items():
    assert sha(gzip.decompress((root / name).read_bytes())) == expected, name
web = json.loads(gzip.decompress((root / 'web-entry-report.json.gz').read_bytes()))
oracle = json.loads(gzip.decompress((root / 'oracle-report.json.gz').read_bytes()))
assert web['source'] == manifest['runtime']
assert sha((root / 'audit-original.mjs').read_bytes()) == web['scriptSHA256']
assert oracle['oracleSHA256'] == manifest['singleBundleOracleSHA256']
rows = web['rows']
assert len(rows) == len({r['type'] for r in rows}) == 33
assert all(r[lane]['callback']['calls'] == 1 and r[lane]['callback']['originalThrow']
           for r in rows for lane in ['native', 'oracle'])
assert all(r['oracle']['callback']['titleRetained'] for r in rows)
lost = [r['type'] for r in rows if not r['native']['callback']['titleRetained']]
kept = [r['type'] for r in rows if r['native']['callback']['titleRetained']]
assert len(lost) == 31 and set(kept) == {'sankey', 'er'}
stubs = [r['type'] for r in rows if r['oracle']['construction']['ok'] and not r['native']['construction']['ok']]
assert len(stubs) == 22
assert all(r['native']['construction']['error']['message'] == 'Public model construction is unavailable' for r in rows if r['type'] in stubs)
constructors = oracle['constructors']
assert len(constructors) == 33
assert set(stubs) == {r['family'] for r in constructors if not r.get('plainObject')}
assert all(r['ok'] and r['samePrototype'] for r in constructors if not r.get('plainObject'))
cases = {r['name']: r for r in oracle['cases']}
assert cases['info-surrogate']['ok'] and 0xd800 in cases['info-surrogate']['svgCodeUnits']
for name in ['radar-hole', 'radar-empty-circle']:
    assert cases[name]['error'] == {'name': 'TypeError', 'message': "Cannot read properties of undefined (reading 'x')"}
assert cases['radar-empty-polygon']['ok']
assert len(web['package']) == 117 and len(web['oracle']) == 124
print(json.dumps({'verified': True, 'families': 33, 'callbackEditsLost': len(lost),
                  'constructorsStubbed': len(stubs), 'originalThrowsPreserved': 33,
                  'nativePackageFiles': len(web['package']), 'oracleEntryFiles': len(web['oracle'])}, indent=2))
