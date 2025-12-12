//! Target parsing and configuration.

use crate::test_utils;
use anyhow::Result;
use lp_glsl::{DecimalFormat, RunMode};

/// Parse target string (e.g., "riscv32.fixed32") into run mode and decimal format.
pub fn parse_target(target: &str) -> Result<(RunMode, DecimalFormat)> {
    let parts: Vec<&str> = target.split('.').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "invalid target format: expected '<arch>.<format>', got '{}'",
            target
        );
    }

    let arch = parts[0];
    let format = parts[1];

    let run_mode = match arch {
        "riscv32" => RunMode::Emulator {
            max_memory: test_utils::DEFAULT_MAX_MEMORY,
            stack_size: test_utils::DEFAULT_STACK_SIZE,
            max_instructions: test_utils::DEFAULT_MAX_INSTRUCTIONS,
        },
        _ => anyhow::bail!("unsupported architecture: {}", arch),
    };

    let decimal_format = match format {
        "fixed32" => DecimalFormat::Fixed32,
        "float" => DecimalFormat::Float,
        _ => anyhow::bail!("unsupported format: {}", format),
    };

    Ok((run_mode, decimal_format))
}