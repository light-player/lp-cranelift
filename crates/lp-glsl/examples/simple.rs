//! Simple example of compiling and executing GLSL code

use lp_glsl::Compiler;

fn main() {
    let mut compiler = Compiler::new();

    // Example 1: Integer arithmetic
    let shader1 = r#"
        int main() {
            int a = 10;
            int b = 32;
            return a + b;
        }
    "#;

    let func1 = compiler.compile_int(shader1).unwrap();
    let result1 = func1();
    println!("Example 1 - Integer arithmetic: {} (expected: 42)", result1);
    assert_eq!(result1, 42);

    // Example 2: Boolean comparison
    let shader2 = r#"
        bool main() {
            int x = 5;
            int y = 10;
            return x < y;
        }
    "#;

    let func2 = compiler.compile_bool(shader2).unwrap();
    let result2 = func2();
    println!(
        "Example 2 - Boolean comparison: {} (expected: 1 for true)",
        result2
    );
    assert_eq!(result2, 1);

    // Example 3: Complex expression
    let shader3 = r#"
        int main() {
            int a = 5;
            int b = 3;
            int c = 2;
            return (a + b) * c - 4;
        }
    "#;

    let func3 = compiler.compile_int(shader3).unwrap();
    let result3 = func3();
    println!(
        "Example 3 - Complex expression: {} (expected: 12)",
        result3
    );
    assert_eq!(result3, 12); // (5 + 3) * 2 - 4 = 12

    println!("\nAll examples passed!");
}

