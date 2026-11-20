#!/usr/bin/env sh
set -e

export RUSTUP_HOME=/tmp/.rustup
export CARGO_HOME=/tmp/.cargo
export PATH="/tmp/.cargo/bin:$PATH"

curl -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain stable
rustup target add wasm32-unknown-unknown

curl -sSfL https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-musl.tar.gz | tar -xz -C /tmp/.cargo/bin

trunk build --release
