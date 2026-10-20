# Historical README performance checkpoints

Archived from the development README on 2026-10-20. Each checkpoint describes
its own artifact and measurement window. Words such as "latest" and API blockers
belong to that checkpoint; later work may supersede them. Use the current
[README](../README.md#performance) for the development status. Raw evidence remains
linked below, including adverse controls and rejected experiments.

# Performance checkpoints

**0.3 remains unreleased. The latest full performance release gate still fails all 126 conditions.**

The latest measured [arena render transport](benchmarks/2026-10-20-er-host-storage/arena-render-packet/),
source `9ae8142`, borrows native post-render graphs and returns browser results
without complete result-JSON buffers. All **36 affected retained-render rows**
have lower paired medians, **1.20–24.16%** lower (median **−15.30%**), but still
take **1.65–18.49× JS time**. All 12 affected first-unchanged-render conditions
improve; one edited Flowchart condition is slightly slower. Parse/source controls
remain mixed: a longer follow-up retains **+2.70%** for Sequence/small fixed
parse. Full browser correctness passes **417/417**, alongside 41 native
integrations, 16 Swift integrations, three C consumers and 168 Swift API
conditions. Local Swift latency is unmeasured: unrelated compiler processes
blocked the guarded attempt. These diagnostics do not satisfy the release gate.

The [source-result packet](benchmarks/2026-10-20-er-host-storage/source-result-packet/),
source `640deee`, removes the complete result JSON round trip while preserving
exact byte limits, dimension bits and callback errors. Its new **126-condition
diagnostic** has lower paired medians in **all 42 source-render rows**, from
0.89% to 20.78% lower; the conventional median of those row changes is
**−13.89%**. Source rendering still takes **1.35–4.63× pinned JS time**.
Parse and retained controls remain mixed: their median row changes are −0.04%
and −0.30%, with adverse rows up to +6.83% and +3.85%. Full browser correctness
passes **405/405**; fresh Swift integration/API checks use the unchanged native
SDK. Nine trials and 500 ms warmup per row make this a scoped diagnostic,
not a replacement release gate or a regression-free claim.

The [owned Sankey construction](benchmarks/2026-10-20-er-host-storage/owned-construction/)
layer, source `b19f38d`, avoids rebuilding a typed model immediately after its
public graph is constructed. All six VM parse rows are **8.65–11.19% lower**
paired, still **8.91–15.08× JS time**. First-use results remain mixed:
medium/fixed first-unchanged render is **+2.242%** (7/15 lower pairs).
One complete local native public-parse condition is 16.21% lower and takes
**1.43× local Node JS time**; its typed-parse control is +1.05% paired.
These local and VM measurements use different hosts and API costs; their
ratios cannot be combined. External builds prevented six other native and all
six Swift latency conditions from completing; rejected windows are excluded.
Fresh native/browser checks, Swift 16 integrations, C3 and 168 API conditions
pass. All controls and the partial timing scope remain in the evidence.

The latest full release-gate matrix remains the historical
[ER token ownership checkpoint](benchmarks/2026-10-20-er-token-ownership/),
source `0b32797`, where all 126 conditions fail. That artifact's source render
takes **1.58–5.12×** JS time; median ratios across sizes/resources are
**2.38× source**, **11.06× parse** and **4.02× retained mutable render**.
These describe that artifact; differences between historical runs are not
causal speedup percentages.

The [dense ER array publisher](benchmarks/2026-10-20-er-host-storage/dense-array/)
removes generic property records from fresh arrays while preserving canonical
IDs and limits. VM ER parse improves 3.82–10.24% paired; complete first-use ER
operations also improve, but parse still costs 4.52–9.96× JS. Medium Canvas
retained rendering regresses 4.64%; Swift mutable parse/lifetime regress about
0.65%, with native allocation counters unchanged. Fresh native, Swift/C and
84 targeted browser checks pass. Full JS performance parity remains open.

The [borrowed ER arena interner](benchmarks/2026-10-20-er-host-storage/borrowed-interner/)
removes repeated owned UTF16 allocations during native public construction.
Matched native public parse improves 10.59%, Swift mutable parse 12.00%, and
native allocation count falls 20.55% for the medium ER fixture. Fresh native,
Swift/C and targeted browser checks pass. Ordinary web parse uses a different
interner; its VM controls remain adverse at +2.80%/+2.74%. No web gain is claimed.

The [ordered ER record publisher](benchmarks/2026-10-20-er-host-storage/record-layout/)
improves VM parse by 7.67–17.15% paired against the preceding artifact, while all
six source-render controls are slower. Full browser correctness passes 390 checks;
fresh local Swift passes 168 API conditions and its ER corpus. Combined native
changes improve Swift ER mutable parse 22.69% against frozen `0b32797`; this is
not an isolated record-publisher result. Web parse still takes 5.07–10.53× JS time.

The [entry read frame](benchmarks/2026-10-20-er-host-storage/entry-frame/)
improves retained ER rendering by 11.02–14.37% paired against the lexer artifact.
The subsequent [error-brand allocation removal](benchmarks/2026-10-20-er-host-storage/error-brands/)
improves it another 7.44–10.01% against the frame artifact, while parse controls
are mixed. Retained ER still takes 3.35–4.30× JS time in that latest scoped run.
These separate measurement windows must not be compounded into a single gain.

The [compiler experiment](benchmarks/2026-10-20-er-host-storage/compiler-o3/)
keeps `PELE_WASM_OPT_LEVEL=3` opt-in: median paired parse time improves 5.71%
across 42 conditions, but WASM grows 43.54% raw / 26.79% Brotli. Startup remains
unmeasured; the default stays `s`.

The [shared ER lexer follow-up](benchmarks/2026-10-20-er-host-storage/lexer/)
improves VM ER parse by 6.1–8.6% paired against the preceding packet artifact.
Local Swift ER mutable/immutable parse improves 20.8%/10.0% against frozen
`0b32797` across the combined native changes; Sequence controls are slightly
slower. These are scoped gains, with the JS performance gate still open.

The [result-transport follow-up](benchmarks/2026-10-20-er-host-storage/result-packet/)
improves retained ER rendering by 9.4–14.9% in paired VM medians against the
preceding host artifact. It still takes 4.4–6.3× JS; no 0.3 release.

The newer [live ER storage checkpoint](benchmarks/2026-10-20-er-host-storage/)
removes persistent model copies and fixes ordinary ER callback state. Its
30-condition VM comparison still shows mixed parse results and 75.7–91.9%
slower retained ER rendering by paired medians against the preceding backend.
The [prepared-entry follow-up](benchmarks/2026-10-20-er-host-storage/prepared/)
fixes its callback regressions; fresh all-family Swift prepared/raw checks pass.
The [caller-root follow-up](benchmarks/2026-10-20-er-host-storage/raw/)
passes focused raw callback checks, but repeated raw rendering regresses 71–88%
against the preceding arena path. Performance parity remains open; no 0.3 release.

The [direct-parse architecture checkpoint](benchmarks/2026-10-20-direct-parse/)
shares source preparation and grammar, then constructs Gantt's canonical public
arena without temporary owned task records. Native public parse removes 438
allocations and has an 11.0% lower matched median. Independent VM parse medians
are 2.5–9.5% lower across six Gantt size/resource conditions. First edited and
callback renders remain slightly slower, and control rows also move; this is
not a general browser win. Full output checks pass, but the new complete
126-condition performance gate still fails every condition.

The earlier [eager Swift checkpoint](benchmarks/2026-10-20-swift-eager/) removes the
second full model JSON buffer and recursive Foundation decode while keeping
`Document.model` eager. Same-Mac, matched immutable-parse medians:

| Workload | Previous bridge | Eager handoff checkpoint | Before / after |
| --- | ---: | ---: | ---: |
| Sequence small | 0.3084 ms | 0.0587 ms | 5.25× |
| Gantt medium | 5.3088 ms | 0.8451 ms | 6.28× |

Each wins all 15 pairs. Source and typed-retained controls remain near-flat;
these two workloads do not establish all-33 speedups or a JS ratio. Separate
guarded runs use 1000 ms measured warmup and identical batch counts. Raw reports,
source/SDK hashes, profiles and interrupted-build exclusions remain linked.
The ARM64 Swift corpus passes all 33 families, including 167,473 mutable drawings
and 270,358 error cases, with five existing explicit transport-budget differences.
Seven new projection integrations and 168 eight-API output conditions pass.
Native UI activation remains unverified: the earlier full suite failed 19 cases
and 53 assertions when the host could not become active/key; a bare AppKit probe
reproduced this without Pele. Universal Intel validation remains required before release.

The [next Gantt projection layer](benchmarks/2026-10-20-normalized-projection/)
removes its first full JSON buffer/reparse: matched Swift parse falls from
**0.8582 to 0.6292 ms**, **26.69% lower paired median, 15/15 wins** against
that preceding eager binary. Sequence parse and source/retained controls stay
near flat. Gains from separate runs are not multiplied. VM results are mixed,
including a Gantt fixed-source slowdown; no browser gain is established.
Current Gantt corpus, eager integrations, 168 consumer API conditions and all
three browser engines pass. The full release gates remain open.

The [static-policy correction](benchmarks/2026-10-20-projection-policy/)
removes 33 unnecessary handler-box allocation requests per builtin registry
construction (69 to 36). Same-Mac parse paired medians improve 0.86% for Gantt
and 2.16% for Sequence, each 13/15 faster; control medians remain near flat with
outliers. VM source medians return near the earlier control, while parse remains
mixed. Full current 33-family Swift corpus and 168 consumer API conditions pass;
this remains a performance checkpoint, not release acceptance.

The [ER immutable projection layer](benchmarks/2026-10-20-normalized-er/)
removes the same JSON buffer/reparse for captured ER. Matched Swift ER-medium
parse falls from 0.8484 to 0.6219 ms (25.67% paired, 15/15 faster). Local control
medians are slightly slower and VM results remain mixed; this change does not
remove mutable web graph construction. ER corpus and current API checks pass.

The [direct ER construction layer](benchmarks/2026-10-20-direct-er/)
avoids a complete typed model and conversion during eligible mutable parsing.
ER-medium Swift mutable parse improves 17.9% paired; native public parse improves
18.5%, confirmed in a second window, but still trails JS. VM ER parse paired
medians improve 0.9–7.4% across six size/resource conditions. Controls and first
use remain mixed, including slower rows. Native, Swift and three-browser checks
pass, while a newly reproduced live resource-callback mismatch remains an open
API parity blocker. No full parity or release readiness claim follows.

The [ER token ownership layer](benchmarks/2026-10-20-er-token-ownership/)
borrows lexer text and creates owned Strings only when grammar actions consume it.
ER-medium core parse improves 13.2% paired, native public parse 7.6%, and Swift
mutable parse 7.6%. Public parse requests 810 fewer allocations. VM results are
mixed, including a 45.2% paired slowdown for large ER Canvas parse; this is not a
universal web win. Controls and the complete failing gate remain disclosed.

[Earlier checkpoints](PERFORMANCE.md) preserve their own matched controls,
raw samples and tradeoffs; gains are not combined. Recent
[rejected text/identity experiments](benchmarks/2026-10-20-parse-candidates/)
remain outside production. No JS speedup or GPU acceleration is claimed.

Independent Linux VM: AMD EPYC-Genoa, 8 vCPUs, 16 GiB, Chromium 156.0.8078.4.
Verified 5 µs clock ticks; nine alternating trials after at least 500 ms warmup.
The separate controlled Gantt comparison uses 15 trials and 1,000 ms warmup.
Large workloads use 200 units, except Sankey (199 links) and Railroad (60 rules).
Source-render measurements with cached Canvas metrics:

| Large workload | Pele JS | Rust/WASM | WASM / JS |
| --- | ---: | ---: | ---: |
| Flowchart | 1.386 ms | 2.485 ms | 1.79× |
| Sequence | 0.523 ms | 1.031 ms | 1.97× |
| Er | 2.826 ms | 6.177 ms | 2.19× |
| Pie | 0.279 ms | 0.617 ms | 2.21× |
| Sankey | 0.864 ms | 1.485 ms | 1.72× |
| Gantt | 0.713 ms | 2.554 ms | 3.58× |
| Railroad | 0.765 ms | 1.856 ms | 2.42× |

A separate [native-only comparison](benchmarks/2026-10-20-native-core/)
before the latest root rewrite removes WASM/Swift entirely: typed retained native/JS medians are 0.80× Flowchart,
0.70× Sequence, 1.04× ER, 0.50× Pie, 0.41× Sankey, 1.15× Gantt and 1.38× Railroad.
Seven medium workloads on the same Mac; this is not all-33 certification.
Native compatibility work is still costly: the latest matched Gantt medium
comparison takes 0.149 ms for native public parse versus 0.082 ms for JS.
Typed native parse takes 0.055 ms in that run; public graph construction adds
work before any WASM crossing. These are stage medians, not subtractable phase
measurements or all-diagram guarantees. Current matched-history diagnostics show 22–25%
lower mutable publication medians against matched native control; the extra
graph/revision return contract and growing arena remain disclosed.

Swift and VM figures cannot be compared across machines. Timings include public
API/ownership costs and exclude startup, downloads and SVG painting. Earlier
reports remain historical; percentage gains are not combined across timer conditions.

