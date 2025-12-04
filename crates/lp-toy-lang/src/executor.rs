//! Simple executor for JIT-compiled toy language programs.

use core::mem;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::jit::JIT;

/// Execute a toy language function using the JIT compiler.
///
/// Compiles the function and executes it with the given arguments.
/// Returns the result value from the function.
///
/// # Safety
///
/// This function uses `mem::transmute` to convert the JIT-compiled code
/// pointer into a function pointer. The caller must ensure the input
/// code is valid and the argument types match.
pub fn execute_function<I, O>(jit: &mut JIT, code: &str, input: I) -> Result<O, String> {
    // Compile the code to machine code
    let code_ptr = jit.compile(code)?;

    // Cast the raw pointer to a typed function pointer.
    // SAFETY: This is unsafe because we're trusting that the generated code
    // is safe to be called and matches the function signature.
    let code_fn = unsafe { mem::transmute::<_, fn(I) -> O>(code_ptr) };

    // Call the JIT-compiled function
    Ok(code_fn(input))
}

/// Execute a single-argument toy language function.
pub fn execute_function_1arg(jit: &mut JIT, code: &str, arg: isize) -> Result<isize, String> {
    execute_function(jit, code, arg)
}

/// Execute a two-argument toy language function.
pub fn execute_function_2args(
    jit: &mut JIT,
    code: &str,
    arg1: isize,
    arg2: isize,
) -> Result<isize, String> {
    execute_function(jit, code, (arg1, arg2))
}

/// Execute a zero-argument toy language function.
pub fn execute_function_0args(jit: &mut JIT, code: &str) -> Result<isize, String> {
    execute_function(jit, code, ())
}
