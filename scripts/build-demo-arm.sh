#!/bin/sh
# Build the fbink-sys demo for a Kindle (ARMv7 hard-float, static musl),
# using zig as the cross C toolchain — no cross-gcc or device sysroot needed.
#
#   ./scripts/build-demo-arm.sh              # debug build
#   ./scripts/build-demo-arm.sh --release    # release build
#
# Output: target/armv7-unknown-linux-musleabihf/<profile>/examples/fbink-sys-demo
# Deploy: scp -O -P 4022 <output> root@127.0.0.1:/mnt/us/
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CC_ARM="$ROOT/scripts/zig-cc-arm"

# Zig's own musl include dirs, so bindgen's ARM parse of fbink.h doesn't trip
# over the host's glibc headers (bindgen injects --target=<triple> itself).
ZIG_LIB="$(zig env | sed -n 's/.*\.lib_dir = "\([^"]*\)".*/\1/p')"
[ -n "$ZIG_LIB" ] || ZIG_LIB=/usr/lib/zig
BINDGEN_ARGS="-isystem $ZIG_LIB/include \
 -isystem $ZIG_LIB/libc/include/arm-linux-musl \
 -isystem $ZIG_LIB/libc/include/generic-musl \
 -isystem $ZIG_LIB/libc/include/arm-linux-any \
 -isystem $ZIG_LIB/libc/include/any-linux-any"

env "CC_armv7_unknown_linux_musleabihf=$CC_ARM" \
    "CC_armv7-unknown-linux-musleabihf=$CC_ARM" \
    "AR_armv7_unknown_linux_musleabihf=ar" \
    "BINDGEN_EXTRA_CLANG_ARGS_armv7_unknown_linux_musleabihf=$BINDGEN_ARGS" \
    "BINDGEN_EXTRA_CLANG_ARGS_armv7-unknown-linux-musleabihf=$BINDGEN_ARGS" \
    "CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=$CC_ARM" \
    "RUSTFLAGS=-C link-self-contained=no" \
    FBINK_TARGET=kindle \
    cargo build -p fbink-sys --example fbink-sys-demo \
    --target armv7-unknown-linux-musleabihf "$@"