# Rejected single canonical host string index

Separate experiment after dense-array `7701135`; **rejected and reverted**. It replaced two invariant-equal permanent host string maps with one map. Temporary observation IDs retained their separate precedence and lifetime. This is different from the earlier rejected object-ID WeakMap-to-Map experiment.

The 24-row VM comparison used 21 trials and 1000 ms warmup. ER parse results split three lower and three higher paired medians:

| ER parse | Paired change | Faster pairs |
|---|---:|---:|
| Small fixed | −1.63% | 15/21 |
| Small Canvas | **+5.25%** | 6/21 |
| Medium fixed | −0.59% | 11/21 |
| Medium Canvas | −1.00% | 13/21 |
| Large fixed | **+2.96%** | 8/21 |
| Large Canvas | **+0.25%** | 10/21 |

All 24 rows and controls remain in [RESULTS.md](RESULTS.md), [summary.json](summary.json), and compressed [comparison.json.gz](comparison.json.gz). No adverse result is discarded as noise; no general latency or heap improvement is claimed. Passing 84 browser checks in 46.6 seconds establishes the tested correctness slice, not grounds for performance adoption.

The frozen control map exactly matches the accepted dense-array candidate. Only `host-er-store.js` differs; WASM bytes and recorded native/SDK input identity are unchanged. [package-freeze.json.gz](package-freeze.json.gz) preserves both 106-file package maps, and the manifest binds the exact patch/source hashes. This appendix verifies these recorded relationships; it performs no new physical remote rehash, browser run or build.

**No first-use run and no native source change** belong to this experiment. Dense-array's first-use, Swift and allocation results remain separate; they cannot establish performance for this rejected candidate. No candidate binary/package is included or substituted for accepted runtime evidence.

[FINDING.md](FINDING.md), [BOUNDARY_REVIEW.md](BOUNDARY_REVIEW.md) and [manifest.json](manifest.json) are original source-stage records. Their “no measurement/not run” statements describe that earlier point and are superseded only for the later browser/comparison evidence retained here. The patch is archived for identification, not application.
