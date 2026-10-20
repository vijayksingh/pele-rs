# Direct public ER construction

Implementation derives from the independently reviewed proposal against `4e9faee`. The initial native feature slice passes 28 public integrations, including all 17,437 frozen ER sources, exact graph tables, admission boundaries and fallback errors. Rebuilt Swift/web validation and matched performance remain separate acceptance requirements.

## Decision and scope

For ordinary declaration-first ER sources, use the existing public parse plan and optional direct construction hook to build the final canonical native arena during the existing grammar walk. Remove the complete temporary `ErModel.entities`/`relationships` representation and its later conversion pass. Keep one transient attribute block, a small header, ordered entity references, and a keyed name-to-ID-atom index. The final state remains the existing `ErPublicState { graph, model: OnceCell::new() }`.

This layer is deliberately conservative. It supports first declarations with attributes/aliases, repeated bare entity references, relationships after their entities, and ordinary metadata. It falls back once for class/style/subgraph declarations, duplicate alias or attribute declarations, and new entities after relationship publication has started. Larger inputs also retain the original path. This keeps **exact initial binary graph bytes**, including node IDs, string table insertion order, attribute property order, unused strings, descriptors, and private aliases. There is no ID remapping or new transport.

All 8/48/200-entity ER workloads in `scripts/perf/cases.mjs` fit the eligible shape and size. The public integration candidate explicitly calls the existing optional hook to prove this, then compares normal PublicEngine output with an explicitly hook-None provider. Those eligibility and exact binary comparisons pass in the native integration slice.

The immutable normalized ER rollout is separate. It changes `ParsedDocument::model_view`; it cannot improve normal mutable web parse. Do not combine its Swift result with this layer's web claim.

## Current web dataflow and measured evidence

The current public web `parse` uses `WasmRegistry::parse`, which calls `PublicEngine::parse_with_limits`. Normal builtin dispatch uses shared `Engine::prepare_parse`/`ParsePlan`; Gantt already has a direct target, ER currently goes through typed `Er::parse` then `create_public`/`initial`. The typed document is then dropped. `adoptMutable` calls `snapshot_binary`, resolves the captured schema registration, decodes PELGBIN1 into the retained JS wire graph, and constructs `ModelBridge` caller objects.

Thus there are four distinct owners/representations: canonical Rust graph, transient copied binary frame, decoded JS baseline graph, and live caller objects. Ordinary mutable parse has no immutable model JSON projection. Foreign JS providers follow their existing separate identity-preserving path. Source rendering uses typed models without public arena construction.

Preceding source-matched diagnostic: `dist/perf-patches/parse-architecture/projection-policy/profile-summary.json`, WASM SHA256 `b71e7365d757889d76d7e8b1b5e6eb4a6e8d95dc44d09f1d20c6acf76b6b834e`, Chromium 156.0.8078.4, 250-us sampling, 2 seconds/profile. Relevant **self** samples:

| Current parse workload | reconcile | decodeGraphBinary | GC | Selected native self samples |
| --- | ---: | ---: | ---: | --- |
| Flowchart large | 21.25% | 4.91% | 8.84% | Arena assign_fresh 2.03%, ordered 1.92%, intern_key 1.83%; creation/interner/allocator stacks remain |
| ER large | 23.70% | 5.01% | 13.23% | Er::parse 5.51%, ErModel::add_entity 3.74%, set_class 1.48%, GraphWriter::property 1.86%, string 1.65% |
| Gantt large | 17.93% | 1.77% | 23.44% | Direct initial_record 2.57%, Context::text 2.56%; direct construction already active |

These samples are not additive speedup estimates. Function self excludes callees; the sampler does not assign GC to one allocation site, and folded/inlined symbols do not identify every exact source operation. The profile shows material native construction/lookup work alongside larger host publication work. It does not prove removing all arena creation is possible or predict a particular percentage win.

Ranked options:

1. **Direct ER construction:** removes a complete native temporary model and a repeated traversal, while replacing linear entity searches with construction-local stable handle lookup. The same final graph and adapters remain. This is the selected bounded architecture slice.
2. **Host publication:** reconcile is a larger sampled block, but the live mutable objects are required and the baseline has real callback/snapshot/method/recovery consumers. Previous packed publication, host-text, and combined ownership-index candidates passed correctness but failed to establish reliable latency/heap improvements. No rerun by renaming that architecture is justified here.
3. **Validation/schema or small interner tweaks:** retain their own correctness value, but the parent measured captured-schema-view work below 0.6% inclusive in current large parse. They do not remove the repeated model representation. Global StringIndex variants were already rejected.

This layer leaves public graph frame bytes/layout and all JS object publication work exactly unchanged. WASM binary code size and code layout can change and must be measured. It cannot by itself close an 11.34x aggregate parse gap. It targets actual native work rather than claiming that every cost is bridge overhead.

## Shared grammar, shared graph layout

`er/parser.rs` gains a private `Target` whose methods describe ER declarations: entity, relationship, metadata header, subgraph, styles and classes. The existing reader/tokenizer still owns all token order, Unicode rules, alias/comment parsing, grammar errors, line positions, and statement boundaries. `Target for ErModel` delegates the old operations in their old order. It does not add string copies to the typed path; class-name slices remain the original borrowed one-element slices.

`er/public_model.rs` extracts three existing publication sequences:

- `publish_entity`: identical attribute keys/comment/type/name evaluation, comment-first property order, arrays, style fields, entity fields, and map key interning. A borrowed `EntityPublication` view serves both the old typed adapter and direct construction. It returns the already-interned entity ID atom for future relationships.
- `publish_spec`: identical three enum strings and record property order.
- `finish_initial`: identical root value evaluation, property key insertion, instance node, and `GraphWriter::finish` validation.

The old adapter still creates all classes/subgraphs/lookup/claimed entries as before. No runtime Context, import, apply, method, render, callback, or schema code changes.

`er/direct_parse.rs` owns:

- the current `GraphWriter`, which remains the canonical string interner and graph storage;
- a header `ErModel` whose entity, relationship, class and subgraph vectors stay empty;
- ordered `(entity-name Atom, entity-reference Atom)` entries until the entity map is published;
- a construction-local `HashMap<String, Atom>` naming the existing native entity ID string;
- relationship node references, and an explicit entity-map publication phase;
- a fallback marker, never published to callers.

Each attribute block is converted immediately and released; no complete typed attributes/entities collection survives until a second pass. Generated entity ID formatting remains Rust. Relationship endpoints reuse existing ID atoms; their original string interner operations were guaranteed hits, so omitting them cannot change table order. HashMap iteration is never used for observable order.

## Exact publication order

Legacy emits every entity and its attributes first, then the entities Map, every relationship/spec, the relationships Array, classes Map, subgraphs Array, lookup Map, claimed Set, and root.

Direct emission follows that same order. The first relationship may implicitly create its previously unknown endpoints before sealing the entity map. After sealing, another new entity requests fallback. Repeated bare references to existing entities remain safe because they cannot change published attributes, alias, or class values. Any duplicate alias declaration or nonempty attribute block requests fallback even if its final value might happen to be redundant. This avoids retaining a deferred entity mirror or compacting/reindexing the graph later.

Classes/styles/subgraphs request fallback. Therefore direct success always has empty classes/subgraphs/lookup/claimed containers; their node order still matches the legacy adapter. Attribute comment-before-key provenance is passed directly from the grammar to the shared publisher. Root metadata is deferred, so overwritten titles/directions do not leave extra interned strings.

## Admission, errors and lifetime

A private eligibility cap accepts only prepared source and prepared title each at most 64 KiB. This is not a new public limit: larger input immediately returns None and follows the original typed path. It is intentionally conservative for the first construction layer. It prevents speculative construction from consuming arbitrary storage before final graph validation; it does not attempt to optimize every valid ER input.

For admitted source size S <=65536, count E distinct entities, A attributes, K attribute keys and R relationships. Each requires source bytes, so the crude separate bounds E,A,K,R <=S suffice. Direct graph nodes are `3E + 2A + 2R + 7`, at most `7S+7 < 1,000,000`. Own properties are `6E + 5A + K + 8R + 11`; adding entity Map atom entries gives at most `22S+11 < 8,000,000` graph work entries. Each dense array length is <=S. Identifiers/labels share interned strings, generated IDs add a bounded prefix and decimal index, and numeric index keys have at most five digits. A loose string-unit bound of `25S + title_bytes + 1024` remains far below 16,000,000; distinct value/index/fixed-key strings are bounded by `7S+128 < 1,000,000`. UTF8 byte lengths upper-bound UTF16 unit counts. These are conservative admission arguments, not performance claims. Final exact `GraphWriter::finish` validation remains authoritative.

The original ER constructor has no Gantt-like per-operation tick budget to replay: it uses GraphWriter and final ModelGraph validation. Direct successful storage is byte-identical and gets that same validation. Existing schema validation still occurs at the original successful parse-plan stage; raw/prepared/typed adaptation continue their existing paths. ER does not customize numeric edge-limit semantics, so its direct hook consistently ignores the captured edge policy, like its existing typed/create implementation.

Unsupported construction marks fallback and returns an internal error. The target and tokenizer are dropped before returning None; PublicEngine then consumes the same already-prepared ParsePlan through its existing typed parse/create path exactly once. No re-preparation, second provider selection, host callback, or partial publication occurs. Genuine shared-grammar errors before a fallback request return unchanged. If final direct graph admission fails, the whole temporary target is dropped and legacy determines the exact original error or success. Syntax errors therefore still precede constructor errors.

The graph is fully built before parse returns. First render still uses the existing empty typed-cache cell and existing importer; edits/methods/callbacks do not trigger a new migration or delayed parse. Applying a graph, rejected apply retries, revision changes, detached objects, caller descriptors/prototypes, IEEE bits, UTF16 strings and original host exceptions remain on the existing runtime paths.

## Public integration and measurement

`crates/pele/tests/public_er_direct_parse.rs` contains four public feature tests:

1. All 17,437 ER corpus sources/options against an explicit `Legacy` wrapper whose default direct hook is None. Compare full binary snapshots or complete errors, not only visible nodes/SVG. The frozen grammar oracle suite remains independently required.
2. Public hook eligibility for the exact measured 8/48/200-entity source shapes, plus normal builtin-engine snapshots versus the forced legacy engine. This prevents benchmarking accidental fallback as direct construction.
3. First unchanged render and first edited render, exact outcome graphs, method-return alias identity, detached readonly NaN payload, saved entity/attribute aliases, root cycle, and retained document behavior after replacing its registry entry with a hook-None provider.
4. Exact 65,535/65,536/65,537-byte prepared source and title boundaries through the public hook, plus ordinary/frontmatter parsing versus forced legacy (successful full graphs and complete syntax errors). Larger source (including a real 16M-unit graph rejection followed by a later syntax error), duplicate attributes/aliases, declarations after relationships, styles/classes/subgraphs, later syntax errors, exact source/edge policy values including null/-0/NaN, and byte-identical legacy fallback.

The four direct-construction integrations and existing ER oracle/public/normalized projection/provider integration targets pass (28 cases). Existing host callback and descriptor checks remain relevant; the separate baseline resource-callback failure is still open. Parent should run whole browser ER behavior and appropriate Swift mutable consumers after rebuilding; immutable normalized projection remains a separate policy and must stay independently covered.

Measure current control versus candidate using identical JS wrapper files and harness/oracle hashes. Native stages: typed parse, public parse, source render, retained typed render and mutable first render, plus allocation counts/requested bytes/peak/live separately. Typed/source ER paths are real negative controls because grammar factoring changes monomorphized code even without logical behavior changes. Other-family controls distinguish shared allocator/code-layout effects.

VM: all three ER sizes, both metrics/canvas resources, parse/dispose plus first unchanged render, edited render, method call and callback-bearing render. Record exact full graph/first-method identity signatures before/after timing. Use balanced trials and matched control/candidate operation counts; preserve raw paired samples, first-use checks, package/source/binary hashes, and unchanged-family controls. No new timing-only path, cached source, deferred materialization or native-only benchmark may substitute for full web behavior. A public-method ER first-use scenario may need adding alongside the current Gantt-specific getTasks scenario; do not label an ER parse+render row as method coverage.

Reject or narrow if exact graph bytes diverge, fallback changes original errors, typed/source controls materially regress, or complete first-use behavior shows costs merely moved. No adoption or speedup claim follows from this private patch.

## Existing callback limitation

A new first-use check, run before this parser change, finds ER resource-callback
mutation/reentrancy mismatches against pinned JS. The same failure exists in
the prior control: measuring callback edits can be overwritten by publication;
semantic field edits can also affect JS's in-progress render, and nested
addEntity calls are rejected by the native busy guard. Direct construction does
not change rendering or fix that failure. Callback acceptance stays blocked;
passing parse/graph/SVG corpus slices do not establish universal API parity.
The failing assertion remains in the first-use comparator.

## Consumed token ownership

The shared ER reader transfers each consumed token String into its existing
consumer instead of cloning it. Its cursor is monotonic: no backtracking,
consumed-token lookup or vector replay occurs. Error reporting observes the
current unconsumed token and unchanged original source; terminal sentinels are
not consumed through this operation. A direct-target fallback drops the reader
and tokenizes the prepared source afresh. Any future rewind/token-reuse design
must revisit this ownership invariant. Lexer and destination allocations remain;
this removes a copy without changing the parser or public contracts.
