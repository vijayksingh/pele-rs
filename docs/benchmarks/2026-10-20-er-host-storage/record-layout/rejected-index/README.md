# Rejected object-ID Map experiment

This separate experiment replaced three private WeakMap indexes with Map indexes while retaining the existing strong object arrays. It followed `bd7df0d` and was rejected and reversed; it is not part of the accepted record-layout runtime.

The 24-row, 21-trial VM comparison is mixed. ER medium parse is **+2.30%** paired with fixed resources (5/21 faster pairs) and **+1.99%** with Canvas (7/21). Every row remains in [comparison.json](comparison.json) and [paired-summary.json](paired-summary.json); adverse results are not dismissed as noise. The report's control package matches the record-layout candidate exactly; only `host-er-store.js` differs in the experiment candidate.

The separate browser run passed 45 checks in 29.1 seconds. Correctness did not justify adoption. No heap, GC, lifetime improvement or regression-free claim follows from this evidence. The archived [static review](STATIC_REVIEW.md) and [manifest](manifest.json) predate the browser/timing run; their “not run” fields describe that earlier point. The candidate patch is preserved for identification, not application.

The appendix does not alter record-layout results, its immutable source/package freeze, or the separate Swift comparison against 0b. No production commit contains this rejected experiment.
