//! # Abstract Syntax Tree
//!
//! The `ast` module defines the data structures that represent the code in a structured way.
//! The parser will produce these structures, and the compiler will consume them.

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
    F32,
    F64,
    Bool,
    String,
    Void,
    Pointer(Box<AstType>),
    Array(Box<AstType>),
    FixedArray { 
        element_type: Box<AstType>,
        size: usize,
    },
    Function {
        args: Vec<AstType>,
        return_type: Box<AstType>,
    },
    Struct {
        name: String,
        fields: Vec<(String, AstType)>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
    },
    // Enhanced type system support
    Ref(Box<AstType>), // Managed reference
    Option(Box<AstType>), // Option<T>
    Result {
        ok_type: Box<AstType>,
        err_type: Box<AstType>,
    }, // Result<T, E>
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
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<AstType>, // Some(type) for variants with data, None for unit variants
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeParameter {
    pub name: String,
    pub constraints: Vec<String>, // For future trait bounds
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanEquals,
    GreaterThanEquals,
    StringConcat,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Integer8(i8),
    Integer16(i16),
    Integer32(i32),
    Integer64(i64),
    Unsigned8(u8),
    Unsigned16(u16),
    Unsigned32(u32),
    Unsigned64(u64),
    Float32(f32),
    Float64(f64),
    Boolean(bool),
    String(String),
    Identifier(String),
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    // Enhanced conditional expression for pattern matching with unified ? syntax
    Conditional {
        scrutinee: Box<Expression>,
        arms: Vec<ConditionalArm>,
    },
    AddressOf(Box<Expression>),
    Dereference(Box<Expression>),
    PointerOffset {
        pointer: Box<Expression>,
        offset: Box<Expression>,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    StructField {
        struct_: Box<Expression>,
        field: String,
    },
    // New expressions for enhanced features
    ArrayLiteral(Vec<Expression>),
    ArrayIndex {
        array: Box<Expression>,
        index: Box<Expression>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        payload: Option<Box<Expression>>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    StringLength(Box<Expression>),
    // For comptime expressions
    Comptime(Box<Expression>),
    // Range expressions
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    },
    // Pattern matching expressions
    PatternMatch {
        scrutinee: Box<Expression>,
        arms: Vec<PatternArm>,
    },
    // Standard library module reference
    StdModule(String),
    // Generic module reference
    Module(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>, // Optional guard condition using ->
    pub body: Expression,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Expression),
    Identifier(String),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
        payload: Option<Box<Pattern>>,
    },
    Wildcard, // _ pattern
    Or(Vec<Pattern>), // | pattern1 | pattern2
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    }, // For range patterns like 1..=10
    Binding {
        name: String,
        pattern: Box<Pattern>,
    }, // For -> binding in patterns
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression(Expression),
    Return(Expression),
    // Enhanced variable declarations supporting all Zen syntax
    VariableDeclaration {
        name: String,
        type_: Option<AstType>, // None for inferred types
        initializer: Option<Expression>,
        is_mutable: bool, // true for ::= and :: T =, false for := and : T =
        declaration_type: VariableDeclarationType,
    },
    VariableAssignment {
        name: String,
        value: Expression,
    },
    PointerAssignment {
        pointer: Expression,
        value: Expression,
    },
    // Loop construct for conditional loops only
    Loop {
        condition: Option<Expression>, // None for infinite loops
        label: Option<String>, // For labeled loops
        body: Vec<Statement>,
    },
    Break {
        label: Option<String>, // For labeled break
    },
    Continue {
        label: Option<String>, // For labeled continue
    },
    // New statements for enhanced features
    ComptimeBlock(Vec<Statement>),
    ModuleImport {
        alias: String,
        module_path: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableDeclarationType {
    InferredImmutable, // :=
    InferredMutable,   // ::=
    ExplicitImmutable, // : T =
    ExplicitMutable,   // :: T =
}



#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub args: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub body: Vec<Statement>,
    pub is_async: bool, // For async functions
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeAlias {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub target_type: AstType,
}

// For C FFI support
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFunction {
    pub name: String,
    pub args: Vec<AstType>,  // Just types, no names for external functions
    pub return_type: AstType,
    pub is_varargs: bool,  // For functions like printf
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub fields: Vec<StructField>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_: AstType,
    pub is_mutable: bool,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub variants: Vec<EnumVariant>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_: AstType,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorDefinition {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub methods: Vec<BehaviorMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: AstType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub type_name: String,
    pub behavior_name: Option<String>, // None for inherent impls
    pub type_params: Vec<TypeParameter>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Function(Function),
    ExternalFunction(ExternalFunction),
    Struct(StructDefinition),
    Enum(EnumDefinition),
    Behavior(BehaviorDefinition),
    Impl(ImplBlock),
    ComptimeBlock(Vec<Statement>),
    ModuleImport {
        alias: String,
        module_path: String,
    },
    TypeAlias(TypeAlias),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub declarations: Vec<Declaration>,
}

// Convenience methods for backward compatibility
impl Program {
    pub fn from_functions(functions: Vec<Function>) -> Self {
        Self {
            declarations: functions.into_iter().map(Declaration::Function).collect(),
        }
    }

    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.declarations.iter().filter_map(|decl| {
            if let Declaration::Function(func) = decl {
                Some(func)
            } else {
                None
            }
        })
    }
} 