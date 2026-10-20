#!/usr/bin/env python3
"""Data-only frozen evidence summary. No live source, network, builds or workloads."""
from pathlib import Path
import collections,gzip,hashlib,json,math,re,statistics
B=Path(__file__).resolve().parent;sha=lambda b:hashlib.sha256(b).hexdigest()
def data(p):
 p=B/p;v=p.read_bytes();return gzip.decompress(v) if p.suffix=='.gz' else v
def load(p):return json.loads(data(p))
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
def paired(a,c):
 ratios=[x/y for x,y in zip(a,c)];m=statistics.median(ratios)
 return dict(pairedPercent=(m-1)*100,pairedRatioMedian=m,pairedRatioMAD=statistics.median(abs(x-m) for x in ratios),pairedRatioMin=min(ratios),pairedRatioMax=max(ratios),candidateFasterPairs=sum(x<1 for x in ratios),controlMedianMs=statistics.median(c),candidateMedianMs=statistics.median(a))
proof=load('source-commit-proof.json.gz');f=load('freeze/source-package-freeze.json.gz');before=load('freeze/source-before.json.gz');w=load('comparison.json.gz');first=load('first-use.json.gz');s=load('swift/swift-comparison-attempt1.json.gz');alloc=load('native/allocation-comparison.json.gz')
assert sha(data('freeze/source-before.json.gz'))==f['beforeSha256'];assert f['sourceFiles']==before['sourceFiles'];assert w['runtimeHashes']==first['runtimeHashes'];assert w['runtimeHashes']['candidate']==f['packageFiles']==proof['packageFiles']
web={}
for title,report,n,trials in [('ordinary',w,24,21),('firstUse',first,36,15)]:
 assert report['succeeded'] and len(report['rows'])==n and report['settings']['trials']==trials;result=[]
 for r in report['rows']:
  assert r['counts']['candidate']==r['counts']['control']
  for side,values in r['samples'].items():
   assert len(values)==trials and all(math.isfinite(v) and v>0 for v in values);close(statistics.median(values),r['medianMs'][side]);close(sorted(values)[math.ceil(len(values)*.95)-1],r['p95BatchMeanMs'][side]);assert r['warmups'][side]['elapsedMs']>=1000
  for k,a,c in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[k],r['medianMs'][a]/r['medianMs'][c])
  result.append(dict(case=r['case'],profile=r['profile'],stage=r['stage'],trials=trials,counts=r['counts'],candidateOverJS=r['candidateOverJS'],**paired(r['samples']['candidate'],r['samples']['control'])))
 web[title]=result
assert s['completed'] and len(s['rows'])==6
m=load('swift/swift-build-manifest.json');assert m==s['builds']['candidate']['manifest'];assert load('swift/control-build-manifest.json')==s['builds']['control']['manifest'];assert m['inputHashes']==load('swift/source-freeze.json.gz')['inputHashes'];assert len(s['changedInputs'])==2
assert sha(data('swift/swift-comparison-attempt1.json.gz'))==load('swift/swift-after-freeze.json')['reportSha256']
sw=[];orders=collections.Counter();guards=0
for ri,r in enumerate(s['rows']):
 assert len(r['raw'])==15
 for p in [r['before'],r['after']]+r['calibration']+r['raw']:
  v=p['values'];assert {k:x for k,x in v['candidate'].items() if k!='elapsedNs'}=={k:x for k,x in v['control'].items() if k!='elapsedNs'}
 for ti,p in enumerate(r['raw']):
  assert p['order']==(['control','candidate'] if (ri+ti+s['settings']['orderOffset'])%2==0 else ['candidate','control']);orders[p['order'][0]]+=1
  for side in ['control','candidate']:
   assert p['values'][side]['count']==r['count'];guards+=1
   assert not any(x['name'] in ['cargo','rustc','swiftc','swift-build','swift-frontend','xcodebuild','ld'] or(x['name']=='xctest' and x.get('testScope')=='pele') for x in p['guards'][side]['externalLoad'])
 a=[p['values']['candidate']['elapsedNs']/r['count']/1e6 for p in r['raw']];c=[p['values']['control']['elapsedNs']/r['count']/1e6 for p in r['raw']];stats=paired(a,c);assert a==r['statistics']['candidateMs'] and c==r['statistics']['controlMs'];close(stats['pairedRatioMedian'],r['statistics']['pairedCandidateOverControl']['median']);assert stats['candidateFasterPairs']==r['statistics']['candidateFasterPairs'];assert r['warmup']['control']['operations']==r['warmup']['candidate']['operations'];assert all(r['warmup'][side]['elapsedNs']>=1e9 for side in ['control','candidate']);sw.append(dict(case=r['case'],stage=r['stage'],trials=15,count=r['count'],**stats))
assert dict(orders)==s['orderCounts']=={'control':45,'candidate':45}
assert len(alloc['rows'])==10 and not alloc['timingSamples'];assert load('native/allocation-review.json')['reportSha256']==sha(data('native/allocation-comparison.json.gz'))
for side in ['control','candidate']:
 nm=load('native/'+side+'/build-manifest.json');assert nm==alloc['runs'][side]['manifest'];assert nm['binaries']==proof['nativeConsumer'][side]['actualFrozenBinaryHashes'];assert nm['sourceInputSha256']==proof['nativeConsumer'][side]['sourceInputSha256'];assert alloc['runs'][side]['binarySHA256']==nm['binaries']['pele-perf-alloc']
for a,c in zip(alloc['runs']['candidate']['responses'],alloc['runs']['control']['responses']):assert a==c
assert all(r['candidate']==r['control'] and all(v==0 for v in r['delta'].values()) for r in alloc['rows'])
api=load('swift/swift-api.json.gz');assert len(api['rows'])==168 and len(api['cases'])==21 and api['buildManifest']==m
assert len(load('swift/c-consumers.json'))==6 and all(x['exitCode']==0 for x in load('swift/c-consumers.json'))
assert 'Executed 15 tests, with 0 failures' in data('logs/swift-corpus.log.gz').decode();assert '84 passed (48.6s)' in data('logs/web.log.gz').decode();assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed; 0 failed',data('logs/native.log.gz').decode())))==27
out=dict(sourceCommit=proof['sourceCommit'],ordinary=web['ordinary'],firstUse=web['firstUse'],swift=sw,swiftPairs=90,swiftOrderCounts=dict(orders),swiftMeasuredGuardSnapshots=guards,allocations=alloc['rows'],allTenAllocationRowsIdentical=True,validation=dict(nativeIntegrations=27,browserChecks=84,swiftIntegrationCases=15,swiftApiConditions=168,strictCConsumers=3,typescript='Retained passing parent log',prior390BrowserSuite='Prior bd7df0d only, not rerun here'),limitations=['The default native sink preserves exact graphs but retains caller Vec through node publication. Identical measured medium counters do not guarantee universal peak memory.','Swift adverse ER mutable/lifetime results retained; no native speedup claim.','VM medians are paired ratios, not confidence intervals or general causal estimates. Shared browser/page and serial warmup differ from release-gate scheduling.','No new full performance release gate, no0.3 or JS parity claim.'])
rejected=load('rejected-index/comparison.json.gz');rf=load('rejected-index/package-freeze.json.gz');rm=load('rejected-index/manifest.json')
assert rejected['succeeded'] and len(rejected['rows'])==24 and rejected['settings']['trials']==21
assert rejected['runtimeHashes']['control']==w['runtimeHashes']['candidate']==rf['control'];assert rejected['runtimeHashes']['candidate']==rf['candidate']
assert [k for k,v in rf['candidate'].items() if v!=rf['control'][k]]==rf['changed']==['host-er-store.js'];assert rf['wasmUnchanged'] and rf['nativeSDKUnchanged'];assert rf['nativeInputSHA256']==m['sourceInputSha256']
assert rm['files'][0]['originalSha256']==rf['control']['host-er-store.js'];assert rm['files'][0]['candidateSha256']==rf['candidate']['host-er-store.js'];assert sha(data('rejected-index/candidate.patch'))==rm['patchSha256']
assert '84 passed (46.6s)' in data('rejected-index/browser.log.gz').decode()
rr=[]
for r in rejected['rows']:
 assert r['counts']['candidate']==r['counts']['control']
 for side,values in r['samples'].items():
  assert len(values)==21 and r['warmups'][side]['elapsedMs']>=1000 and all(math.isfinite(v) and v>0 for v in values);close(statistics.median(values),r['medianMs'][side]);close(sorted(values)[math.ceil(len(values)*.95)-1],r['p95BatchMeanMs'][side])
 for k,a,c in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[k],r['medianMs'][a]/r['medianMs'][c])
 rr.append(dict(case=r['case'],profile=r['profile'],stage=r['stage'],trials=21,candidateOverJS=r['candidateOverJS'],**paired(r['samples']['candidate'],r['samples']['control'])))
rs=dict(status='Rejected and reverted; separate experiment, excluded from accepted dense runtime',rows=rr,reportSha256=sha(data('rejected-index/comparison.json.gz')),browserChecks=84,controlMatchesDense=True,changedPackageFiles=rf['changed'],wasmUnchanged=True,nativeInputSha256=rf['nativeInputSHA256'],firstUseRun=False,nativeSourceChange=False)
(B/'rejected-index/summary.json').write_text(json.dumps(rs,indent=2)+'\n');out['rejectedIndex']={k:v for k,v in rs.items() if k!='rows'};out['rejectedIndex']['rows']=24;out['rejectedIndex']['summary']='rejected-index/summary.json'
rmtext=['# Rejected canonical-string-index results','','All24 rows retained. Positive paired change means slower.','', '| Case | Profile | Stage | Paired change | Faster pairs | Candidate / JS |','|---|---|---|---:|---:|---:|']
for r in rr:rmtext.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedPercent']:+.3f}% | {r['candidateFasterPairs']}/21 | {r['candidateOverJS']:.3f}× |")
(B/'rejected-index/RESULTS.md').write_text('\n'.join(rmtext)+'\n')
(B/'summary.json').write_text(json.dumps(out,indent=2)+'\n')
md=['# Complete dense-array paired results','','Positive change means slower. All rows retained; no control subtraction or adverse-result dismissal.','']
for title,rows in [('Ordinary VM —21 trials',web['ordinary']),('First-use VM —15 trials',web['firstUse']),('Swift —15 pairs against frozen d25',sw)]:
 md+=['## '+title,'','| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs | Candidate / JS |','|---|---|---:|---:|---:|---:|---:|']
 for r in rows:
  js=f"{r['candidateOverJS']:.3f}×" if 'candidateOverJS' in r else '—';md.append(f"| {r['case']} | {r.get('profile','')} {r['stage']} | {r['controlMedianMs']:.6f} | {r['candidateMedianMs']:.6f} | {r['pairedPercent']:+.3f}% | {r['candidateFasterPairs']}/{r['trials']} | {js} |")
 md+=['']
md+=['## Allocation rows —all metrics identical','','| Case / stage | Count | Allocations | Reallocations | Requested bytes | Peak additional bytes | Live delta |','|---|---:|---:|---:|---:|---:|---:|']
for r in alloc['rows']:
 a=r['candidate'];md.append(f"| {r['case']} {r['stage']} | {r['count']} | {a['allocations']} | {a['reallocations']} | {a['requestedBytes']} | {a['peakAdditionalRequestedBytes']} | {a['liveBytesDelta']} |")
(B/'RESULTS.md').write_text('\n'.join(md)+'\n')
files=[]
for p in sorted(B.rglob('*')):
 if not p.is_file() or p.name=='evidence-index.json' or '__pycache__' in p.parts:continue
 raw=p.read_bytes();r=dict(path=str(p.relative_to(B)),bytes=len(raw),sha256=sha(raw))
 if p.suffix=='.gz':v=gzip.decompress(raw);r.update(uncompressedBytes=len(v),uncompressedSha256=sha(v))
 files.append(r)
(B/'evidence-index.json').write_text(json.dumps(dict(schemaVersion=1,method='All other files SHA256; deterministic gzip mtime0; no generated timestamp',files=files,totalBytes=sum(f['bytes'] for f in files)),indent=2)+'\n')
print('Verified VM24 + first-use36 + Swift6 + allocation10; indexed',len(files),'files')
