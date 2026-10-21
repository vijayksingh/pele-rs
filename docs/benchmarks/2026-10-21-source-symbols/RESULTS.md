# Source-symbols warm comparison

Source11f27516; default candidate c0ddd411; e148 control. Raw SHA `a5a897672d24d1e9d0a4d64ceb40b75ff95b27c6f7c84438d5d8646c8a12b425`.

All24 conditions,21 balanced pairs,1000ms warmup and36 activation/continuity checks verified. Exact117 candidate/113 control maps and harnesses frozen before/after. Only candidate WASM differs from a4; no isolated cross-window gain inferred. No first-use/profile run.

| Case | Resource | Stage | Paired Δ vs e148 | Lower /21 | Candidate /JS | Candidate /control medians |
|---|---|---|---:|---:|---:|---:|
| sankey/small | fixed | parse | -4.48% | 15 | 14.268× | 0.979× |
| sankey/small | fixed | render-model | +14.16% | 0 | 3.162× | 1.138× |
| sankey/small | fixed | render-source | -1.30% | 15 | 1.513× | 0.979× |
| sankey/small | canvas | parse | -5.85% | 14 | 14.082× | 0.947× |
| sankey/small | canvas | render-model | +10.21% | 2 | 2.903× | 1.096× |
| sankey/small | canvas | render-source | -1.39% | 14 | 1.476× | 0.992× |
| flowchart/medium | fixed | parse | +4.36% | 7 | 14.230× | 1.063× |
| flowchart/medium | fixed | render-model | +6.92% | 4 | 3.232× | 1.059× |
| flowchart/medium | fixed | render-source | +2.29% | 0 | 1.816× | 1.025× |
| flowchart/medium | canvas | parse | +5.95% | 6 | 13.748× | 1.059× |
| flowchart/medium | canvas | render-model | +3.42% | 2 | 3.148× | 1.046× |
| flowchart/medium | canvas | render-source | +0.61% | 9 | 1.745× | 1.006× |
| sankey/medium | fixed | parse | -31.58% | 19 | 7.720× | 0.686× |
| sankey/medium | fixed | render-model | +14.64% | 0 | 2.214× | 1.123× |
| sankey/medium | fixed | render-source | -1.50% | 15 | 1.040× | 0.986× |
| sankey/medium | canvas | parse | -33.11% | 18 | 7.925× | 0.690× |
| sankey/medium | canvas | render-model | +14.00% | 4 | 2.064× | 1.137× |
| sankey/medium | canvas | render-source | -2.72% | 18 | 1.004× | 0.970× |
| sankey/large | fixed | parse | -34.09% | 20 | 6.824× | 0.665× |
| sankey/large | fixed | render-model | +13.66% | 3 | 2.217× | 1.127× |
| sankey/large | fixed | render-source | -1.38% | 14 | 1.034× | 0.986× |
| sankey/large | canvas | parse | -34.25% | 17 | 6.459× | 0.664× |
| sankey/large | canvas | render-model | +13.87% | 0 | 2.093× | 1.136× |
| sankey/large | canvas | render-source | -0.75% | 15 | 1.019× | 0.997× |

All positive paired deltas are adverse, including negative controls. Paired median deltas and ratios of independent medians are different statistics. This window does not establish release parity or allocation effects.
