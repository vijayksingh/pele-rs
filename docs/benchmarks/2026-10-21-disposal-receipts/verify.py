"""Portable evidence audit. No runtime, build, browser, or benchmark execution."""
from pathlib import Path
import json,gzip,hashlib,math,statistics
root=Path(__file__).resolve().parent
manifest=json.loads((root/'artifact-manifest.json').read_text());items={x['stored']:x for x in manifest['artifacts']}
def raw(n):
 x=items[n];b=(root/n).read_bytes();assert len(b)==x['storedBytes']and hashlib.sha256(b).hexdigest()==x['storedSHA256'],n
 b=gzip.decompress(b)if x['gzip']else b
 assert len(b)==x['originalBytes']and hashlib.sha256(b).hexdigest()==x['originalSHA256'],n
 return b
def load(n):return json.loads(raw(n))
for n in items:raw(n)
s,p,a=[load('provenance/'+n+'.json.gz')for n in ['source','package','after']]
assert s['commit']==p['commit']==a['commit']==manifest['sourceCommit']
assert s['sources']==p['sources']==a['sources']and p['package']==a['package']
assert len(s['sources'])==a['gitInputsVerified']==558 and len(p['package'])==117
node=load('correctness/node.json.gz')
assert len(node['rows'])==122 and all(not x['problems']for x in node['rows'])and node['before']==node['after']==p['package']
assert b'118 passed' in raw('correctness/web.log.gz')and b'2 skipped' in raw('correctness/web.log.gz')
for phase,collected in [('red',False),('green',True)]:
 names=[n for n in items if n.startswith('correctness/'+phase+'/')and n.endswith('disposed-root-reachability.json.gz')];assert len(names)==1
 rows=load(names[0]);assert [x['stage']for x in rows]==['parsed','rendered','helper']
 assert all(x['rootCollected']==collected and x['leafId']=='A'and x['leafIndex']==0 and x['helperRetained']for x in rows)
for n in items:
 if n.startswith('correctness/browser/')and 'sankey-resource-'in n:assert all(not x['problems']for x in load(n)['rows'])
reuse=load('provenance/native-proof-reuse.json');api=load('correctness/swift-api.json.gz')
assert reuse['unchangedNativeInputs']and reuse['nativeInputCount']==370 and reuse['revision']==manifest['sourceCommit']
assert api['mode']=='swift-api-correctness-only'and len(api['rows'])==168 and len(api['cases'])==21
assert items['correctness/swift-api.json.gz']['originalSHA256']==reuse['freshSwiftAPICorrectness']['reportSHA256']
for k,v in [('revision','nativeProofSource'),('sourceInputSha256','nativeInputSHA256'),('binarySha256','consumerSHA256'),('nativeHostArchiveSha256','sdkSHA256')]:assert api['buildManifest'][k]==reuse[v]
assert p['package']['pkg/pele_bg.wasm']=='c0ddd411aebbcd3a2293d66cf7de50f8b0c10fafdf335d7554962922ee42f07c'
warm=load('warm/warm.json.gz');summary=load('paired-summary.json.gz');proof=load('warm-verification.json');execution=load('warm-execution.json');provenance=load('warm/warm-provenance.json.gz')
assert warm['succeeded']and len(warm['rows'])==len(summary['rows'])==24
assert warm['settings']['trials']==21 and warm['settings']['warmupMs']==1000 and warm['settings']['balancedThreeWayOrder']and warm['settings']['matchedWasmBatchCounts']
assert items['warm/warm.json.gz']['originalSHA256']==summary['reportSHA256']==proof['rawSHA256']=='c23fe65fd644c704e11abacdbc5906e0c52f832f2aca26efe5adc4e5c840b2db'
assert provenance['source']==proof['source']=={'control':'11f27516d98c3d89799a7fe2065eec0bd4e147cf','candidate':manifest['sourceCommit']}
assert execution['source']==manifest['sourceCommit']and execution['controlSource']==proof['source']['control']
assert provenance['runtimeHashes']==warm['runtimeHashes']and warm['runtimeHashes']['candidate']==p['package']
assert len(warm['runtimeHashes']['control'])==117
assert sorted(k for k in p['package']if p['package'][k]!=warm['runtimeHashes']['control'][k])==['full.js','graph-storage.js']
assert warm['runtimeHashes']['control']['pkg/pele_bg.wasm']==execution['wasm']==p['package']['pkg/pele_bg.wasm']
assert provenance['probeOnlyControl']['source']=='e1486055d596d4e0ac25e4c78f1d5d95a73f781a'
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
assert probes[0]['roots']['control']==provenance['probeOnlyControl']['root']and provenance['roots']['control']!=probes[0]['roots']['control']
assert not execution['firstUseExecuted']and not execution['profileExecuted']and execution['runExitCode']==execution['fetchExitCode']==execution['verificationExitCode']==0
review=load('provenance/reviewed-source-manifest.json')
assert review['sourceCommit']==manifest['sourceCommit']and all(a['sources'][x['path']]==x['sha256']for x in review['files'])
print(json.dumps(dict(artifacts=len(items),source=manifest['sourceCommit'],sourceInputs=558,packageFiles=117,node=122,browserPassed=118,browserSkipped=2,reachabilityLanes=3,apiConditions=168,apiCases=21,nativeInputsReused=370,warmRows=24,pairs=21,untimedProbes=36,firstUse=False,profile=False),indent=2))
