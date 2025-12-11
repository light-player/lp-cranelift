//! File update helper for bless mode.
//!
//! This module provides a helper struct to update test files in-place when
//! expectations don't match, matching Cranelift's FileUpdate semantics.

use anyhow::{Result, bail};
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use crate::filetest::ComparisonOp;
use lp_glsl::GlslValue;

/// A helper struct to update a file in-place as test expectations are
/// automatically updated.
///
/// This structure automatically handles multiple edits to one file. Our edits
/// are line-based but if editing a previous portion of the file adds lines then
/// all future edits need to know to skip over those previous lines. Note that
/// this assumes that edits are done front-to-back.
pub struct FileUpdate {
    path: PathBuf,
    line_diff: Cell<isize>,
    last_update: Cell<usize>,
}

impl FileUpdate {
    /// Create a new FileUpdate for the given path.
    pub fn new(path: &Path) -> Self {
        FileUpdate {
            path: path.to_path_buf(),
            line_diff: Cell::new(0),
            last_update: Cell::new(0),
        }
    }

    /// Update a `// run:` line with a new expected value.
    pub fn update_run_expectation(
        &self,
        line_number: usize,
        new_value: &GlslValue,
        comparison: ComparisonOp,
    ) -> Result<()> {
        // This is required for correctness of this update.
        assert!(line_number > self.last_update.get());
        self.last_update.set(line_number);

        // Read the old test file
        let old_test = fs::read_to_string(&self.path)?;
        let mut new_test = String::new();
        let mut lines = old_test.lines();
        let lines_to_preserve = (((line_number - 1) as isize) + self.line_diff.get()) as usize;

        // Push everything leading up to the run directive
        for _ in 0..lines_to_preserve {
            if let Some(line) = lines.next() {
                new_test.push_str(line);
                new_test.push('\n');
            }
        }

        // Find and update the run directive line
        if let Some(line) = lines.next() {
            if line.trim().starts_with("// run:") {
                // Parse the line to extract the expression part
                let trimmed = line.trim();
                if let Some(run_content) = trimmed.strip_prefix("// run:") {
                    let run_content = run_content.trim();

                    // Extract the expression (everything before == or ~=)
                    let expression = if let Some(pos) = run_content.rfind(" == ") {
                        run_content[..pos].trim()
                    } else if let Some(pos) = run_content.rfind(" ~= ") {
                        run_content[..pos].trim()
                    } else {
                        bail!("invalid run directive format at line {}", line_number);
                    };

                    // Format the new expected value
                    let expected_str = format_glsl_value(new_value);
                    let op_str = match comparison {
                        ComparisonOp::Exact => "==",
                        ComparisonOp::Approx => "~=",
                    };

                    // Reconstruct the line with proper indentation
                    let indent = line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .collect::<String>();
                    new_test.push_str(&format!(
                        "{}// run: {} {} {}\n",
                        indent, expression, op_str, expected_str
                    ));
                } else {
                    // Malformed run directive, keep original
                    new_test.push_str(line);
                    new_test.push('\n');
                }
            } else {
                // Not a run directive at expected line, keep original line
                new_test.push_str(line);
                new_test.push('\n');
            }
        }

        // Push the rest of the file
        for line in lines {
            new_test.push_str(line);
            new_test.push('\n');
        }

        // Record the difference in line count so future updates can be adjusted
        // accordingly, and then write the file back out to the filesystem.
        let old_line_count = old_test.lines().count();
        let new_line_count = new_test.lines().count();
        self.line_diff
            .set(self.line_diff.get() + (new_line_count as isize - old_line_count as isize));

        fs::write(&self.path, new_test)?;
        Ok(())
    }
}

/// Format a GlslValue as a string for use in test files.
fn format_glsl_value(value: &GlslValue) -> String {
    match value {
        GlslValue::I32(i) => i.to_string(),
        GlslValue::F32(f) => {
            // Format float with enough precision but avoid unnecessary decimals
            if f.fract() == 0.0 {
                format!("{:.1}", f)
            } else {
                format!("{}", f)
            }
        }
        GlslValue::Bool(b) => b.to_string(),
        GlslValue::Vec2(v) => format!("[{}, {}]", v[0], v[1]),
        GlslValue::Vec3(v) => format!("[{}, {}, {}]", v[0], v[1], v[2]),
        GlslValue::Vec4(v) => format!("[{}, {}, {}, {}]", v[0], v[1], v[2], v[3]),
        GlslValue::Mat2x2(m) => {
            format!("[[{}, {}], [{}, {}]]", m[0][0], m[0][1], m[1][0], m[1][1])
        }
        GlslValue::Mat3x3(_) => {
            // For now, just return a placeholder
            "mat3x3(...)".to_string()
        }
        GlslValue::Mat4x4(_) => {
            // For now, just return a placeholder
            "mat4x4(...)".to_string()
        }
    }
}
