# SVG append default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | Indexed control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -1.35% | 11 | 0.058828 | 0.058008 | 0.003906 | 15.06× |
| sankey/small | fixed | render-model | -17.67% | 19 | 0.067734 | 0.082227 | 0.024258 | 2.79× |
| sankey/small | fixed | render-source | -18.74% | 21 | 0.044375 | 0.055039 | 0.028125 | 1.58× |
| sankey/small | canvas | parse | +3.07% | 6 | 0.053906 | 0.052500 | 0.004102 | 13.14× |
| sankey/small | canvas | render-model | -14.43% | 21 | 0.067695 | 0.080430 | 0.026562 | 2.55× |
| sankey/small | canvas | render-source | -20.95% | 21 | 0.044648 | 0.056641 | 0.030703 | 1.45× |
| flowchart/medium | fixed | parse | -0.23% | 11 | 0.819063 | 0.825625 | 0.069180 | 11.84× |
| flowchart/medium | fixed | render-model | +2.06% | 6 | 0.857500 | 0.833438 | 0.262813 | 3.26× |
| flowchart/medium | fixed | render-source | -1.16% | 12 | 0.558437 | 0.568437 | 0.332031 | 1.68× |
| flowchart/medium | canvas | parse | -0.16% | 11 | 0.791250 | 0.789375 | 0.066562 | 11.89× |
| flowchart/medium | canvas | render-model | +2.63% | 3 | 0.927188 | 0.848125 | 0.266406 | 3.48× |
| flowchart/medium | canvas | render-source | -1.01% | 15 | 0.568750 | 0.574531 | 0.361719 | 1.57× |
| sankey/medium | fixed | parse | +4.69% | 5 | 0.221641 | 0.204922 | 0.021016 | 10.55× |
| sankey/medium | fixed | render-model | -18.08% | 21 | 0.342344 | 0.412813 | 0.167148 | 2.05× |
| sankey/medium | fixed | render-source | -25.22% | 21 | 0.228906 | 0.304219 | 0.202617 | 1.13× |
| sankey/medium | canvas | parse | +1.81% | 10 | 0.216719 | 0.238281 | 0.019844 | 10.92× |
| sankey/medium | canvas | render-model | -17.03% | 20 | 0.342031 | 0.415000 | 0.179805 | 1.90× |
| sankey/medium | canvas | render-source | -23.61% | 21 | 0.218203 | 0.286328 | 0.211562 | 1.03× |
| sankey/large | fixed | parse | +3.18% | 9 | 0.800625 | 0.775937 | 0.081016 | 9.88× |
| sankey/large | fixed | render-model | -18.14% | 21 | 1.306875 | 1.597500 | 0.707500 | 1.85× |
| sankey/large | fixed | render-source | -26.04% | 21 | 0.821875 | 1.117500 | 0.820625 | 1.00× |
| sankey/large | canvas | parse | +2.31% | 9 | 0.787812 | 0.795625 | 0.122422 | 6.44× |
| sankey/large | canvas | render-model | -17.77% | 21 | 1.353750 | 1.656250 | 0.761875 | 1.78× |
| sankey/large | canvas | render-source | -24.89% | 21 | 0.901875 | 1.210000 | 0.908125 | 0.99× |
