# Successful ER result packets — unreleased

Runtime source **`7339f38`**, test arithmetic follow-up **`4031a94`**. Successful retained ER results no longer make the full SVG → escaped JSON → JavaScript JSON.parse → TextEncoder round trip. Rust still serializes the exact old Result envelope into a bounded counting writer, including numeric sidecars, before one thin host factory reconstructs the public result. Errors, old exported JSON endpoints and old-WASM fallback remain unchanged.

**No 0.3 release.** This is a scoped improvement against the immediately preceding host artifact, not JS parity or recovery of the earlier arena regression. The latest complete release gate is still the older 126-condition failure.

## Independent VM

[Complete comparison](comparison.json): 24 conditions, three ER sizes plus Sequence-small negative controls, fixed/Canvas resources, parse/retained/source, 21 balanced triples, 1,000 ms warmup. Same AMD EPYC-Genoa VM, Chromium 156, Node 22.23.2 and pinned Pele 1.6.0. The complete 106-file control is the raw-host artifact (`51f1269`); candidate is this packet artifact. Files and full SVG/exact dimensions are checked before/after timing. [Every paired summary](paired-summary.json) preserves adverse and unchanged-control rows.

Positive change means slower. Paired change is the median of per-trial ratios; candidate/JS is a ratio of separate medians.

| ER size | Resources | Retained paired change | Faster trials | Candidate / JS |
| --- | --- | ---: | ---: | ---: |
| er/small | fixed | -12.75% | 20/21 | 6.29× |
| er/small | canvas | -10.82% | 20/21 | 5.16× |
| er/medium | fixed | -11.11% | 21/21 | 5.22× |
| er/medium | canvas | -9.41% | 20/21 | 4.81× |
| er/large | fixed | -10.28% | 21/21 | 5.11× |
| er/large | canvas | -14.89% | 19/21 | 4.41× |

Retained ER rendering improves **9.41–14.89%** in paired medians, with 19–21 faster trials per condition. It still takes **4.41–6.29× JS**. ER parse controls are mixed (−0.72% to +3.06% paired); source controls are mixed (−1.37% to +1.44%). Sequence controls range −1.45% to +2.12%. Separate medians sometimes move in the opposite direction. Do not attribute every control movement to this layer or combine gains across separate windows.

Warmup is serial; one page shares heap/GC/cache pressure across lanes. This ordinary diagnostic records package/platform/load evidence but does not have the raw harness's per-row quiet guard or individual trial-order records. It excludes startup/download, paint and GPU. Batch p95 is not a separately sampled operation tail. No RSS/allocation or full-library performance claim follows.

[Six fresh profiles](profile/summary.json) are included. They establish current attribution, not causal speedups. The previous profile's serialization-plus-final-encoding envelope was not a promised savings fraction; counting still scans and escapes strings.

## Compatibility and verification

- On the same packed runtime, 45 existing browser checks pass across Chromium/Firefox/WebKit (ordinary/prepared/raw callbacks, all 33 entries and lifecycle controls). Nine final packet checks pass across those engines. The first 54-check run had 48 passes and six ineffective-fixture failures; the corrected nine-check run is retained separately.
- Public differential checks compare forced legacy JSON versus packets across ordinary, prepared, raw and caller-owned roots: whole SVG, key order, dimension bits, exact legacy byte counts, Unicode/escaping, numeric presence and limits, nested calls, arbitrary original throws, final-audit precedence and repair.
- Actual NaN outer dimensions and successful packet delivery are positively asserted. A bounded accessibility string tests an envelope over 64 MiB; another produces native success just below 64 MiB followed by the specific combined graph-plus-metadata rejection. This is separate from the still-unexecuted initial raw zero-versus13-byte graph threshold.
- Four runtime files are browser/WASM only. The native/Swift source-and-SDK input aggregate exactly matches the preceding fresh SDK's 168 API conditions, 15 Swift integrations and three C consumers. That is unchanged-input proof, not a new Swift timing or a claim that WASM tests exercise Swift.
- The preceding raw source `51f1269` passed the complete 369-browser suite. That historical run does not become a new full-suite result for this packet layer. Current focused checks are scoped above.
- ER currently returns empty links. Generic LinkInfo serialization is retained, but no unreachable nonempty-ER-links case is claimed as tested.

Two parent fixture assumptions failed before correction: NaN fontSize/measuring values still yielded finite outer dimensions; standalone raw inline icons did not consume the proposed payload. Old/new result rows agreed. The final fixture uses NaN padding and escaped `accDescr` quotes. All failed attempts remain archived; no benchmark timing comes from those correctness runs. A later test-only formula ensures strict normal overflow rather than possible equality; existing successful cross-engine records already exceeded the threshold.

[Independent evidence audit](AUDIT.md) confirms all scoped results, including adverse controls and the tested-fixture provenance. Its archived script preserves the original workspace paths; the public summarizer runs directly here.

[Review](REVIEW.md), [design](DESIGN.md), [source/package/SDK freeze](source-freeze.json) and [evidence index](evidence-index.json) retain exact scope. The review's execution status is historical source-review context; final logs and scalar/bit reports supply execution proof. `summarize.mjs` verifies archives, artifact identity and paired arithmetic without launching a consumer.
