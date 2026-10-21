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
output={'comparisons':[]}
for name,control in [('web-comparison.json','normalized-projection'),('web-before-normalization.json','direct-gantt')]:
    d=json.loads((BASE/name).read_text());assert d['succeeded'] and d['diagnostic'];eq(len(d['rows']),8)
    eq(d['settings']['trials'],21);eq(d['settings']['warmupMs'],2000);assert d['settings']['crossOriginIsolated'] and 0<d['settings']['observedClockResolutionMs']<=.01
    for f,key in [('scripts/perf/compare-web-vm.mjs','harnessSHA256'),('scripts/perf/cases.mjs','casesSHA256'),('node_modules/pele/dist/pele.js','oracleSHA256')]:eq(sha(ROOT/f),d[key])
    for side,path in [('control',BASE/'parse-architecture'/control/'package'),('candidate',ROOT/'dist/package-web/package')]:eq(package(path),d['runtimeHashes'][side]);eq(len(d['runtimeHashes'][side]),105)
    changed=[k for k,v in d['runtimeHashes']['candidate'].items() if d['runtimeHashes']['control'].get(k)!=v];eq(changed,['pkg/pele_bg.wasm'])
    result={'file':name,'sha256':sha(BASE/name),'controlWasm':d['runtimeHashes']['control']['pkg/pele_bg.wasm'],'candidateWasm':d['runtimeHashes']['candidate']['pkg/pele_bg.wasm'],'rows':[]}
    for r in d['rows']:
        for side in ['js','control','candidate']:
            v=r['samples'][side];eq(len(v),21);assert all(math.isfinite(x) and x>0 for x in v)
            eq(q(v,.5),r['medianMs'][side]);eq(q(v,.95),r['p95BatchMeanMs'][side])
            assert r['counts'][side] in [1,2,4,8,16,32,64,128]
            assert r['warmups'][side]['elapsedMs']>=2000 and r['warmups'][side]['operations']>=6
        eq(r['counts']['control'],r['counts']['candidate'])
        for a,b,k in [('candidate','control','candidateOverControl'),('candidate','js','candidateOverJS'),('control','js','controlOverJS')]:eq(r['medianMs'][a]/r['medianMs'][b],r[k])
        ratios=[n/c for n,c in zip(r['samples']['candidate'],r['samples']['control'])]
        result['rows'].append({k:r[k] for k in ['case','profile','stage','medianMs','counts']}|{'medianPercentChange':100*(r['candidateOverControl']-1),'pairedPercentChange':100*(q(ratios,.5)-1),'wins':sum(x<1 for x in ratios),'pairedMinPercentChange':100*(min(ratios)-1),'pairedMaxPercentChange':100*(max(ratios)-1),'pairedRatios':ratios})
    output['comparisons'].append(result)
w=json.loads((BASE/'web.json').read_text());g=json.loads((BASE/'gate.json').read_text());assert w['succeeded'];eq(len(w['rows']),126)
eq(sha(ROOT/'scripts/perf/web-vm.mjs'),w['harnessSHA256'])
for key in ['casesSHA256','oracleSHA256']:eq(w[key],d[key])
for f,h in w['runtimeHashes'].items():eq(h,d['runtimeHashes']['candidate'][f])
slower=[]
for r in w['rows']:
    for side in ['js','wasm']:
        eq(len(r['samples'][side]),9);eq(q(r['samples'][side],.5),r[side+'MedianMs']);eq(q(r['samples'][side],.95),r[side+'P95BatchMeanMs'])
        assert r['warmups'][side]['elapsedMs']>=500 and r['warmups'][side]['operations']>=6
    eq(r['wasmMedianMs']/r['jsMedianMs'],r['wasmOverJS'])
    if r['wasmOverJS']>1:slower.append({k:r[k] for k in ['case','profile','stage','wasmOverJS']})
eq(g,{'passed':not slower,'checked':len(w['rows']),'slowerCount':len(slower),'slower':slower})
output['gate']={'webSHA256':sha(BASE/'web.json'),'gateSHA256':sha(BASE/'gate.json'),'checked':g['checked'],'slowerCount':g['slowerCount'],'passed':g['passed'],'medianRatioByStage':{s:q([r['wasmOverJS'] for r in w['rows'] if r['stage']==s],.5) for s in ['render-source','parse','render-model']},'averageMiddleMedianRatioByStage':{s:statistics.median([r['wasmOverJS'] for r in w['rows'] if r['stage']==s]) for s in ['render-source','parse','render-model']}}
output['allocations']={}
for side in ['control','candidate']:
    m=json.loads((BASE/(side+'-registry')/'manifest.json').read_text());eq(sha(BASE/(side+'-registry')/'registry_allocations'),m['binarySHA256'])
    for f,h in m['files'].items():eq(sha(BASE/'originals'/f if side=='control' and f.endswith('/engine.rs') else ROOT/f),h)
    f=BASE/'control-registry/allocations.json' if side=='control' else BASE/'candidate-registry-allocations.json';a=json.loads(f.read_text());eq(len(a['rows']),5)
    expected={'allocations':69 if side=='control' else 36,'oneByteAllocations':33 if side=='control' else 0,'reallocations':4,'requestedBytes':3948 if side=='control' else 3915}
    assert all(r==expected for r in a['rows']);output['allocations'][side]={'reportSHA256':sha(f),'binarySHA256':m['binarySHA256'],'row':expected}
output['rejectedSwiftAttempts']=[]
for f in sorted(BASE.glob('swift-parse-gantt-sequence-attempt*.json')):
    r=json.loads(f.read_text());assert not r['completed'] and 'Build/Pele test process active; defer this operation' in r['failure']
    eq(r['comparatorSha256'],sha(BASE/'measured-before-order-offset.mjs'))
    output['rejectedSwiftAttempts'].append({'file':f.name,'completed':False,'rawRows':[{k:x[k] for k in ['case','stage']}|{'rawPairs':len(x.get('raw',[]))} for x in r['rows']]})
eq(len(output['rejectedSwiftAttempts']),4)
v=json.loads((BASE/'order-offset-verified.json').read_text());assert v['completed'] and v['mode']=='matched-swift-api-correctness-only';eq(v['settings']['orderOffset'],1);eq(v['comparatorSha256'],sha(ROOT/'scripts/perf/swift/compare.mjs'))
for row in v['rows']:eq(row['raw'],[]);eq(row['before']['order'],['candidate','control'])
for c in v['cases']:
    eq(c['before'],c['after']);eq(c['before']['control'],c['before']['candidate'])
    for side in ['control','candidate']:eq(c['before'][side]['outputs'],[c['expected']]*5)
for row in v['rows']:
    for phase in ['before','after']:
        values=row[phase]['values'];eq(values['control'],values['candidate']);assert all('elapsedNs' not in x for x in values.values())
output['orderOffsetCorrectnessHarnessSHA256']=v['comparatorSha256']
def stats(v):
    m=statistics.median(v)
    return dict(median=m,mad=statistics.median([abs(x-m) for x in v]),min=min(v),max=max(v),sampleP95=sorted(v)[math.ceil(len(v)*.95)-1])


comparator=sha(ROOT/'scripts/perf/swift/compare.mjs')
output['acceptedSwift']={'reports':[],'orderCounts':{'control':0,'candidate':0}}
for path in sorted(BASE.glob('swift-single-*-attempt1.json')):
    d=json.loads(path.read_text())
    assert d['completed'] and d['mode']=='matched-local-swift-api-timing'
    eq(d['comparatorSha256'],comparator)
    eq(sha(ROOT/'node_modules/pele/dist/pele.js'),d['oracleSha256'])
    eq(d['settings']['trials'],15); eq(d['settings']['warmupMsPerBinaryPerStage'],1000);eq(d['settings']['warmupMaximumBatch'],128)
    for side,b in d['builds'].items():
        eq(sha(pathlib.Path(b['binary'])),b['binarySha256']);eq(b['binarySha256'],b['manifest']['binarySha256'])
        eq(sha(pathlib.Path(b['manifestPath'])),b['manifestSha256']);eq(json.loads(pathlib.Path(b['manifestPath']).read_text()),b['manifest'])
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
events=json.loads((BASE/'single-windows.json').read_text());eq(len(events),6)
assert all(e['complete'] and e['code']==0 and e['attempt']==1 for e in events)
output['acceptedSwift']['comparatorSHA256']=comparator
output['acceptedSwift']['candidateSourceInputSHA256']=d['builds']['candidate']['manifest']['sourceInputSha256']
output['acceptedSwift']['controlSourceInputSHA256']=d['builds']['control']['manifest']['sourceInputSha256']

print(json.dumps(output,indent=2))
