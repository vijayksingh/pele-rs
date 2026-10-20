# Borrowed Gantt string lookups

This private change reuses the existing `JsString::short_utf16`/`Borrow<[u16]>`
primitive in Gantt's `Context::own` and `Context::text`. The identity graph,
interner representation, scheduler, public contracts and adapters stay intact.

## Measured target

After the fresh-construction improvement, a native release profile of a
16-render mutable-publication batch observes 1,813 samples inside `State::once`.
`own` has 625 inclusive samples; UTF-16 construction beneath `own`/`set` has
322. `Context::new` has only 38. These nested categories overlap, and optimized
inlining can hide work. They justify examining temporary string ownership before
introducing durable indexes; they do not predict a percentage speedup.
The [recorded profiles](benchmarks/2026-10-20-gantt-builder/native-mutable-profile.json)
identify exact source/binary hashes. Their build predates the construction
commit but already contains that candidate's source.

## Ownership and ordering

For strings of at most 64 UTF-8 bytes, encode UTF-16 into a 64-unit stack buffer
and query the existing HashMap using a borrowed slice. UTF-8 byte length safely
bounds UTF-16 units. Longer strings retain the original owned conversion.

`own` ticks first. An absent key returns without interning or building an index;
a present key still builds/charges the property index and reads its live value.
Only the temporary owned lookup string disappears.

`text` returns the original canonical ID on a hit. The original `intern` already
returns existing IDs before checking string-count/unit admission limits, so no
semantic charge is skipped. A short miss copies the encoded units into one owned
Vec and follows the original `intern` insertion/duplicate/budget path. It does
not encode the string twice. Direct arbitrary UTF-16 interning is untouched,
including lone surrogates. No cached values, retained misses or new public limits.

## Validation boundary

Complete frozen arena fingerprints cover repeated labels at 64/65-byte ASCII,
accented and emoji boundaries, alongside metadata, aliases and interned key
collisions. Public Gantt methods, lifecycle, callbacks and the historical
model/SVG/error corpus remain the behavioral oracle.

The public string-capacity integration fills a real mutable document's table,
reuses an existing clickable class, rejects a new sanitized link, compares the
whole graph after failure, and verifies recovery, revisions and Date aliases.
This exercises admission through actual public operations, not private helpers.

Separate allocation binaries compare public parse and mutable publication at
matched counts one and sixteen; mandatory retained graph storage must stay equal.
Typed parse/source/retained rendering are negative controls. Extra hashing on
short misses can regress unique-label parsing, so native and VM public parse
remain adoption gates. Native CPU, browser VM and Swift checks are distinct.
