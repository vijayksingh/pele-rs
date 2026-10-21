# Retained ER attribution after inline publication and typed Map reads

Recommendation: defer the admission frame. First remove the remaining generic
array-count transport inside admission. The current profile makes that a much
smaller, directly attributable layer. An admission frame remains plausible for
incremental savings, but cannot recover the old retained-render result by itself.
No implementation, source changes, builds, or new runtime measurements here.

## Evidence and method

Input: `../er-inline-publication/runtime/profile/er-medium-render-model-wasm.cpuprofile`.
SHA256: `59a75f605b9b7073d92407aede9ffc87427349a7ef4ddc527f82b0c45c2f7954`.
Associated WASM SHA256 in profile summary:
`5bec378c3b4f798da8839dc60aec86f248a1651c4d5ed3f40217f9db055e69a5`.
CDP Chromium 156, Linux x64, 250-us requested interval, 449 retained operations.
4480 samples carry 1,545,088 us; profile span is 1,545,521 us. The unmatched
433-us tail is not assigned to an invented sample.

`attribute.py` weights each sampled context by its corresponding `timeDeltas`
entry. It walks that sample's actual ancestor chain and counts a predicate at
most once. Repeated call contexts and nested helpers are not added twice.
`attribution.json` contains the raw microsecond totals, exclusive partitions,
explicitly marked overlapping drilldowns, self-time rankings, and projections.
Run the script directly to reproduce this offline analysis. It reads the frozen
profile and writes only its sibling attribution JSON.

The denominator includes program, GC, and idle samples. It is intentionally the
same denominator used by the capture harness, not a selectively normalized
"useful work" total. Sample percentages are evidence for this run, not precise
confidence intervals or uninstrumented timing measurements.

## Exclusive retained-render partition

| Phase | Weighted share |
|---|---:|
| Rust `validate_render`, including descendants | 18.207% |
| JS audit traversal, including descendants | 12.150% |
| Rust `draw`, including descendants | 49.630% |
| Everything outside those three subtrees | 20.014% |

The two audit traversals split into entry 6.092% and final 6.057%. `begin` and
`structure` self time stays in the last row; audit is not being inflated with
its callers. Conversely, reporting only the audit function's 7.027% self time
would undercount its descriptor/classification/visit helpers. The final audit
must remain after callbacks; an entry snapshot cannot substitute for it.

### Admission, disjoint detail

All percentages below use the whole profile denominator and sum to 18.207%.

| Admission work | Share |
|---|---:|
| `HostStore::own_info` subtree | 5.929% |
| property-key cache subtree | 1.563% |
| immutable units cache/transfer subtree | 1.761% |
| other typed-import subtrees | 4.037% |
| other HostStore work | 1.758% |
| validation outside HostStore | 3.160% |

These buckets give nested own-info/key/units helpers priority over their child
imports, so they do not overlap. All HostStore admission descendants together
are 15.047%. This is an optimistic target envelope, not wholly removable work:
string construction, the validator's allocation/work rules, and the replacement
frame reads still have costs.

The remaining generic hotspot is concrete:
`AdmissionStore::array` in `crates/pele-wasm/src/host_er.rs` obtains typed
kind/length, then calls `own_info(id, "")` solely for `.count`.
`own_info` calls generic `primitive("own", ...)`, creates/decodes JS Arrays,
and extracts fields which admission discards. Its 5.929% inclusive share
contains 1.426% `ownInfo` JS self, 1.339% generated string decode, 0.763% Rust
own_info self, 0.602% decodeText, 0.436% wasm-to-JS self, and other glue.
Those component numbers must not be added to the 5.929% again.

Small next candidate: direct `hostErReadOwnCount(context, id) -> u32`, used only
by AdmissionStore::array. A first version can wrap the existing
`owner.ownInfo(id, "")[0]` to retain every current side effect while dropping
Rust's generic argument/result transport. A subsequent internal refactor could
avoid allocating that discarded JS tuple while preserving the same sequence.
Keep MethodStore ownInfo unchanged. Preserve enumeration order, accessor
rejection, writable Array.length exclusion, error diagnostic handling, and the
final empty-key descriptor lookup/value classification even when that value
is discarded: it can intern/identify values or fail. Keep typed kind checking,
length/count work charges, first-error capture, and u32 range checks unchanged.
No persistent count cache; no callback or conversion assumptions needed beyond
those already required by the current path.

### Capture and later live reads

There is **no capture frame in the profile**. `HostRead::capture` has been
inlined into `draw`, along with some later reader calls. The CDP tree identifies
callee contexts but does not distinguish callsites within that inlined caller.
Therefore an exact capture-versus-later-cell percentage cannot honestly be
recovered from this artifact. `field` under draw is used in both phases.

Observable union: draw-side HostStore or HostRead subtrees account for 15.701%
of the total; remaining draw is 33.929%. That reader union includes local
conversion/allocation/formatting, not just crossing the boundary. HostStore
alone is 14.649%; typed import subtrees are 6.295%, already inside that union.
Useful overlapping call-path magnitudes are field 7.114%, join 3.354%, array_get
1.381%, array_length 0.640%, Map membership 0.408%, array_has 0.278%, and
coerce_string 0.240%. The small Map/coercion shares indicate the previous slice
has addressed those particular call paths; they are no longer leading targets.

Other draw work remains material: text layout subtree 10.366%, SVG document
subtree 5.324%, compound layout subtree 1.966%. These are drilldowns, not extra
rows to add to the exclusive phase table. Optimizing reads alone does not
remove them. Exact capture attribution would require a later profile with
separate non-inlined phase markers or equivalent symbols, and then ordinary
performance remeasurement without those markers. No such instrumentation was
added during the freeze.

### Output and operation overhead

The residual 20.014% includes hostOperation self 3.030%, metadata UTF8
`TextEncoder.encode` self 3.432%, serde serialize_str self 2.764%, begin/structure
self 1.899%, program 2.814%, and GC 0.514%, among smaller paths. The encode is
`owner.finish(token, new TextEncoder().encode(json).length)` in full.js. It
allocates UTF8 bytes again only to recover metadata byte length after Rust has
already serialized the bounded JSON. Carrying that exact serialized byte count
through a later output ABI is a separately reviewable candidate. Any such
change must preserve graphBytes-plus-metadata admission and error precedence;
removing the check is not an optimization. The two listed output shares are
self-time floors, not complete output-path attribution.

## Admission-frame ceiling and sequencing

Parent's matched unprofiled medians: current retained 3.14375 ms, old retained
1.77125 ms, pinned JS 0.48609 ms. Reaching old requires removing 43.658% of
current time; reaching JS requires 84.538%. Applying profile fractions to those
medians is only an illustrative Amdahl calculation:

| Impossible zero-cost deletion | Projected remaining time |
|---|---:|
| All admission, including validation | 2.57136 ms |
| All admission HostStore subtrees | 2.67070 ms |
| All admission AND both audits | 2.18941 ms |
| Only the entire admission own_info subtree | 2.95735 ms |

Even deleting admission and both required audits would remain above the old
1.77125-ms result. Actual admission frames retain validation, audit traversal,
string semantics, and pay binary emission, copying, indexing, and frame reads.
The realistic benefit must be less than the broad 15.047% HostStore envelope,
with no measured net benefit established yet. That envelope corresponds to
about 0.473 ms on the reported median, before replacement costs. A direct count
import also retains actual enumeration, so its benefit is below the 5.929%
whole-subtree bound; its advantage is limited scope and preserved read order.

Recommended sequence:

1. Private direct admission-count ABI candidate; targeted public feature-slice
   parity plus the same VM retained benchmark, then decide from measured gain.
2. Separately inspect output metadata byte-count handoff, preserving the exact
   envelope and callback/error ordering. Its observed 3.432% allocation is a
   simpler target than shared mutable capture state.
3. Revisit an admission-only frame only if the remaining measured admission
   transport justifies its complexity. Use the prior proposal's optional caps,
   live fallback, byte-offset-only indexes, exact atoms/UTF16, and unchanged
   error/budget order. Drop mutable frame state before rendering.
4. Do not extend a frame into capture without new phase evidence. Capture can
   join/coerce; later rows interleave reads with callbacks; responsive retries
   recapture. Universal callback invalidation and nested-operation coverage
   add risk for an opportunity this profile cannot isolate.

Parse now matches the old path in the reported three-row sample, but remains
well above pinned JS. Retained rendering still has a large measured gap.
Neither this attribution nor the parse improvement establishes release parity.
