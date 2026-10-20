# Data-only arithmetic check. No browser, subprocess, network or package mutation.
from pathlib import Path
import json, statistics, math
base=Path(__file__).resolve().parent
raw=json.loads((base/'comparison.json').read_text());paired=json.loads((base/'paired-summary.json').read_text())['rows']
assert raw['succeeded'] and len(raw['rows'])==18 and raw['settings']['trials']==21
assert raw['runtimeHashes']['control']==raw['runtimeHashes']['candidate'] and len(raw['runtimeHashes']['candidate'])==112
assert raw['capturedBackend']['candidateEnabled'] and not raw['capturedBackend']['controlEnabled'] and not raw['capturedBackend']['probeInsideTimer']
seen=set();computed=[]
for row,summary in zip(raw['rows'],paired):
 key=(row['case'],row['profile'],row['stage']);assert key not in seen;seen.add(key)
 assert key==(summary['case'],summary['profile'],summary['stage'])
 samples=row['samples'];assert all(len(samples[n])==21 for n in ['js','control','candidate'])
 assert row['counts']['control']==row['counts']['candidate']
 med={n:statistics.median(samples[n]) for n in samples}
 for n in med:assert math.isclose(med[n],row['medianMs'][n],rel_tol=1e-12)
 ratio=med['candidate']/med['control'];over_js=med['candidate']/med['js']
 pairs=[100*(b/a-1) for a,b in zip(samples['control'],samples['candidate'])]
 delta=statistics.median(pairs);wins=sum(b<a for a,b in zip(samples['control'],samples['candidate']))
 assert math.isclose(delta,summary['pairedPercent'],abs_tol=1e-9)
 assert wins==summary['lowerPairs'] and math.isclose(ratio,row['candidateOverControl'],rel_tol=1e-12) and math.isclose(over_js,row['candidateOverJS'],rel_tol=1e-12)
 computed.append({'case':key[0],'profile':key[1],'stage':key[2],'pairedPercent':delta,'lowerPairs':wins,'overControl':ratio,'overJS':over_js})
for case in raw['cases']:
 assert len(case['backendChecks'])==16
 for check in case['backendChecks']:assert check['actual']==check['expected']==('source' if check['runtime']=='candidate' else None)
for stage in ['parse','render-model','render-source']:
 rows=[row for row in computed if row['stage']==stage];assert len(rows)==6
 print(stage,'paired% range',min(r['pairedPercent'] for r in rows),max(r['pairedPercent'] for r in rows),'candidate/JS range',min(r['overJS'] for r in rows),max(r['overJS'] for r in rows))
print('Verified all 18 rows, all 21-pair samples, matched batch counts, complete112-file identity, and48 untimed backend checks.')
