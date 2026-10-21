# Sankey live resource callbacks

Source `8175006345f3b66a6c9fd942814ed0ba5ff3014d`, building on `0cf69f1`. Default WASM SHA256 `d5a8cbdccd9641e63acefb2d5ed66d3baa48c3d9cb70e5dc1917ca7c17f772b4`. The frozen package has117 files; all558 runtime source inputs match the committed revision and package bytes remain unchanged through browser checks.

The [detached render boundary](../../../LLD_DETACHED_RENDER.md) ends the mutable document borrow before resources, uses the shared Rust geometry/SVG pipeline with phase reads, and acknowledges the latest revision without restoring caller objects. Nested helpers retain partial effects and aliases. A separate result acknowledgement permits factory reentry after native admission. Typed host failure operations preserve the existing native/Swift error wire while obtaining browser-native messages from isolated intrinsic operations.

## Direct oracle results

- Node:122/122 actual-Pele scenarios pass.
- Chromium, Firefox and WebKit:105/105 feature tests pass, including122 direct callback comparisons per engine (366 total), all33 low-level entries and existing binary/provider/public-model controls.
- Native57 integrations across11 binaries; fresh macOS arm64 SDK; Swift16 tests with12,481 drawings,16,572 expected errors and zero explicit budget differences; strictC3 consumers; API168 conditions across21 cases. All370 native/SDK inputs and117 package files remain stable through checks.
- The generated source initializer is byte-identical in two runs against this final WASM. TypeScript consumer checks pass.

Comparisons cover complete SVG, dimension bits, resource ordering, ordered property descriptors, aliases/cycles, retained references, arbitrary object/symbol/undefined throw identity, responsive redraws, current/later field reads, nested same/other-owner rendering, helper reentry, partial failure, repair and subsequent calls. Expected values come from pinned actual Pele1.6.0 in each engine. No Rust path supplies its own expected values.

Generic JavaScript Error stack locations remain engine/source implementation details: raw sourceURL/fileName/line/column/stack are retained separately in each lane's top-level engineErrors record. Semantic comparison still checks generic error names/messages and original thrown identity. PeleError source line/column/snippet remain strict. User graph properties are never filtered.

## Retained failures and corrections

The [initial audit](../AUDIT.md) records113/122 mismatches in the prior implementation. First implementation attempt passed120/122; nested icon rendering reset Pele's shared regex cursor and repeated an outer icon. A small UTF16 cursor compatibility helper fixes that behavior while retaining shared Rust text layout.

The next attempt passed122/122 in Node but96/105 browser tests. Four readonly helper cases in Firefox/WebKit needed host-native primitive error messages; WebKit strict-string failures also expose the call expression. Six test failures came from raw oracle shape and an invalid expectation of a redundant descriptor rewrite. The corrected fixtures use actual raw Pele rendering and a real helper array append. Failed reports are retained alongside final reports.

## Acceptance limits

These are feature/corpus-specific correctness results, not complete library/API parity or performance parity. Arbitrary getters/Proxy/coercion, malformed/sparse arrays and callback-created lone UTF16 surrogates remain explicit proof gaps. Fresh SDK SHA256 is `4ecfbbfb33ce6a0fba349ed6e9340074ba0fce37d28d2e35603b6e6a027327bd`; Swift consumer SHA256 is `a5f4427f7b8005b949f9d2c49053692ed5b0ef13421c5fda60e44b20ada218e3`. Independent [VM performance](../../../benchmarks/2026-10-21-sankey-live-render/README.md) rejects this candidate for release. 0.3 remains unreleased until the correctness and JS performance gates pass.

[Artifact manifest](artifact-manifest.json) records original and stored hashes. Large JSON is losslessly gzip-compressed; decompress to recover exact original bytes. No failing rows were removed.
