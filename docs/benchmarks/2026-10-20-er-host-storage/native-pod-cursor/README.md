# Bounded immutable POD row cursor

Source `5db9f35b475a3ee479c925513d78ab237a6cda0e`. This JS-only layer improves the experimental native backend, but that backend remains slower than arena and stays disabled by default. One immutable copied row per header/property/item/string-descriptor table replaces repeated native scalar loads; UTF16 is copied in page-bounded chunks. Full graph restoration, observation and native admission remain.

The two sequential comparisons use the same 113-file control/candidate package maps, with only `native-storage.js` different. WASM is identical: `2310934dd43dc52c20723c3e0217377f9819fef8e146d03ef3a33845e06bdcce`. Candidate storage module is `33224aa2f52b2fc4a4ff9de2f36528f8d422ebe031e3d4831d1634650318cebe`. The first comparison enables native storage on both old and new packages; the second enables it only on the new package, leaving control on arena. Actual source/native probes validate these choices outside timers.

| Paired comparison, six conditions per stage | Parse paired change | Retained paired change | Source-render control |
| --- | ---: | ---: | ---: |
| Old native → cursor native | −29.78% to −22.19% | −37.70% to −32.98% | −0.20% to +1.04% |
| Arena → cursor native | +42.68% to +53.80% | +36.16% to +49.90% | −0.03% to +2.75% |

The old-native comparison has 17–21/21 lower parse pairs and 21/21 lower retained pairs for every condition. Against arena, the cursor still takes 1.413–1.543× for parse and 1.358–1.516× for retained render. Its pinned-JS ratios in that arena comparison are 15.90–23.02× parse and 2.89–4.93× retained. Both source controls, including adverse conditions, are retained. These are separate measured windows; no cross-window subtraction or inferred speedup is used. No first-use result is included.

## Fresh profile

Six medium/fixed 2-second CPU windows compare arena against cursor native. The run exited0, with no failed launch, and retained all raw profiles losslessly compressed. Output equality and 24 source/storage probes surround the sampling windows. This profile uses the exact same package maps as both latency runs.

| Cursor candidate | JS self | WASM self | GC self | NativeStorage JS self | Refresh ancestor (deduplicated) |
| --- | ---: | ---: | ---: | ---: | ---: |
| Parse | 41.58% | 44.62% | 8.27% | 20.55% | 15.24% |
| Retained render | 41.63% | 48.06% | 0.28% | 24.64% | 20.93% |

Shared publication is 38.00% inclusive for parse. Retained publication and matching form a **39.43% deduplicated union**; their individual groups are 20.00% and19.44%. Native construction is21.41% inclusive in parse; native flat storage34.04% overlaps it and admission. Admission is9.95% parse/9.60% retained inclusive. Sankey parser entry is5.65% of parse samples. These groups overlap and must not be added.

The remaining native-reader boundary is still substantial: each newly requested row refreshes the buffer and resolves its page, while shared traversal asks for scalar fields separately. The next candidate is an operation-scoped bounded copied POD window, preserving the same loops and descriptor/callback order. Copy only immutable native words; resolve String/Ref IDs at their original public-value stage. Discard windows at traversal exit and refresh before each new native window. This is an architecture hypothesis requiring its own parity and matched measurements, not a predicted gain from the percentages above. The source-render negative controls contain no graph-reader/publication samples.

## Evidence and checks

`versus-native.json` and `versus-arena.json` retain all36 rows and21-trial samples. Run `python3 summarize.py` to verify arithmetic, exact harness/case hashes, package maps, matched batch counts and96 untimed probes. Run `python3 analyze-profiles.py` for six weighted raw-profile validations and24 profile probes. The original profile summary remains unchanged. Disjoint self categories and deduplicated ancestor groups are explicit; profiling does not measure allocation counts or invocation counts.

`prepared-source.json`, `prepared-package.json`, the committed `source-package-freeze.json`, and `PROVENANCE.json` pin all587 source blobs to Git5db9f35. `swift-input-continuity.json` records unchanged native/Swift input content from validated3166b3d; this JS-only change did not trigger an SDK rebuild. The first focused run stopped on the new Unicode fixture (6passed,1failed,47notrun); the corrected fixture run passed all54. Both logs remain included as `focused-initial.log.gz` and `focused-repaired.log.gz`, losslessly compressed with original-byte hashes. Full-suite and SDK execution results belong to their separate source-pinned evidence and are not inferred here.

The source review covers immutable copied-row lifetime, bounds, lease closure and reentry. The corpus and public memory-growth/Proxy-publication/history-repair cases remain required gates for later reader changes. `index.json` hashes all included files, with raw uncompressed hashes for each CPU profile and focused log.
