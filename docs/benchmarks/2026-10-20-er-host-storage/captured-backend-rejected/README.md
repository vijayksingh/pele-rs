# Captured Sankey storage placement: rejected

The complete opt-in JS-column backend loses substantially against arena storage with identical package bytes. It remains disabled. This is an architecture experiment, not a release or JS parity claim.

| Stage | Six paired median changes versus arena | Candidate / pinned JS |
|---|---:|---:|
| Parse | +430.87% to +559.77% | 61.67× to 87.20× |
| Retained render | +263.37% to +297.38% | 8.40× to 12.06× |
| Direct source render control | −0.51% to +1.37% | 1.36× to 1.97× |

All 18 rows, every 21-pair sample, adverse results, warmup counts and matched batch sizes remain in comparison.json. Three scales × two resource profiles × three stages were exercised. Backend probes outside timers confirm candidate source storage and arena control on retained and fresh parsed models. Both complete 112-file packages are identical, including WASM fbaa003fc95224a27cd8f7a49964a4a271b85f9a1dfaac4c44a3ecc4bed3dee2. The selection flag is the only intended difference.

Six separate CDP profiles diagnose medium/fixed parse, retained render and source control. Weighted admission/adapter/import ancestor union is 75.99% of captured parse and 57.02% of captured retained. These overlapping mechanisms are counted once per sample; they are not a predicted removable fraction. Raw Sankey parser is 0.40% inclusive of captured parse. See FINDINGS.md for source checkpoints and the native-canonical-storage recommendation. Full graph publication, observation, admission and partial-error semantics remain mandatory.

`profiles/*.cpuprofile.gz` contains all six raw profiles, losslessly compressed. Original summary and executed harness remain unchanged, including its initial HTTP-WASM classification mistake. ATTRIBUTION.json and analyze-profiles.py correct that classification from raw frames. The corrected future harness is separately named. RUN_ATTEMPTS.json preserves the failed default-cache browser launch and successful configured retry; the failed launch produced no CPU profiles and installed nothing.

Data-only reproducibility:

```sh
python3 summarize.py
python3 analyze-profiles.py
```

Both checks were run against this portable folder. Neither starts a browser or benchmark. `index.json` hashes every included file except itself. Production package/source hashes and executed harness hashes live in the raw reports; there is no source-only identity substitution. The current broader browser suite is separate evidence and is not claimed complete here.
