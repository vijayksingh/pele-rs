#!/usr/bin/env python3
"""Offline CDP attribution. No builds, profile capture, or source mutations."""
import collections
import gzip
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'profile/er-medium-render-model-wasm.cpuprofile.gz'
source_bytes = gzip.decompress(SOURCE.read_bytes())
profile = json.loads(source_bytes)
nodes = {node['id']: node for node in profile['nodes']}
parents = {}
for node in nodes.values():
    for child in node.get('children', []):
        assert child not in parents, 'Expected a context tree, not a DAG'
        parents[child] = node['id']
assert len(profile['samples']) == len(profile['timeDeltas'])
assert all(delta >= 0 for delta in profile['timeDeltas'])
total = sum(profile['timeDeltas'])
weights = collections.Counter()
phase_self = collections.defaultdict(collections.Counter)
store_re = re.compile(r'^<pele_wasm.*::host_er::HostStore(?: as .*?)?>::')
capture_frames = [n['callFrame']['functionName'] for n in nodes.values() if 'HostRead' in n['callFrame']['functionName'] and '::capture' in n['callFrame']['functionName']]
for leaf, delta in zip(profile['samples'], profile['timeDeltas']):
    stack, current = [], leaf
    while current is not None:
        stack.append(nodes[current]['callFrame'])
        current = parents.get(current)
    functions = [frame['functionName'] for frame in stack]
    has = lambda text: any(text in fn for fn in functions)
    audit = any(frame['functionName'] == 'audit' and frame['url'].endswith('/host-er-store.js') for frame in stack)
    admission = has('::compat::host_er::validate_render::')
    draw = has('::compat::diagrams::er::render::draw::')
    assert sum([audit, admission, draw]) <= 1
    phase = 'audit' if audit else 'admission' if admission else 'draw' if draw else 'other'
    weights['phase/' + phase] += delta
    phase_self[phase][functions[0] or stack[0]['url'] or '(anonymous)'] += delta
    store = any(store_re.match(fn) for fn in functions)
    typed = has('hostErRead')
    if store:
        weights['overlapping/' + phase + '/HostStore_subtree'] += delta
    if typed:
        weights['overlapping/' + phase + '/typed_import_subtree'] += delta
    if audit:
        weights['audit_split/' + ('entry' if 'begin' in functions else 'final' if 'structure' in functions else 'other')] += delta
    if admission:
        # Exclusive buckets; no double count of nested imports/key/cache helpers.
        bucket = 'own_info' if has('::HostStore>::own_info') else 'key_cache' if has('::HostStore>::key') else 'units_cache_transfer' if has('::HostStore>::units') else 'other_typed_import' if typed else 'other_store' if store else 'validation_outside_store'
        weights['admission_split/' + bucket] += delta
    if draw:
        # Capture is inlined, so explicitly keep all live/capture reads together.
        host_read = any('::live_read::HostRead<' in fn and (fn.startswith('<pele[')) for fn in functions)
        bucket = 'HostStore_or_HostRead_subtree' if store or host_read else 'remaining_draw'
        weights['draw_split/' + bucket] += delta
        for method in ['field', 'join', 'array_get', 'array_has', 'array_length', 'map_entries', 'coerce_string']:
            if any(fn.endswith('::live_read::RenderStore>::' + method) for fn in functions):
                weights['overlapping/draw/RenderStore_' + method] += delta
        for name, needle in [('text_layout', '::compat::text::layout_label'), ('svg_document', '::compat::svg::document'), ('compound_layout', '::compat::layout::compound::compound_layout')]:
            if has(needle):
                weights['overlapping/draw/' + name] += delta
    if phase == 'other' and functions[0] == 'encode' and 'hostOperation' in functions:
        weights['other_self/metadata_UTF8_encode'] += delta
    if phase == 'other' and 'serde_json' in functions[0] and '::serialize_str' in functions[0]:
        weights['other_self/serde_serialize_str'] += delta
assert sum(weights['phase/' + phase] for phase in ['audit', 'admission', 'draw', 'other']) == total
assert sum(value for key, value in weights.items() if key.startswith('admission_split/')) == weights['phase/admission']
assert sum(value for key, value in weights.items() if key.startswith('draw_split/')) == weights['phase/draw']
result = {
    'input': str(SOURCE.relative_to(ROOT)),
    'input_sha256': hashlib.sha256(source_bytes).hexdigest(),
    'method': 'Each sampled context is weighted by its matching timeDeltas entry. An ancestor predicate contributes at most once per sample. phase, admission_split, draw_split, audit_split are each mutually exclusive within their own domain; overlapping metrics must not be summed.',
    'sample_count': len(profile['samples']),
    'sampled_us': total,
    'span_us': profile['endTime'] - profile['startTime'],
    'capture_frames': capture_frames,
    'metrics': {key: {'us': value, 'percent_of_all_sampled_time': value * 100 / total} for key, value in sorted(weights.items())},
    'phase_self_top': {phase: [{'name': name, 'us': value, 'percent_of_all_sampled_time': value * 100 / total} for name, value in counts.most_common(20)] for phase, counts in phase_self.items()},
}
current_ms, old_ms, js_ms = 3.14375, 1.77125, .48609
result['illustrative_projections_not_benchmarks'] = {
    'reported_current_ms': current_ms, 'reported_old_ms': old_ms, 'reported_js_ms': js_ms,
    'required_fraction_removed_to_old': 1 - old_ms/current_ms,
    'required_fraction_removed_to_js': 1 - js_ms/current_ms,
    'delete_entire_admission_ms': current_ms * (1 - weights['phase/admission']/total),
    'delete_entire_admission_store_ms': current_ms * (1 - weights['overlapping/admission/HostStore_subtree']/total),
    'delete_entire_admission_and_audits_ms': current_ms * (1 - (weights['phase/admission'] + weights['phase/audit'])/total),
    'delete_entire_own_info_ms': current_ms * (1 - weights['admission_split/own_info']/total),
}
output = Path(__file__).with_name('attribution.json')
expected = json.loads(output.read_text())
assert result['input_sha256'] == expected['input_sha256']
assert result['metrics'] == expected['metrics']
print(json.dumps({key: value for key, value in result.items() if key not in ['phase_self_top', 'method']}, indent=2))
