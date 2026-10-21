# Fixed Gantt construction fields

Gantt public parsing builds a typed model and then its identity-preserving
public arena. Fresh raw/task records previously accumulated fields in temporary
growable vectors before moving them into separately allocated canonical property
storage. Their field counts are fixed: three raw fields and seventeen task
fields. Root construction also knows its thirty-field count.

## Representation

The construction-only record helpers consume an exact-size owning iterator of
static field labels and `Atom` values. Raw/task records pass fixed arrays. Root
construction retains its vector, reserves the declared field count once, and
passes its owning iterator. Empty options pass an empty array iterator.

Field values are evaluated before passing the iterator. This matters: lazily
creating child nodes while consuming fields would change arena and string IDs.
The helper still reserves canonical property storage once from the exact count.
It retains duplicate-key assignment handling; arbitrary runtime records do not
use this construction helper.

The fixed arrays use bounded stack storage per task, independent of task count.
They remove transient heap storage, not required canonical graph storage.
Compiler-generated copies or code layout can affect latency, so allocation
savings alone do not establish a speedup.

## Compatibility invariants

Raw data/start/end values are interned before the raw node and its keys. Task
id/task/section values follow, then start/end/render-end Dates in that order,
then classes and previous-task identity. The task node and its seventeen keys
are created last. Missing start/end remain `Undefined`; missing render-end
remains `Null`. No Date is reused or cloned.

The root node is allocated at its original point before evaluating its fields.
Only scratch capacity reservation moves earlier; it performs no public work,
admission, string or node operation. Its options, sets and excluded-days array
retain their previous evaluation order.

Per-field work charges, string interning, reference admission, duplicate
handling, descriptors and mutation tracking are unchanged. Runtime property
lookup, host synchronization, schema validation, wire encoding and adapters are
outside this change.

## Verification

Existing public construction fixtures freeze complete arena serialization,
including detached nodes, string/node IDs and floating-point bits, rather than
checking SVG alone. Public Gantt behavior, callback, budget and property-growth
integrations cover subsequent use. Browser engines and Swift must consume fresh
builds of the changed Rust source.

Compare independently frozen native consumers for public parse and unchanged
typed parse/source/render paths. Count allocations separately from timing;
report requested bytes, reallocations and live bytes distinctly. Web performance
requires the independent VM, and Swift requires its own local proof. No release
or performance claim follows from this architecture document alone.
