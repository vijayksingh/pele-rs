# Borrowed SVG root assembly

Implemented and validated through native, browser and Swift public interfaces.
The [checkpoint](benchmarks/2026-10-20-svg-root/) records controlled allocation
measurements, VM samples, the failed release gate and local timing limitations.

## Scope and ownership

Replace only private root assembly in `crates/pele/src/compat/svg/mod.rs`. The original is the current source including Cow escaping and corrected numeric rounding (source commit `31b4a04`). Keep the public API, diagram renderers, SVG coordinate formatting, entity decoding, font validation, and native theme stages unchanged.

`RootContent<'a>` borrows the renderer's completed `&'a str` body. It owns four alias flags, transformed byte length, and an edit flag. `RoundedRects<'a>` holds that same borrow and a cursor; each yielded `RadiusEdit` contains two byte offsets and a static replacement string. No borrow escapes `document`; its result remains one owned `String`. No shared state, unsafe code, new dependency, or retained cache.

## Operation

1. Decode/escape title and description in the existing order.
2. Inspect the borrowed body for root aliases once. If the default radius literal exists, walk matching rectangle spans to determine exact transformed length and aliases formed at removal boundaries.
3. Construct the existing SVG prefix and accessibility head. Reserve space for transformed body and closing tag.
4. Walk rectangle spans again, appending original slices and static replacements directly into the final output. Append the closing tag.

The second walk avoids retaining an edit vector proportional to rectangle count. Both walks are linear in input size; temporary working storage is constant. Each tag is searched a bounded number of times. Boundary alias checks inspect only fixed-size prefixes/suffixes, never repeatedly scan all preceding content.

## Exact raw substring behavior

This is deliberately not XML parsing. Match case-sensitive `<rect` followed by an ASCII non-word byte, then consume through the first literal `>` even inside quotes. Newlines, malformed attributes, nested `<rect` text, and non-XML icon markup retain Pele's regex behavior. Nonmatching tags remain byte-for-byte unchanged.

Within each matched tag, remove only the first exact ` rx="var(--pele-radius,4px)"`. Search the logically joined remainder for the first exact ` style="`, including a marker created across the removed substring. Insert `rx:var(--_r);` immediately after that marker. If absent, insert ` style="rx:var(--_r)"` immediately before the terminal `/>` or `>`; recognize `/>` created across the removal too. All other bytes keep their original order. Byte offsets come from ASCII match boundaries and remain UTF-8 boundaries.

Root aliases preserve their existing output order: radius, title, heading, bold. The removed literal contains none of these aliases, and its leading space/trailing quote cannot occur inside an alias, so existing matches survive. Removal can create a new title/heading/bold alias across its boundary; `crossing_ascii` checks those cases. Every edit inserts the radius alias. Insertion points occur after a quote or before a closing slash/angle bracket, so insertion cannot split an existing alias. No newly inserted text creates a title/heading/bold alias.

## Explicit compatibility correction

The old Rust regex `r"<rect\b[^>]*>"` uses a Unicode word boundary. Pele's JavaScript `/<rect\b[^>]*>/g` uses an ASCII word boundary. Therefore `<recté ...>` and `<rect中 ...>` must match Pele, while `<rectangle ...>`, `<rect_ ...>`, and `<rect9 ...>` must not. This candidate intentionally fixes that existing discrepancy; it does not claim byte equality with the old Rust result for those cases. The pinned oracle fixtures already expose the baseline mismatch. Keep this correction explicit in the eventual commit and verification report.

## Errors, callbacks, and limits

Title failure still precedes description failure. Body inspection remains after both and before font assembly. Root assembly invokes no host callback; icon callbacks have already supplied the body. This change neither schedules callbacks nor changes provider, presentation, revision, or partial-error handling.

Do not insert an early output limit here. Existing engine stages render the complete SVG, apply native theme when requested, then enforce the existing byte/UTF-16 limits at their current points. `OutputLimit` bounds accepted output, not trusted renderer peak allocation. No limit, error text, ordering, or transport framing changes.

Each radius replacement is shorter than the removed literal, so transformed body length cannot exceed source length. The final SVG still adds the same prefix/head/closing bytes (except the explicitly corrected Unicode matching cases). Near the 64 MiB transport ceiling, the old rounded path can retain the original body, a complete transformed body, and the final document concurrently, plus per-tag strings. The candidate retains original body plus final output and existing header temporaries, with constant edit storage. It does not allocate a per-rectangle vector. `reserve` uses the computed transformed length; allocator growth policy can still reserve spare capacity. This is a structural allocation hypothesis, not a measured peak-memory or speed claim.

## Required adoption evidence

Use public whole-SVG differential coverage through native source, typed retained, and public mutable lanes, plus web source and mutable lanes. Cover default/no radius, repeated radius, authored style before/after radius, style formed across removal, self-closing forms, alias fragments joined across removal, ASCII/non-ASCII tag boundaries, malformed and quoted `>` content, raw icon markup, title/description failures, font validation, responsive mode, and exact output limits. The fixtures/tests exercise public render interfaces; no private implementation unit tests were added.

Measure matched native source and typed retained workloads, including families without rounded rectangles as negative controls. Keep original/candidate outputs exact against Pele before timing. Report allocations separately from elapsed time. Two rectangle walks may trade some scan work for fewer allocations/copies; adoption requires measured benefit, not a language-level assumption.

## Source identity

- Original SHA-256: `527a11b68410ab69f2c2fe7482916ca0674be95bccec5eef23e2df5167fd5f9b`
- Candidate SHA-256: `baaff919f5110422a0dc076a56fc92b114eb975e9080b48a9178f92bcdb2ef63`

Apply only after checking `original.rs` still equals tracked `svg/mod.rs`. This directory's `control/` is parent-owned benchmark evidence and is not modified by this candidate.
