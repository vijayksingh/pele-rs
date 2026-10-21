# Packet, Radar and Quadrant callback checkpoint

Source `f47a834` shares typed/live geometry for these three renderers. Owned jobs
retain the upstream collection/record identities while reading mutable fields at
their source phases. Rust owns layout, reflow and SVG; host adapters supply generic
membership, object-copy and Map operations.

- Native: 21 feature tests pass, including 48 Packet, 40 Radar and 56 Quadrant
  oracle scenarios. Initial test compilation failures are retained. An unused
  test variant warning was removed and the affected feature rerun successfully.
- Swift: 12 features pass against a fresh ARM64 SDK. New coverage includes all
  48 Packet rows, all 40 Radar rows and 36 shared scenarios across three families
  (12 Quadrant cases). Pie, Info, Gantt, Sankey and all 33 entries also regress.
- C15/C17/C18 strict clients compile and run. The first C15 run reached a stale
  assertion that Pie had no owned job; the test now exercises its owned lifecycle.
  The failed source/log and successful follow-up are retained; no runtime changed.
- Browser: 24/24 features pass across Chromium, Firefox and WebKit. Each engine
  compares all 144 new scenarios against actual pinned Pele (432 comparisons),
  plus Pie/Info and all 33 prepared-entry regressions.

The fresh public audit matches its single callback edit/throw case in **7/33**
families: Sankey, ER, Pie, Info, Packet, Radar and QuadrantChart. **26 still
mismatch.** All 22 class-root constructors succeed. These totals do not establish
full callback, constructor reflection, dynamic coercion or UTF-16 parity.

The before/after maps bind 371 native input paths, 13 SDK files, 447 web inputs
and 118 package files. WASM SHA-256:
`c542357a56fc31b6618fe4ea525f5719a9b3f1299255b05a1e6e626f2f13e8bf`.
Test-only commits followed the frozen runtime; later source is not certified by
these binaries. ARM64 checks do not establish physical Intel/macOS 14 support.
No performance experiment ran. The 0.3 release remains blocked.

`evidence.json.gz` holds 37 lossless records: complete logs, build/input maps,
actual browser comparisons and the current audit. No binary is duplicated.
Verify integrity, counts, full paired reports and source bindings from repo root:

```sh
python3 docs/parity/2026-10-21-chart-callbacks/verify.py --git
```

The verifier reads archived evidence; it does not execute the library. Existing
Gantt calendar warnings, configured Agentflow logging, native graph-coordinate
lifetimes, arbitrary dispatch/coercion, active malformed UTF-16 and the remaining
family callbacks still require work. Swift verifies Error identity, not throwing
JavaScript undefined. Earlier exact WebKit diagnostic differences remain in the
[preceding checkpoint](../2026-10-21-live-render-families/README.md).
