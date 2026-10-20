# Sankey methods over captured storage

Sankey's two existing helper algorithms use the private `CapturedWrite`
interface. `invoke_graph` creates an `ArenaWrite` adapter over the current
`ModelGraph`; every public entry still selects arena storage.

The adapter exposes scalar/positional storage operations, counts, first-content
string lookup, first-slot SameValueZero Map lookup and batched descriptor
queries. It returns owned scalar atoms so no reference crosses mutation.
Projection continues using the separate borrowed `ArenaRead` after the helper.
No mutable `Node` reference escapes the storage boundary.

Rust retains method policy: receiver errors, collection override rejection,
readonly checks, exact prospective budgets, field order and partial mutations.
`findOrCreateNode` still inserts its record into lookup before checking the later
array push. A failed push retains that record and Map entry. `addLink` keeps its
original check-before-record order. The public hook refreshes the eager render
projection after both helper success and failure.

Descriptor queries return only the first matching property slot and whether
any matching descriptor is nonwritable. They inspect every duplicate. Push
updates the first index value, then scalar array length, then every own length
property, retaining flags and order. Its cached index slot remains valid because
intervening work never mutates that target's properties or calls user code.

Map replacement updates only the first equivalent slot and preserves existing
key bits. Only appended zero keys normalize to positive zero; the node's `id`
keeps the argument's exact number bits. String equality compares UTF16 content,
and interning chooses the first matching ID even in duplicate string tables.
Budget checks retain repeated missing-key charges and the existing push work
charge even when an index property is replaced.

Public integration tests exercise duplicate Map slots, NaN/zero payloads,
first string IDs, all-match readonly failures, retained partial state, repair,
all length-property writes and first-only index replacement. Those cases pass
against the preceding arena implementation before the extraction and against
the new adapter, alongside the existing helper/SVG/alias/error corpus.

This layer adds no host selection, model conversion, transaction rollback,
callback capability or performance claim. The complete browser backend must
implement this interface against private staged columns before it can replace
the native arena and initial graph transport.
