#!/usr/bin/env python3
"""Portable data-only verification: no repository imports, network, builds or workloads."""
from pathlib import Path
import json,gzip,hashlib,math,statistics,re,collections
B=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def raw(n):
 p=B/n;b=p.read_bytes();return gzip.decompress(b) if p.suffix=='.gz' else b
def load(n):return json.loads(raw(n))
def write(n,v):(B/n).write_text(json.dumps(v,indent=2,ensure_ascii=True)+'\n',encoding='ascii')
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
f=load('freeze/package-freeze.json.gz');proof=load('freeze/commit-binding.json.gz');source=load('freeze/source-before.json.gz')
assert f['sourceInputsUnchanged'] and f['sourceInputCount']==657==len(source['files'])
assert proof['commit']=='b19f38d6368d1ea8183004d24c58af5828b3f577' and proof['webSourceHashes']==source['files']
assert proof['nativeGitAggregate']==proof['nativeManifestSourceAggregate']==load('native/candidate-build-manifest.json')['sourceInputSha256']
assert len(proof['nativeInputHashes'])==277 and len(proof['swiftGitInputHashes'])==338
assert proof['additionalIntegrationAndDesignHashes']['crates/pele/tests/public_sankey_owned_parse.rs']
assert f['files']==load('freeze/local-package-recheck.json.gz')['hashes'] and all(len(v)==106 for v in f['files'].values())
rows={};groups={};orders={}
for name,trials,harness in [('comparison',21,'compare-web-vm.mjs'),('first-use',15,'compare-first-use-web-vm.mjs')]:
 d=load(name+'.json.gz');assert d['succeeded'] and d['diagnostic'] and len(d['rows'])==24 and len(d['cases'])==4
 assert d['runtimeHashes']==f['files'];assert d['harnessSHA256']==sha(raw('harness/'+harness));assert d['casesSHA256']==sha(raw('harness/cases.mjs'))
 assert d['oracleSHA256']=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d'
 assert d['settings']['trials']==trials and d['settings']['warmupMs']==1000 and d['settings']['crossOriginIsolated']
 assert 0<d['settings']['observedClockResolutionMs']<=.01
 assert {c['id'] for c in d['cases']}=={'sankey/small','sankey/medium','sankey/large','sequence/medium'}
 for c in d['cases']:assert c['checksum']>0 and c['sourceBytes']>0 and all(v['svgBytes']>0 for v in c['checks'])
 stages={'parse','render-model','render-source'} if name=='comparison' else {'render-source','parse-first-render','parse-first-edited-render'}
 assert {(r['case'],r['profile'],r['stage']) for r in d['rows']}=={(c['id'],p,s) for c in d['cases'] for p in ['fixed','canvas'] for s in stages}
 rows[name]=[]
 for r in d['rows']:
  assert r['counts']['control']==r['counts']['candidate'] and all(1<=n<=128 for n in r['counts'].values())
  for side,samples in r['samples'].items():
   assert len(samples)==trials and all(math.isfinite(x) and x>0 for x in samples);close(statistics.median(samples),r['medianMs'][side]);close(sorted(samples)[math.ceil(trials*.95)-1],r['p95BatchMeanMs'][side]);assert r['warmups'][side]['elapsedMs']>=1000 and r['warmups'][side]['operations']>=6
  for key,a,b in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[key],r['medianMs'][a]/r['medianMs'][b])
  ratios=[c/b for c,b in zip(r['samples']['candidate'],r['samples']['control'])];m=statistics.median(ratios)
  rows[name].append(dict(case=r['case'],profile=r['profile'],stage=r['stage'],trials=trials,pairedPercent=100*(m-1),lowerPairs=sum(x<1 for x in ratios),controlMedianMs=r['medianMs']['control'],candidateMedianMs=r['medianMs']['candidate'],candidateOverJS=r['candidateOverJS'],pairedRatioMAD=statistics.median(abs(x-m) for x in ratios)))
 groups[name]={}
 for family in ['sankey','sequence']:
  for stage in sorted(stages):
   selected=[r for r in rows[name] if r['case'].startswith(family+'/') and r['stage']==stage];values=[r['pairedPercent'] for r in selected]
   groups[name][family+'/'+stage]=dict(rows=len(selected),pairedLowerRows=sum(x<0 for x in values),medianPairedPercent=statistics.median(values),minPairedPercent=min(values),maxPairedPercent=max(values),adverseRows=[r for r in selected if r['pairedPercent']>0])
 sequence=[]
 for i in range(trials):
  names=['js','control','candidate'];order=names[i%3:]+names[:i%3]
  if (i//3)%2:order.reverse()
  sequence.append(order)
 orders[name]={'derivedFromFrozenHarness':True,'trials':sequence,'controlBeforeCandidate':sum(o.index('control')<o.index('candidate') for o in sequence),'note':'Raw report does not store per-trial orders; derived from hashed harness. Three-way positions balanced, two-side first order differs by one.'}
alloc=[]
for count in [1,16]:
 d=load(f'native/allocations-{count}.json.gz');assert d['succeeded'] and d['mode']=='native-allocations-only' and not d['rows'] and d['settings']['fixedBatchCount']==count and d['settings']['timingRequested'] is False
 assert len(d['allocationRows'])==8;assert d['harnessSHA256']==sha(raw('harness/compare-native.mjs')) and d['casesSHA256']==sha(raw('harness/cases.mjs'))
 for side in ['control','candidate']:
  m=load(f'native/{side}-build-manifest.json');assert d['artifacts'][side]['manifest']==m;assert d['artifacts'][side]['binarySHA256']==m['binaries']['pele-perf'];assert d['artifacts'][side]['allocationSHA256']==m['binaries']['pele-perf-alloc']
 for r in d['allocationRows']:
  assert r['count']==count and r['control']['checksum']==r['candidate']['checksum'];assert r['control']['liveBytesDelta']==r['candidate']['liveBytesDelta']==0
  alloc.append(dict(r,changes={k:r['candidate'][k]-r['control'][k] for k in ['allocations','reallocations','requestedBytes','freedBytes','peakAdditionalRequestedBytes','liveBytesDelta']}))
before=load('freeze/swift-correctness-before.json.gz');after=load('freeze/swift-correctness-after.json.gz');m=load('swift/candidate-build-manifest.json.gz');api=load('validation/swift-api.json.gz');correct=load('validation/correctness-summary.json')
assert before['current']==after['current'];assert before['manifest']==after['manifest']==m==api['buildManifest'];assert len(m['inputHashes'])==351
assert all(m['inputHashes'][k]==v for k,v in proof['swiftGitInputHashes'].items());assert m['sourceInputSha256']==correct['sourceInputSha256']==before['current']['sha256']
assert after['binarySha256']==m['binarySha256']==correct['candidateBinarySha256'];assert after['targetSha256']==m['nativeHostArchiveSha256']==m['inputHashes']['CPele/macos-arm64/libpele_ffi.a']
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only';assert correct['swiftSelectedTests']==16 and correct['cConsumers']==3
assert 'Executed 16 tests, with 0 failures' in raw('logs/swift-corpus.log.gz').decode();assert '12481 drawings, 16572 errors, 0 explicit budget differences' in raw('logs/swift-corpus.log.gz').decode()
assert len(load('validation/c-consumers.json'))==6 and all(r['exitCode']==0 for r in load('validation/correctness-commands.json'))
assert '48 passed (34.6s)' in raw('logs/focused-web.log.gz').decode() and '3 passed (27.3s)' in raw('logs/sankey-web.log.gz').decode()
native_counts=[int(x) for x in re.findall(r'test result: ok\. (\d+) passed; 0 failed',raw('logs/native-tests.log.gz').decode())];assert sum(native_counts)==32
rejection=raw('logs/native-timing.log.gz').decode();assert 'Build/Pele test process active' in rejection and '42330' in rejection and '42672' in rejection
for e in load('tests/index.json'):assert sha(raw(e['path']))==e['sha256']
p=load('profile/summary.json.gz');assert len(p['profiles'])==6 and p['runtimeHashes']==f['files']['candidate'] and p['wasmSHA256']==f['files']['candidate']['pkg/pele_bg.wasm'];assert p['harnessSHA256']==sha(raw('harness/profile-web.mjs'));assert p['casesSHA256']==sha(raw('harness/cases.mjs'))
for path in B.glob('profile/*.cpuprofile.gz'):
 d=load(str(path.relative_to(B)));assert len(d['samples'])==len(d['timeDeltas']) and len(d['nodes'])>0
committed=load('design/committed-manifest.json');assert committed['commit']==proof['commit'];assert sha(raw('design/source-changes.patch.gz'))==committed['patchSHA256']
for path,h in committed['files'].items():assert h==proof['webSourceHashes'].get(path,proof['additionalIntegrationAndDesignHashes'].get(path))
schedule=load('native/condition-timing-schedule.json.gz');assert schedule['strictGuardUnchanged'] and schedule['independentCompleteConditionWindows'] and schedule['noPoolingAcrossInterruptedWindows'];assert schedule['error']=='Error: Eight minute schedule exhausted'
accepted=[r for r in schedule['conditions'] if r.get('succeeded')];assert len(accepted)==2
assert {r['stage'] for r in accepted}=={'parse-public','parse'}
rejected=[]
for condition in schedule['conditions']:
 for a in condition['attempts']:
  log='logs/'+Path(a['logPath']).name+'.gz';text=raw(log).decode()
  if a['exitCode']!=0:assert 'Build/Pele test process active' in text;rejected.append({'condition':condition['name'],'attempt':a['attempt'],'log':log})
assert len(rejected)==5
native_timing=[]
for entry in accepted:
 d=load('native/accepted/'+Path(entry['output']).name+'.gz');assert d['succeeded'] and len(d['rows'])==1 and len(d['cases'])==1
 assert d['settings']['trials']==15 and d['settings']['warmupMs']==1000 and d['settings']['fixedBatchCount']==16
 assert d['settings']['stages']==[entry['stage']] and d['settings']['selectedCases']==['sankey/medium']
 assert d['harnessSHA256']==sha(raw('harness/compare-native.mjs')) and d['casesSHA256']==sha(raw('harness/cases.mjs'))
 assert d['oracleSHA256']=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d'
 for side in ['control','candidate']:
  m=load(f'native/{side}-build-manifest.json');assert d['artifacts'][side]['manifest']==m and d['artifacts'][side]['binarySHA256']==m['binaries']['pele-perf']
 c=d['cases'][0];assert sha(c['source'].encode())==c['sourceSHA256'];assert c['jsRetainedBeforeVerified'] and c['output']['svgBytes']>0
 r=d['rows'][0];assert r['jsRetainedPostBatchVerified'] and r['case']=='sankey/medium' and r['stage']==entry['stage'];assert r['counts']=={'js':16,'control':16,'candidate':16}
 for side in ['js','control','candidate']:
  assert len(r['samples'][side])==len(r['raw'][side])==15;assert r['warmups'][side]['elapsedNs']>=1_000_000_000
  for sample,measured in zip(r['samples'][side],r['raw'][side]):
   assert measured['count']==16 and measured['checksum']==96;close(sample,measured['elapsedNs']/16/1e6)
  close(statistics.median(r['samples'][side]),r['medians'][side])
 close(r['candidateOverControl'],r['medians']['candidate']/r['medians']['control']);close(r['candidateOverJS'],r['medians']['candidate']/r['medians']['js'])
 ratios=[a/b for a,b in zip(r['samples']['candidate'],r['samples']['control'])]
 native_timing.append({'case':r['case'],'stage':r['stage'],'pairedPercent':(statistics.median(ratios)-1)*100,'lowerPairs':sum(x<1 for x in ratios),'trials':15,'medians':r['medians'],'candidateOverJS':r['candidateOverJS'],'host':d['host'],'acceptedAttempt':entry['attempts'][-1]['attempt'],'sampleScope':'Separate complete condition; no rejected samples pooled'})
assert next(x for x in native_timing if x['stage']=='parse-public')['lowerPairs']==15
assert next(x for x in native_timing if x['stage']=='parse')['lowerPairs']==5
unmeasured=[{'runtime':'native','case':c,'stage':s} for c in ['sankey/medium','sequence/medium'] for s in ['parse-public','parse','render-source','render-retained'] if not(c=='sankey/medium' and s in ['parse-public','parse'])]+[{'runtime':'swift','case':c,'stage':s} for c in ['sankey/medium','sequence/medium'] for s in ['parseMutable','lifetime','parse']]
assert len(unmeasured)==12

summary={'sourceCommit':proof['commit'],'diagnosticNotReleaseGate':True,'nativeSwiftLatency':'Partial: two accepted native conditions; six other native and six Swift conditions unmeasured','nativeTimingRows':native_timing,'unmeasuredConditions':unmeasured,'rejectedConditionAttempts':rejected,'rows':rows,'groups':groups,'orders':orders,'allocationRows':alloc,'correctness':correct,'nativeTests':sum(native_counts),'browserTests':51,'sourceBinding':{'webInputs':657,'swiftInputs':351,'nativeInputs':277,'nativeAggregate':proof['nativeGitAggregate'],'nativeRecordedHead':proof['nativeManifestRevision'],'actualSourceCommit':proof['commit']},'limitations':['No release parity claim; historical full performance gate still fails all126 conditions','VM shared browser/GC/cache and serial warmup; no causal claim for controls','Profiles are candidate-only later samples, not paired before/after cost attribution','Allocation counters count requested bytes in scoped operations, not RSS or process footprint','First-use Sankey fixture covers unchanged/edited render, not a Sankey method/callback timing','Package physical recheck is local; report records VM input hashes but no additional independent remote rehash is claimed']}
write('summary.json',summary)
lines=['# Complete paired rows','', 'Derived from raw samples; negative means candidate lower. No pooling across reports.','']
for name,rs in rows.items():
 lines += ['## '+name,'','| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |','|---|---|---|---:|---:|---:|']
 for r in rs:lines.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedPercent']:+.3f}% | {r['lowerPairs']}/{r['trials']} | {r['candidateOverJS']:.3f}x |")
 lines.append('')
lines += ['## Partial local native timing','','| Case | Stage | Paired change | Lower pairs | Candidate median ms |','|---|---|---:|---:|---:|']
for r in native_timing:lines.append(f"| {r['case']} | {r['stage']} | {r['pairedPercent']:+.4f}% | {r['lowerPairs']}/15 | {r['medians']['candidate']:.9f} |")
lines += ['', 'Six other native and all six Swift conditions remain unmeasured. Rejected windows are excluded, not pooled. Typed parse omits full public projection; local Node results cannot be subtracted from VM browser results.', '']
(B/'RESULTS.md').write_text('\n'.join(lines),encoding='ascii')
files=[{'path':str(p.relative_to(B)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(B.rglob('*')) if p.is_file() and p.name!='evidence-index.json'];write('evidence-index.json',{'schemaVersion':1,'files':files})
print(f'Verified 24 VM + 24 first-use rows,16 allocation rows,657 Git-bound web inputs,277 native inputs,351 Swift inputs,32 native/51 browser/16 Swift/C3/API168; 2 accepted native windows /12 unmeasured local conditions; indexed {len(files)} files')
