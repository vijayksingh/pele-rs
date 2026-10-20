# Sparse source ingress default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

The first-use timer includes parse, edit setup, render and disposal. Source rows time direct source rendering.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | +0.64% | 5 | 0.043711 | 0.043125 | 0.028086 | 1.56× |
| sankey/small | fixed | parse-first-render | -9.26% | 15 | 0.140664 | 0.163477 | 0.029102 | 4.83× |
| sankey/small | fixed | parse-first-edited-render | +30.16% | 0 | 0.288906 | 0.222656 | 0.028438 | 10.16× |
| sankey/small | canvas | render-source | +0.65% | 10 | 0.046367 | 0.046133 | 0.030078 | 1.54× |
| sankey/small | canvas | parse-first-render | -5.59% | 16 | 0.135547 | 0.147578 | 0.030664 | 4.42× |
| sankey/small | canvas | parse-first-edited-render | +31.95% | 0 | 0.280312 | 0.215859 | 0.030508 | 9.19× |
| flowchart/medium | fixed | render-source | -0.32% | 11 | 0.595156 | 0.580469 | 0.333125 | 1.79× |
| flowchart/medium | fixed | parse-first-render | +3.51% | 4 | 1.861875 | 1.819375 | 0.360937 | 5.16× |
| flowchart/medium | fixed | parse-first-edited-render | +4.49% | 7 | 2.330000 | 2.220000 | 0.363828 | 6.40× |
| flowchart/medium | canvas | render-source | -0.96% | 16 | 0.587813 | 0.590625 | 0.331094 | 1.78× |
| flowchart/medium | canvas | parse-first-render | +2.15% | 6 | 1.936875 | 1.879375 | 0.398203 | 4.86× |
| flowchart/medium | canvas | parse-first-edited-render | +3.16% | 3 | 2.306250 | 2.236250 | 0.379609 | 6.08× |
| sankey/medium | fixed | render-source | +0.26% | 10 | 0.217266 | 0.212812 | 0.192305 | 1.13× |
| sankey/medium | fixed | parse-first-render | -17.28% | 20 | 0.491875 | 0.601562 | 0.186250 | 2.64× |
| sankey/medium | fixed | parse-first-edited-render | +14.02% | 4 | 0.997500 | 0.876875 | 0.219375 | 4.55× |
| sankey/medium | canvas | render-source | -1.82% | 14 | 0.226484 | 0.226172 | 0.203516 | 1.11× |
| sankey/medium | canvas | parse-first-render | -19.96% | 21 | 0.513594 | 0.640312 | 0.244141 | 2.10× |
| sankey/medium | canvas | parse-first-edited-render | +6.17% | 6 | 0.988750 | 0.930313 | 0.224453 | 4.41× |
| sankey/large | fixed | render-source | +0.08% | 10 | 0.836563 | 0.837500 | 0.772187 | 1.08× |
| sankey/large | fixed | parse-first-render | -21.88% | 21 | 1.718750 | 2.199375 | 0.780937 | 2.20× |
| sankey/large | fixed | parse-first-edited-render | +11.45% | 3 | 3.503750 | 3.112500 | 0.851250 | 4.12× |
| sankey/large | canvas | render-source | +0.66% | 7 | 0.898438 | 0.893750 | 0.863438 | 1.04× |
| sankey/large | canvas | parse-first-render | -20.30% | 20 | 1.783125 | 2.236250 | 0.992188 | 1.80× |
| sankey/large | canvas | parse-first-edited-render | +11.02% | 0 | 3.571250 | 3.196250 | 0.922188 | 3.87× |
