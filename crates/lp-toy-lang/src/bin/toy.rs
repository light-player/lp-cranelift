//! Toy language example programs and main.

use lpc_toy_lang::translator::Translator;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--clif" {
        // Output LPIR for test cases
        let mut translator = Translator::new();

        println!("=== max function ===");
        let max_code = r#"
fn max ( a , b ) -> ( result )
{
    if a > b {
        result = a
    } else {
        result = b
    }
}
"#;
        match translator.compile(max_code) {
            Ok(func) => println!("{}", func),
            Err(e) => println!("Error: {}", e),
        }

        println!("\n=== sum function ===");
        let sum_code = r#"
fn sum ( n ) -> ( result )
{
    result = 0
    while n > 0 {
        result = result + n
        n = n - 1
    }
}
"#;
        match translator.compile(sum_code) {
            Ok(func) => println!("{}", func),
            Err(e) => println!("Error: {}", e),
        }
        return;
    }

    // Simple test program
    let program = r#"
fn test ( a ) -> ( result )
{
    result = a
}
"#;

    let mut translator = Translator::new();
    match translator.compile(program) {
        Ok(func) => {
            println!("Compiled successfully!");
            println!("Function: {}", func.name());
            println!("Blocks: {}", func.block_count());

            // Try to execute it
            match lpc_toy_lang::executor::execute_function(func, &[5]) {
                Ok(result) => {
                    println!("Execution result: {}", result);
                }
                Err(e) => {
                    eprintln!("Execution error: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            std::process::exit(1);
        }
    }
}
