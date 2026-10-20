# Complete dense-array paired results

Positive change means slower. All rows retained; no control subtraction or adverse-result dismissal.

## Ordinary VM —21 trials

| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs | Candidate / JS |
|---|---|---:|---:|---:|---:|---:|
| sequence/small | fixed parse | 0.132578 | 0.130937 | -2.942% | 15/21 | 17.015× |
| sequence/small | fixed render-model | 0.120547 | 0.116836 | -3.143% | 14/21 | 6.876× |
| sequence/small | fixed render-source | 0.073008 | 0.074805 | +2.691% | 5/21 | 3.006× |
| sequence/small | canvas parse | 0.131563 | 0.129336 | +0.000% | 10/21 | 15.918× |
| sequence/small | canvas render-model | 0.120391 | 0.124492 | +0.533% | 8/21 | 6.105× |
| sequence/small | canvas render-source | 0.074297 | 0.076992 | +1.998% | 5/21 | 2.678× |
| er/small | fixed parse | 0.127461 | 0.119414 | -5.118% | 15/21 | 9.958× |
| er/small | fixed render-model | 0.397969 | 0.394062 | -1.096% | 14/21 | 4.750× |
| er/small | fixed render-source | 0.252109 | 0.250703 | -0.159% | 11/21 | 2.592× |
| er/small | canvas parse | 0.134297 | 0.119258 | -6.422% | 19/21 | 9.280× |
| er/small | canvas render-model | 0.429844 | 0.414219 | -3.765% | 19/21 | 3.945× |
| er/small | canvas render-source | 0.251484 | 0.252109 | +0.280% | 9/21 | 2.088× |
| er/medium | fixed parse | 0.540781 | 0.487812 | -9.175% | 15/21 | 7.333× |
| er/medium | fixed render-model | 2.059375 | 2.068750 | -0.153% | 12/21 | 4.050× |
| er/medium | fixed render-source | 1.352500 | 1.317813 | -1.473% | 14/21 | 2.104× |
| er/medium | canvas parse | 0.537656 | 0.497031 | -8.418% | 17/21 | 6.617× |
| er/medium | canvas render-model | 2.183750 | 2.283125 | +4.642% | 6/21 | 3.664× |
| er/medium | canvas render-source | 1.468750 | 1.434062 | -2.951% | 14/21 | 1.966× |
| er/large | fixed parse | 2.055625 | 1.951250 | -3.818% | 15/21 | 6.317× |
| er/large | fixed render-model | 8.402500 | 8.352500 | -0.817% | 11/21 | 4.022× |
| er/large | fixed render-source | 5.526250 | 5.576250 | -0.221% | 12/21 | 2.163× |
| er/large | canvas parse | 2.171875 | 1.949375 | -10.245% | 20/21 | 4.516× |
| er/large | canvas render-model | 8.927500 | 9.110000 | -3.016% | 13/21 | 3.504× |
| er/large | canvas render-source | 5.827500 | 5.810000 | -0.366% | 11/21 | 1.972× |

## First-use VM —15 trials

| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs | Candidate / JS |
|---|---|---:|---:|---:|---:|---:|
| sequence/small | fixed render-source | 0.073398 | 0.073125 | -1.118% | 8/15 | 2.845× |
| sequence/small | fixed parse-first-render | 0.301016 | 0.295547 | -1.340% | 8/15 | 11.464× |
| sequence/small | fixed parse-first-edited-render | 0.376484 | 0.361641 | -5.195% | 10/15 | 13.675× |
| sequence/small | canvas render-source | 0.073008 | 0.073594 | +0.696% | 5/15 | 2.613× |
| sequence/small | canvas parse-first-render | 0.305391 | 0.312812 | +2.251% | 4/15 | 10.706× |
| sequence/small | canvas parse-first-edited-render | 0.410469 | 0.413125 | -1.009% | 10/15 | 12.993× |
| er/small | fixed render-source | 0.259766 | 0.240547 | -5.714% | 13/15 | 2.587× |
| er/small | fixed parse-first-render | 0.622187 | 0.617813 | -1.101% | 9/15 | 6.367× |
| er/small | fixed parse-first-edited-render | 0.639844 | 0.621094 | -3.080% | 11/15 | 5.198× |
| er/small | fixed parse-first-invoke | 0.220391 | 0.207891 | -5.365% | 12/15 | 19.283× |
| er/small | fixed parse-first-callback | 0.615625 | 0.589063 | -3.127% | 13/15 | 5.038× |
| er/small | canvas render-source | 0.256094 | 0.243281 | -4.948% | 15/15 | 2.253× |
| er/small | canvas parse-first-render | 0.653281 | 0.638594 | -3.643% | 10/15 | 4.577× |
| er/small | canvas parse-first-edited-render | 0.660313 | 0.644219 | -4.183% | 10/15 | 4.995× |
| er/small | canvas parse-first-invoke | 0.219375 | 0.213203 | -2.545% | 9/15 | 19.218× |
| er/small | canvas parse-first-callback | 0.698281 | 0.661094 | -3.641% | 10/15 | 5.007× |
| er/medium | fixed render-source | 1.295938 | 1.277812 | -0.913% | 14/15 | 2.264× |
| er/medium | fixed parse-first-render | 2.787500 | 2.656250 | -2.679% | 13/15 | 4.220× |
| er/medium | fixed parse-first-edited-render | 2.765000 | 2.666250 | -2.934% | 14/15 | 4.308× |
| er/medium | fixed parse-first-invoke | 1.114062 | 0.872187 | -23.086% | 12/15 | 13.590× |
| er/medium | fixed parse-first-callback | 2.750000 | 2.693750 | -2.245% | 11/15 | 4.340× |
| er/medium | canvas render-source | 1.379688 | 1.374063 | -0.610% | 11/15 | 2.043× |
| er/medium | canvas parse-first-render | 2.903750 | 2.858750 | -0.567% | 8/15 | 3.830× |
| er/medium | canvas parse-first-edited-render | 2.816250 | 2.778125 | -0.848% | 9/15 | 4.156× |
| er/medium | canvas parse-first-invoke | 0.936875 | 0.898438 | -4.354% | 9/15 | 13.514× |
| er/medium | canvas parse-first-callback | 2.871250 | 2.869375 | -1.711% | 9/15 | 3.854× |
| er/large | fixed render-source | 5.430000 | 5.477500 | -0.577% | 9/15 | 2.335× |
| er/large | fixed parse-first-render | 10.902500 | 10.565000 | -3.004% | 13/15 | 4.151× |
| er/large | fixed parse-first-edited-render | 11.065000 | 10.792500 | -3.626% | 10/15 | 3.911× |
| er/large | fixed parse-first-invoke | 4.450000 | 3.568750 | -11.533% | 12/15 | 12.467× |
| er/large | fixed parse-first-callback | 10.855000 | 10.407500 | -2.360% | 13/15 | 4.107× |
| er/large | canvas render-source | 5.797500 | 5.757500 | -0.367% | 11/15 | 2.021× |
| er/large | canvas parse-first-render | 11.572500 | 11.105000 | -3.252% | 11/15 | 3.640× |
| er/large | canvas parse-first-edited-render | 11.160000 | 11.097500 | -1.108% | 9/15 | 3.748× |
| er/large | canvas parse-first-invoke | 3.818750 | 3.607500 | -4.484% | 10/15 | 12.069× |
| er/large | canvas parse-first-callback | 11.137500 | 11.025000 | -0.362% | 9/15 | 3.718× |

## Swift —15 pairs against frozen d25

| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs | Candidate / JS |
|---|---|---:|---:|---:|---:|---:|
| sequence/small |  parseMutable | 0.067777 | 0.067660 | +0.096% | 7/15 | — |
| sequence/small |  lifetime | 0.223133 | 0.222936 | -0.330% | 10/15 | — |
| sequence/small |  parse | 0.060570 | 0.061060 | +0.867% | 4/15 | — |
| er/medium |  parseMutable | 0.178896 | 0.179826 | +0.648% | 5/15 | — |
| er/medium |  lifetime | 2.159240 | 2.155474 | +0.644% | 5/15 | — |
| er/medium |  parse | 0.525811 | 0.526613 | -0.490% | 8/15 | — |

## Allocation rows —all metrics identical

| Case / stage | Count | Allocations | Reallocations | Requested bytes | Peak additional bytes | Live delta |
|---|---:|---:|---:|---:|---:|---:|
| flowchart/medium parse-public | 1 | 3431 | 867 | 493299 | 180322 | 0 |
| flowchart/medium parse-public | 16 | 54896 | 13872 | 7892784 | 180322 | 0 |
| er/medium parse-public | 1 | 1987 | 109 | 263576 | 118710 | 0 |
| er/medium parse-public | 16 | 31792 | 1744 | 4217216 | 118710 | 0 |
| er/medium parse | 1 | 1163 | 17 | 173297 | 84274 | 0 |
| er/medium parse | 16 | 18608 | 272 | 2772752 | 84274 | 0 |
| er/medium render-source | 1 | 9689 | 2590 | 1710686 | 534732 | 0 |
| er/medium render-source | 16 | 155024 | 41440 | 27370976 | 534732 | 0 |
| er/medium render-retained | 1 | 8526 | 2573 | 1537389 | 488228 | 0 |
| er/medium render-retained | 16 | 136416 | 41168 | 24598224 | 488228 | 0 |
