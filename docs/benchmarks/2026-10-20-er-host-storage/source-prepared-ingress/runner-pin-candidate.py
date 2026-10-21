#!/usr/bin/env python3
"""Data-only pinning of a parent-frozen native acceptance build; never launches a browser."""
import argparse,hashlib,json,re
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--manifest',required=True,type=Path);p.add_argument('--package',required=True,type=Path);p.add_argument('--source',required=True);args=p.parse_args()
root=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
assert re.fullmatch('[0-9a-f]{40}',args.source)
raw=args.manifest.read_bytes();candidate=json.loads(raw);assert candidate['commit']==args.source
actual={str(f.relative_to(args.package)):sha(f.read_bytes()) for f in sorted(args.package.rglob('*')) if f.is_file()}
assert actual==candidate['package'],'Frozen package manifest differs from supplied package'
references=json.loads((root/'references.json').read_text());prior=references['priorCandidate']['package']
assert len(actual)==116 and len(references['primaryControl']['package'])==113
# This assigned layer changes native files only. Unexpected JS/static drift requires parent review.
assert {k:v for k,v in actual.items() if not k.startswith('pkg/')}=={k:v for k,v in prior.items() if not k.startswith('pkg/')},'Unexpected handwritten/static delta from ef9'
d=root/'files/scripts/perf';path=d/'source-layout-pins.json';pins=json.loads(path.read_text())
pins['candidateSourceCommit']=args.source;pins['candidatePinStatus']='parent frozen build supplied; execution still requires separate GO'
control=references['primaryControl']['package']
pins['candidateHandwrittenHashes']={k:v for k,v in actual.items() if not k.startswith('pkg/') and control.get(k)!=v}
for name,digest in pins['files'].items():assert sha((d/name).read_bytes())==digest,'Harness/probe drift: '+name
path.write_text(json.dumps(pins,indent=2)+'\n')
m=json.loads((root/'manifest.json').read_text());m.update(status='candidate pinned; no transfer or timing authorized',candidateSource=args.source,candidateWasm=actual['pkg/pele_bg.wasm'],candidateManifestSHA256=sha(raw),candidateManifestPath=str(args.manifest),candidatePackagePath=str(args.package))
m['files']={f.name:sha(f.read_bytes()) for f in sorted(d.iterdir()) if f.is_file()}
(root/'manifest.json').write_text(json.dumps(m,indent=2)+'\n');print('Pinned package; no browser or VM work performed')
