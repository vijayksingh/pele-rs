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
source=load('freeze/source-before.json.gz');initial=load('freeze/source-before-initial.json.gz');f=load('freeze/package-freeze.json.gz');m=load('design/manifest.json')
assert len(source['files'])==f['sourceInputCount']==660 and f['sourceInputsUnchanged']
assert source['head']=='ba15a4af059a49a0ee766ae96e0fe939fc863f86';assert source['patchSHA256']==m['patchSha256']==sha(raw('design/candidate.patch.gz'))=='a9ec1aa610c97fea331e1e64cb647b9525d7ef2f614ca322e6057acde8038e0e'
assert source['files']==load('freeze/source-local-recheck.json.gz')['files']
for entry in m['files']:assert source['files'][entry['path']]==entry['candidateSha256']
repair_paths=[p for p in source['files'] if initial['files'].get(p)!=source['files'][p]];assert repair_paths==['crates/pele/tests/public_render_outcome_view.rs']
assert f['files']==load('freeze/package-local-recheck.json.gz')['hashes'] and all(len(v)==106 for v in f['files'].values())
assert f['files']['candidate']['full.js']==source['files']['bindings/web/full.js']
assert sorted(f['changed'])==['full.js','pkg/pele.d.ts','pkg/pele.js','pkg/pele_bg.wasm','pkg/pele_bg.wasm.d.ts']
rows={};groups={};orders={};arena_families={'flowchart','sequence','pie','sankey','gantt','railroad'}
for name,trials,warmup,total,cases,harness in [('comparison',9,500,126,21,'compare-web-vm.mjs'),('first-use',15,1000,50,7,'compare-first-use-web-vm.mjs'),('parse-controls-followup',21,1000,24,4,'compare-web-vm.mjs')]:
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
 for family in sorted(arena_families|{'er','all','arena'}):
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
before=load('freeze/swift-correctness-before.json.gz');after=load('freeze/swift-correctness-after.json.gz');swift=load('swift/candidate-build-manifest.json.gz');control=load('swift/control-build-manifest.json.gz');api=load('validation/swift-api.json.gz');correct=load('validation/correctness-summary.json')
assert before['current']==after['current'];assert before['manifest']==after['manifest']==swift==api['buildManifest'];assert len(swift['inputHashes'])==351
assert before['current']['sha256']==swift['sourceInputSha256']==correct['sourceInputSha256']=='e632dc000ed18bd395fe971a7029faba664278cc58b4e2b525eaa6effecf9e58'
assert after['binarySha256']==correct['candidateBinarySha256']==swift['binarySha256']=='2e1491d5b80da5d64b7f042abe934f47d662901d9d8037eb1646a0d7a03cff9a'
assert after['targetSha256']==swift['nativeHostArchiveSha256']==swift['inputHashes']['CPele/macos-arm64/libpele_ffi.a'];assert correct['controlBinarySha256']==control['binarySha256']=='368ab53bad3b3fdd6daa82cce6a53932a8eed8ea922d5c7683370b4e3acbc0cf'
changed=[p for p in swift['inputHashes'] if swift['inputHashes'][p]!=control['inputHashes'][p]];assert len(changed)==7 and sum(p.startswith('root/') for p in changed)==6
for p,h in swift['inputHashes'].items():
 if p.startswith('root/') and p[5:] in source['files']:assert source['files'][p[5:]]==h
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only'
assert correct['swiftSelectedTests']==16 and correct['cConsumers']==3 and not correct['elapsedTimingPerformed'];assert 'Executed 16 tests, with 0 failures' in raw('logs/swift-corpus.log.gz').decode();assert '12481 drawings, 16572 errors, 0 explicit budget differences' in raw('logs/swift-corpus.log.gz').decode()
assert len(load('validation/c-consumers.json'))==6 and all(x['exitCode']==0 for x in load('validation/correctness-commands.json'))
assert '12 passed (26.4s)' in raw('logs/focused-web.log.gz').decode();native_count=sum(int(x) for x in re.findall(r'test result: ok\. (\d+) passed; 0 failed',raw('logs/native-tests-repaired.log.gz').decode()));assert native_count==41
assert 'error[E0599]' in raw('logs/native-tests.log.gz').decode()
reports=load('tests/index.json');assert len(reports)==12
for entry in reports:
 assert sha(raw(entry['path']))==entry['sha256'];d=load(entry['path']);name=Path(entry['path']).name
 if name=='arena-packet-all-families.json.gz':assert len(d)==66 and all(x['legacy']==x['packet'] for x in d)
 elif name=='arena-packet-callbacks.json.gz':assert len(d)==20 and all(x['legacy']==x['packet'] for x in d)
 elif name=='arena-packet-factory-errors.json.gz':assert len(d)==6 and all(x['fired'] and x['threw'] and x['original'] and x['restored'] and x['recovery']>0 for x in d)
 elif name=='arena-packet-combined-budget.json.gz':
  for mode in [False,True]:
   legacy=next(x for x in d if x['raw']==mode and x['mode']=='legacy');packet=next(x for x in d if x['raw']==mode and x['mode']=='packet');assert dict(packet,mode='legacy')==legacy;assert packet['expectedMetadata']<64*1024*1024<packet['expectedFrame']
 else:raise AssertionError(name)
binding=load('freeze/commit-binding.json.gz');assert binding['commit']=='9ae8142d99408ff1743546bf3736af707533d267';assert binding['sourceHashes']==source['files'];assert len(binding['swiftGitInputHashes'])==338
assert all(swift['inputHashes'][k]==h for k,h in binding['swiftGitInputHashes'].items())
profiles=load('profiles/summary.json.gz');assert len(profiles['profiles'])==18;assert profiles['runtimeHashes']==f['files']['candidate'];assert profiles['wasmSHA256']==f['files']['candidate']['pkg/pele_bg.wasm'];assert profiles['harnessSHA256']==sha(raw('harness/profile-web.mjs'));assert profiles['casesSHA256']==sha(raw('harness/cases.mjs'))
for path in B.glob('profiles/*.cpuprofile.gz'):
 d=load(str(path.relative_to(B)));assert len(d['samples'])==len(d['timeDeltas']) and len(d['nodes'])>0
full=load('tests/full-index.json');assert full['passedTests']==417 and full['explicitReports']==len(full['reports'])==78
assert '417 passed (18.4m)' in raw('logs/full-web.log.gz').decode();assert {r['engine'] for r in full['reports']}=={'chromium','firefox','webkit'}
for entry in full['reports']:assert sha(raw(entry['path']))==entry['uncompressedSha256']
blocked=load('swift/bounded-attempt1-schedule.json.gz');assert blocked['succeeded'] is False and blocked['conditions']==[] and blocked['acceptedConditions']==[]
assert blocked['unmeasuredConditions']==['render','lifetime','parseMutable','parse'] and blocked['retries']==0 and blocked['pooling'] is False
assert 'Build/Pele test process active' in blocked['error'] and blocked['elapsedWallMs']<1000 and blocked['budgetMs']==180000
assert blocked['fullBrowserLogSha256']==sha(raw('logs/full-web.log.gz'));assert blocked['schedulerSha256']==sha(raw('harness/bounded-swift-timing.mjs'));assert blocked['comparatorSha256']==sha(raw('harness/swift/compare.mjs'));assert blocked['guardModuleSha256']==sha(raw('harness/swift/build.mjs'))
assert 'frozen' not in blocked # Guard blocked before timing-side freshness checks; do not claim they ran.
summary={'status':load('status.json'),'rows':rows,'groups':groups,'order':orders,'swiftCorrectness':correct,'nativeTests':native_count,'focusedBrowserTests':12,'fullBrowserTests':417,'fullBrowserReports':78,'swiftTimingAttempt':{'status':'blocked before any condition started','acceptedConditions':0,'unmeasuredConditions':blocked['unmeasuredConditions'],'elapsedWallMs':blocked['elapsedWallMs']},'sourceInputs':660,'packageFilesPerSide':106,'changedSwiftInputs':changed,'limitations':['Immutable660-input mapping9ae8142 established; matching hashes are not a reproducible rebuild','Full417 suite passed; separate24-row performance followup retained, never pooled','Local Swift latency unmeasured; sole bounded timing attempt blocked before any condition. Swift16/C3/API168 remains correctness evidence only','Shared VM browser/GC/cache pressure and serial warmup; controls can regress','ER host backend is a negative control; its first-use adverse rows remain explicit','No independent remote physical rehash or reproducible-build claim','No full performance gate or release parity claim']}
write('summary.json',summary)
lines=['# Complete paired rows','','All raw rows retained. Negative means candidate lower; reports are not pooled.','']
for name,rs in rows.items():
 lines += ['## '+name,'','| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |','|---|---|---|---:|---:|---:|']
 for r in rs:lines.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedPercent']:+.3f}% | {r['lowerPairs']}/{r['trials']} | {r['candidateOverJS']:.3f}x |")
 lines.append('')
(B/'RESULTS.md').write_text('\n'.join(lines),encoding='ascii')
files=[{'path':str(p.relative_to(B)),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(B.rglob('*')) if p.is_file() and p.name!='evidence-index.json'];write('evidence-index.json',{'schemaVersion':1,'files':files})
print(f'Verified126 VM +50 first-use +24 followup rows,660 frozen candidate inputs,106 package files/side,41 native/12 focused+417 full browser/Swift16/C3/API168; indexed{len(files)} files. Commit9ae8142 bound;18 profiles retained. Full417 passes; sole Swift attempt blocked, no latency samples.')
