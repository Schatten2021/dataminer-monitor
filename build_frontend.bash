#!/bin/bash
set -e
if [[ $1 == "--release" || $1 == "release" ]]; then
    cargo build -p frontend --target wasm32-unknown-unknown --release
    wasm-bindgen --target web --out-dir static/wasm --no-typescript --remove-name-section --remove-producers-section target/wasm32-unknown-unknown/release/frontend.wasm
else
    cargo build -p frontend --target wasm32-unknown-unknown
    wasm-bindgen --target web --no-typescript --out-dir static/wasm target/wasm32-unknown-unknown/debug/frontend.wasm
fi