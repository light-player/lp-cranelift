//! Low-level StructReturn calling functions.
//!
//! These functions handle platform-specific calling conventions for
//! calling JIT-compiled functions that use StructReturn.

#[cfg(not(feature = "std"))]
extern crate alloc;
use crate::error::JitCallError;
use cranelift_codegen::ir::{Type, types};
use cranelift_codegen::isa::CallConv;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

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
    validate_call_args(func_ptr, buffer as *mut u8, buffer_size, pointer_type)?;

    // Dispatch to platform-specific implementation
    #[cfg(target_arch = "aarch64")]
    {
        return match (call_conv, pointer_type) {
            (CallConv::AppleAarch64, types::I64) => unsafe {
                call_structreturn_arm64_apple(func_ptr, buffer as *mut u8, buffer_size)
            },
            (CallConv::SystemV, types::I64) => unsafe {
                call_structreturn_arm64_systemv(func_ptr, buffer as *mut u8, buffer_size)
            },
            _ => Err(JitCallError::UnsupportedCallingConvention {
                call_conv,
                pointer_type,
            }),
        };
    }

    #[cfg(target_arch = "riscv32")]
    {
        return match (call_conv, pointer_type) {
            (CallConv::SystemV, types::I32) => unsafe {
                call_structreturn_riscv32(func_ptr, buffer as *mut u8, buffer_size)
            },
            _ => Err(JitCallError::UnsupportedCallingConvention {
                call_conv,
                pointer_type,
            }),
        };
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv32")))]
    {
        let _ = (func_ptr, buffer, buffer_size);
        return Err(JitCallError::UnsupportedCallingConvention {
            call_conv,
            pointer_type,
        });
    }
}

#[cfg(target_arch = "aarch64")]
unsafe fn call_structreturn_arm64_apple(
    func_ptr: *const u8,
    buffer: *mut u8,
    _buffer_size: usize,
) -> Result<(), JitCallError> {
    use core::arch::asm;

    // On AppleAarch64, StructReturn uses x8 register
    // We need to use inline assembly to pass the parameter in x8
    // blr expects the function address in a register (we'll use x9 as temp)
    unsafe {
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

    Ok(())
}

#[cfg(target_arch = "aarch64")]
unsafe fn call_structreturn_arm64_systemv(
    func_ptr: *const u8,
    buffer: *mut u8,
    _buffer_size: usize,
) -> Result<(), JitCallError> {
    // SystemV on ARM64 also uses x8 for StructReturn
    unsafe { call_structreturn_arm64_apple(func_ptr, buffer, _buffer_size) }
}

#[cfg(target_arch = "riscv32")]
unsafe fn call_structreturn_riscv32(
    func_ptr: *const u8,
    buffer: *mut u8,
    _buffer_size: usize,
) -> Result<(), JitCallError> {
    // RISC-V32 SystemV uses first argument register (a0) for StructReturn
    // This matches Rust's extern "C" calling convention
    unsafe {
        let func: extern "C" fn(*mut u8) = core::mem::transmute(func_ptr);
        func(buffer);
    }
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
    let actual_width = if cfg!(target_pointer_width = "32") {
        "32"
    } else if cfg!(target_pointer_width = "64") {
        "64"
    } else {
        "unknown"
    };

    match pointer_type {
        types::I32 if cfg!(target_pointer_width = "32") => Ok(()),
        types::I64 if cfg!(target_pointer_width = "64") => Ok(()),
        _ => Err(JitCallError::PointerTypeMismatch {
            expected: pointer_type,
            actual_pointer_width: String::from(actual_width),
        }),
    }
}
