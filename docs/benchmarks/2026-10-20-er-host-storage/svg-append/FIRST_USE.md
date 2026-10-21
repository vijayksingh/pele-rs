# SVG append default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | Indexed control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | -22.66% | 21 | 0.042812 | 0.055547 | 0.027734 | 1.54× |
| sankey/small | fixed | parse-first-render | -5.51% | 13 | 0.143633 | 0.155625 | 0.028125 | 5.11× |
| sankey/small | fixed | parse-first-edited-render | -8.98% | 16 | 0.217656 | 0.238594 | 0.029063 | 7.49× |
| sankey/small | canvas | render-source | -21.04% | 21 | 0.045313 | 0.057695 | 0.030156 | 1.50× |
| sankey/small | canvas | parse-first-render | -6.58% | 14 | 0.143633 | 0.155352 | 0.030547 | 4.70× |
| sankey/small | canvas | parse-first-edited-render | -5.69% | 18 | 0.230625 | 0.246563 | 0.031016 | 7.44× |
| flowchart/medium | fixed | render-source | -1.51% | 14 | 0.550937 | 0.559062 | 0.316250 | 1.74× |
| flowchart/medium | fixed | parse-first-render | +2.16% | 5 | 1.824375 | 1.787500 | 0.347422 | 5.25× |
| flowchart/medium | fixed | parse-first-edited-render | +0.66% | 9 | 2.190625 | 2.177500 | 0.340781 | 6.43× |
| flowchart/medium | canvas | render-source | +3.13% | 5 | 0.577188 | 0.563906 | 0.337969 | 1.71× |
| flowchart/medium | canvas | parse-first-render | +0.91% | 7 | 1.816875 | 1.806250 | 0.340000 | 5.34× |
| flowchart/medium | canvas | parse-first-edited-render | +2.24% | 6 | 2.234375 | 2.183125 | 0.387031 | 5.77× |
| sankey/medium | fixed | render-source | -25.89% | 20 | 0.203984 | 0.277187 | 0.184883 | 1.10× |
| sankey/medium | fixed | parse-first-render | -11.07% | 17 | 0.578750 | 0.651875 | 0.193359 | 2.99× |
| sankey/medium | fixed | parse-first-edited-render | -8.23% | 18 | 0.848750 | 0.915313 | 0.224922 | 3.77× |
| sankey/medium | canvas | render-source | -25.31% | 21 | 0.218203 | 0.292656 | 0.202969 | 1.08× |
| sankey/medium | canvas | parse-first-render | -10.54% | 16 | 0.615469 | 0.695312 | 0.204609 | 3.01× |
| sankey/medium | canvas | parse-first-edited-render | -6.37% | 13 | 0.877187 | 0.940313 | 0.203672 | 4.31× |
| sankey/large | fixed | render-source | -26.06% | 21 | 0.846563 | 1.145000 | 0.805000 | 1.05× |
| sankey/large | fixed | parse-first-render | -12.79% | 15 | 2.196875 | 2.534375 | 0.957500 | 2.29× |
| sankey/large | fixed | parse-first-edited-render | -7.65% | 16 | 3.178750 | 3.451250 | 0.793125 | 4.01× |
| sankey/large | canvas | render-source | -26.14% | 21 | 0.910937 | 1.228750 | 0.881250 | 1.03× |
| sankey/large | canvas | parse-first-render | -12.50% | 18 | 2.266875 | 2.573125 | 1.030625 | 2.20× |
| sankey/large | canvas | parse-first-edited-render | -8.50% | 15 | 3.223750 | 3.523750 | 0.847812 | 3.80× |
