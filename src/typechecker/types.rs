use crate::ast::primitives::bit_size as primitive_bit_size;
use crate::ast::{
    AstType, FLOAT_TYPES, INTEGER_TYPES, NUMERIC_TYPES, SIGNED_INT_TYPES, UNSIGNED_INT_TYPES,
};
use crate::stdlib_types::StdlibTypeRegistry;

/// Helper functions for working with types
impl AstType {
    /// Check if this type is numeric (integer or float)
    pub fn is_numeric(&self) -> bool {
        NUMERIC_TYPES.contains(self)
    }

    /// Check if this type is an integer type
    pub fn is_integer(&self) -> bool {
        INTEGER_TYPES.contains(self)
    }

    /// Check if this type is a floating-point type
    pub fn is_float(&self) -> bool {
        FLOAT_TYPES.contains(self)
    }

    /// Check if this type is a signed integer
    #[allow(dead_code)]
    pub fn is_signed_integer(&self) -> bool {
        SIGNED_INT_TYPES.contains(self)
    }

    /// Check if this type is an unsigned integer
    pub fn is_unsigned_integer(&self) -> bool {
        UNSIGNED_INT_TYPES.contains(self)
    }

    /// Get the size of the type in bits
    pub fn bit_size(&self) -> Option<usize> {
        // Use centralized bit_size for numeric types
        if let Some(size) = primitive_bit_size(self) {
            return Some(size as usize);
        }
        // Handle Bool separately (not in primitive_bit_size)
        if matches!(self, AstType::Bool) {
            return Some(1);
        }
        None
    }

    /// Get a default value for initialization
    #[allow(dead_code)]
    pub fn default_value(&self) -> String {
        if self.is_integer() {
            return "0".to_string();
        }
        if self.is_float() {
            return "0.0".to_string();
        }
        match self {
            AstType::Bool => "false".to_string(),
            AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => {
                "\"\"".to_string()
            }
            t if t.is_ptr_type() => "null".to_string(),
            _ => "null".to_string(),
        }
    }

    /// Check if this type is a pointer type
    #[allow(dead_code)]
    pub fn is_pointer(&self) -> bool {
        self.is_ptr_type()
    }

    /// Get the pointed-to type if this is a pointer type
    #[allow(dead_code)]
    pub fn pointee_type(&self) -> Option<&AstType> {
        self.ptr_inner()
    }

    /// Check if this is a mutable pointer type
    #[allow(dead_code)]
    pub fn is_mutable_pointer(&self) -> bool {
        self.is_mutable_ptr()
    }

    /// Check if this is a raw/unsafe pointer type
    #[allow(dead_code)]
    pub fn is_raw_pointer(&self) -> bool {
        self.is_raw_ptr()
    }

    /// Convert between pointer types while preserving the pointee type
    #[allow(dead_code)]
    pub fn as_pointer_type(&self, pointer_kind: PointerKind) -> Option<AstType> {
        self.pointee_type().map(|inner| match pointer_kind {
            PointerKind::Immutable => AstType::ptr(inner.clone()),
            PointerKind::Mutable => AstType::mut_ptr(inner.clone()),
            PointerKind::Raw => AstType::raw_ptr(inner.clone()),
        })
    }
}

/// Enum for different pointer kinds
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum PointerKind {
    Immutable, // Ptr<T>
    Mutable,   // MutPtr<T>
    Raw,       // RawPtr<T>
}
