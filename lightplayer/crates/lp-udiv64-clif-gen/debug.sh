#!/bin/bash
set -e

# Script to build, clean, and test the divide64 CLIF generation
# Usage: ./debug.sh [filename]
#   If filename is provided, copies it over lib.rs before building
#   Example: ./debug.sh minimal.rs
#   If no filename provided, uses lib.rs as-is

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
FILETEST_DIR="$WORKSPACE_ROOT/cranelift/filetests/filetests/32bit/udiv/debug"
CLEAN_CLIF="$SCRIPT_DIR/output/divide64_clean.clif"
SRC_DIR="$SCRIPT_DIR/src"
LIB_RS="$SRC_DIR/lib.rs"
LIB_RS_BACKUP="$SRC_DIR/lib.rs.backup"

# Determine output filename based on input file
if [ -n "$1" ]; then
    # Extract base name without .rs extension
    BASE_NAME=$(basename "$1" .rs)
    FINAL_CLIF="$FILETEST_DIR/${BASE_NAME}.clif"
else
    # Default to udiv64.clif if no file specified
    FINAL_CLIF="$FILETEST_DIR/udiv64.clif"
fi

# Handle optional filename argument
if [ -n "$1" ]; then
    SOURCE_FILE="$SRC_DIR/$1"
    if [ ! -f "$SOURCE_FILE" ]; then
        echo "Error: File not found: $SOURCE_FILE"
        echo "Available files in src/:"
        ls -1 "$SRC_DIR"/*.rs 2>/dev/null | xargs -n1 basename || echo "  (none found)"
        exit 1
    fi
    
    echo "=== Using $1 instead of lib.rs ==="
    # Backup original lib.rs if it exists and isn't already a backup
    if [ -f "$LIB_RS" ] && [ ! -f "$LIB_RS_BACKUP" ]; then
        cp "$LIB_RS" "$LIB_RS_BACKUP"
        echo "Backed up original lib.rs to lib.rs.backup"
    fi
    
    # Copy the specified file over lib.rs
    cp "$SOURCE_FILE" "$LIB_RS"
    echo "Copied $1 to lib.rs"
else
    echo "=== Using lib.rs as-is ==="
    # Note: lib.rs.backup will be preserved if it exists
    # To restore manually: mv src/lib.rs.backup src/lib.rs
fi

echo ""
echo "=== Building CLIF from Rust ==="
cd "$SCRIPT_DIR"
bash build.sh

echo ""
echo "=== Cleaning CLIF ==="
bash clean_clif.sh

echo ""
echo "=== Preparing filetest ==="
# Extract the function body (everything between function declaration and closing brace)
FUNC_START=$(grep -n "^function" "$CLEAN_CLIF" | cut -d: -f1)
FUNC_END=$(grep -n "^}" "$CLEAN_CLIF" | tail -1 | cut -d: -f1)

if [ -z "$FUNC_START" ] || [ -z "$FUNC_END" ]; then
    echo "Error: Could not find function boundaries in cleaned CLIF"
    exit 1
fi

# Extract function name and signature
FUNC_LINE=$(sed -n "${FUNC_START}p" "$CLEAN_CLIF")
FUNC_NAME=$(echo "$FUNC_LINE" | sed -n 's/^function %\([^(]*\).*/\1/p')
FUNC_SIG=$(echo "$FUNC_LINE" | sed -n 's/^function %[^(]*(\(.*\)) ->.*/\1/p')

# Count number of parameters
PARAM_COUNT=$(echo "$FUNC_SIG" | grep -o "i32\|i64" | wc -l | tr -d ' ')

echo "Function: $FUNC_NAME"
echo "Signature: $FUNC_SIG"
echo "Parameter count: $PARAM_COUNT"

# Create the final CLIF file with header, function body, and footer
{
    # Header
    echo "test interpret"
    echo "test run"
    echo "target riscv32 has_m has_zbb"
    echo ""
    
    # Function body (from cleaned CLIF, skipping the header we already added)
    sed -n "${FUNC_START},${FUNC_END}p" "$CLEAN_CLIF" | \
        grep -v "^test interpret" | \
        grep -v "^test run" | \
        grep -v "^target riscv32"
    
    echo ""
    echo "; Test cases"
    # Extract test expectations from the Rust source file
    # Look for // run: comments and convert function name if needed
    if [ -n "$1" ]; then
        SOURCE_FILE="$SRC_DIR/$1"
    else
        SOURCE_FILE="$LIB_RS"
    fi
    
    if [ -f "$SOURCE_FILE" ]; then
        # Extract // run: lines and convert function name
        # Replace divide64 with the actual function name from CLIF
        grep "^// run:" "$SOURCE_FILE" | sed "s/^\/\/ run: /; run: /" | sed "s/divide64/%${FUNC_NAME}/g"
    else
        # Fallback: Generate test cases based on parameter count
        if [ "$PARAM_COUNT" -eq 2 ]; then
            echo "; run: %${FUNC_NAME}(0, 100) == 0"
            echo "; run: %${FUNC_NAME}(100, 2) == 1"
            echo "; run: %${FUNC_NAME}(50, 100) == 0"
        elif [ "$PARAM_COUNT" -eq 3 ]; then
            echo "; run: %${FUNC_NAME}(0, 0, 100) == 0"
            echo "; run: %${FUNC_NAME}(0, 100, 2) == 50"
            echo "; run: %${FUNC_NAME}(0, 16, 2) == 8"
            echo "; run: %${FUNC_NAME}(0, 123456, 789) == 156"
            echo "; run: %${FUNC_NAME}(1, 0, 2) == 0x80000000"
            echo "; run: %${FUNC_NAME}(0xa, 0, 0x28000) == 0x40000"
        else
            echo "; TODO: Add test cases for $PARAM_COUNT parameters"
        fi
    fi
} > "$FINAL_CLIF"

echo "Created filetest: $FINAL_CLIF"
echo "Lines: $(wc -l < "$FINAL_CLIF")"

echo ""
echo "=== Running tests ==="
cd "$WORKSPACE_ROOT"
cargo run -p cranelift-tools --bin clif-util -- test "$FINAL_CLIF" 2>&1

# Note about restoring lib.rs
if [ -n "$1" ] && [ -f "$LIB_RS_BACKUP" ]; then
    echo ""
    echo "Note: Original lib.rs was backed up to lib.rs.backup"
    echo "To restore: mv src/lib.rs.backup src/lib.rs"
    echo "Or run: ./debug.sh (without arguments) to restore automatically"
fi

