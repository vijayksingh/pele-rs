"""Read saved dense source profile only; no browser, timing or source mutation."""
from pathlib import Path
import collections,hashlib,json
base=Path(__file__).resolve().parent
source=base.parent/'er-dense-array-first/profile/er-medium-render-source-wasm.cpuprofile'
raw=source.read_bytes();p=json.loads(raw);nodes={n['id']:n for n in p['nodes']};parents={c:n['id'] for n in p['nodes'] for c in n.get('children',[])};weights=collections.Counter()
for nid,us in zip(p['samples'],p['timeDeltas']):weights[nid]+=us
buckets=collections.Counter();details=[]
for nid,us in weights.items():
 f=nodes[nid]['callFrame'];name=f['functionName'];chain=[];at=nid
 while at in parents:at=parents[at];chain.append(nodes[at]['callFrame']['functionName'])
 category=None
 if name=='result' and f.get('url','').endswith('/full.js'):category='facade_result_self'
 elif name=='decode':category='resource_text_decode_self' if any('BrowserResources' in x and 'text_width' in x for x in chain) else 'returned_json_decode_self' if any(x=='render_numbers_with_resources' for x in chain) else 'other_decode_self'
 elif name=='decodeText':category='resource_decodeText_self' if any('BrowserResources' in x and 'text_width' in x for x in chain) else 'returned_decodeText_self' if any(x=='render_numbers_with_resources' for x in chain) else 'other_decodeText_self'
 elif 'render_numbers::Bounded' in name and 'serialize_str' in name:category='bounded_serialize_str_self'
 if category:buckets[category]+=us;details.append(dict(category=category,function=name,weightedSelfUs=us,ancestors=chain[:8]))
total=sum(weights.values());result=dict(source=str(source),sourceSha256=hashlib.sha256(raw).hexdigest(),method='Sum leaf/self timeDeltas by exact frames; classify decode with full ancestor path; categories disjoint, no inclusive sum',samples=len(p['samples']),weightedSampleUs=total,categories={k:dict(weightedSelfUs=v,percent=100*v/total) for k,v in buckets.items()},details=details,interpretation=['Most decode self belongs to text_width callback strings, outside packet scope.','Reference counting retains canonical serde traversal; serialize_str self cannot be assumed removed.','Direct SVG still crosses WASM as UTF8 and is decoded; returned-JSON decode is not fully removable.','Single ER medium source profile is an opportunity locator, not a speedup prediction or all-family ceiling.'])
(base/'profile-attribution.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result['categories'],indent=2))
