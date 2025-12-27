use crate::backend2::transform::identity::IdentityTransform;
use cranelift_codegen::write_function;
use cranelift_module::Linkage;
use cranelift_reader::{ParseOptions, parse_test};
use std::prelude::rust_2015::{String, Vec};

/// Normalize CLIF strings for comparison
fn normalize_clif(clif: &str) -> String {
    clif.lines()
        .map(|line| {
            let line = if let Some(comment_pos) = line.find(';') {
                &line[..comment_pos]
            } else {
                line
            };
            line.trim()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format all functions from a GlModule as CLIF text
fn format_module<M: cranelift_module::Module>(
    module: &crate::backend2::module::gl_module::GlModule<M>,
) -> String {
    let mut result = String::new();
    // Sort functions by name for deterministic output
    let mut funcs: Vec<_> = module.fns.iter().collect();
    funcs.sort_by_key(|(name, _)| *name);
    for (_name, gl_func) in funcs {
        write_function(&mut result, &gl_func.function).unwrap();
        result.push('\n');
    }
    result
}

/// Parse CLIF module input, transform it, and return CLIF strings for comparison
fn parse_and_transform(clif_input: &str) -> (String, String) {
    // Parse the CLIF module
    let test_file =
        parse_test(clif_input, ParseOptions::default()).expect("Failed to parse CLIF module");

    // Build GlModule from parsed functions
    let target = crate::backend2::target::Target::host_jit().unwrap();
    let mut original_module = crate::backend2::module::gl_module::GlModule::<
        cranelift_jit::JITModule,
    >::new_jit(target.clone())
    .unwrap();

    // Add all functions to the module
    for (func, _) in test_file.functions {
        let func_name = format!("{}", func.name);
        // Remove leading % if present
        let func_name = func_name.strip_prefix('%').unwrap_or(&func_name);
        original_module
            .add_function(func_name, Linkage::Local, func.signature.clone(), func)
            .expect("Failed to add function to module");
    }

    // Format the parsed module (before transformation)
    let parsed_buf = format_module(&original_module);

    // Transform the whole module
    let transform = IdentityTransform;
    let transformed_module = original_module
        .apply_transform(transform)
        .expect("Failed to apply identity transform");

    // Format the transformed module
    let transformed_buf = format_module(&transformed_module);

    (parsed_buf, transformed_buf)
}

/// Assert that identity transform produces identical CLIF output
pub fn assert_identity_transform(message: &str, clif_input: &str) {
    let (parsed_buf, transformed_buf) = parse_and_transform(clif_input);

    let normalized_parsed = normalize_clif(&parsed_buf);
    let normalized_transformed = normalize_clif(&transformed_buf);

    assert_eq!(
        normalized_parsed, normalized_transformed,
        "{}\n\
     PARSED:\n{}\n\n\
     TRANSFORMED:\n{}",
        message, parsed_buf, transformed_buf
    );
}
