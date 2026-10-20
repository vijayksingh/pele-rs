# Generated fresh initializer default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

The first-use timer includes parse, edit setup, render and disposal. Source rows time direct source rendering.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | -1.45% | 17 | 0.042852 | 0.043320 | 0.028203 | 1.52× |
| sankey/small | fixed | parse-first-render | -9.55% | 17 | 0.136758 | 0.150391 | 0.028086 | 4.87× |
| sankey/small | fixed | parse-first-edited-render | +15.85% | 3 | 0.252813 | 0.219219 | 0.028164 | 8.98× |
| sankey/small | canvas | render-source | -0.43% | 15 | 0.044961 | 0.045313 | 0.029688 | 1.51× |
| sankey/small | canvas | parse-first-render | -6.64% | 17 | 0.146172 | 0.156328 | 0.031133 | 4.70× |
| sankey/small | canvas | parse-first-edited-render | +14.64% | 2 | 0.254688 | 0.223281 | 0.030664 | 8.31× |
| flowchart/medium | fixed | render-source | -0.98% | 16 | 0.562813 | 0.569063 | 0.312578 | 1.80× |
| flowchart/medium | fixed | parse-first-render | +3.01% | 7 | 1.913125 | 1.836875 | 0.389297 | 4.91× |
| flowchart/medium | fixed | parse-first-edited-render | +2.34% | 7 | 2.274375 | 2.209375 | 0.395156 | 5.76× |
| flowchart/medium | canvas | render-source | -2.79% | 20 | 0.560000 | 0.576875 | 0.323594 | 1.73× |
| flowchart/medium | canvas | parse-first-render | +3.37% | 3 | 1.928750 | 1.870625 | 0.400000 | 4.82× |
| flowchart/medium | canvas | parse-first-edited-render | +3.96% | 4 | 2.468125 | 2.251250 | 0.412188 | 5.99× |
| sankey/medium | fixed | render-source | -1.19% | 15 | 0.211172 | 0.212656 | 0.188828 | 1.12× |
| sankey/medium | fixed | parse-first-render | -19.38% | 20 | 0.472813 | 0.587656 | 0.215547 | 2.19× |
| sankey/medium | fixed | parse-first-edited-render | +0.55% | 7 | 0.851562 | 0.843125 | 0.197031 | 4.32× |
| sankey/medium | canvas | render-source | -0.17% | 11 | 0.240078 | 0.235234 | 0.208984 | 1.15× |
| sankey/medium | canvas | parse-first-render | -19.24% | 21 | 0.490938 | 0.614375 | 0.225391 | 2.18× |
| sankey/medium | canvas | parse-first-edited-render | +2.33% | 5 | 0.884687 | 0.863437 | 0.226641 | 3.90× |
| sankey/large | fixed | render-source | -0.30% | 11 | 0.843125 | 0.843125 | 0.780312 | 1.08× |
| sankey/large | fixed | parse-first-render | -20.74% | 21 | 1.740000 | 2.248750 | 0.812500 | 2.14× |
| sankey/large | fixed | parse-first-edited-render | -1.45% | 17 | 3.025000 | 3.076250 | 0.773125 | 3.91× |
| sankey/large | canvas | render-source | +1.77% | 3 | 0.899062 | 0.885312 | 0.843438 | 1.07× |
| sankey/large | canvas | parse-first-render | -18.91% | 21 | 1.925625 | 2.300625 | 0.939687 | 2.05× |
| sankey/large | canvas | parse-first-edited-render | -2.02% | 14 | 3.205000 | 3.266250 | 0.878438 | 3.65× |
