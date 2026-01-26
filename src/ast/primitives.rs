//! Centralized primitive type definitions
//!
//! This module provides a single source of truth for primitive types,
//! eliminating duplication across the codebase.

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
}
