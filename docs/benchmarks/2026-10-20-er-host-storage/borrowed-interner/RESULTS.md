# Complete results

Paired change is median of trial ratios. Positive means slower. All controls retained.

## Native —15 trials, fixed batch16

| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| flowchart/medium |  parse-public | 0.269289 | 0.266284 | -1.102% | 8/15 |
| flowchart/medium |  parse | 0.095203 | 0.094333 | +0.115% | 7/15 |
| flowchart/medium |  render-source | 0.395443 | 0.395628 | -0.377% | 8/15 |
| flowchart/medium |  render-retained | 0.290776 | 0.291669 | +0.307% | 7/15 |
| er/medium |  parse-public | 0.182865 | 0.161945 | -10.585% | 15/15 |
| er/medium |  parse | 0.094042 | 0.093799 | -0.175% | 9/15 |
| er/medium |  render-source | 0.794693 | 0.790284 | -1.282% | 10/15 |
| er/medium |  render-retained | 0.692800 | 0.693016 | -0.210% | 8/15 |

## Swift —15 pairs, isolated versus record

| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| sequence/small |  parseMutable | 0.067132 | 0.066902 | +0.485% | 7/15 |
| sequence/small |  lifetime | 0.222161 | 0.219825 | -0.630% | 10/15 |
| sequence/small |  parse | 0.060064 | 0.060250 | +0.789% | 7/15 |
| er/medium |  parseMutable | 0.201787 | 0.178163 | -12.002% | 15/15 |
| er/medium |  lifetime | 2.167667 | 2.161500 | -2.443% | 10/15 |
| er/medium |  parse | 0.524835 | 0.523247 | -0.264% | 8/15 |

## VM negative controls —9 trials, 500ms warmup

| Case | Profile / stage | Control ms | Candidate ms | Paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| sequence/small | fixed parse | 0.160703 | 0.133008 | -3.494% | 7/9 |
| sequence/small | fixed render-model | 0.118555 | 0.116680 | -1.152% | 5/9 |
| sequence/small | fixed render-source | 0.074570 | 0.073320 | -1.314% | 6/9 |
| sequence/small | canvas parse | 0.128477 | 0.125625 | -1.718% | 6/9 |
| sequence/small | canvas render-model | 0.125820 | 0.117656 | -6.209% | 7/9 |
| sequence/small | canvas render-source | 0.074570 | 0.075625 | +1.414% | 4/9 |
| flowchart/medium | fixed parse | 0.783125 | 0.778750 | -0.598% | 7/9 |
| flowchart/medium | fixed render-model | 0.901250 | 0.900937 | +1.269% | 4/9 |
| flowchart/medium | fixed render-source | 0.619844 | 0.609375 | -2.040% | 5/9 |
| flowchart/medium | canvas parse | 0.758906 | 0.739844 | -2.162% | 7/9 |
| flowchart/medium | canvas render-model | 0.968125 | 0.932813 | +0.420% | 4/9 |
| flowchart/medium | canvas render-source | 0.629844 | 0.636719 | +0.574% | 4/9 |
| er/medium | fixed parse | 0.526719 | 0.552188 | +2.797% | 2/9 |
| er/medium | fixed render-model | 2.192500 | 2.171250 | -2.495% | 6/9 |
| er/medium | fixed render-source | 1.312813 | 1.318750 | +0.340% | 4/9 |
| er/medium | canvas parse | 0.537500 | 0.553750 | +2.743% | 2/9 |
| er/medium | canvas render-model | 2.233750 | 2.198750 | -1.702% | 6/9 |
| er/medium | canvas render-source | 1.434687 | 1.423750 | -1.654% | 7/9 |

## Allocation counters —count1 and count16

| Case / stage | Count | Allocations control / candidate | Reallocations | Requested bytes | Peak additional bytes | Live delta |
|---|---:|---:|---:|---:|---:|---:|
| flowchart/medium parse-public | 1 | 3431 / 3431 | 867 / 867 | 493299 / 493299 | 180322 / 180322 | 0 / 0 |
| flowchart/medium parse-public | 16 | 54896 / 54896 | 13872 / 13872 | 7892784 / 7892784 | 180322 / 180322 | 0 / 0 |
| er/medium parse-public | 1 | 2501 / 1987 | 584 / 109 | 278808 / 263576 | 119594 / 118710 | 0 / 0 |
| er/medium parse-public | 16 | 40016 / 31792 | 9344 / 1744 | 4460928 / 4217216 | 119594 / 118710 | 0 / 0 |
| er/medium parse | 1 | 1163 / 1163 | 17 / 17 | 173297 / 173297 | 84274 / 84274 | 0 / 0 |
| er/medium parse | 16 | 18608 / 18608 | 272 / 272 | 2772752 / 2772752 | 84274 / 84274 | 0 / 0 |
| er/medium render-source | 1 | 9689 / 9689 | 2590 / 2590 | 1710686 / 1710686 | 534732 / 534732 | 0 / 0 |
| er/medium render-source | 16 | 155024 / 155024 | 41440 / 41440 | 27370976 / 27370976 | 534732 / 534732 | 0 / 0 |
| er/medium render-retained | 1 | 8526 / 8526 | 2573 / 2573 | 1537389 / 1537389 | 488228 / 488228 | 0 / 0 |
| er/medium render-retained | 16 | 136416 / 136416 | 41168 / 41168 | 24598224 / 24598224 | 488228 / 488228 | 0 / 0 |
