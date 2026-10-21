# Pinned Pele callback mutation semantics

Decision: ordinary Sankey resource callbacks mutate the actual public model. Pele 1.6.0 preserves those writes on both successful and throwing renders. Our blanket accepted-graph restoration is a compatibility difference; source inspection and direct oracle execution establish this, without a browser or performance run.

## Executed evidence

`node dist/perf-patches/parse-architecture/callback-mutation-oracle/probe.mjs` ran once with Node v25.9.0. Eight cases passed: width/icon × success/object/symbol/undefined throw. Six original thrown values retained identity. Every case kept changed node labels, cleared lookup, deleted root properties, mutated detached array, cycle/alias identity and an added readonly/nonenumerable data descriptor. Report was written before assertions.

Successful width callback changed node B before its later measurement: trace is `A 6`, `Changed B 5`, `C 4`. Final SVG also contains changed link labels. Successful title-icon callback happens after node/link drawing: its writes persist in the model even though current SVG node labels remain original. This proves phase-sensitive live reads; merely preserving post-call writes cannot fix the entire in-flight rendering difference.

One public helper failure control made `nodes.length` readonly. `findOrCreateNode('X')` inserted X into lookup before Array.push raised TypeError. Node array remained length 3; lookup contains index-3 node; retry returns that exact alias. This partial mutation must still publish from a native helper outcome.

`report.json` pins the actual main oracle SHA d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d, identical to dist/evidence/pele-oracle.js, package and relevant ESM chunks. Main public `parse`/`render` imported from installed Pele; ESM source chunks independently explain the behavior. Source excerpts include original character offsets (bundled files are minified).

## Source and coverage

- `node_modules/pele/dist/esm/chunks/ESCH476E.js`: `gt` obtains the original parsed model; `bt` directly calls `r.render(m,...)`. `Se` only converts RangeError. No graph snapshot or model rollback.
- `node_modules/pele/dist/esm/chunks/CTRBU7NA.js`: `ve` captures each label before width callback, then reads subsequent nodes and later links from the live model. Helper Map insertion precedes Array.push.
- `node_modules/pele/dist/esm/chunks/ND66TM4A.js`: `G` finally restores icon resolver/cache scope, not public-model state.
- `bindings/web/full.js:196`: publishMutableOutcome always reconciles the native graph. Normal render calls it at line 659 before delivering original callback error at lines 664–671.
- `tests/web/sankey-first-render.spec.js`: real oracle throw/recovery comparison makes no callback mutation. It does not prove rollback parity.
- `tests/web/source-layout.spec.js:36–76`: factory-throw callback mutations are asserted restored without running the same mutations on the oracle. Other recipe/canonical/typed-column lanes compare native transports. Cursor restoration fixture likewise uses oracle only for initial unmutated SVG.
- `docs/LLD_ARENA_RENDER_PACKET.md:99–105` explicitly mandates full callback restoration; `docs/LLD_SOURCE_CANONICAL.md:73–89` carries that contract. This documents current binding policy but does not establish upstream compatibility. `docs/LLD_PUBLIC_MODELS.md:105–109,218–230` instead requires partial effects and replacements only where upstream does them.

## Architectural consequence

Do not declare 30.86% matching/publication union removable. Pre-operation discovery remains necessary for unrestricted ordinary edits. Native helper partial effects and families whose render mutates state require accurate publication. Sankey needs live-read phase correctness as well as post-callback write preservation; a read-only native outcome alone is insufficient evidence to omit reconciliation globally. The concrete next contract work is an oracle differential for callback-mutated state, subsequent callback text and SVG, then a family-scoped mutation/read-effect design. No runtime policy changed here.

Scope: ordinary public parse/render, one source, two resources, six arbitrary throws, one partial helper. Does not establish arbitrary Proxy/getter semantics, reentrancy, RangeError identity (upstream deliberately wraps it), all families or browser behavior. No performance estimates or claims. Root bb0fc18055a175de4645129ea25a4917a6fa7d17 remained clean.
