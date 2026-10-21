# Borrowed sparse ingress allocation boundary

The first private eager decoder was rejected before integration. Its physical-only reader could allocate a Vec for tens of millions of one-byte atoms before virtual work admission. Original draft and compile logs remain in `archive-eager-decoder/`.

The replacement physical decoder stores only the borrowed input slices, counts, root atom and layout/revision metadata. Its complete scan uses the same `Reader::node_with` wire codec as the ordinary decoder, with a Header-only sink. No table-sized directory, HashMap, UTF16 vector, Property vector or collection vector exists during this scan.

`SourceCanonical::prepare_overlay` merges ordered replacement iterators with ORIGINAL source records. First it counts the exact complete canonical zero-metadata frame (64 MiB), using raw canonical node-body lengths for replacements and the existing source-derived scalar fields for base records. Second it applies shared `DecoderBudget`: version, <=1,000,000 strings, individual and total <=16,000,000 UTF16 units, <=1,000,000 nodes, <=4,000,000 Map pairs per node and <=8,000,000 total Map atoms/Set atoms/properties. These are the existing decoder constants and order, not new limits.

Only after both passes succeed does materialization reserve replacement maps and vectors. Sorted unique in-extent IDs imply replacement map cardinalities cannot exceed the admitted logical table counts. Replacement string units cannot exceed admitted total units; their collection/property work cannot exceed admitted full graph work. This bounds expansion even for one-byte Undefined atoms. HashMap overhead remains bounded but is not claimed byte-identical to the legacy arena Vec representation. No allocation measurement or peak-memory improvement is claimed.

The ordinary binary reader still performs each count/work gate before its materializing sink reserves the corresponding collection. Canonical input/FFI behavior remains unchanged. Malformed sparse envelope tags/truncation are physical protocol failures; no legacy malformed-binary equivalence is claimed for this separate internal format.

After materialization, existing full virtual admission, structural/schema checks, stale revision policy, eager imported Result and atomic graph/projection/revision commit remain. Rejected input leaves accepted state untouched. Only successful commit invalidates source receipts and clears previous sparse helper patches.

## Public proof scope

`public_source_ingress` compares existing public canonical decode/apply and the new public hidden ingress over real parsed Sankey documents. One feature constructs compact byte payloads directly for an 8,000,001-element Set, two individually permitted Sets exceeding aggregate work, an oversized Map, and over-limit node/string tables. It combines bad root and stale revision, verifies the old Limit error wins, verifies unchanged snapshot/revision, then repairs through valid ingress. It never creates a giant test-side ModelGraph/Vec<Atom>.

A separate physical >64 MiB buffer case proves only the physical byte gate. The browser feature proves invalid observed Array scalar fails before native dispatch, full restoration, history repair and forced legacy comparison. It does not prove the much larger public JS graph needed for canonical >64 MiB combined with invalid Array/Error: shared capture's 16,000,000-unit gate prevents using one huge string as a cheap substitute. That heavy mixed gate remains source-reviewed, not runtime-proven here.
