# Dense baseline rebase

Parent rejected the canonical-string-index experiment after mixed VM results. This candidate now starts from the restored dense-array JS, with firstStrings/lastStrings unchanged. Literal additions and Rust event selection are byte-identical to the independently reviewed prior patch; no oneMap content remains. Previous files, originals, patch, manifest and implementation review are archived in superseded-one-map. Refreshed node syntax and git apply checks passed. No production edits/builds/tests/measurements.
