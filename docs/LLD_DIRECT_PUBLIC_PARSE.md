# Direct construction during public Gantt parsing

Ordinary public Gantt parsing now constructs its final native mutable arena while
consuming the shared Rust grammar. It avoids building a complete owned
`Vec<GanttTask>` and then copying those task strings and fields into `ModelGraph`.
The arena remains the canonical mutable state used by existing methods,
scheduling, callbacks, import, export and adapters.

## Shared preparation and grammar

`compat/engine.rs` owns the private `ParsePlan`. It captures the selected
registration, prepared source, effective configuration, title, source line offset
and edge policy. `ParseContext` borrows these fields. Source admission, detection,
explicit selection, comment handling, entity encoding and the final newline use
the same preparation implementation as typed parsing.

The six ordinary `PublicEngine` parse variants consume this plan through
`PublicDiagramModule::try_parse_public`. The additive trait method defaults to
`None`, retaining typed parse followed by `create_public`. Its erased adapter
calls the captured module instance; selecting or replacing an extension does not
redirect it to a builtin of the same name. An opt-in implementation must preserve
parse errors and admission and must not publish state or invoke host code before
returning `None`.

Gantt opts in through its private construction target. Its grammar runs through the private
`parser::Target` interface. Both `GanttModel` and the arena construction target
use the same token traversal and borrowed `TaskDeclaration` validation. The
immutable/source-render path still creates a typed model. Raw prepared parsing,
adaptation with `PublicDocument::from_module`, and imported graphs keep their
existing paths. Gantt's existing exact-edge creation policy is a no-op, so its
new target does not add an edge check.

## Ownership during construction

`DirectParse` owns construction bookkeeping around a borrowed `Context` and a
header-only `GanttModel`. Each task declaration immediately creates its raw
record, classes array and task record. It retains task references, ordered by-ID
entries and a map from interned ID to the latest task/classes handles. It does
not retain an additional typed task collection.

Duplicate task IDs retain all task objects while replacing the by-ID value in
its original insertion position. Click classes affect the latest matching task.
Links are sanitized before checking whether a task exists; repeated links keep
the last value and the first insertion position. Header metadata and final link
values are materialized when construction finishes, avoiding retention of
superseded metadata values. The original root layout is shared with typed
adaptation through `finish_initial_root`.

Successful construction validates the graph against Gantt's schema. The public
engine then validates the captured complete schema view, moves the effective
configuration into the document and publishes revision zero. No host callback
runs during this builtin construction. Subsequent operation/revision/error
handling uses the existing public engine and Gantt runtime.

## Admission and one legacy fallback

Existing `Context` node, string, reference and work checks remain active during
the direct attempt. Fresh array append retains four separate work ticks, the
length check between ticks three and four, numeric-key interning, reference
admission and default property flags. Runtime property access and mutation are
unchanged.

A successful direct target must also remain within this conservative interior:

| Construction quantity | Maximum for direct publication |
| --- | ---: |
| Nodes | 100,000 |
| Interned strings | 100,000 |
| References | 1,000,000 |
| UTF-16 units | 4,000,000 |
| Work | 10,000,000 |

These are eligibility thresholds, not smaller public limits. A construction
storage failure or a failed finishing/eligibility check drops the unpublished
target and returns `None`. The same prepared plan then runs the established
complete typed parser and constructor exactly once. In particular, an early
direct storage failure cannot hide a later syntax error. Ordinary grammar or
semantic errors reached without storage failure return directly. The legacy
fallback decides the actual result near the public resource boundaries.

The fallback is bounded to one extra parse/construction attempt; it is not a
retry loop. It can cost more CPU and transient allocation on large or adversarial
inputs. The failed arena is dropped before the fallback starts. This layer makes
no uniform latency claim for those inputs and does not relax source, graph or
transport limits.

## Identity and compatibility

Fresh direct construction intentionally changes initial node/string numbering
and interning order. IDs are scoped to a document/session, not a cross-version
numbering promise. All aliases, property order and flags, collection order,
undefined/null distinctions and values must remain equivalent. Once a document
is published, existing ID retention and detached-node rules continue unchanged.
Imported graphs are never renumbered by this path.

Typed adaptation retains its original binary layout, including prepopulated
Date identities and IEEE-754 payloads. Direct source parsing initially creates
unscheduled tasks; the unchanged scheduler creates Dates and retained history
on subsequent operations. Existing native, WASM and Swift graph protocols and
host callback synchronization are unchanged. JavaScript receives the usual
binary graph; no parsing or layout algorithm moves into an adapter.

## Validation and measurement scope

`public_gantt_direct_parse` compares direct construction against typed adaptation
using root-directed alias correspondence, exact property/value content and total
retained node count, then repeats apply and scheduling with complete outcomes.
It also compares parser errors. `public_gantt_direct_limits` exercises successful
legacy fallback with byte-identical binary graphs above the eligibility bound,
and an early storage failure followed by a later syntax error. Both are explicit
targets in `scripts/verify.sh`.

Construction fixtures keep separate complete graph hashes for typed adaptation
and direct parsing. These storage checks complement the independent pinned Pele
behavioral corpus; changed fresh IDs alone do not justify changing observable
model or SVG expectations. Existing source/edge limits, registration, host
function and Gantt lifecycle integrations cover the surrounding boundaries.

Native timing and allocation reports must identify their exact binaries and
source inputs. The measured medium workload supports reduced public construction
work; it does not establish a whole-family or whole-adapter speedup. Allocation
requests, peak requested live bytes, RSS and elapsed time are separate measures.
VM and Swift verification remain separate delivery checks. The complete Pele JS
performance gate still governs a future 0.3 release.

## ER construction target

[Direct ER construction](LLD_DIRECT_ER_PARSE.md) uses this same captured plan
and optional hook. One shared grammar targets either the original typed model
or the final canonical arena. A conservative declaration-first subset avoids a
complete temporary model and conversion pass; remaining syntax and larger input
use the existing typed path once, preserving all established limits and errors.
Public graph bytes, eager publication and existing render/import behavior stay
identical. The separate resource-callback parity failure remains open.
