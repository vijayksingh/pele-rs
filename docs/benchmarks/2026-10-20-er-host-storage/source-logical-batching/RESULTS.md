# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | +28.91% | 1 | 0.072266 | 0.056875 | 0.004062 | 17.79× |
| sankey/small | fixed | render-model | +46.87% | 0 | 0.095859 | 0.065391 | 0.024102 | 3.98× |
| sankey/small | fixed | render-source | -0.18% | 12 | 0.043164 | 0.043086 | 0.028125 | 1.53× |
| sankey/small | canvas | parse | +24.96% | 1 | 0.068203 | 0.054609 | 0.003828 | 17.82× |
| sankey/small | canvas | render-model | +43.51% | 0 | 0.097148 | 0.067461 | 0.026445 | 3.67× |
| sankey/small | canvas | render-source | +0.61% | 9 | 0.045313 | 0.045078 | 0.030078 | 1.51× |
| flowchart/medium | fixed | parse | +2.59% | 7 | 0.833438 | 0.816250 | 0.068633 | 12.14× |
| flowchart/medium | fixed | render-model | +2.46% | 6 | 0.887812 | 0.849688 | 0.255469 | 3.48× |
| flowchart/medium | fixed | render-source | +0.25% | 8 | 0.573125 | 0.570781 | 0.337578 | 1.70× |
| flowchart/medium | canvas | parse | +1.64% | 8 | 0.809687 | 0.784688 | 0.069844 | 11.59× |
| flowchart/medium | canvas | render-model | +1.57% | 7 | 0.874375 | 0.856875 | 0.285234 | 3.07× |
| flowchart/medium | canvas | render-source | -0.03% | 11 | 0.579531 | 0.580000 | 0.367734 | 1.58× |
| sankey/medium | fixed | parse | +23.36% | 4 | 0.258672 | 0.209688 | 0.019258 | 13.43× |
| sankey/medium | fixed | render-model | +40.38% | 0 | 0.468594 | 0.333438 | 0.169961 | 2.76× |
| sankey/medium | fixed | render-source | +1.45% | 4 | 0.205391 | 0.203125 | 0.188672 | 1.09× |
| sankey/medium | canvas | parse | +18.78% | 6 | 0.252656 | 0.209453 | 0.019219 | 13.15× |
| sankey/medium | canvas | render-model | +40.10% | 0 | 0.488438 | 0.348125 | 0.185742 | 2.63× |
| sankey/medium | canvas | render-source | +0.58% | 9 | 0.217578 | 0.216016 | 0.206563 | 1.05× |
| sankey/large | fixed | parse | +18.91% | 5 | 0.955625 | 0.819687 | 0.082422 | 11.59× |
| sankey/large | fixed | render-model | +40.18% | 0 | 1.871875 | 1.351250 | 0.718125 | 2.61× |
| sankey/large | fixed | render-source | +0.58% | 4 | 0.829688 | 0.821875 | 0.780937 | 1.06× |
| sankey/large | canvas | parse | +20.47% | 3 | 0.947812 | 0.785938 | 0.134805 | 7.03× |
| sankey/large | canvas | render-model | +36.39% | 0 | 1.891875 | 1.384375 | 0.755625 | 2.50× |
| sankey/large | canvas | render-source | +0.74% | 9 | 0.907187 | 0.914062 | 0.896563 | 1.01× |
