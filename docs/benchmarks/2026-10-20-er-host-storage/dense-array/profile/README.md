# Dense-array control profiles

Six existing CDP profiles from the dense runtime (`770113546dd302a60b52707194bbda40bb9e070c`): ER medium parse, retained-model render and source render, each for pinned JS and WASM. These retain the borrowed interner, dense arrays and both permanent string maps. They predate the rejected literal candidate.

All raw profiles and the original summary are gzip-compressed without content changes. `summary.json.gz` retains the browser, host, sampling settings and full **106-file runtime hash map**, independently matched to the rejected-literal comparison's dense control. Sampling interval 250µs; target duration 1,500ms per profile.

`PROFILE_FINDINGS.md` explains attribution and limitations. `six-profile-shares.json` retains exact uncompressed profile hashes, sample counts, weighted categories and native top functions. Run `python3 profile-shares.py` to recompute that file from the six compressed inputs; no runtime profiling occurs.

Self categories use `timeDeltas` weights and partition each profile. WASM includes Rust, runtime and glue; JS includes the harness. Family-inclusive samples are deduplicated within a family but overlap across families. GC is not assigned to object owners. Different operation counts mean these percentage shares are not relative latency or allocation measurements.

`hash-self-callers.json` and `profile-attribution.json` preserve the additional ancestry classifications discussed in the findings. They are bounded attribution evidence, not predicted speedups. The policy-sharing proposal remains parked because its deduplicated sampled scope is approximately 0.70%.
