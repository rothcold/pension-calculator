#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$(pwd)=. --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo"
cargo build --locked --manifest-path wasm/Cargo.toml --target wasm32-unknown-unknown --release
mkdir -p dist
cp wasm/target/wasm32-unknown-unknown/release/pension_browser.wasm dist/
cp web/index.html dist/index.html
cp frontend/favicon.svg frontend/favicon-32.png frontend/favicon.ico dist/
echo "Built dist/. Serve this directory over HTTP; do not open HTML as file://."
