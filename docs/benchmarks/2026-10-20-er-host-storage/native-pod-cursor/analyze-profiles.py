# Offline source/profile validation; no runtime or network.
from pathlib import Path
import json,hashlib,re,gzip
base=Path(__file__).resolve().parent;runtime=base;summary=json.loads((runtime/'profile-summary.json').read_text());sha=lambda data:hashlib.sha256(data).hexdigest()
assert summary['succeeded']and len(summary['profiles'])==6
assert sha((runtime/'profile-native-pod-sankey-c9b8.mjs').read_bytes())==summary['harnessSHA256']
a,b=summary['runtimeHashes']['control'],summary['runtimeHashes']['candidate'];assert len(a)==len(b)==113 and set(a)==set(b)
assert [name for name in a if a[name]!=b[name]]==['native-storage.js'];assert a['pkg/pele_bg.wasm']==b['pkg/pele_bg.wasm']=='2310934dd43dc52c20723c3e0217377f9819fef8e146d03ef3a33845e06bdcce'
assert b['native-storage.js']=='33224aa2f52b2fc4a4ff9de2f36528f8d422ebe031e3d4831d1634650318cebe'
checks=0
for check in summary['checks']:
 for probe in check['backends']:
  assert probe['actual']==probe['expected']==('source'if probe['runtime']=='candidate'else None)
  assert probe['storage']==probe['expectedStorage']==('native'if probe['runtime']=='candidate'else None);checks+=1
assert checks==24
rows=[]
for entry in summary['profiles']:
 raw=gzip.decompress((runtime/f"{entry['label']}.cpuprofile.gz").read_bytes());assert sha(raw)==entry['profileSHA256'];data=json.loads(raw)
 assert len(data['samples'])==len(data['timeDeltas']);total=sum(data['timeDeltas']);assert total==entry['weightedUs']
 nodes={n['id']:n for n in data['nodes']};parents={}
 for node in data['nodes']:
  for child in node.get('children',[]):assert child not in parents;parents[child]=node['id']
 refresh_self=0;buffer_self=0;refresh_inclusive=0;publication_or_match=0;roles={};languages={}
 for sampled,weight in zip(data['samples'],data['timeDeltas']):
  assert weight>=0;leaf=nodes[sampled];f=leaf['callFrame'];url=f['url'];name=f['functionName']
  language='gc'if name=='(garbage collector)'else'wasm'if url.startswith('wasm://')or re.search(r'\.wasm(?:$|[?#])',url)else'javascript'if re.match(r'https?:',url)else'engine-or-profiler';languages[language]=languages.get(language,0)+weight
  ancestor=leaf;refresh=False;published=False;seen=set()
  while ancestor:
   assert ancestor['id']not in seen;seen.add(ancestor['id']);af=ancestor['callFrame']
   if af['functionName']=='#refresh'and af['url'].endswith('/native-storage.js'):refresh=True
   if af['functionName']in['reconcileGraph','matchesGraph','snapshotGraph']:published=True
   ancestor=nodes.get(parents.get(ancestor['id']))
  if refresh:
   refresh_inclusive+=weight
   if name=='#refresh':refresh_self+=weight
   if name=='get buffer':buffer_self+=weight
  if published:publication_or_match+=weight
 for name,value in languages.items():assert entry['selfCategories'][name]['weightedUs']==value
 assert sum(v['weightedUs']for v in entry['disjointSelfRoles'].values())==total
 rows.append({'stage':entry['stage'],'runtime':entry['runtime'],'rawSHA256':entry['profileSHA256'],'samples':len(data['samples']),'weightedUs':total,'selfLanguagePercent':{name:100*value/total for name,value in languages.items()},'refreshSelfPercent':100*refresh_self/total,'bufferGetterSelfBelowRefreshPercent':100*buffer_self/total,'refreshInclusivePercent':100*refresh_inclusive/total,'deduplicatedObservationPublicationPercent':100*publication_or_match/total,'disjointSelfRoles':entry['disjointSelfRoles'],'groups':entry['architectureGroups'],'unions':entry['deduplicatedAncestorUnions']})
result={'validation':{'filesPerPackage':113,'onlyDifference':'native-storage.js','sameWasm':True,'untimedProbes':24,'rawProfiles':6,'executedHarnessHash':summary['harnessSHA256']},'rows':rows,'run':{'uploadSession':70926,'runSession':51842,'runExit':0,'fetchSession':38592,'fetchExit':0,'harnessFetchSession':90977,'harnessFetchExit':0,'failedLaunches':0},'interpretation':'Disjoint leaf percentages may be added; ancestor groups overlap and are not additive. No cross-window speedup or allocation/call counts inferred.'}
(base/'ATTRIBUTION.json').write_text(json.dumps(result,indent=2)+'\n')
for row in rows:
 if row['runtime']=='candidate':print(row['stage'],'refresh self/getter/inclusive:',row['refreshSelfPercent'],row['bufferGetterSelfBelowRefreshPercent'],row['refreshInclusivePercent'],'observer/pub union:',row['deduplicatedObservationPublicationPercent'])
print('Validated all six raw profiles,24 untimed storage probes,113-file maps,one JS difference and identical WASM.')
