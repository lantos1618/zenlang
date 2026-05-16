use crate::ast::expressions::BinaryOp;
use crate::ast::expressions::UnaryOp;
use crate::error::Span;
use serde::Serialize;

// ─── Resolved Type ───────────────────────────────────────────────────────────
// Fully resolved types — no generics, no inference variables.
// This is the typechecker's output; codegen only sees these.

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Type {
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

    // Primitives
    Bool,
    Void,

    // Strings
    Str,    // static string: { ptr, len }
    String, // heap string: { ptr, len, cap, alloc }

    // Named type — struct/enum by mangled name
    Named(std::string::String),

    // Struct with known fields (inline definition)
    Struct {
        name: std::string::String,
        fields: Vec<(std::string::String, Type)>,
    },

    // Enum with known variants
    Enum {
        name: std::string::String,
        variants: Vec<(std::string::String, Option<Type>)>,
    },

    // Collections
    Array {
        elem: Box<Type>,
        size: Option<usize>,
    },
    Slice(Box<Type>),

    // Pointers
    Ptr(Box<Type>),
    MutPtr(Box<Type>),
    RawPtr(Box<Type>),

    // Function pointer type
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
    },

    // Diverging expression (return, break, infinite loop)
    Never,

    // Unresolved / error — sema couldn't determine the type
    Unknown,
}

impl Type {
    pub fn display_name(&self) -> std::string::String {
        match self {
            Type::I8 => "i8".into(),
            Type::I16 => "i16".into(),
            Type::I32 => "i32".into(),
            Type::I64 => "i64".into(),
            Type::U8 => "u8".into(),
            Type::U16 => "u16".into(),
            Type::U32 => "u32".into(),
            Type::U64 => "u64".into(),
            Type::Usize => "usize".into(),
            Type::F32 => "f32".into(),
            Type::F64 => "f64".into(),
            Type::Bool => "bool".into(),
            Type::Void => "void".into(),
            Type::Str => "str".into(),
            Type::String => "String".into(),
            Type::Named(n) => n.clone(),
            Type::Struct { name, .. } => name.clone(),
            Type::Enum { name, .. } => name.clone(),
            Type::Array { elem, size } => match size {
                Some(n) => format!("[{}; {}]", elem.display_name(), n),
                None => format!("[{}]", elem.display_name()),
            },
            Type::Slice(elem) => format!("[{}]", elem.display_name()),
            Type::Ptr(inner) => format!("Ptr<{}>", inner.display_name()),
            Type::MutPtr(inner) => format!("MutPtr<{}>", inner.display_name()),
            Type::RawPtr(inner) => format!("RawPtr<{}>", inner.display_name()),
            Type::Function { params, ret } => {
                let ps: Vec<_> = params.iter().map(|p| p.display_name()).collect();
                format!("({}) {}", ps.join(", "), ret.display_name())
            }
            Type::Never => "!".into(),
            Type::Unknown => "?".into(),
        }
    }

    /// Returns true if this type is numeric (integer or float).
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
                | Type::F32
                | Type::F64
        )
    }

    /// Returns true if this type is an integer.
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Usize
        )
    }

    /// Returns true if this type is a float.
    pub fn is_float(&self) -> bool {
        matches!(self, Type::F32 | Type::F64)
    }
}

// ─── Typed AST Nodes ─────────────────────────────────────────────────────────
// These mirror the untyped AST but every expression carries its resolved Type.
// We keep this minimal for now — it will be fleshed out when building the typechecker.

/// A typed expression: the expression kind + its resolved type + span.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedExpression {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub span: Span,
}

/// Typed expression kinds — mirrors Expression but types are resolved.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedExprKind {
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(std::string::String),
    BoolLiteral(bool),
    Variable(std::string::String),

    BinaryOp {
        op: BinaryOp,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
    },
    UnaryOp {
        op: UnaryOp,
        operand: Box<TypedExpression>,
    },

    /// All calls are resolved to concrete (mangled) function names.
    /// No generics, no method lookup — `p.distance(other)` is already
    /// `FunctionCall { function: "Point_distance", args: [p, other] }`.
    FunctionCall {
        function: std::string::String,
        args: Vec<TypedExpression>,
    },

    FieldAccess {
        object: Box<TypedExpression>,
        field: std::string::String,
    },

    IndexAccess {
        object: Box<TypedExpression>,
        index: Box<TypedExpression>,
    },

    StructLiteral {
        type_name: std::string::String,
        fields: Vec<(std::string::String, TypedExpression)>,
    },

    EnumVariant {
        type_name: std::string::String,
        variant: std::string::String,
        payload: Option<Box<TypedExpression>>,
    },

    ArrayLiteral {
        elements: Vec<TypedExpression>,
    },

    Match {
        scrutinee: Box<TypedExpression>,
        arms: Vec<TypedMatchArm>,
        kind: MatchKind,
    },

    Cast {
        expr: Box<TypedExpression>,
        from_type: Type,
        to_type: Type,
    },

    Ref(Box<TypedExpression>),
    MutRef(Box<TypedExpression>),
    Deref(Box<TypedExpression>),

    Closure {
        fn_name: std::string::String,
        env_type: std::string::String,
        captures: Vec<Capture>,
    },

    StringInterpolation {
        parts: Vec<TypedStringPart>,
    },

    Intrinsic {
        name: std::string::String,
        args: Vec<TypedExpression>,
    },

    Assign {
        target: Box<TypedExpression>,
        value: Box<TypedExpression>,
    },

    Block(TypedBlock),

    Return(Option<Box<TypedExpression>>),
    Break,
    Continue,

    Error,
}

/// Sema resolves which kind of control flow `?` represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MatchKind {
    /// `expr ? | true { } | false { }` → if/else
    ConditionalElse,
    /// `expr ? | true { }` → one-shot conditional
    Conditional,
    /// `expr ? { body }` → while loop
    WhileLoop,
    /// `enum ? | Variant {} ...` → switch on tag
    EnumMatch,
    /// `val ? | X {} | Y {}` → if/else chain on values
    ValueMatch,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedMatchArm {
    pub pattern: TypedPattern,
    pub body: TypedBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedPattern {
    Bool(bool),
    EnumVariant {
        type_name: std::string::String,
        variant: std::string::String,
        bindings: Vec<(std::string::String, Type)>,
    },
    Wildcard,
    Value(Box<TypedExpression>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Capture {
    pub name: std::string::String,
    pub ty: Type,
    pub by_ref: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedStringPart {
    Literal(std::string::String),
    Expr(TypedExpression),
}

// ─── Typed Statements ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedStatement {
    pub kind: TypedStatementKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypedStatementKind {
    VarDecl {
        name: std::string::String,
        ty: Type,
        value: TypedExpression,
        mutable: bool,
    },
    Expression(TypedExpression),
}

// ─── Typed Blocks ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedBlock {
    pub statements: Vec<TypedStatement>,
    pub expr: Option<Box<TypedExpression>>,
    pub ty: Type,
    pub span: Span,
}

// ─── Typed Declarations ──────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedFunction {
    pub name: std::string::String,
    pub params: Vec<TypedParam>,
    pub return_type: Type,
    pub body: TypedBlock,
    pub defers: Vec<TypedExpression>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedParam {
    pub name: std::string::String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedTypeDef {
    pub name: std::string::String,
    pub kind: TypeDefKind,
    pub methods: Vec<TypedFunction>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TypeDefKind {
    Struct {
        fields: Vec<(std::string::String, Type)>,
    },
    Enum {
        variants: Vec<TypedVariant>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedVariant {
    pub name: std::string::String,
    pub tag: u32,
    pub payload: Option<Vec<(std::string::String, Type)>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedGlobal {
    pub name: std::string::String,
    pub ty: Type,
    pub value: TypedExpression,
    pub mutable: bool,
    pub span: Span,
}

// ─── Top-Level Program ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
    pub types: Vec<TypedTypeDef>,
    pub globals: Vec<TypedGlobal>,
    pub entry_point: Option<std::string::String>,
}
