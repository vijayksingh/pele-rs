# Eager immutable Swift projection

`Pele.Engine.parse` keeps its existing eager, throwing contract and returns the
same `Document` with public immutable `type` and `model` values. Its transport
uses an additive synchronous C event interface instead of decoding the complete
model envelope through Foundation. Mutable documents, providers and prepared
entry points retain their existing transports.

## Admission and normalization

Both `pele_parse_v2` and `pele_parse_eager_v1` call the same private input parser.
They retain the 16 MiB source and 1 MiB options byte ceilings, UTF-8 validation,
option decoding and captured typed document. The new entry calls public `model_view()` once, preserving module projection
validation, normalized values and depth/byte admission. The captured Gantt
builtin uses [direct normalized projection](LLD_NORMALIZED_PROJECTION.md),
removing its first model JSON buffer and reparse. Other builtin registrations
and arbitrary public providers retain their original bounded JSON normalization.

A borrowed projection has the same fields as the existing document envelope.
The generic `success_envelope_len` counts the exact existing serializer without
retaining its output buffer. The original `document_envelope` is unchanged.
The Swift adapter retains the existing 64 MiB result admission and error class.
An oversized envelope is rejected before returning a document, with no deferred
model access or retained native projection tree.

## Event and ownership interface

The C header defines `PeleProjectionEventV1` and a callback taking at most 128
events. Tags describe null, Boolean, number, UTF-8 string, array/object opening,
object key and container closing. Opening events declare child counts. Array
order and the native object's JavaScript numeric-key ordering are preserved.
String pointers and event storage are borrowed only during the synchronous
callback; consumers must copy anything they retain and must not unwind through C.

Rust traverses the admitted normalized value with stack-resident event batches.
It drops that value before returning. The result retains the original typed
document and small `{type, projectionBytes}` metadata. Retained render and query
operations use the typed document, independently of the metadata bytes.

Swift builds `ConfigValue` eagerly with parse-local reference-owned container
frames, reserved capacities and copied strings. This avoids repeated copying of
partially built value containers and recursive Foundation type probes. Callback
errors are captured locally, abort native traversal and are thrown after the C
call returns. Failed handles are freed; successful handles remain owned by the
existing `Document` lifecycle.

## Compatibility details

- Normalized nonfinite numbers become null. Negative zero becomes positive zero,
  matching the immutable JSON projection; mutable graph numbers keep their
  independent IEEE-bit contract.
- The existing native normalization resolves exact duplicate object keys. Swift
  then preserves Foundation's first value and original UTF-16 key spelling for
  distinct keys that Swift considers canonically equivalent.
- Strings include embedded NUL, leading U+FEFF and non-BMP characters. Swift uses
  the standard UTF-8 decoder and checks exact UTF-8 round-trip equality, retaining
  strict validation on macOS 14. Foundation's encoding constructor would consume
  a leading byte-order mark and lose model content. Lengths count UTF-8 bytes;
  no NUL termination or Foundation `Any`/`NSNumber` conversion is involved.
- `Document.model` remains eager and nonthrowing to read. `decodeModel`, detached
  model ownership, parse error timing and retained operations retain their
  public contracts.
- Existing C symbols, structures and version-2 JSON envelopes are unchanged.
  The new entry and event structure are additive; the ABI version stays 2.

`EagerProjectionIntegrationTests` compares the public Swift result with the
existing C JSON entry: exact number bits, key spelling, nested containers and
batch boundaries, eager syntax/depth/budget failures, config, retained render and
query, detached model lifetime, callback abort and metadata byte counts.
Existing all-family and provider/mutable integration coverage remains required.

## Measurement and release boundary

The motivation is a sampled old Swift consumer: Foundation decoding occupies
90.05% of Gantt-medium and 87.43% of Sequence-small exercise samples. Inclusive
categories overlap; sampling percentages are not predicted latency reductions.
Matched control/candidate binaries, output verification and a compiler/test
quiet guard are required for timing claims. This handoff does not establish web
performance parity and cannot authorize a 0.3 release.
