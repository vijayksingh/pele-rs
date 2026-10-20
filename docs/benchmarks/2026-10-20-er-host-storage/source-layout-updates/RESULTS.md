# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +18.70% | 6 | 0.067695 | 0.056484 | 0.003906 | 17.33× |
| sankey/small | fixed | render-model | +1.72% | 6 | 0.070547 | 0.069336 | 0.024258 | 2.91× |
| sankey/small | fixed | render-source | -1.87% | 16 | 0.043125 | 0.043906 | 0.028125 | 1.53× |
| sankey/small | canvas | parse | +16.34% | 1 | 0.062500 | 0.053281 | 0.003984 | 15.69× |
| sankey/small | canvas | render-model | +2.80% | 7 | 0.070391 | 0.068711 | 0.026406 | 2.67× |
| sankey/small | canvas | render-source | +1.16% | 8 | 0.044648 | 0.044219 | 0.029883 | 1.49× |
| flowchart/medium | fixed | parse | +4.20% | 6 | 0.829062 | 0.798125 | 0.068398 | 12.12× |
| flowchart/medium | fixed | render-model | +1.89% | 7 | 0.864375 | 0.847500 | 0.264609 | 3.27× |
| flowchart/medium | fixed | render-source | +0.03% | 10 | 0.578438 | 0.580781 | 0.331328 | 1.75× |
| flowchart/medium | canvas | parse | +3.04% | 3 | 0.820312 | 0.804688 | 0.062539 | 13.12× |
| flowchart/medium | canvas | render-model | +1.22% | 8 | 0.905625 | 0.907812 | 0.277344 | 3.27× |
| flowchart/medium | canvas | render-source | +1.06% | 7 | 0.608750 | 0.604531 | 0.375703 | 1.62× |
| sankey/medium | fixed | parse | +11.03% | 3 | 0.228750 | 0.206406 | 0.018711 | 12.23× |
| sankey/medium | fixed | render-model | +6.71% | 0 | 0.357422 | 0.336250 | 0.169570 | 2.11× |
| sankey/medium | fixed | render-source | +0.11% | 9 | 0.207188 | 0.207578 | 0.186328 | 1.11× |
| sankey/medium | canvas | parse | +6.34% | 6 | 0.235938 | 0.225859 | 0.018945 | 12.45× |
| sankey/medium | canvas | render-model | +7.55% | 0 | 0.367578 | 0.340859 | 0.181523 | 2.02× |
| sankey/medium | canvas | render-source | -0.93% | 13 | 0.219063 | 0.219922 | 0.199687 | 1.10× |
| sankey/large | fixed | parse | +6.80% | 6 | 0.851875 | 0.803125 | 0.108164 | 7.88× |
| sankey/large | fixed | render-model | +9.39% | 0 | 1.496875 | 1.368750 | 0.733125 | 2.04× |
| sankey/large | fixed | render-source | -1.64% | 14 | 0.909688 | 0.910937 | 0.878437 | 1.04× |
| sankey/large | canvas | parse | +5.78% | 8 | 0.870313 | 0.811562 | 0.076445 | 11.38× |
| sankey/large | canvas | render-model | +7.58% | 0 | 1.518125 | 1.400625 | 0.779844 | 1.95× |
| sankey/large | canvas | render-source | -0.48% | 12 | 0.914375 | 0.905000 | 0.868125 | 1.05× |
