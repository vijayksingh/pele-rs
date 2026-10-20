# Profile target versus actual duration

The executed program uses `settings.durationMs = 5000`. This is a loop target, not an exact five-second observation guarantee.

In the unchanged `profile/runtime/fetched/profiles/summary.json.gz`, the `sankey-large-fixed-parse-candidate` entry records `elapsedMs = 7580.785000801086` and `profileSpanUs = 7876194`. The corresponding raw profile has `endTime - startTime = 7876194` microseconds. Other operation-loop elapsed times are 5001.704999923706, 5000.010000228882 and 5000.75 ms.

The original archived `profile/RESULTS.md` used compressed "valid5s" wording. Its bytes are preserved as historical evidence; this note and the authored README correct that wording. Weighted attribution remains computed from the exact raw time deltas. No measurements were rerun or altered.
