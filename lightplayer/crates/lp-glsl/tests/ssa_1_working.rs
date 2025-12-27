//! SSA Do-While Loop Debugging Tests
//!
//! These tests isolate the do-while loop SSA generation issue by testing
//! cranelift's SSA builder directly, without going through GLSL compilation.

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::types::I32;
use cranelift_codegen::ir::{Block, Function, InstBuilder};
use cranelift_frontend::{SSABuilder, Variable};

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

    let print_debug_info = |line_number: u32,
                            func: &Function,
                            block0: Block,
                            block1: Block,
                            block2: Block,
                            block3: Block| {
        eprintln!("--------------------------------");
        eprintln!("DEBUG line {}", line_number);
        eprintln!("block0: {:?}", func.dfg.block_params(block0));
        eprintln!("block1: {:?}", func.dfg.block_params(block1));
        eprintln!("block2: {:?}", func.dfg.block_params(block2));
        eprintln!("block3: {:?}", func.dfg.block_params(block3));
    };

    print_debug_info(line!(), &func, block0, block1, block2, block3);

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
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    // Now use variables - this creates block parameters correctly
    let z2 = ssa.use_var(&mut func, z_var, I32, block1).0;
    let y3 = ssa.use_var(&mut func, y_var, I32, block1).0;
    let z3 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1);
        cur.ins().iadd(z2, y3)
    };
    ssa.def_var(z_var, z3, block1);
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    let y4 = ssa.use_var(&mut func, y_var, I32, block1).0;
    assert_eq!(y4, y3);
    let brif_block1_block3_block2 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1);
        cur.ins().brif(y4, block3, &[], block2, &[])
    };
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    // block2
    ssa.declare_block(block2);
    ssa.declare_block_predecessor(block2, brif_block1_block3_block2);
    ssa.seal_block(block2, &mut func);
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    let z4 = ssa.use_var(&mut func, z_var, I32, block2).0;
    assert_eq!(z4, z3);
    let x3 = ssa.use_var(&mut func, x_var, I32, block2).0;
    let z5 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2);
        cur.ins().isub(z4, x3)
    };
    ssa.def_var(z_var, z5, block2);
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    let y5 = ssa.use_var(&mut func, y_var, I32, block2).0;
    assert_eq!(y5, y3);
    {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2);
        cur.ins().return_(&[y5])
    };
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    // block3
    ssa.declare_block(block3);
    ssa.declare_block_predecessor(block3, brif_block1_block3_block2);
    ssa.seal_block(block3, &mut func);
    print_debug_info(line!(), &func, block0, block1, block2, block3);
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
    print_debug_info(line!(), &func, block0, block1, block2, block3);
    // block1 after all predecessors have been visited.
    ssa.declare_block_predecessor(block1, jump_block3_block1);
    ssa.seal_block(block1, &mut func);
    assert_eq!(func.dfg.block_params(block1)[0], z2);
    assert_eq!(func.dfg.block_params(block1)[1], y3);
    assert_eq!(func.dfg.resolve_aliases(x3), x1);

    // Print the generated CLIF code
    eprintln!("\n=== Generated CLIF IR ===");
    eprintln!("{}", func);
}
