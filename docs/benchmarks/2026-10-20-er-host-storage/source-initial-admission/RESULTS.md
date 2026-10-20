# Fused initial-admission default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +10.22% | 5 | 0.061641 | 0.055898 | 0.003750 | 16.44× |
| sankey/small | fixed | render-model | +2.94% | 6 | 0.068477 | 0.066523 | 0.024297 | 2.82× |
| sankey/small | fixed | render-source | -0.72% | 14 | 0.042852 | 0.043086 | 0.027617 | 1.55× |
| sankey/small | canvas | parse | +8.33% | 5 | 0.061875 | 0.056797 | 0.003711 | 16.67× |
| sankey/small | canvas | render-model | +0.74% | 9 | 0.071914 | 0.071250 | 0.026484 | 2.72× |
| sankey/small | canvas | render-source | -0.44% | 12 | 0.044492 | 0.044531 | 0.031367 | 1.42× |
| flowchart/medium | fixed | parse | +3.83% | 5 | 0.855625 | 0.827500 | 0.064141 | 13.34× |
| flowchart/medium | fixed | render-model | +5.05% | 2 | 0.870938 | 0.835000 | 0.278047 | 3.13× |
| flowchart/medium | fixed | render-source | -0.73% | 14 | 0.567656 | 0.573125 | 0.350938 | 1.62× |
| flowchart/medium | canvas | parse | +7.12% | 2 | 0.865313 | 0.806250 | 0.069414 | 12.47× |
| flowchart/medium | canvas | render-model | +5.68% | 5 | 0.905625 | 0.854375 | 0.270469 | 3.35× |
| flowchart/medium | canvas | render-source | -0.90% | 15 | 0.571094 | 0.578281 | 0.363594 | 1.57× |
| sankey/medium | fixed | parse | +2.19% | 8 | 0.219531 | 0.221328 | 0.019063 | 11.52× |
| sankey/medium | fixed | render-model | +10.79% | 1 | 0.367266 | 0.333281 | 0.169609 | 2.17× |
| sankey/medium | fixed | render-source | -1.19% | 12 | 0.210859 | 0.215000 | 0.195859 | 1.08× |
| sankey/medium | canvas | parse | +0.42% | 10 | 0.212344 | 0.212891 | 0.019219 | 11.05× |
| sankey/medium | canvas | render-model | +13.04% | 3 | 0.400781 | 0.349844 | 0.195469 | 2.05× |
| sankey/medium | canvas | render-source | -0.18% | 11 | 0.219531 | 0.219922 | 0.217422 | 1.01× |
| sankey/large | fixed | parse | -7.12% | 15 | 0.749844 | 0.807344 | 0.076641 | 9.78× |
| sankey/large | fixed | render-model | +9.32% | 0 | 1.436563 | 1.308750 | 0.693750 | 2.07× |
| sankey/large | fixed | render-source | +1.38% | 5 | 0.829375 | 0.816875 | 0.769063 | 1.08× |
| sankey/large | canvas | parse | -4.51% | 12 | 0.816094 | 0.784375 | 0.079141 | 10.31× |
| sankey/large | canvas | render-model | +9.03% | 1 | 1.526250 | 1.395625 | 0.799219 | 1.91× |
| sankey/large | canvas | render-source | +1.00% | 7 | 0.922188 | 0.906875 | 0.869062 | 1.06× |

Frozen4beb candidate versus original e148 default, all24 rows/21 pairs. All115 package hashes and8 untimed full/update activation checks passed before/after; run12976 and fetch43179 exited0. No first-use execution.

No overall acceptance: four of six Sankey parse rows and all six retained rows are adverse; all four Flowchart parse/retained controls are adverse. Large Sankey parse is lower in this matched window, but JS parse remains9.78–16.67× faster across sizes. Do not compound deltas from previous windows or call the default controls neutral.
