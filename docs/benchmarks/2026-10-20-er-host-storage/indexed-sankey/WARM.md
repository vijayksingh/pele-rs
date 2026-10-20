# Indexed Sankey default warm comparison

Paired change is median(candidate trial/control same-index trial)-1; JS ratios and absolute milliseconds use each lane median. All 24 rows retained; no cross-window compounding. Diagnostic three-way shared-browser workload, not release gate.

| Case | Profile | Stage | Paired change | Lower / 21 | Candidate ms | Arena ms | JS ms | Candidate / JS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| sankey/small | fixed | parse | -6.69% | 20 | 0.054219 | 0.058438 | 0.003789 | 14.31× |
| sankey/small | fixed | render-model | -0.15% | 11 | 0.079961 | 0.082617 | 0.024141 | 3.31× |
| sankey/small | fixed | render-source | -1.47% | 16 | 0.054414 | 0.055234 | 0.027422 | 1.98× |
| sankey/small | canvas | parse | -5.31% | 14 | 0.055586 | 0.058086 | 0.003906 | 14.23× |
| sankey/small | canvas | render-model | -0.50% | 14 | 0.078203 | 0.079102 | 0.026445 | 2.96× |
| sankey/small | canvas | render-source | +0.56% | 7 | 0.055977 | 0.055977 | 0.030156 | 1.86× |
| flowchart/medium | fixed | parse | +0.66% | 9 | 0.826563 | 0.835625 | 0.066328 | 12.46× |
| flowchart/medium | fixed | render-model | -3.00% | 13 | 0.839062 | 0.862812 | 0.262578 | 3.20× |
| flowchart/medium | fixed | render-source | +0.38% | 7 | 0.581875 | 0.581875 | 0.339687 | 1.71× |
| flowchart/medium | canvas | parse | +0.94% | 8 | 0.802500 | 0.798750 | 0.060664 | 13.23× |
| flowchart/medium | canvas | render-model | -1.10% | 14 | 0.842813 | 0.854062 | 0.267031 | 3.16× |
| flowchart/medium | canvas | render-source | -0.70% | 13 | 0.577187 | 0.578125 | 0.343594 | 1.68× |
| sankey/medium | fixed | parse | -5.14% | 15 | 0.208672 | 0.219219 | 0.018711 | 11.15× |
| sankey/medium | fixed | render-model | -0.12% | 11 | 0.403437 | 0.401406 | 0.168320 | 2.40× |
| sankey/medium | fixed | render-source | -0.75% | 15 | 0.271875 | 0.274141 | 0.186875 | 1.45× |
| sankey/medium | canvas | parse | -3.64% | 15 | 0.212109 | 0.227266 | 0.019102 | 11.10× |
| sankey/medium | canvas | render-model | -1.26% | 16 | 0.415937 | 0.422031 | 0.182656 | 2.28× |
| sankey/medium | canvas | render-source | -0.13% | 12 | 0.291406 | 0.290078 | 0.204141 | 1.43× |
| sankey/large | fixed | parse | -4.55% | 17 | 0.812500 | 0.861875 | 0.102500 | 7.93× |
| sankey/large | fixed | render-model | -0.27% | 12 | 1.658125 | 1.713125 | 0.693906 | 2.39× |
| sankey/large | fixed | render-source | +0.22% | 10 | 1.140625 | 1.137500 | 0.785313 | 1.45× |
| sankey/large | canvas | parse | -5.01% | 17 | 0.785937 | 0.807813 | 0.076055 | 10.33× |
| sankey/large | canvas | render-model | -0.30% | 12 | 1.663750 | 1.677500 | 0.777656 | 2.14× |
| sankey/large | canvas | render-source | -1.56% | 15 | 1.205625 | 1.217188 | 0.879375 | 1.37× |
