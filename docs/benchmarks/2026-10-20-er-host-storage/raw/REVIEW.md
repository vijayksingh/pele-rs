# Independent caller-root adoption review

Source-only review of `files/` against `originals/` and the current legacy `ModelBridge.snapshot`, binary encoder, raw entry constructor and model registry. No production edits, builds, tests, browser work or performance measurements. The files are authoritative during correction; the initial 07f116 patch/manifest predates the caller-initial audit correction and must be refreshed before application.

## Findings and disposition

1. **Native integration compile blocker:** `Replacement` omitted required `PublicDiagramModule::render_public`. Requested an `Er.render_public` forwarding implementation. **Resolved:** independently reread the added forwarding implementation; signature matches the required trait method.
2. **Initial Error-brand transport edge:** caller-initial audit performs the second `errorKind` read but discards its result. If a Proxy reports a recognized Error prototype for the branch test and an unrecognized one for the payload read, legacy encoding rejects `undefined` errorType before config serialization. Requested a deferred invalid-transport flag, rejected after the complete zero-metadata size gate, like invalid array length. **Resolved:** independently reread `invalidErrorBrand`, the unconditional second read per Error node, and rejection after the zero-frame gate before config. This is a narrow preservation of a read already deliberately included; it does not promise universal Proxy equivalence.

Both findings are resolved in current private files. No remaining concrete source blocker; ready for parent compile and public validation once the patch/manifest is refreshed.

## Architecture and ordering

`HostErEntrySelection` retains an Arc to the selected exact builtin ER registration. Other codecs, including a custom module with the same descriptor/schema, stay on the legacy path. Schema access and policy binding do not parse, import a drawable or create an arena. The WASM selector allocates its Rc origin token only for eligible raw selection. `bind_prepared_host_er` checks registry-open, then origin, then config decoding. Registration replacement cannot silently select another policy. Selection and policy hold no caller root.

Caller admission eagerly traverses original objects into the existing identity/string/history ledger. It does not retain properties or Atom values as a second model. Imported caller objects are not inserted into `registry.owners`; compatible last-live helper routing remains available. New objects created by subsequent native methods still use the established direct-owner publication path.

The initial zero-metadata byte equation matches `encodeGraphBinary`: 32 framing bytes plus root Atom, string lengths/UTF16 and canonical node/property/payload sizes. Adding 13 bytes reproduces the old snapshot metadata `{"type":"er"}`. Both gates use captured admission counts, even if first config serialization mutates caller data. Incremental host table/UTF16/reference guards retain the snapshot discovery stage; native graph validation remains represented by registry-derived IDs and bounded structural counts. No limit was raised. The initial Date intrinsic read, array length read and second Error-brand read happen before config, matching snapshot traversal; malformed scalar payload rejection must stay at the encoder stage.

The first config serialization stays under `registryCall`, after initial traversal/zero-frame admission. Binding rechecks open after traversal. A successful adoption enters the existing raw host operation, which performs the second first-render config serialization. No native or JS mutable borrow spans a resource callback. Existing host operation nesting and original-throw recovery are reused.

`documents.set` occurs before helper eligibility for caller roots, retaining primitive weak-key rejection at adoption. Failed provisional owners are disposed and release their retained references; their local native handle and selection are freed. Successful owners retain the caller root and exact aliases; disposal drops owner references without altering caller data. Prepared/source adoption is unchanged.

## Deliberate behavior corrections

These must remain explicit in final documentation and public tests:

- Failed provisional raw owners no longer become same-job fallback owners for borrowed helpers.
- Config serialization writes to caller objects remain live instead of being overwritten by reconciliation of a pre-config snapshot.

Those corrections are separate from preserved callback/resource semantics and do not justify unrelated changes to provider selection or helper routing.

## Execution gates

Parent must compile the new native tests and WASM exports, run the complete caller callback/root controls and forced-legacy error/order controls in all three engines, and retain raw evidence. Check original thrown identity, first/next SVG and dimensions, nested raw callbacks, repair/reuse, borrowed helper routing after source disposal, close/origin precedence and weak-key behavior. Near-64-MiB zero-versus-13-byte admission is established here by source arithmetic only; the candidate has no new near-limit browser fixture. No runtime parity or speed claim follows from this review.
