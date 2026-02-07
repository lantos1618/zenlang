//! Statement nodes in the AST

use std::collections::HashMap;
use std::fmt;

use super::expressions::Expression;
use super::fields::{AstFields, FieldValue};
use super::types::AstType;
use crate::error::Span;

/// A statement with optional source location information
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedStatement {
    pub stmt: Statement,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Expression {
        expr: Expression,
        span: Option<Span>,
    },
    Return {
        expr: Expression,
        span: Option<Span>,
    },
    // Enhanced variable declarations supporting all Zen syntax
    VariableDeclaration {
        name: String,
        type_: Option<AstType>, // None for inferred types
        initializer: Option<Expression>,
        is_mutable: bool, // true for ::= and :: T =, false for := and : T =
        declaration_type: VariableDeclarationType,
        span: Option<Span>, // Source location for error reporting
    },
    #[allow(dead_code)]
    VariableAssignment {
        name: String,
        value: Expression,
        span: Option<Span>,
    },
    PointerAssignment {
        pointer: Expression,
        value: Expression,
        span: Option<Span>,
    },
    // Loop construct supporting all Zen loop variations
    Loop {
        kind: LoopKind,
        label: Option<String>, // For labeled loops
        body: Vec<Statement>,
        span: Option<Span>,
    },
    Break {
        label: Option<String>, // For labeled break
        span: Option<Span>,
    },
    Continue {
        label: Option<String>, // For labeled continue
        span: Option<Span>,
    },
    // New statements for enhanced features
    ComptimeBlock {
        statements: Vec<Statement>,
        span: Option<Span>,
    },
    #[allow(dead_code)]
    ModuleImport { alias: String, module_path: String },
    // Defer statement for cleanup - traditional defer syntax
    #[allow(dead_code)]
    Defer {
        statement: Box<Statement>,
        span: Option<Span>,
    },
    // @this.defer() for scope-based cleanup
    #[allow(dead_code)]
    ThisDefer {
        expr: Expression,
        span: Option<Span>,
    },
    // Destructuring import: { io, maths } = @std
    DestructuringImport {
        names: Vec<String>,
        source: Expression,
        span: Option<Span>,
    },
    // Block of statements - used for defer blocks, etc.
    Block {
        statements: Vec<Statement>,
        span: Option<Span>,
    },
}

impl Statement {
    /// Returns the variant name of this statement as a static string.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Statement::Expression { .. } => "Expression",
            Statement::Return { .. } => "Return",
            Statement::VariableDeclaration { .. } => "VariableDeclaration",
            Statement::VariableAssignment { .. } => "VariableAssignment",
            Statement::PointerAssignment { .. } => "PointerAssignment",
            Statement::Loop { .. } => "Loop",
            Statement::Break { .. } => "Break",
            Statement::Continue { .. } => "Continue",
            Statement::ComptimeBlock { .. } => "ComptimeBlock",
            Statement::ModuleImport { .. } => "ModuleImport",
            Statement::Defer { .. } => "Defer",
            Statement::ThisDefer { .. } => "ThisDefer",
            Statement::DestructuringImport { .. } => "DestructuringImport",
            Statement::Block { .. } => "Block",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableDeclarationType {
    InferredImmutable, // = (plain assignment creates immutable in Zen spec)
    InferredMutable,   // ::=
    ExplicitImmutable, // : T (with type annotation, immutable)
    ExplicitMutable,   // :: T (with type annotation, mutable)
}

impl fmt::Display for VariableDeclarationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableDeclarationType::InferredImmutable => write!(f, "InferredImmutable"),
            VariableDeclarationType::InferredMutable => write!(f, "InferredMutable"),
            VariableDeclarationType::ExplicitImmutable => write!(f, "ExplicitImmutable"),
            VariableDeclarationType::ExplicitMutable => write!(f, "ExplicitMutable"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopKind {
    // loop { } - infinite loop
    Infinite,
    // loop condition { } - while-like loop
    Condition(Expression),
}

// ============================================================================
// AstFields implementation
// ============================================================================

impl AstFields for Statement {
    fn ast_fields(&self) -> Vec<(&'static str, FieldValue)> {
        match self {
            Statement::Expression { expr, .. } => {
                vec![("expr", FieldValue::expr(expr))]
            }
            Statement::Return { expr, .. } => {
                vec![("expr", FieldValue::expr(expr))]
            }
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                is_mutable,
                declaration_type,
                ..
            } => vec![
                ("name", FieldValue::String(name.clone())),
                (
                    "var_type",
                    match type_ {
                        Some(t) => FieldValue::ty(t),
                        None => FieldValue::Null,
                    },
                ),
                (
                    "initializer",
                    match initializer {
                        Some(e) => FieldValue::expr(e),
                        None => FieldValue::Null,
                    },
                ),
                ("is_mutable", FieldValue::Bool(*is_mutable)),
                (
                    "declaration_type",
                    FieldValue::String(declaration_type.to_string()),
                ),
            ],
            Statement::VariableAssignment { name, value, .. } => vec![
                ("name", FieldValue::String(name.clone())),
                ("value", FieldValue::expr(value)),
            ],
            Statement::PointerAssignment { pointer, value, .. } => vec![
                ("pointer", FieldValue::expr(pointer)),
                ("value", FieldValue::expr(value)),
            ],
            Statement::Loop {
                kind, label, body, ..
            } => vec![
                (
                    "kind",
                    match kind {
                        LoopKind::Infinite => FieldValue::String("Infinite".to_string()),
                        LoopKind::Condition(cond) => FieldValue::Struct {
                            name: "LoopKind".to_string(),
                            fields: HashMap::from([
                                (
                                    "kind".to_string(),
                                    FieldValue::String("Condition".to_string()),
                                ),
                                ("condition".to_string(), FieldValue::expr(cond)),
                            ]),
                        },
                    },
                ),
                ("label", FieldValue::opt_label(label)),
                ("body", FieldValue::stmt_array(body)),
            ],
            Statement::Break { label, .. } => {
                vec![("label", FieldValue::opt_label(label))]
            }
            Statement::Continue { label, .. } => {
                vec![("label", FieldValue::opt_label(label))]
            }
            Statement::ComptimeBlock { statements, .. } => {
                vec![("statements", FieldValue::stmt_array(statements))]
            }
            Statement::ModuleImport { alias, module_path } => vec![
                ("alias", FieldValue::String(alias.clone())),
                ("module_path", FieldValue::String(module_path.clone())),
            ],
            Statement::Defer { statement, .. } => {
                vec![("statement", FieldValue::Stmt(statement.clone()))]
            }
            Statement::ThisDefer { expr, .. } => {
                vec![("expr", FieldValue::expr(expr))]
            }
            Statement::DestructuringImport { names, source, .. } => vec![
                ("names", FieldValue::string_array(names)),
                ("source", FieldValue::expr(source)),
            ],
            Statement::Block { statements, .. } => {
                vec![("statements", FieldValue::stmt_array(statements))]
            }
        }
    }
}
