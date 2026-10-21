#!/usr/bin/env python3
"""Data-only package pinning. No browser, build, transfer or timing."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path
p=argparse.ArgumentParser()
p.add_argument('--manifest',required=True,type=Path)
p.add_argument('--package',required=True,type=Path)
p.add_argument('--source',required=True)
p.add_argument('--wasm',required=True)
p.add_argument('--reviewed-source-manifest',required=True,type=Path)
a=p.parse_args();root=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
assert re.fullmatch('[0-9a-f]{40}',a.source)
assert re.fullmatch('[0-9a-f]{64}',a.wasm)
m=json.loads((root/'manifest.json').read_text())
assert m['probeReady'],'Detached prepare/run/finish/Ack continuity probe must be frozen first'
raw=a.manifest.read_bytes();frozen=json.loads(raw);assert frozen['commit']==a.source
actual={str(f.relative_to(a.package)):sha(f.read_bytes()) for f in sorted(a.package.rglob('*')) if f.is_file()}
assert actual==frozen['package'],'Supplied package differs from frozen map'
assert actual['pkg/pele_bg.wasm']==a.wasm
refs=json.loads((root/'references.json').read_text());prior=refs['priorCandidate']['package'];control=refs['primaryControl']['package']
assert len(actual)==117 and len(prior)==117 and len(control)==113
assert set(actual)==set(prior), 'No package file additions/removals'
assert {k:v for k,v in actual.items() if k.startswith('pkg/')}=={k:v for k,v in prior.items() if k.startswith('pkg/')}, 'JS-only layer requires exact previous WASM/glue/declarations'
allowed={'full.js','graph.js','graph-storage.js'}
changed={name for name in set(actual)|set(prior) if not name.startswith('pkg/') and actual.get(name)!=prior.get(name)}
assert changed==allowed, 'Unexpected static delta from 7417fed: '+str(changed)
reviewraw=a.reviewed_source_manifest.read_bytes();review=json.loads(reviewraw)
assert review['schemaVersion']==1 and review['sourceCommit']==a.source
reviewfiles={e['path']:e['sha256'] for e in review['files']}
assert len(reviewfiles)==len(review['files']), 'Duplicate review paths'
required={'bindings/web/full.js','bindings/web/graph.js','bindings/web/graph-storage.js'}
assert required.issubset(reviewfiles), 'All three owned runtime sources require review'
for name in required:
 blob=subprocess.check_output(['git','show',a.source+':'+name],cwd=root.parents[3])
 assert sha(blob)==reviewfiles[name], 'Committed source differs from reviewed bytes: '+name
for name in allowed:
 assert actual[name]==reviewfiles['bindings/web/'+name], 'Packed file differs from reviewed bytes: '+name
files=root/'files/scripts/perf';path=files/'source-layout-pins.json';pins=json.loads(path.read_text())
for name,digest in pins['files'].items():assert sha((files/name).read_bytes())==digest,'Runner drift: '+name
pins.update(candidateSourceCommit=a.source,candidateWasm=a.wasm,candidatePackageHashes=actual,candidatePackagePath=str(a.package),candidateManifestSHA256=sha(raw),candidatePinStatus='Pinned; each execution phase still needs parent GO',candidateHandwrittenHashes={k:v for k,v in actual.items() if not k.startswith('pkg/') and control.get(k)!=v})
path.write_text(json.dumps(pins,indent=2)+'\n')
m.update(status='candidate pinned; no transfer or timing authorized',candidateSource=a.source,candidateWasm=a.wasm,candidateManifestSHA256=sha(raw),candidateManifestPath=str(a.manifest),candidatePackagePath=str(a.package),reviewedSourceManifestSHA256=sha(reviewraw),reviewedStaticDelta=sorted(allowed))
m['files']={f.name:sha(f.read_bytes()) for f in sorted(files.iterdir()) if f.is_file()}
(root/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
print('Pinned 117 files; exact three static changes reviewed; WASM/glue/declarations unchanged; no execution')
