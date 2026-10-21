"""Data-only portable adaptation of profile/analyze.py; exact original retained."""
from pathlib import Path
import json,re,collections,hashlib,gzip
root=Path(__file__).resolve().parent; folder=root/'profile'; report=json.loads(gzip.decompress((folder/'summary.json.gz').read_bytes()));out=[]
assert report['succeeded'] and report['inputsUnchanged'] and report['browserClosed'] and report['modelsDisposed']
for p in report['profiles']:
 path=folder/'raw'/(p['label']+'.cpuprofile.gz'); data=gzip.decompress(path.read_bytes()); assert hashlib.sha256(data).hexdigest()==p['profileSHA256']; j=json.loads(data); ns={n['id']:n for n in j['nodes']};par={c:n['id'] for n in j['nodes'] for c in n.get('children',[])}
 total=sum(j['timeDeltas']); self=collections.Counter();anc=collections.Counter();dec=collections.Counter();stacks=collections.Counter()
 for id,w in zip(j['samples'],j['timeDeltas']):
  chain=[]; k=id
  while k in ns:chain.append(ns[k]['callFrame']);k=par.get(k)
  f=chain[0];name=f['functionName'];url=f['url']; names=[f['functionName'] for f in chain];txt='\n'.join(names)
  decode=name in ('decode','decodeText','getStringFromWasm0')
  role='engine-other' if not url else ('wasm-other' if '.wasm' in url or url.startswith('wasm:') else 'js-other')
  if name=='(garbage collector)':role='gc'
  elif decode:role='text-decode'
  elif name=='wasm-to-js':role='wasm-to-js'
  elif 'charCodeAt' in name or 'char_code_at' in name:role='char-code'
  elif url.endswith('/layout-storage.js'):role='js-layout'
  elif url.endswith('/graph-storage.js'):role='js-graph'
  elif url.endswith('/live-render.js'):role='js-live-read'
  elif url.endswith('/pkg/pele.js'):role='js-generated-other'
  elif 'source_store::' in name or 'source_input::' in name or 'captured_admission::' in name:role='native-source-storage-admission'
  elif 'sankey::live_phase' in name or 'detached_render::Reader' in name:role='native-live-read'
  elif 'sankey::render::' in name or 'sankey::layout::' in name or 'compat::svg::' in name:role='native-geometry-svg'
  self[role]+=w
  flags={'nativeApply':any(n=='apply_source_input' or n.startswith('mutabledocument_apply_source_input') for n in names),'sparseEncode':any(n=='encodeSourceInput' for n in names),'liveReaderBoundary':any('detached_render::Reader>::' in n for n in names),'liveReaderKey':any('detached_render::Reader>::key' in n for n in names),'liveReaderUnits':any('detached_render::Reader>::units' in n for n in names),'prepareMutableInput':any(n=='prepareMutableInput' for n in names),'snapshotGraph':any(n=='snapshotGraph' for n in names),'matchesGraph':any(n=='matchesGraph' for n in names),'liveReaderCall':any('detached_render::Reader>::call' in n for n in names),'liveHostPhase':any('sankey::live_phase' in n for n in names),'textDecode':any(n in ('decode','decodeText','getStringFromWasm0') for n in names),'charCodeAt':any('charCodeAt' in n or 'char_code_at' in n for n in names),'requireString':any('require_string' in n or n=='requireString' for n in names),'resultFactory':any('hostErRenderResult' in n or 'render_packet::emit' in n for n in names),'resourceWidth':any('::text_width' in n for n in names)}
  for name,v in flags.items():
   if v:anc[name]+=w
  if decode:
   category='result-factory' if flags['resultFactory'] else 'resource-width' if flags['resourceWidth'] else 'live-reader-call' if flags['liveReaderCall'] else 'inlined-live-phase' if flags['liveHostPhase'] else 'other'
   dec[category]+=w;stacks[' > '.join(names[:10])]+=w
 assert sum(self.values())==total
 percent=lambda d:{k:{'weightedUs':v,'percent':100*v/total} for k,v in d.most_common()}
 out.append({'label':p['label'],'sampleCount':len(j['samples']),'totalUs':total,'disjointSelf':percent(self),'deduplicatedAncestor':percent(anc),'disjointDecodeSelfByAncestor':percent(dec),'topDecodeStacks':[{'stack':s,'percent':100*v/total}for s,v in stacks.most_common(12)]})
result={'sourcePins':report['sourcePins'],'profiles':out,'method':'Leaf categories partition all weighted samples. Each ancestor category counts a sample once; categories overlap and must not be added. Decode subcategories are an ordered, disjoint attribution of decode/getStringFromWasm/decodeText leaf samples. Inlining may hide exact reader methods. Old automatic native-parser group matches IndexedSankey generic type names; ignore that group: no parsing occurs in these windows.','untimedCallbacks':[{'case':c['case'],'phase':c['phase'],'counts':{k:v['callbackCalls']for k,v in c['results'].items()}}for c in report['checks']]}
assert result == json.loads(gzip.decompress((folder/'attribution.json.gz').read_bytes()))
assert len(out)==4
print('Verified four complete profile attributions from raw weighted samples.')
for r in out:
 print(r['label']);print('SELF',[(k,round(v['percent'],2))for k,v in r['disjointSelf'].items()]);print('DECODE',[(k,round(v['percent'],2))for k,v in r['disjointDecodeSelfByAncestor'].items()]);print('ANCESTORS',[(k,round(v['percent'],2))for k,v in r['deduplicatedAncestor'].items()])
