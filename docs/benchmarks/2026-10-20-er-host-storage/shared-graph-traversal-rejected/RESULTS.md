# Complete paired rows

Negative means candidate lower. Separate windows/binaries stay separate; rejected samples are never pooled.

## comparison

| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sequence/small | fixed | parse | -1.842% | 12/21 | 16.131x |
| sequence/small | fixed | render-model | +1.721% | 6/21 | 5.693x |
| sequence/small | fixed | render-source | -0.673% | 13/21 | 2.391x |
| sequence/small | canvas | parse | -1.763% | 11/21 | 14.855x |
| sequence/small | canvas | render-model | -1.023% | 11/21 | 4.986x |
| sequence/small | canvas | render-source | +1.141% | 9/21 | 2.473x |
| sankey/small | fixed | parse | -1.184% | 13/21 | 13.657x |
| sankey/small | fixed | render-model | +0.395% | 10/21 | 3.198x |
| sankey/small | fixed | render-source | +1.325% | 3/21 | 1.935x |
| sankey/small | canvas | parse | +0.071% | 10/21 | 19.131x |
| sankey/small | canvas | render-model | +3.158% | 8/21 | 3.044x |
| sankey/small | canvas | render-source | +1.062% | 9/21 | 1.835x |
| flowchart/medium | fixed | parse | +0.699% | 10/21 | 12.413x |
| flowchart/medium | fixed | render-model | +9.161% | 4/21 | 3.362x |
| flowchart/medium | fixed | render-source | +0.583% | 10/21 | 1.675x |
| flowchart/medium | canvas | parse | +2.102% | 7/21 | 11.633x |
| flowchart/medium | canvas | render-model | -0.302% | 11/21 | 3.291x |
| flowchart/medium | canvas | render-source | -1.398% | 14/21 | 1.653x |
| sequence/medium | fixed | parse | +1.100% | 8/21 | 12.213x |
| sequence/medium | fixed | render-model | +0.371% | 9/21 | 4.749x |
| sequence/medium | fixed | render-source | +1.226% | 10/21 | 1.856x |
| sequence/medium | canvas | parse | +1.545% | 8/21 | 11.568x |
| sequence/medium | canvas | render-model | +1.056% | 7/21 | 4.076x |
| sequence/medium | canvas | render-source | -1.654% | 12/21 | 1.695x |
| er/medium | fixed | parse | +3.551% | 5/21 | 6.566x |
| er/medium | fixed | render-model | +1.169% | 8/21 | 4.148x |
| er/medium | fixed | render-source | -1.086% | 13/21 | 1.823x |
| er/medium | canvas | parse | +2.124% | 7/21 | 7.162x |
| er/medium | canvas | render-model | -0.995% | 14/21 | 3.433x |
| er/medium | canvas | render-source | -0.830% | 15/21 | 1.656x |
| sankey/medium | fixed | parse | +4.531% | 6/21 | 11.834x |
| sankey/medium | fixed | render-model | +1.134% | 7/21 | 2.424x |
| sankey/medium | fixed | render-source | +1.466% | 4/21 | 1.440x |
| sankey/medium | canvas | parse | +2.258% | 5/21 | 10.536x |
| sankey/medium | canvas | render-model | +0.914% | 5/21 | 2.310x |
| sankey/medium | canvas | render-source | +2.141% | 4/21 | 1.367x |
| sankey/large | fixed | parse | +2.524% | 8/21 | 10.820x |
| sankey/large | fixed | render-model | +0.475% | 10/21 | 2.215x |
| sankey/large | fixed | render-source | +0.585% | 9/21 | 1.415x |
| sankey/large | canvas | parse | -0.227% | 11/21 | 9.814x |
| sankey/large | canvas | render-model | -0.765% | 15/21 | 2.076x |
| sankey/large | canvas | render-source | +2.413% | 1/21 | 1.394x |

## first-use

| Case | Resource | Stage | Paired change | Lower pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sankey/small | fixed | render-source | -0.144% | 9/15 | 1.875x |
| sankey/small | fixed | parse-first-render | -3.078% | 9/15 | 4.828x |
| sankey/small | fixed | parse-first-edited-render | -0.494% | 8/15 | 8.552x |
| sankey/small | canvas | render-source | -0.416% | 10/15 | 1.877x |
| sankey/small | canvas | parse-first-render | +2.436% | 5/15 | 5.286x |
| sankey/small | canvas | parse-first-edited-render | +0.136% | 7/15 | 8.365x |
| flowchart/medium | fixed | render-source | -2.391% | 10/15 | 1.769x |
| flowchart/medium | fixed | parse-first-render | +1.075% | 6/15 | 5.060x |
| flowchart/medium | fixed | parse-first-edited-render | +3.013% | 4/15 | 6.247x |
| flowchart/medium | canvas | render-source | +0.379% | 7/15 | 1.631x |
| flowchart/medium | canvas | parse-first-render | +3.330% | 4/15 | 5.368x |
| flowchart/medium | canvas | parse-first-edited-render | -0.780% | 11/15 | 6.109x |
| sequence/medium | fixed | render-source | -0.073% | 8/15 | 1.937x |
| sequence/medium | fixed | parse-first-render | +0.000% | 7/15 | 6.899x |
| sequence/medium | fixed | parse-first-edited-render | -1.522% | 9/15 | 8.373x |
| sequence/medium | canvas | render-source | -1.516% | 11/15 | 1.833x |
| sequence/medium | canvas | parse-first-render | +1.590% | 4/15 | 6.714x |
| sequence/medium | canvas | parse-first-edited-render | -0.060% | 8/15 | 7.841x |
| er/medium | fixed | render-source | +0.786% | 3/15 | 2.012x |
| er/medium | fixed | parse-first-render | -0.601% | 8/15 | 4.078x |
| er/medium | fixed | parse-first-edited-render | -0.370% | 8/15 | 4.291x |
| er/medium | fixed | parse-first-invoke | -0.282% | 8/15 | 14.127x |
| er/medium | fixed | parse-first-callback | +0.378% | 6/15 | 4.251x |
| er/medium | canvas | render-source | +2.244% | 3/15 | 1.842x |
| er/medium | canvas | parse-first-render | -4.258% | 10/15 | 4.268x |
| er/medium | canvas | parse-first-edited-render | +1.086% | 5/15 | 3.827x |
| er/medium | canvas | parse-first-invoke | -1.386% | 8/15 | 9.307x |
| er/medium | canvas | parse-first-callback | +0.938% | 7/15 | 4.130x |
| sankey/medium | fixed | render-source | +0.744% | 7/15 | 1.461x |
| sankey/medium | fixed | parse-first-render | -0.097% | 8/15 | 2.871x |
| sankey/medium | fixed | parse-first-edited-render | -0.426% | 9/15 | 4.172x |
| sankey/medium | canvas | render-source | +0.559% | 6/15 | 1.428x |
| sankey/medium | canvas | parse-first-render | +0.423% | 6/15 | 2.756x |
| sankey/medium | canvas | parse-first-edited-render | +0.949% | 5/15 | 3.863x |
| sankey/large | fixed | render-source | -0.055% | 8/15 | 1.406x |
| sankey/large | fixed | parse-first-render | -0.439% | 9/15 | 2.851x |
| sankey/large | fixed | parse-first-edited-render | -1.219% | 8/15 | 4.310x |
| sankey/large | canvas | render-source | -0.665% | 11/15 | 1.429x |
| sankey/large | canvas | parse-first-render | +1.620% | 4/15 | 2.715x |
| sankey/large | canvas | parse-first-edited-render | +0.295% | 5/15 | 3.990x |

## Separate local Swift core comparison

| Case | Stage | Paired change | Lower pairs | Control median ms | Candidate median ms |
|---|---|---:|---:|---:|---:|
| sankey/medium | parseMutable | -0.9226% | 8/15 | 0.070243 | 0.070325 |
| sankey/medium | parse | +0.0499% | 7/15 | 0.267497 | 0.266912 |
| sankey/medium | render | +1.0223% | 5/15 | 0.361123 | 0.365001 |

lifetime rejected in full by external rustc45785; unmeasured.
