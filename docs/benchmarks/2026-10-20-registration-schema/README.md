# Captured registration schema checkpoint

Measured native sources are commit bfee120; JavaScript is fa9f4e8. The WASM
package was built from the exact candidate source bytes before committing;
Swift consumer metadata records the committed native revision and input hashes.

[Complete VM samples](web.json) cover 21 workloads, two measurement resources
and three stages. [Release gate](gate.json) validates all eight tracked runtime
file hashes, oracle, high-resolution timer and the complete matrix: **126/126
conditions are slower than JS**. No release is authorized by this report.
Stage median ratios: source 2.63x, parse 11.37x, retained mutable 4.02x.
Shared-VM variance and garbage collection remain in raw samples. Differences
from the previous report are observations, not causal speedup percentages.

The code caches only immutable schema validation and membership sets, lazily
at existing operation stages. All per-graph checks and public arbitrary-schema
validation remain. See [architecture](../../REGISTRATION_SCHEMA_CACHE.md).
Seventeen native integration tests and 54 browser tests across Chromium,
Firefox and WebKit passed. No unit tests were added.

[Swift correctness](swift-verified.json) verifies all eight actual API stages
for every workload (168 conditions), current source/SDK/archive fingerprints,
complete SVG hashes/bytes and exact dimensions. Eight Swift integration tests
passed, including all 33 numeric families, output-limit behavior and the new
31-case rounding-boundary corpus across source, immutable and mutable lanes.
[Swift timings](swift-timing.json) contain 21 medium-stage conditions with seven
trials, 100ms warmup and a passing build/test quiet guard. Timings include actual
Swift/FFI ownership and decoding. No cross-machine JS ratio is claimed.

Only the ARM64 SDK was rebuilt here. Universal Intel packaging and a passing
complete VM performance gate remain release prerequisites. Inline number
formatting and string-index variants were rejected; borrowed lexer storage
remains unadopted because timing evidence is inconclusive. The borrowed-outcome
API proposal is parked: measured clone cost alone is too small to justify it.
