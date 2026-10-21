# Accepted-baseline warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +2.18% | 9 | 0.055703 | 0.054609 | 0.004023 | 13.84× |
| sankey/small | fixed | render-model | +13.78% | 2 | 0.074805 | 0.065312 | 0.024219 | 3.09× |
| sankey/small | fixed | render-source | +1.56% | 1 | 0.043398 | 0.042734 | 0.028047 | 1.55× |
| sankey/small | canvas | parse | -1.08% | 12 | 0.057578 | 0.055977 | 0.004219 | 13.65× |
| sankey/small | canvas | render-model | +13.63% | 1 | 0.076523 | 0.067344 | 0.026172 | 2.92× |
| sankey/small | canvas | render-source | +2.40% | 3 | 0.045352 | 0.044570 | 0.030430 | 1.49× |
| flowchart/medium | fixed | parse | +6.90% | 6 | 0.851875 | 0.802813 | 0.060430 | 14.10× |
| flowchart/medium | fixed | render-model | +8.78% | 0 | 0.972813 | 0.837188 | 0.258516 | 3.76× |
| flowchart/medium | fixed | render-source | +0.48% | 9 | 0.582812 | 0.578281 | 0.337891 | 1.72× |
| flowchart/medium | canvas | parse | +3.45% | 4 | 0.814375 | 0.785312 | 0.060430 | 13.48× |
| flowchart/medium | canvas | render-model | +2.51% | 9 | 0.900000 | 0.901875 | 0.290859 | 3.09× |
| flowchart/medium | canvas | render-source | +0.19% | 9 | 0.579062 | 0.576563 | 0.354687 | 1.63× |
| sankey/medium | fixed | parse | -27.53% | 19 | 0.161250 | 0.237891 | 0.018945 | 8.51× |
| sankey/medium | fixed | render-model | +11.56% | 0 | 0.372891 | 0.332813 | 0.172305 | 2.16× |
| sankey/medium | fixed | render-source | +2.06% | 5 | 0.208359 | 0.203750 | 0.195391 | 1.07× |
| sankey/medium | canvas | parse | -25.63% | 19 | 0.163047 | 0.217500 | 0.019297 | 8.45× |
| sankey/medium | canvas | render-model | +12.87% | 1 | 0.385469 | 0.340625 | 0.183047 | 2.11× |
| sankey/medium | canvas | render-source | +0.63% | 8 | 0.230234 | 0.225625 | 0.209375 | 1.10× |
| sankey/large | fixed | parse | -33.29% | 21 | 0.571250 | 0.861250 | 0.079531 | 7.18× |
| sankey/large | fixed | render-model | +14.55% | 2 | 1.493125 | 1.302500 | 0.728750 | 2.05× |
| sankey/large | fixed | render-source | +0.39% | 8 | 0.875000 | 0.857500 | 0.798125 | 1.10× |
| sankey/large | canvas | parse | -30.36% | 20 | 0.557187 | 0.793125 | 0.077227 | 7.21× |
| sankey/large | canvas | render-model | +13.66% | 4 | 1.530000 | 1.345000 | 0.744375 | 2.06× |
| sankey/large | canvas | render-source | +2.17% | 4 | 0.908125 | 0.882187 | 0.852500 | 1.07× |
