// Shared helpers for meta AST introspection

use crate::ast::{self, AstType, Declaration, Expression, Pattern, Statement};
use std::collections::HashMap;
use std::rc::Rc;

use crate::comptime::values::ComptimeValue;
use crate::comptime::ASTNodeValue;

pub fn field_info(name: &str, value: ComptimeValue) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "FieldInfo".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(name.to_string())),
            ("value".to_string(), value),
        ]),
    }
}

pub fn ast_node(value: ASTNodeValue) -> ComptimeValue {
    ComptimeValue::ASTNode(Rc::new(value))
}

pub fn ast_expr(e: Expression) -> ComptimeValue {
    ast_node(ASTNodeValue::Expression(e))
}

pub fn ast_stmt(s: Statement) -> ComptimeValue {
    ast_node(ASTNodeValue::Statement(s))
}

pub fn ast_type(t: AstType) -> ComptimeValue {
    ast_node(ASTNodeValue::Type(t))
}

pub fn ast_pattern(p: Pattern) -> ComptimeValue {
    ast_node(ASTNodeValue::Pattern(p))
}

pub fn opt_expr(e: &Option<Box<Expression>>) -> ComptimeValue {
    match e {
        Some(expr) => ast_expr(*expr.clone()),
        None => ComptimeValue::Null,
    }
}

pub fn opt_pattern(p: &Option<Box<Pattern>>) -> ComptimeValue {
    match p {
        Some(pat) => ast_pattern(*pat.clone()),
        None => ComptimeValue::Null,
    }
}

pub fn opt_type(t: &Option<AstType>) -> ComptimeValue {
    match t {
        Some(ty) => ast_type(ty.clone()),
        None => ComptimeValue::Null,
    }
}

/// Option<String> where None becomes "" (used for optional labels)
pub fn opt_label(s: &Option<String>) -> ComptimeValue {
    ComptimeValue::String(s.clone().unwrap_or_default())
}

/// Trait for arm types (MatchArm, PatternArm, ConditionalArm) that all share the same structure.
pub trait ArmLike {
    fn pattern(&self) -> &Pattern;
    fn guard(&self) -> &Option<Expression>;
    fn body(&self) -> &Expression;
}

impl ArmLike for ast::MatchArm {
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

impl ArmLike for ast::PatternArm {
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

impl ArmLike for ast::ConditionalArm {
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

/// Convert match arms to a fields array. Used by QuestionMatch, Conditional, PatternMatch.
pub fn match_arms_to_fields<A: ArmLike>(
    struct_name: &str,
    scrutinee: &Expression,
    arms: &[A],
) -> Vec<ComptimeValue> {
    vec![
        field_info("scrutinee", ast_expr(scrutinee.clone())),
        field_info(
            "arms",
            ComptimeValue::Array(
                arms.iter()
                    .map(|arm| ComptimeValue::Struct {
                        name: struct_name.to_string(),
                        fields: HashMap::from([
                            ("pattern".to_string(), ast_pattern(arm.pattern().clone())),
                            (
                                "guard".to_string(),
                                match arm.guard() {
                                    Some(g) => ast_expr(g.clone()),
                                    None => ComptimeValue::Null,
                                },
                            ),
                            ("body".to_string(), ast_expr(arm.body().clone())),
                        ]),
                    })
                    .collect(),
            ),
        ),
    ]
}

pub fn function_arg(name: &str, ty: &AstType) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "FunctionArg".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(name.to_string())),
            ("arg_type".to_string(), ast_type(ty.clone())),
        ]),
    }
}

pub fn function_to_fields(f: &ast::Function) -> Vec<ComptimeValue> {
    vec![
        field_info("name", ComptimeValue::String(f.name.clone())),
        field_info("type_params", type_params_to_array(&f.type_params)),
        field_info(
            "args",
            ComptimeValue::Array(
                f.args
                    .iter()
                    .map(|(name, ty)| function_arg(name, ty))
                    .collect(),
            ),
        ),
        field_info("return_type", ast_type(f.return_type.clone())),
        field_info(
            "body",
            ComptimeValue::Array(f.body.iter().map(|s| ast_stmt(s.clone())).collect()),
        ),
        field_info("is_varargs", ComptimeValue::Bool(f.is_varargs)),
        field_info("is_public", ComptimeValue::Bool(f.is_public)),
    ]
}

pub fn type_params_to_array(tps: &[ast::TypeParameter]) -> ComptimeValue {
    ComptimeValue::Array(
        tps.iter()
            .map(|tp| ComptimeValue::Struct {
                name: "TypeParameter".to_string(),
                fields: HashMap::from([
                    ("name".to_string(), ComptimeValue::String(tp.name.clone())),
                    (
                        "constraints".to_string(),
                        ComptimeValue::Array(
                            tp.constraints
                                .iter()
                                .map(|c| ComptimeValue::Struct {
                                    name: "TraitConstraint".to_string(),
                                    fields: HashMap::from([(
                                        "trait_name".to_string(),
                                        ComptimeValue::String(c.trait_name.clone()),
                                    )]),
                                })
                                .collect(),
                        ),
                    ),
                ]),
            })
            .collect(),
    )
}

pub fn methods_to_array(methods: &[ast::Function]) -> ComptimeValue {
    ComptimeValue::Array(
        methods
            .iter()
            .map(|m| ast_node(ASTNodeValue::Declaration(Declaration::Function(m.clone()))))
            .collect(),
    )
}

pub fn parameter_to_value(p: &ast::Parameter) -> ComptimeValue {
    ComptimeValue::Struct {
        name: "Parameter".to_string(),
        fields: HashMap::from([
            ("name".to_string(), ComptimeValue::String(p.name.clone())),
            ("param_type".to_string(), ast_type(p.type_.clone())),
            ("is_mutable".to_string(), ComptimeValue::Bool(p.is_mutable)),
        ]),
    }
}
