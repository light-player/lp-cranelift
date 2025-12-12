# Phase 7: Fix Memory Access Issues

## Goal
Fix invalid memory writes to address 0x00000000, likely caused by incorrect stack setup or memory layout.

## Prerequisites
- Previous phases completed: Core functionality works

## Affected Test Files

This test fails with memory access errors:

```bash
# Test the fix:
cargo run --bin clif-util -- test filetests/filetests/runtests/uadd_overflow.clif
```

## Error Pattern

```
Emulator execution failed: Invalid memory write at address 0x00000000 (size: 1 bytes) at PC 0x00000044
```

Writing to address 0 is invalid and suggests:
- Stack pointer is incorrectly initialized (points to 0)
- Memory layout is wrong
- Stack allocation failed

## Root Cause Analysis

The emulator should set up memory like this:
- Code section: Loaded at some base address
- RAM section: Available for stack/heap
- Stack pointer: Should point to top of RAM (grows downward)

If SP is 0, it means stack initialization failed.

## Implementation Steps

### Step 1: Check Stack Initialization

File: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

Location: `call_function` method (around lines 365-465)

**Current code** (from earlier reading):
```rust
// Set up stack pointer (x2/sp) to end of RAM
let ram_size = self.memory.ram().len();
let stack_top = super::memory::DEFAULT_RAM_START + ram_size as u32;
self.regs[2] = (stack_top - 16) as i32; // 16-byte aligned, with some space
```

**Potential issues**:
1. `DEFAULT_RAM_START` might be 0
2. `ram_size` might be 0
3. Stack pointer calculation might overflow or underflow

**Check**:
- What is `DEFAULT_RAM_START`? (should be non-zero)
- Is RAM properly allocated?
- Is stack pointer set before function call?

### Step 2: Check Memory Layout

File: `lightplayer/crates/lp-riscv-tools/src/emu/memory.rs`

**Investigation**:
1. How is memory laid out?
2. Where is RAM located?
3. Where is the stack located?

**Expected layout**:
- Code: 0x00000000 - 0x00001000 (or similar)
- RAM: 0x00100000 - 0x00200000 (1MB RAM)
- Stack: Top of RAM, grows downward

### Step 3: Verify Stack Pointer Setup

File: `cranelift/filetests/src/test_run.rs`

Location: `EmulatorExecutor::new` (around lines 246-305)

**Current code** (from earlier reading):
```rust
// Set up stack pointer (stack starts at top of RAM, grows downward)
let stack_base = ram_size as u32;
emulator.set_register(Gpr::Sp, stack_base as i32);
emulator.set_pc(0);
```

**Potential issues**:
1. `ram_size` might be 0
2. Stack base calculation might be wrong
3. Stack pointer might not be set before execution

**Fix**:
```rust
// Ensure RAM is allocated
let ram_size = 1024 * 1024; // 1MB RAM
let ram = vec![0; ram_size];

// Set up stack pointer at top of RAM (with some margin)
let stack_base = ram_size as u32;
let stack_pointer = stack_base - 16; // 16-byte aligned, with margin
emulator.set_register(Gpr::Sp, stack_pointer as i32);
```

### Step 4: Add Memory Access Validation

File: `lightplayer/crates/lp-riscv-tools/src/emu/memory.rs`

**Add validation** to prevent writes to address 0:

```rust
pub fn write_byte(&mut self, addr: u32, value: u8) -> Result<(), EmulatorError> {
    if addr == 0 {
        return Err(EmulatorError::InvalidMemoryAccess {
            kind: MemoryAccessKind::Write,
            address: addr,
            size: 1,
            reason: "Attempted to write to address 0 (null pointer)",
            // ...
        });
    }
    // ... rest of write logic
}
```

### Step 5: Check Function Call Setup

File: `lightplayer/crates/lp-riscv-tools/src/emu/emulator.rs`

Location: `call_function` method

**Verify**:
1. Stack pointer is set before calling function
2. Stack has enough space
3. Function arguments are passed correctly (might use stack for some args)

## Debugging Strategy

1. **Add logging**:
   ```rust
   eprintln!("Stack setup: ram_size={}, stack_base={}, sp={}", 
             ram_size, stack_base, emulator.get_register(Gpr::Sp));
   ```

2. **Check memory layout**:
   ```rust
   eprintln!("Memory layout: code_start={:08x}, ram_start={:08x}, ram_size={}", 
             code_start, ram_start, ram_size);
   ```

3. **Trace memory writes**:
   - Log all memory writes
   - Identify which instruction writes to address 0
   - Check if it's a stack operation (SP-relative)

4. **Verify stack operations**:
   - Check if function uses stack for local variables
   - Verify stack pointer is correct when function starts
   - Check if stack grows correctly

## Testing

After making changes:

```bash
# Test memory access fix:
cargo run --bin clif-util -- test filetests/filetests/runtests/uadd_overflow.clif
```

## Common Issues

1. **Stack pointer not initialized**: SP is 0 at function start
   - **Fix**: Ensure SP is set in `call_function` before executing

2. **Stack overflow**: Function uses more stack than available
   - **Fix**: Increase RAM size or check stack usage

3. **Incorrect stack calculation**: Stack pointer calculation is wrong
   - **Fix**: Verify stack grows downward correctly
   - **Fix**: Ensure stack pointer is within valid RAM range

4. **Null pointer dereference**: Code tries to write to address 0
   - **Fix**: Add validation to catch and report this clearly
   - **Fix**: Check if this is a bug in generated code or emulator

## Success Criteria

- Test compiles and runs without memory access errors
- Stack pointer is properly initialized (non-zero)
- No writes to address 0
- Test may still fail for other reasons (wrong results, etc.)

## Next Phase

Once memory access is fixed, proceed to Phase 8 to handle compilation errors.

