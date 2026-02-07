//! Type representations in the AST

use crate::well_known::well_known;
use std::fmt;

// ============================================================================
// TYPE NAME UTILITIES
// Centralized helpers to avoid string parsing duplication across codebase
// ============================================================================

/// Strip generic type parameters from a type string.
/// Examples: "HashMap<K, V>" -> "HashMap", "Vec<T>" -> "Vec", "i32" -> "i32"
///
/// This is used in 17+ places across the codebase to get base type names.
#[inline]
pub fn strip_generic_params(type_str: &str) -> &str {
    type_str.split('<').next().unwrap_or(type_str)
}

/// Check if a string looks like a type name (starts with uppercase).
/// Used for heuristic type detection in LSP and parsing.
///
/// This is used in 15+ places across the codebase.
#[inline]
pub fn looks_like_type_name(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

/// Resolve String type from stdlib - returns the struct type definition
/// String is defined in stdlib/string.zen as:
/// struct String {
///     data: Ptr<u8>
///     len: u64
///     capacity: usize
///     allocator: Allocator
/// }
pub fn resolve_string_struct_type() -> AstType {
    crate::stdlib_types::stdlib_types().get_string_type()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AstType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,
    F32,
    F64,
    Bool,
    StaticLiteral, // Internal: Compiler-known string literals (LLVM use only)
    StaticString,  // User-facing: StringLiteral (compile-time, immutable, no allocator)
    // String is defined in stdlib/string.zen as a struct, not a compiler primitive
    // Use resolve_string_struct_type() helper to get the struct type
    Void,
    // [T] - Slice type (pointer + length, no ownership)
    Slice(Box<AstType>),
    // [T; N] - Fixed-size array (inline, stack-allocated)
    FixedArray {
        element_type: Box<AstType>,
        size: usize,
    },
    // Note: Vec<T>, DynVec<T>, HashMap<K,V> etc. are stdlib generic types,
    // represented as AstType::Generic { name: "Vec", type_args: [T] }
    Function {
        args: Vec<AstType>,
        return_type: Box<AstType>,
    },
    FunctionPointer {
        param_types: Vec<AstType>,
        return_type: Box<AstType>,
    },
    Struct {
        name: String,
        fields: Vec<(String, AstType)>,
    },
    #[allow(dead_code)]
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    // Enhanced type system support
    Ref(Box<AstType>), // Managed reference
    // NOTE: Option<T> and Result<T, E> are now defined in stdlib/core/option.zen and stdlib/core/result.zen
    // They should be referenced as Generic { name: "Option", type_args: [T] } or Generic { name: "Result", type_args: [T, E] }
    Range {
        start_type: Box<AstType>,
        end_type: Box<AstType>,
        inclusive: bool,
    }, // Range types for .. and ..=
    // For generic types (future)
    Generic {
        name: String,
        type_args: Vec<AstType>,
    },
    // For enum type references (e.g., when MyOption is used as an identifier)
    #[allow(dead_code)]
    EnumType {
        name: String,
    },
    // For stdlib module references (e.g., math, io when imported)
    StdModule,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<AstType>, // Some(type) for variants with data, None for unit variants
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParameter {
    pub name: String,
    pub constraints: Vec<TraitConstraint>, // Trait bounds like T: Geometric + Serializable
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitConstraint {
    pub trait_name: String,
}

// ============================================================================
// Helper constructors for pointer types (use these instead of direct variants)
// These create Generic types that codegen recognizes via well_known registry
// ============================================================================

impl AstType {
    /// Returns the variant name of this type as a static string.
    pub fn variant_name(&self) -> &'static str {
        match self {
            AstType::I8 => "I8",
            AstType::I16 => "I16",
            AstType::I32 => "I32",
            AstType::I64 => "I64",
            AstType::U8 => "U8",
            AstType::U16 => "U16",
            AstType::U32 => "U32",
            AstType::U64 => "U64",
            AstType::Usize => "Usize",
            AstType::F32 => "F32",
            AstType::F64 => "F64",
            AstType::Bool => "Bool",
            AstType::StaticLiteral => "StaticLiteral",
            AstType::StaticString => "StaticString",
            AstType::Void => "Void",
            AstType::Slice(_) => "Slice",
            AstType::FixedArray { .. } => "FixedArray",
            AstType::Function { .. } => "Function",
            AstType::FunctionPointer { .. } => "FunctionPointer",
            AstType::Struct { .. } => "Struct",
            AstType::Enum { .. } => "Enum",
            AstType::Ref(_) => "Ref",
            AstType::Range { .. } => "Range",
            AstType::Generic { .. } => "Generic",
            AstType::EnumType { .. } => "EnumType",
            AstType::StdModule => "StdModule",
        }
    }

    /// Create a Ptr<T> type (immutable pointer)
    pub fn ptr(inner: AstType) -> AstType {
        AstType::Generic {
            name: well_known().ptr_name().to_string(),
            type_args: vec![inner],
        }
    }

    /// Create a MutPtr<T> type (mutable pointer)
    pub fn mut_ptr(inner: AstType) -> AstType {
        AstType::Generic {
            name: well_known().mut_ptr_name().to_string(),
            type_args: vec![inner],
        }
    }

    /// Create a RawPtr<T> type (raw/unsafe pointer)
    pub fn raw_ptr(inner: AstType) -> AstType {
        AstType::Generic {
            name: well_known().raw_ptr_name().to_string(),
            type_args: vec![inner],
        }
    }

    pub fn is_ptr_type(&self) -> bool {
        matches!(self, AstType::Generic { name, type_args } if type_args.len() == 1 && well_known().is_ptr(name))
    }

    pub fn is_immutable_ptr(&self) -> bool {
        matches!(self, AstType::Generic { name, type_args } if type_args.len() == 1 && well_known().is_immutable_ptr(name))
    }

    pub fn is_mutable_ptr(&self) -> bool {
        matches!(self, AstType::Generic { name, type_args } if type_args.len() == 1 && well_known().is_mutable_ptr(name))
    }

    pub fn is_raw_ptr(&self) -> bool {
        matches!(self, AstType::Generic { name, type_args } if type_args.len() == 1 && well_known().is_raw_ptr(name))
    }

    pub fn ptr_inner(&self) -> Option<&AstType> {
        match self {
            AstType::Generic { name, type_args }
                if type_args.len() == 1 && well_known().is_ptr(name) =>
            {
                Some(&type_args[0])
            }
            _ => None,
        }
    }

    /// Get the type name for struct, enum, and generic types
    /// Returns None for primitive types
    pub fn get_type_name(&self) -> Option<String> {
        match self {
            AstType::Struct { name, .. } => Some(name.clone()),
            AstType::Enum { name, .. } => Some(name.clone()),
            AstType::Generic { name, .. } => Some(name.clone()),
            _ => None,
        }
    }
}

// Display implementation for generating clean type names
impl fmt::Display for AstType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstType::I8 => write!(f, "i8"),
            AstType::I16 => write!(f, "i16"),
            AstType::I32 => write!(f, "i32"),
            AstType::I64 => write!(f, "i64"),
            AstType::U8 => write!(f, "u8"),
            AstType::U16 => write!(f, "u16"),
            AstType::U32 => write!(f, "u32"),
            AstType::U64 => write!(f, "u64"),
            AstType::Usize => write!(f, "usize"),
            AstType::F32 => write!(f, "f32"),
            AstType::F64 => write!(f, "f64"),
            AstType::Bool => write!(f, "bool"),
            AstType::StaticLiteral => write!(f, "StringLiteral"), // Display as StringLiteral to users
            AstType::StaticString => write!(f, "StringLiteral"),
            // String is now a struct type from stdlib, resolved via Struct variant
            AstType::Void => write!(f, "void"),
            AstType::Slice(inner) => write!(f, "[{}]", inner),
            AstType::FixedArray { element_type, size } => {
                write!(f, "[{}; {}]", element_type, size)
            }
            AstType::Function { args, return_type } => {
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") {}", return_type)
            }
            AstType::FunctionPointer {
                param_types,
                return_type,
            } => {
                write!(f, "fn(")?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") {}", return_type)
            }
            AstType::Struct { name, .. } => write!(f, "{}", name),
            AstType::Enum { name, .. } => write!(f, "{}", name),
            AstType::Ref(inner) => write!(f, "Ref<{}>", inner),
            // Option and Result are now Generic types - handled above
            AstType::Range {
                start_type,
                inclusive,
                ..
            } => {
                if *inclusive {
                    write!(f, "Range<{}..={}>", start_type, start_type)
                } else {
                    write!(f, "Range<{}..{}>", start_type, start_type)
                }
            }
            AstType::Generic { name, type_args } => {
                write!(f, "{}", name)?;
                if !type_args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            AstType::EnumType { name } => write!(f, "{}", name),
            AstType::StdModule => write!(f, "std"),
        }
    }
}
