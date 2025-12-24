//! SSA Do-While Loop Debugging Tests
//!
//! These tests isolate the do-while loop SSA generation issue by testing
//! cranelift's SSA builder directly, without going through GLSL compilation.

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::types::I32;
use cranelift_codegen::ir::{Function, InstBuilder};
use cranelift_frontend::{Variable, SSABuilder};

/// Test 1: Reproduce the failing do-while loop scenario
///
/// This test mimics our do-while loop structure:
/// - block0: Initialize variables (i=0, sum=0), jump to body_block
/// - body_block: Modify variables (sum = sum + i, i = i + 1), jump to cond_block
/// - cond_block: Use variable i from body_block (i < 5), branch back to body_block or exit
/// - exit_block: Return sum
///
/// Key Issue: When cond_block uses `i` from body_block, it should create a block parameter
/// but creates an alias instead.
///
/// Expected Failure: The test should demonstrate that block parameters aren't created
/// when cond_block uses variables from body_block.
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
    // But wait - we need to declare the predecessor from cond_block first!
    // Actually, the back edge was already declared via brif_cond_body_exit above
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

/// Test 1: Reproduce the failing do-while loop scenario
///
/// This test mimics our do-while loop structure:
/// - block0: Initialize variables (i=0, sum=0), jump to body_block
/// - body_block: Modify variables (sum = sum + i, i = i + 1), jump to cond_block
/// - cond_block: Use variable i from body_block (i < 5), branch back to body_block or exit
/// - exit_block: Return sum
///
/// Key Issue: When cond_block uses `i` from body_block, it should create a block parameter
/// but creates an alias instead.
///
/// Expected Failure: The test should demonstrate that block parameters aren't created
/// when cond_block uses variables from body_block.
#[test]
fn test_do_while_loop_ssa_failing() {
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
    // But wait - we need to declare the predecessor from cond_block first!
    // Actually, the back edge was already declared via brif_cond_body_exit above
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

/// Test 2: Working Cranelift Pattern
///
/// This test copies the exact pattern from `program_with_loop` in cranelift's tests.
/// It shows the correct pattern:
/// - Predecessors are declared BEFORE using variables in the block
/// - Variables are used AFTER predecessor is declared
/// - Block is sealed AFTER all predecessors are declared
#[test]
fn test_do_while_loop_ssa_working() {
    let mut func = Function::new();
    let mut ssa = SSABuilder::default();
    let block0 = func.dfg.make_block();
    let block1 = func.dfg.make_block();
    let block2 = func.dfg.make_block();
    let block3 = func.dfg.make_block();
    {
        let mut cur = FuncCursor::new(&mut func);
        cur.insert_block(block0);
        cur.insert_block(block1);
        cur.insert_block(block2);
        cur.insert_block(block3);
    }
    // Here is the pseudo-program we want to translate:
    // block0:
    //    x = 1;
    //    y = 2;
    //    z = x + y;
    //    jump block1
    // block1:
    //    z = z + y;
    //    brif y, block3, block2;
    // block2:
    //    z = z - x;
    //    return y
    // block3:
    //    y = y - x
    //    jump block1

    // block0
    ssa.declare_block(block0);
    ssa.seal_block(block0, &mut func);
    let x_var = Variable::new(0);
    let x1 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().iconst(I32, 1)
    };
    ssa.def_var(x_var, x1, block0);
    let y_var = Variable::new(1);
    let y1 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().iconst(I32, 2)
    };
    ssa.def_var(y_var, y1, block0);
    let z_var = Variable::new(2);
    let x2 = ssa.use_var(&mut func, x_var, I32, block0).0;
    let y2 = ssa.use_var(&mut func, y_var, I32, block0).0;
    let z1 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().iadd(x2, y2)
    };
    ssa.def_var(z_var, z1, block0);
    let jump_block0_block1 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0);
        cur.ins().jump(block1, &[])
    };
    assert_eq!(ssa.use_var(&mut func, x_var, I32, block0).0, x1);
    assert_eq!(ssa.use_var(&mut func, y_var, I32, block0).0, y1);
    assert_eq!(x2, x1);
    assert_eq!(y2, y1);

    // block1
    ssa.declare_block(block1);
    // KEY: Declare predecessor BEFORE using variables
    ssa.declare_block_predecessor(block1, jump_block0_block1);
    // Now use variables - this creates block parameters correctly
    let z2 = ssa.use_var(&mut func, z_var, I32, block1).0;
    let y3 = ssa.use_var(&mut func, y_var, I32, block1).0;
    let z3 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1);
        cur.ins().iadd(z2, y3)
    };
    ssa.def_var(z_var, z3, block1);
    let y4 = ssa.use_var(&mut func, y_var, I32, block1).0;
    assert_eq!(y4, y3);
    let brif_block1_block3_block2 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1);
        cur.ins().brif(y4, block3, &[], block2, &[])
    };

    // block2
    ssa.declare_block(block2);
    ssa.declare_block_predecessor(block2, brif_block1_block3_block2);
    ssa.seal_block(block2, &mut func);
    let z4 = ssa.use_var(&mut func, z_var, I32, block2).0;
    assert_eq!(z4, z3);
    let x3 = ssa.use_var(&mut func, x_var, I32, block2).0;
    let z5 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2);
        cur.ins().isub(z4, x3)
    };
    ssa.def_var(z_var, z5, block2);
    let y5 = ssa.use_var(&mut func, y_var, I32, block2).0;
    assert_eq!(y5, y3);
    {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2);
        cur.ins().return_(&[y5])
    };

    // block3
    ssa.declare_block(block3);
    ssa.declare_block_predecessor(block3, brif_block1_block3_block2);
    ssa.seal_block(block3, &mut func);
    let y6 = ssa.use_var(&mut func, y_var, I32, block3).0;
    assert_eq!(y6, y3);
    let x4 = ssa.use_var(&mut func, x_var, I32, block3).0;
    assert_eq!(x4, x3);
    let y7 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block3);
        cur.ins().isub(y6, x4)
    };
    ssa.def_var(y_var, y7, block3);
    let jump_block3_block1 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block3);
        cur.ins().jump(block1, &[])
    };

    // block1 after all predecessors have been visited.
    ssa.declare_block_predecessor(block1, jump_block3_block1);
    ssa.seal_block(block1, &mut func);
    assert_eq!(func.dfg.block_params(block1)[0], z2);
    assert_eq!(func.dfg.block_params(block1)[1], y3);
    assert_eq!(func.dfg.resolve_aliases(x3), x1);
}

