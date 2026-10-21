# ER entry read frame — unreleased

Runtime source **`53b035e`**. Normal retained ER rendering reuses data already visited by its required entry audit through a bounded transient read frame. Public JS objects remain authoritative. The view is discarded before callbacks/coercion; nested renders use live reads. Raw rendering, source rendering, methods and construction do not select this frame. Exceeding its optional scratch cap falls back to live imports.

**No 0.3 release or JS parity claim.** This measures against the preceding lexical-dispatch package. All six ER retained-render rows improve, while four ER parse controls are adverse. The latest complete release gate remains the earlier 126-condition failure; this diagnostic does not replace it.

## Measured result

One Linux Chromium page, independent control/candidate WASM instances and pinned JS, 21 trials per row, at least 1000 ms warmup per lane. ER retained-render paired medians improve **11.02–14.37%**, with **20–21/21** faster pairs. Candidate retained rendering still costs **3.82–5.51× JS** by separate medians.

ER parse controls are mixed: small/fixed **+3.85%**, medium/fixed **+1.60%**, medium/Canvas **+4.80%**, large/fixed **+4.42%**; the other two improve slightly. Do not infer that every API improved or dismiss adverse controls.

| Case | Profile | Stage | Paired change | Faster pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sequence/small | fixed | parse | -0.89% | 12/21 | 15.94× |
| sequence/small | fixed | render-model | +0.13% | 10/21 | 7.92× |
| sequence/small | fixed | render-source | -0.42% | 12/21 | 2.99× |
| sequence/small | canvas | parse | -2.22% | 11/21 | 16.09× |
| sequence/small | canvas | render-model | -0.37% | 11/21 | 5.88× |
| sequence/small | canvas | render-source | -1.32% | 14/21 | 2.60× |
| er/small | fixed | parse | +3.85% | 5/21 | 12.20× |
| er/small | fixed | render-model | -13.70% | 21/21 | 5.51× |
| er/small | fixed | render-source | -0.67% | 12/21 | 2.51× |
| er/small | canvas | parse | -0.74% | 13/21 | 11.75× |
| er/small | canvas | render-model | -14.37% | 21/21 | 4.53× |
| er/small | canvas | render-source | -0.72% | 16/21 | 2.26× |
| er/medium | fixed | parse | +1.60% | 8/21 | 9.72× |
| er/medium | fixed | render-model | -12.73% | 21/21 | 4.62× |
| er/medium | fixed | render-source | -0.83% | 14/21 | 2.22× |
| er/medium | canvas | parse | +4.80% | 3/21 | 8.79× |
| er/medium | canvas | render-model | -11.02% | 20/21 | 4.00× |
| er/medium | canvas | render-source | -0.95% | 15/21 | 2.00× |
| er/large | fixed | parse | +4.42% | 5/21 | 8.66× |
| er/large | fixed | render-model | -12.41% | 20/21 | 4.37× |
| er/large | fixed | render-source | -1.06% | 14/21 | 2.24× |
| er/large | canvas | parse | -0.85% | 11/21 | 5.78× |
| er/large | canvas | render-model | -12.40% | 20/21 | 3.82× |
| er/large | canvas | render-source | -0.59% | 14/21 | 1.87× |

Paired median change uses each candidate/control trial ratio. The JS ratio uses separate medians. These are distinct statistics. [comparison.json](comparison.json) retains all raw samples and negative controls; [paired-summary.json](paired-summary.json) contains recomputed paired statistics.

## Profile and limits

Six 1500 ms Canvas profiles cover ER medium source, parse and retained operations in JS and WASM. Retained WASM attribution: audit **10.23% self / 20.55% inclusive**; error-brand dispatcher plus its find closure **6.43% self**; JS frame encoding **4.22% self**; native frame operations **2.67% self**. Groups overlap: do not sum them or infer causal savings. Compressed raw profiles and the original [profile summary](profile/summary.json) are included.

The comparator has no per-row quiet-process guard. It checks the isolated high-resolution clock, package hashes and full SVG/IEEE dimensions before/after, and records host load/memory. One page shares GC/cache pressure. Observed raw order/timestamps are not saved: the hash-verified schedule gives each lane seven first positions per row and control/candidate precedence 11/10. Retained setup and a warm render are outside batch timing; timed retained operations include entry audit/frame costs. Parse includes disposal.

The 8 MiB cap covers native frame bytes plus offsets, not overlapping JS/native process peak. No allocation or peak-memory claim follows. Native/Swift inputs remain the df3597c aggregate `8f4fbc262e2ce2b1784823f4dd635cb6e725b0084a7a75a507c36690763b6bea` (351 files); no new native/Swift timing is claimed.

## Identity and validation

The freeze verifies 447 immutable source/consumer/web blobs at 53b035e and 95 directly packaged source files. All 106 candidate files match the retained local package and profile report; all 106 control files match the preceding lexical artifact. A fresh independent remote SHA walk confirms both frozen VM directories after measurement. Generated WASM identity is verified, not independently reproduced by a new build.

See [source-freeze.json](source-freeze.json), [independent-vm-rehash.json](independent-vm-rehash.json), [native-input-check.json](native-input-check.json), and [AUDIT.md](AUDIT.md). Runtime/package copies are not duplicated here.

| Attempt | Result | Interpretation |
|---|---|---|
| Initial browser slice | 53 passed, 7 failed | Six packet fixtures did not disable the new capability; one frame fixture assumed a stable NaN payload from a fresh literal. |
| Packet + frame retry | 10 passed, 2 failed | All nine packet checks passed; Firefox/WebKit still rejected the overly specific initial NaN-payload assertion. |
| Final frame + other-family controls | 18 passed | Frame passes in all three engines; 15 other-family controls pass. Final fixture checks NaN input and preservation of its observed bits. |
| Additional helper controls | 3 passed | Shared helper identity controls pass in all three engines. |
| WASM build and TypeScript | Passed | Build/type logs retained; no new native build claimed. |

Runtime remained unchanged during fixture corrections. Failed-attempt logs are preserved. **Final-run per-case JSON was lost**: its result is supported by the retained final log. Complete earlier retry JSON remains under `tests/attempt2/`; it is not presented as the missing final reports. Counts describe different slices and must not be added as unique coverage.

## Reproduce summaries

From this folder:

```sh
python3 summarize.py
```

This deterministic saved-data verifier recomputes every timing row and all six profile top lists, cross-checks frozen tables, writes `paired-summary.json`, and creates `evidence-index.json` with sizes/SHA256 values for every other bundled file. Gzip uses `mtime=0`. It launches no compiler, browser, consumer, timing or network operation. `audit.json` preserves the original independent audit; the portable summarizer does not rerun remote/source verification.

To repeat the diagnostic from the repository root, provide independent immutable package directories matching the recorded hash tables:

```sh
node scripts/perf/compare-web-vm.mjs \
  --control "$CONTROL_PACKAGE" --candidate "$CANDIDATE_PACKAGE" \
  --cases sequence/small,er/small,er/medium,er/large \
  --profiles fixed,canvas --stages parse,render-model,render-source \
  --trials 21 --warmup-ms 1000 --output "$OUTPUT/comparison.json"

node scripts/perf/profile-web.mjs \
  --package "$CANDIDATE_PACKAGE" --cases er/medium \
  --duration-ms 1500 --output "$OUTPUT/profile"
```

Use the pinned oracle and harness hashes. Run alone; changed inputs constitute a new comparison. The complete release gate remains separate.
