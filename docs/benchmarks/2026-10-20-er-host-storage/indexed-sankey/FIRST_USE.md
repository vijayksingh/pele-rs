# Indexed Sankey default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | Arena ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | +1.45% | 7 | 0.055273 | 0.054648 | 0.029297 | 1.89× |
| sankey/small | fixed | parse-first-render | -4.50% | 14 | 0.156523 | 0.164570 | 0.028437 | 5.50× |
| sankey/small | fixed | parse-first-edited-render | -1.17% | 12 | 0.231484 | 0.234687 | 0.028438 | 8.14× |
| sankey/small | canvas | render-source | +2.02% | 6 | 0.058242 | 0.057109 | 0.030117 | 1.93× |
| sankey/small | canvas | parse-first-render | -3.25% | 14 | 0.164141 | 0.168984 | 0.033828 | 4.85× |
| sankey/small | canvas | parse-first-edited-render | -3.18% | 15 | 0.236406 | 0.245391 | 0.030859 | 7.66× |
| flowchart/medium | fixed | render-source | -5.12% | 21 | 0.538125 | 0.567969 | 0.307500 | 1.75× |
| flowchart/medium | fixed | parse-first-render | +1.35% | 6 | 1.911875 | 1.895000 | 0.374219 | 5.11× |
| flowchart/medium | fixed | parse-first-edited-render | -1.46% | 14 | 2.200625 | 2.241250 | 0.378203 | 5.82× |
| flowchart/medium | canvas | render-source | -4.35% | 20 | 0.568125 | 0.597344 | 0.334688 | 1.70× |
| flowchart/medium | canvas | parse-first-render | -1.02% | 13 | 1.858125 | 1.895000 | 0.399375 | 4.65× |
| flowchart/medium | canvas | parse-first-edited-render | +3.03% | 3 | 2.261875 | 2.211875 | 0.352656 | 6.41× |
| sankey/medium | fixed | render-source | +2.04% | 4 | 0.284219 | 0.278906 | 0.189531 | 1.50× |
| sankey/medium | fixed | parse-first-render | -2.26% | 16 | 0.659219 | 0.677656 | 0.217188 | 3.04× |
| sankey/medium | fixed | parse-first-edited-render | -4.08% | 20 | 0.921250 | 0.965625 | 0.204922 | 4.50× |
| sankey/medium | canvas | render-source | +2.64% | 3 | 0.300234 | 0.292109 | 0.201953 | 1.49× |
| sankey/medium | canvas | parse-first-render | -0.07% | 11 | 0.695000 | 0.693438 | 0.213594 | 3.25× |
| sankey/medium | canvas | parse-first-edited-render | -1.83% | 13 | 0.969375 | 0.979062 | 0.245625 | 3.95× |
| sankey/large | fixed | render-source | -0.47% | 13 | 1.140938 | 1.135312 | 0.795000 | 1.44× |
| sankey/large | fixed | parse-first-render | -0.87% | 14 | 2.495000 | 2.506875 | 0.992500 | 2.51× |
| sankey/large | fixed | parse-first-edited-render | -2.09% | 14 | 3.435000 | 3.473750 | 0.776562 | 4.42× |
| sankey/large | canvas | render-source | +0.36% | 9 | 1.198125 | 1.197812 | 0.859062 | 1.39× |
| sankey/large | canvas | parse-first-render | +6.50% | 7 | 2.783750 | 2.570000 | 0.847187 | 3.29× |
| sankey/large | canvas | parse-first-edited-render | -0.67% | 13 | 3.485000 | 3.505000 | 0.989375 | 3.52× |
