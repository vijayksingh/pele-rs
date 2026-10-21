"""Read-only independent arithmetic/provenance audit of saved Swift evidence."""
import argparse, hashlib, json, math, pathlib, re, statistics
ROOT = pathlib.Path(__file__).resolve().parents[4]
BASE = pathlib.Path(__file__).resolve().parent
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()

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

def stats(v):
    m=statistics.median(v)
    return dict(median=m,mad=statistics.median([abs(x-m) for x in v]),min=min(v),max=max(v),sampleP95=sorted(v)[math.ceil(len(v)*.95)-1])

out={'reports': [], 'profiles': []}
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--comparator', type=pathlib.Path, default=ROOT/'scripts/perf/swift/compare.mjs', help='Exact executed harness; use archived snapshot after source cleanup')
comparator=sha(parser.parse_args().comparator)
for path in sorted(BASE.glob('swift-*-attempt2.json')):
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
                    eq(p['order'],['candidate','control'] if (idx+i)%2 else ['control','candidate'])
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
    eq(first,d['orderCounts']);eq(first,{'control':8,'candidate':7});out['reports'].append(r_out)


webpath=BASE/'web-comparison.json';web=json.loads(webpath.read_text())
assert web['succeeded'] and web['diagnostic'] and len(web['rows'])==24
assert web['settings']['trials']==15 and web['settings']['warmupMs']==1000
assert web['settings']['crossOriginIsolated'] and 0<web['settings']['observedClockResolutionMs']<=.01
eq(sha(ROOT/'scripts/perf/compare-web-vm.mjs'),web['harnessSHA256'])
eq(sha(ROOT/'scripts/perf/cases.mjs'),web['casesSHA256'])
eq(sha(ROOT/'node_modules/pele/dist/pele.js'),web['oracleSHA256'])
out['web']={'reportSHA256':sha(webpath),'rows':[]}
for row in web['rows']:
    for side in ['js','control','candidate']:
        samples=row['samples'][side];eq(len(samples),15);assert all(x>0 and math.isfinite(x) for x in samples)
        eq(statistics.median(samples),row['medianMs'][side]);eq(max(samples),row['p95BatchMeanMs'][side])
        assert row['warmups'][side]['elapsedMs']>=1000 and row['warmups'][side]['operations']>=6
        assert row['counts'][side] in [1,2,4,8,16,32,64,128]
    eq(row['counts']['control'],row['counts']['candidate'])
    eq(row['medianMs']['candidate']/row['medianMs']['control'],row['candidateOverControl'])
    eq(row['medianMs']['candidate']/row['medianMs']['js'],row['candidateOverJS'])
    eq(row['medianMs']['control']/row['medianMs']['js'],row['controlOverJS'])
    ratios=[n/c for n,c in zip(row['samples']['candidate'],row['samples']['control'])]
    out['web']['rows'].append({k:row[k] for k in ['case','profile','stage','counts','medianMs']}|{'medianPercentChange':(row['candidateOverControl']-1)*100,'pairedPercentChange':(statistics.median(ratios)-1)*100,'wins':sum(x<1 for x in ratios),'pairedMinPercentChange':(min(ratios)-1)*100,'pairedMaxPercentChange':(max(ratios)-1)*100,'pairedRatios':ratios})
a,b=web['runtimeHashes']['control'],web['runtimeHashes']['candidate']
out['web']['changedRuntimeFiles']=[k for k in sorted(a.keys()|b.keys()) if a.get(k)!=b.get(k)]
eq(out['web']['changedRuntimeFiles'],['pkg/pele_bg.wasm'])
events=json.loads((BASE/'quiet-windows.json').read_text());eq(len(events),7);assert events[0]['complete']==False
first=json.loads((BASE/'swift-parse-sequence-small-attempt1.json').read_text());assert not first['completed'] and not first['rows'] and not first['cases'] and 'EACCES' in first['failure']
for e in events[1:]:assert e['complete'] and e['code']==0 and e['attempt']==2
profile=json.loads((BASE/'post-profile/profile.json').read_text());attribution=json.loads((BASE/'post-profile/attribution.json').read_text())
eq(profile['buildManifest'],d['builds']['candidate']['manifest'])
cases={c['id']:c for report in BASE.glob('swift-*-attempt2.json') for c in json.loads(report.read_text())['cases']}
filters={'exercise':lambda s:'Prepared.exercise(' in s,'foundation':lambda s:'JSONDecoder.decode<' in s,'configValue':lambda s:'ConfigValue.init(from:)' in s or 'Decodable.init(from:) in conformance ConfigValue' in s,'nativeParse':lambda s:'pele_parse_eager_v1' in s,'swiftEvents':lambda s:'receiveEagerProjection(' in s,'registration':lambda s:'Engine::with_builtins' in s,'envelopeCount':lambda s:'success_envelope_len' in s,'projection':lambda s:'normalized_projection' in s or 'project_model' in s or 'document_envelope' in s,'legacyProjection':lambda s:'project_model' in s,'legacyModelParse':lambda s:'parse_model_json' in s}
for p in profile['profiles']:
    sample=BASE/'post-profile'/p['filename'];eq(sha(sample),p['sampleSHA256']);eq(p['verified'],cases[p['case']]['before']['candidate'])
    text=sample.read_text().split('Call graph:')[1].split('Total number in stack')[0]; stack=[];counts=dict.fromkeys(filters,0)
    for line in text.splitlines():
        m=re.match(r'^([ +!:|]*)(\d+) (.+)$',line)
        if not m:continue
        depth=len(m[1]);count=int(m[2]);frame=m[3]
        while stack and stack[-1][0]>=depth:stack.pop()
        if filters['exercise'](frame) or any(filters['exercise'](s) for _,s in stack):
            for k,f in filters.items():
                if f(frame) and not any(f(s) for _,s in stack):counts[k]+=count
        stack.append((depth,frame))
    expected=next(x for x in attribution['profiles'] if x['case']==sample.name.replace('.sample.txt',''))
    eq(counts,expected['counts']);eq(sha(sample),expected['sourceSHA256']);eq({k:100*v/counts['exercise'] for k,v in counts.items()},expected['percentOfExercise']);out['profiles'].append(expected)


for side,package in [('control',BASE/'parse-architecture/direct-gantt/package'),('candidate',ROOT/'dist/package-web/package')]:
    actual={str(p.relative_to(package)):sha(p) for p in package.rglob('*') if p.is_file()}
    eq(actual,web['runtimeHashes'][side])
    out['web'][side+'PackageFileCount']=len(actual)
gatepath=BASE/'parse-architecture/normalized-projection/gate.json';gatewebpath=gatepath.with_name('web.json')
gate=json.loads(gatepath.read_text());gateweb=json.loads(gatewebpath.read_text());assert gateweb['succeeded']
eq(sha(ROOT/'scripts/perf/web-vm.mjs'),gateweb['harnessSHA256']);eq(web['casesSHA256'],gateweb['casesSHA256']);eq(web['oracleSHA256'],gateweb['oracleSHA256'])
for path,h in gateweb['runtimeHashes'].items():eq(h,web['runtimeHashes']['candidate'][path])
slower=[]
for row in gateweb['rows']:
    for side,key in [('js','jsMedianMs'),('wasm','wasmMedianMs')]:
        eq(statistics.median(row['samples'][side]),row[key]);eq(max(row['samples'][side]),row[side+'P95BatchMeanMs'])
    eq(row['wasmMedianMs']/row['jsMedianMs'],row['wasmOverJS'])
    if row['wasmOverJS']>1:slower.append({k:row[k] for k in ['case','profile','stage','wasmOverJS']})
eq(gate,{'passed':not slower,'checked':len(gateweb['rows']),'slowerCount':len(slower),'slower':slower})
out['gate']={'reportSHA256':sha(gatewebpath),'gateSHA256':sha(gatepath),'passed':gate['passed'],'checked':gate['checked'],'slowerCount':gate['slowerCount'],'medianRatioByStage':{stage:statistics.median([r['wasmOverJS'] for r in gateweb['rows'] if r['stage']==stage]) for stage in ['render-source','parse','render-model']}}

out['comparatorSha256']=comparator;out['checks']='All assertions passed; current input aggregate and frozen package contents checked separately without executing consumer/build/browser.'
print(json.dumps(out,indent=2))
