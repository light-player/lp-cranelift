//! Number format parsing with format tracking.

extern crate alloc;
use alloc::{format, string::ToString};

/// Number format used in test expectations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumFormat {
    /// Hexadecimal: `0x00040000`
    Hex,
    /// Decimal integer: `65536`
    Decimal,
    /// Fixed32 literal: `4.0fx32` (converted to fixed16x16)
    Fixed32,
    /// Float32 literal: `0.0f32`
    Float32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumType {
    I32,
    U32,
    F32,
}

/// Parsed number (just the value, no format).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TestNum {
    I32(i32),
    U32(u32),
    F32(f32),
}

impl TestNum {
    /// Get the value as i32, converting if necessary.
    pub fn as_i32(self) -> Result<i32> {
        match self {
            TestNum::I32(value) => Ok(value),
            TestNum::U32(value) => value
                .try_into()
                .map_err(|_| ParseError::new(&format!("u32 value {} out of range for i32", value))),
            TestNum::F32(value) => {
                // Convert float to i32 (truncate)
                Ok(value as i32)
            }
        }
    }

    /// Get the value as u32, converting if necessary.
    pub fn as_u32(self) -> Result<u32> {
        match self {
            TestNum::U32(value) => Ok(value),
            TestNum::I32(value) => value
                .try_into()
                .map_err(|_| ParseError::new(&format!("i32 value {} out of range for u32", value))),
            TestNum::F32(value) => {
                if value < 0.0 {
                    return Err(ParseError::new(&format!(
                        "negative float {} cannot be converted to u32",
                        value
                    )));
                }
                Ok(value as u32)
            }
        }
    }

    /// Get the value as f32, converting if necessary.
    pub fn as_f32(self) -> Result<f32> {
        match self {
            TestNum::F32(value) => Ok(value),
            TestNum::I32(value) => Ok(value as f32),
            TestNum::U32(value) => Ok(value as f32),
        }
    }
}

/// Error type for number parsing.
#[derive(Debug, Clone)]
pub struct ParseError {
    message: alloc::string::String,
}

impl ParseError {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Result type for number parsing.
pub type Result<T> = core::result::Result<T, ParseError>;

/// Parse a number string into a TestNum with format tracking.
///
/// Returns both the number and its format.
/// Supports:
/// - Hex: `0x00040000` or `0X00040000`
/// - Decimal: `65536` or `-65536`
/// - Fixed32 literal: `4.0fx32` or `-4.0fx32` (converted to fixed16x16)
/// - Float32 literal: `0.0f32` or `-0.0f32`
pub fn parse_number(s: &str) -> Result<(TestNum, NumFormat)> {
    let s = s.trim();

    // Check for hex format
    if s.starts_with("0x") || s.starts_with("0X") {
        let hex_str = &s[2..];
        // Parse as u32 first, then convert to i32 (handles 0xffffffff case)
        let value_u32 = u32::from_str_radix(hex_str, 16)
            .map_err(|_| ParseError::new(&format!("invalid hex number '{}'", s)))?;
        let value = value_u32 as i32;
        return Ok((TestNum::I32(value), NumFormat::Hex));
    }

    // Check for fixed32 literal: <number>fx32
    if s.ends_with("fx32") {
        let float_str = &s[..s.len() - 4];
        let float_val: f32 = float_str
            .parse()
            .map_err(|_| ParseError::new(&format!("invalid fixed32 literal '{}'", s)))?;
        // Convert to fixed16x16: multiply by 65536 and round (no_std compatible)
        // For positive: add 0.5, for negative: subtract 0.5
        let fixed_val = if float_val >= 0.0 {
            ((float_val * 65536.0) + 0.5) as i32
        } else {
            ((float_val * 65536.0) - 0.5) as i32
        };
        return Ok((TestNum::I32(fixed_val), NumFormat::Fixed32));
    }

    // Check for float32 literal: <number>f32
    if s.ends_with("f32") {
        let float_str = &s[..s.len() - 3];
        let float_val: f32 = float_str
            .parse()
            .map_err(|_| ParseError::new(&format!("invalid float32 literal '{}'", s)))?;
        return Ok((TestNum::F32(float_val), NumFormat::Float32));
    }

    // Try parsing as decimal integer
    if let Ok(value) = s.parse::<i32>() {
        return Ok((TestNum::I32(value), NumFormat::Decimal));
    }

    // Try parsing as decimal unsigned integer
    if let Ok(value) = s.parse::<u32>() {
        return Ok((TestNum::U32(value), NumFormat::Decimal));
    }

    Err(ParseError::new(&format!("unable to parse number '{}'", s)))
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::format;

    #[test]
    fn test_parse_hex() {
        let (num, format) = parse_number("0x00040000").unwrap();
        assert!(matches!(num, TestNum::I32(262144)));
        assert_eq!(format, NumFormat::Hex);

        let (num, format) = parse_number("0X00040000").unwrap();
        assert!(matches!(num, TestNum::I32(262144)));
        assert_eq!(format, NumFormat::Hex);

        let (num, format) = parse_number("0xffffffff").unwrap();
        assert!(matches!(num, TestNum::I32(-1)));
        assert_eq!(format, NumFormat::Hex);
    }

    #[test]
    fn test_parse_decimal() {
        let (num, format) = parse_number("65536").unwrap();
        assert!(matches!(num, TestNum::I32(65536)));
        assert_eq!(format, NumFormat::Decimal);

        let (num, format) = parse_number("-65536").unwrap();
        assert!(matches!(num, TestNum::I32(-65536)));
        assert_eq!(format, NumFormat::Decimal);

        let (num, format) = parse_number("0").unwrap();
        assert!(matches!(num, TestNum::I32(0)));
        assert_eq!(format, NumFormat::Decimal);
    }

    #[test]
    fn test_parse_fixed32() {
        let (num, format) = parse_number("4.0fx32").unwrap();
        assert!(matches!(num, TestNum::I32(262144))); // 4.0 * 65536
        assert_eq!(format, NumFormat::Fixed32);

        let (num, format) = parse_number("1.0fx32").unwrap();
        assert!(matches!(num, TestNum::I32(65536))); // 1.0 * 65536
        assert_eq!(format, NumFormat::Fixed32);

        let (num, format) = parse_number("0.0fx32").unwrap();
        assert!(matches!(num, TestNum::I32(0)));
        assert_eq!(format, NumFormat::Fixed32);

        let (num, format) = parse_number("-1.0fx32").unwrap();
        assert!(matches!(num, TestNum::I32(-65536)));
        assert_eq!(format, NumFormat::Fixed32);

        let (num, format) = parse_number("0.5fx32").unwrap();
        assert!(matches!(num, TestNum::I32(32768))); // 0.5 * 65536
        assert_eq!(format, NumFormat::Fixed32);
    }

    #[test]
    fn test_parse_float32() {
        let (num, format) = parse_number("0.0f32").unwrap();
        assert!(matches!(num, TestNum::F32(0.0)));
        assert_eq!(format, NumFormat::Float32);

        let (num, format) = parse_number("1.5f32").unwrap();
        assert!(matches!(num, TestNum::F32(1.5)));
        assert_eq!(format, NumFormat::Float32);

        let (num, format) = parse_number("-1.5f32").unwrap();
        assert!(matches!(num, TestNum::F32(-1.5)));
        assert_eq!(format, NumFormat::Float32);
    }

    #[test]
    fn test_conversions() {
        let i32_val = TestNum::I32(42);
        assert_eq!(i32_val.as_i32().unwrap(), 42);
        assert_eq!(i32_val.as_u32().unwrap(), 42);
        assert_eq!(i32_val.as_f32().unwrap(), 42.0);

        let u32_val = TestNum::U32(42);
        assert_eq!(u32_val.as_i32().unwrap(), 42);
        assert_eq!(u32_val.as_u32().unwrap(), 42);
        assert_eq!(u32_val.as_f32().unwrap(), 42.0);

        let f32_val = TestNum::F32(42.5);
        assert_eq!(f32_val.as_i32().unwrap(), 42); // truncates
        assert_eq!(f32_val.as_f32().unwrap(), 42.5);
    }

    #[test]
    fn test_format_preservation() {
        let (_num, format) = parse_number("0x00040000").unwrap();
        assert_eq!(format, NumFormat::Hex);

        let (_num, format) = parse_number("4.0fx32").unwrap();
        assert_eq!(format, NumFormat::Fixed32);

        let (_num, format) = parse_number("1.5f32").unwrap();
        assert_eq!(format, NumFormat::Float32);
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse_number("invalid").is_err());
        assert!(parse_number("0xinvalid").is_err());
        assert!(parse_number("4.0fx").is_err()); // missing 32
        assert!(parse_number("4.0f32x").is_err()); // wrong suffix
    }
}
