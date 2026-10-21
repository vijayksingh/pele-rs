# Independent ER projection review

No blocking runtime finding after the private fixture correction below. The one-line ER builtin opt-in is a bounded application of the existing normalized projector; it does not broaden public-provider policy or change the formatter. The refreshed candidate is ready for the parent's compilation/integration validation, not an assertion that those checks have passed.

Reviewed the README, eligibility audit, private source/test copies, original engine, pinned ER model/lexer/parser and corpus metadata. No build, test, consumer or benchmark was executed. Only the private integration fixture, patch/manifest, README and this review were changed at the parent's request; production source was untouched. The corrected private Rust test was formatted through the repository's configured rustfmt.

## Blocking fixture issue found and corrected

The original order/Unicode fixture was not grammar-valid. `class 2 first` lexes `2` as NUM, while the CLASS parser's `ids()` accepts only UNICODE_TEXT/STYLE_TEXT. Also, unquoted standalone `1` followed by whitespace and the next entity declaration can be lexed as the ONLY_ONE cardinality token. Both conditions would make the fixture's parse().unwrap() fail before testing projection.

The private correction quotes numeric-looking entity names and relation endpoints, preserving the intended string IDs and insertion order `["2", "10", "1"]`. It retains duplicate aliases, appended attributes and repeated class definitions. Explicit assertions now check class-definition order and appended style values. Repeated class assignment uses valid identifier `A` in the separate XML-valid retained-render model, with an assertion for `"default first first"`. This preserves intended public coverage without changing grammar or runtime behavior.

These assertions were checked against the current lexer/parser/model source, not executed. In particular, quoted numeric names use ENTITY_NAME and are stripped by `name()`, repeated aliases retain the first nonempty alias, `add_class` appends style strings, and `set_class` appends each class assignment. The parent must still compile/run the corrected feature slice.

Refreshed immutable review identity:

- Patch SHA256: `6a32ff81f87a0e115d47eefada58dacea3eb4f3293a2545aa8d8aa30522d036e`.
- Runtime engine candidate SHA256, unchanged: `d0d02bd0d3b772e1447854e63914aed21a9919ee48d4a74af121f9622bbd4c85`.
- Corrected integration test SHA256: `052684a0835fb630b840d1198599b80592d5520914aff9156ef5f3e6e024a2c5`.
- Original engine SHA256: `9805911042d8fac20d3343bdeba5ae720a4629c3089aa3dcd5628bcae61ce1b2`.
- Frozen fixture SHA256: `724cd2238adf0492e0bce879034da115e1e6589e27b67c17e52c74f7ac1967c1`; independently read **17,437 cases**.

Manifest and private file hashes were refreshed after formatting. The manifest explicitly notes that the proposal's earlier apply-check preceded this fixture-only correction.

## Runtime and architecture

ER's captured builtin registration changes from public `register(Er)` to private `register_projected::<_, NormalizedBuiltin>(Er)`. Public registration still creates Legacy, even for the ER ID or the same Er type. The existing zero-sized policy representation is retained; this opt-in does not reintroduce a per-builtin policy allocation. Registration replacement and Engine destruction cannot retarget the Arc registration captured by an existing ParsedDocument.

The concrete serialized ER tree is shallow and pure: derived records, unit enum strings, optional strings, vectors and pairs. Its HashSet/HashMap parser bookkeeping is explicitly skipped. Subgraphs contain string IDs rather than recursive model references. The inspected tree has no raw JSON, custom framing, float/Date fields, arbitrary nested Value, serializer side effects or host callback. DiagramType uses the existing enum serializer. Thus its normal public source path is within the supported formatter operations; no compatibility retry is expected from this tree, although zero fallback has not been measured.

The normalized formatter itself is unchanged. serde_json remains the authority for Serialize dispatch, escapes, punctuation and exact counted bytes; Object::insert and downstream enumeration retain the existing normalization rules. Model validation still precedes strategy dispatch once. Generic depth, byte admission, serializer-error ordering and exceptional fallback are inherited from the already reviewed primitive. This source review does not authorize opting in other families, especially RawValue-backed Sequence/Usecase.

## Four public integrations

**Full corpus:** the new test crosses actual `ParsedDocument::model_view` for builtin ER and an independent public-registered legacy ER. It compares ordered object entries, exact string content, number bits where present, array order, complete document-envelope Result/string equality, and complete parse/projection errors. Failed parses additionally compare error envelopes. The fixture count is pinned; cases are not skipped. Deserialize keeps source/ID/options and ignores large expected SVG/model fields. Documents/projections are released per case. Existing oracle tests remain necessary: differential agreement between two Rust paths alone is not a new independent JS-oracle proof.

**Reachable ordering/Unicode and retained use:** the corrected source checks numeric-looking pair keys remain strings in insertion order, first alias retention, appended attributes/classes, BOM, combining Unicode, astral content, NUL/control/backslash escaping and absent options. XML controls are deliberately kept in projection-only input. A separate valid model compares full retained-render results and exact projection before/after rendering. This is actual optimized builtin coverage, not only a legacy-provider probe.

**Exact byte boundary and late syntax:** public typed-model serialization into a count-only writer derives the fixture's base and asserts a three-byte growth per added name byte (pair key, entity ID suffix, label). Accessibility-title padding selects exact limit−1, limit and limit+1 residues. Source stays below independent input admission. Both paths compare acceptance and complete Limit errors, including ER diagram identity and exact message. Accepted giant models are checked for admission, not complete Value equality; smaller corpus cases provide semantic comparison. The malformed declaration appended after the otherwise oversized model must yield the same Syntax error before projection, preserving parse/projection precedence. Runtime size assertions prevent future model-field changes from silently weakening these boundaries.

**Captured policy and provider isolation:** an external ER-ID RawValue provider increments a Serialize counter. A builtin document survives replacement; the first replacement survives a second registration; all survive Engine destruction. Provider projection must serialize exactly once and preserve last duplicate value/BOM. The later validation-failing provider must fail before another Serialize call. This protects legacy isolation. It does not exercise raw fallback, arbitrary-depth or unusual numeric branches on the optimized ER path. ER has no reachable numeric model payload; a generic number-bit comparator is not evidence for new optimized numeric coverage.

No source-inspection uncertainty remains about the two corrected lexer problems or the one-line policy selection. Compilation, actual 17,437-case execution, giant-boundary resource behavior, rebuilt Swift immutable ER corpus/eager handoff and matched projection/parse/allocation measurements remain unverified by this reviewer. Parent validation should retain the existing Gantt/provider checks. No performance prediction, regression-free claim, JS parity or release approval follows from this review.
