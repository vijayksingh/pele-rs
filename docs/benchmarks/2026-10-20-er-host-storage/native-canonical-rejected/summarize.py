# Data only: no browser, build, network, package mutation or performance run.
from pathlib import Path
import json,hashlib,gzip,re,statistics,math
base=Path(__file__).resolve().parent
sha=lambda data:hashlib.sha256(data).hexdigest()
if (base/'index.json').exists():
 for item in json.loads((base/'index.json').read_text())['files']:
  content=(base/item['path']).read_bytes();assert sha(content)==item['sha256'],item['path']
  if 'uncompressedSha256' in item:assert sha(gzip.decompress(content))==item['uncompressedSha256']
bench=json.loads((base/'comparison.json').read_text());profile=json.loads((base/'profile-summary-original.json').read_text())
assert bench['succeeded'] and profile['succeeded'] and len(bench['rows'])==18 and len(profile['profiles'])==6
assert bench['settings']['trials']==21
assert bench['runtimeHashes']==profile['runtimeHashes'] and bench['runtimeHashes']['control']==bench['runtimeHashes']['candidate'] and len(bench['runtimeHashes']['candidate'])==113
assert sha((base/'compare-web-vm.mjs').read_bytes())==bench['harnessSHA256']
assert sha((base/'profile-native-canonical-sankey.mjs').read_bytes())==profile['harnessSHA256']
assert sha((base/'cases.mjs').read_bytes())==bench['casesSHA256']==profile['casesSHA256']
assert bench['nativeCanonicalBackend']['candidateEnabled'] and not bench['nativeCanonicalBackend']['controlEnabled']
assert not bench['nativeCanonicalBackend']['probeInsideTimer'] and not profile['settings']['probeInsideProfile']
rows=[];seen=set();bench_probes=0;profile_probes=0
for row in bench['rows']:
 key=(row['case'],row['profile'],row['stage']);assert key not in seen;seen.add(key)
 assert all(len(row['samples'][runtime])==21 for runtime in ['control','candidate','js'])
 assert row['counts']['control']==row['counts']['candidate']
 med={name:statistics.median(values) for name,values in row['samples'].items()}
 for name,value in med.items():assert math.isclose(value,row['medianMs'][name],rel_tol=1e-12)
 ratio=med['candidate']/med['control'];js=med['candidate']/med['js']
 assert math.isclose(ratio,row['candidateOverControl'],rel_tol=1e-12) and math.isclose(js,row['candidateOverJS'],rel_tol=1e-12)
 delta=statistics.median(100*(b/a-1) for a,b in zip(row['samples']['control'],row['samples']['candidate']))
 rows.append({'case':key[0],'profile':key[1],'stage':key[2],'pairedPercent':delta,'lowerPairs':sum(b<a for a,b in zip(row['samples']['control'],row['samples']['candidate'])),'overArena':ratio,'overJS':js})
for case in bench['cases']:
 for check in case['backendChecks']:
  assert check['actual']==check['expected']==('source' if check['runtime']=='candidate' else None)
  assert check['actualStorage']==check['expectedStorage']==('native' if check['runtime']=='candidate' else None);bench_probes+=1
for check in profile['checks']:
 for probe in check['backends']:
  assert probe['actual']==probe['expected']==('source' if probe['runtime']=='candidate' else None)
  assert probe['storage']==probe['expectedStorage']==('native' if probe['runtime']=='candidate' else None);profile_probes+=1
assert bench_probes==48 and profile_probes==24
paired={'rows':rows,'stageRanges':{stage:{'pairedPercent':[min(r['pairedPercent'] for r in rows if r['stage']==stage),max(r['pairedPercent'] for r in rows if r['stage']==stage)],'overArena':[min(r['overArena'] for r in rows if r['stage']==stage),max(r['overArena'] for r in rows if r['stage']==stage)],'overJS':[min(r['overJS'] for r in rows if r['stage']==stage),max(r['overJS'] for r in rows if r['stage']==stage)]}for stage in ['parse','render-model','render-source']}}
(base/'paired-summary.json').write_text(json.dumps(paired,indent=2)+'\n')
# Supplemental targeted groups; never add overlapping inclusive percentages.
# Normalize Rust crate hashes so wrapper symbols do not silently miss a regex.
def frame_text(f):return re.sub(r'\[[0-9a-f]+\]','',f['functionName'])+' '+f['url']
patterns={
 'memory-refresh':re.compile(r'#refresh .*\/native-storage\.js'),
 'native-admission':re.compile(r'captured_admission|captured_transport|admit_observed|admit_outbound|canonical_frame_bytes|validate_schema|validate_structure|preflight_encoding'),
 'native-flat-store':re.compile(r'public::flat_store::'),
 'native-wrapper-and-leases':re.compile(r'pele_wasm::native_canonical::'),
 'sankey-parser':re.compile(r'sankey::parser|sankey::Sankey as .*DiagramModule>::parse'),
 'sankey-render-layout':re.compile(r'sankey::(?:render|layout|measure)'),
 'shared-publication':re.compile(r'\breconcileGraph\b'),
 'shared-observation-match':re.compile(r'\b(?:snapshotGraph|matchesGraph)\b'),
 'native-storage-js':re.compile(r'/native-storage\.js'),
}
analysis=[]
for entry in profile['profiles']:
 raw=gzip.decompress((base/(entry['label']+'.cpuprofile.gz')).read_bytes());assert sha(raw)==entry['profileSHA256'];data=json.loads(raw)
 nodes={n['id']:n for n in data['nodes']};parents={}
 for node in nodes.values():
  for child in node.get('children',[]):assert child not in parents;parents[child]=node['id']
 assert len(data['samples'])==len(data['timeDeltas']);total=sum(data['timeDeltas']);assert total==entry['weightedUs']
 groups={name:{'selfUs':0,'inclusiveUs':0}for name in patterns};language={};getter=0;getter_inside_refresh=0;boundary_union=0
 for sampled,weight in zip(data['samples'],data['timeDeltas']):
  assert weight>=0;leaf=nodes[sampled];f=leaf['callFrame'];url=f['url']
  lang='gc' if f['functionName']=='(garbage collector)' else 'wasm' if url.startswith('wasm://') or re.search(r'\.wasm(?:$|[?#])',url) else 'javascript' if re.match(r'https?:',url) else 'engine-or-profiler'
  language[lang]=language.get(lang,0)+weight
  visited=set();seen_groups=set();node=leaf
  while node:
   assert node['id'] not in visited;visited.add(node['id']);text=frame_text(node['callFrame'])
   for name,pattern in patterns.items():
    if pattern.search(text):
     seen_groups.add(name)
     if node is leaf:groups[name]['selfUs']+=weight
   node=nodes.get(parents.get(node['id']))
  for name in seen_groups:groups[name]['inclusiveUs']+=weight
  if seen_groups.intersection({'native-admission','native-flat-store','native-wrapper-and-leases','shared-publication','shared-observation-match','native-storage-js'}):boundary_union+=weight
  if f['functionName']=='get buffer':
   getter+=weight
   if 'memory-refresh' in seen_groups:getter_inside_refresh+=weight
 for name,value in language.items():assert value==entry['selfCategories'][name]['weightedUs']
 groups={name:{**values,'selfPercent':100*values['selfUs']/total,'inclusivePercent':100*values['inclusiveUs']/total}for name,values in groups.items()}
 analysis.append({'stage':entry['stage'],'runtime':entry['runtime'],'samples':len(data['samples']),'weightedUs':total,'selfLanguage':{name:100*value/total for name,value in language.items()},'groups':groups,'getBufferSelfPercent':100*getter/total,'getBufferSelfInsideRefreshPercent':100*getter_inside_refresh/total,'deduplicatedBoundaryUnionPercent':100*boundary_union/total,'refreshPlusBufferDisjointSelfPercent':groups['memory-refresh']['selfPercent']+100*getter_inside_refresh/total})
result={'packageFiles':113,'benchmarkRows':18,'benchmarkUntimedProbes':bench_probes,'profileWindows':6,'profileUntimedProbes':profile_probes,'notes':['Original executed summary retained unchanged. Supplemental group matching normalizes Rust crate hashes, fixing the original native-wrapper regex coverage.','Language/self roles are disjoint; inclusive groups and unions overlap. No predicted gain or invocation counts.','All get-buffer self percentages distinguish those occurring below memory-refresh.'],'profiles':analysis}
(base/'ATTRIBUTION.json').write_text(json.dumps(result,indent=2)+'\n')
print('Verified 18 rows/21 pairs,113-file identity,48 benchmark probes,6 raw weighted profiles,24 profile probes and exact harness/case hashes.')
for stage,values in paired['stageRanges'].items():print(stage,values)
for row in analysis:
 if row['runtime']=='candidate':print(row['stage'],'refresh+buffer self',row['refreshPlusBufferDisjointSelfPercent'],'refresh inclusive',row['groups']['memory-refresh']['inclusivePercent'])
