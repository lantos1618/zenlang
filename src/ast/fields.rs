//! Field-value abstraction for AST introspection.
//!
//! Provides a `FieldValue` enum that can represent any AST node or primitive,
//! plus traits (`AstFields`, `ArmLike`, `ProtocolMethodLike`) and shared helper
//! functions used by the comptime meta system.

use std::collections::HashMap;

use super::declarations::{BehaviorMethod, Declaration, Function, Parameter, TraitMethod};
use super::expressions::{ConditionalArm, Expression, MatchArm, PatternArm};
use super::patterns::Pattern;
use super::statements::Statement;
use super::types::{AstType, TypeParameter};

// ============================================================================
// FieldValue — a comptime-independent representation of AST field data
// ============================================================================

/// A value that can appear as a field of an AST node.
///
/// Unlike `ComptimeValue` (which lives in `src/comptime/`), `FieldValue` lives
/// in `src/ast/` and therefore does **not** create a circular dependency.
/// The comptime layer can convert `FieldValue` → `ComptimeValue` when needed.
#[derive(Debug, Clone)]
pub enum FieldValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    String(String),
    Array(Vec<FieldValue>),
    Struct {
        name: String,
        fields: HashMap<String, FieldValue>,
    },
    Expr(Box<Expression>),
    Stmt(Box<Statement>),
    Decl(Box<Declaration>),
    Type(Box<AstType>),
    Pat(Box<Pattern>),
    Null,
}

// ============================================================================
// Convenience constructors
// ============================================================================

impl FieldValue {
    // --- scalars / wrappers --------------------------------------------------

    pub fn expr(e: &Expression) -> Self {
        FieldValue::Expr(Box::new(e.clone()))
    }

    pub fn boxed_expr(e: &Expression) -> Self {
        FieldValue::Expr(Box::new(e.clone()))
    }

    pub fn stmt(s: &Statement) -> Self {
        FieldValue::Stmt(Box::new(s.clone()))
    }

    pub fn decl(d: &Declaration) -> Self {
        FieldValue::Decl(Box::new(d.clone()))
    }

    pub fn ty(t: &AstType) -> Self {
        FieldValue::Type(Box::new(t.clone()))
    }

    pub fn boxed_ty(t: &AstType) -> Self {
        FieldValue::Type(Box::new(t.clone()))
    }

    pub fn pat(p: &Pattern) -> Self {
        FieldValue::Pat(Box::new(p.clone()))
    }

    // --- option helpers ------------------------------------------------------

    pub fn opt_expr(e: &Option<Box<Expression>>) -> Self {
        match e {
            Some(expr) => FieldValue::Expr(expr.clone()),
            None => FieldValue::Null,
        }
    }

    pub fn opt_type(t: &Option<Box<AstType>>) -> Self {
        match t {
            Some(ty) => FieldValue::Type(ty.clone()),
            None => FieldValue::Null,
        }
    }

    pub fn opt_pattern(p: &Option<Box<Pattern>>) -> Self {
        match p {
            Some(pat) => FieldValue::Pat(pat.clone()),
            None => FieldValue::Null,
        }
    }

    pub fn opt_label(s: &Option<String>) -> Self {
        match s {
            Some(label) => FieldValue::String(label.clone()),
            None => FieldValue::Null,
        }
    }

    // --- array helpers -------------------------------------------------------

    pub fn string_array(v: &[String]) -> Self {
        FieldValue::Array(v.iter().map(|s| FieldValue::String(s.clone())).collect())
    }

    pub fn expr_array(v: &[Expression]) -> Self {
        FieldValue::Array(v.iter().map(FieldValue::expr).collect())
    }

    pub fn stmt_array(v: &[Statement]) -> Self {
        FieldValue::Array(v.iter().map(FieldValue::stmt).collect())
    }

    pub fn type_array(v: &[AstType]) -> Self {
        FieldValue::Array(v.iter().map(FieldValue::ty).collect())
    }

    pub fn pat_array(v: &[Pattern]) -> Self {
        FieldValue::Array(v.iter().map(FieldValue::pat).collect())
    }
}

// ============================================================================
// AstFields — generic "give me all fields" trait
// ============================================================================

/// Implemented by AST node types that can enumerate their fields as
/// `(name, FieldValue)` pairs. The comptime meta system uses this to expose
/// AST structure to user-level comptime code.
pub trait AstFields {
    fn ast_fields(&self) -> Vec<(&'static str, FieldValue)>;
}

// ============================================================================
// ArmLike — shared interface for match-arm-shaped structs
// ============================================================================

/// Trait for arm types (MatchArm, PatternArm, ConditionalArm) that all share
/// the same `pattern`, optional `guard`, and `body` structure.
pub trait ArmLike {
    fn pattern(&self) -> &Pattern;
    fn guard(&self) -> &Option<Expression>;
    fn body(&self) -> &Expression;
}

impl ArmLike for MatchArm {
    fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    fn guard(&self) -> &Option<Expression> {
        &self.guard
    }
    fn body(&self) -> &Expression {
        &self.body
    }
}

impl ArmLike for PatternArm {
    fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    fn guard(&self) -> &Option<Expression> {
        &self.guard
    }
    fn body(&self) -> &Expression {
        &self.body
    }
}

impl ArmLike for ConditionalArm {
    fn pattern(&self) -> &Pattern {
        &self.pattern
    }
    fn guard(&self) -> &Option<Expression> {
        &self.guard
    }
    fn body(&self) -> &Expression {
        &self.body
    }
}

// ============================================================================
// ProtocolMethodLike — shared interface for behavior/trait method definitions
// ============================================================================

/// Trait for method-like declarations that appear in behaviors and traits.
pub trait ProtocolMethodLike {
    fn method_name(&self) -> &str;
    fn method_params(&self) -> &[Parameter];
    fn method_return_type(&self) -> Option<&AstType>;
}

impl ProtocolMethodLike for BehaviorMethod {
    fn method_name(&self) -> &str {
        &self.name
    }
    fn method_params(&self) -> &[Parameter] {
        &self.params
    }
    fn method_return_type(&self) -> Option<&AstType> {
        Some(&self.return_type)
    }
}

impl ProtocolMethodLike for TraitMethod {
    fn method_name(&self) -> &str {
        &self.name
    }
    fn method_params(&self) -> &[Parameter] {
        &self.params
    }
    fn method_return_type(&self) -> Option<&AstType> {
        Some(&self.return_type)
    }
}

// ============================================================================
// Shared helper functions
// ============================================================================

/// Build the standard fields vec for any match-arm-like expression
/// (QuestionMatch, Conditional, PatternMatch).
///
/// Produces the equivalent structure to `match_arms_to_fields` in
/// `src/comptime/meta/helpers.rs`, but returns `FieldValue` instead of
/// `ComptimeValue`.
pub fn match_arms_fields<A: ArmLike>(
    struct_name: &str,
    scrutinee: &Expression,
    arms: &[A],
) -> Vec<(&'static str, FieldValue)> {
    vec![
        ("scrutinee", FieldValue::expr(scrutinee)),
        (
            "arms",
            FieldValue::Array(
                arms.iter()
                    .map(|arm| {
                        let mut fields = HashMap::new();
                        fields.insert("pattern".to_string(), FieldValue::pat(arm.pattern()));
                        fields.insert(
                            "guard".to_string(),
                            match arm.guard() {
                                Some(g) => FieldValue::expr(g),
                                None => FieldValue::Null,
                            },
                        );
                        fields.insert("body".to_string(), FieldValue::expr(arm.body()));
                        FieldValue::Struct {
                            name: struct_name.to_string(),
                            fields,
                        }
                    })
                    .collect(),
            ),
        ),
    ]
}

/// Convert a slice of `TypeParameter` into a `FieldValue::Array` of structs.
///
/// Each element is a `FieldValue::Struct { name: "TypeParameter", fields }` with
/// `"name"` and `"constraints"` keys — mirroring `type_params_to_array` in
/// `src/comptime/meta/helpers.rs`.
pub fn type_params_fields(tps: &[TypeParameter]) -> FieldValue {
    FieldValue::Array(
        tps.iter()
            .map(|tp| {
                let mut fields = HashMap::new();
                fields.insert("name".to_string(), FieldValue::String(tp.name.clone()));
                fields.insert(
                    "constraints".to_string(),
                    FieldValue::Array(
                        tp.constraints
                            .iter()
                            .map(|c| {
                                let mut cfields = HashMap::new();
                                cfields.insert(
                                    "trait_name".to_string(),
                                    FieldValue::String(c.trait_name.clone()),
                                );
                                FieldValue::Struct {
                                    name: "TraitConstraint".to_string(),
                                    fields: cfields,
                                }
                            })
                            .collect(),
                    ),
                );
                FieldValue::Struct {
                    name: "TypeParameter".to_string(),
                    fields,
                }
            })
            .collect(),
    )
}

/// Build a `FieldValue::Struct` representing a single function argument
/// (name + type), mirroring `function_arg` in helpers.rs.
pub fn function_arg_field(name: &str, ty: &AstType) -> FieldValue {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), FieldValue::String(name.to_string()));
    fields.insert("arg_type".to_string(), FieldValue::ty(ty));
    FieldValue::Struct {
        name: "FunctionArg".to_string(),
        fields,
    }
}

/// Build a `FieldValue::Struct` representing a `Parameter` (name, type,
/// mutability), mirroring `parameter_to_value` in helpers.rs.
pub fn parameter_field(p: &Parameter) -> FieldValue {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), FieldValue::String(p.name.clone()));
    fields.insert("param_type".to_string(), FieldValue::ty(&p.type_));
    fields.insert("is_mutable".to_string(), FieldValue::Bool(p.is_mutable));
    FieldValue::Struct {
        name: "Parameter".to_string(),
        fields,
    }
}

/// Convert a slice of `Function` into a `FieldValue::Array` of declaration
/// nodes, mirroring `methods_to_array` in helpers.rs.
pub fn methods_field(methods: &[Function]) -> FieldValue {
    FieldValue::Array(
        methods
            .iter()
            .map(|m| FieldValue::decl(&Declaration::Function(m.clone())))
            .collect(),
    )
}

/// Convert a slice of protocol methods (BehaviorMethod or TraitMethod) into a
/// `FieldValue::Array` of structs, each with `name`, `params`, and
/// `return_type` fields.
pub fn protocol_methods_field<M: ProtocolMethodLike>(
    struct_name: &str,
    methods: &[M],
) -> FieldValue {
    FieldValue::Array(
        methods
            .iter()
            .map(|m| {
                let mut fields = HashMap::new();
                fields.insert(
                    "name".to_string(),
                    FieldValue::String(m.method_name().to_string()),
                );
                fields.insert(
                    "params".to_string(),
                    FieldValue::Array(m.method_params().iter().map(parameter_field).collect()),
                );
                fields.insert(
                    "return_type".to_string(),
                    match m.method_return_type() {
                        Some(ty) => FieldValue::ty(ty),
                        None => FieldValue::Null,
                    },
                );
                FieldValue::Struct {
                    name: struct_name.to_string(),
                    fields,
                }
            })
            .collect(),
    )
}
