#!/usr/bin/env sh
set -eu

if ! command -v rustup >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --default-toolchain stable
fi

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
rustup toolchain install stable --profile minimal
rustup target add wasm32-unknown-unknown --toolchain stable
RUSTUP_TOOLCHAIN=stable sh scripts/build-web.sh
