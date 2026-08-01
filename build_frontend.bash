#!/bin/bash
set -e
cargo build -p frontend --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript --out-dir static/wasm target/wasm32-unknown-unknown/debug/frontend.wasm