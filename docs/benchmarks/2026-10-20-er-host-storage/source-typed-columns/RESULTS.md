# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -6.76% | 16 | 0.055469 | 0.060000 | 0.004219 | 13.15× |
| sankey/small | fixed | render-model | -14.12% | 20 | 0.057930 | 0.068008 | 0.024297 | 2.38× |
| sankey/small | fixed | render-source | +0.18% | 9 | 0.042773 | 0.042617 | 0.027500 | 1.56× |
| sankey/small | canvas | parse | -7.04% | 17 | 0.049180 | 0.052500 | 0.004063 | 12.11× |
| sankey/small | canvas | render-model | -15.57% | 21 | 0.058633 | 0.068906 | 0.026211 | 2.24× |
| sankey/small | canvas | render-source | +1.29% | 9 | 0.045938 | 0.045625 | 0.031406 | 1.46× |
| flowchart/medium | fixed | parse | +5.67% | 6 | 0.847813 | 0.795625 | 0.067656 | 12.53× |
| flowchart/medium | fixed | render-model | +3.07% | 8 | 0.875625 | 0.896875 | 0.267813 | 3.27× |
| flowchart/medium | fixed | render-source | +1.51% | 6 | 0.573594 | 0.568437 | 0.341094 | 1.68× |
| flowchart/medium | canvas | parse | +3.63% | 5 | 0.840937 | 0.803437 | 0.066641 | 12.62× |
| flowchart/medium | canvas | render-model | +8.24% | 2 | 0.925625 | 0.854062 | 0.274453 | 3.37× |
| flowchart/medium | canvas | render-source | +1.52% | 3 | 0.592187 | 0.586875 | 0.345938 | 1.71× |
| sankey/medium | fixed | parse | -36.09% | 21 | 0.131563 | 0.204141 | 0.019141 | 6.87× |
| sankey/medium | fixed | render-model | -14.50% | 21 | 0.282969 | 0.330312 | 0.170781 | 1.66× |
| sankey/medium | fixed | render-source | +2.54% | 5 | 0.211641 | 0.205312 | 0.198906 | 1.06× |
| sankey/medium | canvas | parse | -32.15% | 17 | 0.152422 | 0.213125 | 0.020664 | 7.38× |
| sankey/medium | canvas | render-model | -13.55% | 21 | 0.296016 | 0.343828 | 0.180234 | 1.64× |
| sankey/medium | canvas | render-source | -0.25% | 13 | 0.221641 | 0.221719 | 0.212187 | 1.04× |
| sankey/large | fixed | parse | -41.28% | 19 | 0.497500 | 0.813750 | 0.079531 | 6.26× |
| sankey/large | fixed | render-model | -13.40% | 20 | 1.145938 | 1.347812 | 0.709062 | 1.62× |
| sankey/large | fixed | render-source | +1.43% | 7 | 0.833750 | 0.824063 | 0.789062 | 1.06× |
| sankey/large | canvas | parse | -40.52% | 21 | 0.465781 | 0.767812 | 0.084688 | 5.50× |
| sankey/large | canvas | render-model | -13.07% | 21 | 1.200625 | 1.378438 | 0.770000 | 1.56× |
| sankey/large | canvas | render-source | +1.59% | 6 | 0.906563 | 0.883750 | 0.888438 | 1.02× |
