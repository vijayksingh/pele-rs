# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -3.79% | 12 | 0.056094 | 0.055430 | 0.003906 | 14.36× |
| sankey/small | fixed | render-model | -15.24% | 20 | 0.059375 | 0.068945 | 0.024141 | 2.46× |
| sankey/small | fixed | render-source | +1.53% | 7 | 0.044922 | 0.043516 | 0.029141 | 1.54× |
| sankey/small | canvas | parse | -9.28% | 19 | 0.049102 | 0.053203 | 0.003906 | 12.57× |
| sankey/small | canvas | render-model | -14.78% | 21 | 0.057500 | 0.067656 | 0.026250 | 2.19× |
| sankey/small | canvas | render-source | -0.61% | 12 | 0.044609 | 0.044883 | 0.030508 | 1.46× |
| flowchart/medium | fixed | parse | +6.69% | 3 | 0.848750 | 0.793437 | 0.067109 | 12.65× |
| flowchart/medium | fixed | render-model | +4.98% | 6 | 0.874062 | 0.839063 | 0.258906 | 3.38× |
| flowchart/medium | fixed | render-source | +2.01% | 3 | 0.568906 | 0.559219 | 0.351016 | 1.62× |
| flowchart/medium | canvas | parse | +1.68% | 8 | 0.823750 | 0.817812 | 0.063008 | 13.07× |
| flowchart/medium | canvas | render-model | +10.65% | 1 | 0.945938 | 0.844375 | 0.273984 | 3.45× |
| flowchart/medium | canvas | render-source | +3.19% | 1 | 0.592188 | 0.573437 | 0.357031 | 1.66× |
| sankey/medium | fixed | parse | -30.22% | 19 | 0.145078 | 0.209766 | 0.020586 | 7.05× |
| sankey/medium | fixed | render-model | -16.71% | 21 | 0.276484 | 0.332813 | 0.168906 | 1.64× |
| sankey/medium | fixed | render-source | -0.88% | 13 | 0.201484 | 0.203203 | 0.193516 | 1.04× |
| sankey/medium | canvas | parse | -31.73% | 19 | 0.157188 | 0.226875 | 0.019922 | 7.89× |
| sankey/medium | canvas | render-model | -14.05% | 19 | 0.310234 | 0.357734 | 0.192422 | 1.61× |
| sankey/medium | canvas | render-source | +0.77% | 10 | 0.224609 | 0.221250 | 0.216406 | 1.04× |
| sankey/large | fixed | parse | -35.31% | 20 | 0.510313 | 0.785938 | 0.077969 | 6.55× |
| sankey/large | fixed | render-model | -15.70% | 20 | 1.114062 | 1.328750 | 0.720469 | 1.55× |
| sankey/large | fixed | render-source | -0.71% | 12 | 0.826875 | 0.834063 | 0.835312 | 0.99× |
| sankey/large | canvas | parse | -34.32% | 19 | 0.595313 | 0.935000 | 0.091250 | 6.52× |
| sankey/large | canvas | render-model | -13.88% | 21 | 1.174688 | 1.390938 | 0.777813 | 1.51× |
| sankey/large | canvas | render-source | +0.90% | 7 | 0.875000 | 0.872813 | 0.864688 | 1.01× |
