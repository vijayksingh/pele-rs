# Sparse source ingress default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

The first-use timer includes parse, edit setup, render and disposal. Source rows time direct source rendering.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | -1.10% | 14 | 0.042578 | 0.043008 | 0.028281 | 1.51× |
| sankey/small | fixed | parse-first-render | -4.27% | 18 | 0.137031 | 0.143086 | 0.028203 | 4.86× |
| sankey/small | fixed | parse-first-edited-render | +19.70% | 3 | 0.291016 | 0.258906 | 0.029727 | 9.79× |
| sankey/small | canvas | render-source | -1.30% | 14 | 0.045156 | 0.045898 | 0.030078 | 1.50× |
| sankey/small | canvas | parse-first-render | -4.37% | 16 | 0.137422 | 0.147031 | 0.031133 | 4.41× |
| sankey/small | canvas | parse-first-edited-render | +18.97% | 2 | 0.264609 | 0.224844 | 0.030742 | 8.61× |
| flowchart/medium | fixed | render-source | +0.95% | 5 | 0.565313 | 0.556094 | 0.314922 | 1.80× |
| flowchart/medium | fixed | parse-first-render | +1.59% | 7 | 1.959375 | 1.910000 | 0.403125 | 4.86× |
| flowchart/medium | fixed | parse-first-edited-render | +1.07% | 7 | 2.284375 | 2.270000 | 0.358359 | 6.37× |
| flowchart/medium | canvas | render-source | +0.43% | 8 | 0.581406 | 0.578906 | 0.345234 | 1.68× |
| flowchart/medium | canvas | parse-first-render | +3.49% | 3 | 1.926875 | 1.862500 | 0.375234 | 5.14× |
| flowchart/medium | canvas | parse-first-edited-render | +7.60% | 5 | 2.529375 | 2.233750 | 0.343750 | 7.36× |
| sankey/medium | fixed | render-source | +0.65% | 7 | 0.206094 | 0.204688 | 0.188359 | 1.09× |
| sankey/medium | fixed | parse-first-render | -17.78% | 21 | 0.475781 | 0.583125 | 0.214922 | 2.21× |
| sankey/medium | fixed | parse-first-edited-render | +2.91% | 5 | 0.905625 | 0.866875 | 0.221484 | 4.09× |
| sankey/medium | canvas | render-source | +1.81% | 7 | 0.222422 | 0.217891 | 0.208984 | 1.06× |
| sankey/medium | canvas | parse-first-render | -20.32% | 21 | 0.501250 | 0.626563 | 0.208281 | 2.41× |
| sankey/medium | canvas | parse-first-edited-render | +5.36% | 2 | 0.941875 | 0.874062 | 0.230000 | 4.10× |
| sankey/large | fixed | render-source | +1.70% | 2 | 0.828750 | 0.819687 | 0.774375 | 1.07× |
| sankey/large | fixed | parse-first-render | -19.93% | 19 | 1.765000 | 2.182500 | 0.825938 | 2.14× |
| sankey/large | fixed | parse-first-edited-render | -1.00% | 12 | 3.136250 | 3.161250 | 0.797188 | 3.93× |
| sankey/large | canvas | render-source | +1.01% | 5 | 0.895937 | 0.887812 | 0.880000 | 1.02× |
| sankey/large | canvas | parse-first-render | -19.93% | 19 | 1.801250 | 2.235000 | 0.985312 | 1.83× |
| sankey/large | canvas | parse-first-edited-render | -1.45% | 12 | 3.227500 | 3.260000 | 0.886563 | 3.64× |
