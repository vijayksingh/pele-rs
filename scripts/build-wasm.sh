#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"
if [[ "$(wasm-bindgen --version)" != 'wasm-bindgen 0.2.128' ]]; then
    echo 'Install matching CLI: cargo install wasm-bindgen-cli --version 0.2.128 --locked' >&2
    exit 1
fi
# Explicit opt-in experiment; the smaller default remains measured and reproducible.
pele_wasm_opt_level="${PELE_WASM_OPT_LEVEL:-s}"
case "$pele_wasm_opt_level" in s|3) ;; *) echo 'PELE_WASM_OPT_LEVEL must be s or 3' >&2; exit 2 ;; esac
pele_wasm_build_args=(--locked --release -p pele-wasm --target wasm32-unknown-unknown)
case "${PELE_WASM_CAPTURED_SANKEY:-0}" in
    0) ;;
    1) pele_wasm_build_args+=(--features captured-sankey) ;;
    *) echo 'PELE_WASM_CAPTURED_SANKEY must be 0 or 1' >&2; exit 2 ;;
esac
# rust-lld on macOS needs the LLVM library shipped in the toolchain's lib/.
DYLD_FALLBACK_LIBRARY_PATH="$rust_bin/../lib:${DYLD_FALLBACK_LIBRARY_PATH:-/usr/local/lib:/usr/lib}" \
CARGO_PROFILE_RELEASE_PANIC=abort CARGO_PROFILE_RELEASE_OPT_LEVEL="$pele_wasm_opt_level" \
  cargo build "${pele_wasm_build_args[@]}"
mkdir -p dist/wasm dist/evidence
# Test oracle only, never included in the distributed WASM package.
cp node_modules/pele/dist/pele.js dist/evidence/pele-oracle.js
mkdir -p dist/evidence/pele-esm
cp -R node_modules/pele/dist/esm/. dist/evidence/pele-esm/
wasm-bindgen target/wasm32-unknown-unknown/release/pele_wasm.wasm \
  --target web --out-dir dist/wasm --out-name pele
mkdir -p bindings/web/pkg
cp dist/wasm/* bindings/web/pkg/
cp third_party/v8/LICENSE bindings/web/V8-LICENSE
cp third_party/fdlibm/LICENSE bindings/web/FDLIBM-LICENSE
cp LICENSE bindings/web/LICENSE
# NOTICE is already included by npm files; append dependency/runtime notices
# without changing any executable JS/WASM bytes.
cat NOTICE crates/pele/assets/RUST-DEPENDENCY-LICENSES.txt \
  crates/pele/assets/RUST-STDLIB-LICENSES.txt > bindings/web/NOTICE
cp third_party/PELE-LICENSE bindings/web/PELE-LICENSE
cp third_party/MERMAID-LICENSE bindings/web/MERMAID-LICENSE
cp crates/pele/assets/fonts/OFL.txt bindings/web/FONT-OFL.txt
mkdir -p dist/release dist/package-web
package_archive="$(npm pack ./bindings/web --pack-destination dist/release --silent)"
case "$package_archive" in
    ''|*[!a-zA-Z0-9._-]*) echo 'Invalid npm package archive name' >&2; exit 1 ;;
esac
# Test exactly the package just packed; removed files must not survive staging.
rm -rf "$repo_dir/dist/package-web/package"
tar -xzf "dist/release/$package_archive" -C dist/package-web
printf '%s\n' "$package_archive"
