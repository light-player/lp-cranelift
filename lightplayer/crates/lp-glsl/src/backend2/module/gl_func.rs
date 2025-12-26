//! Function metadata structure

use cranelift_codegen::ir::Signature;
use cranelift_module::FuncId;
use alloc::string::String;

/// Function metadata (doesn't store Function IR, just metadata)
pub struct GlFunc {
    pub name: String,
    pub clif_sig: Signature,
    pub func_id: FuncId,
    // Note: GLSL signature not needed for Phase 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use cranelift_codegen::ir::{types, AbiParam};
    use cranelift_codegen::isa::CallConv;

    #[test]
    fn test_gl_func_creation() {
        let mut sig = Signature::new(CallConv::SystemV);
        sig.params.push(AbiParam::new(types::I32));
        sig.returns.push(AbiParam::new(types::I32));
        
        // Note: FuncId creation requires a Module, so this is a minimal test
        // Note: FuncId creation requires a Module, so we can't create a real one in unit tests
        // This test just verifies the structure can be created with valid data
        let _func = GlFunc {
            name: String::from("test"),
            clif_sig: sig,
            func_id: FuncId::from_u32(0), // Dummy ID for test
        };
        
        assert_eq!(_func.name, "test");
        assert_eq!(_func.clif_sig.params.len(), 1);
    }
}
