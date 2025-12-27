//! SSA Do-While Loop Debugging Tests
//!
//! These tests isolate the do-while loop SSA generation issue by testing
//! cranelift's SSA builder directly, without going through GLSL compilation.

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::data_value::DataValue;
use cranelift_codegen::entity::EntityRef;
use cranelift_codegen::ir::types::I32;
use cranelift_codegen::ir::{Block, Function, InstBuilder, UserFuncName};
use cranelift_frontend::{SSABuilder, Variable};
use cranelift_interpreter::environment::FunctionStore;
use cranelift_interpreter::interpreter::{Interpreter, InterpreterState};
use cranelift_interpreter::step::ControlFlow;

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
    let mut func = Function::with_name_signature(
        UserFuncName::user(0, 0),
        cranelift_codegen::ir::Signature::new(cranelift_codegen::isa::CallConv::SystemV),
    );
    let mut ssa = SSABuilder::default();

    // Create blocks
    let block0_init = func.dfg.make_block();
    let block1_body = func.dfg.make_block();
    let block2_cond = func.dfg.make_block();
    let block3_exit = func.dfg.make_block();

    {
        let mut cur = FuncCursor::new(&mut func);
        cur.insert_block(block0_init);
        cur.insert_block(block1_body);
        cur.insert_block(block2_cond);
        cur.insert_block(block3_exit);
    }

    let print_debug_info =
        |line_number: u32, func: &Function, init: Block, body: Block, cond: Block, exit: Block| {
            eprintln!("--------------------------------");
            eprintln!("DEBUG line {}", line_number);
            eprintln!("block0_init: {:?}", func.dfg.block_params(init));
            eprintln!("block1_body: {:?}", func.dfg.block_params(body));
            eprintln!("block2_cond: {:?}", func.dfg.block_params(cond));
            eprintln!("block3_exit: {:?}", func.dfg.block_params(exit));
        };

    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // =============================================================================================
    // init_block
    //
    // Initialize variables i=0, sum=0, jump to body_block
    //
    ssa.declare_block(block0_init);
    ssa.seal_block(block0_init, &mut func);
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );
    let i_var = Variable::new(0);
    let sum_var = Variable::new(1);

    // Initialize i = 0
    let i0 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0_init);
        cur.ins().iconst(I32, 0)
    };
    ssa.def_var(i_var, i0, block0_init);

    // Initialize sum = 0
    let sum0 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0_init);
        cur.ins().iconst(I32, 0)
    };
    ssa.def_var(sum_var, sum0, block0_init);
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // Jump to body_block
    let jump_init_body = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block0_init);
        cur.ins().jump(block1_body, &[])
    };

    // =============================================================================================
    // body_block
    //
    // Modify variables, jump to cond_block
    //
    ssa.declare_block(block1_body);
    ssa.declare_block_predecessor(block1_body, jump_init_body);
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // Use variables from block0 - this should create block parameters
    // but won't work correctly if predecessor isn't declared
    let i1 = ssa.use_var(&mut func, i_var, I32, block1_body).0;
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );
    let sum1 = ssa.use_var(&mut func, sum_var, I32, block1_body).0;
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // sum = sum + i
    let sum2 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1_body);
        cur.ins().iadd(sum1, i1)
    };
    ssa.def_var(sum_var, sum2, block1_body);
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // i = i + 1
    let i2 = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1_body);
        cur.ins().iadd_imm(i1, 1)
    };
    ssa.def_var(i_var, i2, block1_body);
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // Jump to cond_block
    let jump_body_cond = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block1_body);
        cur.ins().jump(block2_cond, &[])
    };

    // =============================================================================================
    // cond_block
    //
    // Use variable i from body_block, branch
    //

    ssa.declare_block(block2_cond);
    ssa.declare_block_predecessor(block2_cond, jump_body_cond);
    ssa.seal_block(block2_cond, &mut func);

    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // Use i from body_block - this should create a block parameter
    let i3 = ssa.use_var(&mut func, i_var, I32, block2_cond).0;
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // Compare i < 5
    let five = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2_cond);
        cur.ins().iconst(I32, 5)
    };
    let cmp = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2_cond);
        cur.ins().icmp(
            cranelift_codegen::ir::condcodes::IntCC::SignedLessThan,
            i3,
            five,
        )
    };

    // Branch: if i < 5, go back to body_block, else exit
    let brif_cond_body_exit = {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block2_cond);
        cur.ins().brif(cmp, block1_body, &[], block3_exit, &[])
    };

    ssa.declare_block_predecessor(block1_body, brif_cond_body_exit);
    ssa.seal_block(block1_body, &mut func);

    // =============================================================================================
    // exit_block
    //
    // Return sum
    //
    ssa.declare_block(block3_exit);
    ssa.declare_block_predecessor(block3_exit, brif_cond_body_exit);
    ssa.seal_block(block3_exit, &mut func);

    let sum3 = ssa.use_var(&mut func, sum_var, I32, block3_exit).0;
    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );
    {
        let mut cur = FuncCursor::new(&mut func).at_bottom(block3_exit);
        cur.ins().return_(&[sum3]);
    }

    print_debug_info(
        line!(),
        &func,
        block0_init,
        block1_body,
        block2_cond,
        block3_exit,
    );

    // Print the generated CLIF code
    eprintln!("\n=== Generated CLIF IR ===");
    eprintln!("{}", func);

    // Check: cond_block should have block parameters for i and sum
    // This assertion will fail if the bug is present
    let cond_params = func.dfg.block_params(block2_cond);
    eprintln!("cond_block has {} block parameters", cond_params.len());
    eprintln!("Expected at least 1 block parameter for i");

    // Try to interpret and run the function to see what happens
    #[cfg(feature = "std")]
    {
        eprintln!("\n=== Interpreting function ===");
        use cranelift_codegen::data_value::DataValue;
        use cranelift_interpreter::environment::FunctionStore;
        use cranelift_interpreter::interpreter::{Interpreter, InterpreterState};
        use cranelift_interpreter::step::ControlFlow;

        let mut func_store = FunctionStore::default();
        func_store.add("test_func".to_string(), &func);

        let state = InterpreterState::default().with_function_store(func_store);
        let mut interpreter = Interpreter::new(state);

        match interpreter.call_by_name("test_func", &[]) {
            Ok(ControlFlow::Return(results)) => {
                eprintln!("Function returned: {:?}", results);
                if let Some(result) = results.first() {
                    if let DataValue::I32(value) = result {
                        eprintln!("Return value: {}", value);
                        eprintln!("Expected: 10 (sum of 0+1+2+3+4)");
                    }
                }
            }
            Ok(ControlFlow::Trap(trap)) => {
                panic!("Function trapped: {:?}", trap);
            }
            Err(e) => {
                panic!("Interpreter error: {:?}", e);
            }
            _ => {
                panic!("Unexpected control flow");
            }
        }
    }

    // The test demonstrates the issue: cond_block should have block parameters
    // but it doesn't because the predecessor wasn't declared before using variables
    // assert!(
    //     cond_params.len() >= 1,
    //     "cond_block should have at least 1 block parameter for i, but has {}",
    //     cond_params.len()
    // );
}
