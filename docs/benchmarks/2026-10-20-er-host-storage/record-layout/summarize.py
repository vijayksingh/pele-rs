#!/usr/bin/env python3
"""Recompute bundled frozen data only; no source reads, build, browser or network."""
from pathlib import Path
import collections,gzip,hashlib,json,math,re
B=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def bytes_at(path):
 p=B/path;return gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes()
def load(path):return json.loads(bytes_at(path))
q=lambda a,p:sorted(a)[max(0,math.ceil(len(a)*p)-1)]
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
def paired(a,c):
 ratios=[x/y for x,y in zip(a,c)];m=q(ratios,.5)
 return {'pairedPercent':(m-1)*100,'candidateFasterPairs':sum(x<1 for x in ratios),'ratioOfMediansPercent':(q(a,.5)/q(c,.5)-1)*100,'pairedRatioMedian':m,'pairedRatioMAD':q([abs(x-m) for x in ratios],.5),'pairedRatioMin':min(ratios),'pairedRatioMax':max(ratios),'pairedRatioSampleP95':q(ratios,.95)}
ordinary=load('comparison.json');first=load('first-use.json');profile=load('profile/summary.json');freeze=load('source-package-freeze.json');remote=load('remote-package-hashes.json');original_audit=load('review/audit.json')
assert ordinary['runtimeHashes']==first['runtimeHashes']==remote
assert ordinary['runtimeHashes']['candidate']==freeze['packageFiles']==profile['runtimeHashes']
assert ordinary['runtimeHashes']['control']==freeze['controlPackageFiles']
assert sha(bytes_at('comparison.json'))==original_audit['comparisonSHA256']
assert sha(bytes_at('remote-package-hashes.json'))==original_audit['remoteRehash']['sha256']
assert all(len(v)==106 for v in remote.values())
web={}
for title,report,n,trials in [('ordinary',ordinary,24,21),('firstUse',first,36,15)]:
 assert report['succeeded'] and len(report['rows'])==n and report['settings']['trials']==trials
 result=[]
 for r in report['rows']:
  samples=r['samples'];assert all(len(a)==trials for a in samples.values());assert r['counts']['control']==r['counts']['candidate']
  for lane,values in samples.items():
   assert all(math.isfinite(x) and x>0 for x in values);close(q(values,.5),r['medianMs'][lane]);close(q(values,.95),r['p95BatchMeanMs'][lane]);assert r['warmups'][lane]['elapsedMs']>=1000;count=r['counts'][lane];assert 1<=count<=128 and count&(count-1)==0
  for key,a,c in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[key],r['medianMs'][a]/r['medianMs'][c])
  result.append({'case':r['case'],'profile':r['profile'],'stage':r['stage'],'trials':trials,'medianMs':r['medianMs'],'counts':r['counts'],'candidateOverJS':r['candidateOverJS'],**paired(samples['candidate'],samples['control'])})
 web[title]={'reportSha256':sha(bytes_at('comparison.json' if title=='ordinary' else 'first-use.json')),'trials':trials,'rows':result}
 # Derived schedule, not observed raw-order telemetry.
 orders=[]
 for i in range(trials):
  names=['js','control','candidate'];order=names[i%3:]+names[:i%3]
  if (i//3)%2:order.reverse()
  orders.append(order)
 web[title]['derivedFirstPositionsPerRow']=dict(collections.Counter(x[0] for x in orders))
 web[title]['derivedControlCandidatePrecedencePerRow']=dict(collections.Counter('control' if x.index('control')<x.index('candidate') else 'candidate' for x in orders))
profiles=[]
for row in profile['profiles']:
 path=f"profile/{row['case'].replace('/','-')}-{row['stage']}-{row['runtime']}.cpuprofile.gz";data=load(path);by={n['id']:n for n in data['nodes']};weights=collections.defaultdict(int)
 assert len(data['samples'])==len(data['timeDeltas'])
 for nid,delta in zip(data['samples'],data['timeDeltas']):
  assert delta>=0;f=by[nid]['callFrame'];weights[f['functionName'] or f['url'] or '(anonymous)']+=delta
 total=sum(weights.values());top=sorted(weights.items(),key=lambda item:-item[1])[:35]
 assert [name for name,_ in top]==[x['name'] for x in row['top']]
 for (_,us),saved in zip(top,row['top']):close(100*us/total,saved['percent'])
 profiles.append({'case':row['case'],'stage':row['stage'],'runtime':row['runtime'],'rawSha256':sha(bytes_at(path)),'samples':len(data['samples']),'weightedSampleUs':total,'top35Recomputed':True,'top5':row['top'][:5]})
assert len(profiles)==6
sw=load('native/swift-comparison-attempt1.json.gz');assert sw['completed'] and len(sw['rows'])==6
manifest=load('native/swift-build-manifest.json');assert sw['builds']['candidate']['manifest']==manifest
assert manifest['inputHashes']==freeze['nativeSDKInputMap'];assert manifest['sourceInputSha256']==freeze['nativeSourceInputSha256'];assert manifest['binarySha256']==freeze['consumerBinarySha256']
sw_rows=[];orders=collections.Counter();guards=0
for ri,r in enumerate(sw['rows']):
 assert len(r['raw'])==15
 for phase in [r['before'],r['after']]+r['calibration']+r['raw']:
  c=phase['values']['control'];a=phase['values']['candidate'];assert {k:v for k,v in c.items() if k!='elapsedNs'}=={k:v for k,v in a.items() if k!='elapsedNs'}
 for ti,pair in enumerate(r['raw']):
  assert pair['order']==(['control','candidate'] if (ri+ti+sw['settings']['orderOffset'])%2==0 else ['candidate','control']);orders[pair['order'][0]]+=1
  for side in ['control','candidate']:
   assert pair['values'][side]['count']==r['count'];guards+=1
   assert not any(p['name'] in ['cargo','rustc','swiftc','swift-build','swift-frontend','xcodebuild','ld'] or(p['name']=='xctest' and p.get('testScope')=='pele') for p in pair['guards'][side]['externalLoad'])
 a=[p['values']['candidate']['elapsedNs']/r['count']/1e6 for p in r['raw']];c=[p['values']['control']['elapsedNs']/r['count']/1e6 for p in r['raw']];stats=paired(a,c)
 close(stats['pairedRatioMedian'],r['statistics']['pairedCandidateOverControl']['median']);assert stats['candidateFasterPairs']==r['statistics']['candidateFasterPairs'];assert r['warmup']['candidate']['elapsedNs']>=1e9 and r['warmup']['control']['elapsedNs']>=1e9;assert r['warmup']['candidate']['operations']==r['warmup']['control']['operations']
 for x,y in zip(a,r['statistics']['candidateMs']):close(x,y)
 for x,y in zip(c,r['statistics']['controlMs']):close(x,y)
 sw_rows.append({'case':r['case'],'stage':r['stage'],'trials':15,'count':r['count'],'controlMedianMs':q(c,.5),'candidateMedianMs':q(a,.5),**stats})
assert dict(orders)==sw['orderCounts']=={'control':45,'candidate':45}
api=load('native/swift-api.json.gz');assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only';assert api['buildManifest']==manifest
assert all(x['exitCode']==0 for x in load('native/c-consumers.json'))
assert len(load('native/c-consumers.json'))==6
assert '390 passed (12.9m)' in bytes_at('logs/browser-full.log.gz').decode()
assert '12 passed' in bytes_at('logs/browser-initial.log.gz').decode()
assert 'Executed 15 tests, with 0 failures' in bytes_at('logs/swift-integrations.log.gz').decode()
native_log=bytes_at('logs/native-integrations.log.gz').decode();assert sum(map(int,re.findall(r'test result: ok\. (\d+) passed; 0 failed',native_log)))==24
summary={'schemaVersion':1,'sourceCommit':freeze['sourceCommit'],'defaultOptimizedBuildOnly':True,'nativeSourceInputSha256':freeze['nativeSourceInputSha256'],'ordinary':web['ordinary'],'firstUse':web['firstUse'],'swift':{'reportSha256':sha(bytes_at('native/swift-comparison-attempt1.json.gz')),'rows':sw_rows,'pairs':90,'firstOrderCounts':dict(orders),'savedMeasuredGuardsVerified':guards,'controlSource':sw['builds']['control']['manifest']['revision'],'candidateBuildSource':manifest['revision'],'scope':'Combined native changes versus frozen0b, not isolated record-layout effect; no retries or pooled windows.'},'profiles':profiles,'validation':{'nativeIntegrations':24,'initialBrowserChecks':12,'fullBrowserChecks':390,'fullBrowserDuration':'12.9m','newRecordBrowserReports':3,'swiftIntegrations':15,'erDrawings':7535,'erErrors':9902,'erExplicitBudgetDifferences':0,'swiftApiConditions':168,'strictCConsumers':3},'pointInTimeNote':'review/REVIEW.md and review/audit.json predate final browser completion and this Swift timing. Their earlier running/not-audited fields are preserved, superseded here by separately retained completed logs/reports. native/correctness.json likewise predates the authorized timing window.'}
rejected=load('rejected-index/comparison.json');rm=load('rejected-index/manifest.json')
assert rejected['succeeded'] and len(rejected['rows'])==24 and rejected['settings']['trials']==21
assert rejected['runtimeHashes']['control']==ordinary['runtimeHashes']['candidate']
assert set(rejected['runtimeHashes']['control'])==set(rejected['runtimeHashes']['candidate'])
assert [k for k,v in rejected['runtimeHashes']['candidate'].items() if v!=rejected['runtimeHashes']['control'][k]]==['host-er-store.js']
assert rejected['runtimeHashes']['control']['host-er-store.js']==rm['originalSHA256']
assert rejected['runtimeHashes']['candidate']['host-er-store.js']==rm['candidateSHA256']
assert sha(bytes_at('rejected-index/candidate.patch'))==rm['patchSHA256']
assert '45 passed (29.1s)' in bytes_at('rejected-index/browser.log.gz').decode()
rejected_rows=[]
for r in rejected['rows']:
 assert r['counts']['control']==r['counts']['candidate']
 for lane,values in r['samples'].items():
  assert len(values)==21 and all(math.isfinite(x) and x>0 for x in values)
  close(q(values,.5),r['medianMs'][lane]);close(q(values,.95),r['p95BatchMeanMs'][lane]);assert r['warmups'][lane]['elapsedMs']>=1000
 for key,a,c in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[key],r['medianMs'][a]/r['medianMs'][c])
 rejected_rows.append({'case':r['case'],'profile':r['profile'],'stage':r['stage'],'trials':21,'medianMs':r['medianMs'],'candidateOverJS':r['candidateOverJS'],**paired(r['samples']['candidate'],r['samples']['control'])})
rejected_summary={'status':'Rejected and reversed; not accepted record-layout runtime','reportSha256':sha(bytes_at('rejected-index/comparison.json')),'controlMatchesAcceptedRecordCandidate':True,'onlyChangedPackageFile':'host-er-store.js','browserChecks':45,'rows':rejected_rows}
(B/'rejected-index/paired-summary.json').write_text(json.dumps(rejected_summary,indent=2)+'\n')
summary['rejectedIndex']={'status':rejected_summary['status'],'reportSha256':rejected_summary['reportSha256'],'summary':'rejected-index/paired-summary.json','rows':24,'trials':21,'browserChecks':45}
(B/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
md=['# Complete paired results','','All rows retained. Paired change is the median of candidate/control trial ratios; JS ratio is the ratio of separate medians. Source controls are included in both independent VM runs.','']
for title,key in [('Ordinary VM — 21 trials','ordinary'),('First-use VM — 15 trials','firstUse')]:
 md+=['## '+title,'','| Case | Profile | Stage | Paired change | Faster pairs | Candidate / JS |','|---|---|---|---:|---:|---:|']
 for r in summary[key]['rows']:md.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedPercent']:+.3f}% | {r['candidateFasterPairs']}/{r['trials']} | {r['candidateOverJS']:.3f}× |")
 md+=['']
md+=['## Swift — combined native changes versus 0b32797','','| Case | Stage | Control ms | Candidate ms | Paired change | Faster pairs |','|---|---|---:|---:|---:|---:|']
for r in sw_rows:md.append(f"| {r['case']} | {r['stage']} | {r['controlMedianMs']:.6f} | {r['candidateMedianMs']:.6f} | {r['pairedPercent']:+.3f}% | {r['candidateFasterPairs']}/15 |")
(B/'RESULTS.md').write_text('\n'.join(md)+'\n')
files=[]
for path in sorted(B.rglob('*')):
 if not path.is_file() or path.name=='evidence-index.json' or '__pycache__' in path.parts:continue
 raw=path.read_bytes();record={'path':path.relative_to(B).as_posix(),'bytes':len(raw),'sha256':sha(raw)}
 if path.suffix=='.gz':
  decoded=gzip.decompress(raw);record.update(uncompressedBytes=len(decoded),uncompressedSha256=sha(decoded))
 files.append(record)
(B/'evidence-index.json').write_text(json.dumps({'schemaVersion':1,'method':'SHA256 of every other bundled file; deterministic gzip mtime=0; no generated timestamp.','files':files,'totalBytes':sum(x['bytes'] for x in files)},indent=2)+'\n')
print(f"Verified ordinary24 + first-use36 + Swift6 rows, six profiles,390 browser log and native/Swift/C correctness; indexed {len(files)} files.")
