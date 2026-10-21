# Native proof helper import correction

The first helper invocation failed during module loading with `ERR_MODULE_NOT_FOUND`: the relative import resolved to `/Users/vijaysingh/unlocalhosted/scripts/perf/swift/build.mjs`, one parent directory above the repository. No test, build, or proof phase ran in that invocation.

The import was corrected to `../../../../scripts/perf/swift/build.mjs`. `node --check` and the `before` phase then passed, producing `native-proof-before.json` for commit `d7caf8c684ac6b6fda3918a2bcbd91c5953b92c9`.

This note records the reported failure and correction; it is not a reconstructed raw stderr log. Original terminal output remains in the parent execution record.
