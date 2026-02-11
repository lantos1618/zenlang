use crate::error::Span;

/// Parser-level type representation.
///
/// These types may be unresolved — `Named("Point")` hasn't been looked up yet,
/// `Inferred` means the typechecker must figure it out. The typechecker resolves
/// these into fully concrete `Type` values (see `typed.rs`).
#[derive(Debug, Clone, PartialEq)]
pub enum AstType {
    // Integers
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Usize,

    // Floats
    F32,
    F64,

    // Other primitives
    Bool,
    Void,

    // Strings
    Str,    // static string: { ptr, len }
    String, // heap string: { ptr, len, cap, alloc }

    // User-defined / unresolved named type
    Named(std::string::String),

    // Generic type application: `Channel<SensorReading>`, `Result<T, E>`
    Generic {
        name: std::string::String,
        type_args: Vec<AstType>,
    },

    // Collections
    Array {
        elem: Box<AstType>,
        size: Option<usize>,
    },
    Slice(Box<AstType>),

    // Pointers
    Ptr(Box<AstType>),
    MutPtr(Box<AstType>),
    RawPtr(Box<AstType>),

    // Function type: `(i32, i32) i32`
    Function {
        params: Vec<AstType>,
        ret: Box<AstType>,
    },

    // Self type — used in method signatures and behavior definitions
    SelfType,

    // Type to be inferred by the typechecker
    Inferred,
}

impl AstType {
    /// Returns the span-free name for display/error messages.
    pub fn display_name(&self) -> std::string::String {
        match self {
            AstType::I8 => "i8".into(),
            AstType::I16 => "i16".into(),
            AstType::I32 => "i32".into(),
            AstType::I64 => "i64".into(),
            AstType::U8 => "u8".into(),
            AstType::U16 => "u16".into(),
            AstType::U32 => "u32".into(),
            AstType::U64 => "u64".into(),
            AstType::Usize => "usize".into(),
            AstType::F32 => "f32".into(),
            AstType::F64 => "f64".into(),
            AstType::Bool => "bool".into(),
            AstType::Void => "void".into(),
            AstType::Str => "str".into(),
            AstType::String => "String".into(),
            AstType::Named(n) => n.clone(),
            AstType::Generic { name, type_args } => {
                let args: Vec<_> = type_args.iter().map(|a| a.display_name()).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            AstType::Array { elem, size } => match size {
                Some(n) => format!("[{}; {}]", elem.display_name(), n),
                None => format!("[{}]", elem.display_name()),
            },
            AstType::Slice(elem) => format!("[{}]", elem.display_name()),
            AstType::Ptr(inner) => format!("Ptr<{}>", inner.display_name()),
            AstType::MutPtr(inner) => format!("MutPtr<{}>", inner.display_name()),
            AstType::RawPtr(inner) => format!("RawPtr<{}>", inner.display_name()),
            AstType::Function { params, ret } => {
                let ps: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                format!("({}) {}", ps.join(", "), ret.display_name())
            }
            AstType::SelfType => "Self".into(),
            AstType::Inferred => "_".into(),
        }
    }
}

/// A typed parameter in a function/method/closure signature.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: std::string::String,
    pub ty: AstType,
    pub mutable: bool,
    pub span: Span,
}
