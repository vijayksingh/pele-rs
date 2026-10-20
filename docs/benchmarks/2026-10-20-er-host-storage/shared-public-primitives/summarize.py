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
source=load('freeze/source-before.json.gz');f=load('freeze/package-freeze.json.gz');binding=load('freeze/commit-binding.json.gz')
COMMIT='1906d683476b7cff1d76221a02a2591ce6d424b1'
assert source['sourceCommit']==f['sourceCommit']==binding['commit']==COMMIT
assert len(source['files'])==f['sourceInputCount']==662 and f['sourceInputsUnchanged']
assert binding['sourceHashes']==source['files'] and len(binding['swiftGitInputHashes'])==340
assert f['files']==load('freeze/package-local-recheck.json.gz')['hashes'] and all(len(v)==106 for v in f['files'].values())
rows={};groups={};orders={};arena_families={'sankey'}
for name,trials,warmup,total,cases,harness in [('comparison',21,1000,30,5,'compare-web-vm.mjs'),('first-use',15,1000,24,4,'compare-first-use-web-vm.mjs')]:
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
 for family in sorted(arena_families|{'sequence','er','all'}):
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
before=load('freeze/swift-before-correctness.json.gz');after=load('freeze/swift-after-correctness.json.gz');swift=load('swift/candidate-build-manifest.json.gz');control=load('swift/control-build-manifest.json.gz');api=load('validation/swift-api.json.gz');correct=load('validation/correctness-summary.json')
assert before['current']==after['current'];assert before['manifest']==after['manifest']==swift==api['buildManifest'];assert len(swift['inputHashes'])==353
assert after['current']['sha256']==swift['sourceInputSha256']==correct['sourceInputSha256']=='d898df768e0bab3d3f2f21848a79d1ac9b32ade7cf6756b4643c14c4824ce7fc'
assert after['binarySha256']==correct['candidateBinarySha256']==swift['binarySha256']=='16900d32f60685e93d92a1e697650d3fb34e5e0f63be6ad7c5be9c8305fcfdbf'
assert after['nativeArchiveSha256']==swift['nativeHostArchiveSha256']==swift['inputHashes']['CPele/macos-arm64/libpele_ffi.a']=='6cef08bdbbd45cd21db6d302b606968f2bd70c72a3fc5cf687a1c8bdb15b2fe1'
assert correct['controlBinarySha256']==control['binarySha256']=='2e1491d5b80da5d64b7f042abe934f47d662901d9d8037eb1646a0d7a03cff9a'
assert all(swift['inputHashes'][k]==h for k,h in binding['swiftGitInputHashes'].items())
changed=[p for p in sorted(set(swift['inputHashes'])|set(control['inputHashes'])) if swift['inputHashes'].get(p)!=control['inputHashes'].get(p)]
for p,h in swift['inputHashes'].items():
 if p.startswith('root/') and p[5:] in source['files']:assert source['files'][p[5:]]==h
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only'
assert correct['correctness']=={'swiftTests':16,'drawings':12481,'expectedErrors':16572,'budgetDifferences':0,'apiCases':21,'apiConditions':168,'cConsumers':3,'timing':False}
assert 'Executed 16 tests, with 0 failures' in raw('logs/swift-corpus.log.gz').decode();assert '12481 drawings, 16572 errors, 0 explicit budget differences' in raw('logs/swift-corpus.log.gz').decode()
assert len(load('validation/c-consumers.json'))==6 and all(x['exitCode']==0 for x in load('validation/correctness-commands.json'))
native={}
for name,count in [('shared-construction-native',18),('sankey-captured-projection-native-repaired',20),('sankey-captured-projection-before-repaired',10)]:
 text=raw('logs/'+name+'.log.gz').decode();native[name]=sum(int(x) for x in re.findall(r'test result: ok\. (\d+) passed; 0 failed',text));assert native[name]==count
assert '9 passed; 1 failed' in raw('logs/sankey-captured-projection-before.log.gz').decode()
web=load('freeze/web-validation.json.gz');assert web['sourceCommit']==COMMIT and web['sourceInputCount']==662 and web['sourceInputsUnchanged'] and web['focusedBrowserTests']==24
assert sum(sum(int(x) for x in re.findall(r'(\d+) passed \(',raw('logs/'+name+'.gz').decode())) for name in web['focusedFiles'])==24
for side in ['control','candidate']:assert web['sizes'][side]['sha256']==f['files'][side]['pkg/pele_bg.wasm']
schedule=load('swift/attempt1/schedule.json.gz');assert schedule['succeeded'] is False and schedule['acceptedConditions']==['render','lifetime'] and schedule['unmeasuredConditions']==['parseMutable','parse'];assert schedule['retries']==0 and schedule['pooling'] is False and schedule['budgetMs']==180000 and schedule['elapsedWallMs']<180000
assert schedule['schedulerSha256']==sha(raw('harness/bounded-swift-timing.mjs'));assert schedule['comparatorSha256']==sha(raw('harness/swift/compare.mjs'));assert schedule['guardModuleSha256']==sha(raw('harness/swift/build.mjs'))
assert schedule['frozen']['sourceInputSha256']==swift['sourceInputSha256'] and schedule['frozen']['candidateBinarySha256']==swift['binarySha256'] and schedule['frozen']['controlBinarySha256']==control['binarySha256']
local=[];orderTotals=collections.Counter()
for stage,offset in [('render',0),('lifetime',1)]:
 d=load('swift/attempt1/'+stage+'.json.gz');assert d['completed'] and d['revision']==COMMIT and d['worktree']==[] and len(d['rows'])==1
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
assert dict(orderTotals)=={'control':15,'candidate':15}
rejected=load('swift/attempt1/parseMutable.json.gz');assert rejected['completed'] is False;assert 'Build/Pele test process active' in raw('swift/attempt1/parseMutable.log.gz').decode();assert 'swift-build' in raw('swift/attempt1/parseMutable.log.gz').decode()
summary={'status':load('status.json'),'rows':rows,'groups':groups,'order':orders,'swiftCorrectness':correct,'nativeTests':native,'focusedBrowserTests':24,'swiftTiming':{'accepted':local,'orderCounts':dict(orderTotals),'unmeasured':['parseMutable','parse'],'rejectedReportExcludedInFull':True,'attemptElapsedWallMs':schedule['elapsedWallMs']},'sourceInputs':662,'packageFilesPerSide':106,'changedSwiftInputs':changed,'wasmSize':web['sizes'],'limitations':['Combined shared-construction48a29aa and captured-projection1906d68 versus arena9ae8142; not isolated projection-only','All54 VM rows retained, including all6 adverse Sankey parse and6 adverse retained rows; no preparation-speed claim','Local30 accepted Swift pairs only; parseMutable rejected in full, parse not attempted; no pooling/retry','Global compiler guard passed complete conditions, but external app load remains; not an idle-host claim','Current focused browser24 only; previous full417 belongs to9ae8142','Local frozen package physical hashes and Git source mapping do not prove reproducible build or independent remote physical rehash','No full performance gate, cross-host ratio arithmetic, or release parity claim']}
write('summary.json',summary)
lines=['# Complete paired rows','','Negative means candidate lower. Reports and rejected samples are never pooled.','']
for name,rs in rows.items():
 lines+=['## '+name,'','| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |','|---|---|---|---:|---:|---:|']
 for r in rs:lines.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedPercent']:+.3f}% | {r['lowerPairs']}/{r['trials']} | {r['candidateOverJS']:.3f}x |")
 lines.append('')
lines+=['## Accepted local Swift conditions','','| Case | Stage | Paired change | Lower pairs | Control median ms | Candidate median ms |','|---|---|---:|---:|---:|---:|']
for r in local:lines.append(f"| {r['case']} | {r['stage']} | {r['pairedPercent']:+.4f}% | {r['lowerPairs']}/15 | {r['controlMedianMs']:.6f} | {r['candidateMedianMs']:.6f} |")
lines+=['','parseMutable rejected in full by external swift-build; parse unmeasured.','']
(B/'RESULTS.md').write_text('\n'.join(lines),encoding='ascii')
files=[{'path':str(p.relative_to(B)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(B.rglob('*')) if p.is_file() and p.name!='evidence-index.json'];write('evidence-index.json',{'schemaVersion':1,'files':files})
print(f'Verified30 VM +24 first-use rows,30 accepted Swift pairs; rejected parseMutable excluded.662 source /353 Swift inputs bound1906d68;18+20 native,baseline10,focused24,Swift16/C3/API168. Indexed{len(files)} files.')
