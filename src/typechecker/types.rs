use crate::ast::AstType;

/// Helper functions for working with types
impl AstType {
    /// Check if this type is numeric (integer or float)
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            AstType::I8
                | AstType::I16
                | AstType::I32
                | AstType::I64
                | AstType::U8
                | AstType::U16
                | AstType::U32
                | AstType::U64
                | AstType::Usize
                | AstType::F32
                | AstType::F64
        )
    }

    /// Check if this type is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            AstType::I8
                | AstType::I16
                | AstType::I32
                | AstType::I64
                | AstType::U8
                | AstType::U16
                | AstType::U32
                | AstType::U64
                | AstType::Usize
        )
    }

    /// Check if this type is a floating-point type
    pub fn is_float(&self) -> bool {
        matches!(self, AstType::F32 | AstType::F64)
    }

    /// Check if this type is a signed integer
    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            AstType::I8 | AstType::I16 | AstType::I32 | AstType::I64
        )
    }

    /// Check if this type is an unsigned integer
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            AstType::U8 | AstType::U16 | AstType::U32 | AstType::U64 | AstType::Usize
        )
    }

    /// Get the size of the type in bits
    pub fn bit_size(&self) -> Option<usize> {
        match self {
            AstType::I8 | AstType::U8 => Some(8),
            AstType::I16 | AstType::U16 => Some(16),
            AstType::I32 | AstType::U32 | AstType::F32 => Some(32),
            AstType::I64 | AstType::U64 | AstType::F64 | AstType::Usize => Some(64), // usize is 64-bit on 64-bit platforms
            AstType::Bool => Some(1),
            _ => None,
        }
    }
}
