#!/bin/sh

TARGET_DIR="target/release"
X64_TOOLCHAIN="nightly-x86_64-apple-darwin"
X86_TOOLCHAIN="nightly-i686-apple-darwin"
CARGO_OUTPUT="$TARGET_DIR/libAudioPlugin_Unidecim.dylib"
X86_TARGET="$TARGET_DIR/libAudioPlugin_Unidecim_x86.dylib"
X64_TARGET="$TARGET_DIR/libAudioPlugin_Unidecim_x64.dylib"
FAT_TARGET="$TARGET_DIR/AudioPlugin_Unidecim.dylib"

rustup run $X64_TOOLCHAIN cargo build --release && mv $CARGO_OUTPUT $X64_TARGET
rustup run $X86_TOOLCHAIN cargo build --release && mv $CARGO_OUTPUT $X86_TARGET
lipo $X86_TARGET $X64_TARGET -output $FAT_TARGET -create

