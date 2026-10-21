# Complete paired rows

Negative means candidate lower. Reports and rejected samples are never pooled.

## comparison

| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sankey/small | fixed | parse | +6.224% | 7/21 | 14.950x |
| sankey/small | fixed | render-model | +5.446% | 3/21 | 3.286x |
| sankey/small | fixed | render-source | +0.283% | 7/21 | 1.972x |
| sankey/small | canvas | parse | +0.514% | 10/21 | 14.030x |
| sankey/small | canvas | render-model | +2.206% | 6/21 | 2.940x |
| sankey/small | canvas | render-source | +7.163% | 3/21 | 1.970x |
| sequence/medium | fixed | parse | -0.716% | 12/21 | 10.887x |
| sequence/medium | fixed | render-model | +5.330% | 5/21 | 4.841x |
| sequence/medium | fixed | render-source | -1.149% | 14/21 | 1.865x |
| sequence/medium | canvas | parse | -1.922% | 18/21 | 10.967x |
| sequence/medium | canvas | render-model | +2.169% | 8/21 | 4.179x |
| sequence/medium | canvas | render-source | +0.636% | 9/21 | 1.698x |
| er/medium | fixed | parse | -0.395% | 11/21 | 6.444x |
| er/medium | fixed | render-model | -0.366% | 11/21 | 4.070x |
| er/medium | fixed | render-source | +1.261% | 4/21 | 1.984x |
| er/medium | canvas | parse | -0.318% | 11/21 | 3.463x |
| er/medium | canvas | render-model | +2.638% | 2/21 | 3.788x |
| er/medium | canvas | render-source | +1.709% | 4/21 | 1.748x |
| sankey/medium | fixed | parse | +1.812% | 10/21 | 11.313x |
| sankey/medium | fixed | render-model | +0.200% | 9/21 | 2.376x |
| sankey/medium | fixed | render-source | -1.024% | 18/21 | 1.390x |
| sankey/medium | canvas | parse | +4.084% | 9/21 | 11.317x |
| sankey/medium | canvas | render-model | +0.386% | 8/21 | 2.242x |
| sankey/medium | canvas | render-source | -1.611% | 19/21 | 1.373x |
| sankey/large | fixed | parse | +1.571% | 7/21 | 9.625x |
| sankey/large | fixed | render-model | +0.155% | 10/21 | 2.258x |
| sankey/large | fixed | render-source | +0.301% | 9/21 | 1.474x |
| sankey/large | canvas | parse | +0.592% | 9/21 | 10.469x |
| sankey/large | canvas | render-model | +0.038% | 9/21 | 2.097x |
| sankey/large | canvas | render-source | -0.912% | 16/21 | 1.400x |

## first-use

| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sankey/small | fixed | render-source | -0.211% | 8/15 | 1.780x |
| sankey/small | fixed | parse-first-render | +0.456% | 7/15 | 5.507x |
| sankey/small | fixed | parse-first-edited-render | -4.218% | 12/15 | 8.256x |
| sankey/small | canvas | render-source | -0.736% | 10/15 | 1.900x |
| sankey/small | canvas | parse-first-render | -1.210% | 9/15 | 5.402x |
| sankey/small | canvas | parse-first-edited-render | -0.554% | 8/15 | 8.053x |
| sequence/medium | fixed | render-source | +0.497% | 6/15 | 1.983x |
| sequence/medium | fixed | parse-first-render | -0.576% | 8/15 | 6.622x |
| sequence/medium | fixed | parse-first-edited-render | +1.386% | 4/15 | 7.888x |
| sequence/medium | canvas | render-source | +1.186% | 4/15 | 1.791x |
| sequence/medium | canvas | parse-first-render | -0.266% | 9/15 | 5.971x |
| sequence/medium | canvas | parse-first-edited-render | +1.572% | 5/15 | 8.123x |
| sankey/medium | fixed | render-source | +0.479% | 6/15 | 1.518x |
| sankey/medium | fixed | parse-first-render | -0.518% | 10/15 | 2.844x |
| sankey/medium | fixed | parse-first-edited-render | -0.984% | 9/15 | 4.100x |
| sankey/medium | canvas | render-source | +0.298% | 4/15 | 1.422x |
| sankey/medium | canvas | parse-first-render | +1.537% | 7/15 | 2.767x |
| sankey/medium | canvas | parse-first-edited-render | -1.454% | 9/15 | 4.043x |
| sankey/large | fixed | render-source | +0.308% | 7/15 | 1.416x |
| sankey/large | fixed | parse-first-render | +19.037% | 2/15 | 3.685x |
| sankey/large | fixed | parse-first-edited-render | +1.380% | 4/15 | 3.847x |
| sankey/large | canvas | render-source | +0.633% | 5/15 | 1.385x |
| sankey/large | canvas | parse-first-render | -0.309% | 8/15 | 2.584x |
| sankey/large | canvas | parse-first-edited-render | +2.548% | 4/15 | 3.990x |

## Accepted local Swift conditions

| Case | Stage | Paired change | Lower pairs | Control median ms | Candidate median ms |
|---|---|---:|---:|---:|---:|
| sankey/medium | render | -0.4776% | 9/15 | 0.366110 | 0.362974 |
| sankey/medium | lifetime | +0.0043% | 7/15 | 0.593240 | 0.582079 |

parseMutable rejected in full by external swift-build; parse unmeasured.
