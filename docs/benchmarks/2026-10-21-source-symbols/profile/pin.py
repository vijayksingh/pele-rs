"""Data-only pin from an already verified source-symbols runner; no execution."""
from pathlib import Path
import json,hashlib
root=Path(__file__).parent;runner=root.parent/'source-symbols-runner';p=root/'files/scripts/perf'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
m=json.loads((runner/'manifest.json').read_text());pins=json.loads((runner/'files/scripts/perf/source-layout-pins.json').read_text())
assert m.get('candidateSource') and m['candidateSource']==pins['candidateSourceCommit']
f=Path(m['candidateManifestPath']);assert sha(f)==m['candidateManifestSHA256']
package=Path(m['candidatePackagePath']);actual={str(x.relative_to(package)):sha(x) for x in package.rglob('*') if x.is_file()};assert actual==pins['candidatePackageHashes']
inputs=json.loads((p/'inputs.json').read_text());inputs.update(ready=True,sourcePins={'control':pins['controlSourceCommit'],'candidate':pins['candidateSourceCommit']},packages={'control':pins['controlPackageHashes'],'candidate':actual},candidateManifestSHA256=sha(f),runnerManifestSHA256=sha(runner/'manifest.json'))
(p/'inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
m=json.loads((root/'manifest.json').read_text());m['status']='Pinned only; separate profiling GO required';m['files']={str(x.relative_to(root)):sha(x) for x in root.rglob('*') if x.is_file() and x.name!='manifest.json'};(root/'manifest.json').write_text(json.dumps(m,indent=2)+'\n')
print('Pinned inputs; no runtime launched')
