#!/usr/bin/env sh
set -eu
cd "$(dirname "$0")/.."
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$(pwd)=. --remap-path-prefix=${CARGO_HOME:-$HOME/.cargo}=/cargo"
cargo build --locked --manifest-path wasm/Cargo.toml --target wasm32-unknown-unknown --release
mkdir -p dist
cp wasm/target/wasm32-unknown-unknown/release/pension_browser.wasm dist/
cp web/index.html web/theme.js web/robots.txt web/sitemap.xml dist/
mkdir -p dist/methodology
cp web/methodology/index.html dist/methodology/
mkdir -p dist/styles
cp web/styles/alloy.css web/styles/teal.css dist/styles/
cp frontend/favicon.svg frontend/favicon-32.png frontend/favicon.ico dist/
echo "Built dist/. Serve this directory over HTTP; do not open HTML as file://."
