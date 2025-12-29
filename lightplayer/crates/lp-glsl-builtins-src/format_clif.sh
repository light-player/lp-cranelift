#!/bin/bash
set -e

# Extract the function from the generated CLIF and format it for filetests
INPUT="output/divide64.clif"
OUTPUT="output/divide64_formatted.clif"

echo "Formatting CLIF for filetests..."

# Find the function start (block0)
FUNC_START=$(grep -n "^function\|^block0" "$INPUT" | head -1 | cut -d: -f1)

if [ -z "$FUNC_START" ]; then
    echo "Error: Could not find function start"
    exit 1
fi

# Extract function signature and body
# Remove Rust metadata comments (lines starting with ; symbol, ; instance, ; abi, ; kind, ; write_cvalue, etc.)
# Keep essential CLIF structure

{
    # Write filetests header
    echo "test interpret"
    echo "test run"
    echo "target riscv32 has_m"
    echo ""
    
    # Extract function signature (should be on line with "function")
    FUNCTION_LINE=$(sed -n "${FUNC_START}p" "$INPUT")
    
    # Convert function signature from aarch64 to riscv32 format
    # Change: u0:2(i32, i32, i32) -> i32 apple_aarch64
    # To: %divide64(i32, i32, i32) -> i32
    echo "function %divide64(i32, i32, i32) -> i32 {"
    
    # Extract blocks, removing Rust metadata
    sed -n "${FUNC_START},$p" "$INPUT" | \
        grep -v "^;" | \
        grep -v "^$" | \
        sed 's/^function.*$/function %divide64(i32, i32, i32) -> i32 {/' | \
        sed 's/apple_aarch64/system_v/g' | \
        sed 's/u0:2/%divide64/g'
    
    echo "}"
    
} > "$OUTPUT"

echo "Formatted CLIF written to: $OUTPUT"
echo ""
echo "Note: This is a basic extraction. Manual cleanup may be needed to:"
echo "  - Remove remaining Rust metadata"
echo "  - Fix target-specific instructions"
echo "  - Ensure proper riscv32 compatibility"

