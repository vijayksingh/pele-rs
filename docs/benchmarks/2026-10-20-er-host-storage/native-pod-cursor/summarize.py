# Data-only verification of both sequential latency windows. No browser or build.
from pathlib import Path
import json,statistics,hashlib,math,gzip
base=Path(__file__).resolve().parent;sha=lambda data:hashlib.sha256(data).hexdigest()
if (base/'index.json').exists():
 for item in json.loads((base/'index.json').read_text())['files']:
  data=(base/item['path']).read_bytes();assert sha(data)==item['sha256'],item['path']
  if 'uncompressedSha256'in item:assert sha(gzip.decompress(data))==item['uncompressedSha256']
for log in json.loads((base/'PROVENANCE.json').read_text())['focusedLogs']:
 raw=gzip.decompress((base/log['path']).read_bytes());assert len(raw)==log['uncompressedBytes']and sha(raw)==log['uncompressedSha256']
profile=json.loads((base/'profile-summary.json').read_text());results={}
for label in ['native','arena']:
 raw=json.loads((base/f'versus-{label}.json').read_text());assert raw['succeeded']and len(raw['rows'])==18 and raw['settings']['trials']==21
 assert raw['runtimeHashes']==profile['runtimeHashes'];assert len(raw['runtimeHashes']['candidate'])==113
 assert raw['nativeCanonicalBackend']['candidateEnabled']and raw['nativeCanonicalBackend']['controlEnabled']==(label=='native')
 assert not raw['nativeCanonicalBackend']['probeInsideTimer'];assert sha((base/'compare-web-vm.mjs').read_bytes())==raw['harnessSHA256'];assert sha((base/'cases.mjs').read_bytes())==raw['casesSHA256']==profile['casesSHA256']
 seen=set();rows=[]
 for row in raw['rows']:
  key=(row['case'],row['profile'],row['stage']);assert key not in seen;seen.add(key)
  assert all(len(values)==21 for values in row['samples'].values());assert row['counts']['control']==row['counts']['candidate']
  med={name:statistics.median(values)for name,values in row['samples'].items()}
  for name,value in med.items():assert math.isclose(value,row['medianMs'][name],rel_tol=1e-12)
  ratio=med['candidate']/med['control'];js=med['candidate']/med['js'];assert math.isclose(ratio,row['candidateOverControl'],rel_tol=1e-12)and math.isclose(js,row['candidateOverJS'],rel_tol=1e-12)
  pairs=[100*(b/a-1)for a,b in zip(row['samples']['control'],row['samples']['candidate'])]
  rows.append({'case':key[0],'profile':key[1],'stage':key[2],'pairedPercent':statistics.median(pairs),'lowerPairs':sum(b<a for a,b in zip(row['samples']['control'],row['samples']['candidate'])),'overControl':ratio,'overJS':js})
 probes=0
 for case in raw['cases']:
  for check in case['backendChecks']:
   enabled=check['runtime']=='candidate'or label=='native'
   assert check['expected']==check['actual']==('source'if enabled else None)
   assert check['expectedStorage']==check['actualStorage']==('native'if enabled else None);probes+=1
 assert probes==48
 ranges={stage:{'pairedPercent':[min(r['pairedPercent']for r in rows if r['stage']==stage),max(r['pairedPercent']for r in rows if r['stage']==stage)],'overControl':[min(r['overControl']for r in rows if r['stage']==stage),max(r['overControl']for r in rows if r['stage']==stage)],'overJS':[min(r['overJS']for r in rows if r['stage']==stage),max(r['overJS']for r in rows if r['stage']==stage)]}for stage in ['parse','render-model','render-source']}
 results[label]={'rows':rows,'ranges':ranges,'untimedProbes':probes}
 for stage,values in ranges.items():print(label,stage,values)
(base/'paired-summary.json').write_text(json.dumps(results,indent=2)+'\n')
print('Verified both18-row windows,all21-pair samples,96 untimed backend/storage probes,matched batch counts and exact package/harness/case hashes. Do not combine windows into a cross-window speedup.')
