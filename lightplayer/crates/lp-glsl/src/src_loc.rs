use std::fmt;
use std::string::String;

/// Source location in GLSL code.
///
/// Tracks line and column information for error reporting.
/// Gracefully degrades when location information is unavailable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-indexed). 0 means unknown.
    pub line: usize,
    /// Column number (1-indexed). 0 means unknown.
    pub column: usize,
    /// Optional filename for multi-file support.
    pub filename: Option<String>,
}

impl SourceLocation {
    /// Create a new source location.
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            filename: None,
        }
    }

    /// Create a source location with a filename.
    pub fn with_file(line: usize, column: usize, filename: String) -> Self {
        Self {
            line,
            column,
            filename: Some(filename),
        }
    }

    /// Create an unknown location (for errors without source context).
    pub fn unknown() -> Self {
        Self {
            line: 0,
            column: 0,
            filename: None,
        }
    }

    /// Check if this location is unknown.
    pub fn is_unknown(&self) -> bool {
        self.line == 0 && self.column == 0
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_unknown() {
            write!(f, "<unknown>")
        } else if let Some(ref filename) = self.filename {
            write!(f, "{}:{}:{}", filename, self.line, self.column)
        } else {
            write!(f, "{}:{}", self.line, self.column)
        }
    }
}