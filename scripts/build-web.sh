#!/usr/bin/env sh
# Builds the analysis crates to WebAssembly and writes the JS bindings to web/pkg.
# Needs: rustup target wasm32-unknown-unknown, and wasm-bindgen-cli matching the
# wasm-bindgen version in Cargo.lock (0.2.129). wasm-opt is used when available.
set -eu
cd "$(dirname "$0")/.."

cargo build -p anatomy-wasm --target wasm32-unknown-unknown --profile wasm
wasm-bindgen --target web --no-typescript --out-dir web/pkg \
  target/wasm32-unknown-unknown/wasm/anatomy_wasm.wasm

if command -v wasm-opt >/dev/null 2>&1; then
  wasm-opt -Os --enable-bulk-memory --enable-nontrapping-float-to-int \
    web/pkg/anatomy_wasm_bg.wasm -o web/pkg/anatomy_wasm_bg.wasm
fi

ls -l web/pkg
