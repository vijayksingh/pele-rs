# Adaptive Gantt property lookup

Gantt's public arena remains the owner of every property, descriptor and object
identity. This change makes small records use that existing contiguous storage
for lookup instead of constructing another hash table for each record during
every operation. Larger records retain the existing randomized hash index.

## Evidence and scope

A native release profile after borrowed string lookup recorded 1,406 samples
inside a 16-render mutable-publication batch. Property-index handling accounts
for 180 inclusive samples (12.8%); `own` has 341, `set` 175, and Context
construction 40. These categories overlap and optimized inlining can hide work.
They identify a candidate, not a predicted speedup. The sampled batch grows from
254 to 1,838 retained nodes; it does not measure fixed-size steady state.

A timer-free graph census found 48 task records with 17 properties and 48 raw
records with three properties in the medium fixture. The root has 30 properties;
the task array has 48. The large fixture has 200 task/raw records and a 200-entry
task array. This motivates a bounded threshold of 32 properties, not a claim
that 32 is optimal for every workload.

The change is confined to operation-local property indexing. Parsing,
scheduling, typed rendering, interning, Map/Set indexes, public graph protocols
and host adapters retain their existing behavior. There is no persistent cache,
second model or new public API.

## Native representation

A private `PropertyIndex` has two variants:

- `Linear` searches canonical string IDs in the current graph property slice.
  It owns no copied keys or positions and performs at most 32 comparisons.
- `Hashed` stores the existing canonical-ID-to-position HashMap for larger
  records, preserving first-occurrence lookup and randomized hashing.

The outer operation-local map records which nodes have been touched.
`property_position` uses its entry API to initialize or borrow the index with
one lookup. It returns only a position, so no property borrow survives a graph
mutation or host callback. Values and descriptors are always read from the
current graph.

Appending the 33rd property promotes a linear record before the property is
pushed. Promotion indexes its existing properties in order, then adds the
already-proven-absent key. Later growth uses the existing hashed path. This
bounds linear work for arbitrary caller-owned records, not only built-in task
shapes. The threshold is private and does not affect public admission limits.

## Semantic invariants

The first access still charges the complete current property count, including
for small or empty records. `own` still charges its initial tick before resolving
a key; a key absent from the string table returns without creating an index.
`set` retains receiver checks, its initial tick, interning, first-touch charge,
readonly checks, and reference admission in that order. Promotion adds no new
logical work charge, matching the previous index update on insertion.

The shared work-charge helper preserves saturating arithmetic and the exact
200,000,000 limit. Failed charges retain their accumulated count and do not
publish an index. Reference admission still occurs before cache insertion and
graph append. No budget is increased or suppressed.

Canonical string IDs preserve duplicate UTF-16 key behavior. Both variants and
promotion select the first matching property, including its descriptor. Values,
missing fields and readonly state are never cached. Raw number bits, Date
identity, aliases, detached nodes and complete retained history remain in the
same graph.

Existing active-model delta handling removes the index for each replaced node.
The next access charges that node again and chooses the appropriate variant,
including after a large record shrinks. Whole-document apply naturally starts
another operation-local Context. Callback ordering and partial mutation/error
publication are unchanged.

## Validation and adoption

`public_gantt_property_scale` exercises the feature through a real host
`getTasks` override: it grows a record to 40 fields, reads and overwrites them,
replaces it through a synchronized delta with duplicate canonical keys and a
readonly NaN field, then grows it again to 42 properties. It verifies exact
number bits, descriptor precedence, live values, revisions and complete rendered
output across two operations. Existing Gantt, host-function, graph fingerprint
and budget integration coverage remains necessary.

Allocation measurements must distinguish temporary indexes from required
retained arena storage. Compare native mutable batches with matched history,
then verify browser and Swift delivery lanes independently. Source and typed
rendering are negative controls. Linear scans can cost more for repeatedly read
late fields, and the enum may change outer-map bucket size; accept only on
measured results, not presumed allocation savings. This document makes no
performance or cross-platform completion claim.
