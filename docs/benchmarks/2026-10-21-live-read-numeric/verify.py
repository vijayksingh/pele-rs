"""Portable data-only verification; no browser, build, or benchmark execution."""
import gzip, hashlib, json, math, pathlib, statistics
root=pathlib.Path(__file__).resolve().parent
manifest=json.loads((root/'artifact-manifest.json').read_text())
items={x['stored']:x for x in manifest['artifacts']}
def raw(name):
 item=items[name];data=(root/name).read_bytes()
 assert len(data)==item['storedBytes'] and hashlib.sha256(data).hexdigest()==item['storedSHA256'],name
 decoded=gzip.decompress(data) if item['gzip'] else data
 assert len(decoded)==item['originalBytes'] and hashlib.sha256(decoded).hexdigest()==item['originalSHA256'],name
 return decoded
def load(name):return json.loads(raw(name))
for name in items:raw(name)
report=load('warm/warm.json.gz');summary=load('paired-summary.json.gz');proof=load('warm-verification.json')
assert report['succeeded'] and len(report['rows'])==len(summary['rows'])==24
assert report['settings']['trials']==21 and report['settings']['warmupMs']==1000
assert report['settings']['balancedThreeWayOrder'] and report['settings']['matchedWasmBatchCounts']
assert hashlib.sha256(raw('warm/warm.json.gz')).hexdigest()==proof['rawSHA256']==summary['reportSHA256']
for row,expected in zip(report['rows'],summary['rows']):
 assert (row['case'],row['profile'],row['stage'])==(expected['case'],expected['profile'],expected['stage'])
 c,n,j=(row['samples'][key] for key in ['control','candidate','js'])
 assert len(c)==len(n)==len(j)==21 and row['counts']['control']==row['counts']['candidate']
 for values,key in [(c,'control'),(n,'candidate'),(j,'js')]:
  assert all(math.isfinite(v) and v>0 for v in values)
  assert math.isclose(statistics.median(values),row['medianMs'][key],rel_tol=1e-12)
 assert math.isclose(100*(statistics.median(b/a for a,b in zip(c,n))-1),expected['pairedMedianPercent'],abs_tol=1e-10)
 assert sum(b<a for a,b in zip(c,n))==expected['lowerPairs']
 assert math.isclose(statistics.median(n)/statistics.median(j),expected['candidateOverJS'],rel_tol=1e-12)
 assert math.isclose(100*(statistics.median(n)/statistics.median(c)-1),expected['medianRatioPercent'],abs_tol=1e-10)
probes=[load('warm/warm-layout-continuity.json'),load('warm/warm-activation.json'),load('warm/warm-ingress-activation.json.gz')]
assert [len(p['checks']) for p in probes]==proof['probeChecks']==[8,16,12]
assert all(p['succeeded'] and not p['probeInsideTimer'] for p in probes)
source=load('provenance/source.json.gz');package=load('provenance/package.json.gz');after=load('provenance/after.json.gz')
assert source['commit']==package['commit']==after['commit']==manifest['sourceCommit']
assert source['sources']==package['sources']==after['sources'] and after['gitInputsVerified']
assert package['package']==after['package']==report['runtimeHashes']['candidate']
assert len(package['package'])==117 and len(report['runtimeHashes']['control'])==113
for key,filename in [('harnessSHA256','compare-web-vm.mjs'),('casesSHA256','cases.mjs')]:
 assert hashlib.sha256(raw('executed/scripts/perf/'+filename)).hexdigest()==report[key]
node=load('correctness/node.json.gz');assert len(node['rows'])==122
assert all(not row['problems'] for row in node['rows']) and node['before']==node['after']==package['package']
assert b'60 passed' in raw('correctness/web.log.gz')
for name in items:
 if name.startswith('correctness/browser/') and 'sankey-resource-' in name:
  assert all(not row['problems'] for row in load(name)['rows'])
api=load('correctness/swift-api.json.gz');reuse=load('provenance/native-proof-reuse.json')
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only'
assert reuse['unchangedNativeInputs'] and reuse['nativeInputCount']==370
assert api['buildManifest']['revision']==reuse['nativeProofSource']
assert api['buildManifest']['sourceInputSha256']==reuse['nativeInputSHA256']
assert api['buildManifest']['binarySha256']==reuse['consumerSHA256']
assert api['buildManifest']['nativeHostArchiveSha256']==reuse['sdkSHA256']
execution=load('warm-execution.json');assert execution['failedAttempts']==0 and not execution['firstUseStarted']
assert sum(row['pairedMedianPercent']>0 for row in summary['rows'])==execution['adversePairedRows']==15
assert sum(row['medianRatioPercent']>0 for row in summary['rows'])==execution['adverseMedianRatioRows']==17
profile=load('profile/summary.json.gz');profileManifest=load('profile/manifest.json')
assert profile['succeeded'] and profile['inputsUnchanged'] and profile['modelsDisposed'] and profile['browserClosed']
assert profile['sourcePins']['candidate']==manifest['sourceCommit']
assert profile['inputsBefore']==profile['inputsAfter']
assert profile['inputsBefore']['packages']==report['runtimeHashes']
assert len(profile['profiles'])==4
for name,item in items.items():
 if name.startswith('profile/') and name!='profile/manifest.json':
  original=item['source'].split('live-read-numeric-profiler/',1)[1]
  assert item['originalSHA256']==profileManifest['files'][original],name
print(json.dumps({'verifiedArtifacts':len(items),'warmRows':24,'pairedTrialsPerRow':21,'untimedProbes':36,'nodeComparisons':122,'browserTests':60,'apiConditions':168,'source':manifest['sourceCommit'],'groups':summary['groups']},indent=2))
