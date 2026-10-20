#!/usr/bin/env python3
from pathlib import Path
import json,hashlib,argparse
p=argparse.ArgumentParser();p.add_argument('phase',choices=['warm','first-use']);a=p.parse_args()
root=Path(__file__).resolve().parent;r=root/'runtime';folder=r/('fetch-'+a.phase);e=folder/'evidence';sha=lambda b:hashlib.sha256(b).hexdigest()
q=json.loads((e/(a.phase+'-provenance.json')).read_text());raw=e/(a.phase+'.json');report=json.loads(raw.read_text())
expected='1429f77380082b13424f8942824a86437dccf988c9200d228fc487b2437f8272' if a.phase=='warm' else '5469078edefad399f109ad29f4c3f25a56ce3834bfdbe4a635c4f3b7ea463bb0'
assert q['succeeded'] and q['verifiedRows']==24 and q['harnessSHA256']==expected and report['succeeded'] and len(report['rows'])==24
assert report['harnessSHA256']==expected and q['runtimeHashes']==report['runtimeHashes']
assert len(q['runtimeHashes']['candidate'])==117 and len(q['runtimeHashes']['control'])==117
for key,suffix,count in [('continuity','layout-continuity',8),('activation','activation',16),('ingressActivation','ingress-activation',12)]:
 b=(e/(a.phase+'-'+suffix+'.json')).read_bytes();d=json.loads(b)
 assert d['succeeded'] and d['browserClosedBeforeTiming'] and len(d['checks'])==count and sha(b)==q[key]['reportSHA256']
assert json.loads((e/(a.phase+'-layout-continuity.json')).read_text())['readonlyJobsVerified']
refs=json.loads((root/'references.json').read_text());assert q['probeOnlyControl']['hashes']==refs['probeLegacyControl']['package'];assert q['reusedTimedControlProof']==refs['reusedControlProof']
for path,digest in refs['reusedControlProof'].items():assert sha(Path(path).read_bytes())==digest
assert sha(raw.read_bytes())==q['reportSHA256']
assert report['settings']['trials']==21 and report['settings']['warmupMs']==1000
for row in report['rows']:
 for samples in row['samples'].values():assert len(samples)==21
 assert row['counts']['candidate']==row['counts']['control']
for f in (root/'files').rglob('*'):
 if f.is_file():assert f.read_bytes()==(folder/f.relative_to(root)).read_bytes()
record={'phase':a.phase,'verifiedRows':24,'probeChecks':[8,16,12],'allPackageAndHarnessChecks':True,'source':q['source'],'rawSHA256':q['reportSHA256'],'artifacts':{str(f.relative_to(r)):sha(f.read_bytes()) for f in sorted(e.glob(a.phase+'*.json'))}}
(r/(a.phase+'-verification.json')).write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:v for k,v in record.items() if k!='artifacts'},indent=2))
