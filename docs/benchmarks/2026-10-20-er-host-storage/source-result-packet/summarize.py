#!/usr/bin/env python3
"""Portable data-only verification and summary. No imports of repository code,
network access, source inspection, compilation, browser or workload execution.
"""
from pathlib import Path
import collections,gzip,hashlib,json,math,statistics
B=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def raw(name):
 p=B/name;b=p.read_bytes();return gzip.decompress(b) if p.suffix=='.gz' else b
def load(name):return json.loads(raw(name))
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
def write(name,value):
 (B/name).write_text(json.dumps(value,indent=2,ensure_ascii=True)+'\n',encoding='ascii')
w=load('comparison.json.gz');f=load('freeze/package-freeze.json.gz');before=load('freeze/source-before.json.gz');status=load('status.json')
assert w['succeeded'] and w['diagnostic'] and len(w['rows'])==126 and len(w['cases'])==21
assert w['settings']['trials']==9 and w['settings']['warmupMs']==500
assert w['settings']['crossOriginIsolated'] and 0<w['settings']['observedClockResolutionMs']<=.01
assert f['sourceUnchanged'] and f['nativeSDKUnchanged'] and f['optimization']=='s'
assert sha(raw('freeze/source-before.json.gz'))==f['sourceBeforeSHA256'];assert len(before['files'])==657
assert sha(raw('logs/wasm-build.log.gz'))==f['buildLogSHA256'];assert sha(raw('harness/compare-web-vm.mjs'))==w['harnessSHA256'];assert sha(raw('harness/cases.mjs'))==w['casesSHA256']
assert w['oracleSHA256']=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d'
assert w['runtimeHashes']=={k:f[k] for k in ['control','candidate']}==load('freeze/independent-vm-rehash.json.gz')['hashes']==load('freeze/local-package-verification.json.gz')['hashes']
assert all(len(v)==106 for v in w['runtimeHashes'].values())
changed=sorted(k for k,v in f['candidate'].items() if f['control'].get(k)!=v)
assert changed==sorted(f['changed'])==['full.js','pkg/pele.d.ts','pkg/pele.js','pkg/pele_bg.wasm','pkg/pele_bg.wasm.d.ts']
manifest=load('design/candidate-manifest.json');assert manifest['patchSha256']==sha(raw('design/candidate.patch.gz'))==before['patchSHA256']
for v in manifest['files']:assert before['files'][v['path']]==v['candidateSha256']
assert before['files']['bindings/web/full.js']==f['candidate']['full.js']
families=['flowchart','sequence','er','pie','sankey','gantt','railroad'];case_ids={f'{f}/{s}' for f in families for s in ['small','medium','large']}
assert {c['id'] for c in w['cases']}==case_ids
for c in w['cases']:
 assert {v['profile'] for v in c['checks']}=={'fixed','canvas'} and all(v['svgBytes']>0 for v in c['checks']);assert c['checksum']>0 and c['sourceBytes']>0 and len(c['sourceSHA256'])==64
expected={(c,p,s) for c in case_ids for p in ['fixed','canvas'] for s in ['render-source','parse','render-model']}
assert {(r['case'],r['profile'],r['stage']) for r in w['rows']}==expected
rows=[]
for r in w['rows']:
 assert r['counts']['candidate']==r['counts']['control'] and all(1<=n<=128 for n in r['counts'].values())
 for side,samples in r['samples'].items():
  assert len(samples)==9 and all(math.isfinite(x) and x>0 for x in samples)
  close(statistics.median(samples),r['medianMs'][side]);close(sorted(samples)[math.ceil(len(samples)*.95)-1],r['p95BatchMeanMs'][side])
  assert r['warmups'][side]['operations']>=6 and r['warmups'][side]['elapsedMs']>=500
 for name,a,b in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[name],r['medianMs'][a]/r['medianMs'][b])
 ratios=[a/c for a,c in zip(r['samples']['candidate'],r['samples']['control'])];median=statistics.median(ratios)
 rows.append(dict(case=r['case'],profile=r['profile'],stage=r['stage'],trials=9,counts=r['counts'],pairedRatioMedian=median,pairedPercent=(median-1)*100,pairedRatioMAD=statistics.median(abs(x-median) for x in ratios),pairedRatioMin=min(ratios),pairedRatioMax=max(ratios),candidateFasterPairs=sum(x<1 for x in ratios),controlMedianMs=r['medianMs']['control'],candidateMedianMs=r['medianMs']['candidate'],jsMedianMs=r['medianMs']['js'],candidateOverControl=r['candidateOverControl'],candidateOverJS=r['candidateOverJS']))
groups={}
for stage in ['render-source','parse','render-model']:
 selected=[r for r in rows if r['stage']==stage];values=[r['pairedPercent'] for r in selected]
 groups[stage]=dict(rows=len(selected),pairedLowerRows=sum(x<0 for x in values),medianOfRowPairedPercents=statistics.median(values),minPairedPercent=min(values),maxPairedPercent=max(values),minCandidateOverJS=min(r['candidateOverJS'] for r in selected),maxCandidateOverJS=max(r['candidateOverJS'] for r in selected),adverseRows=[dict(case=r['case'],profile=r['profile'],pairedPercent=r['pairedPercent'],candidateFasterPairs=r['candidateFasterPairs']) for r in selected if r['pairedPercent']>0])
family={f:statistics.median(r['pairedPercent'] for r in rows if r['stage']=='render-source' and r['case'].split('/')[0]==f) for f in families}
orders=[];positions={n:collections.Counter() for n in ['js','control','candidate']}
for trial in range(9):
 names=['js','control','candidate'];order=names[trial%3:]+names[:trial%3]
 if (trial//3)%2:order.reverse()
 orders.append(order)
 for n in names:positions[n][order.index(n)]+=1
assert all(dict(v)=={0:3,1:3,2:3} for v in positions.values())
control_first=sum(o.index('control')<o.index('candidate') for o in orders);assert control_first==5
reuse1=load('swift/swift-reuse-before.json.gz');reuse2=load('swift/swift-reuse-after.json.gz');correct=load('swift/correctness.json');api=load('swift/source-result-packet-swift-api.json.gz')
assert reuse1['current']==reuse2['current'];assert len(reuse1['current']['files'])==351
assert reuse1['current']['files']==before['nativeInputs']==reuse1['manifest']['inputHashes']==api['buildManifest']['inputHashes']
assert reuse1['current']['sha256']==before['nativeInputSHA256']==f['nativeInputSHA256']=='439fe2e6a023eebcafb980508b447162cdd5cd9c07519200df47f5ab3fcaea28'
assert reuse1['binarySha256']==reuse2['binarySha256']==api['buildManifest']['binarySha256']=='554ef37b648587962511c5e9eaa7ef639c469d87be601996817a1c9d762957fa'
assert reuse1['archiveSha256']==reuse2['archiveSha256']==reuse1['targetArchiveSha256']==reuse2['targetArchiveSha256']
assert api['mode']=='swift-api-correctness-only' and len(api['rows'])==168 and len(api['cases'])==21
assert correct['swiftSelectedIntegrationCases']==15 and 'Executed 15 tests, with 0 failures' in raw('logs/swift-corpus.log.gz').decode()
assert '33 passed (31.0s)' in raw('logs/focused-web.log.gz').decode()
assert len(load('prior-native/c-consumers.json'))==6 and all(r['exitCode']==0 for r in load('prior-native/c-consumers.json'))
assert sha(raw('prior-native/c-consumers.json'))==correct['strictCConsumers']['referenceSha256']
assert sha(raw('prior-native/dense-commit-binding.json.gz'))==correct['strictCConsumers']['nativeCommitBindingSha256']
test_index=load('tests/focused-index.json');assert test_index['reportCount']==24
for entry in test_index['reports']:
 assert sha(raw(entry['path']))==entry['uncompressedSha256'];data=load(entry['path']);name=Path(entry['path']).name
 if name=='source-packet-all-families.json.gz':
  assert len(data)==34
  for r in data:
   assert r['packet']==r['legacy'];assert {k:v for k,v in r['packet'].items() if k!='transport'}==r['oracle'];assert r['packet']['ok'] and r['packet']['transport']['ok']
  assert next(r for r in data if r['id']=='flowchart/links')['packet']['value']['links']
 elif name.startswith('source-packet-free-'):
  assert len(data['rows'])==33 and all(r['actual']==r['oracle'] and r['actual']['ok'] for r in data['rows']);assert data['packets']==(33 if name=='source-packet-free-true.json.gz' else 0)
 elif name=='source-packet-controls.json.gz':
  assert all(r['packet']==r['legacy'] for r in data['rows']);assert all(r['packet']['original'] for r in data['rows'] if r['variant'].startswith('throw-'));assert not next(r for r in data['rows'] if r['variant']=='nan-dimensions')['packet']['threw']
 elif name=='source-packet-envelope-limit.json.gz':
  assert dict(data[1],mode='legacy')==data[0];assert data[1]['expectedBytes']>64*1024*1024;assert data[1]['failure']['message']=='Browser numeric transport exceeds byte budget'
 elif name=='er-result-packet-differential.json.gz':assert all(r['packet']==r['json'] for r in data['rows'])
 elif name=='er-result-packet-errors.json.gz':assert all(r['packet']==r['json'] for r in data)
 elif name=='er-result-packet-budget.json.gz':
  for mode in [False,True]:
   old=next(r for r in data if r['raw']==mode and r['mode']=='json');new=next(r for r in data if r['raw']==mode and r['mode']=='packet');assert dict(new,mode='json')==old
 else:raise AssertionError(name)
size=load('package-size.json')
for side in ['control','candidate']:assert size['wasm'][side]['rawSha256']==f[side]['pkg/pele_bg.wasm']
for key,value in size['deltas'].items():assert value['bytes']==size['wasm']['candidate'][key]-size['wasm']['control'][key];close(value['percent'],(size['wasm']['candidate'][key]/size['wasm']['control'][key]-1)*100)
profiles=load('profile/summary.json.gz')
assert profiles['runtimeHashes']==f['candidate'] and profiles['wasmSHA256']==f['candidate']['pkg/pele_bg.wasm']
assert profiles['harnessSHA256']==sha(raw('harness/profile-web.mjs')) and profiles['casesSHA256']==w['casesSHA256']
assert profiles['durationMs']==1500 and profiles['samplingIntervalUs']==250 and len(profiles['profiles'])==18
profile_rows=[]
for row in profiles['profiles']:
 name=row['case'].replace('/','-')+'-'+row['stage']+'-'+row['runtime']+'.cpuprofile.gz'
 data=load('profile/'+name);nodes={n['id']:n for n in data['nodes']};self_time=collections.Counter()
 for index,node_id in enumerate(data.get('samples',[])):
  frame=nodes[node_id]['callFrame'];label=frame['functionName'] or frame.get('url') or '(anonymous)'
  self_time[label]+=data.get('timeDeltas',[])[index] if index<len(data.get('timeDeltas',[])) else 250
 total=sum(self_time.values());assert total>0 and row['operations']>0
 for item in row['top']:close(item['percent'],100*self_time[item['name']]/total)
 assert all(row['top'][i]['percent']>=row['top'][i+1]['percent'] for i in range(len(row['top'])-1))
 profile_rows.append(dict(case=row['case'],stage=row['stage'],runtime=row['runtime'],operations=row['operations'],sampleCount=len(data.get('samples',[])),weightedSelfUs=total,rawProfileSha256=sha(raw('profile/'+name))))
assert {(r['case'],r['stage'],r['runtime']) for r in profile_rows}=={(f+'/medium',s,r) for f in ['sankey','sequence','railroad'] for s in ['parse','render-source','render-model'] for r in ['js','wasm']}
commit_proof=None
if status['candidateSourceCommit']:
 commit_proof=load('freeze/source-commit-proof.json.gz');assert commit_proof['commit']==status['candidateSourceCommit'];assert commit_proof['files']==before['files'];assert commit_proof['allMatched']
if status['fullBrowser']['status']=='passed':assert f"{status['fullBrowser']['checks']} passed" in raw('logs/full-web.log.gz').decode()
limitations=['Diagnostic only; no new full performance release gate or JS parity claim.','Nine three-way trials per row, 500 ms warmup, max128 operations per batch; shared browser/page heaps and caches.','Paired percentages are median(candidate[i]/control[i])-1. Family/stage aggregates are medians of row percentages, not pooled samples or causal estimates.','All adverse parse and retained controls are retained; no subtraction or noise/regression-free claim.','Harness verifies complete SVG and Object.is dimensions before/after. Report keeps check byte lengths and checksums, not the complete SVG; public E2E reports separately retain full results and counts.','Three-way positions balance 3/3/3 per runtime; control precedes candidate in 5 of9 trials per row. Raw execution orders are implied by the frozen harness, not recorded separately.','Swift evidence is fresh correctness using identical dense native inputs/SDK/binary. C3 is reused prior identical-native validation. No Swift performance or new native build claim.','WASM gzip/Brotli sizes are compressed-file diagnostics, not live memory or runtime latency.','Full405 browser correctness and immutable640deee source binding are separate from a performance release gate. Eighteen same-package profiles are diagnostic samples, not paired latency or an isolated before/after profile comparison.']
out=dict(status=status,comparisonSha256=sha(raw('comparison.json.gz')),rows=rows,stageSummary=groups,sourceFamilyMedianOfRowPairedPercents=family,trialOrderDerivedFromFrozenHarness=orders,controlBeforeCandidatePerRow=control_first,candidateBeforeControlPerRow=9-control_first,packageChangedFiles=changed,packageFileCountPerSide=106,sourceFiles=657,nativeInputCount=351,validation=dict(focusedBrowser=33,explicitFocusedReports=24,swiftSelected=15,swiftAPI=168,swiftWorkloads=21,strictC='Prior3 reused',fullBrowser=status['fullBrowser']),wasmSize=size,profiles=profile_rows,limitations=limitations)
write('summary.json',out)
md=['# Complete source-result packet comparison','','All126 rows retained. Positive paired change means slower. Paired change is a median of within-trial ratios, distinct from the ratio of standalone medians.','', '| Case | Profile | Stage | Control ms | Candidate ms | Paired change | Faster pairs | Candidate / JS |','|---|---|---|---:|---:|---:|---:|---:|']
for r in rows:md.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['controlMedianMs']:.6f} | {r['candidateMedianMs']:.6f} | {r['pairedPercent']:+.3f}% | {r['candidateFasterPairs']}/9 | {r['candidateOverJS']:.3f}x |")
(B/'RESULTS.md').write_text('\n'.join(md)+'\n',encoding='ascii')
files=[]
for p in sorted(B.rglob('*')):
 if not p.is_file() or p.name=='evidence-index.json' or '__pycache__' in p.parts:continue
 data=p.read_bytes();entry=dict(path=p.relative_to(B).as_posix(),bytes=len(data),sha256=sha(data))
 if p.suffix=='.gz':plain=gzip.decompress(data);entry.update(uncompressedBytes=len(plain),uncompressedSha256=sha(plain))
 files.append(entry)
write('evidence-index.json',dict(schemaVersion=1,method='Every other file SHA256. Deterministic gzip mtime0. No generated timestamp in summary/index.',files=files,totalBytes=sum(x['bytes'] for x in files)))
print('Verified126 VM rows,106 files/side,657 source hashes,Swift15/API168,24 explicit browser reports,18 profiles; indexed',len(files),'files')
