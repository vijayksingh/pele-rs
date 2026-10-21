# Functional parity checklist (read-only audit)

Audited runtime `d7caf8c684ac6b6fda3918a2bcbd91c5953b92c9`; read-only public oracle and package entry probes, no builds or benchmarks. Public oracle probe uses exact Pele 1.6.0 bundle SHA d1ebc1889f3069d7759fe5db34c4ad4b4823285edbee2b537d0ef3d505a76d8d. These are unresolved checks, not a full-parity certification. PELE_PARITY.md is an old baseline inventory; current guards/tests take precedence over its baseline markers.

## Selected and shared API work

- [ ] C/Swift Sankey same-document callback helper/render reentry. Root selected this slice; no competing implementation here. Current FFI public.rs document() retains a MutexGuard and try_lock rejects reentry as `Native public document is already in use`; browser detached jobs do not establish C/Swift parity.
- [ ] All 33 family resource callback mutation matrices: before/inside width/icon/clock, original object/symbol/undefined throws, responsive redraw, current/next label/endpoint/value reads, next-call recovery, same/other-owner helpers and nested renders. Browser Sankey has actual pinned 122-case evidence; ER has separate host-live controls. The prepared per-type entry probe confirms all other 31 lose a title edit made by the first width callback before it throws. Both lanes call width once and preserve the original thrown object. Generic snapshot restoration therefore fails this concrete live-state case; broader callback phases still need coverage. Source detached_render.rs currently selects only Sankey. This is a test/behavior gap, not a claim all scenarios fail in all families.
- [ ] 22 exposed constructors. `graph.js` registration substitutes a universal throwing constructor. Pinned public constructors all succeeded in this probe, with matching prototype and own-key reports: venn, ishikawa, swimlane, agentflow, requirement, block, railroad, c4, architecture, sankey, state, gantt, sequence, xychart, class, er, flowchart, kanban, mindmap, timeline, journey, quadrantChart. Constructor-created roots stay unregistered for high-level render; raw entry/borrowed-helper behavior must follow pinned rules.
- [ ] Active UTF-16 output domain. Canonical graph storage preserves raw code units; many render imports reject them. Info title `\ud800` succeeds and remains a surrogate in pinned SVG; native public_info.rs:298 explicitly expects a gap. Do not silently replace with U+FFFD or claim JSON roundtrip closes SVG parity.
- [ ] Error class/message/order and partial-state parity. Radar hole/empty circle case confirmed pinned TypeError; native public_radar.rs:596 expects Semantic gap. Empty polygon recovery succeeds. Generic engine stack coordinates are diagnostics, not PeleError source locations.

## Family-specific guarded runtime domains

The source-guards.json.gz companion lists every matched source path/line and exact guard. A guard can be lane-specific; host interception may support a subset. Absence of a guard is not proof. These are dynamic/string-domain gaps, not automatically missing named public methods.

| Family | Explicit guarded domains / remaining feature check |
| --- | --- |
| flowchart | Sparse active arrays; cyclic/nonscalar rendered properties; replaced push/builder methods; object numeric/string coercion. |
| sequence | Sparse active collections; replaced push/builder callbacks; object numeric/string coercion; active UTF-16. |
| class | Active lone-surrogate output. |
| state | Sparse active collections; replaced push/builder callbacks; object numeric/string coercion; active UTF-16. |
| er | Active lone-surrogate output. Browser live-store callback controls exist; universal callback matrix not established here. |
| pie | Active lone-surrogate output. |
| gantt | No literal not-implemented guard found in this scan. Scheduling/callback/override controls exist; this is not blanket parity proof. |
| gitGraph | Sparse active arrays; active text/commit-type coercion; replaced iterators/Map.values; nonnumeric/NaN sequence sorting; lone-surrogate output. |
| mindmap | Borrowed runtime receiver/collection semantics; arbitrary coercion; sparse arrays; active UTF-16 and replaced dispatch paths. |
| timeline | Active lone-surrogate output; overridden collection push. |
| journey | Borrowed primitive receiver; sparse arrays; nonnumber score render; arbitrary-object actor sorting; lone-surrogate output. |
| xychart | Active/builder lone-surrogate output; overridden internal method dispatch. |
| quadrantChart | Active lone-surrogate output. |
| requirement | Active lone-surrogate output; existing corpus records additional active nonscalar differences. |
| c4 | Borrowed runtime receivers; String indexing; arbitrary coercion; sparse/overridden builder and render paths; active UTF-16. |
| sankey | Active lone-surrogate output. Browser 122 callback matrix passed; C/Swift same-document reentry remains selected work. |
| block | Borrowed runtime receivers; String indexing; arbitrary coercion; active cyclic/sparse and overridden paths; UTF-16. |
| packet | Active lone-surrogate output. |
| kanban | Borrowed runtime receiver; mixed indentation/object coercion; sparse active arrays; active UTF-16 and overridden methods. |
| architecture | Borrowed runtime receiver; String indexing; arbitrary coercion; sparse/overridden paths; active UTF-16. |
| radar | Sparse arrays; nonprimitive numeric coercion; lone-surrogate output. Empty circle/sparse axes use wrong Semantic error instead of pinned TypeError. |
| treemap | Active lone-surrogate output. |
| info | Active lone-surrogate output: confirmed successful pinned SVG, explicit native rejection. |
| agentflow | Sparse active collections; overridden push/method dispatch; object numeric/string coercion; active UTF-16. |
| usecase | Sparse render arrays; callable/host scalar JSON coercion. |
| railroad | Active lone-surrogate output. |
| treeView | Active runtime text coercion; sparse children; lone-surrogate output. |
| venn | Borrowed runtime receiver; String indexing; arbitrary numeric/string coercion; sparse/overridden paths; active UTF-16. |
| cynefin | Overridden domain getter dispatch. |
| wardley | Nonnumeric geometry coercion. |
| swimlane | Shares Flowchart mutable codec and its unsupported active/runtime coercion paths. |
| ishikawa | Borrowed runtime receiver; String indexing; arbitrary coercion; sparse/overridden paths; active UTF-16. |
| eventmodeling | Active text/object coercion; sparse arrays; replaced Array.join; unknown runtime entity kinds; lone-surrogate output. |

## Existing tests that deliberately accept differences

- public_info.rs `info_keeps_lone_utf16_values_but_reports_the_rendering_gap_and_recovers`.
- public_radar.rs `radar_explicit_runtime_shape_gaps_preserve_graph_and_recover`.
- public_block.rs `active_cycle_surrogate_and_replaced_method_gaps_reject_then_repair`.
- public_git_graph.rs `git_graph_explicit_runtime_gaps_and_work_guards_retain_graph_and_recover`.
- public_mindmap.rs `mindmap_explicit_runtime_shape_gaps_keep_graph_and_recover`.
- public_kanban.rs `kanban_optional_values_partial_decoration_and_explicit_shape_gaps`.
- public_eventmodeling.rs `event_modeling_existing_utf16_hard_wrap_output_exclusions_remain_explicit_gaps`.
- public_state.rs/public_sequence.rs and other family feature suites include UTF-16, sparse and error recovery assertions; passing totals must not erase expected-gap branches.

Named methods: no newly absent declared ordinary helper identified by this audit. Actual dynamic overrides, borrowed receivers, coercion and constructor capability remain separate. Do not invent missing method names from broad gaps.

## Release/distribution, separate from semantics

- [ ] Rebuild fresh WASM/C/Swift artifacts from the final functional commit and repeat relevant public consumer proofs. Existing artifact success is tied to older source revisions.
- [ ] Final all-family native/Web/Swift consolidation, exact original throws and package/type exports.
- [ ] Physical platform coverage and release publication remain independent gates. Rosetta is not physical Intel/macOS 14 proof; a local package is not a published release. Historical v0.2 release is documented, not evidence of current full parity.

## Reproduce this audit's actual oracle checks

`node docs/parity/2026-10-21-runtime-audit/oracle-probe.mjs > /tmp/pele-functional-oracle.json`

Probe verifies bundle hash before import. It preserves complete SVG/code units for Info/Radar, exact error name/message, and constructor results for every one of the 33 source-family representatives. All 22 nonplain prototype constructors succeeded. The separate web-entry-report.json.gz compares the actual d7caf package and pinned Pele entry bundle. Other native differences are visible in the recorded source guards and explicit gap assertions; no comprehensive all-input parity claim follows.
