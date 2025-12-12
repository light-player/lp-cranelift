//! Shared parsing utilities for test file structure.

/// Section boundaries in a test file.
#[derive(Debug, Clone, Copy)]
pub struct SectionBoundaries {
    /// Line index where GLSL source ends (exclusive).
    pub glsl_end: usize,
    /// Line index where run expectations start.
    pub run_start: usize,
}

/// Find section boundaries in a test file.
/// Returns the end of GLSL code and the start of run expectations.
pub fn find_section_boundaries(lines: &[String]) -> SectionBoundaries {
    // Find the end of GLSL code (last non-comment, non-directive line)
    let mut glsl_end = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("// test")
            && !trimmed.starts_with("// target")
            && !trimmed.starts_with("// run:")
        {
            glsl_end = i + 1;
        }
    }

    // Find the start of run expectations (first `// #run:` or `// run:` after GLSL)
    let mut run_start = lines.len();
    for (i, line) in lines.iter().enumerate().skip(glsl_end) {
        let trimmed = line.trim();
        if trimmed.starts_with("// #run:") || trimmed.starts_with("// run:") {
            run_start = i;
            break;
        }
    }

    SectionBoundaries { glsl_end, run_start }
}

/// CLIF section boundaries within the expectations section.
#[derive(Debug, Clone, Copy)]
pub struct ClifSectionBoundaries {
    /// Start of compile section (inclusive).
    pub compile_start: Option<usize>,
    /// End of compile section (exclusive).
    pub compile_end: Option<usize>,
    /// Start of transform section (inclusive).
    pub transform_start: Option<usize>,
    /// End of transform section (exclusive).
    pub transform_end: Option<usize>,
}

/// Find CLIF section boundaries within the expectations section.
/// `expectations_start` and `expectations_end` define the range to search within.
pub fn find_clif_section_boundaries(
    lines: &[String],
    expectations_start: usize,
    expectations_end: usize,
) -> ClifSectionBoundaries {
    let mut compile_start = None;
    let mut compile_end = None;
    let mut transform_start = None;
    let mut transform_end = None;

    for (i, line) in lines.iter().enumerate().skip(expectations_start).take(expectations_end - expectations_start) {
        let trimmed = line.trim();
        if trimmed.starts_with("// #compile:") {
            compile_start = Some(i);
        } else if trimmed.starts_with("// #transform:") {
            if compile_start.is_some() && compile_end.is_none() {
                compile_end = Some(i);
            }
            transform_start = Some(i);
        } else if trimmed == "//" {
            if compile_start.is_some() && compile_end.is_none() {
                compile_end = Some(i);
            } else if transform_start.is_some() && transform_end.is_none() {
                transform_end = Some(i);
            }
        }
    }

    ClifSectionBoundaries {
        compile_start,
        compile_end,
        transform_start,
        transform_end,
    }
}