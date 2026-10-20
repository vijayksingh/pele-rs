#!/bin/bash
set -euo pipefail
[[ "$(git rev-parse HEAD)" == bb0fc18055a175de4645129ea25a4917a6fa7d17 ]]
CARGO_BUILD_JOBS=2 bash scripts/build-wasm.sh > dist/evidence/source-typed-columns-build-wasm.log 2>&1
node dist/perf-patches/parse-architecture/source-typed-columns-runtime/freeze.mjs package
python3 - <<'PY'
import pathlib,shutil
p=pathlib.Path('dist/perf-patches/parse-architecture/source-typed-columns-runtime/candidate-package');assert not p.exists();shutil.copytree('dist/package-web/package',p)
PY
for repeat in 1 2; do
 node scripts/generate-source-initializer.mjs dist/package-web/package bindings/web/layout-storage.js "dist/perf-patches/parse-architecture/source-typed-columns-runtime/generated-initializer-$repeat.js" >> dist/evidence/source-typed-columns-generator.log 2>&1
 cmp bindings/web/source-initializer.js "dist/perf-patches/parse-architecture/source-typed-columns-runtime/generated-initializer-$repeat.js"
done
npx playwright test --workers=1 --output=dist/evidence/source-typed-columns-web-artifacts --grep 'sankey pinned corpus|arena packet|binary WASM consumers|binary graph publication|all 33 packed|prepared entries keep|canonical built-in objects|acknowledged public binary|mutable Pie models|mutable Info metadata|native model methods|host callback throws|outline timeline graph|imported outline helpers|seven additional mutable|fresh public Sankey|source layout|typed source columns|source ingress preserves|fresh source publication|inherited fixed-field|railroad complete SVG|SVG root output' > dist/evidence/source-typed-columns-web.log 2>&1
node dist/perf-patches/parse-architecture/source-typed-columns-runtime/freeze.mjs after
