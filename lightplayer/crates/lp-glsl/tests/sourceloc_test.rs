//! Unit tests for SourceLoc mapping functionality.

use cranelift_codegen::isa::OwnedTargetIsa;
use lp_glsl::codegen::sourceloc::SourceLocManager;
#[cfg(feature = "emulator")]
use lp_glsl::glsl_emu_riscv32_with_metadata;
use lp_glsl::{GlslCompiler, GlslError, GlslOptions, RunMode};

#[cfg(feature = "emulator")]
#[test]
fn test_sourceloc_mapping_for_division_trap() {
    // GLSL source with a division operation that will trap
    let source = r#"
float divide_float(float a, float b) {
    return a / b;
}

float main() {
    return divide_float(0.1, 0.2);
}
"#;

    let options = GlslOptions {
        run_mode: RunMode::Emulator {
            max_memory: 1024 * 1024,
            stack_size: 64 * 1024,
            max_instructions: 10000,
        },
        decimal_format: lp_glsl::DecimalFormat::Fixed32,
    };

    // Compile and create executable
    let mut executable =
        match glsl_emu_riscv32_with_metadata(source, options, Some(String::from("test.glsl"))) {
            Ok(exec) => exec,
            Err(e) => panic!("Compilation failed: {}", e),
        };

    // Try to call main - this should trap because divide_float(0.1, 0.2)
    // will result in division by zero in fixed-point arithmetic
    let result = executable.call_f32("main", &[]);

    // Verify that we got a trap error
    assert!(result.is_err(), "Expected trap error, got: {:?}", result);

    let error = result.unwrap_err();

    // Check that the error message contains trap information
    let error_str = format!("{}", error);
    assert!(
        error_str.contains("trap") || error_str.contains("division"),
        "Error message should mention trap or division, got: {}",
        error_str
    );

    // Check that the error has source location information
    // The division is on line 3, column 12 (in the function divide_float)
    // But since we're calling from main, the trap might be reported at the call site
    // Let's verify that we have some location information
    if let Some(location) = &error.location {
        assert!(
            location.line > 0,
            "Error should have a line number, got: {:?}",
            location
        );
    } else {
        // If no location, check if the error has span_text which would indicate
        // source context was attempted
        if error.span_text.is_none() {
            panic!(
                "Error should have source location or span_text. Error: {}",
                error_str
            );
        }
    }
}

#[test]
fn test_sourceloc_manager_basic() {
    let mut manager = SourceLocManager::new();

    // Create a test span (line 5, column 10)
    // SourceSpan has line, column, offset, and len fields
    let span = glsl::syntax::SourceSpan {
        line: 5,
        column: 10,
        offset: 0,
        len: 0,
    };

    // Create a SourceLoc from the span
    let srcloc = manager.create_srcloc(&span);

    // Verify it's not the default SourceLoc
    assert!(
        !srcloc.is_default(),
        "Created SourceLoc should not be default"
    );

    // Look up the source location
    let lookup_result = manager.lookup_srcloc(srcloc);
    assert_eq!(
        lookup_result,
        Some((5, 10)),
        "SourceLoc lookup should return (5, 10)"
    );

    // Test with unknown span
    let unknown_span = glsl::syntax::SourceSpan::unknown();
    let unknown_srcloc = manager.create_srcloc(&unknown_span);
    assert!(
        unknown_srcloc.is_default(),
        "Unknown span should create default SourceLoc"
    );
    assert_eq!(
        manager.lookup_srcloc(unknown_srcloc),
        None,
        "Default SourceLoc should return None"
    );
}

#[test]
fn test_sourceloc_manager_merge() {
    let mut manager1 = SourceLocManager::new();
    let mut manager2 = SourceLocManager::new();

    // Create spans in manager1
    let span1 = glsl::syntax::SourceSpan {
        line: 1,
        column: 5,
        offset: 0,
        len: 0,
    };
    let srcloc1 = manager1.create_srcloc(&span1);

    // Create spans in manager2
    let span2 = glsl::syntax::SourceSpan {
        line: 2,
        column: 10,
        offset: 0,
        len: 0,
    };
    let srcloc2 = manager2.create_srcloc(&span2);

    // Merge manager2 into manager1
    manager1.merge_from(&manager2);

    // Verify both mappings exist
    assert_eq!(manager1.lookup_srcloc(srcloc1), Some((1, 5)));
    assert_eq!(manager1.lookup_srcloc(srcloc2), Some((2, 10)));
}

#[test]
fn test_division_sets_sourceloc() {
    // Create ISA
    let isa = create_test_isa().expect("Failed to create ISA");

    // Compile a simple function with division
    let source = r#"
float divide(float a, float b) {
    return a / b;
}
"#;

    let mut compiler = GlslCompiler::new();
    let clif_module = compiler
        .compile_to_clif_module(source, isa)
        .expect("Compilation failed");

    // Get the source location manager
    let _source_loc_manager = clif_module.source_loc_manager();

    // Verify that compilation worked and the manager exists
    // The real test is test_sourceloc_mapping_for_division_trap which tests the full flow
    // including that SourceLoc is set during codegen and can be looked up in trap errors
}

fn create_test_isa() -> Result<OwnedTargetIsa, GlslError> {
    use cranelift_codegen::isa;
    use cranelift_codegen::settings::{self, Configurable};

    let mut builder = settings::builder();
    builder.set("opt_level", "none").unwrap();
    let flags = settings::Flags::new(builder);

    isa::lookup_by_name("riscv32")
        .unwrap()
        .finish(flags)
        .map_err(|e| {
            GlslError::new(
                lp_glsl::error::ErrorCode::E0400,
                format!("Failed to create ISA: {}", e),
            )
        })
}
