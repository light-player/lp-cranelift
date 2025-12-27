//! Identity transform implementation

use crate::backend2::transform::pipeline::{Transform, TransformContext};
use crate::backend2::transform::shared::{
    copy_instruction, copy_stack_slots, copy_value_aliases, create_blocks, map_entry_block_params,
};
use crate::error::{ErrorCode, GlslError};
use alloc::vec::Vec;
use cranelift_codegen::ir::{Block, Function, Inst, Signature, StackSlot, Value};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use hashbrown::HashMap;

/// Identity transform - copies functions exactly without modification
pub struct IdentityTransform;

impl Transform for IdentityTransform {
    fn transform_signature(&self, sig: &Signature) -> Signature {
        sig.clone()
    }

    fn transform_function<M: cranelift_module::Module>(
        &self,
        old_func: &Function,
        _ctx: &mut TransformContext<'_, M>,
    ) -> Result<Function, GlslError> {
        // 1. Create new function with same signature
        let mut new_func = Function::with_name_signature(old_func.name.clone(), old_func.signature.clone());

        // 2. Copy stack slots
        let stack_slot_map = copy_stack_slots(old_func, &mut new_func)?;

        // 3. Create builder
        let mut builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut new_func, &mut builder_ctx);

        // 4. Create maps
        let mut block_map = HashMap::new();
        let mut value_map = HashMap::new();

        // 5. Create blocks and map entry params
        create_blocks(old_func, &mut builder, &mut block_map, &mut value_map)?;

        // 6. Get entry block
        let entry_block = old_func
            .layout
            .entry_block()
            .ok_or_else(|| GlslError::new(ErrorCode::E0301, "Function has no entry block"))?;
        let new_entry_block = block_map[&entry_block];

        // 7. Verify entry block params
        map_entry_block_params(old_func, entry_block, new_entry_block, &mut builder, &value_map)?;

        // 8. Copy all instructions
        for old_block in old_func.layout.blocks() {
            let insts: Vec<Inst> = old_func.layout.block_insts(old_block).collect();

            let new_block = block_map[&old_block];
            builder.switch_to_block(new_block);

            for old_inst in insts {
                copy_instruction(
                    old_func,
                    old_inst,
                    &mut builder,
                    &mut value_map,
                    Some(&stack_slot_map),
                    &block_map,
                )?;
            }
        }

        // 10. Seal all blocks
        builder.seal_all_blocks();

        // 11. Finalize builder
        builder.finalize();

        // 12. Copy value aliases
        copy_value_aliases(old_func, &mut new_func, &value_map)?;

        Ok(new_func)
    }
}
