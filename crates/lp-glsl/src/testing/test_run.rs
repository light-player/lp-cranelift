//! Test JIT execution with result verification
//! Pattern: cranelift/filetests/src/test_run.rs

use anyhow::{Context, Result};
use std::vec::Vec;

pub fn run_test(source: &str) -> Result<()> {
    // Parse run directives: ; run: <expected_result>
    let run_directives = parse_run_directives(source)?;
    
    if run_directives.is_empty() {
        anyhow::bail!("No 'run' directives found");
    }
    
    // Compile and execute
    for directive in run_directives {
        match directive.expected_type {
            ExpectedType::Int(expected) => {
                let mut compiler = crate::Compiler::new();
                let func = compiler.compile_int(source)
                    .context("Failed to compile for run test")?;
                let result = func();
                
                if result != expected {
                    anyhow::bail!(
                        "Run test failed: expected {}, got {}",
                        expected,
                        result
                    );
                }
            }
            ExpectedType::Bool(expected) => {
                let mut compiler = crate::Compiler::new();
                let func = compiler.compile_bool(source)
                    .context("Failed to compile for run test")?;
                let result = func();
                let expected_val = if expected { 1 } else { 0 };
                
                if result != expected_val {
                    anyhow::bail!(
                        "Run test failed: expected {}, got {}",
                        expected,
                        result != 0
                    );
                }
            }
        }
    }
    
    Ok(())
}

struct RunDirective {
    expected_type: ExpectedType,
}

enum ExpectedType {
    Int(i32),
    Bool(bool),
}

fn parse_run_directives(source: &str) -> Result<Vec<RunDirective>> {
    let mut directives = Vec::new();
    
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(run_spec) = trimmed.strip_prefix("; run:") {
            let spec = run_spec.trim();
            
            // Parse "== <value>"
            if let Some(expected_str) = spec.strip_prefix("==").map(str::trim) {
                // Try parsing as int
                if let Ok(val) = expected_str.parse::<i32>() {
                    directives.push(RunDirective {
                        expected_type: ExpectedType::Int(val),
                    });
                } 
                // Try parsing as bool
                else if expected_str == "true" {
                    directives.push(RunDirective {
                        expected_type: ExpectedType::Bool(true),
                    });
                } else if expected_str == "false" {
                    directives.push(RunDirective {
                        expected_type: ExpectedType::Bool(false),
                    });
                }
            }
        }
    }
    
    Ok(directives)
}

