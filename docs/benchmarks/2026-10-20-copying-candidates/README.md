# Copying candidates: rejected experiments

These focused, isolated high-resolution VM runs diagnose candidates. They do
not replace the complete 126-condition release gate. The production direct
comparison report remains [the current published performance observation](../2026-10-20-direct-comparison/README.md).

The inline coordinate formatter replaces most temporary coordinate Strings
with stack bytes and retains an owned large/nonfinite fallback. It passed 15
browser checks across three engines and the 31-case rounding-boundary native
integration. Against the independently packed rounding-fixed control, its 18
Canvas source/retained conditions were mixed: six retained conditions regressed
by more than 5%. The formatter was rejected and removed from tracked source.
The separate Math.round tie correction remains in `d29798a`.

The indexed UTF-16 interning variant avoids owned key clones using secret-keyed
hashes, exact collision comparison and graph string IDs. The default variant
was slower on most measured parse conditions. Its identity-secondary-hasher
variant also failed to demonstrate a reliable CPU win: the unchanged Sequence
parse control regressed about 20%, while Railroad retained rendering regressed
about 14% against the later control. Native collision diagnostics and public
integration checks passed, but the implementation remains rejected. Raw early
and later controls are retained here so drift is visible. These results do not
establish a causal slowdown for every row or a general allocation benefit.

## Go reference inspection

Inspected [tuios sketch at 5d290099](https://github.com/Gaurav-Gosain/tuios/tree/5d29009957907e933b563498c44c2aafd2e0e12d/sketch).
Its README targets the same Pele 458322a5 oracle and lists eight diagram
families. Source inspection found no sketch performance benchmark establishing
the reported 2x claim. No Go speed claim is accepted as measured evidence here.

Concrete candidates identified from its source:

- Lexer tokens retain source substrings until native model construction.
- SVG escaping returns unchanged strings directly when escaping is unnecessary.
- Rectangle-radius rewriting scans literal tags instead of allocating regex
  replacements; its ASCII word boundary follows the pinned JS expression.

Its Parse/RenderSVG API retains opaque native models. That path does not include
our ordinary JS mutable model graph exchange and host publication. Native Go,
native Rust/Swift and browser WASM require distinct matched measurements.

Current direct-comparator CPU samples additionally show that immutable schema
validation is worth reusing for small Sequence models. Snapshot-inclusive
percentages include validation; the owned clone alone is only about 0.4–0.8%
of the three sampled retained workloads. The proposed borrowed-outcome API is
therefore parked until its benefit justifies expanding the public interface.
