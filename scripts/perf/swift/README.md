# Actual Swift API consumer

This separate release consumer imports the source-built Pele Swift package. It never implements a diagram algorithm. Node imports the existing 21 deterministic workloads from `scripts/perf/cases.mjs`, loads the pinned development-only Pele oracle, and checks complete SVG SHA-256, UTF-8 byte count, diagram type and exact IEEE dimension bits before and after work. Raw UTF-16 and cyclic extra values also cross snapshot/apply in correctness verification.

Build after the native SDK finishes and relevant source freezes:

```sh
node scripts/perf/swift/build.mjs
node scripts/perf/swift/run.mjs
```

While staged outside the checkout, pass `--root /absolute/path/to/pele-rs` to both commands. No build or timing runs merely by importing the build helpers. Correctness builds may run alongside unrelated compilation; before/after input hashes and exact SDK slice comparison still reject changing or stale inputs. Timed runs retain the global compiler/Pele-test guard. Build first runs `cargo build --locked --release -p pele-ffi --target aarch64-apple-darwin` (or `x86_64-apple-darwin` on Intel) to establish a current explicit-target archive, matching the universal SDK producer. It does **not** replace the Swift SDK. The existing local XCFramework must contain exactly the current archive (or the same host architecture slice); stale SDKs fail with an instruction to rebuild the SDK separately. A clean checkout that would select the published binary instead of the current SDK is rejected.

The build manifest hashes all Rust/FFI sources, manifests, build inputs, Swift SDK files, C headers, every XCFramework file, this consumer's sources, shared cases, and the release executable. Inputs must stay unchanged during build and measurement. Source-freeze correspondence is verified by explicit-target Cargo freshness plus exact byte equality of the linked host archive slice; an untargeted archive is never substituted. Compiler, SDK, architecture, revision, dirty state, CPU and system load accompany reports.

## API stages

| Stage | Work inside the timer |
|---|---|
| `parse` | Actual immutable parse, including full Swift model projection and owned-document release |
| `render-source` | Typed source render, result crossing and Swift decoding |
| `render-retained` | Typed retained render without mutable graph publication |
| `parseMutable` | Actual Swift parseMutable, including its fresh C registry/native graph construction, followed by close |
| `snapshot` | Native export/validation, binary C crossing, Swift owned graph decoding/disposal |
| `apply` | Full predecoded graph encoding, C crossing/validation/import, revision response |
| `render` | Actual MutableDocument.render, including post-render graph publication and Swift graph/result decoding |
| `lifetime` | parseMutable → snapshot → no-op apply → render → close, including temporary owned values disposal |

These are actual API costs. They include FFI, Swift Foundation encoding/decoding, registry work and extra mutable publication. Do not subtract native component medians and call the remainder pure FFI overhead. No JavaScript ratios or allocator/RSS comparisons are produced.

Options and host closure construction stay outside measured loops; serialization intrinsic to public API calls stays inside. Per-operation autorelease pools include draining in the measured ownership scope. Retained stages create a fresh document and warm render outside each batch. Apply reuses its predecoded baseline graph. Batches cap at 128 operations and 25,000 workload units to bound retained scheduling/Date arena growth; before/after node counts are reported. Full output hashing and verification rendering occur outside the timer. Startup, module loading, control pipes, AppKit/DOM decoding, painting and GPU work are excluded.

Correctness-only mode executes all eight API paths once for every selected workload, without reading an operation timer. Timing requires an explicitly coordinated quiet window:

```sh
node scripts/perf/swift/run.mjs --quiet-run \
  --cases flowchart/medium,er/medium,gantt/medium \
  --trials 7 --batch-ms 12 --warmup-ms 100 \
  --output dist/evidence/perf-swift-api.json
```

`--filter medium` selects all seven medium workloads; `--stages snapshot,apply,render` selects stages. Swift independently refuses measured commands unless the driver sets `PELE_PERF_ALLOW_TIMING=1`. The driver rejects active Rust/Swift builds, links and Pele xctest processes before timing; it never stops unrelated work. Test ownership uses process argument paths containing this checkout or a Pele test bundle. Unrelated xctest processes (including Kamui) are recorded as external load and do not block indefinitely. Arguments are inspected only for ownership; reports include command names, scopes, CPU and PIDs, never argument/environment dumps. Results describe a shared host and do not claim a globally idle machine.

Six one-operation warmups and at least the requested warmup work duration precede bounded calibration. Trials retain raw batch averages, median/MAD/min/max and p95 of **batch means**, not per-operation tail latency. There is no cross-runtime ratio or causal attribution from this consumer alone. Exact outputs are checked again after each stage request and each case. Any source, SDK, executable or output mismatch aborts without writing a success report.
