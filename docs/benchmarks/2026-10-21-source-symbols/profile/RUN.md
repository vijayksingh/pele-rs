# Public parse diagnostic — prepared only

No execution until explicit profile GO after warm review. Four windows: medium control/candidate, large candidate/control; fixed measurer;5s each,500ms warmup,250us CDP sampling. One browser/page,20s profile workload total. Public parse + model.type.length + dispose is the same measured operation as warm1429. No render, model swap, digest or instrumentation inside profile. No forced GC.

Reuses original initializer profiler sampling/tree/self/inclusive logic and current four-window cleanup/input-freeze scaffold. This is an adapted harness, not byte-identical to the old nine-window harness. Parent-requested cases/duration changed; historical code untouched. All time deltas must be finite and nonnegative. Raw profile saved BEFORE validation; any anomaly stops and preserves failure. No clamping or automatic retry.

Before and after each case, fresh public models from all three runtimes must match an identity-aware own-data descriptor/IEEE-bit digest and exact pinned Pele SVG/dimensions/callback trace. Verification is outside CDP; all native models disposed, engines closed, browser closed. No prototype interception. Actual route proof remains the unchanged runner36 probes outside timing.

Inputs pin exact117 candidate/113 e148 file maps, source commits, oracle, cases and harness hashes. Candidate unset until source-symbols-runner pin succeeds. `python3 dist/perf-patches/parse-architecture/source-symbols-profiler/pin.py` is data-only. Then preserve executed code + manifest with raw output. No current execution or attribution claims.

After separate GO, at VM root `/home/admin/pele-perf.jL9wzP`:
```sh
PLAYWRIGHT_BROWSERS_PATH="$PWD/browsers" node parse-architecture/source-symbols-profiler/files/scripts/perf/profile-source-symbols.mjs --execute-approved --control parse-architecture/svg-append-e148/candidate-package --candidate parse-architecture/source-symbols-runtime/candidate-package --output parse-architecture/source-symbols-profiler/profiles
```

Classifier anchors actual parser function paths, excludes IndexedSankey/SourceSymbols appearing only inside generic argument names. SourceSymbols actual methods are a separate construction group. Leaf roles form a disjoint partition; ancestor groups/unions deduplicate samples but overlap one another. Keep raw full frame names/URLs and verify actual new native names before interpreting. No inferred allocation/call counts or additive ceilings. A failing digest/probe is a preflight failure, never weakened to make profiling run.
