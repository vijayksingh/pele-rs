# Source-layout default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -3.94% | 15 | 0.054297 | 0.056289 | 0.003984 | 13.63× |
| sankey/small | fixed | render-model | -14.93% | 20 | 0.056328 | 0.066250 | 0.024062 | 2.34× |
| sankey/small | fixed | render-source | +0.53% | 10 | 0.045977 | 0.045352 | 0.029883 | 1.54× |
| sankey/small | canvas | parse | -1.21% | 11 | 0.053164 | 0.054141 | 0.004023 | 13.21× |
| sankey/small | canvas | render-model | -15.00% | 21 | 0.058047 | 0.068398 | 0.026289 | 2.21× |
| sankey/small | canvas | render-source | -0.35% | 12 | 0.044219 | 0.044453 | 0.030391 | 1.46× |
| flowchart/medium | fixed | parse | +5.05% | 7 | 0.854688 | 0.816250 | 0.068086 | 12.55× |
| flowchart/medium | fixed | render-model | +9.02% | 1 | 0.898438 | 0.825312 | 0.274219 | 3.28× |
| flowchart/medium | fixed | render-source | -0.93% | 13 | 0.573438 | 0.583594 | 0.353125 | 1.62× |
| flowchart/medium | canvas | parse | +0.96% | 8 | 0.802813 | 0.790938 | 0.063398 | 12.66× |
| flowchart/medium | canvas | render-model | +4.12% | 6 | 0.894063 | 0.880625 | 0.275156 | 3.25× |
| flowchart/medium | canvas | render-source | -0.61% | 14 | 0.586719 | 0.591250 | 0.364375 | 1.61× |
| sankey/medium | fixed | parse | -29.57% | 20 | 0.144375 | 0.207422 | 0.021211 | 6.81× |
| sankey/medium | fixed | render-model | -15.39% | 21 | 0.281406 | 0.334219 | 0.167578 | 1.68× |
| sankey/medium | fixed | render-source | -0.23% | 12 | 0.205625 | 0.205703 | 0.192031 | 1.07× |
| sankey/medium | canvas | parse | -29.42% | 19 | 0.159609 | 0.222344 | 0.020391 | 7.83× |
| sankey/medium | canvas | render-model | -15.33% | 21 | 0.299844 | 0.354141 | 0.186719 | 1.61× |
| sankey/medium | canvas | render-source | -1.15% | 14 | 0.218281 | 0.220312 | 0.216328 | 1.01× |
| sankey/large | fixed | parse | -34.73% | 21 | 0.565625 | 0.862188 | 0.088555 | 6.39× |
| sankey/large | fixed | render-model | -15.02% | 21 | 1.149375 | 1.358125 | 0.720781 | 1.59× |
| sankey/large | fixed | render-source | -0.42% | 11 | 0.821562 | 0.820938 | 0.800000 | 1.03× |
| sankey/large | canvas | parse | -37.52% | 21 | 0.510781 | 0.817969 | 0.081680 | 6.25× |
| sankey/large | canvas | render-model | -13.28% | 21 | 1.200937 | 1.391562 | 0.775000 | 1.55× |
| sankey/large | canvas | render-source | +0.30% | 10 | 0.935313 | 0.932500 | 0.914375 | 1.02× |
