# Semantic layout cursor default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +5.34% | 4 | 0.059883 | 0.055391 | 0.003867 | 15.48× |
| sankey/small | fixed | render-model | -16.23% | 21 | 0.055195 | 0.066250 | 0.023828 | 2.32× |
| sankey/small | fixed | render-source | +1.57% | 9 | 0.043320 | 0.043125 | 0.028125 | 1.54× |
| sankey/small | canvas | parse | +5.21% | 6 | 0.057500 | 0.053867 | 0.003711 | 15.49× |
| sankey/small | canvas | render-model | -14.19% | 21 | 0.059922 | 0.068594 | 0.026328 | 2.28× |
| sankey/small | canvas | render-source | +1.67% | 4 | 0.045430 | 0.045000 | 0.030000 | 1.51× |
| flowchart/medium | fixed | parse | +1.75% | 10 | 0.833750 | 0.825000 | 0.064570 | 12.91× |
| flowchart/medium | fixed | render-model | +8.96% | 2 | 0.945625 | 0.828438 | 0.257578 | 3.67× |
| flowchart/medium | fixed | render-source | +1.46% | 4 | 0.570312 | 0.561250 | 0.337187 | 1.69× |
| flowchart/medium | canvas | parse | +3.31% | 6 | 0.818125 | 0.789375 | 0.064336 | 12.72× |
| flowchart/medium | canvas | render-model | +5.69% | 2 | 0.931875 | 0.864375 | 0.276328 | 3.37× |
| flowchart/medium | canvas | render-source | +2.32% | 2 | 0.574219 | 0.564687 | 0.365234 | 1.57× |
| sankey/medium | fixed | parse | -9.20% | 18 | 0.202812 | 0.214219 | 0.019180 | 10.57× |
| sankey/medium | fixed | render-model | -11.69% | 21 | 0.293359 | 0.331641 | 0.170977 | 1.72× |
| sankey/medium | fixed | render-source | +2.04% | 0 | 0.210156 | 0.204609 | 0.186484 | 1.13× |
| sankey/medium | canvas | parse | -7.62% | 19 | 0.194375 | 0.212344 | 0.019219 | 10.11× |
| sankey/medium | canvas | render-model | -12.68% | 21 | 0.350313 | 0.396953 | 0.194219 | 1.80× |
| sankey/medium | canvas | render-source | +0.77% | 6 | 0.218906 | 0.215859 | 0.202031 | 1.08× |
| sankey/large | fixed | parse | -11.58% | 17 | 0.739688 | 0.818125 | 0.091133 | 8.12× |
| sankey/large | fixed | render-model | -12.46% | 21 | 1.157500 | 1.327500 | 0.715938 | 1.62× |
| sankey/large | fixed | render-source | +2.02% | 3 | 0.836875 | 0.818750 | 0.770938 | 1.09× |
| sankey/large | canvas | parse | -13.44% | 18 | 0.690313 | 0.817813 | 0.090469 | 7.63× |
| sankey/large | canvas | render-model | -12.51% | 21 | 1.193750 | 1.375937 | 0.751250 | 1.59× |
| sankey/large | canvas | render-source | +1.23% | 7 | 0.888438 | 0.885312 | 0.855937 | 1.04× |

Frozen05c23 candidate versus original e148 default; same4beb WASM. All24 rows and21 samples/lane retained,115 package bytes verified before/after,8 full2/update1 activation checks passed. Warm20925/fetch66699 exited0. No first-use or profiling execution. Paired sourcecontrols and adverse rows remain above; no cross-window compounding.

Scoped result: all6 retained Sankey rows lower11.69–16.23%,21/21pairs each; medium/large parse lower7.62–13.44%. Small parse remains5.21–5.34% adverse. All8 direct-source controls and all4 Flowchart parse/retained controls are adverse.14/24paired rows adverse overall; shared/default neutrality is not established. JS parse remains7.63–15.49× and retained1.59–2.32× faster. No cross-window causal claim about cursor-only gain.
