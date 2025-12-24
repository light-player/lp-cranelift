#!/bin/bash

# Get the script's directory and workspace root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Try to find workspace root if script is run from elsewhere
# Look for Cargo.toml or cranelift directory
find_workspace_root() {
    local dir="$1"
    while [ "$dir" != "/" ]; do
        if [ -f "$dir/Cargo.toml" ] && [ -d "$dir/cranelift" ]; then
            echo "$dir"
            return 0
        fi
        dir="$(dirname "$dir")"
    done
    return 1
}

# If workspace root detection from script location fails, try current directory
if [ ! -f "$WORKSPACE_ROOT/Cargo.toml" ] || [ ! -d "$WORKSPACE_ROOT/cranelift" ]; then
    WORKSPACE_ROOT="$(find_workspace_root "$(pwd)")" || {
        echo "Error: Could not find workspace root. Please run from the workspace root directory." >&2
        exit 1
    }
fi

# Change to workspace root
cd "$WORKSPACE_ROOT" || {
    echo "Error: Failed to change to workspace root: $WORKSPACE_ROOT" >&2
    exit 1
}

# Default test path
TEST_PATH="cranelift/filetests/filetests/32bit/"

# If a test path is provided as an argument, use it instead
if [ $# -gt 0 ]; then
    TEST_PATH="$1"
fi

# Disable concurrency for better error messages
export CRANELIFT_FILETESTS_THREADS=1

cargo run -p cranelift-tools --bin clif-util -- test "$TEST_PATH"
