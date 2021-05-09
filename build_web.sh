#! /bin/sh
cargo build --target "wasm32-unknown-unknown" --features web
wasm-bindgen --out-dir target --out-name wasm --target web --no-typescript target/wasm32-unknown-unknown/debug/game-of-life-bevy.wasm
