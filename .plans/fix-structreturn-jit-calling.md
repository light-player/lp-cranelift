# Fix StructReturn JIT Calling Issue

## Problem Statement

The test app (`apps/test-structreturn`) successfully compiles CLIF with StructReturn and generates correct machine code, but the function call hangs when executed via JIT. The CLIF generation is correct, indicating the issue is in how we call the JIT-compiled function from Rust.

## Analysis from cranelift-examples

### Key Findings

1. **StructReturn Parameter Position:**
   - In `lowering-structs/types.rs` (line 61-62): StructReturn is added FIRST to params when it's the only special param
   - In `struct-layouts/main.rs` (line 247-251): When BOTH StructArgument and StructReturn exist, StructReturn is SECOND (after StructArgument)
   - **Our code correctly adds StructReturn FIRST** since we have no StructArgument params

2. **Function Signature:**
   - CLIF signature: `(i64 sret) -> ()` for 64-bit, `(i32 sret) -> ()` for 32-bit
   - Function takes ONE pointer parameter (StructReturn) and returns void
   - This matches our implementation

3. **Calling from Rust/JIT:**
   - The examples use ObjectModule (for object files), not JITModule
   - No direct examples of calling StructReturn functions via JIT from Rust
   - Our test app needs to match the exact ABI calling convention

4. **Entry Block Parameter Setup:**
   - In `lowering-structs/lower.rs` (line 47-50): StructReturn parameter is added to entry block FIRST before other params
   - In `struct-layouts/main.rs` (line 184-185): StructReturn is accessed as the SECOND block param (index 1) when StructArgument is first
   - **Our code correctly accesses StructReturn as index 0** since it's the only param

### Root Cause Hypothesis

The function call hangs likely due to:

1. **Calling convention mismatch**: Using `extern "C"` may not match the actual calling convention used by Cranelift (apple_aarch64 vs SystemV)
2. **Function pointer type mismatch**: The pointer type in the signature (i64/i32) must match the actual pointer size, but we're casting to `*mut f32` which may not match the ABI expectations
3. **Memory protection/alignment**: The buffer might need specific alignment or the memory might not be accessible
4. **ABI parameter passing**: The StructReturn pointer might need to be passed in a specific register or with specific alignment

## Solution Approach

### Step 1: Fix Function Pointer Signature

The function signature in CLIF is `(i64 sret) -> ()` for ARM64. We need to match this exactly:

- Use the pointer type that matches the ISA (i64 for ARM64, i32 for RISC-V32)
- Don't use `extern "C"` - use the calling convention that matches the signature's `call_conv`
- For apple_aarch64, we may need to use a different calling convention

### Step 2: Match Calling Convention

The signature uses `CallConv::triple_default(&triple)` which gives us:
- `CallConv::AppleAarch64` for macOS ARM64
- `CallConv::SystemV` for Linux ARM64 and RISC-V32

We need to ensure our function pointer type matches this calling convention.

### Step 3: Verify Buffer Alignment and Accessibility

Ensure the buffer:
- Is properly aligned (4-byte alignment for f32 should be sufficient)
- Is in writable memory
- Has the correct size

### Step 4: Test Different Calling Approaches

Try different approaches:
1. Use raw function pointer with correct pointer type: `fn(*mut u8) -> ()` or `fn(u64) -> ()`
2. Match the exact calling convention from the signature
3. Ensure the pointer is passed correctly (may need to cast to the pointer type used in signature)

## Implementation Plan

### Phase 1: Fix Test App Function Call

**File: `apps/test-structreturn/src/main.rs`**

1. **Get the exact calling convention from the signature:**
   - Use `sig.call_conv` to determine the calling convention
   - Store this for use when creating the function pointer

2. **Match the pointer type exactly:**
   - Use `pointer_type` (i64 or i32) instead of `*mut f32`
   - Cast the buffer pointer to match the signature's pointer type

3. **Create function pointer with correct signature:**
   ```rust
   // For i64 pointer (ARM64):
   let func: unsafe extern "C" fn(u64) = unsafe { mem::transmute(code_ptr) };
   func(buffer.as_mut_ptr() as u64);
   
   // OR try matching the exact calling convention:
   // May need platform-specific calling conventions
   ```

4. **Add error handling and debugging:**
   - Add checks for buffer alignment
   - Verify pointer is valid before calling
   - Add signal handlers to catch crashes

### Phase 2: Test on Multiple ISAs

1. **Test on native ISA (ARM64 macOS):**
   - Verify StructReturn works with apple_aarch64 calling convention
   - Test vec2, vec3, vec4 returns

2. **Test on RISC-V32 (if available):**
   - Verify StructReturn works with SystemV calling convention
   - Test with i32 pointer type

3. **Test function calls (not just main):**
   - Test calling a function that returns vec3
   - Verify StructReturn works in both caller and callee

### Phase 3: Apply Fixes to Main Codebase

Once the test app works:

1. **Verify compiler.rs wrappers:**
   - Ensure `compile_vec2`, `compile_vec3`, etc. use the same pattern
   - Test that filetests pass

2. **Update documentation:**
   - Document the correct way to call StructReturn functions
   - Add examples for different ISAs

## Technical Details

### StructReturn ABI on Different Platforms

**ARM64 (macOS - AppleAarch64):**
- StructReturn pointer passed in x8 register (first argument register)
- Pointer is 64-bit (i64)
- Calling convention: AppleAarch64

**ARM64 (Linux - SystemV):**
- StructReturn pointer passed in x0 register
- Pointer is 64-bit (i64)
- Calling convention: SystemV

**RISC-V32:**
- StructReturn pointer passed in a0 register
- Pointer is 32-bit (i32)
- Calling convention: SystemV

### Function Pointer Type Matching

The key insight: The function signature in CLIF uses the pointer TYPE (i64/i32), not a Rust pointer type. When calling from Rust, we need to:

1. Cast the function pointer to match the signature's parameter type
2. Pass the buffer pointer as the correct integer type (u64/u32)
3. Ensure the calling convention matches

## Testing Strategy

1. **Minimal test case:**
   - Simple function that writes 3 f32s to StructReturn buffer
   - Verify values are correct after call

2. **Progressive testing:**
   - Test vec2 (2 f32s)
   - Test vec3 (3 f32s)
   - Test vec4 (4 f32s)
   - Test function calls (caller allocates buffer, callee writes)

3. **Cross-ISA testing:**
   - Test on native ISA first
   - Test on RISC-V32 if available
   - Document any ISA-specific differences

## Success Criteria

1. ✅ Test app compiles CLIF with StructReturn correctly (already working)
2. ✅ Test app executes StructReturn function without hanging
3. ✅ Test app verifies correct values are written to buffer
4. ✅ Test app works on ARM64 (native)
5. ✅ Test app works on RISC-V32 (if available)
6. ✅ GLSL filetests pass with StructReturn
7. ✅ No architecture-specific workarounds needed

## Files to Modify

1. `apps/test-structreturn/src/main.rs` - Fix function pointer signature and calling
2. Verify `crates/lp-glsl/src/compiler.rs` - Ensure wrappers match the pattern
3. Test `crates/lp-glsl-filetests` - Verify filetests work with StructReturn

## Solution Found

### Root Cause

The issue was **not** a function pointer type mismatch, but a **register assignment mismatch**:

- On ARM64 AppleAarch64, StructReturn uses **x8** register (not x0)
- When calling with `extern "C" fn(*mut f32)`, Rust puts the parameter in **x0**
- The JIT-compiled function expects the StructReturn pointer in **x8**
- Result: Function was called but received wrong/uninitialized pointer, writing to wrong memory

### Fix

Use inline assembly to pass the StructReturn pointer in x8:

```rust
#[cfg(target_arch = "aarch64")]
unsafe fn call_structreturn_apple_aarch64(func_ptr: *const u8, buffer: *mut f32) {
    asm!(
        "mov x8, {buffer}",
        "mov x9, {func}",
        "blr x9",
        buffer = in(reg) buffer as u64,
        func = in(reg) func_ptr as u64,
        out("x8") _,
        out("x9") _,
        clobber_abi("C"),
    );
}
```

Key points:
- Move buffer pointer to x8 (StructReturn register)
- Move function pointer to x9 (temporary register for blr)
- Call function with `blr x9`
- Mark x8 and x9 as outputs to prevent optimization

### Test Results

✅ Test app now works correctly on ARM64 AppleAarch64
✅ StructReturn writes correct values to buffer
✅ Function returns without hanging

## Next Steps

1. ✅ Fix the function pointer calling convention in test app (DONE)
2. Apply the same pattern to compiler.rs wrappers for production use
3. Handle SystemV calling convention (StructReturn in first arg register, typically x0)
4. Handle RISC-V32 calling convention (StructReturn in first arg register, typically a0)
5. Run full filetest suite to verify everything works
6. Document the calling convention requirements for StructReturn

