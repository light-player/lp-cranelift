#!/bin/bash
# Build lp-builtins-app executable with aggressive optimizations

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LIGHTPLAYER_DIR="$WORKSPACE_ROOT/lightplayer"
BUILTINS_APP="$LIGHTPLAYER_DIR/apps/lp-builtins-app"
TARGET="riscv32imac-unknown-none-elf"
OUTPUT_DIR="$LIGHTPLAYER_DIR/target/$TARGET/release"

echo "Building lp-builtins-app for $TARGET with aggressive optimizations..."

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

RUSTFLAGS="-C opt-level=1 \
           -C panic=abort \
           -C overflow-checks=off \
           -C debuginfo=0 \
           -C link-dead-code=off \
           -C codegen-units=1" \
cargo build \
    --target $TARGET \
    --package lp-builtins-app \
    --release \
    --bin lp-builtins-app

echo ""
echo "Built executable: $OUTPUT_DIR/lp-builtins-app"
ls -lh "$OUTPUT_DIR/lp-builtins-app"

# Show some stats
echo ""
echo "Executable stats:"
nm "$OUTPUT_DIR/lp-builtins-app" 2>/dev/null | grep "__lp_" | wc -l | xargs echo "  __lp_* symbols:"
nm "$OUTPUT_DIR/lp-builtins-app" 2>/dev/null | grep -E "^\s*[Tt]\s+(memcpy|memset|memcmp)$" | wc -l | xargs echo "  mem* symbols:"

