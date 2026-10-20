# Raw admission frame boundary and Proxy-stage coverage

Reviewed frozen raw candidate `28d50a787bc811fea177d3dfceaf67b028b404f4d5b760b4d7dc309f1951bfca`.
Source-only audit. No production edits, browser execution, builds, or large graph
allocation. Parent explicitly accepted disclosure of the exact-size runtime gap.

## Exact boundary

Let M = 64 * 1024 * 1024 = 67,108,864 and G be the captured graph byte count,
excluding metadata. The old initial input encoding uses empty metadata; the
old adopted snapshot adds UTF8 `{"type":"er"}`, exactly 13 bytes.

- G <= 67,108,851: both frame checks pass.
- G = 67,108,851 is the last passing +13 case: equality with M is allowed.
- G in [67,108,852, 67,108,864]: the empty-metadata check passes, +13 fails.
- G >= 67,108,865: the first check fails before config serialization.

There are thirteen distinct integer byte sizes in the distinguishing interval.
A claim that G = M-13 should fail would be an off-by-one error.

Both implementations use the same size expression:

    G = 32 + atomBytes(root) + 4*S + 2*U + sum(nodeBytes)

S is canonical string-table count; U is total UTF16 units, including interned
property keys. Node sizes include descriptors, payload, kind, and collection
headers. The new owner captures counts once before config. full.js checks
`callerFrameAdmission(admission)` before the first finiteConfigJSON, binds the
native policy (including the registry-open/config decode checks), validates
structural counts, and checks `frameAdmission(admission,13)` before committing
ownership. It intentionally reuses the first admission even if config code
changes the caller graph. Re-auditing a reduced graph for the +13 gate would be
an observable stage change.

A future exact-size integration experiment should record config-call count and
width-call count, not just the eventual error:

- first-gate rejection: zero config calls, zero width calls;
- distinguishing-interval rejection with valid config: one config call, zero
  width calls, exact transport-budget error, no committed ownership;
- first config throws or produces invalid configuration: that error precedes
  the later +13 gate, with the original throw identity retained;
- config can clear the large payload before returning; the captured +13 check
  must still reject. This distinguishes captured sizing from recomputation.

The threshold control G=M-13 cannot merely render an unchanged near-limit graph:
later render-result metadata is larger than 13 bytes and may independently
reject. Its first config callback should clear the large collection/padding so
subsequent operation auditing uses a small live model, while initial adoption
still consumes the captured size. This requires deliberate stage expectations,
not a blanket render-success assertion on a giant model.

## Why no small safe public fixture was added

For ordinary supported records, a conservative encoded-size upper bound is:

    G <= 37 + 4*S + 2*U + 13*N + 14*R

N is node count and R is property/collection reference-work count. A Date's
13-byte base dominates other node bases; each property costs at most14 bytes;
collection atoms cost at most9 bytes per work item. Existing caps are S,N <=1M,
U <=16M and R <=8M. Even allowing the entire string budget, N=10,000 and
R=100,000 can reach at most37,530,037 bytes, below the distinguishing interval.
Even simultaneously taking the maximum million-node and million-string tables
still requires at least1,293,487 reference records by this generous bound.
Those maxima themselves already describe a large fixture.

A concrete ordinary-data construction could use roughly two million numeric
Map pairs (36M encoded bytes) plus about15.55M UTF16 padding units and a small
valid ER root. Pair count/padding could be solved analytically from its exact
base size, with an additional tiny atom to handle parity. But this means a
multi-million-entry live Map and large string, followed in the legacy path by
millions of atom records, the ~64MiB frame, WASM transfer, native decode, and a
second ~64MiB output frame. Actual peak memory/time is engine-dependent; this
is not a prudent default feature slice. No such allocation was attempted.

Sparse array length does not inflate encoded property bytes. Repeated aliases
are counted once as nodes. Repeated strings reuse canonical IDs. Private-owner
counter mutation, lower test-only limits, synthetic graph-wire input, or patched
intrinsics would not prove the requested public production boundary. Proxy
synthetic-property stress could lower retained JS source objects but still
requires millions of descriptor visits and expensive legacy records; it also
adds the unrelated observable reflection contract. It is not used as a shortcut.

Result: the size equation, strict comparison, ordering, and captured-count reuse
have direct source evidence; exact near-64MiB public execution remains unproven.
No runtime proof is claimed for that boundary.

## Proxy encoding-stage controls

The existing raw-order test already contains:

- Proxy Array with length0.5, returned during snapshot traversal;
- Proxy Date whose native Date.getTime receiver check throws;
- Proxy Error whose second brand read becomes undefined after ownKeys.

It compares frozen legacy routing with host routing, including exact error
objects and recorded stage traces, and writes the artifact before assertions.
Its original assertions only required both routes to agree. They could both
move to an incorrect later failure and still agree.

`proxy-stage-assertions.patch` adds positive requirements without another test:
Array and Error must fail with exact `PeleError`, semantic code, and
`Invalid public binary graph transport`; traces must be ['length'] and [].
Date must fail with native TypeError and []; the existing differential comparison
continues checking each engine's exact native diagnostic, avoiding a fabricated
cross-engine message. These empty/no-config traces distinguish the pre-config
encoding/snapshot stages from later importer failures.

The Error trap uses an intentional prototype-read count to target the second
brand read. The positive exact error/message assertion makes drift visible;
the test does not claim arbitrary Proxy behavior is universally supported.

Parent reports current Chromium/Firefox outcomes match those expected stages,
and later reports the focused three-engine slice passing. Those are parent-run
results, not independently rerun here. The fragment's syntax check passed.
Patch SHA256: `39d2cb2b140faedb83c44b828977c4d7c556d69481a4b6537db4a4caf43fdead`.
