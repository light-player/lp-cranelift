#!/bin/bash

# Default test path
TEST_PATH="cranelift/filetests/filetests/32bit/"

# If a test path is provided as an argument, use it instead
if [ $# -gt 0 ]; then
    TEST_PATH="$1"
fi

cargo run -p cranelift-tools --bin clif-util -- test "$TEST_PATH"
