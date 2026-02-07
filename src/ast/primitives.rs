//! Centralized primitive type definitions
//!
//! This module provides a single source of truth for primitive types,
//! eliminating duplication across the codebase.

use std::fmt;
use std::str::FromStr;

use super::AstType;

/// All numeric primitive types (integers and floats)
pub const NUMERIC_TYPES: &[AstType] = &[
    AstType::I8,
    AstType::I16,
    AstType::I32,
    AstType::I64,
    AstType::U8,
    AstType::U16,
    AstType::U32,
    AstType::U64,
    AstType::Usize,
    AstType::F32,
    AstType::F64,
];

/// Signed integer types
pub const SIGNED_INT_TYPES: &[AstType] = &[AstType::I8, AstType::I16, AstType::I32, AstType::I64];

/// Unsigned integer types
pub const UNSIGNED_INT_TYPES: &[AstType] = &[
    AstType::U8,
    AstType::U16,
    AstType::U32,
    AstType::U64,
    AstType::Usize,
];

/// All integer types (signed and unsigned)
pub const INTEGER_TYPES: &[AstType] = &[
    AstType::I8,
    AstType::I16,
    AstType::I32,
    AstType::I64,
    AstType::U8,
    AstType::U16,
    AstType::U32,
    AstType::U64,
    AstType::Usize,
];

/// Floating point types
pub const FLOAT_TYPES: &[AstType] = &[AstType::F32, AstType::F64];

/// All primitive types including bool and void
pub const ALL_PRIMITIVES: &[AstType] = &[
    AstType::I8,
    AstType::I16,
    AstType::I32,
    AstType::I64,
    AstType::U8,
    AstType::U16,
    AstType::U32,
    AstType::U64,
    AstType::Usize,
    AstType::F32,
    AstType::F64,
    AstType::Bool,
    AstType::Void,
];

/// Primitive type names and their corresponding AstType variants
pub const PRIMITIVE_TYPE_MAP: &[(&str, AstType)] = &[
    ("i8", AstType::I8),
    ("i16", AstType::I16),
    ("i32", AstType::I32),
    ("i64", AstType::I64),
    ("u8", AstType::U8),
    ("u16", AstType::U16),
    ("u32", AstType::U32),
    ("u64", AstType::U64),
    ("usize", AstType::Usize),
    ("f32", AstType::F32),
    ("f64", AstType::F64),
    ("bool", AstType::Bool),
    ("void", AstType::Void),
    ("StaticString", AstType::StaticString),
];

/// Parse a primitive type from its string name
pub fn primitive_from_str(name: &str) -> Option<AstType> {
    PRIMITIVE_TYPE_MAP
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| t.clone())
}

/// Get the string name of a primitive type
pub fn primitive_to_str(ty: &AstType) -> Option<&'static str> {
    PRIMITIVE_TYPE_MAP
        .iter()
        .find(|(_, t)| t == ty)
        .map(|(n, _)| *n)
}

/// Check if a string represents a primitive type name
pub fn is_primitive_name(name: &str) -> bool {
    PRIMITIVE_TYPE_MAP.iter().any(|(n, _)| *n == name)
}

/// Numeric primitive type names (integers and floats)
pub const NUMERIC_TYPE_NAMES: &[&str] = &[
    "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "f32", "f64",
];

/// Check if a string represents a numeric primitive type name
pub fn is_numeric_type_name(name: &str) -> bool {
    NUMERIC_TYPE_NAMES.contains(&name)
}

/// Get the bit size of a numeric type
pub fn bit_size(ty: &AstType) -> Option<u32> {
    match ty {
        AstType::I8 | AstType::U8 => Some(8),
        AstType::I16 | AstType::U16 => Some(16),
        AstType::I32 | AstType::U32 | AstType::F32 => Some(32),
        AstType::I64 | AstType::U64 | AstType::Usize | AstType::F64 => Some(64),
        _ => None,
    }
}

/// Get the byte size of a primitive type (for sizeof operations).
///
/// Returns the size in bytes for primitive types. For non-primitive types,
/// returns None -- callers should handle structs, pointers, etc. themselves.
pub fn byte_size(ty: &AstType) -> Option<usize> {
    match ty {
        AstType::Bool => Some(1),
        AstType::Void => Some(0),
        other => bit_size(other).map(|bits| bits as usize / 8),
    }
}

/// Get an integer AstType from a bit size and signedness.
///
/// Useful for numeric promotion where you know the target width and sign.
/// Returns None for unrecognized bit sizes.
pub fn int_from_bit_size(bits: usize, signed: bool) -> Option<AstType> {
    match (signed, bits) {
        (true, 8) => Some(AstType::I8),
        (true, 16) => Some(AstType::I16),
        (true, 32) => Some(AstType::I32),
        (true, 64) => Some(AstType::I64),
        (false, 8) => Some(AstType::U8),
        (false, 16) => Some(AstType::U16),
        (false, 32) => Some(AstType::U32),
        (false, 64) => Some(AstType::U64),
        _ => None,
    }
}

/// Error type for parsing an AstType from a string.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseAstTypeError(pub String);

impl fmt::Display for ParseAstTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown type: {}", self.0)
    }
}

impl FromStr for AstType {
    type Err = ParseAstTypeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        primitive_from_str(s).ok_or_else(|| ParseAstTypeError(s.to_string()))
    }
}

/// Promote two numeric types to their common type for arithmetic
pub fn promote_numeric(left: &AstType, right: &AstType) -> Option<AstType> {
    let left_size = bit_size(left)?;
    let right_size = bit_size(right)?;
    let max_size = left_size.max(right_size);

    // Float promotion takes precedence
    if left.is_float() || right.is_float() {
        return Some(if max_size >= 64 {
            AstType::F64
        } else {
            AstType::F32
        });
    }

    // Integer promotion
    let is_signed = left.is_signed_integer() || right.is_signed_integer();
    Some(match (is_signed, max_size) {
        (true, 8) => AstType::I8,
        (true, 16) => AstType::I16,
        (true, 32) => AstType::I32,
        (true, _) => AstType::I64,
        (false, 8) => AstType::U8,
        (false, 16) => AstType::U16,
        (false, 32) => AstType::U32,
        (false, _) => AstType::U64,
    })
}

// ============================================================================
// AstType methods for primitive type checks
// Note: is_numeric, is_integer, is_float, etc. are defined in typechecker/types.rs
// ============================================================================

impl AstType {
    /// Check if this is any primitive type
    pub fn is_primitive(&self) -> bool {
        ALL_PRIMITIVES.contains(self)
    }

    /// Get the primitive type name as a string
    pub fn primitive_name(&self) -> Option<&'static str> {
        primitive_to_str(self)
    }

    /// Get the byte size of this primitive type (1, 2, 4, or 8).
    /// Returns None for non-primitive types.
    pub fn byte_size(&self) -> Option<usize> {
        byte_size(self)
    }
}

// ============================================================================
// ZEN LANGUAGE CONSTRUCTS (for LSP highlighting and validation)
// ============================================================================

/// Identifiers that introduce syntax constructs in Zen.
/// These are not keyword tokens (only `pub` is a keyword token),
/// but they have special meaning in parser context and should be
/// highlighted as keywords in IDEs.
pub const SYNTAX_INTRODUCERS: &[&str] = &["fn", "struct", "enum", "const", "var", "mut"];

/// Control flow identifiers
pub const CONTROL_FLOW: &[&str] = &["loop", "break", "continue", "return"];

/// Literal identifiers (parsed as identifiers but represent values)
pub const LITERAL_IDENTIFIERS: &[&str] = &["true", "false", "null"];

/// Compile-time and resource management
pub const COMPTIME_IDENTIFIERS: &[&str] = &["comptime", "defer"];

/// Self references (have special meaning in methods)
pub const SELF_IDENTIFIERS: &[&str] = &["self", "Self"];

/// Check if an identifier is a syntax introducer (fn, struct, enum, etc.)
pub fn is_syntax_introducer(name: &str) -> bool {
    SYNTAX_INTRODUCERS.contains(&name)
}

/// Check if an identifier is a control flow construct
pub fn is_control_flow(name: &str) -> bool {
    CONTROL_FLOW.contains(&name)
}

/// Check if an identifier is a literal (true, false, null)
pub fn is_literal_identifier(name: &str) -> bool {
    LITERAL_IDENTIFIERS.contains(&name)
}

/// Check if an identifier should be highlighted as keyword-like in IDE
/// (for semantic highlighting purposes)
pub fn is_keyword_like(name: &str) -> bool {
    name == "pub"  // The only actual keyword token
        || is_syntax_introducer(name)
        || is_control_flow(name)
        || is_literal_identifier(name)
        || COMPTIME_IDENTIFIERS.contains(&name)
}

/// Check if an identifier cannot be used as a user-defined name
/// (primitives, literals, self references)
pub fn is_reserved_identifier(name: &str) -> bool {
    is_primitive_name(name) || is_literal_identifier(name) || SELF_IDENTIFIERS.contains(&name)
}

// ============================================================================
// STDLIB TYPE QUERIES - Use stdlib_types module for dynamic discovery
// ============================================================================
//
// All stdlib type queries should use crate::stdlib_types::stdlib_types()
// which parses actual .zen files at startup and stays in sync with the stdlib.
//
// IMPORTANT: stdlib_types() is pre-initialized in main() before any user file
// parsing begins. This prevents deadlock (since stdlib_types() itself uses
// the parser internally).

/// Pointer type names - these are language primitives, not stdlib types
pub const POINTER_TYPES: &[&str] = &["Ptr", "MutPtr", "RawPtr"];

/// Constructor method names (methods that typically create new instances)
/// These are language conventions, not stdlib-specific
pub const CONSTRUCTOR_METHODS: &[&str] = &[
    "new",
    "init",
    "create",
    "default",
    "from",
    "with_capacity",
    "with_step",
];

/// Check if a type name is a pointer type (language primitive)
pub fn is_pointer_type(name: &str) -> bool {
    POINTER_TYPES.contains(&name)
}

/// Check if a method name is a constructor (language convention)
pub fn is_constructor_method(name: &str) -> bool {
    CONSTRUCTOR_METHODS.contains(&name)
}

/// Check if an identifier is a boolean literal
pub fn is_boolean_literal(name: &str) -> bool {
    name == "true" || name == "false"
}

/// Check if an identifier is a null/void literal
pub fn is_null_literal(name: &str) -> bool {
    name == "null"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_from_str() {
        assert_eq!(primitive_from_str("i32"), Some(AstType::I32));
        assert_eq!(primitive_from_str("u64"), Some(AstType::U64));
        assert_eq!(primitive_from_str("bool"), Some(AstType::Bool));
        assert_eq!(primitive_from_str("unknown"), None);
    }

    #[test]
    fn test_is_primitive() {
        assert!(AstType::I32.is_primitive());
        assert!(AstType::Bool.is_primitive());
        assert!(!AstType::Struct {
            name: "Foo".to_string(),
            fields: vec![]
        }
        .is_primitive());
    }

    #[test]
    fn test_bit_size() {
        assert_eq!(bit_size(&AstType::I8), Some(8));
        assert_eq!(bit_size(&AstType::I32), Some(32));
        assert_eq!(bit_size(&AstType::F64), Some(64));
    }

    #[test]
    fn test_promote_numeric() {
        assert_eq!(
            promote_numeric(&AstType::I32, &AstType::I64),
            Some(AstType::I64)
        );
        assert_eq!(
            promote_numeric(&AstType::I32, &AstType::F32),
            Some(AstType::F32)
        );
        assert_eq!(
            promote_numeric(&AstType::U8, &AstType::U16),
            Some(AstType::U16)
        );
    }

    #[test]
    fn test_byte_size() {
        assert_eq!(byte_size(&AstType::Bool), Some(1));
        assert_eq!(byte_size(&AstType::I8), Some(1));
        assert_eq!(byte_size(&AstType::U8), Some(1));
        assert_eq!(byte_size(&AstType::I16), Some(2));
        assert_eq!(byte_size(&AstType::I32), Some(4));
        assert_eq!(byte_size(&AstType::F32), Some(4));
        assert_eq!(byte_size(&AstType::I64), Some(8));
        assert_eq!(byte_size(&AstType::F64), Some(8));
        assert_eq!(byte_size(&AstType::Usize), Some(8));
        assert_eq!(byte_size(&AstType::Void), Some(0));
    }

    #[test]
    fn test_int_from_bit_size() {
        assert_eq!(int_from_bit_size(8, true), Some(AstType::I8));
        assert_eq!(int_from_bit_size(16, true), Some(AstType::I16));
        assert_eq!(int_from_bit_size(32, true), Some(AstType::I32));
        assert_eq!(int_from_bit_size(64, true), Some(AstType::I64));
        assert_eq!(int_from_bit_size(8, false), Some(AstType::U8));
        assert_eq!(int_from_bit_size(32, false), Some(AstType::U32));
        assert_eq!(int_from_bit_size(128, true), None);
    }

    #[test]
    fn test_from_str_trait() {
        assert_eq!("i32".parse::<AstType>(), Ok(AstType::I32));
        assert_eq!("f64".parse::<AstType>(), Ok(AstType::F64));
        assert_eq!("bool".parse::<AstType>(), Ok(AstType::Bool));
        assert!("unknown".parse::<AstType>().is_err());
    }
}
