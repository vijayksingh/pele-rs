# Numeric live-read ABI warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -6.91% | 19 | 0.052656 | 0.057266 | 0.003711 | 14.19× |
| sankey/small | fixed | render-model | +125.23% | 0 | 0.152539 | 0.067891 | 0.024375 | 6.26× |
| sankey/small | fixed | render-source | +1.37% | 5 | 0.043711 | 0.042930 | 0.027539 | 1.59× |
| sankey/small | canvas | parse | -7.86% | 17 | 0.049805 | 0.052539 | 0.003711 | 13.42× |
| sankey/small | canvas | render-model | +115.61% | 0 | 0.143867 | 0.066367 | 0.026289 | 5.47× |
| sankey/small | canvas | render-source | +2.10% | 7 | 0.046484 | 0.045703 | 0.031641 | 1.47× |
| flowchart/medium | fixed | parse | +7.19% | 4 | 0.857187 | 0.800313 | 0.069023 | 12.42× |
| flowchart/medium | fixed | render-model | +3.40% | 6 | 0.877500 | 0.847500 | 0.264609 | 3.32× |
| flowchart/medium | fixed | render-source | -0.44% | 12 | 0.596250 | 0.594844 | 0.351328 | 1.70× |
| flowchart/medium | canvas | parse | +0.73% | 10 | 0.793125 | 0.792500 | 0.062344 | 12.72× |
| flowchart/medium | canvas | render-model | -0.17% | 11 | 0.949687 | 0.912188 | 0.299063 | 3.18× |
| flowchart/medium | canvas | render-source | -0.13% | 11 | 0.600781 | 0.602500 | 0.364063 | 1.65× |
| sankey/medium | fixed | parse | -33.69% | 18 | 0.137578 | 0.207344 | 0.018672 | 7.37× |
| sankey/medium | fixed | render-model | +97.24% | 0 | 0.645469 | 0.326719 | 0.166055 | 3.89× |
| sankey/medium | fixed | render-source | +2.37% | 5 | 0.211875 | 0.205000 | 0.192812 | 1.10× |
| sankey/medium | canvas | parse | -32.38% | 20 | 0.138672 | 0.209141 | 0.018711 | 7.41× |
| sankey/medium | canvas | render-model | +97.83% | 0 | 0.676719 | 0.346406 | 0.184922 | 3.66× |
| sankey/medium | canvas | render-source | +0.63% | 9 | 0.226250 | 0.222266 | 0.222813 | 1.02× |
| sankey/large | fixed | parse | -39.86% | 21 | 0.460938 | 0.758750 | 0.076328 | 6.04× |
| sankey/large | fixed | render-model | +89.85% | 0 | 2.561250 | 1.341250 | 0.716875 | 3.57× |
| sankey/large | fixed | render-source | +0.15% | 9 | 0.820938 | 0.820937 | 0.775937 | 1.06× |
| sankey/large | canvas | parse | -39.39% | 19 | 0.473437 | 0.791562 | 0.075586 | 6.26× |
| sankey/large | canvas | render-model | +86.72% | 0 | 2.570000 | 1.373125 | 0.752500 | 3.42× |
| sankey/large | canvas | render-source | +0.64% | 9 | 0.891250 | 0.885312 | 0.877500 | 1.02× |
