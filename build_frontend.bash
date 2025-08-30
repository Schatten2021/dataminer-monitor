#!/bin/bash
cd frontend || return
cargo build
wasm-bindgen --target web --no-typescript --out-dir ../static/wasm target/wasm32-unknown-unknown/debug/frontend.wasm