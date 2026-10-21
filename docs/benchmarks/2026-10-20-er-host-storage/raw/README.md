# Caller-owned ER host adoption — unreleased

Source **`51f1269`** removes the raw caller-root snapshot/binary/native-graph/reconciliation round trip. Rust retains grammar, policy, admission, live reads, layout and SVG. Imported objects remain authoritative and preserve identity. This fixes six raw callback comparisons that previously failed in every browser; it is **not a performance win**.

No 0.3 release. The latest complete release gate remains the older `0b32797` artifact, which fails all 126 conditions. These diagnostics do not replace that gate or establish full API parity.

## Independent VM measurements

AMD EPYC-Genoa, 8 logical CPUs, 16 GiB, Node 22.23.2, Chromium 156. Candidate is the complete packed raw-host artifact; control is the preceding prepared-host artifact (`80bed95` plus its admission-count layer), identified by all 106 package-file hashes. Pinned Pele 1.6.0 is the third lane. All report inputs are immutable and rehashed afterward.

[Raw comparison](raw-comparison.json): 18 conditions, 15 balanced three-way trials, fixed and Canvas resources, 1,000 ms minimum warmup, matched batch counts and exact complete SVG/dimension/link/state/identity checks before and after. `raw-first-inclusive` times a fresh descriptor/alias preserving caller clone, all adoption/admission, first render and disposal. `raw-repeat` explicitly excludes its initial clone/adoption and disposal. `clone-only` is disclosed in the report; do not subtract it as causal Rust cost. Renderer/schema caches are warm. Startup, download, paint and GPU are excluded. Shared-page heap/GC and serial warmup remain limitations.

Positive change means slower. Paired change is the median of per-trial ratios; artifact/JS is the ratio of separate medians.

| Case | Resources | Stage | Paired change vs control | Faster trials | Candidate / JS |
| --- | --- | --- | ---: | ---: | ---: |
| er/small | fixed | raw-first-inclusive | +8.14% | 4/15 | 6.09× |
| er/small | fixed | raw-repeat | +73.11% | 0/15 | 7.00× |
| er/small | canvas | raw-first-inclusive | +6.40% | 3/15 | 5.34× |
| er/small | canvas | raw-repeat | +77.14% | 1/15 | 6.16× |
| er/medium | fixed | raw-first-inclusive | +18.72% | 0/15 | 4.30× |
| er/medium | fixed | raw-repeat | +77.06% | 0/15 | 5.83× |
| er/medium | canvas | raw-first-inclusive | +17.22% | 0/15 | 4.08× |
| er/medium | canvas | raw-repeat | +71.18% | 0/15 | 5.25× |
| er/large | fixed | raw-first-inclusive | +21.90% | 0/15 | 4.36× |
| er/large | fixed | raw-repeat | +84.81% | 0/15 | 6.50× |
| er/large | canvas | raw-first-inclusive | +27.15% | 0/15 | 4.16× |
| er/large | canvas | raw-repeat | +87.58% | 0/15 | 5.39× |

First-inclusive raw rendering regresses 6.40–27.15% in paired medians; repeated raw rendering regresses 71.18–87.58%. First-inclusive candidate/JS ratios are 4.08–6.09×; repeated ratios are 5.25–7.00×. All adverse rows remain in the report.

[Ordinary controls](ordinary-comparison.json): 12 conditions, nine balanced trials and 500 ms warmup, ER medium/large × fixed/Canvas × parse/retained/source. Paired changes range from −2.03% to +3.63%; 11/12 are adverse. Separate median ratios are mixed. Parse remains 4.41–10.64× JS; retained render remains 5.14–6.13× JS. No claim that this raw-only layer improves ordinary parse.

The first raw-harness attempt failed before input freeze because its imported guard module was missing on the VM. [Original failure](vm-raw-attempt1.log.gz) is retained. The exact guard was uploaded, inputs frozen, then the complete successful window ran. No elapsed sample was filtered or guard weakened.

The [independent evidence audit](AUDIT.md) reproduces all recorded statistics and documents provenance limits. First position is balanced at five per lane/row; pair precedence is 8/7, totaling 144/126. The ordinary diagnostic has no per-row quiet guard. The archived audit script preserves its original private-workspace paths; `summarize.mjs` verifies the published report directly.

The [private diagnostic source](benchmark/README.md), [input manifest](raw-inputs.json), [all paired summaries](paired-summary.json), [six fresh profiles](profile/summary.json) and [evidence index](evidence-index.json) are included. Profile sample percentages are attribution, not latency improvements. The dominant parse `allocate` frame belongs to JavaScript object publication, not Rust malloc. GC samples do not identify their allocation site.

## Correctness and platform checks

- 45 focused public browser checks pass across Chromium, Firefox and WebKit: all 33 packed entries, ordinary/prepared/raw callbacks, six formerly failing raw comparisons, ten exotic Ref-root controls, original arbitrary throws, nested rendering, config order, native weak-key failures, repair, ownership and disposal.
- 14 native public integration tests pass, including exact builtin selection, captured policy across replacement/drop, custom codecs, prepared/fallback publication and registry controls.
- Fresh ARM64 SDK and Swift consumer: 168 public API conditions pass. Fifteen Swift integrations pass, including 7,535 mutable drawings and 9,902 errors for the selected corpus, eager projection and all 33 prepared/caller-model lanes.
- Three strict C consumers pass against that SDK: ownership/error lifetimes, mutable public API and parse limits. TypeScript, formatting and diff checks pass.
- The [consolidated browser run](full-web.log.gz) passes all 369 checks across Chromium, Firefox and WebKit (12.9 minutes). Intel SDK and AppKit activation/focus release checks remain open. These are not waived.

Failed provisional owners no longer become borrowed-helper fallback owners. Config serialization writes now remain live instead of being overwritten by a pre-config snapshot. Both deliberate corrections have explicit public checks.

The two captured frame gates retain the old zero-metadata and +13-byte equations and order. The exact near-64-MiB distinguishing interval has source review but no new large E2E fixture; [frame audit](FRAME_GATE_REVIEW.md) explains the memory cost and remaining gap. No limit was raised and no tiny synthetic fixture is presented as proof.

[Source/package/SDK freeze](source-freeze.json) records 456 source/SDK input hashes and all 106 packed-file hashes. Logs and detailed browser reports are indexed with both source and archived hashes. The ordinary and raw VM reports bind the same candidate package. Source review: [raw adoption review](REVIEW.md).
