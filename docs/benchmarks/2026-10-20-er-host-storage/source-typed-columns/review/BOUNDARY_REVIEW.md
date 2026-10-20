# Independent rebase review

GO for a separate layer after the parent decides the e518 benchmark. No source blocker found. This is not runtime or performance approval.

Bound patch b7cfd9d9bf285a8aa0cfcf59a16d9c7f26fedbab175c328d3e8a436b82410c87 and manifest a55d60aa48afb4cd45b8d6509435787ed68e55d141c420decaaf92177c610691. Verified every original against current root and every candidate against the manifest. Seven files. Compared old-original→new-original with old-candidate→new-candidate: upstream added/deleted bytes identical; only source_store has an upstream delta. Apply-check with whitespace errors rejected passes.

The e518 SourceCandidate private text access is retained exactly. Typed output changes only full recipe serialization; it cannot alter prepared overlay ownership, first structural admission, schema ordering, original-versus-replacement text selection, eager import, receipts or atomic commit. The latter paths remain in the unchanged e518 files or outside typed hunks.

Native width multiplication is checked before payload emission; padding and writes remain under the same bounded Writer. Scalar IDs remain u32; Number payloads use raw to_bits and LE words, never transport f64 conversion. IdentityNone emits the same fixed01 atoms from the shared iterator. Update01 complete string/node patches and bounded canonical fallback remain unchanged. Full03's independent physical budget may reject its recipe and use the existing canonical fallback; no canonical admission is weakened.

JS retains one compact payload-only owned buffer, not the incoming full packet or mutable view. Packet-relative zero padding and explicit LE reads handle unaligned incoming byte offsets; host-endian Uint32 views are used only on little-endian hosts. The source-reviewed big-endian DataView branch reads explicit LE. Legacy01/02 triples remain behind the same column access interface. Number low/high words remain intact until the existing semantic conversion points, including NaN bits. Unused truncated base rows still avoid premature semantic-reference validation.

Execution gates remain: native03 compile/public integrations; generator reproducibility against actual03; public browser lifecycle across engines with forced02/canonical fallback, unaligned packets and overwrite-after-decode; ordinary native/Swift checks. Big-endian execution, real identity exhaustion and the new03 padding-at-limit edge remain unexecuted. No timing or allocation claim. No root files changed by this review.
