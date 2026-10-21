# ER host storage

Status: unreleased architecture under integration and measurement. Performance
parity is required before 0.3. A smaller representation is not proof of speed.

## Ownership

Ordinary browser builtin ER parsing creates the actual public JavaScript
objects through a Rust construction sink. Native and Swift retain their existing
owned document contracts. Exported low-level MutableDocument, revision, binary,
prepared and caller-authored model contracts remain unchanged.

Facade prepared/standalone ER parsing also publishes host storage. Its hidden
capability is additive; the exported low-level prepared parser still returns
the same arena document and binary/revision APIs.

Caller-authored raw ER adoption selects the exact builtin codec before traversal
and retains its captured registration. It audits the original objects into the
identity/string accounting ledger, binds policy/config without a native graph,
and commits ownership only after both admission gates succeed. The initial
zero-metadata binary-size gate precedes config serialization; the captured
snapshot-size gate adds the former 13-byte type metadata after binding. Binding
rechecks the originating registry is open. Replacement cannot reselect policy.
Non-ER/custom codecs and exported binary APIs retain their existing paths.

Imported caller objects receive no direct helper-owner mapping: successful
last-live helper routing remains intact. Failed provisional owners are discarded.
Config serialization writes stay live instead of being overwritten by an older
snapshot. Those two corrections have explicit public integration coverage;
object identity, callback reads, arbitrary throws and nested operations use the
same host operation contract as parsed models.

The browser policy retains captured registration/config, not a JavaScript model
or a persistent native graph. The JavaScript owner holds public objects, identity
maps, string/host-reference history and scalar byte/reference accounting per
retained node. It stores no property mirror, decoded graph or typed ErModel.

Core defines three storage interfaces: construction, method operations and live
render reads. Grammar, helpers, validation, topology, layout, text and SVG remain
Rust. The adapter implements object identity, descriptors, collection operations,
primitive conversion, resources and lifecycle. This seam can support further
modules without making the core depend on a particular application or JS values.

## Eager construction

One captured ParsePlan performs preparation/provider selection. The direct ER
construction target shares the existing grammar. If conservative direct syntax
cannot represent a source, its unpublished owner is discarded, that same plan
runs the legacy grammar once, and the shared publisher writes a fresh host owner.
The temporary typed model drops before parse returns. Counting-only admission
preserves legacy graph/transport limits before fallback publication; it retains
unique strings and scalar counts, not completed nodes or another model.

Prepared input uses a separate borrowed PreparedParsePlan. It preserves source,
title, offset and edge-limit presence verbatim, captures provider/config before
parsing, and executes the existing prepared grammar once. Schema and counting
admission follow grammar; the host factory runs only after both succeed. The
temporary typed model drops before return. Exact native ER type identity opts
in; custom codecs keep their captured prepared conversion. No ordinary source
preprocessing or direct-source hook is substituted.

The constructor assigns fresh canonical IDs locally and streams at most 128
ordered storage events / 64 KiB per transfer. Large UTF16 strings use bounded
fragments. Nodes that fit use one inline event; larger nodes retain ordered
begin/payload/property/end events. Both encodings use the same publication and
accounting rules. Each completed event writes the final public object; pending state is
one object being completed or fragments of one string. Rust keeps an ephemeral
interner and bounded transfer buffer, then drops both. No first-use conversion
or deferred native upgrade is allowed.

Fresh publication accumulates exact per-node byte/reference totals. Duplicate
properties and Map/Set entries use actual object operations for accounting.
Array length rewrites/truncation retain full-audit fallback. Final admission
includes every string/node, descriptors, root and metadata; it does not need to
reread ordinary fresh records. Partial failure discards pending publication.

## Operations and resources

Every operation audits current own data and retained history at entry and exit.
Detached objects remain roots. Objects reclassified as host references retain
their previous node storage charge. No full graph scan occurs at each width or
icon callback. Successful/error outcomes retain caller mutations; cleanup never
restores stale values. Original arbitrary callback throws take priority over
secondary admission failures. Repair retries audit the current caller objects.

A HostStore exists only for one invocation. It owns its transfer context and
bounded optional caches for immutable string units and property-key strings.
A full cache skips caching; it does not reject a previously valid model. Work
and text budgets still charge every use. No mutable field or conversion-result
cache exists. Rust interior borrows end before host imports.

Typed own/index/kind/UTF16 imports avoid generic command arrays for frequent
reads. Packed scalar replies carry canonical IDs and descriptor flags exactly;
numbers use an operation-local immediate raw-bit slot. Generic method mutation
commands remain separate. Native host errors pass through the same captured
exception path, including browser-specific primitive conversion behavior.
Map membership transfers use at most 128 pairs / 2,309 bytes and preserve the
previous first-excess-pair failure boundary. Primitive string conversion stays
native; object, function and opaque-value conversion still enters the host.
Admission array counts use a direct scalar import while retaining the original
descriptor enumeration and empty-key classification, errors and temporary IDs.

Same-model nested helpers, render and raw render may enter while a resource is
suspended. Each invocation has independent geometry, read context and numeric
slot. Temporary observed handles survive until the outer operation ends, even
when nested admission promotes the underlying object. Active disposal remains
rejected. The policy contains no permanent JS root and no core Send weakening.

## Operation-local entry reads

Outermost normal host ER renders can encode the own-data observations already
made by entry audit into one optional flat read frame. Native admission and safe
initial reads use bounded byte offsets rather than repeating per-field host
imports. This is temporary storage for one operation, with no property mirror
or retained imported model. UTF16 validation and per-occurrence work charges
remain at their original native read positions.

The native frame plus node/string offsets is capped at 8 MiB. A full or invalid
optional frame falls back to the existing live reader, with no new public Limit.
JS staging and native copies may coexist; this logical cap is not process peak.
All callbacks/resources and generic user conversion boundaries drop the frame
before entering host code, and it is never restored. Borrowed frame accesses
return owned values before an import. Captured row/style handles retain their
existing lifetime; later reads remain live.

Raw rendering remains live because config serialization can mutate the model
after entry audit. Nested rendering remains live to preserve temporary canonical
IDs across nested admission. Both stay supported. Methods, source rendering,
construction and old JSON/packet exports keep their existing paths. Universal
Proxy/intrinsic observation equivalence remains outside the documented model
contract; the initial raw Proxy rejection controls are unchanged.

## Successful result transport

Additive browser-only render endpoints transfer successful SVG, dimension bits
and links directly. The same serde serializer counts the complete legacy JSON
Result envelope with its original 64 MiB write cutoff, including numeric
sidecars. Final structural audit still precedes counting; final graph-plus-
metadata admission still follows the response. Errors and old exported JSON
methods retain their existing transport. Native and Swift are unchanged.

One host factory reconstructs dimensions from four u32 halves and creates the
same public key order. The facade uses the exact counted metadata length rather
than JSON-parsing and UTF8-encoding the complete SVG again. It retains old-WASM
feature fallback. No model read, callback phase or renderer algorithm changes.
Counting still scans and escapes output; fewer buffers are not a speed claim.

Differential browser checks cover ordinary/prepared/raw/caller roots, exact
metadata lengths, public shape, computed numeric bits, Unicode, nested callbacks,
original throws, audit precedence and repair. Bounded accessibility strings
exercise both the standalone envelope ceiling and a successful native response
rejected by combined graph-plus-metadata admission. Nonempty ER links remain
unreachable through the current renderer and are not claimed as tested here.

## Live read schedule

- Select the horizontal wrapper before the first drawing callback.
- Capture each pass's topology and scalar values. Retain attribute/style array
  handles; replacing a property does not retarget a captured array.
- Measure edge labels first, then read retained node styles at each node phase.
- Read attribute length/index live. Retain one row handle, then read and measure
  type, name, joined keys and comment in order. Never prefetch later cells.
- Read direction at its layout use, title after layout, and accessibility after
  body/icon evaluation. Preserve conversion/error order and original throws.
- A second pass captures again. Mutations persist even if that SVG loses.

Caller primitive conversions can themselves run code. Read batching or caching
must not cross an unproven conversion/resource boundary. The typed/native path
uses this same drawing algorithm with borrowed arrays and strings, preserving
its existing presentation-capture phase. Ordinary browser host rendering exposes
no new presentation API. Existing explicit unsupported prototype/accessor and
intrinsic-mutation policies remain in force.

## Verification and release

Public integration/E2E checks cover exact complete source/model/error output,
constructor/fallback/cap boundaries, retained identities, all helpers, callbacks,
nested operations, arbitrary throws, repair, Unicode transfer boundaries and
provider ownership. Native/Swift negative controls are required because shared
publication/rendering code changes too. Exported low-level and prepared/raw
lanes require their own proof; ordinary host-lane results do not establish them.

Initial independent VM experiments exposed substantial regressions: the first
host backend took 4.19 times the old backend's ER-medium parse time, and per-node
batching still took 2.32 times. Profiles identified repeated boundary string
conversion and generic array transfers. Streaming and typed reads must be
measured, including cold parse, first unchanged/edited/callback render, helpers,
retained/raw paths and source controls. Source-derived crossing counts are not
speed measurements. Keep adverse samples and exact artifacts; do not move costs
into first use or substitute a focused run for the complete JS performance gate.

No full API parity, performance parity or release-readiness claim follows from
this architecture document.

The streamed backend passed 339 browser tests across Chromium, Firefox and
WebKit. The subsequent inline-node/typed-Map layer passed 30 focused browser
checks, including both 7,000-entry oversized-node encodings. Current native
ER publication checks, 168 Swift API conditions, eight Swift integration tests
and three C integration consumers pass. Native public-parse allocation requests,
reallocations and bytes match the preceding token-ownership baseline; borrowed
render arrays remove 668 allocation requests from each measured ER render.

The inline checkpoint's focused independent VM run is still adverse: ER-medium parse takes
0.715 ms versus 0.722 ms for the old backend and 0.110 ms for JS; retained render
takes 3.144 ms versus 1.771 ms and 0.486 ms respectively. These are nine-trial
fixed-resource diagnostic medians, not the complete release gate. The latest
complete gate remains the previous 126-condition failure. The broader
30-condition comparison reports 75.7–91.9% slower paired retained ER render
medians against the old backend, with mixed parse measurements.

Prepared publication now passes the six previously failing comparisons in each
browser. Two focused runs cover ER callbacks/construction plus all 33 packed
entries, normal-render policy, provider/lifecycle and low-level error controls.
Twelve native integrations, 168 Swift API conditions, eight ER/eager Swift tests,
seven all-family prepared/caller-model Swift integrations and three C consumers
pass against the fresh ARM64 SDK. Caller-authored raw ER adoption now keeps
host objects directly. Forty-five focused browser checks pass across all three
engines, including the six previously failing callback comparisons, ten exotic
Ref-root controls, config/error ordering and lifecycle checks. Fourteen native
integrations pass. The consolidated 369-test browser suite passes; fresh ARM64
SDK checks pass 168 Swift API conditions, 15 Swift integrations and three C
consumers for this raw layer. The exact near-64-MiB zero-versus-13-byte gate has source review
but no new large E2E fixture; it remains an explicit validation gap.
No full API or performance parity claim; no 0.3 release.
