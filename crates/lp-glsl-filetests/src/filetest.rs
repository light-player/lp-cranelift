//! Run GLSL filetests
//! Pattern: cranelift/filetests/src/runone.rs

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestTarget {
    Host(Option<lp_glsl::FixedPointFormat>),
    Riscv32(Option<lp_glsl::FixedPointFormat>),
}

impl TestTarget {
    pub fn fixed_point_format(&self) -> Option<lp_glsl::FixedPointFormat> {
        match self {
            TestTarget::Host(fmt) | TestTarget::Riscv32(fmt) => *fmt,
        }
    }

    pub fn is_host(&self) -> bool {
        matches!(self, TestTarget::Host(_))
    }

    pub fn is_riscv32(&self) -> bool {
        matches!(self, TestTarget::Riscv32(_))
    }

    pub fn is_fixed64(&self) -> bool {
        matches!(
            self.fixed_point_format(),
            Some(lp_glsl::FixedPointFormat::Fixed32x32)
        )
    }
}

/// Build an ISA for the given test target
pub fn build_isa_for_target(target: TestTarget) -> Result<cranelift_codegen::isa::OwnedTargetIsa> {
    use cranelift_codegen::isa::lookup;
    use cranelift_codegen::settings;
    use cranelift_codegen::settings::Configurable;
    use target_lexicon::Triple;

    let mut flag_builder = settings::builder();
    flag_builder.set("is_pic", "false").expect("set is_pic");
    flag_builder
        .set("use_colocated_libcalls", "false")
        .expect("set use_colocated_libcalls");
    flag_builder
        .set("enable_multi_ret_implicit_sret", "true")
        .expect("set enable_multi_ret_implicit_sret");
    let flags = settings::Flags::new(flag_builder);

    match target {
        TestTarget::Host(_) => {
            let isa_builder = cranelift_native::builder().map_err(|e| {
                anyhow::anyhow!(
                    "Failed to build native ISA: {}. This is a configuration error, not a runtime error.",
                    e
                )
            })?;
            Ok(isa_builder
                .finish(flags)
                .map_err(|e| anyhow::anyhow!("Failed to finish native ISA builder: {}", e))?)
        }
        TestTarget::Riscv32(_) => {
            let triple = Triple {
                architecture: target_lexicon::Architecture::Riscv32(
                    target_lexicon::Riscv32Architecture::Riscv32imac,
                ),
                vendor: target_lexicon::Vendor::Unknown,
                operating_system: target_lexicon::OperatingSystem::None_,
                environment: target_lexicon::Environment::Unknown,
                binary_format: target_lexicon::BinaryFormat::Elf,
            };
            let isa_builder = lookup(triple)
                .map_err(|e| anyhow::anyhow!("Failed to lookup riscv32 ISA: {:?}", e))?;
            Ok(isa_builder
                .finish(flags)
                .map_err(|e| anyhow::anyhow!("Failed to finish riscv32 ISA builder: {}", e))?)
        }
    }
}

pub fn run_filetest(path: &Path) -> Result<()> {
    let source =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    // Parse test directives
    let test_compile = source.contains("test compile");
    let test_run = source.contains("test run");
    let test_error = source.contains("test error");
    let test_fixed32 = source.contains("test fixed32");
    let test_fixed64 = source.contains("test fixed64");

    // Parse target directives
    let mut targets = parse_target_directives(&source)?;

    // Filter out fixed64 targets and emit warnings
    let original_target_count = targets.len();
    targets.retain(|target| !target.is_fixed64());
    let skipped_count = original_target_count - targets.len();
    if skipped_count > 0 {
        eprintln!(
            "warning: Skipping {} fixed64 target(s) in {} - 128-bit operations not yet supported",
            skipped_count,
            path.display()
        );
    }

    // If no targets specified, default to host
    // If all explicitly specified targets were fixed64, still default to host
    let targets = if targets.is_empty() {
        vec![TestTarget::Host(None)]
    } else {
        targets
    };

    // Legacy fixed-point format support (for backward compatibility)
    // If target specifies format, use that; otherwise check legacy directives
    // Note: For compile tests, we still apply the transformation even if targets are filtered out
    // (we just can't run the code, but we can verify the IR transformation)
    let legacy_fixed_point_format = if test_fixed32 {
        Some(lp_glsl::FixedPointFormat::Fixed16x16)
    } else if test_fixed64 {
        // Even if fixed64 targets are filtered out (can't run), we still apply the transformation
        // for compile tests to verify the IR is correct
        Some(lp_glsl::FixedPointFormat::Fixed32x32)
    } else {
        None
    };

    // Extract just the GLSL code (lines that don't start with ; or test directives)
    let glsl_source = extract_glsl_source(&source);

    if test_error {
        crate::test_error::run_test(path, &source, &glsl_source)?;
    }

    if test_compile {
        // For compile tests, use first target or legacy format
        let fixed_point_format = targets
            .first()
            .and_then(|t| t.fixed_point_format())
            .or(legacy_fixed_point_format);
        // If test has fixed32 directive OR target specifies fixed-point format,
        // check CLIF AFTER fixed-point transformation
        // Otherwise, check CLIF BEFORE fixed-point transformation (target-agnostic)
        let check_fixed_point_clif = test_fixed32 || fixed_point_format.is_some();
        crate::test_compile::run_test(
            path,
            &source,
            &glsl_source,
            fixed_point_format,
            check_fixed_point_clif,
        )?;
    }

    if test_run {
        crate::test_run::run_test(path, &source, &glsl_source, &targets)?;
    }

    if !test_compile && !test_run && !test_error {
        anyhow::bail!(
            "No test directives found (expected 'test compile', 'test run', or 'test error')"
        );
    }

    Ok(())
}

pub fn parse_target_directives(source: &str) -> Result<Vec<TestTarget>> {
    let mut targets = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(comment_content) = trimmed.strip_prefix("//") {
            let content = comment_content.trim();
            if let Some(target_spec) = content.strip_prefix("target") {
                let target_spec = target_spec.trim();

                // Parse format: "host", "host.fixed32", "host.fixed64", "riscv32", "riscv32.fixed32", "riscv32.fixed64"
                if let Some((arch, format_str)) = target_spec.split_once('.') {
                    let fixed_point_format = match format_str {
                        "fixed32" => Some(lp_glsl::FixedPointFormat::Fixed16x16),
                        "fixed64" => Some(lp_glsl::FixedPointFormat::Fixed32x32),
                        _ => anyhow::bail!("Unknown fixed-point format: {}", format_str),
                    };

                    match arch.trim() {
                        "host" => targets.push(TestTarget::Host(fixed_point_format)),
                        "riscv32" => targets.push(TestTarget::Riscv32(fixed_point_format)),
                        _ => anyhow::bail!("Unknown target architecture: {}", arch),
                    }
                } else {
                    // No format specified
                    match target_spec.trim() {
                        "host" => targets.push(TestTarget::Host(None)),
                        "riscv32" => targets.push(TestTarget::Riscv32(None)),
                        _ => anyhow::bail!("Unknown target architecture: {}", target_spec),
                    }
                }
            }
        }
    }

    Ok(targets)
}

fn extract_glsl_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Keep all lines that don't start with // test, // target, // CHECK, // run:, or // EXPECT_ERROR:
            // This preserves regular GLSL comments
            if let Some(comment_content) = trimmed.strip_prefix("//") {
                let content = comment_content.trim();
                !content.starts_with("test ")
                    && !content.starts_with("target")
                    && !content.starts_with("CHECK")
                    && !content.starts_with("run:")
                    && !content.starts_with("EXPECT_ERROR:")
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
