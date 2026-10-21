# Independent lexer timing audit

Audit of the completed frozen reports only. No build, benchmark, consumer or browser execution. `audit.py` recomputes statistics and verifies recorded identities; `audit.json` contains every audited row and hash.

## Result and scope

The raw arithmetic supports the reported ER parse improvements. Swift measures the combined native changes from `0b32797` to `df3597c`, not isolated lexer causality: 15 changed inputs comprise 14 source paths and the SDK archive. The VM packages differ only in `pkg/pele_bg.wasm`; all 106 files per side agree with the corresponding result-packet control and lexical candidate freezes. These are diagnostics, not a complete release gate or JS parity proof.

## Swift: six conditions

| Case | Stage | Control ms | Candidate ms | Paired change | Faster pairs |
|---|---|---:|---:|---:|---:|
| sequence/small | parseMutable | 0.066510 | 0.067294 | +0.792% | 4/15 |
| sequence/small | lifetime | 0.217421 | 0.219482 | +1.209% | 3/15 |
| sequence/small | parse | 0.060039 | 0.060314 | +0.710% | 5/15 |
| er/medium | parseMutable | 0.261893 | 0.206865 | -20.765% | 15/15 |
| er/medium | lifetime | 2.245823 | 2.160854 | -3.650% | 12/15 |
| er/medium | parse | 0.579423 | 0.521007 | -9.994% | 15/15 |

All six rows contain 15 matched pairs (90 total), with exactly 45 control-first and 45 candidate-first. Each row alternates order; the 8/7 imbalance reverses on the next row. Warmups use equal operation counts per side and at least 1000 ms per side, with maximum warmup batches of 128. Calibration doubles counts and selects one identical count for both binaries (128, 64, 128, 64, 8, 32). All saved nanosecond-to-ms divisions, paired ratios, percent changes, medians, MAD, extrema and p95 values recompute correctly. No rejected or partial report is pooled.

Every saved before/after, calibration and measured response matches the case oracle SVG hash/byte count, type and IEEE dimension bits; all non-timing paired response fields agree. The two case-level five-output verification passes agree before and after. This is output correctness for the selected fixtures, not full model/error/callback coverage.

The guard records 260 saved timed-request checks (180 measured requests plus 80 calibration requests). None records a forbidden builder or Pele test process. The hashed guard intentionally permits other host load: Chrome renderer reaches 45.1% CPU in saved checks, WindowServer 18.7%, Codex renderer 16.7%, and launchd 42.3%; benchmark process activity is also recorded. Warmup reports aggregate counts and duration, not each individual guard record. “Guard accepted” must not be restated as an idle or exclusive host.

## Frozen identity and SDK freshness

The actual frozen control binary matches `75258a9666006497d1bc2fe871dea2c511599e4ee8f3058f70528c6ddcb1687c`; the actual candidate binary matches `0cdf86d1573f95dca88d0c5e103e2e08533f4ed0bba01054fd2128b98955cb45`. Both original manifest byte hashes match the report. Recomputed aggregates match all 348 control and 351 candidate input hashes. Every tracked source/consumer input was independently read from the stated immutable Git revision: 335 control blobs and 338 candidate blobs, with no missing paths. No present-day source is substituted for the frozen tree.

All 351 candidate native inputs also agree with `runtime/source-freeze.json`. Its complete 457-entry source freeze contains 444 tracked source/consumer/web blobs, all verified at `df3597c`, plus 13 SDK files; all 13 actual SDK files match. The actual SDK archive and explicit Rust target archive both match `c18d9b1df64003ae0c2f947e437e0a93c7a9311267a550c96ac8e5ba34808b5b`. The frozen build harness checks Cargo freshness, SDK/target equality and before/after source aggregates before recording its manifest. This audit validates those recorded conditions and current artifact identities; it does not rerun compilation. Historical control SDK archive contents are represented by the frozen manifest, while its actual linked consumer binary is independently hash-verified.

Comparator, workload generator, build/guard source and pinned oracle hashes match the executed report. Deterministic case source hashes and bytes were recomputed from the hash-verified generator without running a renderer.

## VM: 24 rows

| Case | Profile | Stage | Paired change | Faster pairs | Candidate / JS |
|---|---|---|---:|---:|---:|
| sequence/small | fixed | parse | -1.003% | 12/21 | 17.060× |
| sequence/small | fixed | render-model | -1.662% | 12/21 | 6.967× |
| sequence/small | fixed | render-source | -1.622% | 11/21 | 3.017× |
| sequence/small | canvas | parse | +3.576% | 8/21 | 16.373× |
| sequence/small | canvas | render-model | -4.393% | 13/21 | 5.965× |
| sequence/small | canvas | render-source | -0.529% | 14/21 | 2.613× |
| er/small | fixed | parse | -6.061% | 17/21 | 12.161× |
| er/small | fixed | render-model | -1.134% | 14/21 | 6.071× |
| er/small | fixed | render-source | -6.145% | 18/21 | 2.481× |
| er/small | canvas | parse | -7.304% | 18/21 | 10.609× |
| er/small | canvas | render-model | +0.786% | 10/21 | 5.612× |
| er/small | canvas | render-source | -4.303% | 16/21 | 2.167× |
| er/medium | fixed | parse | -7.691% | 15/21 | 9.494× |
| er/medium | fixed | render-model | +1.045% | 9/21 | 5.209× |
| er/medium | fixed | render-source | -3.205% | 18/21 | 2.217× |
| er/medium | canvas | parse | -6.941% | 17/21 | 9.156× |
| er/medium | canvas | render-model | +0.523% | 10/21 | 4.673× |
| er/medium | canvas | render-source | -3.729% | 15/21 | 2.061× |
| er/large | fixed | parse | -8.057% | 21/21 | 8.500× |
| er/large | fixed | render-model | +1.068% | 7/21 | 5.031× |
| er/large | fixed | render-source | -3.791% | 19/21 | 2.279× |
| er/large | canvas | parse | -8.566% | 17/21 | 6.025× |
| er/large | canvas | render-model | +0.301% | 10/21 | 4.483× |
| er/large | canvas | render-source | -2.328% | 18/21 | 2.071× |

All 24 rows have 21 samples per lane (504 triples, 1512 lane batches). Recomputed medians, p95, ratios of medians and paired changes agree. Control/candidate counts match in every row; JS may use a separately calibrated count. Each lane warms for at least 1000 ms. The hash-verified cyclic/reverse schedule gives each lane seven first positions per row, with control preceding candidate 11 times versus 10. Raw observed order and timer start/end timestamps are not saved; that schedule is source-derived, not independent execution telemetry.

All six ER parse rows improve by paired medians of 6.061–8.566%, but remain 6.025–12.161× JS by separate medians. ER retained rendering is slightly adverse in five rows (+0.301–1.068%) and improves in small/fixed (−1.134%); Sequence canvas parse is +3.576%. Those rows remain disclosed. Paired changes and ratios of separate medians are different statistics and must not be interchanged.

The VM harness validates full SVG strings and Object.is dimensions for retained rendering, source rendering and freshly parsed models before and after timing; successful completion implies those assertions passed. The report preserves only case/profile output byte counts, not the full output artifacts. It does not validate every public model or callback behavior.

There is no per-row process quiet guard in this VM comparator. It records host load/memory before and after, enforces an isolated high-resolution clock and ten-minute browser deadline, and verifies package identity before/after. One page shares JS heaps, GC and cache pressure; retained setup/warm render is outside each timed batch, while parse includes disposal. No inference of no regressions, allocation reduction or release readiness follows.

The candidate/control 106-file tables match their independent frozen manifests exactly. A fresh remote byte recheck at `2026-10-20T07:37:13.972Z`, after timing completed at `07:30:24.590Z`, independently walks `parse-architecture/er-result-packet-first/package` and `parse-architecture/er-lexical-dispatch-first/package`. All 106 paths and SHA256 values per side in `../er-lexical-dispatch-first/independent-vm-rehash.json` exactly match the benchmark report and respective source freezes. Parent ran that fresh Node SHA reader over SSH independently of the benchmark scripts; this reviewer compared every recorded result. It confirms that both frozen VM directories still contained the exact measured package bytes at recheck time. File bodies were not downloaded, so this is verification of the independent remote byte-reading record, not a second local byte rehash or uninterrupted monitoring. Current frame-build output is not substituted.
