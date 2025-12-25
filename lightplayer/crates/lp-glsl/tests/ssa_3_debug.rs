//! SSA Do-While Loop Debugging Tests
//!
//! These tests isolate the do-while loop SSA generation issue by testing
//! cranelift's SSA builder directly, without going through GLSL compilation.

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::types::I32;
use cranelift_codegen::ir::{Function, InstBuilder};
use cranelift_frontend::{Variable, SSABuilder};

/// Test: Debug version - trying to fix the failing scenario
///
/// This test mimics our do-while loop structure:
/// - block0: Initialize variables (i=0, sum=0), jump to body_block
/// - body_block: Modify variables (sum = sum + i, i = i + 1), jump to cond_block
/// - cond_block: Use variable i from body_block (i < 5), branch back to body_block or exit
/// - exit_block: Return sum
///
/// Changes made based on working test:
/// 1. Declare predecessor for body_block before using variables
/// 2. Declare predecessor for cond_block before using variables
///
/// Still investigating: Why cond_block still has 0 block parameters?
#[test]
fn test_do_while_loop_ssa_debug() {
    let mut func = Function::new();
    let mut ssa = SSABuilder::default();

    // Create blocks
    let block0 = func.dfg.make_block();
    let body_block = func.dfg.make_block();
    let cond_block = func.dfg.make_block();
    let exit_block = func.dfg.make_block();

    {
        let mut cur = FuncCursor::new(&mut func);
        cur.insert_block(block0);
        cur.insert_block(body_block);
        cur.insert_block(cond_block);
        cur.insert_block(exit_block);
    }

    // block0: Initialize variables i=0, sum=0, jump to body_block
    ssa.declare_block(block0);
    ssa.seal_block(block0, &mut func);

    let i_var = Variable::new(0);
    let sum_var = Variable::new(1);

    // Initialize i = 0
    let i0 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().iconst(I32, 0)
    };
    ssa.def_var(i_var, i0, block0);

    // Initialize sum = 0
    let sum0 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().iconst(I32, 0)
    };
    ssa.def_var(sum_var, sum0, block0);

    // Jump to body_block
    let _jump_block0_body = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().jump(body_block, &[])
    };

    // body_block: Modify variables, jump to cond_block
    ssa.declare_block(body_block);
    // KEY ISSUE: We don't declare the predecessor here before using variables
    // This mimics our current code where we switch to body_block and use variables
    // without declaring the predecessor first

    // Use variables from block0 - this should create block parameters
    // but won't work correctly if predecessor isn't declared
    let i1 = ssa.use_var(&mut func, i_var, I32, body_block).0;
    let sum1 = ssa.use_var(&mut func, sum_var, I32, body_block).0;

    // sum = sum + i
    let sum2 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(body_block);
        cur.ins().iadd(sum1, i1)
    };
    ssa.def_var(sum_var, sum2, body_block);

    // i = i + 1
    let i2 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(body_block);
        cur.ins().iadd_imm(i1, 1)
    };
    ssa.def_var(i_var, i2, body_block);

    // Jump to cond_block
    let _jump_body_cond = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(body_block);
        cur.ins().jump(cond_block, &[])
    };

    // cond_block: Use variable i from body_block, branch
    ssa.declare_block(cond_block);
    // KEY ISSUE: We don't declare the predecessor here before using variables
    // This is where the bug manifests - we switch to cond_block and use i
    // but the predecessor hasn't been declared yet, so it creates an alias instead
    // of a block parameter

    // Use i from body_block - this should create a block parameter
    let i3 = ssa.use_var(&mut func, i_var, I32, cond_block).0;

    // Compare i < 5
    let five = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(cond_block);
        cur.ins().iconst(I32, 5)
    };
    let cmp = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(cond_block);
        cur.ins().icmp(cranelift_codegen::ir::condcodes::IntCC::SignedLessThan, i3, five)
    };

    // Branch: if i < 5, go back to body_block, else exit
    let brif_cond_body_exit = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(cond_block);
        cur.ins().brif(cmp, body_block, &[], exit_block, &[])
    };

    // exit_block: Return sum
    ssa.declare_block(exit_block);
    ssa.declare_block_predecessor(exit_block, brif_cond_body_exit);
    ssa.seal_block(exit_block, &mut func);

    let sum3 = ssa.use_var(&mut func, sum_var, I32, exit_block).0;
    {
        let mut cur = FuncCursor::new(&mut func).at_bottom(exit_block);
        cur.ins().return_(&[sum3]);
    }

    // Now seal cond_block first (mimicking our current code)
    ssa.seal_block(cond_block, &mut func);

    // Then seal body_block (mimicking our current code)
    ssa.seal_block(body_block, &mut func);

    // Check: cond_block should have block parameters for i and sum
    // This assertion will fail if the bug is present
    let cond_params = func.dfg.block_params(cond_block);
    eprintln!("cond_block has {} block parameters", cond_params.len());
    eprintln!("Expected at least 1 block parameter for i");

    // The test demonstrates the issue: cond_block should have block parameters
    // but it doesn't because the predecessor wasn't declared before using variables
    assert!(
        cond_params.len() >= 1,
        "cond_block should have at least 1 block parameter for i, but has {}",
        cond_params.len()
    );
}