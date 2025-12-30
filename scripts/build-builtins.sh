#!/bin/bash
# Build lp-builtins static library with aggressive optimizations
# This reduces the library size and number of symbols significantly

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LIGHTPLAYER_DIR="$WORKSPACE_ROOT/lightplayer"
BUILTINS_CRATE="$LIGHTPLAYER_DIR/crates/lp-builtins"
TARGET="riscv32imac-unknown-none-elf"
OUTPUT_DIR="$LIGHTPLAYER_DIR/target/$TARGET/release"

echo "Building lp-builtins for $TARGET with aggressive optimizations..."

# Ensure target is installed
if ! rustup target list --installed | grep -q "^$TARGET$"; then
    echo "Installing target $TARGET..."
    rustup target add $TARGET
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Build using cargo but with RUSTFLAGS for optimization
# Using rustc directly would require handling dependencies manually
cd "$LIGHTPLAYER_DIR"

# Use nightly toolchain for compiler_builtins support
export RUSTUP_TOOLCHAIN=nightly

RUSTFLAGS="-C opt-level=2 \
           -C panic=abort \
           -C overflow-checks=off \
           -C debuginfo=0 \
           -C link-dead-code=off \
           -C codegen-units=1" \
cargo build \
    --target $TARGET \
    --package lp-builtins \
    --release \
    --features baremetal

echo ""
echo "Built library: $OUTPUT_DIR/liblp_builtins.a"
ls -lh "$OUTPUT_DIR/liblp_builtins.a"

# Show some stats
echo ""
echo "Library stats:"
ar t "$OUTPUT_DIR/liblp_builtins.a" | wc -l | xargs echo "  Object files:"
nm "$OUTPUT_DIR/liblp_builtins.a" 2>/dev/null | grep "__lp_" | wc -l | xargs echo "  __lp_* symbols:"

