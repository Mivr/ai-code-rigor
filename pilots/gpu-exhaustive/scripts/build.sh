#!/bin/sh
# Build both compiler variants inside a Debian bookworm Rust container (the binaries run on
# newer-glibc hosts too). Usage: scripts/build.sh <workdir>   (workdir holds src, caches, bin)
set -eu
W=${1:?workdir}
mkdir -p "$W/home" "$W/cargo-home" "$W/target" "$W/bin"
run() {
  docker run --rm -u "$(id -u):$(id -g)" -e HOME=/w/home -e CARGO_HOME=/w/cargo-home \
    -e CARGO_TARGET_DIR=/w/target -v "$W:/w" -w /w/src rust:1-bookworm sh -c "$1"
}
run "rustc --version && cargo build --release --locked"
cp "$W/target/release/gpu-exhaustive" "$W/bin/gpu-exhaustive-wgsl"
run "cargo build --release --locked --features spirv"
cp "$W/target/release/gpu-exhaustive" "$W/bin/gpu-exhaustive-spirv"
ls -la "$W/bin"
