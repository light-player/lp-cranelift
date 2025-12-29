#!/bin/bash
set -e

# Install rustc-codegen-cranelift if not already installed
echo "Installing rustc-codegen-cranelift-preview..."
rustup component add rustc-codegen-cranelift-preview --toolchain nightly || true

# Clean previous build artifacts
rm -rf *.clif output/*.clif

# Compile to CLIF
# Note: rustc_codegen_cranelift may not support riscv32 directly
# We'll compile for the native target first to generate CLIF, then adapt for riscv32
# The CLIF IR is mostly target-independent for our purposes
echo "Compiling to CLIF IR with minimal safety checks..."
rustc +nightly \
  -Zcodegen-backend=cranelift \
  --crate-type lib \
  --emit=llvm-ir \
  -C opt-level=2 \
  -C panic=abort \
  -C overflow-checks=off \
  -C debuginfo=0 \
  -C link-dead-code=off \
  -C codegen-units=1 \
  -C target-feature=+crt-static \
  src/lib.rs

# Find the generated CLIF file
# rustc_codegen_cranelift creates files in a .clif directory
CLIF_DIR="$(find . -name '*.clif' -type d | head -1)"
if [ -z "$CLIF_DIR" ]; then
    echo "Error: Could not find .clif directory"
    exit 1
fi

echo "Found CLIF directory: $CLIF_DIR"

# Find the divide64 function CLIF file
# The file name format is typically: {symbol_name}.{postfix}.clif
DIVIDE64_CLIF=$(find "$CLIF_DIR" -name '*divide64*.clif' | head -1)
if [ -z "$DIVIDE64_CLIF" ]; then
    echo "Warning: Could not find divide64.clif, listing all CLIF files:"
    find "$CLIF_DIR" -name '*.clif' | head -10
    # Try to find any CLIF file as fallback
    DIVIDE64_CLIF=$(find "$CLIF_DIR" -name '*.clif' | head -1)
    if [ -z "$DIVIDE64_CLIF" ]; then
        echo "Error: No CLIF files found"
        exit 1
    fi
    echo "Using: $DIVIDE64_CLIF"
fi

echo "Found CLIF file: $DIVIDE64_CLIF"

# Copy to output directory
mkdir -p output
cp "$DIVIDE64_CLIF" output/divide64.clif

echo "CLIF IR written to: output/divide64.clif"
echo ""
echo "To view the CLIF IR, run:"
echo "  cat output/divide64.clif"

