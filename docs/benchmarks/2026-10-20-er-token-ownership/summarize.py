"""Recompute saved matched samples; never executes a runtime or benchmark."""
import json,math,pathlib,statistics,hashlib,sys
p=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else __file__).resolve()
if p.is_file():p=p.parent
q=lambda a:sorted(a)[(len(a)-1)//2]
def equal(a,b):assert math.isclose(a,b,rel_tol=1e-12,abs_tol=1e-12),(a,b)
def load(f):return json.loads(f.read_text())
def digest(f):return hashlib.sha256(f.read_bytes()).hexdigest()
rows=[];order={'control':0,'candidate':0};exclusions=[]
for kind,selection in [('swift','single-windows.json'),('native','native-windows.json')]:
 for event in load(p/selection):
  f=p/pathlib.Path(event['output']).name
  if not event['complete']:
   exclusions.append({**event,'includedPairs':0});continue
  d=load(f);assert d.get('completed' if kind=='swift' else 'succeeded') and d.get('completedAt');assert len(d['rows'])==1;r=d['rows'][0]
  if kind=='swift':
   assert len(r['raw'])==15
   assert all(r['warmup'][s]['elapsedNs']>=1e9 for s in ['control','candidate'])
   assert r['warmup']['control']['operations']==r['warmup']['candidate']['operations']
   values={s:[t['values'][s]['elapsedNs']/t['values'][s]['count']/1e6 for t in r['raw']] for s in ['control','candidate']}
   for t in r['raw']:
    assert set(t['order'])=={'control','candidate'};order[t['order'][0]]+=1
    assert t['values']['control']['count']==t['values']['candidate']['count']==r['count']
    assert t['values']['control']['output']==t['values']['candidate']['output']==r['before']['values']['control']['output']==r['before']['values']['candidate']['output']==r['after']['values']['control']['output']==r['after']['values']['candidate']['output']
   for s in values:
    for a,b in zip(values[s],r['statistics'][s+'Ms']):equal(a,b)
  else:
   values={s:[t['elapsedNs']/t['count']/1e6 for t in r['raw'][s]] for s in ['js','control','candidate']}
   for s in values:
    assert len(values[s])==15;assert all(t['count']==16 for t in r['raw'][s]);assert r['warmups'][s]['elapsedNs']>=1e9
    for a,b in zip(values[s],r['samples'][s]):equal(a,b)
    equal(q(values[s]),r['medians'][s])
  paired=[a/b for a,b in zip(values['candidate'],values['control'])];median=q(paired)
  if kind=='swift':equal(median,r['statistics']['pairedCandidateOverControl']['median']);assert sum(x<1 for x in paired)==r['statistics']['candidateFasterPairs']
  rows.append({'category':kind,'case':r['case'],'stage':r['stage'],'controlMs':q(values['control']),'candidateMs':q(values['candidate']),'jsMs':q(values['js']) if kind=='native' else None,'pairedPercent':(median-1)*100,'fasterPairs':sum(x<1 for x in paired),'report':f.name,'reportSHA256':digest(f)})
if (p/'web-comparison.json').exists():
 d=load(p/'web-comparison.json');assert d['succeeded'];assert d['runtimeHashes']['candidate']==load(p/'package-freeze.json')
 for r in d['rows']:
  for s in ['js','control','candidate']:assert len(r['samples'][s])==21;equal(q(r['samples'][s]),r['medianMs'][s]);assert r['warmups'][s]['elapsedMs']>=1000
  assert r['counts']['control']==r['counts']['candidate'];rat=[a/b for a,b in zip(r['samples']['candidate'],r['samples']['control'])]
  rows.append({'category':'vm','case':r['case'],'profile':r['profile'],'stage':r['stage'],'controlMs':r['medianMs']['control'],'candidateMs':r['medianMs']['candidate'],'jsMs':r['medianMs']['js'],'pairedPercent':(q(rat)-1)*100,'fasterPairs':sum(x<1 for x in rat),'report':'web-comparison.json'})
output={'rows':rows,'swiftFirstSideOrder':order,'excludedAttempts':exclusions,'scope':'Separate paired windows; no pooled or multiplied gains; raw arithmetic recomputed.'};(p/'summary.json').write_text(json.dumps(output,indent=2)+'\n');print(json.dumps({'rows':len(rows),'order':order,'exclusions':len(exclusions)}))
