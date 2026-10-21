# Functional port checkpoint

All 33 diagram modules are implemented. This checkpoint closes the selected ordinary
constructor, callback, helper, resource and package-export work. **0.3 is not released**;
the existing Pele JS performance gate remains unmet and performance work was paused.

| Final fresh-artifact lane | Actual result |
| --- | --- |
| Swift, macOS arm64 SDK |99 features passed, zero failures/skips, 55 classes |
| Strict C consumers |7 clients compile and run successfully |
| Packed WASM browsers |387 passed, 129 each Chromium/Firefox/WebKit, zero failures/skips |
| Package type catalog |root/full/lazy/core/registry and all 33 model names pass |
| Public runtime oracle audit |33/33 callback matches; 22 class roots construct |

The native source 427 and SDK 13 maps are unchanged across build/test. Web source
499 and package 119 maps remain unchanged through the final run. Native
compiled inputs are pinned from `73b0988`; the final web shell source is `0eeba91`.
The native build's later snapshot HEAD `5c505be` contains only a browser test addition;
all native compiled inputs are identical. Exact SHA256 maps and command receipts
are in the archive. This host run is not a new universal Intel SDK or macOS14 run.
The separate earlier AppKit GUI 31-pass lane retains its original SDK/source identity.

## Corrections and scope

Constructed Agentflow diagnostics, retained GitGraph iterables, Flow subgraph dispatch,
State getters, Radar sparse shapes, original calendar errors, native presentation/image
reentry, opaque Swift host values, and Gantt direct Calendar methods use shared Rust
owned operations. Constructor shells preserve receiver/subclass/newTarget behavior
and native authority. Renderer-local schema identity remains invariant across
local and authenticated registrations. Package type exports include all 33 models.

The final numeric Swift matrix exposed a real XYChart port error: an inverted
`>=` break never terminated after a NaN stride. The exact Pele `<` loop condition
now terminates in typed/live/native/WASM lanes. A focused public regression matches
normal/raw SVG, dimension bits, callback traces, identities and recovery. The stopped
48.3 GiB attempt and sampled stack are retained, not counted as a pass.

Firefox lazily inserts intrinsic constructor `name`/`length` properties. Two separately
imported copies of the same pinned Pele artifact expose different orders under the
same reflection workload; raw controls and rejected attempts are retained under
`api-final-constructor-order`. Only constructor intrinsic descriptor ordering is
excluded in Firefox. Chromium/WebKit keep exact order comparisons. Names, arity,
descriptor flags, prototype member order, method/getter reflection, original errors,
class identity and newTarget comparisons remain strict in all engines.

This is functional compatibility evidence within the documented host boundary,
not proof of arbitrary Proxy/coercion behavior, modified builtins, every malformed
value or native reconstruction of JavaScript collection mutation history. Existing
constructor boundary details remain in [the constructor contract](../../LLD_PUBLIC_CONSTRUCTORS.md).
No timing result, performance parity, release approval or publication is inferred.

## Retained attempts and portable verification

Initial browser 372 run: 352 pass/20 fail. First rebuilt 381 attempt: 249 pass/5 fail/
127 not run, interrupted. Broad Swift attempts and the later NaN run were interrupted;
none is an overall pass. The first Swift C20 build failed definite initialization;
its local-storage repair is included. The first C20 client expected the wrong JSON
envelope; its correction preserves every lifecycle/error/fallback assertion. Stale
Swift exact revision counts now check monotonic progress and current snapshots,
while retaining SVG/bits/state/alias/error assertions. No failures were deleted.

Run `python3 verify.py`. It verifies archive/manifest hashes, every original byte
stream and the recorded outcome checks without Git, packages, SDK or browsers.
`python3 verify.py --restore NEW_DIR` restores all original evidence paths. Generated
libraries, executables, app/test bundles and dSYM are excluded from payloads; paths,
sizes and SHA256 remain under `excludedBuildArtifacts`. Content deduplication retains
all original report names and observations. `capturedHead` is archive assembly HEAD;
actual tested code and artifacts are identified by the per-run maps.
