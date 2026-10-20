# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +20.09% | 2 | 0.064219 | 0.054688 | 0.003672 | 17.49× |
| sankey/small | fixed | render-model | +24.70% | 0 | 0.084141 | 0.067695 | 0.024258 | 3.47× |
| sankey/small | fixed | render-source | +1.25% | 4 | 0.044570 | 0.043594 | 0.027891 | 1.60× |
| sankey/small | canvas | parse | +16.77% | 1 | 0.062617 | 0.053281 | 0.003594 | 17.42× |
| sankey/small | canvas | render-model | +21.01% | 0 | 0.094922 | 0.075625 | 0.028086 | 3.38× |
| sankey/small | canvas | render-source | -1.53% | 12 | 0.045703 | 0.045625 | 0.030977 | 1.48× |
| flowchart/medium | fixed | parse | +0.27% | 9 | 0.815938 | 0.820000 | 0.067734 | 12.05× |
| flowchart/medium | fixed | render-model | +5.19% | 4 | 0.884375 | 0.845937 | 0.252422 | 3.50× |
| flowchart/medium | fixed | render-source | -0.11% | 11 | 0.569063 | 0.571719 | 0.325078 | 1.75× |
| flowchart/medium | canvas | parse | +2.18% | 6 | 0.819688 | 0.796562 | 0.062461 | 13.12× |
| flowchart/medium | canvas | render-model | +6.13% | 4 | 0.909062 | 0.871250 | 0.267813 | 3.39× |
| flowchart/medium | canvas | render-source | +0.57% | 7 | 0.594375 | 0.591875 | 0.366641 | 1.62× |
| sankey/medium | fixed | parse | +4.92% | 8 | 0.236562 | 0.222031 | 0.019688 | 12.02× |
| sankey/medium | fixed | render-model | +21.69% | 0 | 0.414375 | 0.347500 | 0.167812 | 2.47× |
| sankey/medium | fixed | render-source | +1.81% | 7 | 0.211641 | 0.204688 | 0.186484 | 1.13× |
| sankey/medium | canvas | parse | +16.63% | 3 | 0.247109 | 0.213906 | 0.018867 | 13.10× |
| sankey/medium | canvas | render-model | +22.00% | 0 | 0.417031 | 0.343281 | 0.182109 | 2.29× |
| sankey/medium | canvas | render-source | +1.71% | 6 | 0.222969 | 0.217344 | 0.203750 | 1.09× |
| sankey/large | fixed | parse | +8.61% | 5 | 0.860313 | 0.793750 | 0.076953 | 11.18× |
| sankey/large | fixed | render-model | +19.80% | 0 | 1.593750 | 1.328125 | 0.707031 | 2.25× |
| sankey/large | fixed | render-source | +1.53% | 4 | 0.829375 | 0.815000 | 0.798750 | 1.04× |
| sankey/large | canvas | parse | +11.74% | 6 | 0.860000 | 0.764688 | 0.074258 | 11.58× |
| sankey/large | canvas | render-model | +18.33% | 0 | 1.638750 | 1.390625 | 0.802500 | 2.04× |
| sankey/large | canvas | render-source | +0.39% | 9 | 0.940000 | 0.942813 | 0.895000 | 1.05× |
