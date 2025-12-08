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

