# Independent prepared-ingress / source-text review

No concrete source blocker in the six changed files bound by `SWIFT_PARSE_REVIEW.json`. Source-only review against ef9 runtime; current f9aa728 is documentation-only relative to these originals. No build or test was executed by this reviewer. Boundary's separate execution remains its own evidence.

## Ordered admission and proof ownership

The borrowed physical decoder and `SourceCanonical::prepare_overlay` are unchanged. Complete canonical size remains first, followed by decoder version/string-table/UTF16/node-table/global-work gates, before replacement vectors materialize. The removed repeat pass ran these same predicates on that exact materialized candidate.

`SourceIngress::prepare` now consumes its exclusive owner borrow and retains the exact complete overlay in private fields. `PreparedSourceIngress::admit` performs the first authoritative structural validation; only its scoped immutable reader reaches the engine closure. Stale revision then revision exhaustion remain before `compiled_schema()`. Schema IDs are still checked by the existing shared walker, in root, node prototype, property, collection order. No summary changes the first failing ID. Registration/module repeated structural checks are omitted only for that already-validated immutable candidate. `AdmittedSourceIngress` alone carries it into the same eager importer and commit.

The private source owner still checks exact layout identity. Creator TypeId filtering remains unchanged; forwarded custom-provider capabilities cannot enter this path. No `TrustedGraphRead`, global receipt, generation cache or unrelated API gains trust. Failed checks drop the prepared overlay; accepted state and revision remain unchanged. Commit still imports before adopting, stores a latent importer Result, invalidates accepted admission state, then advances document revision at the existing boundary. Partial-helper and later repair behavior remains covered by existing public fixtures.

## Text semantics

The initially proposed public `CapturedRead::original_utf8` hook was removed after review. Public/custom readers retain the strict default importer without a new trust surface. Private `imported_source` constructs both `CapturedLogical` and the optional text policy from the same concrete immutable `SourceCandidate`, and calls the same importer body.

Original source catalog strings are valid Rust UTF8, with units recorded from that exact immutable text at catalog creation. Candidate text lookup checks current extent and replacement presence first. Every replacement `JsString`, including valid UTF16 equal to the old source, follows the existing strict chunk decoder. Lone high/low surrogates, missing fields, duplicate first-match fields and canonical array-key interpretation therefore retain their existing behavior. Original-key equality uses exact UTF8 equality; replacement equality still compares exact UTF16, without normalization.

The importer still queries string length before returning an owned label, performs label-cache lookup in the same order, and charges the same UTF8 byte lengths at the same points. Cached duplicate nodes charge once per cloned label; first labels charge twice. Title/accessibility reads stay after node/link import. No new lazy projection or second importer was introduced.

## Public feature coverage and limits

The revised feature slice compares sparse ingress with the existing full-graph public apply path. It adds root unknown-method versus detached unknown-prototype ordering, stale revision precedence, original catalog-key replacement, first matching duplicate field with a successful render, supplementary UTF16 replacement, both lone-surrogate failures, empty labels and repair. A compact repeated-node fixture crosses the existing128MiB owned-label budget and verifies latent error, revision, snapshot and successful repair/helper behavior. Existing physical65MiB, table limits, aggregate work, malformed-root/stale and partial-helper fixtures remain intact.

Review corrected a test `BuiltinFunction(MethodId(...))` type error and strengthened an initially invalid renamed-key fixture so unrelated missing IDs cannot mask first-match behavior. Runtime execution is still necessary; no performance or regression-free claim follows from source review. Compiled-schema failure position is source-proven here: the selected builtin uses a valid static schema, so this fixture does not manufacture an unreachable builtin schema failure.

Final freeze: all six reviewed file hashes match manifest and integrated e518012 runtime. Patch `ff0704f5264f47ce3c4022c8d42b5c0ea96f3c817936f9360b1af49fa47388e7`; manifest `fa2657e2db57acb5cb07c1c4537795c4a7743e6ad529bdd02f2f6b62234648fe`. Host-only `admit_observed` cfg follows its remaining callers; public CapturedRead remains unchanged. Boundary reports 32 unique isolated public tests passing; initial fixture compilation and missing isolated corpus failures remain preserved in its logs. This review itself executed no tests.
