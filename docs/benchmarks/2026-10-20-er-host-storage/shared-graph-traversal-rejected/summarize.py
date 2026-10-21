#!/usr/bin/env python3
"""Portable data-only verifier. No repository imports, network, builds or workloads."""
from pathlib import Path
import json,gzip,hashlib,statistics,math,re,collections
B=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
def raw(n):
 p=B/n;b=p.read_bytes();return gzip.decompress(b) if p.suffix=='.gz' else b
def load(n):return json.loads(raw(n))
def write(n,v):(B/n).write_text(json.dumps(v,indent=2,ensure_ascii=True)+'\n',encoding='ascii')
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
source=load('freeze/source-before.json.gz');f=load('freeze/package-freeze.json.gz');binding=load('freeze/commit-patch-binding.json.gz');m=load('design/manifest.json')
COMMIT='365ec6d0df050199f9efb93d23d9ce7d352c60fa'
PATCH='4ce4d35f82f294bfb84fc8e1a801e7064caec9757d680ba9d3e2d67c6f53dae4'
assert source['sourceCommit']==f['rustSourceCommit']==binding['baseCommit']==COMMIT
assert len(source['files'])==662 and binding['baseSourceHashes']==source['files'] and len(binding['swiftGitInputHashes'])==341
assert sha(raw('design/candidate.patch.gz'))==m['patchSha256']==f['patchSHA256']==binding['patchSha256']==PATCH
assert f['sameWasm'] and f['files']['candidate']['pkg/pele_bg.wasm']==f['files']['control']['pkg/pele_bg.wasm']=='3f6c40133130905f170ba582b3b9ef0c8718e795f2305e402200d5877563fc5b'
assert f['changed']==['graph-storage.js','graph.js','package.json'] and len(f['files']['control'])==106 and len(f['files']['candidate'])==107
assert f['files']==load('freeze/package-local-recheck.json.gz')['files']
assert len(binding['candidateSourceHashes'])==663
for entry in m['files']:
 assert source['files'].get(entry['path'])==entry['originalSha256']
 assert binding['candidateSourceHashes'][entry['path']]==entry['candidateSha256']==f['files']['candidate'][entry['path'].removeprefix('bindings/web/')]
rows={};groups={};orders={};arena_families={'sankey'}
for name,trials,warmup,total,cases,harness in [('comparison',21,1000,42,7,'compare-web-vm.mjs'),('first-use',15,1000,40,6,'compare-first-use-web-vm.mjs')]:
 d=load(name+'.json.gz');assert d['succeeded'] and d['diagnostic'] and len(d['rows'])==total and len(d['cases'])==cases
 assert d['runtimeHashes']==f['files'];assert d['harnessSHA256']==sha(raw('harness/'+harness));assert d['casesSHA256']==sha(raw('harness/cases.mjs'))
 assert d['oracleSHA256']=='d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d';assert d['settings']['trials']==trials and d['settings']['warmupMs']==warmup
 assert d['settings']['crossOriginIsolated'] and 0<d['settings']['observedClockResolutionMs']<=.01
 ids={c['id'] for c in d['cases']};expected=set()
 for c in d['cases']:
  assert c['checksum']>0 and c['sourceBytes']>0 and all(v['svgBytes']>0 for v in c['checks'])
  stages=['parse','render-model','render-source'] if name!='first-use' else ['render-source','parse-first-render','parse-first-edited-render']+(['parse-first-invoke','parse-first-callback'] if c['id'].split('/')[0] in ['er','gantt'] else [])
  expected.update((c['id'],p,s) for p in ['fixed','canvas'] for s in stages)
 assert {(r['case'],r['profile'],r['stage']) for r in d['rows']}==expected
 rows[name]=[]
 for r in d['rows']:
  assert r['counts']['candidate']==r['counts']['control'] and all(1<=x<=128 for x in r['counts'].values())
  for side,samples in r['samples'].items():
   assert len(samples)==trials and all(math.isfinite(x) and x>0 for x in samples);close(statistics.median(samples),r['medianMs'][side]);close(sorted(samples)[math.ceil(trials*.95)-1],r['p95BatchMeanMs'][side]);assert r['warmups'][side]['elapsedMs']>=warmup and r['warmups'][side]['operations']>=6
  for key,a,b in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(r[key],r['medianMs'][a]/r['medianMs'][b])
  ratios=[a/b for a,b in zip(r['samples']['candidate'],r['samples']['control'])];mid=statistics.median(ratios)
  rows[name].append({'case':r['case'],'profile':r['profile'],'stage':r['stage'],'pairedPercent':(mid-1)*100,'lowerPairs':sum(x<1 for x in ratios),'trials':trials,'controlMedianMs':r['medianMs']['control'],'candidateMedianMs':r['medianMs']['candidate'],'candidateOverJS':r['candidateOverJS'],'pairedRatioMAD':statistics.median(abs(x-mid) for x in ratios)})
 groups[name]={}
 for family in sorted(arena_families|{'sequence','er','flowchart','all'}):
  for stage in sorted({r['stage'] for r in rows[name]}):
   subset=[r for r in rows[name] if r['stage']==stage and (family=='all' or (family=='arena' and r['case'].split('/')[0] in arena_families) or r['case'].startswith(family+'/'))]
   if not subset:continue
   values=[r['pairedPercent'] for r in subset];groups[name][family+'/'+stage]={'rows':len(subset),'lowerRows':sum(v<0 for v in values),'medianOfRowPairedPercents':statistics.median(values),'minPairedPercent':min(values),'maxPairedPercent':max(values),'minCandidateOverJS':min(r['candidateOverJS'] for r in subset),'maxCandidateOverJS':max(r['candidateOverJS'] for r in subset),'adverseRows':[r for r in subset if r['pairedPercent']>0]}
 sequence=[];positions={x:collections.Counter() for x in ['js','control','candidate']}
 for i in range(trials):
  names=['js','control','candidate'];order=names[i%3:]+names[:i%3]
  if (i//3)%2:order.reverse()
  sequence.append(order)
  for x in names:positions[x][order.index(x)]+=1
 assert all(set(v.values())=={trials//3} for v in positions.values())
 orders[name]={'derivedFromHashedHarness':True,'orders':sequence,'controlBeforeCandidate':sum(x.index('control')<x.index('candidate') for x in sequence),'limitation':'Three-way positions balanced; two-side first order differs by one. Per-trial order not retained explicitly in report.'}
before=load('freeze/swift-before-correctness.json.gz');after=load('freeze/swift-after-correctness.json.gz');swift=load('swift/candidate-build-manifest.json.gz');control=load('swift/control-build-manifest.json.gz');api=load('validation/swift-api.json.gz')
assert before['current']==after['current'];assert before['manifest']==after['manifest']==swift==api['buildManifest'];assert len(swift['inputHashes'])==354
assert after['current']['sha256']==swift['sourceInputSha256']=='d4019492b901d9fa0f8b62075ab48463fd10597056b0265b8139a641a8d1bb58'
assert after['binarySha256']==swift['binarySha256']=='920faee3e167e654c06ec7953240cc5471bbafefd1164beca1b59516920fcf90'
assert after['nativeArchiveSha256']==swift['nativeHostArchiveSha256']==swift['inputHashes']['CPele/macos-arm64/libpele_ffi.a']=='134c97654cda1344b3588857539161843e584b4de96b47e9ff6eb0e2b1a4a463'
assert control['binarySha256']=='16900d32f60685e93d92a1e697650d3fb34e5e0f63be6ad7c5be9c8305fcfdbf'
assert all(swift['inputHashes'][k]==h for k,h in binding['swiftGitInputHashes'].items())
changed=[p for p in sorted(set(swift['inputHashes'])|set(control['inputHashes'])) if swift['inputHashes'].get(p)!=control['inputHashes'].get(p)]
for p,h in swift['inputHashes'].items():
 if p.startswith('root/') and p[5:] in source['files']:assert source['files'][p[5:]]==h
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only'
correct=after['correctness'];assert correct=={'swiftTests':16,'drawings':12481,'expectedErrors':16572,'budgetDifferences':0,'apiCases':21,'apiConditions':168,'cConsumers':3,'timing':False}
assert 'Executed 16 tests, with 0 failures' in raw('logs/swift-corpus.log.gz').decode();assert '12481 drawings, 16572 errors, 0 explicit budget differences' in raw('logs/swift-corpus.log.gz').decode()
assert len(load('validation/c-consumers.json'))==6 and all(x['exitCode']==0 for x in load('validation/correctness-commands.json'))
native=sum(int(x) for x in re.findall(r'test result: ok\. (\d+) passed; 0 failed',raw('logs/captured-write-native.log.gz').decode()));assert native==22
web=load('tests/index.json');assert (web['passed'],web['interrupted'],web['notRun'])==(147,1,269)
webLog=raw('logs/full-web.log.gz').decode();assert '147 passed (5.2m)' in webLog and '1 interrupted' in webLog and '269 did not run' in webLog
for report in web['reports']:assert sha(raw(report['path']))==report['sha256']
schedule=load('swift/attempt1/schedule.json.gz');assert schedule['succeeded'] is False and schedule['acceptedConditions']==['parseMutable','parse','render'] and schedule['unmeasuredConditions']==['lifetime'];assert schedule['retries']==0 and schedule['pooling'] is False and schedule['budgetMs']==180000 and schedule['elapsedWallMs']<180000
assert schedule['schedulerSha256']==sha(raw('harness/bounded-swift-timing.mjs'));assert schedule['comparatorSha256']==sha(raw('harness/swift/compare.mjs'));assert schedule['guardModuleSha256']==sha(raw('harness/swift/build.mjs'))
assert schedule['frozen']['sourceInputSha256']==swift['sourceInputSha256'] and schedule['frozen']['candidateBinarySha256']==swift['binarySha256'] and schedule['frozen']['controlBinarySha256']==control['binarySha256']
local=[];orderTotals=collections.Counter()
for stage,offset in [('parseMutable',0),('parse',1),('render',0)]:
 d=load('swift/attempt1/'+stage+'.json.gz');assert d['completed'] and d['revision']==COMMIT and len(d['rows'])==1
 assert d['comparatorSha256']==sha(raw('harness/swift/compare.mjs'))
 assert d['builds']['control']['manifest']==control and d['builds']['candidate']['manifest']==swift
 assert d['settings']['trials']==15 and d['settings']['warmupMsPerBinaryPerStage']==1000 and d['settings']['orderOffset']==offset
 r=d['rows'][0];assert r['case']=='sankey/medium' and r['stage']==stage and len(r['raw'])==15 and 1<=r['count']<=128
 for side in ['control','candidate']:assert r['warmup'][side]['elapsedNs']>=1e9
 for check in ['before','after']:
  for side in ['control','candidate']:
   assert r[check]['values'][side]['output']==d['cases'][0]['expected']
   assert all(output==d['cases'][0]['expected'] for output in d['cases'][0][check][side]['outputs'])
 ratios=[];values={'control':[],'candidate':[]}
 for i,t in enumerate(r['raw']):
  expected=['control','candidate'] if (i+offset)%2==0 else ['candidate','control'];assert t['order']==expected;orderTotals[expected[0]]+=1
  a=t['values']['candidate'];b=t['values']['control'];assert a['count']==b['count'] and a['output']==b['output'] and a['checksum']==b['checksum']
  if stage=='render':assert a['arenaBefore']==a['arenaAfter'] and b['arenaBefore']==b['arenaAfter']
  for side in values:values[side].append(t['values'][side]['elapsedNs']/t['values'][side]['count']/1e6)
  ratios.append(a['elapsedNs']/b['elapsedNs'])
 stats=r['statistics'];mid=statistics.median(ratios);close(mid,stats['pairedCandidateOverControl']['median']);assert sum(x<1 for x in ratios)==stats['candidateFasterPairs']
 for side in values:
  for a,b in zip(values[side],stats[side+'Ms']):close(a,b)
 local.append({'case':r['case'],'stage':stage,'trials':15,'pairedPercent':(mid-1)*100,'lowerPairs':sum(x<1 for x in ratios),'controlMedianMs':statistics.median(values['control']),'candidateMedianMs':statistics.median(values['candidate']),'pairedRatioMin':min(ratios),'pairedRatioMax':max(ratios)})
assert dict(orderTotals)=={'control':23,'candidate':22}
rejected=load('swift/attempt1/lifetime.json.gz');assert rejected['completed'] is False;assert 'Build/Pele test process active' in raw('swift/attempt1/lifetime.log.gz').decode();assert 'rustc' in raw('swift/attempt1/lifetime.log.gz').decode() and '45785' in raw('swift/attempt1/lifetime.log.gz').decode()
profile=load('profiles-1906/summary.json.gz');profileFreeze=load('profiles-1906/package-freeze.json.gz');profileBind=load('profiles-1906/commit-binding.json.gz')
assert profileBind['commit']==profileFreeze['sourceCommit']=='1906d683476b7cff1d76221a02a2591ce6d424b1'
assert len(profile['profiles'])==12 and profile['runtimeHashes']==profileFreeze['files']['candidate'] and profile['wasmSHA256']==profileFreeze['files']['candidate']['pkg/pele_bg.wasm']=='cee0b15b999c68b1317bdc70ffc90000c53d22f27c1387eae2413c243a198453'
assert profile['harnessSHA256']==sha(raw('harness/profile-web.mjs')) and profile['casesSHA256']==sha(raw('harness/cases.mjs'))
profileRows=[]
for path in sorted(B.glob('profiles-1906/*.cpuprofile.gz')):
 d=load(str(path.relative_to(B)));assert len(d['samples'])==len(d['timeDeltas']) and len(d['nodes'])>0
 profileRows.append({'path':str(path.relative_to(B)),'samples':len(d['samples']),'totalDeltaUs':sum(d['timeDeltas'])})
assert len(profileRows)==12
summary={'status':load('status.json'),'rows':rows,'groups':groups,'order':orders,'swiftCorrectness':correct,'nativeHelperIntegrationTests':native,'browserTests':{'passed':147,'interrupted':1,'notRun':269},'swiftTiming':{'accepted':local,'orderCounts':dict(orderTotals),'unmeasured':['lifetime'],'rejectedReportExcludedInFull':True,'attemptElapsedWallMs':schedule['elapsedWallMs']},'baseSourceInputs':662,'candidateSourceInputs':663,'packageFiles':{'control':106,'candidate':107},'changedSwiftInputs':changed,'profiles1906':{'sourceCommit':profileBind['commit'],'wasmSha256':profile['wasmSHA256'],'records':profileRows},'limitations':['Traversal-v1 performance rejected/provisional; revision2 not included','Web comparison isolates JS traversal with identical365 Rust/WASM, while Swift compares365 core versus1906; do not compound them','All82 VM conditions retained, including adverse retained/render/source/first-use rows','Three complete Swift conditions only; lifetime failed after9 raw pairs, all rejected samples excluded','Browser full suite stopped:147passed,1interrupted,269notrun; not417passed','Twelve profiles belong to older1906 cee0 WASM, not traversal candidate3f6c','Physical hashes are local frozen-copy checks; no independent remote rehash or reproducible-build claim','No release parity, absence-of-regression, or accepted traversal claim']}
write('summary.json',summary)
lines=['# Complete paired rows','','Negative means candidate lower. Separate windows/binaries stay separate; rejected samples are never pooled.','']
for name,rs in rows.items():
 lines+=['## '+name,'','| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |','|---|---|---|---:|---:|---:|']
 for r in rs:lines.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedPercent']:+.3f}% | {r['lowerPairs']}/{r['trials']} | {r['candidateOverJS']:.3f}x |")
 lines.append('')
lines+=['## Separate local Swift core comparison','','| Case | Stage | Paired change | Lower pairs | Control median ms | Candidate median ms |','|---|---|---:|---:|---:|---:|']
for r in local:lines.append(f"| {r['case']} | {r['stage']} | {r['pairedPercent']:+.4f}% | {r['lowerPairs']}/15 | {r['controlMedianMs']:.6f} | {r['candidateMedianMs']:.6f} |")
lines+=['','lifetime rejected in full by external rustc45785; unmeasured.','']
(B/'RESULTS.md').write_text('\n'.join(lines),encoding='ascii')
files=[{'path':str(p.relative_to(B)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(B.rglob('*')) if p.is_file() and p.name!='evidence-index.json'];write('evidence-index.json',{'schemaVersion':1,'files':files})
print(f'Verified42 VM +40 first-use rows,45 accepted Swift pairs; lifetime rejected in full. Same-WASM traversal-v1 rejected. Browser147passed/1interrupted/269notrun; older1906 profiles12 separately pinned. Indexed{len(files)} files.')
