# Sankey resource callbacks: measured contract and next slice

Read-only proposal. No runtime change. Frozen native bb0fc18055a175de4645129ea25a4917a6fa7d17, WASM 6e14438c387a81dddec03647f13b3092efb704b617838055911e4b91bcd907cc. Native candidate loaded directly in Node from the frozen typed03 package, with explicit WASM bytes. No browser, build or timing run.

## Direct differential

The original eight oracle cases and their manifest remain unchanged. `differential.mjs` adds 56 oracle/native cases: current ID, later ID, link value, link endpoint, title, lookup and their combination; width/icon resources; success/object/symbol/undefined throws. Every initial graph matches, every callback fires, all 42 original throw identities match. Post-operation model graphs differ in all 56. Four successful width cases also differ in SVG/callback trace. Recovery output differs in 48; lookup-only writes affect public state but not rendering.

`responsive.mjs` adds 14 success cases with maxWidth 220 and a long later label. Twelve current outputs differ, including width and height changes and callback counts. `helper-reentry.mjs` adds one feature comparison: Pele permits findOrCreateNode('X') inside width, returns the exact lookup alias and preserves the new node. Native rejects it with `Public model document is already in use`.

All reports save full SVG, exact dimension bits, callback traces, own-property descriptors/graph aliases, post-state and subsequent rendering. These are correctness samples, not performance measurements. Native and oracle package identities are pinned in the reports. They establish a broader behavior gap than restoration alone.

## Exact upstream read phases

Source: `node_modules/pele/dist/esm/chunks/CTRBU7NA.js` functions `pe` (layout), `ve` (draw), `$e` (responsive entry); `DKTLEL5E.js::f` (fit loop); `ESCH476E.js::bt` (public render).

1. Each draw computes layout from current `nodes.length`, link endpoint indices and link values before its first width callback. Derived positions, links, values and thickness stay captured for that draw.
2. Each placed node obtains `t.nodes[index].id` when that node starts. Its rect and node-color lookup happen before its width callback. The captured label is reused for that node's text after the callback. Changing the current node's ID does not rewrite already-captured label text.
3. Later nodes obtain their IDs after earlier node callbacks. Changing B in A's width callback changes B's measured text during this draw.
4. After all node measurements, links read current source/target node labels using the already-captured layout endpoint indices. A's changed ID therefore changes its link labels even when A's own text was captured before the mutation. Current link values/endpoints remain the layout snapshot for this draw.
5. Title is read after link construction. Its label pipeline may call icon and width resources. Changing title from a node callback affects this draw; changing title during its own icon callback does not replace the title value already captured.
6. Root accessibility fields are read at final SVG document creation, after title resources (source inspection; not separately exercised here).
7. Responsive fit may redraw several times, recomputing layout from the now-mutated model at every draw. The maxWidth probe proves value and endpoint mutations can alter the SAME public render result on later draws.
8. No model rollback follows success or throw. Icon global resolver/cache scope is restored in finally; public model is not.

Current native `sankey/public_model.rs::render_public` lends one preimported Legacy/Indexed projection into `render.rs::render` for the whole call. `draw` reads its labels/title and recomputes layout from that same projection on each fit pass. `full.js::publishMutableOutcome` subsequently reconciles old native state over callback writes. Both boundaries require correction.

## Smallest coherent feature, not a post-return shortcut

Recommend one family-scoped **live render session** for the captured builtin Sankey registration, with ordinary native source/Swift paths retaining the current typed input adapter. This is one shared Rust geometry/SVG implementation with a live input adapter, not another parser, diagram renderer or canonical graph store.

A. Split `render.rs` input access into exact phases above. Preserve layout/math and output sinks. Introduce a private render-read interface whose default adapter reads existing Input. Live adapter obtains only the field required at the original phase and owns that value across a subsequent callback. It must not prefetch later labels/title or reimport the whole model after each callback. At every responsive draw it reloads geometry inputs; it cannot reuse an earlier draw's link values/endpoints. The existing label access/coercion/error policy must be factored rather than replaced with eager importer validation, which would reject unrelated mutations too early.

B. Bind the browser adapter to the actual public root through generic property/index/string operations. Family field selection and phase ordering remain Rust. Current label values are copied/retained before the width callback; later reads consult the live root, including replaced node arrays. No persistent JS mirror, Proxy wrapper or lazy arena conversion. Native errors from host field access retain the original exception channel. Wrong-type/deleted fields and accessor reads need explicit differential gates before claiming unrestricted parity; passing the string-only probes is insufficient.

C. A trusted Sankey read-only render outcome acknowledges revision/result without overwriting ordinary JS state. Keep the accepted native baseline for next operation's existing observation/admission, and mark observation unconfirmed after resource execution. Do not adopt callback edits eagerly on exit: doing so can introduce validation/errors that Pele does not perform. Native helper partial effects still use their normal publication path. Other families/providers retain existing policy; readonly status must come from the captured builtin creator capability, never schema/type strings or public forwarding.

D. A complete callback feature must permit same-owner helper/render reentry. A mutable document borrow/busy guard cannot span user callbacks. Obtain owned/Arc render configuration and operation context first, release document access, then run the Rust render session. Nested helpers use the existing shared Rust helper implementation and advance their own accepted revision; outer completion reads current revision and must not overwrite nested state or rewind to its entry revision. Scoped resource resolver stacks and original-exception priority remain intact. Dispose remains governed by the binding's explicit lifecycle API. This requires a concrete revision-token protocol review before coding; current source does not yet provide it.

A+B+C address the 70 mutation probes. D is independently demonstrated and prevents calling A+B+C alone complete public callback parity. Do not quietly retain busy rejection while claiming all callback behavior fixed. If initial implementation intentionally excludes reentrant helpers, label it a narrower plain-data mutation correction with the remaining oracle mismatch recorded.

## Scope / files / gates

Likely scope: `sankey/render.rs` and a private render-read adapter; `sankey/public_model.rs` captured readonly/session capability; public engine and WASM wrapper only for scoped document borrowing/revision acknowledgement; `bindings/web/full.js` and generic live field bridge; ONE public Sankey callback differential feature. No generated initializers, recipe format, source parser, other-family renderers or shared restoration algorithm rewrite.

Before implementation agree the session/revision design and exact field-error handling. Then one consolidated feature gate compares oracle/native full SVG, dimensions, trace, graph descriptors/aliases and arbitrary exception identities across successful and throwing callbacks; current/later IDs, values/endpoints/title/lookup; responsive repeat draws; deletion/wrong type; helper partial errors, same-owner and other-owner nesting; next-operation repair. Keep ordinary unmutated corpus/source/native controls. Existing source-layout restoration assertions must be revised as a deliberate compatibility correction, not silently weakened.

The 30.86% retained matching/publication profile union is not an estimated saving. Pre-operation mutation discovery remains needed; helper outcomes and other families can mutate native state. This architecture is motivated by observed semantic mismatch first. No performance promise.
