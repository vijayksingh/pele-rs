#!/usr/bin/env python3
"""Recompute saved evidence only. No build, benchmark, browser or network access."""
from pathlib import Path
import collections, gzip, hashlib, json, math
BASE=Path(__file__).resolve().parent
load=lambda p:json.loads(p.read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
quantile=lambda a,p:sorted(a)[max(0,math.ceil(len(a)*p)-1)]
def close(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
w=load(BASE/'comparison.json');profile=load(BASE/'profile/summary.json');audit=load(BASE/'audit.json');remote=load(BASE/'independent-vm-rehash.json');freeze=load(BASE/'source-freeze.json');native=load(BASE/'native-input-check.json')
assert w['succeeded'] and len(w['rows'])==24
assert w['runtimeHashes']==remote['runtimeHashes']
assert w['runtimeHashes']['candidate']==profile['runtimeHashes']==freeze['packageFiles']
assert native['sha256']==freeze['nativeSourceInputSha256']=='8f4fbc262e2ce2b1784823f4dd635cb6e725b0084a7a75a507c36690763b6bea'
assert len(native['files'])==351
rows=[]
for row in w['rows']:
 samples=row['samples'];assert set(samples)=={'js','control','candidate'}
 assert all(len(x)==21 for x in samples.values())
 assert row['counts']['control']==row['counts']['candidate']
 for lane,values in samples.items():
  assert all(math.isfinite(v) and v>0 for v in values)
  close(quantile(values,.5),row['medianMs'][lane]);close(quantile(values,.95),row['p95BatchMeanMs'][lane])
  assert row['warmups'][lane]['elapsedMs']>=1000
 for key,numerator,denominator in [('candidateOverControl','candidate','control'),('candidateOverJS','candidate','js'),('controlOverJS','control','js')]:close(row[key],row['medianMs'][numerator]/row['medianMs'][denominator])
 ratios=[a/c for a,c in zip(samples['candidate'],samples['control'])];median=quantile(ratios,.5)
 result={'case':row['case'],'profile':row['profile'],'stage':row['stage'],'pairs':len(ratios),'counts':row['counts'],'medianMs':row['medianMs'],'pairedCandidateOverControl':{'median':median,'mad':quantile([abs(x-median) for x in ratios],.5),'min':min(ratios),'max':max(ratios),'sampleP95':quantile(ratios,.95)},'pairedPercent':(median-1)*100,'candidateFasterPairs':sum(x<1 for x in ratios),'ratioOfMediansPercent':(row['candidateOverControl']-1)*100,'candidateOverJS':row['candidateOverJS']}
 saved=next(x for x in audit['timing']['rows'] if (x['case'],x['profile'],x['stage'])==(row['case'],row['profile'],row['stage']))
 close(result['pairedPercent'],saved['pairedPercent']);assert result['candidateFasterPairs']==saved['candidateWins'];rows.append(result)
# Reproduce profiler self percentages from raw weighted sample deltas.
profile_checks=[]
for item in profile['profiles']:
 path=BASE/'profile'/f"{item['case'].replace('/','-')}-{item['stage']}-{item['runtime']}.cpuprofile.gz";raw=gzip.decompress(path.read_bytes());data=json.loads(raw);nodes={n['id']:n for n in data['nodes']};totals=collections.defaultdict(int)
 assert len(data['samples'])==len(data['timeDeltas'])
 for nid,dt in zip(data['samples'],data['timeDeltas']):
  frame=nodes[nid]['callFrame'];totals[frame['functionName'] or frame['url'] or '(anonymous)']+=dt
 total=sum(totals.values());top=sorted(totals.items(),key=lambda x:-x[1])[:35]
 assert [n for n,_ in top]==[x['name'] for x in item['top']]
 for (_,value),saved in zip(top,item['top']):close(100*value/total,saved['percent'])
 expected=next(x for x in audit['profiles'] if (x['case'],x['stage'],x['runtime'])==(item['case'],item['stage'],item['runtime']))
 assert sha(raw)==expected['rawSha256']
 profile_checks.append({'case':item['case'],'stage':item['stage'],'runtime':item['runtime'],'rawSha256':sha(raw),'samples':len(data['samples']),'weightedSampleUs':total,'top35Verified':True})
summary={'schemaVersion':1,'sourceCommit':freeze['sourceCommit'],'comparisonSha256':sha((BASE/'comparison.json').read_bytes()),'trials':21,'rows':rows,'profileChecks':profile_checks,'interpretation':'Paired median changes are computed from aligned trials, distinct from ratios of separate medians. All controls remain included; these descriptive diagnostic samples establish neither JS parity nor absence of regressions.'}
(BASE/'paired-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
files=[]
for path in sorted(BASE.rglob('*')):
 if not path.is_file() or path.name=='evidence-index.json' or '__pycache__' in path.parts:continue
 raw=path.read_bytes();item={'path':path.relative_to(BASE).as_posix(),'bytes':len(raw),'sha256':sha(raw)}
 if path.suffix=='.gz':
  decoded=gzip.decompress(raw);item.update(uncompressedBytes=len(decoded),uncompressedSha256=sha(decoded))
 files.append(item)
index={'schemaVersion':1,'method':'SHA256 of every bundled regular file except this index; gzip members use deterministic mtime=0. No timestamp is generated here.','files':files,'totalBytes':sum(x['bytes'] for x in files)}
(BASE/'evidence-index.json').write_text(json.dumps(index,indent=2)+'\n')
print(f"Verified {len(rows)} comparison rows and {len(profile_checks)} profiles; indexed {len(files)} files ({index['totalBytes']} bytes).")
