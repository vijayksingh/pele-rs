# Captured web correctness checkpoint

Source `00d2411`, opt-in `captured-sankey` release-profile WASM
`fbaa003fc95224a27cd8f7a49964a4a271b85f9a1dfaac4c44a3ecc4bed3dee2`:
450 browser tests pass, zero failures/skips, across Chromium, Firefox and WebKit.
This comprises 417 existing checks and 33 selected-backend checks. Tests enable
only their candidate renderer; controls and ordinary suite pages retain defaults.

All 408 source inputs match immutable Git source and stayed unchanged from build
through correctness. All 112 packed files stayed unchanged. Source, package and
after-run records preserve their hashes; the complete raw log is losslessly
compressed. Native/Swift SDK proof is separate and remains pinned to `25d87cb`.

Correctness does not approve performance: this JS-column backend is
[rejected for activation](../captured-backend-rejected/). Its default selector is
off, and v0.3 remains blocked. The later native-owned-storage proposal has no
browser or performance claim from this checkpoint.

Run `node verify.mjs`; optionally pass `--repo /path/to/pele-rs` to recompute all
408 source hashes directly from the pinned commit. No browser/build/timing runs.
