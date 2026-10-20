# Shared ER lexical dispatch — unreleased

Source **`df3597c`** removes per-line lowercase buffers and repeated direction searches, then dispatches keyword/cardinality checks by the first ASCII character. Direction-rule priority, rightmost matching candidate, cross-line whitespace, LF-only tail behavior and existing Unicode predicates remain unchanged. Grammar actions and storage interfaces are unchanged.

**No 0.3 release.** Native/Swift and VM results are separately scoped. The latest complete release gate remains the older 126-condition failure; these diagnostic gains do not establish JS parity or full release readiness.

## Independent VM

[All 24 conditions](comparison.json): three ER sizes plus Sequence-small controls, fixed/Canvas resources, parse/retained/source, 21 balanced triples and 1,000 ms warmup. Control is the immediately preceding result-packet artifact (`7339f38`); candidate is this lexer artifact. All 106 files per package are hashed before/after. Only `pkg/pele_bg.wasm` differs. Complete source/retained/fresh-model SVG and exact dimensions match before/after timing.

Positive paired change means slower. Candidate/JS is a ratio of separate medians.

| ER size | Resources | Parse paired change | Faster trials | Candidate / JS |
| --- | --- | ---: | ---: | ---: |
| er/small | fixed | -6.06% | 17/21 | 12.16× |
| er/small | canvas | -7.30% | 18/21 | 10.61× |
| er/medium | fixed | -7.69% | 15/21 | 9.49× |
| er/medium | canvas | -6.94% | 17/21 | 9.16× |
| er/large | fixed | -8.06% | 21/21 | 8.50× |
| er/large | canvas | -8.57% | 17/21 | 6.03× |

ER parse improves **6.06–8.57%** paired, with 15–21 faster trials per condition, but remains **6.03–12.16× JS**. ER source rendering improves 2.33–6.15% paired. Retained controls range −1.13% to +1.07%, with five of six adverse. All Sequence controls and sign differences between paired and separate medians remain in [paired summaries](paired-summary.json). No gain is compounded with the preceding packet window.

One page shares heap/GC/cache pressure; warmup is serial. Host-load/package identity is recorded, but this comparator does not emit per-row quiet guards or individual trial-order records. Download/startup, paint and GPU are excluded. This is a focused warm public-API diagnostic, not the complete release gate.

A [fresh independent remote rehash](independent-vm-rehash.json) after timing rereads every file in both immutable package directories and matches all 212 path/hash pairs. That is a remote byte check, not a downloaded second local hash or continuous monitoring. [Independent audit](AUDIT.md) records the complete provenance and limitations.

## Local Swift public APIs

[Complete paired report](swift-comparison.json.gz): frozen `0b32797` control versus fresh `df3597c` ARM64 SDK/consumer, 15 pairs per condition, 1,000 ms warmup, matching operation counts and 45/45 control-first/candidate-first ordering. All before/after, warmup/calibration and measured response identities match the pinned selected outputs.

| Case | Actual Swift API | Paired change | Faster pairs |
| --- | --- | ---: | ---: |
| ER medium | parseMutable | −20.77% | 15/15 |
| ER medium | lifetime | −3.65% | 12/15 |
| ER medium | immutable parse | −9.99% | 15/15 |
| Sequence small | parseMutable | +0.79% | 4/15 |
| Sequence small | lifetime | +1.21% | 3/15 |
| Sequence small | immutable parse | +0.71% | 5/15 |

This is **combined native evolution from `0b32797`**, not isolated lexer causality: 14 source inputs plus the SDK changed. It is not a Swift-versus-JS ratio. Real FFI/ownership/projection and per-operation autorelease drains stay inside the API timer; transport protocol and verification remain outside. Lifetime includes parseMutable, snapshot, no-op apply, render and close.

All 260 saved guards reject compiler/linker/build processes and Pele tests; none was bypassed. Desktop load remains shared: an accepted sample records a Chrome renderer near 43% CPU. No globally idle-machine or process-peak claim. The single complete accepted window is retained; no rejected/partial result is pooled.

## Compatibility

- 21 native public integrations pass, including the frozen ER parser/full-SVG corpus, direct/legacy construction, fallback/prepared admission, captured raw policy and registry controls.
- Twelve focused browser checks pass across Chromium/Firefox/WebKit: the pinned ER corpus, 70 added direction/keyword/cardinality/Unicode source cases and exact result-packet/byte-gate controls.
- Fresh ARM64 SDK: 168 Swift API conditions, 15 Swift integrations and three strict C consumers pass. Selected Swift corpus includes 7,535 mutable drawings and 9,902 errors with zero explicit budget differences; prepared/raw tests exercise all 33 families.
- The added grammar fixture passes against the preceding unmodified lexer and against this candidate. Its first baseline had an incorrect positive mixed-case core header: both implementations rejected that source equally. The corrected fixture keeps the supported header and mixed-case direction text. Initial failure records are retained; no parser result was waived.
- Intel SDK, AppKit activation/focus and complete release/performance gates remain open. The historical 369-browser suite belongs to `51f1269`, not this artifact.

[Source/package/SDK freeze](source-freeze.json), [Swift build manifest](swift-build-manifest.json), [source review](REVIEW.md), [independent audit](AUDIT.md) and [all archive hashes](evidence-index.json) are included. The audit checks frozen Git objects and actual local SDK/consumer binaries; it does not substitute the later working tree for historical inputs. `summarize.mjs` verifies the published arithmetic and identities without executing a renderer.
