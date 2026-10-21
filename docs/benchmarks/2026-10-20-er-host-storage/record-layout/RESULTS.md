# Complete paired results

All rows retained. Paired change is the median of candidate/control trial ratios; JS ratio is the ratio of separate medians. Source controls are included in both independent VM runs.

## Ordinary VM — 21 trials

| Case | Profile | Stage | Paired change | Faster pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sequence/small | fixed | parse | -2.514% | 14/21 | 16.635× |
| sequence/small | fixed | render-model | -1.522% | 13/21 | 6.845× |
| sequence/small | fixed | render-source | -0.554% | 11/21 | 2.988× |
| sequence/small | canvas | parse | +2.667% | 10/21 | 15.560× |
| sequence/small | canvas | render-model | +1.425% | 8/21 | 5.882× |
| sequence/small | canvas | render-source | -1.951% | 12/21 | 2.511× |
| er/small | fixed | parse | -7.668% | 16/21 | 9.921× |
| er/small | fixed | render-model | -3.396% | 17/21 | 4.447× |
| er/small | fixed | render-source | +4.048% | 5/21 | 2.589× |
| er/small | canvas | parse | -8.117% | 17/21 | 10.534× |
| er/small | canvas | render-model | -0.111% | 12/21 | 4.326× |
| er/small | canvas | render-source | +1.542% | 3/21 | 2.258× |
| er/medium | fixed | parse | -10.190% | 17/21 | 7.405× |
| er/medium | fixed | render-model | +0.732% | 5/21 | 4.128× |
| er/medium | fixed | render-source | +1.676% | 7/21 | 2.307× |
| er/medium | canvas | parse | -13.679% | 16/21 | 7.552× |
| er/medium | canvas | render-model | +1.270% | 4/21 | 3.370× |
| er/medium | canvas | render-source | +0.890% | 6/21 | 1.984× |
| er/large | fixed | parse | -17.151% | 20/21 | 5.075× |
| er/large | fixed | render-model | -0.030% | 11/21 | 4.213× |
| er/large | fixed | render-source | +1.942% | 5/21 | 2.322× |
| er/large | canvas | parse | -12.313% | 15/21 | 5.182× |
| er/large | canvas | render-model | +0.691% | 8/21 | 3.464× |
| er/large | canvas | render-source | +0.857% | 4/21 | 2.151× |

## First-use VM — 15 trials

| Case | Profile | Stage | Paired change | Faster pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sequence/small | fixed | render-source | +1.271% | 4/15 | 2.681× |
| sequence/small | fixed | parse-first-render | +1.503% | 4/15 | 11.815× |
| sequence/small | fixed | parse-first-edited-render | -3.499% | 12/15 | 14.964× |
| sequence/small | canvas | render-source | +0.370% | 6/15 | 2.645× |
| sequence/small | canvas | parse-first-render | +2.265% | 6/15 | 11.739× |
| sequence/small | canvas | parse-first-edited-render | +0.985% | 5/15 | 12.987× |
| er/small | fixed | render-source | +6.031% | 4/15 | 2.629× |
| er/small | fixed | parse-first-render | -1.124% | 9/15 | 6.496× |
| er/small | fixed | parse-first-edited-render | -2.409% | 10/15 | 5.179× |
| er/small | fixed | parse-first-invoke | -8.728% | 14/15 | 19.136× |
| er/small | fixed | parse-first-callback | +0.306% | 7/15 | 5.513× |
| er/small | canvas | render-source | +1.992% | 4/15 | 2.153× |
| er/small | canvas | parse-first-render | -0.350% | 9/15 | 4.532× |
| er/small | canvas | parse-first-edited-render | -0.593% | 11/15 | 4.960× |
| er/small | canvas | parse-first-invoke | -5.437% | 12/15 | 18.132× |
| er/small | canvas | parse-first-callback | -2.811% | 12/15 | 4.777× |
| er/medium | fixed | render-source | +2.163% | 2/15 | 2.310× |
| er/medium | fixed | parse-first-render | +1.017% | 7/15 | 4.520× |
| er/medium | fixed | parse-first-edited-render | -2.403% | 11/15 | 4.588× |
| er/medium | fixed | parse-first-invoke | -12.313% | 13/15 | 13.812× |
| er/medium | fixed | parse-first-callback | -1.601% | 8/15 | 4.053× |
| er/medium | canvas | render-source | +2.226% | 1/15 | 2.101× |
| er/medium | canvas | parse-first-render | -1.985% | 11/15 | 3.813× |
| er/medium | canvas | parse-first-edited-render | -1.129% | 9/15 | 3.840× |
| er/medium | canvas | parse-first-invoke | -6.498% | 9/15 | 12.741× |
| er/medium | canvas | parse-first-callback | -1.454% | 9/15 | 3.939× |
| er/large | fixed | render-source | +3.445% | 1/15 | 2.386× |
| er/large | fixed | parse-first-render | +0.023% | 7/15 | 4.240× |
| er/large | fixed | parse-first-edited-render | -1.096% | 10/15 | 4.443× |
| er/large | fixed | parse-first-invoke | -11.634% | 11/15 | 11.475× |
| er/large | fixed | parse-first-callback | -4.102% | 9/15 | 4.199× |
| er/large | canvas | render-source | +2.380% | 2/15 | 2.133× |
| er/large | canvas | parse-first-render | -3.554% | 12/15 | 3.416× |
| er/large | canvas | parse-first-edited-render | +0.077% | 7/15 | 4.007× |
| er/large | canvas | parse-first-invoke | -12.979% | 11/15 | 9.777× |
| er/large | canvas | parse-first-callback | -2.646% | 10/15 | 3.843× |

## Swift — combined native changes versus 0b32797

| Case | Stage | Control ms | Candidate ms | Paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| sequence/small | parseMutable | 0.068017 | 0.067317 | +0.198% | 7/15 |
| sequence/small | lifetime | 0.222066 | 0.220992 | -1.543% | 9/15 |
| sequence/small | parse | 0.060448 | 0.060550 | +0.351% | 6/15 |
| er/medium | parseMutable | 0.261300 | 0.201809 | -22.689% | 15/15 |
| er/medium | lifetime | 2.274578 | 2.169979 | -3.501% | 12/15 |
| er/medium | parse | 0.584850 | 0.524719 | -9.445% | 14/15 |
