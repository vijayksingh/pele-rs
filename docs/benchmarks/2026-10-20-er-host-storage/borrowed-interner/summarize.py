#!/usr/bin/env python3
"""Summarize bundled frozen data only. No source read, network, compiler or workload."""
from pathlib import Path
import json,gzip,hashlib,math,statistics,collections,re
B=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def data(p):
 p=B/p;v=p.read_bytes();return gzip.decompress(v) if p.suffix=='.gz' else v
def load(p):return json.loads(data(p))
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
def paired(a,c):
 ratios=[x/y for x,y in zip(a,c)];m=statistics.median(ratios)
 return dict(pairedPercent=(m-1)*100,pairedRatioMedian=m,pairedRatioMAD=statistics.median(abs(x-m) for x in ratios),pairedRatioMin=min(ratios),pairedRatioMax=max(ratios),candidateFasterPairs=sum(x<1 for x in ratios),controlMedianMs=statistics.median(c),candidateMedianMs=statistics.median(a))
proof=load('source-package-proof.json');n=load('native/native-comparison-attempt1.json');w=load('web-comparison.json');s=load('swift/swift-comparison-attempt1.json.gz');alloc=load('native/allocation-comparison.json');review=load('native/allocation-review.json')
assert n['succeeded'] and len(n['rows'])==8 and n['settings']['trials']==15 and n['settings']['fixedBatchCount']==16 and n['settings']['warmupMs']==1000
native=[]
for side in ['control','candidate']:
 m=load('native/'+side+'/build-manifest.json');assert m==n['artifacts'][side]['manifest'];assert m['binaries']==proof['nativeConsumer'][side]['actualFrozenBinaryHashes'];assert m['sourceInputSha256']==proof['nativeConsumer'][side]['sourceInputSha256'];assert n['artifacts'][side]['binarySHA256']==m['binaries']['pele-perf'];assert alloc['runs'][side]['manifest']==m;assert alloc['runs'][side]['binarySHA256']==m['binaries']['pele-perf-alloc']
for r in n['rows']:
 for side,values in r['samples'].items():
  assert len(values)==15 and r['counts'][side]==16 and r['warmups'][side]['elapsedNs']>=1e9
  for x,v in zip(values,r['raw'][side]):close(x,v['elapsedNs']/v['count']/1e6);assert v['count']==16
  close(statistics.median(values),r['medians'][side])
 close(r['candidateOverControl'],r['medians']['candidate']/r['medians']['control']);close(r['candidateOverJS'],r['medians']['candidate']/r['medians']['js'])
 for a,c in zip(r['raw']['candidate'],r['raw']['control']):assert {k:v for k,v in a.items() if k!='elapsedNs'}=={k:v for k,v in c.items() if k!='elapsedNs'}
 assert r['jsRetainedPostBatchVerified'];native.append(dict(case=r['case'],stage=r['stage'],trials=15,jsMedianMs=r['medians']['js'],candidateOverJS=r['candidateOverJS'],**paired(r['samples']['candidate'],r['samples']['control'])))
assert review['reportSha256']==sha(data('native/allocation-comparison.json'));assert len(alloc['rows'])==10 and alloc['timingSamples'] is False
for i,command in enumerate(alloc['commands']):
 a=alloc['runs']['candidate']['responses'][i];c=alloc['runs']['control']['responses'][i];assert a['ok'] and c['ok']
 if command['op']!='allocations':assert a==c
for r in alloc['rows']:
 for k,v in r['delta'].items():assert r['candidate'][k]-r['control'][k]==v
 assert r['candidate']['checksum']==r['control']['checksum']
negative=[r for r in alloc['rows'] if not(r['case']=='er/medium' and r['stage']=='parse-public')];assert len(negative)==8 and all(r['candidate']==r['control'] for r in negative)
assert w['succeeded'] and len(w['rows'])==18 and w['settings']['trials']==9 and w['settings']['warmupMs']==500
assert w['runtimeHashes']['candidate']==proof['packageFiles'] and w['runtimeHashes']['control']==proof['controlPackageFiles']
web=[]
for r in w['rows']:
 assert r['counts']['control']==r['counts']['candidate']
 for side,values in r['samples'].items():
  assert len(values)==9 and r['warmups'][side]['elapsedMs']>=500 and all(math.isfinite(v) and v>0 for v in values);close(statistics.median(values),r['medianMs'][side]);close(sorted(values)[math.ceil(len(values)*.95)-1],r['p95BatchMeanMs'][side])
 for k,a,c in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[k],r['medianMs'][a]/r['medianMs'][c])
 web.append(dict(case=r['case'],profile=r['profile'],stage=r['stage'],trials=9,scope='Negative control: ordinary host path does not execute borrowed GraphWriter helper',candidateOverJS=r['candidateOverJS'],**paired(r['samples']['candidate'],r['samples']['control'])))
assert s['completed'] and len(s['rows'])==6
assert s['builds']['candidate']['manifest']==load('swift/swift-build-manifest.json');assert s['builds']['control']['manifest']==load('swift/control-build-manifest.json')
assert s['builds']['candidate']['manifest']['inputHashes']==load('swift/source-freeze.json')['inputHashes'];assert sha(data('swift/swift-comparison-attempt1.json.gz'))==load('swift/swift-after-freeze.json')['reportSha256'];assert len(s['changedInputs'])==3
sw=[];orders=collections.Counter();guards=0
for ri,r in enumerate(s['rows']):
 assert len(r['raw'])==15
 for phase in [r['before'],r['after']]+r['calibration']+r['raw']:
  v=phase['values'];assert {k:x for k,x in v['control'].items() if k!='elapsedNs'}=={k:x for k,x in v['candidate'].items() if k!='elapsedNs'}
 for ti,p in enumerate(r['raw']):
  assert p['order']==(['control','candidate'] if (ri+ti+s['settings']['orderOffset'])%2==0 else ['candidate','control']);orders[p['order'][0]]+=1
  for side in ['control','candidate']:
   assert p['values'][side]['count']==r['count'];guards+=1
   assert not any(x['name'] in ['cargo','rustc','swiftc','swift-build','swift-frontend','xcodebuild','ld'] or(x['name']=='xctest' and x.get('testScope')=='pele') for x in p['guards'][side]['externalLoad'])
 a=[p['values']['candidate']['elapsedNs']/r['count']/1e6 for p in r['raw']];c=[p['values']['control']['elapsedNs']/r['count']/1e6 for p in r['raw']];v=paired(a,c)
 assert a==r['statistics']['candidateMs'] and c==r['statistics']['controlMs'];close(v['pairedRatioMedian'],r['statistics']['pairedCandidateOverControl']['median']);assert v['candidateFasterPairs']==r['statistics']['candidateFasterPairs'];assert r['warmup']['control']['operations']==r['warmup']['candidate']['operations'];assert all(r['warmup'][x]['elapsedNs']>=1e9 for x in ['control','candidate']);sw.append(dict(case=r['case'],stage=r['stage'],trials=15,count=r['count'],**v))
assert dict(orders)==s['orderCounts']=={'control':45,'candidate':45}
api=load('swift/swift-api.json.gz');assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only';assert api['buildManifest']==s['builds']['candidate']['manifest']
assert len(load('swift/c-consumers.json'))==6 and all(x['exitCode']==0 for x in load('swift/c-consumers.json'))
assert '27 passed (19.0s)' in data('logs/web.log.gz').decode() or '27 passed (19s)' in data('logs/web.log.gz').decode()
assert 'Executed 15 tests, with 0 failures' in data('logs/swift-corpus.log.gz').decode();assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed; 0 failed',data('logs/native.log.gz').decode())))==27
out=dict(sourceCommit=proof['sourceCommit'],native=native,allocations=alloc['rows'],unchangedAllocationControlRows=8,swift=sw,swiftOrderCounts=dict(orders),swiftMeasuredGuardSnapshots=guards,webNegativeControls=web,validation=dict(nativeIntegrations=27,browserChecks=27,swiftIntegrationCases=15,swiftApiConditions=168,strictCConsumers=3,prior390BrowserSuite='Prior bd7df0d only; not rerun or claimed for this runtime'),limitations=['Native completed report reflects strict harness guard checks, but individual guard snapshots/order timestamps are not stored. Swift preserves each measured guard.','VM is a short 9-trial/500ms diagnostic without per-row process guard; all18 rows are negative controls, including positive changes.','Allocation counters are requested bytes, peak additional requested bytes and live delta, not RSS. Timed and allocation binaries are separate.','Native timing covers one medium source for each of two families. Integration threshold/unique cases are not a unique-heavy performance distribution.','No new release performance gate; no0.3 or JS parity claim.'])
(B/'summary.json').write_text(json.dumps(out,indent=2)+'\n')
md=['# Complete results','','Paired change is median of trial ratios. Positive means slower. All controls retained.','']
for title,rows in [('Native —15 trials, fixed batch16',native),('Swift —15 pairs, isolated versus record',sw),('VM negative controls —9 trials, 500ms warmup',web)]:
 md+=['## '+title,'','| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs |','|---|---|---:|---:|---:|---:|']
 for r in rows:md.append(f"| {r['case']} | {r.get('profile','')} {r['stage']} | {r['controlMedianMs']:.6f} | {r['candidateMedianMs']:.6f} | {r['pairedPercent']:+.3f}% | {r['candidateFasterPairs']}/{r['trials']} |")
 md+=['']
md+=['## Allocation counters —count1 and count16','','| Case / stage | Count | Allocations control / candidate | Reallocations | Requested bytes | Peak additional bytes | Live delta |','|---|---:|---:|---:|---:|---:|---:|']
for r in alloc['rows']:
 fields=['allocations','reallocations','requestedBytes','peakAdditionalRequestedBytes','liveBytesDelta'];values=[f"{r['control'][k]} / {r['candidate'][k]}" for k in fields];md.append(f"| {r['case']} {r['stage']} | {r['count']} | "+' | '.join(values)+' |')
(B/'RESULTS.md').write_text('\n'.join(md)+'\n')
index=[]
for p in sorted(B.rglob('*')):
 if not p.is_file() or p.name=='evidence-index.json' or '__pycache__' in p.parts:continue
 raw=p.read_bytes();entry=dict(path=str(p.relative_to(B)),bytes=len(raw),sha256=sha(raw))
 if p.suffix=='.gz':v=gzip.decompress(raw);entry.update(uncompressedBytes=len(v),uncompressedSha256=sha(v))
 index.append(entry)
(B/'evidence-index.json').write_text(json.dumps(dict(schemaVersion=1,method='SHA256 every other bundled file; gzip mtime0; no generated timestamp',files=index,totalBytes=sum(x['bytes'] for x in index)),indent=2)+'\n')
print('Verified native8 + Swift6 + VM18 rows, allocation10, validation logs; indexed',len(index),'files')
for r in native:print(r['case'],r['stage'],round(r['pairedPercent'],3),r['candidateFasterPairs'])
for r in web:print('VMcontrol',r['case'],r['profile'],r['stage'],round(r['pairedPercent'],3),r['candidateFasterPairs'])
