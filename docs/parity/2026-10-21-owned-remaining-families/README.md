# Owned callbacks across all 33 families

Functional checkpoint, not full API parity or release acceptance. Class,
Sequence, parsed Agentflow and Gantt now join the other 29 families on the owned
callback path. Parsing, scheduling, geometry and SVG stay in Rust. The fresh web
edit/throw audit matches 33/33 families, with all 22 class-root factories constructing.

| Lane | Observed result |
| --- | --- |
| Native public integrations |55 distinct features; final 8 pass together |
| Swift, rebuilt host SDK |44 distinct features;43 pass together, then one corrected error observer passes |
| Strict C clients |15/17/18/19 compile and execute successfully |
| Browsers |201 distinct checks pass across 207 attempted; six exact TypeError-message checks remain |
| New paired cases |987 recorded;985 match exactly, two WebKit warning rows differ only in TypeError text |

The new Swift matrices compare 325 oracle rows: Class 80, Sequence 72,
Agentflow 64, Gantt rendering 72, start-date 17 and scheduling 20. Browser Sequence
also covers four Map-iterator history cases unavailable from native final graph
coordinates. Browser matrices total 329 cases per engine across Chromium,
Firefox and WebKit. The six remaining browser feature failures are recorded,
not normalized or counted as passes.

## Repairs and retained failures

The first fresh 42-feature Swift run exposed a Class cached-revision failure
and an inherited-accessor regression affecting State and the all 33-entry feature.
Native-owned phases now have a metadata-only revision read before implicit
nested Swift operations; explicit stale apply remains rejected. The ordinary
native getter fallback was restored. After rebuilding the SDK, the original
Class 80/State/all 33 features pass. A new revision regression initially asserted
the wrong Swift error type; changing only that observer to `CompatibilityError`
preserves the exact stale-revision message and passes its focused rerun.

The first browser run passed 192/201 checks. Sequence's nested helper publication
cleared and rebuilt its actor Map, making a suspended iterator visit an unchanged
actor again. Ordered in-place Map publication fixes all 76 Sequence rows in each
engine. The focused 24-check run also passes binary graphs, source initialization
and all 33 raw entries. Six Firefox/WebKit constructor/warning features still
differ in native-engine TypeError expression text. Their complete reports remain
in the archive. Broader API work includes constructed Agentflow logging, iterable
tag cases, constructor reflection and these error expressions.

The C19 lifecycle client initially assumed root coordinate 0 and a clock callback
for a calendar operation. It now reads the actual graph root, handles truthiness
replies and checks reentry from the UTC-offset callback that the resource really
uses. Duplicate/foreign carrier acceptance, one-use execution, cancellation and
recovery assertions remain intact. All four C clients pass against the repaired
SDK.

## Artifact identity

Core/WASM runtime: `c79704b`. Native repair: `efa8e24`. Web Map repair:
`020f285`. Swift observer correction: `394f7e7`. Native inputs 420 and SDK
files 13 remain identical through the final checks. Web inputs and 118 package
files are pinned before/after their runs; only `graph-storage.js` changes between
the two web packages. Both use WASM SHA256
`f49bd82f03d642f324625be7220cb1e82df220f6110d045b59c3c621a8ae208b`.

`evidence.json.gz` preserves lossless logs, input hashes, paired reports, failed
sources and the 33-family audit. `manifest.json` authenticates that archive.
Run `python3 docs/parity/2026-10-21-owned-remaining-families/verify.py --git` to
verify the recorded results against their committed inputs. This is functional
verification; no performance measurement or 0.3 release was performed.
