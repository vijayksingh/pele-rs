"""Read-only saved-evidence audit; no consumer, browser, build, or benchmark execution."""
import hashlib,json,math,pathlib,statistics
BASE=pathlib.Path(__file__).resolve().parent
ROOT=BASE.parents[3]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
q=lambda a,p:sorted(a)[max(0,math.ceil(len(a)*p)-1)]
def eq(a, b):
    if isinstance(a, float) or isinstance(b, float):
        assert math.isclose(a, b, rel_tol=1e-12, abs_tol=1e-12), (a,b)
    elif isinstance(a, dict):
        assert a.keys()==b.keys(), (a.keys(),b.keys())
        for k in a: eq(a[k], b[k])
    elif isinstance(a, list):
        assert len(a)==len(b)
        for x,y in zip(a,b): eq(x,y)
    else: assert a==b, (a,b)


def package(root):return {str(p.relative_to(root)):sha(p) for p in root.rglob('*') if p.is_file()}
output={}
PUBLIC=ROOT/'docs/benchmarks/2026-10-20-normalized-er'
freeze=json.loads((BASE/'audit-source-freeze.json').read_text())
assert freeze['exactInputMap'] and freeze['inputCount']==347
eq(freeze['liveInputAggregate'],'a5515fc0b18fe01dcb7de6c36844fb6a4db9340b3874402d64c1c07f5afe38a5')
eq(freeze['liveInputAggregate'],freeze['manifestInputAggregate'])
eq(freeze['candidateBinarySHA256'],freeze['expectedCandidateBinarySHA256'])
eq(freeze['controlBinarySHA256'],freeze['expectedControlBinarySHA256'])
eq(set(freeze['changedInputPaths']),{'root/crates/pele/src/compat/engine.rs','CPele/macos-arm64/libpele_ffi.a'})
output['sourceFreeze']=freeze
output['sourceFreezeSHA256']=sha(BASE/'audit-source-freeze.json')
events=json.loads((BASE/'single-windows.json').read_text());eq(len(events),10)
eq(sum(x['complete'] for x in events),6)
output['excludedWindows']=[]
for e in events:
    eq(e['code'],0 if e['complete'] else 1)
    if e['complete']:continue
    log=ROOT/e['log'];assert 'Build/Pele test process active; defer this operation' in log.read_text()
    p=ROOT/e['output']
    if p.exists():
        d=json.loads(p.read_text());assert not d['completed']
        assert 'Build/Pele test process active; defer this operation' in d['failure']
        assert all(not r['raw'] for r in d['rows'])
    output['excludedWindows'].append({'stage':e['stage'],'case':e['workload'],'attempt':e['attempt'],'reportExists':p.exists(),'logSHA256':sha(log),'measuredPairsIncluded':0})
eq(len(output['excludedWindows']),4)
def stats(v):
    m=statistics.median(v)
    return dict(median=m,mad=statistics.median([abs(x-m) for x in v]),min=min(v),max=max(v),sampleP95=sorted(v)[math.ceil(len(v)*.95)-1])


comparator=sha(ROOT/'scripts/perf/swift/compare.mjs')
output['acceptedSwift']={'reports':[],'orderCounts':{'control':0,'candidate':0}}
for event in events:
    if not event['complete']: continue
    path=ROOT/event['output']
    d=json.loads(path.read_text())
    assert d['completed'] and d['mode']=='matched-local-swift-api-timing'
    eq(d['comparatorSha256'],comparator)
    eq(sha(ROOT/'node_modules/pele/dist/pele.js'),d['oracleSha256'])
    eq(d['settings']['trials'],15); eq(d['settings']['warmupMsPerBinaryPerStage'],1000);eq(d['settings']['warmupMaximumBatch'],128)
    for side,b in d['builds'].items():
        eq(b['binarySha256'],b['manifest']['binarySha256'])
        if side=='control': eq(sha(pathlib.Path(b['binary'])),b['binarySha256'])
        else: eq(b['binarySha256'],freeze['candidateBinarySHA256'])
        manifest_path=pathlib.Path(b['manifestPath']) if side=='control' else BASE/'audit-candidate-build-manifest.json'
        eq(sha(manifest_path),b['manifestSha256']);eq(json.loads(manifest_path.read_text()),b['manifest'])
    a,b=(d['builds'][s]['manifest'] for s in ['control','candidate'])
    for key in ['architecture','swift','sdk']:eq(a[key],b[key])
    for key in a['inputHashes']:
        if key.startswith('consumer/') or key=='root/scripts/perf/cases.mjs':eq(a['inputHashes'][key],b['inputHashes'][key])
    changed=[{'path': k,'control': a['inputHashes'].get(k),'candidate': b['inputHashes'].get(k)} for k in sorted(a['inputHashes'].keys()|b['inputHashes'].keys()) if a['inputHashes'].get(k)!=b['inputHashes'].get(k)]
    eq(changed,d['changedInputs'])
    cases={c['id']:c for c in d['cases']}
    for c in cases.values():
        eq(hashlib.sha256(c['source'].encode()).hexdigest(),c['sourceSha256']);eq(len(c['source'].encode()),c['sourceBytes'])
        eq(c['before'],c['after']);eq(c['before']['control'],c['before']['candidate'])
        for side in ['control','candidate']:
            eq(c['before'][side]['outputs'],[c['expected']]*5)
    r_out={'file':path.name,'sha256':sha(path),'startedAt':d['startedAt'],'completedAt':d['completedAt'],'rows':[]}
    first={'control':0,'candidate':0}
    for idx,r in enumerate(d['rows']):
        c=cases[r['case']]; maximum=max(1,min(128,25000//c['units'])); eq(len(r['raw']),15)
        assert 0<r['count']<=maximum
        eq(d['settings']['orderOffset'], 1 if r['case']=='er/medium' else 0)
        w=r['warmup'];assert w['pairs']>=6
        for side in ['control','candidate']:
            eq(w[side]['operations'],6+(w['pairs']-6)*maximum);assert w[side]['elapsedNs']>=1e9
        for group in ['before','after','calibration','raw']:
            pairs=r[group] if isinstance(r[group],list) else [r[group]]
            for i,p in enumerate(pairs):
                values=p['values']; eq({k:v for k,v in values['control'].items() if k!='elapsedNs'},{k:v for k,v in values['candidate'].items() if k!='elapsedNs'})
                if group in ['calibration','raw']:
                    eq(p['order'],['candidate','control'] if (d['settings']['orderOffset']+idx+i)%2 else ['control','candidate'])
                    eq(set(p['guards']),{'control','candidate'})
                    for guard in p['guards'].values():
                        for process in guard['externalLoad']:
                            assert process['name'] not in ['cargo','rustc','swiftc','swift-build','swift-frontend','xcodebuild','ld']
                            assert process['name']!='xctest' or process.get('testScope')!='pele'
                for v in values.values():
                    eq(v['output'],c['expected']);eq(v['caseID'],r['case']);eq(v['stage'],r['stage']);assert 0<v['count']<=maximum
                    if group in ['calibration','raw']:assert isinstance(v['elapsedNs'],int) and 0<v['elapsedNs']<2**53
                if group=='raw':eq(p['trial'],i);eq(values['control']['count'],r['count']);first[p['order'][0]]+=1
        counts=[x['values']['control']['count'] for x in r['calibration']]
        eq(counts[0],1);eq(counts[-1],r['count'])
        for i,n in enumerate(counts[1:],1):eq(n,min(maximum,counts[i-1]*2));assert max(x['elapsedNs'] for x in r['calibration'][i-1]['values'].values())<12e6
        assert counts[-1]==maximum or max(x['elapsedNs'] for x in r['calibration'][-1]['values'].values())>=12e6
        cv=[p['values']['control']['elapsedNs']/r['count']/1e6 for p in r['raw']];nv=[p['values']['candidate']['elapsedNs']/r['count']/1e6 for p in r['raw']];ratios=[n/c for n,c in zip(nv,cv)]
        s={'controlMs':cv,'candidateMs':nv,'pairedCandidateOverControl':stats(ratios),'ratioOfMedians':statistics.median(nv)/statistics.median(cv),'candidateFasterPairs':sum(x<1 for x in ratios),'pairedPercentChanges':[(x-1)*100 for x in ratios],'ratios':ratios}
        eq(s,r['statistics'])
        r_out['rows'].append({'case':r['case'],'stage':r['stage'],'count':r['count'],'controlMs':stats(cv),'candidateMs':stats(nv),'medianPercentChange':(s['ratioOfMedians']-1)*100,'pairedPercentChange':(statistics.median(ratios)-1)*100,'pairedMinPercentChange':(min(ratios)-1)*100,'pairedMaxPercentChange':(max(ratios)-1)*100,'wins':s['candidateFasterPairs'],'warmup':w})
    eq(first,d['orderCounts']);eq(first,{'control':7,'candidate':8} if d['settings']['orderOffset'] else {'control':8,'candidate':7});output['acceptedSwift']['reports'].append(r_out)
    for side in first:output['acceptedSwift']['orderCounts'][side]+=first[side]



eq(len(output['acceptedSwift']['reports']),6);eq(output['acceptedSwift']['orderCounts'],{'control':45,'candidate':45})
eq(sha(PUBLIC/'measured-compare.mjs'),comparator)
output['acceptedSwift']['comparatorSHA256']=comparator
output['acceptedSwift']['candidateInputAggregate']=d['builds']['candidate']['manifest']['sourceInputSha256']
eq(output['acceptedSwift']['candidateInputAggregate'],freeze['liveInputAggregate'])
summary=json.loads((PUBLIC/'summary.json').read_text())
eq(summary['acceptedWindows'],6);eq(summary['compilerInterruptedExcluded'],4);eq(summary['firstSideOrder'],output['acceptedSwift']['orderCounts'])
computed=[]
for e in events:
    if not e['complete']:continue
    report=next(x for x in output['acceptedSwift']['reports'] if x['file']==pathlib.Path(e['output']).name)
    row=report['rows'][0]
    computed.append({'case':row['case'],'stage':row['stage'],'controlMs':row['controlMs']['median'],'candidateMs':row['candidateMs']['median'],'pairedPercent':row['pairedPercentChange'],'fasterPairs':row['wins'],'report':report['file']})
eq(computed,summary['swift'])
output['publicSummaryMatchesRaw']=True
v=json.loads((BASE/'swift-verified.json').read_text());eq(v['mode'],'swift-api-correctness-only');assert v['completedAt']
eq(len(v['cases']),21);eq(len(v['rows']),168)
eq(v['buildManifest'],json.loads((BASE/'audit-candidate-build-manifest.json').read_text()))
case_by_id={c['id']:c for c in v['cases']}
for c in v['cases']:
    eq(hashlib.sha256(c['source'].encode()).hexdigest(),c['sourceSha256']);eq(len(c['source'].encode()),c['sourceBytes'])
    eq(c['prepared']['outputs'],[c['output']]*5)
for r in v['rows']:
    z=r['correctness'];eq(z['caseID'],r['case']);eq(z['stage'],r['stage']);eq(z['output'],case_by_id[r['case']]['output']);eq(z['count'],1);assert 'elapsedNs' not in z
for c in case_by_id: eq(len([r for r in v['rows'] if r['case']==c]),8)
output['swiftCorrectness']={'reportSHA256':sha(BASE/'swift-verified.json'),'cases':21,'apiRows':168,'timingClaim':False}
w=json.loads((BASE/'web-comparison.json').read_text());assert w['diagnostic'] and w['succeeded'];eq(len(w['rows']),18)
eq(w['settings']['trials'],21);eq(w['settings']['warmupMs'],2000);assert w['settings']['crossOriginIsolated'];assert 0<w['settings']['observedClockResolutionMs']<=.01
for path,key in [('scripts/perf/compare-web-vm.mjs','harnessSHA256'),('scripts/perf/cases.mjs','casesSHA256'),('node_modules/pele/dist/pele.js','oracleSHA256')]:eq(sha(ROOT/path),w[key])
wf=json.loads((BASE/'audit-web-freeze.json').read_text())
eq(wf['candidateHashes'],w['runtimeHashes']['candidate']);eq(wf['controlHashes'],w['runtimeHashes']['control']);eq(len(wf['candidateHashes']),105);eq(len(wf['controlHashes']),105)
for path,h in wf['controlEvidencePaths'].items():eq(sha(ROOT/path),h)
prior=json.loads((BASE.parent/'projection-policy/web-comparison.json').read_text());eq(w['runtimeHashes']['control'],prior['runtimeHashes']['candidate'])
prior=json.loads((BASE.parent/'projection-policy/web.json').read_text());assert all(w['runtimeHashes']['control'][k]==h for k,h in prior['runtimeHashes'].items())
eq(wf['changedPaths'],['pkg/pele_bg.wasm'])
output['web']={'reportSHA256':sha(BASE/'web-comparison.json'),'freezeSHA256':sha(BASE/'audit-web-freeze.json'),'candidatePackageFiles':105,'controlHashesComparedToPriorComparison':105,'controlHashesComparedToPriorFullGate':len(prior['runtimeHashes']),'rows':[]}
for r in w['rows']:
    for side in ['js','control','candidate']:
        vals=r['samples'][side];eq(len(vals),21);assert all(math.isfinite(x) and x>0 for x in vals)
        eq(q(vals,.5),r['medianMs'][side]);eq(q(vals,.95),r['p95BatchMeanMs'][side])
        assert r['counts'][side] in [1,2,4,8,16,32,64,128]
        assert r['warmups'][side]['elapsedMs']>=2000 and r['warmups'][side]['operations']>=6
    eq(r['counts']['control'],r['counts']['candidate'])
    for a,b,k in [('candidate','control','candidateOverControl'),('candidate','js','candidateOverJS'),('control','js','controlOverJS')]:eq(r['medianMs'][a]/r['medianMs'][b],r[k])
    ratios=[n/c for n,c in zip(r['samples']['candidate'],r['samples']['control'])]
    output['web']['rows'].append({k:r[k] for k in ['case','profile','stage','medianMs']}|{'medianPercentChange':100*(r['candidateOverControl']-1),'pairedPercentChange':100*(statistics.median(ratios)-1),'fasterPairs':sum(x<1 for x in ratios),'pairedMinPercent':100*(min(ratios)-1),'pairedMaxPercent':100*(max(ratios)-1)})
output['publicCopies']={}
for p in PUBLIC.iterdir():
    if p.suffix=='.md': continue # Authored public narrative is not an exact report copy.
    src=BASE/p.name
    if src.is_file():eq(sha(p),sha(src));output['publicCopies'][p.name]=sha(p)
    elif p.suffix=='.gz':
        import gzip
        src=BASE/p.name[:-3]
        if src.is_file():eq(gzip.decompress(p.read_bytes()),src.read_bytes());output['publicCopies'][p.name]=sha(p)
output['allChecksPassed']=True
(BASE/'results-audit.json').write_text(json.dumps(output,indent=2)+'\n')
print(json.dumps({'allChecksPassed':True,'swift':computed,'excluded':output['excludedWindows'],'webERRows':[r for r in output['web']['rows'] if r['case']=='er/medium']},indent=2))
