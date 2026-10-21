# Detached Sankey callback-parity warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | SVG append control ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -4.29% | 13 | 0.055898 | 0.055547 | 0.003984 | 14.03× |
| sankey/small | fixed | render-model | +223.45% | 0 | 0.219531 | 0.069219 | 0.024102 | 9.11× |
| sankey/small | fixed | render-source | +0.65% | 10 | 0.042734 | 0.042734 | 0.027969 | 1.53× |
| sankey/small | canvas | parse | -7.54% | 18 | 0.051328 | 0.056523 | 0.004180 | 12.28× |
| sankey/small | canvas | render-model | +193.86% | 0 | 0.209375 | 0.071563 | 0.026758 | 7.82× |
| sankey/small | canvas | render-source | +1.18% | 7 | 0.050313 | 0.049414 | 0.033086 | 1.52× |
| flowchart/medium | fixed | parse | +1.45% | 8 | 0.828750 | 0.806875 | 0.065117 | 12.73× |
| flowchart/medium | fixed | render-model | +4.53% | 8 | 0.886250 | 0.853125 | 0.255234 | 3.47× |
| flowchart/medium | fixed | render-source | +0.78% | 5 | 0.568281 | 0.563906 | 0.319297 | 1.78× |
| flowchart/medium | canvas | parse | +1.36% | 8 | 0.790000 | 0.782812 | 0.062422 | 12.66× |
| flowchart/medium | canvas | render-model | +1.16% | 6 | 0.881250 | 0.873750 | 0.272031 | 3.24× |
| flowchart/medium | canvas | render-source | -0.03% | 11 | 0.583750 | 0.585625 | 0.347578 | 1.68× |
| sankey/medium | fixed | parse | -33.84% | 20 | 0.136797 | 0.206328 | 0.018984 | 7.21× |
| sankey/medium | fixed | render-model | +214.33% | 0 | 1.038750 | 0.329687 | 0.165859 | 6.26× |
| sankey/medium | fixed | render-source | +0.92% | 5 | 0.206719 | 0.204688 | 0.191016 | 1.08× |
| sankey/medium | canvas | parse | -36.51% | 18 | 0.137344 | 0.210156 | 0.019141 | 7.18× |
| sankey/medium | canvas | render-model | +214.31% | 0 | 1.083750 | 0.345000 | 0.180000 | 6.02× |
| sankey/medium | canvas | render-source | +1.17% | 8 | 0.227969 | 0.225859 | 0.207813 | 1.10× |
| sankey/large | fixed | parse | -40.21% | 20 | 0.468125 | 0.776094 | 0.077852 | 6.01× |
| sankey/large | fixed | render-model | +207.82% | 0 | 4.070000 | 1.326250 | 0.722813 | 5.63× |
| sankey/large | fixed | render-source | -0.33% | 12 | 0.844688 | 0.846563 | 0.778750 | 1.08× |
| sankey/large | canvas | parse | -38.29% | 21 | 0.483437 | 0.781250 | 0.082773 | 5.84× |
| sankey/large | canvas | render-model | +216.82% | 0 | 4.352500 | 1.392500 | 0.755313 | 5.76× |
| sankey/large | canvas | render-source | +0.39% | 7 | 0.880937 | 0.879062 | 0.831563 | 1.06× |
