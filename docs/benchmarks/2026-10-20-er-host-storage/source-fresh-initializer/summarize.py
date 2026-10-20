import json,statistics,hashlib,pathlib
root=pathlib.Path(__file__).resolve().parent
report=json.loads((root/'evidence/warm.json').read_text())
assert report['succeeded'] and len(report['rows'])==24
rows=[]
for r in report['rows']:
 c=r['samples']['control'];n=r['samples']['candidate'];j=r['samples']['js']
 assert len(c)==len(n)==len(j)==21
 ratios=[b/a for a,b in zip(c,n)]
 rows.append({'case':r['case'],'profile':r['profile'],'stage':r['stage'],'controlMs':r['medianMs']['control'],'candidateMs':r['medianMs']['candidate'],'jsMs':r['medianMs']['js'],'pairedMedianPercent':100*(statistics.median(ratios)-1),'lowerPairs':sum(b<a for a,b in zip(c,n)),'pairs':21,'medianRatioPercent':100*(r['candidateOverControl']-1),'candidateOverJS':r['candidateOverJS'],'controlOverJS':r['controlOverJS']})
groups={}
for family in ['sankey','flowchart']:
 for stage in ['parse','render-model','render-source']:
  selected=[r for r in rows if r['case'].startswith(family+'/') and r['stage']==stage]
  groups[family+'/'+stage]={'rows':len(selected),'pairedPercentRange':[min(x['pairedMedianPercent'] for x in selected),max(x['pairedMedianPercent'] for x in selected)],'medianPairedPercent':statistics.median(x['pairedMedianPercent'] for x in selected),'adverseRows':sum(x['pairedMedianPercent']>0 for x in selected),'candidateOverJSRange':[min(x['candidateOverJS'] for x in selected),max(x['candidateOverJS'] for x in selected)]}
summary={'reportSHA256':hashlib.sha256((root/'evidence/warm.json').read_bytes()).hexdigest(),'settings':report['settings'],'groups':groups,'rows':rows,'adverseRows':[r for r in rows if r['pairedMedianPercent']>0],'method':'Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.'}
summary['adverseMedianRatioRows']=[r for r in rows if r['medianRatioPercent']>0]
(root/'paired-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
lines=['# Source-layout default warm comparison','',summary['method'],'','| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |','|---|---|---|---:|---:|---:|---:|---:|---:|']
for r in rows:lines.append(f"| {r['case']} | {r['profile']} | {r['stage']} | {r['pairedMedianPercent']:+.2f}% | {r['lowerPairs']} | {r['candidateMs']:.6f} | {r['controlMs']:.6f} | {r['jsMs']:.6f} | {r['candidateOverJS']:.2f}× |")
(root/'RESULTS.md').write_text('\n'.join(lines)+'\n')
print(json.dumps(groups,indent=2))
