# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -4.51% | 16 | 0.052070 | 0.056328 | 0.003750 | 13.89× |
| sankey/small | fixed | render-model | -13.51% | 21 | 0.059297 | 0.069727 | 0.024414 | 2.43× |
| sankey/small | fixed | render-source | -2.23% | 18 | 0.042812 | 0.043828 | 0.027734 | 1.54× |
| sankey/small | canvas | parse | -5.77% | 18 | 0.051602 | 0.054375 | 0.003867 | 13.34× |
| sankey/small | canvas | render-model | -14.42% | 20 | 0.058008 | 0.068320 | 0.026484 | 2.19× |
| sankey/small | canvas | render-source | -0.53% | 13 | 0.044453 | 0.044805 | 0.029883 | 1.49× |
| flowchart/medium | fixed | parse | +1.22% | 9 | 0.830000 | 0.819062 | 0.064727 | 12.82× |
| flowchart/medium | fixed | render-model | -0.91% | 11 | 0.891250 | 0.907812 | 0.273359 | 3.26× |
| flowchart/medium | fixed | render-source | -0.03% | 11 | 0.578594 | 0.578750 | 0.336875 | 1.72× |
| flowchart/medium | canvas | parse | +2.36% | 6 | 0.841250 | 0.826875 | 0.065391 | 12.86× |
| flowchart/medium | canvas | render-model | +3.30% | 8 | 0.915313 | 0.899375 | 0.282891 | 3.24× |
| flowchart/medium | canvas | render-source | -0.11% | 11 | 0.584531 | 0.587188 | 0.341641 | 1.71× |
| sankey/medium | fixed | parse | -29.08% | 20 | 0.140078 | 0.201016 | 0.018711 | 7.49× |
| sankey/medium | fixed | render-model | -17.77% | 20 | 0.277344 | 0.339531 | 0.173242 | 1.60× |
| sankey/medium | fixed | render-source | +0.50% | 9 | 0.204609 | 0.204375 | 0.194141 | 1.05× |
| sankey/medium | canvas | parse | -29.77% | 19 | 0.144297 | 0.204766 | 0.018672 | 7.73× |
| sankey/medium | canvas | render-model | -15.37% | 20 | 0.297891 | 0.346250 | 0.181094 | 1.64× |
| sankey/medium | canvas | render-source | +0.54% | 9 | 0.216250 | 0.215312 | 0.214922 | 1.01× |
| sankey/large | fixed | parse | -35.00% | 19 | 0.510000 | 0.779844 | 0.075742 | 6.73× |
| sankey/large | fixed | render-model | -15.03% | 21 | 1.115000 | 1.340625 | 0.708438 | 1.57× |
| sankey/large | fixed | render-source | +1.06% | 7 | 0.853125 | 0.840625 | 0.817188 | 1.04× |
| sankey/large | canvas | parse | -35.28% | 17 | 0.507812 | 0.785625 | 0.077305 | 6.57× |
| sankey/large | canvas | render-model | -15.71% | 21 | 1.189687 | 1.392813 | 0.755000 | 1.58× |
| sankey/large | canvas | render-source | -1.42% | 18 | 0.874687 | 0.888438 | 0.845937 | 1.03× |
