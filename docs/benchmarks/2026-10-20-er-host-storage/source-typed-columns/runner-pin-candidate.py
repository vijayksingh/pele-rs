#!/usr/bin/env python3
"""Data-only pinning of a parent-frozen native acceptance build; never launches a browser."""
import argparse,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--manifest',required=True,type=Path);p.add_argument('--package',required=True,type=Path);p.add_argument('--source',required=True);p.add_argument('--reviewed-source-manifest',required=True,type=Path);args=p.parse_args()
root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
assert re.fullmatch('[0-9a-f]{40}',args.source)
raw=args.manifest.read_bytes();candidate=json.loads(raw);assert candidate['commit']==args.source
actual={str(f.relative_to(args.package)):sha(f.read_bytes()) for f in sorted(args.package.rglob('*')) if f.is_file()}
assert actual==candidate['package'],'Frozen package manifest differs from supplied package'
references=json.loads((root/'references.json').read_text());prior=references['priorCandidate']['package']
assert len(actual)==116 and len(references['primaryControl']['package'])==113
# Typed03 explicitly changes decoder and generated initializer; require reviewed exact bytes.
reviewed_raw=args.reviewed_source_manifest.read_bytes();reviewed=json.loads(reviewed_raw)
reviewed_files={entry['path']:entry['candidateSHA256'] for entry in reviewed['files']}
allowed={'layout-storage.js','source-initializer.js'}
changed={k for k in set(actual)|set(prior) if not k.startswith('pkg/') and actual.get(k)!=prior.get(k)}
assert changed==allowed,'Unexpected handwritten/static delta from e518: '+str(changed)
for name in allowed:assert actual[name]==reviewed_files['bindings/web/'+name],'Packed file differs from reviewed candidate: '+name
assert 'scripts/generate-source-initializer.mjs' in reviewed_files,'Reviewed generator identity required'
d=root/'files/scripts/perf';path=d/'source-layout-pins.json';pins=json.loads(path.read_text())
pins['candidateSourceCommit']=args.source;pins['candidatePinStatus']='parent frozen build supplied; execution still requires separate GO'
control=references['primaryControl']['package']
pins['candidateHandwrittenHashes']={k:v for k,v in actual.items() if not k.startswith('pkg/') and control.get(k)!=v}
for name,digest in pins['files'].items():assert sha((d/name).read_bytes())==digest,'Harness/probe drift: '+name
path.write_text(json.dumps(pins,indent=2)+'\n')
m=json.loads((root/'manifest.json').read_text());m.update(status='candidate pinned; no transfer or timing authorized',candidateSource=args.source,candidateWasm=actual['pkg/pele_bg.wasm'],candidateManifestSHA256=sha(raw),candidateManifestPath=str(args.manifest),candidatePackagePath=str(args.package),reviewedSourceManifestSHA256=sha(reviewed_raw),reviewedGeneratorSHA256=reviewed_files['scripts/generate-source-initializer.mjs'])
m['files']={f.name:sha(f.read_bytes()) for f in sorted(d.iterdir()) if f.is_file()}
(root/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print('Pinned package; no browser or VM work performed')
