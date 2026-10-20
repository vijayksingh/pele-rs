#!/usr/bin/env python3
"""Existing CDP data only; no browser, process sampling or benchmarks."""
from pathlib import Path
from collections import defaultdict
import json,hashlib,gzip
DIRECTORY=Path(__file__).resolve().parent
def category(frame):
    name,url=frame['functionName'],frame['url']
    if name=='(garbage collector)': return 'GC'
    if name.startswith('wasm-to-js') or name.startswith('js-to-wasm'): return 'WASM/JS boundary shim'
    if '.wasm' in url: return 'WASM Rust/runtime/glue'
    if url or frame.get('scriptId','0') not in ('','0'): return 'JS (including harness)'
    return 'V8/system/unattributed'
def analyze(path):
    raw=gzip.decompress(path.read_bytes());d=json.loads(raw);ns={n['id']:n for n in d['nodes']};parents={c:n['id'] for n in d['nodes'] for c in n.get('children',[])}
    assert len(d['samples'])==len(d['timeDeltas']);total=sum(d['timeDeltas']);buckets=defaultdict(int);native=defaultdict(int);other=defaultdict(int);families=defaultdict(int)
    groups={'ER lexer':lambda s:'::diagrams::er::lexer::' in s,
            'HostConstruction methods':lambda s:s.startswith('<pele_wasm[') and '::host_er::HostConstruction' in s,
            'Rust hash tables and SipHash':lambda s:'hashbrown[' in s or '::hash_one' in s or '::hash::sip::' in s,
            'Native atom encoding':lambda s:'::host_er::construction_atom' in s,
            'ER parser':lambda s:'::diagrams::er::parser::' in s,
            'Rust format':lambda s:'::fmt::' in s,
            'Rust allocator':lambda s:'dlmalloc[' in s or '__rust_alloc' in s or '__rust_realloc' in s or '__rust_dealloc' in s,
            'ER capture/live-read':lambda s:'::diagrams::er::live_read::' in s,
            'Host admission':lambda s:'::host_er::Admission' in s or '::host_er::validate_render' in s}
    incl={g:0 for g in groups}
    for node,w in zip(d['samples'],d['timeDeltas']):
        frame=ns[node]['callFrame'];name=frame['functionName'];cat=category(frame);buckets[cat]+=w
        if cat=='WASM Rust/runtime/glue':
            native[name]+=w
            for group,match in groups.items():
                if match(name):families[group]+=w
        elif cat=='V8/system/unattributed':other[name]+=w
        seen=set();cur=node
        while cur in ns:
            f=ns[cur]['callFrame'];s=f['functionName']
            if category(f)=='WASM Rust/runtime/glue':
                for group,match in groups.items():
                    if group not in seen and match(s):incl[group]+=w;seen.add(group)
            cur=parents.get(cur)
    rows=lambda v:[{'name':k,'microseconds':n,'percentOfWholeProfile':n/total*100} for k,n in sorted(v.items(),key=lambda x:-x[1])]
    return {'file':path.name.removesuffix('.gz'),'sha256':hashlib.sha256(raw).hexdigest(),'samples':len(d['samples']),'totalMicroseconds':total,
            'selfBuckets':rows(buckets),'topWASMSelf':rows(native)[:25],'unattributedSelf':rows(other),
            'nativeFamilySelf':rows(families),'nativeFamilyInclusiveDeduplicated':rows(incl)}
reports=[analyze(p) for p in sorted(DIRECTORY.glob('*.cpuprofile.gz'))];assert len(reports)==6
output=Path(__file__).with_name('six-profile-shares.json')
output.write_text(json.dumps({'method':'timeDeltas-weighted self buckets are mutually exclusive. WASM includes Rust/library/runtime/glue, not solely grammar. Boundary shims separate. Family inclusive samples deduplicated within a family; different families overlap and must not be summed. Dense770 profiles, two permanent string maps; pre-literal and post-borrowed-interner.','metadata':json.loads(gzip.decompress((DIRECTORY/'summary.json.gz').read_bytes())),'profiles':reports},indent=2)+'\n')
for r in reports:
    print(r['file'], {x['name']:round(x['percentOfWholeProfile'],3) for x in r['selfBuckets']})
    if r['file']=='er-medium-parse-wasm.cpuprofile':
        print('native top',[(x['name'],round(x['percentOfWholeProfile'],3)) for x in r['topWASMSelf'][:15]])
        print('families self',[(x['name'],round(x['percentOfWholeProfile'],3)) for x in r['nativeFamilySelf']])
        print('families inclusive',[(x['name'],round(x['percentOfWholeProfile'],3)) for x in r['nativeFamilyInclusiveDeduplicated']])
