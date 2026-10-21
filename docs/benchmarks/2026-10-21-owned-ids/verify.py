"""Data-only audit of a rejected experiment; no runtime execution."""
from pathlib import Path
import json,gzip,hashlib,math,statistics
root=Path(__file__).resolve().parent;manifest=json.loads((root/'artifact-manifest.json').read_text());items={x['stored']:x for x in manifest['artifacts']}
def raw(n):
 x=items[n];b=(root/n).read_bytes();assert len(b)==x['storedBytes'] and hashlib.sha256(b).hexdigest()==x['storedSHA256'],n;b=gzip.decompress(b)if x['gzip']else b;assert len(b)==x['originalBytes'] and hashlib.sha256(b).hexdigest()==x['originalSHA256'],n;return b
def load(n):return json.loads(raw(n))
for n in items:raw(n)
s,p,a=[load('provenance/'+n+'.json.gz')for n in ['source','package','after']]
assert s['commit']==p['commit']==a['commit']==manifest['sourceCommit']
assert s['sources']==p['sources']==a['sources'] and p['package']==a['package'] and a['gitInputsVerified']==558
warm=load('warm/warm.json.gz');summary=load('paired-summary.json.gz');proof=load('warm-verification.json');execution=load('warm-execution.json');provenance=load('warm/warm-provenance.json.gz')
assert warm['succeeded'] and len(warm['rows'])==len(summary['rows'])==24
assert warm['settings']['trials']==21 and warm['settings']['warmupMs']==1000 and warm['settings']['balancedThreeWayOrder'] and warm['settings']['matchedWasmBatchCounts']
assert items['warm/warm.json.gz']['originalSHA256']==summary['reportSHA256']==proof['rawSHA256']=='f96c82028dd4c46adce3459c76980c4846b8eced01527cfc9cccc4fe799cc6a3'
assert provenance['source']==proof['source']==execution['source']=={'control':'11f27516d98c3d89799a7fe2065eec0bd4e147cf','candidate':manifest['sourceCommit']}
assert provenance['runtimeHashes']==warm['runtimeHashes'] and warm['runtimeHashes']['candidate']==p['package']
assert provenance['probeOnlyControl']['source']==execution['probeOnlyControl']=='e1486055d596d4e0ac25e4c78f1d5d95a73f781a'
assert len(warm['runtimeHashes']['control'])==len(p['package'])==117
assert sorted(k for k in p['package']if p['package'][k]!=warm['runtimeHashes']['control'][k])==['full.js','graph.js']
delta=load('provenance/package-delta.json');assert delta['delta']==['full.js','graph.js'] and delta['unchangedGeneratedPackage']
assert p['package']['pkg/pele_bg.wasm']==warm['runtimeHashes']['control']['pkg/pele_bg.wasm']==delta['unchangedWASM']
for key,filename in [('harnessSHA256','compare-web-vm.mjs'),('casesSHA256','cases.mjs')]:assert items['executed/scripts/perf/'+filename]['originalSHA256']==warm[key]
for row,e in zip(warm['rows'],summary['rows']):
 assert (row['case'],row['profile'],row['stage'])==(e['case'],e['profile'],e['stage'])
 c,n,j=(row['samples'][key]for key in ['control','candidate','js']);assert len(c)==len(n)==len(j)==21 and row['counts']['control']==row['counts']['candidate']
 for vs,key in [(c,'control'),(n,'candidate'),(j,'js')]:assert all(math.isfinite(v)and v>0 for v in vs)and math.isclose(statistics.median(vs),row['medianMs'][key],rel_tol=1e-12)
 assert math.isclose(100*(statistics.median(b/a for a,b in zip(c,n))-1),e['pairedMedianPercent'],abs_tol=1e-10)and sum(b<a for a,b in zip(c,n))==e['lowerPairs']
 assert math.isclose(statistics.median(n)/statistics.median(j),e['candidateOverJS'],rel_tol=1e-12)and math.isclose(statistics.median(n)/statistics.median(c),e['candidateOverControl'],rel_tol=1e-12)
probes=[load('warm/warm-layout-continuity.json'),load('warm/warm-activation.json'),load('warm/warm-ingress-activation.json.gz')]
assert [len(x['checks'])for x in probes]==proof['probeChecks']==[8,16,12]
assert all(x['succeeded']and not x['probeInsideTimer']for x in probes)
assert probes[0]['roots']['control']==provenance['probeOnlyControl']['root'] and provenance['roots']['control']!=probes[0]['roots']['control']
assert not execution['firstUseRun']and not execution['profileRun']and execution['verificationPassed']and execution['warmExit']==0
node=load('correctness/node.json.gz');assert len(node['rows'])==122 and all(not x['problems']for x in node['rows'])and node['before']==node['after']==p['package']
assert b'117 passed' in raw('correctness/web.log.gz')
for n in items:
 if n.startswith('correctness/browser/')and 'sankey-resource-'in n:assert all(not x['problems']for x in load(n)['rows'])
reuse=load('provenance/native-proof-reuse.json');api=load('correctness/swift-api.json.gz');assert reuse['unchangedNativeInputs']and reuse['nativeInputCount']==370
assert len(api['rows'])==168 and len(api['cases'])==21 and api['mode']=='swift-api-correctness-only'
for k,v in [('revision','nativeProofSource'),('sourceInputSha256','nativeInputSHA256'),('binarySha256','consumerSHA256'),('nativeHostArchiveSha256','sdkSHA256')]:assert api['buildManifest'][k]==reuse[v]
revert=load('provenance/revert-proof.json');assert revert['sourceInputsVerified']==558 and revert['restoredRuntimeSource']==proof['source']['control']
restored=load('provenance/restoration-complete.json.gz')
assert restored['sourceCommit']==revert['sourceCommit'] and restored['restoredRuntimeSource']==revert['restoredRuntimeSource']
assert restored['sourceInputsVerified']==558 and restored['packageFilesVerified']==117 and restored['packageExactlyRestored']
assert restored['packageHashes']==warm['runtimeHashes']['control'] and restored['wasm']==delta['unchangedWASM']
print(json.dumps(dict(artifacts=len(items),rejectedCandidate=manifest['sourceCommit'],timedControl=proof['source']['control'],probeOnlyControl=execution['probeOnlyControl'],rows=24,trials=21,probes=36,node=122,browser=117,api=168,nativeInputsReused=370,revertedSource=revert['sourceCommit']),indent=2))
