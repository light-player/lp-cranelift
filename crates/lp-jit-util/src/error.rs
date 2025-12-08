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
        actual_pointer_width: String,
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

