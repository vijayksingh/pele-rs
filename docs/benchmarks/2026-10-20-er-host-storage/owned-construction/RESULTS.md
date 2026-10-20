# Complete paired rows

Derived from raw samples; negative means candidate lower. No pooling across reports.

## comparison

| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sankey/small | fixed | parse | -8.653% | 20/21 | 15.082x |
| sankey/small | fixed | render-model | +0.081% | 10/21 | 3.997x |
| sankey/small | fixed | render-source | -0.720% | 15/21 | 1.953x |
| sankey/small | canvas | parse | -9.688% | 19/21 | 14.708x |
| sankey/small | canvas | render-model | +2.105% | 9/21 | 3.928x |
| sankey/small | canvas | render-source | +0.136% | 10/21 | 1.889x |
| sequence/medium | fixed | parse | +0.584% | 9/21 | 11.476x |
| sequence/medium | fixed | render-model | -2.779% | 11/21 | 5.614x |
| sequence/medium | fixed | render-source | +0.739% | 6/21 | 1.981x |
| sequence/medium | canvas | parse | -0.110% | 12/21 | 11.389x |
| sequence/medium | canvas | render-model | -1.105% | 14/21 | 4.713x |
| sequence/medium | canvas | render-source | -0.804% | 14/21 | 1.698x |
| sankey/medium | fixed | parse | -10.410% | 18/21 | 11.377x |
| sankey/medium | fixed | render-model | -0.196% | 11/21 | 2.831x |
| sankey/medium | fixed | render-source | +1.132% | 3/21 | 1.380x |
| sankey/medium | canvas | parse | -10.890% | 16/21 | 11.406x |
| sankey/medium | canvas | render-model | -0.413% | 11/21 | 2.699x |
| sankey/medium | canvas | render-source | -0.981% | 12/21 | 1.434x |
| sankey/large | fixed | parse | -11.187% | 17/21 | 10.776x |
| sankey/large | fixed | render-model | -0.691% | 11/21 | 2.763x |
| sankey/large | fixed | render-source | +1.598% | 7/21 | 1.436x |
| sankey/large | canvas | parse | -10.501% | 15/21 | 8.911x |
| sankey/large | canvas | render-model | +0.287% | 9/21 | 2.541x |
| sankey/large | canvas | render-source | +1.917% | 3/21 | 1.354x |

## first-use

| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sankey/small | fixed | render-source | +0.069% | 7/15 | 1.939x |
| sankey/small | fixed | parse-first-render | -5.232% | 12/15 | 6.260x |
| sankey/small | fixed | parse-first-edited-render | -8.542% | 13/15 | 8.956x |
| sankey/small | canvas | render-source | +1.163% | 5/15 | 1.908x |
| sankey/small | canvas | parse-first-render | -7.375% | 13/15 | 5.631x |
| sankey/small | canvas | parse-first-edited-render | -0.439% | 9/15 | 7.935x |
| sequence/medium | fixed | render-source | +1.450% | 3/15 | 1.994x |
| sequence/medium | fixed | parse-first-render | -3.307% | 10/15 | 8.099x |
| sequence/medium | fixed | parse-first-edited-render | +0.368% | 6/15 | 8.538x |
| sequence/medium | canvas | render-source | +0.627% | 6/15 | 1.812x |
| sequence/medium | canvas | parse-first-render | -0.545% | 9/15 | 7.142x |
| sequence/medium | canvas | parse-first-edited-render | -1.145% | 14/15 | 7.635x |
| sankey/medium | fixed | render-source | -1.862% | 9/15 | 1.471x |
| sankey/medium | fixed | parse-first-render | +2.242% | 7/15 | 4.209x |
| sankey/medium | fixed | parse-first-edited-render | -3.810% | 11/15 | 4.804x |
| sankey/medium | canvas | render-source | -2.466% | 14/15 | 1.413x |
| sankey/medium | canvas | parse-first-render | -3.175% | 10/15 | 3.728x |
| sankey/medium | canvas | parse-first-edited-render | -4.725% | 10/15 | 4.397x |
| sankey/large | fixed | render-source | -1.271% | 10/15 | 1.406x |
| sankey/large | fixed | parse-first-render | -4.533% | 10/15 | 3.486x |
| sankey/large | fixed | parse-first-edited-render | -6.382% | 13/15 | 4.528x |
| sankey/large | canvas | render-source | -2.146% | 11/15 | 1.358x |
| sankey/large | canvas | parse-first-render | -0.974% | 8/15 | 3.700x |
| sankey/large | canvas | parse-first-edited-render | -2.479% | 13/15 | 4.295x |

## Partial local native timing

| Case | Stage | Paired change | Lower pairs | Candidate median ms |
|---|---|---:|---:|---:|
| sankey/medium | parse-public | -16.2082% | 15/15 | 0.055750000 |
| sankey/medium | parse | +1.0464% | 5/15 | 0.019208375 |

Six other native and all six Swift conditions remain unmeasured. Rejected windows are excluded, not pooled. Typed parse omits full public projection; local Node results cannot be subtracted from VM browser results.
