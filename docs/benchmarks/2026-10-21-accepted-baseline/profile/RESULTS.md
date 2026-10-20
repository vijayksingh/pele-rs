# Accepted-baseline retained profile

Valid evidence is attempt2 only: candidatea4f933d9af2cd3ccd5cda7eac46dc1048a296249, sameWASM96f4e39, e148control. Same7aae1db1 harness: four serial5-second fixed medium/large retained windows,250µs sampling and500ms warmup. Parse once outside profiles. Full SVG/dimensions and ordered traces match pinned Pele before/after;48/200 width callbacks each lane. All117/113 package, source pins, oracle/harness/cases hashes unchanged; models/browser closed.

| Candidate | Prepare inclusive | Match inclusive | Snapshot / sparse encode / apply sampled | Reader boundary union | Live-phase decode leaf |
|---|---:|---:|---|---:|---:|
| sankey-medium-fixed-render-model-candidate | 10.33% | 9.97% | 0 / 0 / 0 | 20.63% | 0.82% |
| sankey-large-fixed-render-model-candidate | 10.16% | 9.84% | 0 / 0 / 0 | 21.76% | 0.28% |

The redundant snapshot/encode/apply path is absent from sampled stacks on these unchanged repeated-render inputs. Preparation is now predominantly matchesGraph and has similar sample share to this run’s e148control (control matching9.61/9.53%). Zero sampled frames are not invocation counts: separate warm continuity/feature probes establish exact accepted-receipt zero-apply behavior. Matching remains necessary to detect public mutation; this is not permission to skip it.

Remaining explicit live Reader call/key/units ancestor union20.63%medium/21.76%large includes call9.80/10.61,key7.85/7.60,units2.98/3.56. All overlap the shared render phase. Native geometry/SVG named leaf partition17.68/17.72%; liveHostPhase74.41/77.39% includes geometry and SVG, not just boundary cost. Total decode leaf8.43/7.92% is mostly result factory3.96/4.36 and width callback3.21/3.22; live control decode0.82/0.28. charCodeAt leaf0.17/0.19%. Remaining native work includes layout settle, SVG coordinate/raw writes, string search and counted JSON metadata.

If later work targets retained rendering, the remaining coarse boundary candidate is an ordered live-read operation that resolves only currently unconditional get/index chains per Rust phase, preserving original object handles, exceptions and every callback boundary. It must not pre-read conditional fields or cache values across callbacks. These samples do not justify another full-model conversion, unsafe no-op scan, or a charCodeAt-only optimization. No implementation or speed promise in this report.

Attribution is weighted by raw CDP timeDeltas. Leaf categories partition all samples. Ancestor categories count each sample once per category but overlap; never add them as independent ceilings. Inlining obscures exact leaf ownership. Historical auto native-parser regex matches IndexedSankey type names; ignore that group: no parse happens in these windows. Corrected analyzer is data-only; no profiling retry is embedded.

Attempt1 remains rejected under runtime/: first control14741sample profile had timeDeltas[1158]=-224µs and[1189]=-409µs, span5,057,385µs and sum5,057,007µs. Strict validator stopped before any candidatewindow. Browser/models closed, input hashes unchanged. Parent authorized one fresh full run in distinct runtime-attempt2; it passed the unchanged nonnegative-weight assertion. No clamping/dropping or third run. Failed rawSHAcb96fcca4839838cc0359ea3bd81e7c28294eb9bb49e16ff572d158ce5e7cc00.

VM idle confirmed. No first-use. No isolated gain inferred across prior windows.
