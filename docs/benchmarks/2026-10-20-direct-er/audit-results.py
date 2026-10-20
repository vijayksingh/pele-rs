"""Read-only saved-evidence audit; no consumer, browser, build, or benchmark execution."""
import hashlib,json,math,pathlib,statistics
BASE=pathlib.Path(__file__).resolve().parent
ROOT=next(p for p in BASE.parents if (p/'scripts/perf/swift/build.mjs').is_file())
# Public archived copy uses the retained private frozen binaries when available.
PRIVATE=ROOT/'dist/perf-patches/parse-architecture/direct-er'
if BASE.name=='2026-10-20-direct-er' and PRIVATE.is_dir(): BASE=PRIVATE
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
PUBLIC=ROOT/'docs/benchmarks/2026-10-20-direct-er'
freeze=json.loads((BASE/'audit-source-freeze.json').read_text())
eq(freeze['revision'],'175daa8fd76fc1e25f488e4957b6970b4b6d4fe2')
eq(len(freeze['inputHashes']),348)
eq(freeze['sourceInputSha256'],'7c37902a471c01561ec3d927887a5e3b15e78deff6ee37d2ae7a1584955d48d0')
output['sourceFreeze']=freeze
output['sourceFreezeSHA256']=sha(BASE/'audit-source-freeze.json')
events=json.loads((BASE/'single-windows.json').read_text());eq(len(events),11)
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
        assert all(len(r['raw'])<15 for r in d['rows'])
    output['excludedWindows'].append({'stage':e['stage'],'case':e['workload'],'attempt':e['attempt'],'reportExists':p.exists(),'logSHA256':sha(log),'excludedRawPairs':sum(len(r['raw']) for r in d.get('rows',[])), 'measuredPairsIncluded':0})
eq(len(output['excludedWindows']),5)
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
        else: eq(b['binarySha256'],freeze['binarySha256'])
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
eq(sha(PUBLIC/'measured-swift-comparison.mjs'),comparator)
output['acceptedSwift']['comparatorSHA256']=comparator
output['acceptedSwift']['candidateInputAggregate']=d['builds']['candidate']['manifest']['sourceInputSha256']
eq(output['acceptedSwift']['candidateInputAggregate'],freeze['sourceInputSha256'])
# Current native inputs, frozen binaries and reports; no executable is started.
import subprocess
files=subprocess.check_output(['rg','--files','crates/pele/src','scripts/perf/native/src'],cwd=ROOT,text=True).strip().splitlines()+['crates/pele/Cargo.toml','Cargo.toml','scripts/perf/native/Cargo.toml','scripts/perf/native/Cargo.lock']
h=hashlib.sha256();native_files={}
for name in sorted(files):
    data=(ROOT/name).read_bytes();h.update(name.encode()+b'\0');h.update(data);native_files[name]=hashlib.sha256(data).hexdigest()
ndirs={'control':BASE.parent/'normalized-er/control-native','candidate':ROOT/'scripts/perf/native/target/release'}
native_manifests={s:json.loads((p/'build-manifest.json').read_text()) for s,p in ndirs.items()}
eq(h.hexdigest(),native_manifests['candidate']['sourceInputSha256'])
for side,p in ndirs.items():
    for name,hashvalue in native_manifests[side]['binaries'].items():eq(sha(p/name),hashvalue)
output['nativeFreeze']={'inputAggregate':h.hexdigest(),'inputFiles':native_files,'manifests':native_manifests}
# SDK host archive equals the source-verified build manifest's linked slice.
m=json.loads((BASE/'audit-candidate-build-manifest.json').read_text())
eq(sha(pathlib.Path(m['nativeArchive'])),m['nativeHostArchiveSha256'])
eq(sha(ROOT/'target'/m['rustTarget']/'release/libpele_ffi.a'),m['nativeHostArchiveSha256'])
output['sdkArchiveMatchesSourceBuiltTarget']=True

def hashes(d,harness):
    for path,key in [(harness,'harnessSHA256'),('scripts/perf/cases.mjs','casesSHA256'),('node_modules/pele/dist/pele.js','oracleSHA256')]:eq(sha(ROOT/path),d[key])
def native_artifacts(d):
    hashes(d,'scripts/perf/compare-native.mjs')
    for side,a in d['artifacts'].items():
        eq(a['manifest'],native_manifests[side]);eq(a['binarySHA256'],native_manifests[side]['binaries']['pele-perf'])
        if 'allocationSHA256' in a:eq(a['allocationSHA256'],native_manifests[side]['binaries']['pele-perf-alloc'])
    for c in d['cases']:
        eq(hashlib.sha256(c['source'].encode()).hexdigest(),c['sourceSHA256']);assert c['jsRetainedBeforeVerified']
def summarized(r,control,candidate):
    ratios=[n/c for n,c in zip(candidate,control)]
    return {'case':r['case'],'stage':r['stage'],'controlMs':statistics.median(control),'candidateMs':statistics.median(candidate),'pairedPercent':100*(statistics.median(ratios)-1),'fasterPairs':sum(n<1 for n in ratios),'medianPercent':100*(statistics.median(candidate)/statistics.median(control)-1),'pairedMinPercent':100*(min(ratios)-1),'pairedMaxPercent':100*(max(ratios)-1)}
output['native']=[]
nevents=json.loads((BASE/'native-windows.json').read_text());eq(len(nevents),5)
for e in nevents:
    assert e['complete'] and e['code']==0
    p=ROOT/e['output'];d=json.loads(p.read_text());assert d['succeeded'];native_artifacts(d)
    eq(d['settings']['trials'],15);eq(d['settings']['warmupMs'],1000);eq(d['settings']['fixedBatchCount'],16);assert not d['settings']['separateAllocationBinaries']
    eq(len(d['rows']),1)
    for r in d['rows']:
        for side in ['js','control','candidate']:
            eq(r['counts'][side],16);eq(len(r['raw'][side]),15)
            eq([v['elapsedNs']/v['count']/1e6 for v in r['raw'][side]],r['samples'][side]);eq(statistics.median(r['samples'][side]),r['medians'][side])
            for v in r['raw'][side]:eq(v['count'],16);assert isinstance(v['elapsedNs'],int) and 0<v['elapsedNs']<2**53
            assert r['warmups'][side]['elapsedNs']>=1e9 and (r['warmups'][side]['operations']-6)%32==0
        for a,b in zip(r['raw']['control'],r['raw']['candidate']):eq({k:v for k,v in a.items() if k!='elapsedNs'},{k:v for k,v in b.items() if k!='elapsedNs'})
        eq(r['medians']['candidate']/r['medians']['control'],r['candidateOverControl']);eq(r['medians']['candidate']/r['medians']['js'],r['candidateOverJS']);assert r['jsRetainedPostBatchVerified']
        out=summarized(r,r['samples']['control'],r['samples']['candidate']);out.update(report=p.name,reportSHA256=sha(p),jsMs=r['medians']['js']);output['native'].append(out)
output['nativeOrder']={'sourceVerified':True,'rawOrderNotRecorded':True,'orderRule':'cyclic names by trial % 3; reverse every 3 trials','firstPositions':{}}
names=['js','control','candidate']
orders=[]
for i in range(15):
    order=[names[(j+i%3)%3] for j in range(3)]
    if (i//3)%2:order.reverse()
    orders.append(order)
output['nativeOrder']['firstPositions']={n:sum(x[0]==n for x in orders) for n in names}
a=json.loads((BASE/'native-allocations.json').read_text());native_artifacts(a);assert a['succeeded'];eq(a['mode'],'native-allocations-only');assert a['settings']['timingRequested'] is False;eq(a['rows'],[]);eq(len(a['allocationRows']),10)
for r in a['allocationRows']:
    eq(r['count'],1)
    for side in ['control','candidate']:
        c=r[side];assert 'elapsedNs' not in c;eq(c['requestedBytes']-c['freedBytes'],c['liveBytesDelta']);assert c['peakAdditionalRequestedBytes']>=max(0,c['liveBytesDelta'])
    eq(r['control']['checksum'],r['candidate']['checksum'])
    if (r['case'],r['stage'])!=('er/medium','parse-public'):eq(r['control'],r['candidate'])
output['allocations']={'reportSHA256':sha(BASE/'native-allocations.json'),'rows':a['allocationRows'],'scope':'one operation after prepared state/reset; instrumentation off before JSON; allocator requested bytes and net live delta, not RSS or absolute retained heap','timingClaim':False}
# Swift 168 consumer APIs remain exact correctness-only.
v=json.loads((BASE/'swift-verified.json').read_text());eq(v['mode'],'swift-api-correctness-only');eq(len(v['rows']),168);eq(len(v['cases']),21);eq(v['buildManifest'],m)
vc={c['id']:c for c in v['cases']}
for c in vc.values():eq(c['prepared']['outputs'],[c['output']]*5)
for r in v['rows']:eq(r['correctness']['output'],vc[r['case']]['output']);assert 'elapsedNs' not in r['correctness']
output['swiftCorrectness']={'rows':168,'cases':21,'sha256':sha(BASE/'swift-verified.json')}
# Web package freeze and paired arithmetic. Order follows the hashed harness,
# since this report stores per-condition arrays rather than raw order records.
w=json.loads((BASE/'web-comparison.json').read_text());hashes(w,'scripts/perf/compare-web-vm.mjs');assert w['succeeded'] and w['diagnostic'];eq(len(w['rows']),30);eq(w['settings']['trials'],21);eq(w['settings']['warmupMs'],1000)
actual=package(ROOT/'dist/package-web/package');eq(actual,w['runtimeHashes']['candidate']);eq(len(actual),105)
prior=json.loads((BASE.parent/'normalized-er/web-comparison.json').read_text());eq(w['runtimeHashes']['control'],prior['runtimeHashes']['candidate']);eq(len(w['runtimeHashes']['control']),105)
output['web']={'candidateHashes':actual,'controlHashes':w['runtimeHashes']['control'],'controlProvenance':'exact match to prior normalized-er/web-comparison.json candidate package hashes','reportSHA256':sha(BASE/'web-comparison.json'),'rows':[]}
for r in w['rows']:
    for side in ['js','control','candidate']:
        vals=r['samples'][side];eq(len(vals),21);eq(q(vals,.5),r['medianMs'][side]);eq(q(vals,.95),r['p95BatchMeanMs'][side]);assert r['warmups'][side]['elapsedMs']>=1000
    eq(r['counts']['control'],r['counts']['candidate'])
    for a,b,k in [('candidate','control','candidateOverControl'),('candidate','js','candidateOverJS'),('control','js','controlOverJS')]:eq(r['medianMs'][a]/r['medianMs'][b],r[k])
    out=summarized(r,r['samples']['control'],r['samples']['candidate']);out.update(profile=r['profile']);output['web']['rows'].append(out)
# Public staged evidence copies and summary independently agree with raw reports.
summary=json.loads((PUBLIC/'summary.json').read_text());eq(summary['swiftAcceptedWindows'],6);eq(summary['swiftCompilerInterruptedExcluded'],5);eq(summary['nativeAcceptedWindows'],5);eq(summary['swiftFirstSideOrder'],{'control':45,'candidate':45})
for s in summary['rows']:
    if s['category']=='native':r=next(x for x in output['native'] if x['report']==s['report']);eq(s['jsMs'],r['jsMs'])
    else:
        report=next(x for x in output['acceptedSwift']['reports'] if x['file']==s['report']);z=report['rows'][0];r={'controlMs':z['controlMs']['median'],'candidateMs':z['candidateMs']['median'],'pairedPercent':z['pairedPercentChange'],'fasterPairs':z['wins']}
    for k in ['controlMs','candidateMs','pairedPercent','fasterPairs']:eq(s[k],r[k])
output['publicSummaryMatchesRaw']=True;output['publicCopies']={}
for p in PUBLIC.iterdir():
    if p.suffix=='.md' or p.name in ['audit-results.py','results-audit.json']:continue # Parent refreshes evolving audit copies after final review.
    src=BASE/p.name
    if src.is_file():eq(sha(p),sha(src));output['publicCopies'][p.name]=sha(p)
    elif p.suffix=='.gz':
        import gzip
        src=BASE/p.name[:-3]
        if src.is_file():eq(gzip.decompress(p.read_bytes()),src.read_bytes());output['publicCopies'][p.name]=sha(p)
casefreeze={c['id']:c for c in json.loads((BASE/'audit-case-freeze.json').read_text())}
for c in w['cases']:
    eq(c['sourceSHA256'],casefreeze[c['id']]['sourceSHA256']);eq(c['sourceBytes'],casefreeze[c['id']]['sourceBytes'])
for e in nevents:
    d=json.loads((ROOT/e['output']).read_text())
    for c in d['cases']:eq(c['source'],casefreeze[c['id']]['source'])
eq(sha(PUBLIC/'measured-comparison.mjs'),w['harnessSHA256'])
output['caseSourceFreezeSHA256']=sha(BASE/'audit-case-freeze.json')
# Separate confirmation and identical-binary A/A diagnostics; never pooled.
output['nativeConfirmation']=[]
cevents=json.loads((BASE/'native-confirmation-windows.json').read_text());eq(len(cevents),4)
for e in cevents:assert e['complete'] and e['code']==0 and e['attempt']==1
for p in [ROOT/e['output'] for e in cevents]+[BASE/'native-aa-gantt-public.json']:
    d=json.loads(p.read_text());assert d['succeeded'];is_aa=p.name.startswith('native-aa')
    if is_aa:
        hashes(d,'scripts/perf/compare-native.mjs');eq(d['artifacts']['control'],d['artifacts']['candidate']);eq(d['artifacts']['candidate']['manifest'],native_manifests['candidate'])
    else:native_artifacts(d)
    eq(d['settings']['trials'],15);eq(d['settings']['warmupMs'],1000);eq(d['settings']['fixedBatchCount'],16)
    eq(len(d['rows']),1);r=d['rows'][0]
    for side in ['js','control','candidate']:
        eq(r['counts'][side],16);eq(len(r['raw'][side]),15)
        eq([v['elapsedNs']/v['count']/1e6 for v in r['raw'][side]],r['samples'][side]);eq(statistics.median(r['samples'][side]),r['medians'][side]);assert r['warmups'][side]['elapsedNs']>=1e9
        for v in r['raw'][side]:eq(v['count'],16);assert 0<v['elapsedNs']<2**53
    for aa,bb in zip(r['raw']['control'],r['raw']['candidate']):eq({k:v for k,v in aa.items() if k!='elapsedNs'},{k:v for k,v in bb.items() if k!='elapsedNs'})
    eq(r['medians']['candidate']/r['medians']['control'],r['candidateOverControl']);eq(r['medians']['candidate']/r['medians']['js'],r['candidateOverJS']);assert r['jsRetainedPostBatchVerified']
    z=summarized(r,r['samples']['control'],r['samples']['candidate']);z.update(report=p.name,sha256=sha(p),identicalBinaryAA=is_aa);output['nativeConfirmation'].append(z)
f=json.loads((BASE/'first-use.json').read_text());hashes(f,'scripts/perf/compare-first-use-web-vm.mjs');eq(sha(PUBLIC/'measured-first-use.mjs'),f['harnessSHA256']);eq(f['runtimeHashes'],w['runtimeHashes']);assert f['succeeded'];eq(len(f['rows']),38);eq(f['settings']['trials'],15);eq(f['settings']['warmupMs'],1000);assert f['settings']['freshDocumentPerOperation'] and f['settings']['disposeInsideTimer'];assert 'parse-first-callback' not in f['settings']['stages']
output['firstUse']={'sha256':sha(BASE/'first-use.json'),'callbackMeasured':False,'rows':[]}
for c in f['cases']:
    eq(c['sourceSHA256'],casefreeze[c['id']]['sourceSHA256']);eq(c['sourceBytes'],casefreeze[c['id']]['sourceBytes'])
    eq(len(c['checks']),6 if c['id']=='sequence/small' else 8)
    for check in c['checks']:
        if check['stage']=='parse-first-invoke':assert check['firstCallSignatureBytes']>0 and len(check['firstCallSignatureSha256'])==64
for r in f['rows']:
    for side in ['js','control','candidate']:
        vals=r['samples'][side];eq(len(vals),15);eq(q(vals,.5),r['medianMs'][side]);eq(q(vals,.95),r['p95BatchMeanMs'][side]);assert r['warmups'][side]['elapsedMs']>=1000;assert r['counts'][side] in [1,2,4,8,16,32,64,128]
    eq(r['counts']['control'],r['counts']['candidate'])
    for aa,bb,k in [('candidate','control','candidateOverControl'),('candidate','js','candidateOverJS'),('control','js','controlOverJS')]:eq(r['medianMs'][aa]/r['medianMs'][bb],r[k])
    z=summarized(r,r['samples']['control'],r['samples']['candidate']);z.update(profile=r['profile']);output['firstUse']['rows'].append(z)
g=json.loads((BASE/'web.json').read_text());hashes(g,'scripts/perf/web-vm.mjs');assert g['succeeded'];eq(g['settings']['trials'],9);eq(g['settings']['minimumWarmupMs'],500);assert g['settings']['crossOriginIsolated'] and 0<g['settings']['observedClockResolutionMs']<=.01
eq(len(g['runtimeHashes']),8)
for name,hashvalue in g['runtimeHashes'].items():eq(hashvalue,actual[name])
expected={(c,p,s) for c in casefreeze for p in ['fixed','canvas'] for s in ['render-source','parse','render-model']};eq(len(expected),126);eq(len(g['rows']),126);eq(len(g['cases']),21)
for c in g['cases']:eq(c['sourceSHA256'],casefreeze[c['id']]['sourceSHA256']);eq(sorted(x['profile'] for x in c['checks']),['canvas','fixed'])
slower=[]
for r in g['rows']:
    key=(r['case'],r['profile'],r['stage']);assert key in expected;expected.remove(key)
    for side in ['js','wasm']:
        eq(len(r['samples'][side]),9);eq(q(r['samples'][side],.5),r[side+'MedianMs']);eq(q(r['samples'][side],.95),r[side+'P95BatchMeanMs']);assert r['warmups'][side]['elapsedMs']>=500
    eq(r['wasmMedianMs']/r['jsMedianMs'],r['wasmOverJS'])
    if r['wasmMedianMs']>r['jsMedianMs']:slower.append({k:r[k] for k in ['case','profile','stage','wasmOverJS']})
assert not expected
gate={'passed':not slower,'checked':126,'slowerCount':len(slower),'slower':slower};eq(gate,json.loads((BASE/'gate.json').read_text()))
output['fullGate']={'sha256':sha(BASE/'web.json'),'gateSHA256':sha(BASE/'gate.json'),'checkerSHA256':sha(ROOT/'scripts/perf/check-web-parity.mjs'),'passed':False,'checked':126,'slowerCount':126,'stageMedianRatios':{s:statistics.median(r['wasmOverJS'] for r in g['rows'] if r['stage']==s) for s in ['render-source','parse','render-model']}}
output['allChecksPassed']=True
(BASE/'results-audit.json').write_text(json.dumps(output,indent=2)+'\n')
print(json.dumps({'passed':True,'swift':[x['rows'][0] for x in output['acceptedSwift']['reports']],'native':output['native'],'excluded':output['excludedWindows'],'webERParse':[r for r in output['web']['rows'] if r['case'].startswith('er/') and r['stage']=='parse']},indent=2))
