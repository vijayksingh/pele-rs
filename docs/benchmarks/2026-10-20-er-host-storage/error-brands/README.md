# ER error-brand allocation removal — unreleased

Source **a472853** replaces per-object `Object.keys(...).find(...)` with eight ordered comparisons. The constructor object remains captured at module initialization; prototype values are read dynamically in the same short-circuit order, with one `Object.getPrototypeOf` call. Rust parsing/layout and public behavior stay unchanged. This small JS storage helper runs during graph classification and auditing.

**No 0.3 release or JS parity claim.** This compares against entry-frame source 53b035e, not the original arena or a complete release baseline.

## Frozen VM result

[Comparison](comparison.json): 24 conditions, three ER sizes plus Sequence-small controls, two resource profiles, three stages, 21 balanced triples and 1,000 ms warmup. AMD EPYC-Genoa VM, Chromium 156. Both 106-file retained package inventories match the report; control exactly matches the previous frame candidate. **Only host-er-store.js differs.** [Identity](identity.json) records full commit/source hashes and the native input aggregate 8f4fbc26… unchanged from the frame/lexer freeze. This JS-only layer required no SDK rebuild and supplies no new Swift timing.

Positive paired change means slower. Paired change is median per-trial candidate/control ratio; candidate/JS is ratio of separate medians.

| Retained ER | Fixed paired | Canvas paired | Faster trials, fixed/canvas | Candidate/JS, fixed/canvas |
| --- | ---: | ---: | ---: | ---: |
| Small | -8.52% | -7.44% | 18/21, 18/21 | 4.30×, 4.03× |
| Medium | -8.84% | -10.01% | 20/21, 18/21 | 3.95×, 3.48× |
| Large | -9.94% | -8.58% | 20/21, 19/21 | 4.03×, 3.35× |

Retained ER improves 7.44–10.01%, with 18–20 faster trials; it remains 3.35–4.30× JS. [All paired rows](paired-summary.json) retain adverse controls: five of six ER parse conditions are slower, +0.63%…+3.84%; large Canvas parse improves 1.26%. ER source ranges −0.52%…+2.24%. Sequence controls range −0.77%…+2.44%. These movements cannot all be attributed to the helper change. Do not add gains across separate measurement windows.

Shared-page heap/GC/cache pressure and serial warmup remain limitations. This diagnostic lacks per-row quiet guards and individual trial-order records. It verifies full SVG and exact dimensions before/after timing and complete package hashes; it excludes startup, paint/GPU and RSS. Batch p95 is not an operation-tail sample. No broader performance claim follows.

## Public compatibility evidence

The final feature fixture passes in Chromium, Firefox and WebKit: **eight brands × two implementations × three engines = 48 observations**. Candidate is compared with pinned JS through normal rendering and standalone raw rendering. Whole results, own descriptor flags, prototype identity, cycles, Map/Set aliases, unread diagnostic accessors, helpers, callback throws and repair agree. All 24 paired brand comparisons match.

Resource **RangeError is intentionally wrapped** by both public APIs into `PeleError("Diagram is nested too deeply.", "limit")`; other error objects retain original thrown identity. The corrected fixture compares `error.name`, not minified constructor names.

Fixture attempts are retained, not hidden:

1. Initial run: 39 existing checks passed; three new tests failed because the private control-package URL was not served.
2. Next three failed because the bundled oracle lacks renderer.renderRaw; the fixture now uses its public standalone ER entry.
3. Next three failed on incorrect RangeError identity expectations; minified constructor-name assumptions were also corrected. Complete observations are archived.
4. Final corrected fixture: three passed against pinned JS. [Frozen fixture](fixture.spec.js), [logs](logs/final.log.gz), and per-engine final JSON retain execution proof. Type check log is included.

These are 39 existing checks plus 3 final feature checks, not a new full-library suite. No constructor-global-rebinding or duplicate-prototype runtime campaign is claimed; captured-constructor/precedence/single-read equivalence was reviewed in source.

Run `node summarize.mjs` here to verify hashes, paired arithmetic and all final observations without starting a browser. [Evidence index](evidence-index.json) hashes every archived input. Original logs and failed reports are compressed to keep this bundle small.
