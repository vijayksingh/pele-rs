"""Portable data-only adaptation; exact original analyzer is archived."""
"""Offline exact weighted attribution. No runtime or profile mutation."""
from pathlib import Path
import json,re,hashlib,collections,math,gzip
root=Path(__file__).parent;data=root/'profile/runtime/fetched/profiles';s=json.loads(gzip.decompress((data/'summary.json.gz').read_bytes()))
assert s['succeeded'] and s['inputsUnchanged'] and s['browserClosed'] and s['modelsDisposed'] and len(s['profiles'])==4
assert s['inputsBefore']==s['inputsAfter']
rx=re.compile
rules={
 'gc':rx(r'^\(garbage collector\)$'),
 'native-allocator':rx(r'^(?:<)?(?:dlmalloc::|__rust_(?:alloc|dealloc)|malloc$|realloc$|free$)'),
 'native-grammar':rx(r'^(?:<)?pele::compat::diagrams::sankey::parser::|^<pele::compat::diagrams::sankey::Sankey as [^>]+>::parse(?:$|\s)'),
 'native-symbol-builder':rx(r'^<pele::compat::diagrams::sankey::indexed::SourceSymbols(?:>| as pele::compat::diagrams::sankey::parser::Target>)::'),
 'native-admission':rx(r'^(?:<)?pele::compat::public::(?:captured_admission::|captured_transport::Counter|graph_read::validate_|engine::Registration>::(?:validate_read|compiled_schema)|schema::ModelSchema(?:View)?>::validate)|^<&mut pele::compat::public::captured_admission::|^<pele::compat::diagrams::sankey::source_store::InitialRead as '),
 'native-layout-export':rx(r'^<pele::compat::diagrams::sankey::source_store::SourceCanonical>::(?:layout_packet|recipe)|^pele::compat::diagrams::sankey::source_store::(?:column|group|expr|atom9)|^pele_wasm::source_layout::'),
 'native-binary-writer':rx(r'^<pele::compat::public::binary::Writer>::'),
 'native-source-entry':rx(r'^<pele::compat::public::engine::Codec<pele::compat::diagrams::sankey::Sankey> as pele::compat::public::engine::Erased>::try_parse_public$'),
 'native-logical-read':rx(r'^<pele::compat::diagrams::sankey::source_store::SourceCanonical(?:>| as [^>]+>)::(?:header|base_header|base_property|property_batch|properties|map_batch|map_entries|string_length|string_lengths|string_chunks)'),
 'native-disposal':rx(r'^__wbg_mutabledocument_free$|^core::ptr::drop_glue::<pele::compat::public::engine::PublicDocument'),
 'js-publication':rx(r'^(?:publish|tryInitializeSourceLayout|eligible|reconcileGraph)$'),
 'js-layout-decode':rx(r'^(?:decodeSourceLayout|decodeLayoutOutcome|readLayoutBase|readCurrentStrings|readCatalog|readText|readOverrides|readGroups|readColumns|readColumn|readExpr)$'),
 'js-disposal':rx(r'^(?:dispose|free|abandon|disposeModel)$'),
 'wasm-output-copy':rx(r'^__wbg_new_from_slice_'),
}
def name(f):return re.sub(r'\[[0-9a-f]+\]','',f['functionName'])
def lang(f):
 if f['functionName']=='(garbage collector)':return 'gc'
 if f['url'].startswith('wasm://') or re.search(r'\.wasm(?:$|[?#])',f['url']):return 'wasm'
 return 'javascript' if f['url'].startswith('http') else 'engine-or-profiler'
def groups(f):
 n=name(f);l=lang(f);out=set()
 for k,p in rules.items():
  if k.startswith('native-') and l!='wasm':continue
  if k.startswith('js-') and l!='javascript':continue
  if p.search(n):out.add(k)
 return out
unions={'native-grammar-symbols':['native-grammar','native-symbol-builder'],'native-admission-export':['native-admission','native-layout-export','native-binary-writer'],'js-decode-publication':['js-layout-decode','js-publication'],'disposal':['native-disposal','js-disposal']}
out={'sourcePins':s['sourcePins'],'inputs':s['inputsBefore'],'rules':{k:v.pattern for k,v in rules.items()},'unions':unions,'scope':'Disjoint self languages/priority roles; each ancestor/group/union deduplicates samples, groups overlap. Anchors classify actual callable owners, not SourceSymbols or IndexedSankey in generic argument lists. Inline work may be charged to source-entry. No allocation/call counts or latency inference.','profiles':[]}
for e in s['profiles']:
 raw=gzip.decompress((data/(e['label']+'.cpuprofile.gz')).read_bytes());assert hashlib.sha256(raw).hexdigest()==e['profileSHA256'];p=json.loads(raw);assert len(p['samples'])==len(p['timeDeltas']) and all(math.isfinite(w) and w>=0 for w in p['timeDeltas']);nodes={n['id']:n for n in p['nodes']};parents={}
 for n in p['nodes']:
  for c in n.get('children',[]):assert c not in parents;parents[c]=n['id']
 total=sum(p['timeDeltas']);langs=collections.Counter();roles=collections.Counter();sg=collections.Counter();ag=collections.Counter();uc=collections.Counter();sr=collections.Counter();ir=collections.Counter();frames={}
 for sid,w in zip(p['samples'],p['timeDeltas']):
  n=nodes[sid];f=n['callFrame'];g=groups(f);langs[lang(f)]+=w;roles[next((k for k in rules if k in g),lang(f)+'-other')]+=w;sg.update({k:w for k in g});key=json.dumps(f,sort_keys=True);sr[key]+=w;seen=set();seenids=set();allg=set()
  while n:
   assert n['id'] not in seenids;seenids.add(n['id']);f=n['callFrame'];key=json.dumps(f,sort_keys=True);frames[key]=f
   if key not in seen:ir[key]+=w;seen.add(key)
   allg.update(groups(f));n=nodes.get(parents.get(n['id']))
  ag.update({k:w for k in allg});uc.update({k:w for k,v in unions.items() if allg.intersection(v)})
 assert total>0 and sum(langs.values())==sum(roles.values())==total
 pct=lambda d:{k:100*v/total for k,v in d.most_common()}
 x={'label':e['label'],'case':e['case'],'runtime':e['runtime'],'weightedUs':total,'sampleCount':len(p['samples']),'operations':e['operations'],'disjointSelfLanguages':pct(langs),'disjointSelfPriorityRoles':pct(roles),'groups':{k:{'selfPercent':100*sg[k]/total,'inclusivePercent':100*ag[k]/total} for k in rules},'deduplicatedAncestorUnions':pct(uc),'frames':[{'frame':frames[k],'selfPercent':100*sr[k]/total,'inclusivePercent':100*v/total} for k,v in ir.most_common()]};out['profiles'].append(x)
assert out==json.loads(gzip.decompress((root/'profile/runtime/attribution.json.gz').read_bytes()))
print('Verified four exact anchored parse-profile attributions.')
for x in out['profiles']:
 print(x['label'], 'languages',x['disjointSelfLanguages']);print('groups',{k:{t:round(v,3) for t,v in vals.items()} for k,vals in x['groups'].items() if vals['inclusivePercent']>.2});print('unions',x['deduplicatedAncestorUnions'])
