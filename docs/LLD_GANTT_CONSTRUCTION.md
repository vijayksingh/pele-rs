# Native Gantt graph construction

The typed Rust parser already returns `GanttModel`. Mutable public parsing then
constructs its identity graph. This change replaces general JavaScript property
assignment during that fresh construction with private Rust builder operations.
Runtime mutation, scheduling, callbacks, graph apply and publication are unchanged.

## Evidence

A release native sample of the 48-task public parse loop placed 2,327 samples
under `create_public` out of 3,588 in the measured `State::once` branch. Major
`Context::record` branches had 1,614 and 209 inclusive samples, with field
assignment, UTF-16 allocation and HashMap work beneath them. These counts overlap
their ancestors; they are not additive percentages or a predicted speedup.
The completed eight-second parse profile preceded an unrelated Swift build
that interrupted the subsequent mutable-render profile. Only the completed
parse profile supports this decision.

Fresh arrays previously used general `push`, including a scan of every existing
property after each append to update an explicit length descriptor. Construction
arrays have implicit length and numeric fields only. That scan made dense-array
construction quadratic. Fresh small records also built temporary property maps.

## Private primitives

`InitialObject` owns an integer node ID and is consumed when its fields are
finished. Creating the root early and finishing it after its children preserves
node allocation order. Field insertion uses a bounded linear search of the small
fixed field list, preserves first-position duplicate assignment, and reserves its
property capacity once. It does not allocate a property HashMap.

`initial_array` appends default numeric properties and updates structural length
directly. It performs no lookup for caller overrides or explicit descriptors:
only this construction path owns those fresh arrays. Runtime `record`, `array`,
`set` and `push` retain all existing checks. No cache, unsafe code, dependency,
new public API, alternate model or scheduler is introduced.

## Compatibility and resource accounting

Node creation, child creation, string interning and field insertion retain their
existing order. Task/byId aliases, separate Date identities, unused arena entries,
undefined/null distinctions, collection ordering and number bits are preserved.
No field-name table is pre-interned: a value may establish a string's first ID.

Each fresh field still ticks once, interns its key, then reserves a reference
before insertion. A fresh array append replays four separate work ticks, checks
the length limit between ticks three and four, then interns the numeric key and
reserves its reference. Node-count checks still run for each property, including
the last allowed empty node. Only the first empty property-index charge of zero
is omitted; a preceding positive tick has already passed the same threshold.
All existing limits, error precedence and final schema validation remain.

## Validation

`gantt-construction.json` pins the complete binary graph produced by the frozen
pre-change Rust/WASM package. Native and browser consumers compare every byte's
hash, including all node/string IDs, order and unused entries. These are storage
regressions; the pinned Pele method/model/SVG/error corpus independently remains
the behavioral oracle. Cases cover empty models, 8/48/200 tasks, metadata,
duplicate IDs/sections, classes/links, dates, Unicode and values colliding with
property names and numeric keys. The public lifecycle, host-function and binary
integration slices check the unchanged runtime path.

Performance evidence must use frozen independent binaries and exact output
checks. Typed parse and retained render are negative controls. Allocation counts
are separate from CPU timings; VM browser and local Swift verification are
separate delivery lanes. This construction change does not eliminate mutable
arena copying, reconciliation or required retained Date history. The 0.3 release
gate continues to require matching Pele JS across the complete web matrix.
