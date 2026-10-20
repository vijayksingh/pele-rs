# Captured graph admission

Structural and schema admission share an immutable storage reader. The current
adapter borrows the native `ModelGraph`; a future captured host adapter can lend
bounded batches without building another native graph. No host backend is
selected by this change.

`AdmissionRead` exposes table extents, root, string lengths, node headers and
scoped property/Map/Set slices. Native slices borrow their original storage;
they do not clone atoms or fill intermediate record arrays. A host adapter must
check its reply bounds and lend operation-local scratch. The validator owns all
reference, flags, work and schema policy. Storage cannot supply a validation
certificate or skip historical records.

Structural validation preserves this order:

1. Version, then checked summation of every string length.
2. Table and UTF16 limits, then root references.
3. Each node's complete work charge, then its ordered properties, then Map keys
   and values or Set values.

Schema identity validation remains a separate pass after structural success:
root builtin, then each node's prototype, property builtins and collection
builtins. Arbitrary schema validation and membership-set construction stay at
their existing call sites. Lazy registration, provider export/apply and family
projection stages are unchanged.

Validation permits the existing generic graph values: detached history, cycles,
duplicate strings/keys/Map entries, lone UTF16 surrogates, nonfinite number bits,
host-reference IDs and sparse arrays. It does not impose required family fields,
reachability, array-length descriptor consistency or host-value existence.

Public binary decoding and byte limits remain separate gates with their original
ordering. The reader does not authorize bypassing them. Exact binary-equivalent
admission, a host adapter and complete lifecycle integration are still required
before a graphless backend can be enabled. This extraction claims no speedup.

Public integration covers mixed structural/schema faults, rejected adoption and
apply/invoke recovery, revision stability, and byte-preserving historical values
through adoption, apply, snapshot and render. Existing registry, schema, binary,
provider and Sankey integrations exercise the same public wrappers.
