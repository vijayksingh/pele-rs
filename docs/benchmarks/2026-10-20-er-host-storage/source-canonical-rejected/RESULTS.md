# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +52.21% | 2 | 0.085742 | 0.057852 | 0.003789 | 22.63× |
| sankey/small | fixed | render-model | +70.44% | 0 | 0.116094 | 0.067031 | 0.024102 | 4.82× |
| sankey/small | fixed | render-source | +1.88% | 5 | 0.044414 | 0.043594 | 0.027891 | 1.59× |
| sankey/small | canvas | parse | +57.33% | 1 | 0.086523 | 0.056797 | 0.003867 | 22.37× |
| sankey/small | canvas | render-model | +70.22% | 0 | 0.117617 | 0.068398 | 0.026484 | 4.44× |
| sankey/small | canvas | render-source | +2.11% | 4 | 0.045586 | 0.044688 | 0.030195 | 1.51× |
| flowchart/medium | fixed | parse | +2.67% | 4 | 0.810625 | 0.786875 | 0.066641 | 12.16× |
| flowchart/medium | fixed | render-model | +13.56% | 2 | 0.965313 | 0.839063 | 0.268672 | 3.59× |
| flowchart/medium | fixed | render-source | +4.30% | 0 | 0.582969 | 0.560156 | 0.338125 | 1.72× |
| flowchart/medium | canvas | parse | +6.44% | 4 | 0.853125 | 0.802188 | 0.070078 | 12.17× |
| flowchart/medium | canvas | render-model | +7.91% | 3 | 0.923750 | 0.853438 | 0.260000 | 3.55× |
| flowchart/medium | canvas | render-source | +1.30% | 6 | 0.610469 | 0.605312 | 0.381797 | 1.60× |
| sankey/medium | fixed | parse | +52.94% | 5 | 0.321406 | 0.210547 | 0.018750 | 17.14× |
| sankey/medium | fixed | render-model | +68.92% | 0 | 0.565000 | 0.333594 | 0.170664 | 3.31× |
| sankey/medium | fixed | render-source | +0.04% | 10 | 0.204375 | 0.204375 | 0.195586 | 1.04× |
| sankey/medium | canvas | parse | +52.18% | 3 | 0.326953 | 0.221328 | 0.018789 | 17.40× |
| sankey/medium | canvas | render-model | +66.85% | 0 | 0.605781 | 0.360781 | 0.187852 | 3.22× |
| sankey/medium | canvas | render-source | +3.30% | 2 | 0.226797 | 0.219531 | 0.216250 | 1.05× |
| sankey/large | fixed | parse | +54.97% | 3 | 1.214688 | 0.795313 | 0.074922 | 16.21× |
| sankey/large | fixed | render-model | +65.93% | 0 | 2.193750 | 1.320000 | 0.681562 | 3.22× |
| sankey/large | fixed | render-source | -0.04% | 11 | 0.832500 | 0.832500 | 0.785312 | 1.06× |
| sankey/large | canvas | parse | +49.04% | 2 | 1.245313 | 0.826563 | 0.077734 | 16.02× |
| sankey/large | canvas | render-model | +63.47% | 0 | 2.305000 | 1.399375 | 0.788750 | 2.92× |
| sankey/large | canvas | render-source | +1.95% | 7 | 0.915937 | 0.892812 | 0.864375 | 1.06× |
