"""Portable data-only source/SDK evidence verification. No process execution."""
from pathlib import Path
import json,gzip,hashlib,re,math,statistics
root=Path(__file__).resolve().parent;manifest=json.loads((root/'artifact-manifest.json').read_text());items={x['stored']:x for x in manifest['artifacts']}
def raw(n):
 x=items[n];b=(root/n).read_bytes();assert len(b)==x['storedBytes'] and hashlib.sha256(b).hexdigest()==x['storedSHA256'],n;b=gzip.decompress(b)if x['gzip']else b;assert len(b)==x['originalBytes'] and hashlib.sha256(b).hexdigest()==x['originalSHA256'],n;return b
def load(n):return json.loads(raw(n))
for n in items:raw(n)
records=[load('native/'+n+'.json.gz')for n in ['before-sdk','after-sdk','before-correctness','after-correctness']];b=records[0];a=records[-1]
assert all(x['commit']==manifest['sourceCommit']for x in records)
assert records[1]['current']==records[2]['current']==a['current']
for x in records:
 for key in ['extraHashes','driverHashes','packageHashes']:assert x[key]==b[key]
assert [k for k in b['current']['files']if b['current']['files'][k]!=a['current']['files'][k]]==['CPele/macos-arm64/libpele_ffi.a']
assert len(a['current']['files'])==370 and len(b['gitInputHashes'])==402
for key,h in b['gitInputHashes'].items():
 path=b['gitInputPaths'][key];assert h==(b['extraHashes'][path]if key.startswith('correctness/')else b['current']['files'][key])
consumer=load('native/swift-build-manifest.json.gz');assert consumer==a['manifest'] and consumer['inputHashes']==a['current']['files'] and consumer['sourceInputSha256']==a['current']['sha256']
assert consumer['nativeHostArchiveSha256']==a['nativeArchiveSha256']==a['current']['files']['CPele/macos-arm64/libpele_ffi.a']
assert consumer['binarySha256']==a['binarySha256']
summary=load('native/native-evidence-summary.json');assert summary['sourceInputSHA256']==a['current']['sha256'] and summary['correctness']==a['correctness']
assert summary['protectedControlsVerified'] and summary['sourceAndCorrectnessHashesVerified'] and not summary['timing']
backup=load('native/prior-sdk-verification.json');assert backup['fileCount']==13 and backup['exactPathAndByteTreeMatch']
assert backup['files']=={k[6:]:v for k,v in b['current']['files'].items()if k.startswith('CPele/')}
commands=load('native/correctness-commands.json');assert len(commands)==15 and all(x['exitCode']==0 for x in commands)
prefix='logs/source-symbols-correctness-';native=raw(prefix+'native-features.log.gz').decode();counts=[int(v)for v in re.findall(r'test result: ok\. (\d+) passed; 0 failed;',native)];assert len(counts)==11 and sum(counts)==57
corpus=raw(prefix+'swift-corpus.log.gz').decode();assert 'Executed 16 tests, with 0 failures' in corpus and '12481 drawings, 16572 errors, 0 explicit budget differences' in corpus
api=load(prefix+'swift-api.json.gz');assert api['buildManifest']==consumer and api['mode']=='swift-api-correctness-only' and len(api['rows'])==168 and len(api['cases'])==21
c=load('native/c-consumers.json');assert len(c)==6 and all(x['exitCode']==0 for x in c)
for field,file in [('nativeLogSha256','native-features.log'),('swiftLogSha256','swift-corpus.log'),('apiReportSha256','swift-api.json')]:assert a['correctness'][field]==items[prefix+file+'.gz']['originalSHA256']
source=load('web/source.json.gz');package=load('web/package.json.gz');assert source['commit']==package['commit']==manifest['sourceCommit'] and source['sources']==package['sources']
assert len(package['package'])==117 and package['package']['pkg/pele_bg.wasm']=='c0ddd411aebbcd3a2293d66cf7de50f8b0c10fafdf335d7554962922ee42f07c'
node=load('logs/source-symbols-node.json.gz');assert len(node['rows'])==122 and all(not x['problems']for x in node['rows']) and node['before']==node['after']==package['package']
assert raw('web/generated-initializer-1.js')==raw('web/generated-initializer-2.js')
after=load('web/after.json.gz');webCorrect=load('web/correctness.json');delta=load('web/package-delta.json')
assert after['commit']==manifest['sourceCommit'] and after['sources']==source['sources'] and after['package']==package['package'] and after['gitInputsVerified']==len(after['sources'])==558
assert webCorrect['commit']==manifest['sourceCommit'] and webCorrect['native']==a['correctness'] and webCorrect['browserTests']==114
assert delta['staticBytesUnchanged'] and delta['delta']==['pkg/pele_bg.wasm'] and delta['wasm']==package['package']['pkg/pele_bg.wasm']
for original,h in webCorrect['artifacts'].items():assert items['logs/'+Path(original).name+'.gz']['originalSHA256']==h
assert b'114 passed' in raw('logs/source-symbols-web.log.gz')
reports=[n for n in items if n.startswith('web/reports/')];assert reports
for n in reports:
 if 'sankey-resource-' in n:assert all(not row['problems']for row in load(n)['rows'])
warm=load('warm/warm.json.gz');summary=load('paired-summary.json.gz');proof=load('warm-verification.json');execution=load('warm-execution.json')
assert warm['succeeded'] and len(warm['rows'])==len(summary['rows'])==24
assert warm['settings']['trials']==21 and warm['settings']['warmupMs']==1000 and warm['settings']['balancedThreeWayOrder'] and warm['settings']['matchedWasmBatchCounts']
assert items['warm/warm.json.gz']['originalSHA256']==summary['reportSHA256']==proof['rawSHA256']
assert warm['runtimeHashes']['candidate']==package['package']
for key,filename in [('harnessSHA256','compare-web-vm.mjs'),('casesSHA256','cases.mjs')]:assert items['executed/scripts/perf/'+filename]['originalSHA256']==warm[key]
for row,expected in zip(warm['rows'],summary['rows']):
 assert (row['case'],row['profile'],row['stage'])==(expected['case'],expected['profile'],expected['stage'])
 c,n,j=(row['samples'][key] for key in ['control','candidate','js']);assert len(c)==len(n)==len(j)==21 and row['counts']['control']==row['counts']['candidate']
 for values,key in [(c,'control'),(n,'candidate'),(j,'js')]:
  assert all(math.isfinite(x) and x>0 for x in values);assert math.isclose(statistics.median(values),row['medianMs'][key],rel_tol=1e-12);assert row['medianMs'][key]==expected['medianMs'][key]
 assert math.isclose(100*(statistics.median(b/a for a,b in zip(c,n))-1),expected['pairedMedianPercent'],abs_tol=1e-10)
 assert sum(b<a for a,b in zip(c,n))==expected['lowerPairs']
 assert math.isclose(statistics.median(n)/statistics.median(j),expected['candidateOverJS'],rel_tol=1e-12)
 assert math.isclose(statistics.median(n)/statistics.median(c),expected['candidateOverControl'],rel_tol=1e-12)
probes=[load('warm/warm-layout-continuity.json'),load('warm/warm-activation.json'),load('warm/warm-ingress-activation.json.gz')]
assert [len(x['checks'])for x in probes]==proof['probeChecks']==[8,16,12] and all(x['succeeded'] and not x['probeInsideTimer'] for x in probes)
assert not execution['firstUseRun'] and execution['verificationPassed'] and execution['warmExit']==0
assert sum(x['pairedMedianPercent']>0 for x in summary['rows'])==12
profile=load('profile/runtime/fetched/profiles/summary.json.gz');profileManifest=load('profile/manifest.json')
assert profile['succeeded'] and profile['inputsUnchanged'] and profile['modelsDisposed'] and profile['browserClosed']
assert profile['sourcePins']['candidate']==manifest['sourceCommit'] and profile['inputsBefore']==profile['inputsAfter']
assert profile['inputsBefore']['packages']==warm['runtimeHashes'] and len(profile['profiles'])==4
assert profile['inputsBefore']['harness']==items['profile/runtime/fetched/files/scripts/perf/profile-source-symbols.mjs.gz']['originalSHA256']
assert profile['inputsBefore']['cases']==items['profile/runtime/fetched/files/scripts/perf/cases.mjs']['originalSHA256']
assert len(profile['checks'])==4
for check in profile['checks']:assert check['results']['js']==check['results']['control']==check['results']['candidate']
for n,item in items.items():
 if n.startswith('profile/') and n!='profile/manifest.json' and item['source'].startswith('dist/perf-patches/parse-architecture/source-symbols-profiler/'):
  original=item['source'].split('source-symbols-profiler/',1)[1];assert item['originalSHA256']==profileManifest['files'][original],n
print(json.dumps(dict(artifacts=len(items),source=manifest['sourceCommit'],native=57,nativeBinaries=11,swift=16,drawings=12481,expectedErrors=16572,budgetDifferences=0,c=3,api=168,apiCases=21,node=122,browser=114,warmRows=24,pairedTrials=21,untimedProbes=36,parseProfiles=4),indent=2))
