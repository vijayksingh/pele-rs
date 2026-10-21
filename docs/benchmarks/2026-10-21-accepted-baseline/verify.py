"""Portable evidence verification; no build/browser/benchmark execution."""
from pathlib import Path
import json,gzip,hashlib,math,statistics
root=Path(__file__).resolve().parent
manifest=json.loads((root/'artifact-manifest.json').read_text());items={x['stored']:x for x in manifest['artifacts']}
def raw(name):
 item=items[name];b=(root/name).read_bytes();assert len(b)==item['storedBytes'] and hashlib.sha256(b).hexdigest()==item['storedSHA256'],name
 b=gzip.decompress(b) if item['gzip'] else b
 assert len(b)==item['originalBytes'] and hashlib.sha256(b).hexdigest()==item['originalSHA256'],name
 return b
def load(name):return json.loads(raw(name))
for n in items:raw(n)
s,p,a=[load('provenance/'+n+'.json.gz') for n in ['source','package','after']]
before=load('provenance/package-before-commit.json.gz')
assert s['sources']==before['sources']==p['sources']==a['sources'] and before['package']==p['package']==a['package']
assert p['commit']==a['commit']
assert p['priorPackageManifestSHA256']==items['provenance/package-before-commit.json.gz']['originalSHA256']
assert p['afterManifestSHA256']==items['provenance/after.json.gz']['originalSHA256']
assert a['commit']==manifest['sourceCommit'] and a['gitInputsVerified']==len(a['sources'])==558
review=load('provenance/reviewed-source-manifest.json');assert review['sourceCommit']==a['commit']
for f in review['files']:assert a['sources'][f['path']]==f['sha256']
delta=load('provenance/package-delta.json');assert delta['delta']==['full.js','graph-storage.js','graph.js']
assert len(a['package'])==delta['packageFiles']==117 and a['package']['pkg/pele_bg.wasm']==delta['unchangedWASM']
correct=load('provenance/correctness.json');assert not correct['timing']
for original,sha in correct['artifacts'].items():
 n='correctness/'+Path(original).name.removeprefix('accepted-baseline-')+'.gz';assert items[n]['originalSHA256']==sha
node=load('correctness/node.json.gz');assert len(node['rows'])==correct['nodeCases']==122 and all(not row['problems'] for row in node['rows'])
assert node['before']==node['after']==a['package']
assert b'111 passed' in raw('correctness/web.log.gz') and b'3 passed' in raw('correctness/feature.log.gz')
assert correct['browserConsolidated']+correct['browserFeature']==114
feature=[n for n in items if n.startswith('correctness/feature/')]
assert len(feature)==3
for n in feature:
 report=load(n);assert report['restored'] and len(report['rows'])==2
 for row in report['rows']:
  calls=row['unchangedCalls'];assert calls['apply_binary']==calls['apply_source_input']==0 and calls['finish_detached_render']==3
  assert calls['prepare_detached_render_raw' if row['raw'] else 'prepare_detached_render_numbers']==3
  for key in ['callbackOriginal','observationOriginal','orphanAlias','oracleOrphanAlias','factoryAlias','factoryOriginal']:assert row[key]
  assert row['failedApplyCalls']==0 and row['partialError']==row['oraclePartialError']
  assert row['partialGraph']==row['oraclePartialGraph'] and row['finalGraph']==row['oracleFinalGraph']
  for stage in row['stages']:assert stage['value']==stage['expected'] and stage['graph']==stage['oracleGraph']
for n in items:
 if n.startswith('correctness/web/') and 'sankey-resource-' in n:
  assert all(not row['problems'] for row in load(n)['rows'])
red=[load(n) for n in items if n.startswith('correctness/red-chrome/')]
assert len(red)==1
assert all(row['unchangedCalls']['apply_source_input']==2 and row['unchangedCalls']['finish_detached_render']==3 for row in red[0]['rows'])
assert b"Executable doesn't exist" in raw('correctness/red.log.gz')
assert b'1 failed' in raw('correctness/red-chrome.log.gz')
api=load('correctness/swift-api.json.gz');reuse=load('provenance/native-proof-reuse.json')
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only'
assert reuse['revision']==a['commit'] and reuse['unchangedNativeInputs'] and reuse['nativeInputCount']==370
for k,v in [('revision','nativeProofSource'),('sourceInputSha256','nativeInputSHA256'),('binarySha256','consumerSHA256'),('nativeHostArchiveSha256','sdkSHA256')]:assert api['buildManifest'][k]==reuse[v]
assert raw('generator/generated-initializer-1.js')==raw('generator/generated-initializer-2.js')
warm=load('warm/warm.json.gz');summary=load('paired-summary.json.gz');proof=load('warm-verification.json');execution=load('warm-execution.json')
assert warm['succeeded'] and len(warm['rows'])==len(summary['rows'])==24
assert warm['settings']['trials']==21 and warm['settings']['warmupMs']==1000
assert warm['settings']['balancedThreeWayOrder'] and warm['settings']['matchedWasmBatchCounts']
assert items['warm/warm.json.gz']['originalSHA256']==summary['reportSHA256']==proof['rawSHA256']==execution['rawSHA256']
assert warm['runtimeHashes']['candidate']==a['package']
for key,filename in [('harnessSHA256','compare-web-vm.mjs'),('casesSHA256','cases.mjs')]:
 assert hashlib.sha256(raw('executed/scripts/perf/'+filename)).hexdigest()==warm[key]
for row,expected in zip(warm['rows'],summary['rows']):
 assert (row['case'],row['profile'],row['stage'])==(expected['case'],expected['profile'],expected['stage'])
 c,n,j=(row['samples'][key] for key in ['control','candidate','js'])
 assert len(c)==len(n)==len(j)==21 and row['counts']['control']==row['counts']['candidate']
 for values,key in [(c,'control'),(n,'candidate'),(j,'js')]:
  assert all(math.isfinite(x) and x>0 for x in values)
  assert math.isclose(statistics.median(values),row['medianMs'][key],rel_tol=1e-12)
 assert math.isclose(100*(statistics.median(b/a for a,b in zip(c,n))-1),expected['pairedMedianPercent'],abs_tol=1e-10)
 assert sum(b<a for a,b in zip(c,n))==expected['lowerPairs']
 assert math.isclose(statistics.median(n)/statistics.median(j),expected['candidateOverJS'],rel_tol=1e-12)
 assert math.isclose(100*(statistics.median(n)/statistics.median(c)-1),expected['medianRatioPercent'],abs_tol=1e-10)
probes=[load('warm/warm-layout-continuity.json'),load('warm/warm-activation.json'),load('warm/warm-ingress-activation.json.gz')]
assert [len(x['checks']) for x in probes]==proof['probeChecks']==[8,16,12]
assert all(x['succeeded'] and not x['probeInsideTimer'] for x in probes)
for row in probes[0]['checks']:
 if row['runtime']=='candidate' and row['modelType']=='sankey':
  assert row['applies']==[] and row['prepareCalls']==1 and len(row['jobs'])==1
  assert row['jobs'][0]['runs']==row['jobs'][0]['finishes']==row['jobs'][0]['materialized']==1
  assert row['update']['layoutId']==row['metadata']['layoutId']
assert sum(row['pairedMedianPercent']>0 for row in summary['rows'])==execution['adversePairedRows']==19
assert sum(row['medianRatioPercent']>0 for row in summary['rows'])==execution['adverseMedianRatioRows']==19
assert not execution['firstUseStarted'] and execution['runnerExit']==execution['verificationExit']==0
pin=load('pin-attempt1.json');assert not pin['browserLaunched'] and not pin['transferred'] and pin['exit']==1
assert pin['files']['package.json']==items['provenance/package-before-commit.json.gz']['originalSHA256']
profile=load('profile/runtime-attempt2/profiles-attempt2/summary.json.gz');profileManifest=load('profile/manifest.json')
assert profile['succeeded'] and profile['inputsUnchanged'] and profile['modelsDisposed'] and profile['browserClosed']
assert profile['sourcePins']['candidate']==manifest['sourceCommit'] and profile['inputsBefore']==profile['inputsAfter']
assert profile['inputsBefore']['packages']==warm['runtimeHashes'] and len(profile['profiles'])==4
for n,item in items.items():
 if n.startswith('profile/') and n!='profile/manifest.json':
  original=item['source'].split('accepted-baseline-profiler/',1)[1]
  assert item['originalSHA256']==profileManifest['files'][original],n
failure=load('profile/runtime/failure-manifest.json')
assert not failure['candidateWindowRan'] and failure['browserClosed'] and failure['modelsDisposed'] and failure['inputsUnchanged']
rejected=load('profile/runtime/profiles/sankey-medium-fixed-render-model-control.cpuprofile.gz')
assert [[i,x] for i,x in enumerate(rejected['timeDeltas']) if x<0]==failure['negativeDeltas']==[[1158,-224],[1189,-409]]
print(json.dumps(dict(artifacts=len(items),source=a['commit'],sourceInputs=558,packageFiles=117,nodeComparisons=122,browserTests=114,swiftAPI=168,swiftCases=21,nativeInputsReused=370,correctnessFailedAttemptsPreserved=2,pinFailurePreserved=True,warmRows=24,trialsPerRow=21,untimedProbes=36),indent=2))
