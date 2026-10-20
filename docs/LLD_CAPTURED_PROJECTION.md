# Sankey projection from captured storage

Sankey's eager render projection uses the private `CapturedRead` interface.
The existing `imported(&ModelGraph)` entry wraps `ArenaRead`; all current callers
still use native arena storage. No browser backend is enabled here.

## Storage boundary

The interface exposes scalar node headers, first-match ordered own-field
queries, bounded property prefixes and immutable UTF16 chunks. It exposes no
JavaScript values, `Node` vectors or alternate persistent graph.

Its associated `Value: Borrow<Atom> + Clone` lets the arena return `&Atom` tied
to the immutable graph's lifetime. Membership maps and ordered vectors retain
their previous pointer-sized values. A future host adapter can return owned
scalar `Atom` values, independent of temporary transfer buffers. Neither form
allocates a wrapper per atom.

Own-field queries batch names in one traversal and stop once every requested
field has its first match. Property batches contain at most 128 entries and ten
UTF16 key units per entry; Rust still recognizes canonical array indexes.
Returned counts must fit both output capacity and the remaining property count,
and the cursor must advance. The importer rejects invalid internal batches.

String visitation borrows at most 1,024 UTF16 units per call. The decoder carries
a leading surrogate across chunks, rejects unpaired surrogates, and checks the
complete unit count against the string length. It creates only the required
render-label String, without an intermediate UTF16 copy.

## Preserved projection contract

The importer validates complete `nodes` membership and then complete `links`
membership before interpreting any node label. Holes, duplicate keys, endpoint
alias/index rules, nonnumeric values and late metadata retain their prior order
and errors. Valid strings are fully converted before the exact existing 128 MiB
label-expansion charge. Optional metadata remains outside that charge.

Accepted edits and successful or partially failing helpers still refresh the
projection eagerly. Projection errors stay cached in accepted state until render
or repair. The canonical owned-parser normalization and all helper algorithms
remain separate from this read interface.

## Verification scope

Public feature tests exercise edited and caller-authored raw graphs, exact SVG,
detached aliases, sparse arrays, malformed endpoints, partial helpers, lossless
extra data, Unicode pairs across chunk boundaries, lone surrogates, error
precedence, the existing expansion limit and repair. The frozen 11,616-case
construction digest remains useful, but ordinary owned parsing bypasses this
importer and cannot establish edited/raw projection correctness alone.

This is a representation boundary for the complete host-storage migration.
The arena, initial snapshot and decode still exist; no additional performance
gain follows merely from introducing this interface. Host selection requires
the full lifecycle and separate matched VM evidence.
