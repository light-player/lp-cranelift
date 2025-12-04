# embive-program Status

## Current State: Phase 1 - Hello World (99% Complete)

### ✅ What Works
1. **Build System**: Successfully compiles no_std RISC-V binary with Cranelift
2. **Runtime**: runtime-embive provides entry point, allocator, syscalls, panic handler
3. **ELF Loading**: lp-riscv-tools can parse and load RISC-V ELF files
4. **Test Framework**: Full-stack test builds the program and loads it into emulator
5. **Syscall Interface**: Test can handle write, done, and panic syscalls

### 🔄 Current Blocker: RVC (Compressed Instructions)

The Rust compiler generates compressed (RVC) instructions for the `riscv32imac` target.
These are 16-bit instructions (opcode 0x00, 0x01, 0x02) that save code space.

**Problem**: The lp-riscv-tools emulator currently only supports standard 32-bit RISC-V instructions.

**Test Output**:
```
[3/4] Running in RISC-V emulator...
Error: InvalidInstruction { pc: 604, instruction: 289505410, reason: "Unknown opcode: 0x02", ...}
```

### Solutions

**Option 1: Add RVC Support to Emulator** (Recommended for real hardware compatibility)
- Pros: Matches real riscv32imac hardware, smaller binaries
- Cons: Requires implementing ~40 compressed instruction variants
- Status: Not yet implemented

**Option 2: Use Custom Stdlib Without RVC** 
- Pros: Works with current emulator
- Cons: Complex to build, doesn't match real hardware
- Status: Not feasible with standard Rust toolchain

**Option 3: Baremetal Assembly-Only Test**
- Pros: Simple, no compressed instructions
- Cons: Doesn't test the full Rust/Cranelift toolchain
- Status: Could be used as intermediate step

### Next Steps

1. **Immediate**: Add basic RVC decoding to lp-riscv-tools emulator
2. **Then**: Complete Phase 1 hello world test  
3. **Finally**: Add Phase 2 toy language demo

### Test Instructions

```bash
# Build the program
cargo build --package embive-program --target riscv32imac-unknown-none-elf --release

# Run the test (currently fails at RVC instruction)
cargo test --package lp-riscv-tools --features std riscv_nostd -- --ignored --nocapture
```

### Files Created

- `apps/embive-program/` - Simple no_std hello world program
- `crates/runtime-embive/` - RISC-V runtime support
- `crates/lp-riscv-shared/` - Toy language compilation utilities (for Phase 2)
- `crates/lp-riscv-tools/src/elf_loader.rs` - ELF loading utility
- `crates/lp-riscv-tools/tests/riscv_nostd_test.rs` - Full-stack test

