# Offline only: correct HTTP .wasm classification without changing raw evidence.
from pathlib import Path
import json, hashlib, re, gzip
base=Path(__file__).resolve().parent
source=json.loads((base/'profiles/summary.json').read_text())
rows=[]
for report in source['profiles']:
    path=base/'profiles'/(report['label']+'.cpuprofile')
    raw=gzip.decompress(path.with_suffix(path.suffix+'.gz').read_bytes())
    assert hashlib.sha256(raw).hexdigest()==report['profileSHA256']
    profile=json.loads(raw);nodes={n['id']:n for n in profile['nodes']}
    parents={child:n['id'] for n in profile['nodes'] for child in n.get('children',[])}
    groups={name:re.compile(pattern) for name,pattern in report['groupPatterns'].items()}
    categories={name:0 for name in ['wasm','javascript','gc','engine-or-profiler']}
    union=0;parser=0;total=sum(profile['timeDeltas'])
    for leaf,weight in zip(profile['samples'],profile['timeDeltas']):
        frame=nodes[leaf]['callFrame'];url=frame['url'];name=frame['functionName']
        category='gc' if name=='(garbage collector)' else 'wasm' if url.startswith('wasm://') or re.search(r'\.wasm(?:$|[?#])',url) else 'javascript' if re.match(r'https?:',url) else 'engine-or-profiler'
        categories[category]+=weight
        matching=set();current=leaf
        while current is not None:
            frame=nodes[current]['callFrame'];text=frame['functionName']+' '+frame['url']
            matching.update(name for name,pattern in groups.items() if pattern.search(text))
            current=parents.get(current)
        if matching.intersection(['native-admission','captured-native-adapter','captured-js-imports']):union+=weight
        if 'sankey-raw-parser' in matching:parser+=weight
    rows.append({'stage':report['stage'],'runtime':report['runtime'],'profileSHA256':report['profileSHA256'],'weightedUs':total,'sampleCount':len(profile['samples']),'selfCategories':{name:{'weightedUs':us,'percent':100*us/total} for name,us in categories.items()},'admissionAdapterImportsUnionPercent':100*union/total,'rawParserInclusivePercent':100*parser/total,'architectureGroups':report['architectureGroups'],'topSelf':report['topSelf'][:20],'topInclusive':report['topInclusive'][:20]})
assert source['runtimeHashes']['control']==source['runtimeHashes']['candidate']
assert len(source['profiles'])==6 and source['succeeded']
assert all(all(v['actual']==('source' if v['runtime']=='candidate' else None) for v in c['backends']) for c in source['checks'])
out={'sourceSummarySHA256':hashlib.sha256((base/'profiles/summary.json').read_bytes()).hexdigest(),'executedHarnessSHA256':source['harnessSHA256'],'wasmSHA256':source['wasmSHA256'],'packageFiles':len(source['runtimeHashes']['candidate']),'browser':source['browser'],'correction':'Original summary classified HTTP-served .wasm frames as JavaScript. This offline result recognizes both HTTP .wasm and wasm://. Shared compiled WASM frame URLs can point to control even during candidate execution; runtime identity comes from the profiled operation, not frame URL. Raw profiles and original summary are unchanged.','phaseCallCounts':None,'rows':rows}
(base/'ATTRIBUTION.json').write_text(json.dumps(out,indent=2)+'\n')
for row in rows:
 print(row['stage'],row['runtime'],{k:round(v['percent'],2) for k,v in row['selfCategories'].items()},'union',round(row['admissionAdapterImportsUnionPercent'],2),'parser',round(row['rawParserInclusivePercent'],2))
