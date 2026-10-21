# Sparse source ingress default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

The first-use timer includes parse, edit setup, render and disposal. Source rows time direct source rendering.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | +1.46% | 4 | 0.043711 | 0.043164 | 0.028047 | 1.56× |
| sankey/small | fixed | parse-first-render | -3.98% | 17 | 0.139063 | 0.145781 | 0.028828 | 4.82× |
| sankey/small | fixed | parse-first-edited-render | +20.27% | 0 | 0.273516 | 0.223594 | 0.028242 | 9.68× |
| sankey/small | canvas | render-source | +0.87% | 5 | 0.045156 | 0.044766 | 0.029766 | 1.52× |
| sankey/small | canvas | parse-first-render | -7.04% | 18 | 0.142695 | 0.154180 | 0.030820 | 4.63× |
| sankey/small | canvas | parse-first-edited-render | +22.20% | 1 | 0.268750 | 0.223125 | 0.031641 | 8.49× |
| flowchart/medium | fixed | render-source | -0.27% | 11 | 0.587187 | 0.589219 | 0.340000 | 1.73× |
| flowchart/medium | fixed | parse-first-render | +1.62% | 6 | 1.841250 | 1.813750 | 0.349141 | 5.27× |
| flowchart/medium | fixed | parse-first-edited-render | +6.06% | 7 | 2.297500 | 2.221250 | 0.331875 | 6.92× |
| flowchart/medium | canvas | render-source | -0.55% | 11 | 0.575625 | 0.578125 | 0.344922 | 1.67× |
| flowchart/medium | canvas | parse-first-render | +2.47% | 7 | 1.896875 | 1.860000 | 0.354531 | 5.35× |
| flowchart/medium | canvas | parse-first-edited-render | +2.14% | 5 | 2.260000 | 2.205000 | 0.409687 | 5.52× |
| sankey/medium | fixed | render-source | +1.89% | 0 | 0.203203 | 0.199844 | 0.187266 | 1.09× |
| sankey/medium | fixed | parse-first-render | -18.47% | 21 | 0.495781 | 0.600781 | 0.210586 | 2.35× |
| sankey/medium | fixed | parse-first-edited-render | +2.31% | 9 | 0.893438 | 0.872813 | 0.225625 | 3.96× |
| sankey/medium | canvas | render-source | +1.64% | 6 | 0.222578 | 0.220703 | 0.206563 | 1.08× |
| sankey/medium | canvas | parse-first-render | -20.67% | 21 | 0.476719 | 0.596563 | 0.198906 | 2.40× |
| sankey/medium | canvas | parse-first-edited-render | +2.59% | 4 | 0.906875 | 0.881562 | 0.218516 | 4.15× |
| sankey/large | fixed | render-source | +1.24% | 5 | 0.841875 | 0.831563 | 0.779687 | 1.08× |
| sankey/large | fixed | parse-first-render | -20.55% | 19 | 1.765625 | 2.208125 | 0.885938 | 1.99× |
| sankey/large | fixed | parse-first-edited-render | +1.81% | 6 | 3.115000 | 3.127500 | 0.781875 | 3.98× |
| sankey/large | canvas | render-source | +0.99% | 4 | 0.889687 | 0.882812 | 0.850000 | 1.05× |
| sankey/large | canvas | parse-first-render | -21.04% | 18 | 1.796250 | 2.277500 | 0.856563 | 2.10× |
| sankey/large | canvas | parse-first-edited-render | -2.01% | 11 | 3.238750 | 3.288750 | 0.907500 | 3.57× |
