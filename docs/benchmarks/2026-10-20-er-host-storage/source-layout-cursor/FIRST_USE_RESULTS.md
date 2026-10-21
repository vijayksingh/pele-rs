# Semantic layout cursor default first-use comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | render-source | -1.36% | 12 | 0.042891 | 0.043320 | 0.028047 | 1.53× |
| sankey/small | fixed | parse-first-render | +0.93% | 8 | 0.140820 | 0.140625 | 0.028242 | 4.99× |
| sankey/small | fixed | parse-first-edited-render | +16.70% | 2 | 0.259922 | 0.225859 | 0.028594 | 9.09× |
| sankey/small | canvas | render-source | -0.08% | 11 | 0.049453 | 0.049648 | 0.031836 | 1.55× |
| sankey/small | canvas | parse-first-render | -1.21% | 12 | 0.144180 | 0.147383 | 0.030625 | 4.71× |
| sankey/small | canvas | parse-first-edited-render | +19.39% | 3 | 0.264063 | 0.223281 | 0.031133 | 8.48× |
| flowchart/medium | fixed | render-source | +0.19% | 9 | 0.571562 | 0.572500 | 0.339297 | 1.68× |
| flowchart/medium | fixed | parse-first-render | +5.52% | 6 | 1.930625 | 1.821250 | 0.343437 | 5.62× |
| flowchart/medium | fixed | parse-first-edited-render | +4.70% | 3 | 2.315625 | 2.221875 | 0.361094 | 6.41× |
| flowchart/medium | canvas | render-source | +0.33% | 8 | 0.591406 | 0.589375 | 0.344297 | 1.72× |
| flowchart/medium | canvas | parse-first-render | +3.69% | 5 | 1.945000 | 1.871875 | 0.424219 | 4.58× |
| flowchart/medium | canvas | parse-first-edited-render | +2.98% | 8 | 2.361250 | 2.281250 | 0.370156 | 6.38× |
| sankey/medium | fixed | render-source | -2.47% | 19 | 0.206797 | 0.209609 | 0.185469 | 1.11× |
| sankey/medium | fixed | parse-first-render | -11.28% | 18 | 0.519063 | 0.587031 | 0.225469 | 2.30× |
| sankey/medium | fixed | parse-first-edited-render | +7.36% | 6 | 0.926875 | 0.871875 | 0.225078 | 4.12× |
| sankey/medium | canvas | render-source | -1.65% | 20 | 0.213359 | 0.217578 | 0.203672 | 1.05× |
| sankey/medium | canvas | parse-first-render | -11.25% | 18 | 0.547031 | 0.609063 | 0.244844 | 2.23× |
| sankey/medium | canvas | parse-first-edited-render | +7.99% | 5 | 0.940000 | 0.871562 | 0.235547 | 3.99× |
| sankey/large | fixed | render-source | -1.59% | 17 | 0.835938 | 0.849688 | 0.772500 | 1.08× |
| sankey/large | fixed | parse-first-render | -13.31% | 16 | 1.928125 | 2.251875 | 0.771563 | 2.50× |
| sankey/large | fixed | parse-first-edited-render | +4.67% | 7 | 3.271250 | 3.103750 | 0.770938 | 4.24× |
| sankey/large | canvas | render-source | -1.80% | 19 | 0.875938 | 0.889375 | 0.840000 | 1.04× |
| sankey/large | canvas | parse-first-render | -11.77% | 17 | 2.047500 | 2.400625 | 0.884688 | 2.31× |
| sankey/large | canvas | parse-first-edited-render | +6.40% | 1 | 3.305000 | 3.120000 | 0.897812 | 3.68× |
