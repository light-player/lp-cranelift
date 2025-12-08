# Abstract StructReturn Calling Convention Wrapper

## Problem Statement

Currently, calling JIT-compiled functions with StructReturn requires platform-specific inline assembly to match the ABI requirements:
- ARM64 AppleAarch64: StructReturn pointer must be in x8 register
- ARM64 SystemV: StructReturn pointer must be in x8 register  
- RISC-V32 SystemV: StructReturn pointer must be in a0 register (first argument)

This magic is scattered and hard to maintain. We need a clean abstraction that:
1. Automatically detects the correct calling convention
2. Handles register assignment correctly
3. Works across different ISAs
4. Is type-safe and well-tested

## Design Goals

1. **Clean API**: Simple function call that "just works"
2. **Type Safety**: Compile-time guarantees where possible
3. **Testability**: Easy to test with mock functions
4. **Maintainability**: Clear separation of concerns
5. **Performance**: Minimal overhead (ideally zero-cost abstraction)

## Architecture

### Location

**Decision: Create `crates/lp-jit-util/`** - a new utility crate for JIT function calling and wrapping.

**Rationale:**
- Centralized location for all JIT-related utilities
- Handles function wrapping, calling conventions, and StructReturn
- Reusable across LP crates (`lp-glsl`, test apps, etc.)
- Follows `lp-*` naming convention

**Workspace Integration:**
- Add to `Cargo.toml` workspace members (around line 190)
- Place alongside other LP crates: `lp-glsl`, `lp-riscv-shared`, `lp-riscv-tools`

### API Design

The crate provides utilities for:
1. **Calling StructReturn functions** - Handle platform-specific calling conventions
2. **Wrapping StructReturn functions** - Create Rust-friendly wrappers that hide the complexity
3. **Function signature detection** - Automatically determine calling convention from signature

```rust
// Core calling function - handles platform-specific calling conventions
pub unsafe fn call_structreturn<T>(
    func_ptr: *const u8,
    buffer: *mut T,
    buffer_size: usize,
    call_conv: CallConv,
    pointer_type: Type,
) -> Result<(), JitCallError>
where
    T: Copy;

// Function wrapper - creates a Box<dyn Fn() -> Vec<T>> from StructReturn function
pub struct StructReturnWrapper<T> {
    func_ptr: *const u8,
    buffer_size: usize,
    call_conv: CallConv,
    pointer_type: Type,
    _phantom: PhantomData<T>,
}

impl<T> StructReturnWrapper<T>
where
    T: Copy + Default,
{
    pub fn new(
        func_ptr: *const u8,
        buffer_size: usize,
        call_conv: CallConv,
        pointer_type: Type,
    ) -> Result<Self, JitCallError>;
    
    pub fn call(&self) -> Vec<T>;
    
    pub fn into_boxed_fn(self) -> Box<dyn Fn() -> Vec<T>>;
}

// Convenience function to create wrapped function
pub fn wrap_structreturn_function<T>(
    func_ptr: *const u8,
    buffer_size: usize,
    call_conv: CallConv,
    pointer_type: Type,
) -> Result<Box<dyn Fn() -> Vec<T>>, JitCallError>
where
    T: Copy + Default;
```

### Implementation Strategy

1. **Calling Convention Detection**
   - Use `CallConv` enum from Cranelift
   - Map to platform-specific calling convention handler

2. **Platform-Specific Handlers**
   - ARM64 AppleAarch64: Inline assembly to move buffer to x8
   - ARM64 SystemV: Inline assembly to move buffer to x8
   - RISC-V32 SystemV: Use regular function call (StructReturn in a0 = first arg)
   - Others: Panic with helpful error message

3. **Type Safety**
   - Generic over buffer element type
   - Validate pointer types match ISA expectations
   - Ensure buffer size matches expected return size

## Implementation Plan

### Phase 1: Create Crate Structure

**Update: `Cargo.toml` (workspace root)**
Add to workspace members (around line 190):
```toml
  "crates/lp-jit-util",
```

**File: `crates/lp-jit-util/Cargo.toml`**
```toml
[package]
name = "lp-jit-util"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
cranelift-codegen = { workspace = true, default-features = false }
target-lexicon = { workspace = true }
```

**File: `crates/lp-jit-util/src/lib.rs`**
```rust
//! Utilities for calling and wrapping JIT-compiled functions.
//!
//! This crate provides abstractions for:
//! - Calling StructReturn functions with correct calling conventions
//! - Wrapping StructReturn functions into Rust-friendly closures
//! - Handling platform-specific ABI requirements

pub mod call;
pub mod wrapper;
pub mod error;

pub use call::call_structreturn;
pub use wrapper::{StructReturnWrapper, wrap_structreturn_function};
pub use error::JitCallError;
```

**File: `crates/lp-jit-util/src/call.rs`**
- Core StructReturn calling implementation
- Platform-specific calling logic

**File: `crates/lp-jit-util/src/wrapper.rs`**
- Function wrapping utilities
- Box<dyn Fn() -> Vec<T>> creation

**File: `crates/lp-jit-util/src/error.rs`**
- Error types
- Error handling

### Phase 2: Core Implementation

**File: `crates/lp-jit-util/src/call.rs`**

```rust
//! Low-level StructReturn calling functions.
//!
//! These functions handle platform-specific calling conventions for
//! calling JIT-compiled functions that use StructReturn.

use cranelift_codegen::isa::CallConv;
use cranelift_codegen::ir::Type;
use crate::error::JitCallError;

/// Call a JIT-compiled function that uses StructReturn.
///
/// # Safety
/// - `func_ptr` must be a valid function pointer to a JIT-compiled function
/// - `buffer` must point to valid, writable memory of at least `buffer_size` bytes
/// - The function signature must match: `fn(*mut T) -> ()` where T is the element type
/// - The calling convention must match the one used when compiling the function
pub unsafe fn call_structreturn<T>(
    func_ptr: *const u8,
    buffer: *mut T,
    buffer_size: usize,
    call_conv: CallConv,
    pointer_type: Type,
) -> Result<(), JitCallError>
where
    T: Copy,
{
    // Validate inputs
    validate_call_args(func_ptr, buffer, buffer_size, pointer_type)?;
    
    // Dispatch to platform-specific implementation
    match (call_conv, pointer_type) {
        (CallConv::AppleAarch64, Type::I64) => {
            call_structreturn_arm64_apple(func_ptr, buffer as *mut u8, buffer_size)
        }
        (CallConv::SystemV, Type::I64) if cfg!(target_arch = "aarch64") => {
            call_structreturn_arm64_systemv(func_ptr, buffer as *mut u8, buffer_size)
        }
        (CallConv::SystemV, Type::I32) if cfg!(target_arch = "riscv32") => {
            call_structreturn_riscv32(func_ptr, buffer as *mut u8, buffer_size)
        }
        _ => Err(JitCallError::UnsupportedCallingConvention {
            call_conv,
            pointer_type,
        }),
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn call_structreturn_arm64_apple(
    func_ptr: *const u8,
    buffer: *mut u8,
    _buffer_size: usize,
) -> Result<(), JitCallError> {
    use std::arch::asm;
    
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
    
    Ok(())
}

#[cfg(target_arch = "aarch64")]
unsafe fn call_structreturn_arm64_systemv(
    func_ptr: *const u8,
    buffer: *mut u8,
    _buffer_size: usize,
) -> Result<(), JitCallError> {
    // SystemV on ARM64 also uses x8 for StructReturn
    call_structreturn_arm64_apple(func_ptr, buffer, _buffer_size)
}

#[cfg(target_arch = "riscv32")]
unsafe fn call_structreturn_riscv32(
    func_ptr: *const u8,
    buffer: *mut u8,
    _buffer_size: usize,
) -> Result<(), JitCallError> {
    // RISC-V32 SystemV uses first argument register (a0) for StructReturn
    // This matches Rust's extern "C" calling convention
    let func: extern "C" fn(*mut u8) = std::mem::transmute(func_ptr);
    func(buffer);
    Ok(())
}

fn validate_call_args(
    func_ptr: *const u8,
    buffer: *mut u8,
    buffer_size: usize,
    pointer_type: Type,
) -> Result<(), JitCallError> {
    if func_ptr.is_null() {
        return Err(JitCallError::NullFunctionPointer);
    }
    
    if buffer.is_null() {
        return Err(JitCallError::NullBuffer);
    }
    
    if buffer_size == 0 {
        return Err(JitCallError::ZeroBufferSize);
    }
    
    // Validate pointer type matches platform
    match pointer_type {
        Type::I32 if cfg!(target_pointer_width = "32") => Ok(()),
        Type::I64 if cfg!(target_pointer_width = "64") => Ok(()),
        _ => Err(JitCallError::PointerTypeMismatch {
            expected: pointer_type,
            actual_pointer_width: cfg!(target_pointer_width),
        }),
    }
}
```

**File: `crates/lp-jit-util/src/error.rs`**

```rust
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::ir::Type;
use std::fmt;

#[derive(Debug, Clone)]
pub enum JitCallError {
    NullFunctionPointer,
    NullBuffer,
    ZeroBufferSize,
    PointerTypeMismatch {
        expected: Type,
        actual_pointer_width: &'static str,
    },
    UnsupportedCallingConvention {
        call_conv: CallConv,
        pointer_type: Type,
    },
}

impl fmt::Display for JitCallError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JitCallError::NullFunctionPointer => {
                write!(f, "Function pointer is null")
            }
            JitCallError::NullBuffer => {
                write!(f, "Buffer pointer is null")
            }
            JitCallError::ZeroBufferSize => {
                write!(f, "Buffer size must be greater than zero")
            }
            JitCallError::PointerTypeMismatch { expected, actual_pointer_width } => {
                write!(
                    f,
                    "Pointer type mismatch: expected {:?} but platform pointer width is {}",
                    expected, actual_pointer_width
                )
            }
            JitCallError::UnsupportedCallingConvention { call_conv, pointer_type } => {
                write!(
                    f,
                    "Unsupported calling convention {:?} with pointer type {:?}",
                    call_conv, pointer_type
                )
            }
        }
    }
}

impl std::error::Error for JitCallError {}
```

### Phase 3: Higher-Level API (Optional)

**File: `crates/lp-jit-util/src/wrapper.rs`**

```rust
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::ir::Type;
use crate::call::call_structreturn;
use crate::error::JitCallError;
use std::marker::PhantomData;

/// A wrapper for a StructReturn function that provides a Rust-friendly interface.
///
/// This wrapper handles buffer allocation and calling convention details,
/// presenting a simple `Fn() -> Vec<T>` interface.
pub struct StructReturnWrapper<T> {
    func_ptr: *const u8,
    buffer_size: usize,
    call_conv: CallConv,
    pointer_type: Type,
    _phantom: PhantomData<T>,
}

impl<T> StructReturnWrapper<T>
where
    T: Copy + Default,
{
    /// Create a new wrapper for a StructReturn function.
    ///
    /// # Safety
    /// - `func_ptr` must be a valid function pointer to a JIT-compiled function
    /// - The function signature must match: `fn(*mut T) -> ()` with StructReturn
    /// - The calling convention must match the one used when compiling the function
    pub unsafe fn new(
        func_ptr: *const u8,
        buffer_size: usize,
        call_conv: CallConv,
        pointer_type: Type,
    ) -> Result<Self, JitCallError> {
        // Validate inputs
        if func_ptr.is_null() {
            return Err(JitCallError::NullFunctionPointer);
        }
        if buffer_size == 0 {
            return Err(JitCallError::ZeroBufferSize);
        }
        
        Ok(Self {
            func_ptr,
            buffer_size,
            call_conv,
            pointer_type,
            _phantom: PhantomData,
        })
    }
    
    /// Call the wrapped function and return the result.
    pub fn call(&self) -> Vec<T> {
        let mut buffer = vec![T::default(); self.buffer_size];
        
        unsafe {
            call_structreturn(
                self.func_ptr,
                buffer.as_mut_ptr(),
                self.buffer_size,
                self.call_conv,
                self.pointer_type,
            ).expect("StructReturn call failed");
        }
        
        buffer
    }
}

impl<T> Clone for StructReturnWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            func_ptr: self.func_ptr,
            buffer_size: self.buffer_size,
            call_conv: self.call_conv,
            pointer_type: self.pointer_type,
            _phantom: PhantomData,
        }
    }
}

/// Convenience function to create a boxed closure from a StructReturn function.
///
/// This is the primary API for wrapping StructReturn functions.
pub fn wrap_structreturn_function<T>(
    func_ptr: *const u8,
    buffer_size: usize,
    call_conv: CallConv,
    pointer_type: Type,
) -> Result<Box<dyn Fn() -> Vec<T>>, JitCallError>
where
    T: Copy + Default + 'static,
{
    let wrapper = unsafe {
        StructReturnWrapper::new(func_ptr, buffer_size, call_conv, pointer_type)?
    };
    
    Ok(Box::new(move || wrapper.call()))
}
```

### Phase 4: Update Test App

**Update: `apps/test-structreturn/src/main.rs`**

Replace the inline assembly and platform-specific calling code with the new utility:

```rust
// Before:
#[cfg(target_arch = "aarch64")]
unsafe fn call_structreturn_apple_aarch64(func_ptr: *const u8, buffer: *mut f32) {
    asm!(...);
}

// After:
use lp_jit_util::call_structreturn;

// In test function:
unsafe {
    call_structreturn(
        code_ptr,
        buffer.as_mut_ptr(),
        buffer_size,
        call_conv,
        pointer_type,
    )?;
}
```

**Changes:**
1. Remove `call_structreturn_apple_aarch64` and platform-specific functions
2. Remove `use std::arch::asm;` import
3. Replace all platform-specific calling code with `lp_jit_util::call_structreturn`
4. Update error handling to use `JitCallError` from the utility
5. Simplify the calling logic - no more `match call_conv` blocks
6. Add `lp-jit-util` dependency to `apps/test-structreturn/Cargo.toml`

**Update: `apps/test-structreturn/Cargo.toml`**
```toml
[dependencies]
# ... existing dependencies ...
lp-jit-util = { path = "../../crates/lp-jit-util" }
```

**Benefits:**
- Removes ~50 lines of platform-specific code
- Makes test app cleaner and easier to understand
- Tests the utility crate in real usage
- Demonstrates correct usage for other code

### Phase 5: Integration with GLSL Compiler

**Update: `crates/lp-glsl/src/compiler.rs`**
- Use `lp_jit_util::wrap_structreturn_function` in wrapper functions
- Replace `Box<dyn Fn() -> ...>` creation with utility function
- Remove duplicate calling convention logic

**Update: `crates/lp-glsl/Cargo.toml`**
```toml
[dependencies]
# ... existing dependencies ...
lp-jit-util = { path = "../lp-jit-util" }
```

**Changes:**
1. Find all places where `Box<dyn Fn() -> Vec<f32>>` is created for StructReturn functions
2. Replace with `lp_jit_util::wrap_structreturn_function`
3. Remove inline assembly or platform-specific calling code
4. Simplify error handling

### Phase 3: Comprehensive Tests

**File: `crates/lp-jit-util/tests/call.rs`**

```rust
use lp_jit_util::call::call_structreturn;
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::ir::Type;

#[test]
fn test_arm64_apple_aarch64() {
    // Create a simple test function that writes to buffer
    // Compile it with AppleAarch64 calling convention
    // Call it using our abstraction
    // Verify results
}

#[test]
fn test_riscv32_systemv() {
    // Similar test for RISC-V32
}

#[test]
fn test_error_handling() {
    // Test null pointer handling
    // Test unsupported calling conventions
    // Test pointer type mismatches
}

#[test]
fn test_buffer_alignment() {
    // Test with different buffer alignments
    // Ensure alignment doesn't break calls
}
```

**File: `crates/lp-jit-util/tests/wrapper.rs`**

```rust
use lp_jit_util::{wrap_structreturn_function, call_structreturn};
use cranelift_codegen::isa::CallConv;
use cranelift_codegen::ir::Type;

#[test]
fn test_wrapper_creates_closure() {
    // Test that wrapper creates a callable closure
    // Verify closure can be called multiple times
    // Verify results are correct
}

#[test]
fn test_wrapper_clone() {
    // Test that wrapper can be cloned
    // Verify cloned wrapper works independently
}
```

**File: `crates/lp-jit-util/tests/integration.rs`**

```rust
// Integration tests that use actual Cranelift compilation
// Test end-to-end: compile CLIF -> call via abstraction -> verify results
```

## File Structure

```
crates/lp-jit-util/
├── Cargo.toml
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── call.rs                # Core StructReturn calling logic
│   ├── wrapper.rs             # Function wrapping utilities
│   └── error.rs               # Error types
└── tests/
    ├── call.rs                # Unit tests for calling
    ├── wrapper.rs             # Unit tests for wrapping
    └── integration.rs         # Integration tests with Cranelift
```

## Testing Strategy

### Unit Tests
1. **Error Handling**: Test all error paths
2. **Platform Detection**: Verify correct platform-specific code paths
3. **Type Validation**: Test pointer type validation logic

### Integration Tests
1. **End-to-End**: Compile CLIF → Call via abstraction → Verify results
2. **Cross-ISA**: Test ARM64 and RISC-V32
3. **Real Functions**: Test with actual vec2/vec3/vec4/mat2/mat3/mat4 returns

### Test Utilities
- Helper functions to create test JIT functions
- Mock calling convention handlers for testing
- Buffer validation utilities

## Success Criteria

1. ✅ Clean API that hides platform-specific details
2. ✅ Works correctly on ARM64 (AppleAarch64 and SystemV)
3. ✅ Works correctly on RISC-V32 (SystemV)
4. ✅ Comprehensive test coverage (>90%)
5. ✅ Zero runtime overhead (inline assembly, no allocations)
6. ✅ Clear error messages for unsupported configurations
7. ✅ Well-documented with examples

## Migration Path

1. **Phase 1**: Create crate and basic implementation
2. **Phase 2**: Add tests and verify correctness
3. **Phase 3**: Update test-structreturn app to use the utility
4. **Phase 4**: Integrate into lp-glsl compiler
5. **Phase 5**: Remove old inline assembly code from all locations
6. **Phase 6**: Verify all tests pass, including filetests

## Future Enhancements

1. **Support More ISAs**: x86-64, other RISC-V variants
2. **Performance Profiling**: Ensure zero-cost abstraction
3. **Calling Convention Detection**: Auto-detect from function signature
4. **Type-Safe Wrappers**: Generate type-safe wrappers at compile time

## Open Questions

1. **Crate Name**: `lp-jit-util` vs `lp-jit-helpers` vs `lp-jit-call`?
2. **Wrapper API**: Box<dyn Fn()> vs Fn trait object vs custom wrapper struct?
3. **Buffer Management**: Allocate in wrapper vs require caller to provide buffer?
4. **Error Handling**: Result vs panic for unsupported platforms?

## Decision Log

- **Crate Name**: `lp-jit-util` - Clear utility crate name, follows naming convention
- **Location**: New crate - Keeps concerns separated, reusable across LP crates
- **API Style**: Both - Low-level `call_structreturn` for direct calls, high-level `wrap_structreturn_function` for convenience
- **Wrapper API**: Box<dyn Fn() -> Vec<T>> - Matches existing pattern in `lp-glsl/src/compiler.rs`
- **Buffer Management**: Allocate in wrapper - Simplifies API, caller doesn't need to manage buffers
- **Error Handling**: Result - Allows caller to handle gracefully, panic for unsupported platforms in implementation

