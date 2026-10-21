# Default-native Swift timing: 25d87cb to 3166b3d

One authorized attempt completed all four Sankey medium conditions in 40.18 seconds. Each condition contains 15 matched pairs after at least 1,000 ms of warmup per binary. All 60 pairs are retained. Order offsets alternate 0/1/0/1, giving 30 control-first and 30 candidate-first pairs overall. There were no retries, exclusions or pooled partial windows.

| Stage | Paired median change | Candidate lower pairs | Full paired change range |
| --- | ---: | ---: | ---: |
| Mutable parse | -0.2990% | 10/15 | -7.0276% to +37.3004% |
| Immutable parse | -0.1250% | 8/15 | -9.8920% to +16.3833% |
| Retained render | +1.3285% | 5/15 | -2.8672% to +3.4151% |
| Lifetime | -0.3373% | 8/15 | -82.4282% to +95.8140% |

Positive changes are adverse. Retained render is adverse here; lifetime has large outliers in both directions. These results establish neither a causal speedup nor practical equivalence. They cover one fixture on a shared macOS host, not the opt-in WASM storage backend, a full release gate or cross-host JavaScript parity. The two commits include the exact native/SDK changes listed in `summary.json`; do not attribute their combined comparison to one isolated primitive or compound these separate condition windows.

The original global compiler/Pele-test guard remained unchanged, including compiler processes at zero reported CPU. Its checks passed; the host was not idle. Full before/after process snapshots, per-pair external load, warmup/calibration and all raw elapsed nanoseconds remain in the compressed records. Passing sampled guard checks does not imply continuous absence of other workload. No causal explanation is assigned to the outliers.

Control is the frozen `captured-native-core` consumer built at `25d87cb49bb451d3e7c1e70c2856ccb6fef4abd0`; candidate is the fresh default-native consumer built at `3166b3dd18569ad0bc20f173a8941dd628609ddc`. The JS-only cursor commit `5db9f35b475a3ee479c925513d78ab237a6cda0e` retained all 362 candidate native/consumer/SDK inputs. Exact before/after maps, build manifests, SDK archive hashes and binary hashes are included. The executed driver checked current and frozen candidate binaries and the frozen control before and after the attempt. The portable bundle omits those binaries and SDK archives; its verifier checks their recorded bindings, not a new physical rehash or rebuild.

The archived `prepared-manifest.json` says "prepared, not executed" because it is the unchanged pre-run pin record. Completed execution is established by `schedule.json` and all four accepted reports. The scheduler's original 180-second ceiling and exclusive `attempt1` directory are preserved. Its absolute paths and source-tree imports are historical execution evidence, not a portable benchmark command.

Run only the data verifier from this folder:

```sh
node verify.mjs
```

It recomputes per-operation times, paired ratios, median/MAD/range/P95, wins, order counts and all 60 output/checksum comparisons; checks exact case outputs before/after; verifies unchanged input maps, aggregate hashes and harness pins; and verifies every packaged file against `evidence-index.json`. It does not invoke binaries, build, benchmark, access Git or use the network. `provenance.json` records original paths and uncompressed byte hashes. Raw JSON/logs and executable source snapshots are losslessly gzip-compressed. `summary.json` can be regenerated with `node verify.mjs --write-summary`; the file index must then be regenerated if any bytes change.

Fresh default-native Swift/C/API correctness and frozen opt-in browser validation are documented in the adjacent `native-canonical-correctness` and `native-canonical-browser` bundles. Those checks are separate from this timing attempt.
