#!/usr/bin/env python3
"""Offline weighted attribution; no runtime imports or execution."""
from pathlib import Path
import json,re,hashlib,collections
root=Path(__file__).parent; data=root/'runtime/cpu-profiles'; summary=json.loads((data/'summary.json').read_text())
assert summary['succeeded'] and summary['inputsUnchanged'] and summary['browserClosed'] and summary['modelsDisposed']
assert summary['inputsBefore']==summary['inputsAfter'] and len(summary['profiles'])==9
rx=lambda p:re.compile(p)
rules={
 'native-validation-count':rx(r'captured_admission::|captured_transport::|graph_read::(?:validate_|GraphReadView>::canonical_binary_size)|Registration>::validate_read'),
 'native-source-entry':rx(r'engine::Erased>::try_parse_public$'),
 'native-structural-stage':rx(r'graph_read::validate_(?:structure|initial_structure)|captured_admission::validate(?:_initial|_with)?(?:::<|<)'),
 'native-schema-stage':rx(r'engine::Registration>::validate_read|captured_admission::validate_schema_ids'),
 'native-canonical-count-stage':rx(r'graph_read::GraphReadView>::canonical_binary_size|captured_transport::canonical_frame_bytes'),
 'native-schema-definition':rx(r'schema::ModelSchema(?:View)?>::validate|Registration>::compiled_schema'),
 'native-layout-encoding':rx(r'source_store::SourceCanonical>::(?:layout_packet|recipe)|source_layout::prepare_graph'),
 'native-source-catalog':rx(r'source_store::SourceCanonical>::(?:add_text|add_key)'),
 'native-parser':rx(r'^(?:<)?pele::compat::diagrams::sankey::(?:parser|indexed)::|sankey::Sankey as .*DiagramModule>::parse'),
 'native-render':rx(r'^(?:<)?pele::compat::diagrams::sankey::(?:render|layout|measure)::|compat::svg::'),
 'native-json-count':rx(r'render_numbers::Counted|render_packet::prepare'),
 'native-json-other':rx(r'serde_json::|serde_core::|serde::'),
 'native-logical-read':rx(r'source_store::SourceCanonical.*>::(?:property_batch|properties|map_batch|map_entries|base_property|base_header|header|string_length|string_lengths)'),
 'native-schema-metadata-boundary':rx(r'^(?:mutabledocument_)?schema(?:_cache_key)?(?:$|\s)|^registration$|^metadata$'),
 'js-fresh-initializer':rx(r'^fresh-initializer$'),
 'js-publication':rx(r'^reconcileGraph$'),
 'js-matching':rx(r'^matchesGraph$'),
 'js-observation':rx(r'^snapshotGraph$'),
 'js-layout-decode':rx(r'^(?:decodeSourceLayout|decodeLayoutOutcome|readLayoutBase|readCurrentStrings|readText|readOverrides|readGroups|readColumn|readExpr)$'),
 'js-layout-access':rx(r'^layout-access$'),
 'js-binary':rx(r'^(?:encodeGraphBinary|decodeGraphBinary)$'),
 'allocator':rx(r'\b(?:malloc|calloc|realloc|free|__rust_alloc|__rust_dealloc)\b|dlmalloc::'),
 'gc':rx(r'^\(garbage collector\)$')}
def norm(f):return re.sub(r'\[[0-9a-f]+\]','',f['functionName'])
def lang(f):
 if f['functionName']=='(garbage collector)':return 'gc'
 if f['url'].startswith('wasm://') or re.search(r'\.wasm(?:$|[?#])',f['url']):return 'wasm'
 return 'javascript' if f['url'].startswith('http') else 'engine-or-profiler'
def groups(f):
 n=norm(f); found={k for k,r in rules.items() if r.search(n)}
 if '/source-initializer.js' in f['url']:found.add('js-fresh-initializer')
 if '/layout-storage.js' in f['url'] and n in {'bind','key','propertyWords','itemWords','writeAtom','selectProperty','selectItem','evaluateSemantic','property','item','group','evaluate','propertySlots','propertyCount','readProperty','atom','atomTag','atomPayload','itemAtoms','kind','aux','propertyKey','propertyFlags','atomNumberBits','itemAtomSlot','prototypeId','nodeKind','nodeCount','arrayLength','stringValueAt','stringCount'}:found.add('js-layout-access')
 return found
unions={'native-validation-count':['native-validation-count'], 'native-initial-admission':['native-structural-stage','native-schema-stage','native-canonical-count-stage'], 'js-publish-match-observe':['js-publication','js-matching','js-observation'], 'js-layout-decode-access':['js-layout-decode','js-layout-access'], 'js-graph-boundary':['js-fresh-initializer','js-publication','js-matching','js-observation','js-layout-decode','js-layout-access','js-binary']}
out={'sourcePins':summary['sourcePins'],'inputs':summary['inputsBefore'],'verifiedProfiles':9,'scope':'Weighted timeDeltas. Self languages and priority roles disjoint. Ancestors and unions deduplicate each sample. Groups overlap. Not latency, allocations or invocation counts. Native parser excludes render generic type arguments; JSON excludes render wrapper names. Inlined phases may lack frames. decode with no URL remains engine-or-profiler, not invented JS or Rust.','rules':{k:v.pattern for k,v in rules.items()},'unions':unions,'profiles':[]}
for entry in summary['profiles']:
 raw=(data/(entry['label']+'.cpuprofile')).read_bytes();assert hashlib.sha256(raw).hexdigest()==entry['profileSHA256'];p=json.loads(raw);nodes={n['id']:n for n in p['nodes']};parents={c:n['id'] for n in p['nodes'] for c in n.get('children',[])}; total=sum(p['timeDeltas']); langs=collections.Counter();roles=collections.Counter();selfg=collections.Counter();ancg=collections.Counter();unionc=collections.Counter();selfr=collections.Counter();incr=collections.Counter();frames={}
 for sid,w in zip(p['samples'],p['timeDeltas']):
  node=nodes[sid]; f=node['callFrame'];langs[lang(f)]+=w;g=groups(f);selfg.update({k:w for k in g});role=next((k for k in rules if k in g),lang(f)+'-other');roles[role]+=w; key=json.dumps(f,sort_keys=True);frames[key]=f;selfr[key]+=w;seen=set();allg=set()
  while node:
   f=node['callFrame'];key=json.dumps(f,sort_keys=True);frames[key]=f
   if key not in seen:incr[key]+=w;seen.add(key)
   allg.update(groups(f));node=nodes.get(parents.get(node['id']))
  ancg.update({k:w for k in allg});unionc.update({k:w for k,ks in unions.items() if allg.intersection(ks)})
 assert sum(langs.values())==total and sum(roles.values())==total
 pc=lambda d:{k:round(100*v/total,5) for k,v in d.most_common()}
 rawrows=[{'frame':frames[k],'selfPercent':100*selfr[k]/total,'inclusivePercent':100*v/total} for k,v in incr.most_common()]
 out['profiles'].append({'label':entry['label'],'runtime':entry['runtime'],'stage':entry['stage'],'weightedUs':total,'sampleCount':len(p['samples']),'disjointSelfLanguages':pc(langs),'disjointSelfPriorityRoles':pc(roles),'groups':{k:{'selfPercent':100*selfg[k]/total,'inclusivePercent':100*ancg[k]/total} for k in rules},'deduplicatedAncestorUnions':pc(unionc),'frames':rawrows})
(root/'attribution.json').write_text(json.dumps(out,indent=2)+'\n')
for p in out['profiles']:
 print(p['stage'],p['runtime'],'UNIONS',p['deduplicatedAncestorUnions'])
 if p['runtime']=='candidate':print('GROUPS',{k:{t:round(v,3) for t,v in vals.items()} for k,vals in p['groups'].items() if vals['inclusivePercent']>.2})
