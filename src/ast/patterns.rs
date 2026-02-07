//! Pattern matching constructs

use std::collections::HashMap;

use super::expressions::Expression;
use super::fields::{AstFields, FieldValue};

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
    // For pattern matching like .Some(val) or .None
    EnumLiteral {
        variant: String,
        payload: Option<Box<Pattern>>,
    },
    Or(Vec<Pattern>),    // | pattern1 | pattern2
    Tuple(Vec<Pattern>), // (pattern1, pattern2, ...)
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
    }, // For range patterns like 1..=10
    #[allow(dead_code)]
    Binding {
        name: String,
        pattern: Box<Pattern>,
    }, // For -> binding in patterns
    Type {
        type_name: String,
        binding: Option<String>, // Optional binding like: i32 -> n
    },
    Guard {
        pattern: Box<Pattern>,
        condition: Box<Expression>,
    },
}

impl Pattern {
    /// Returns the variant name of this pattern as a static string.
    pub fn variant_name(&self) -> &'static str {
        match self {
            Pattern::Literal(_) => "Literal",
            Pattern::Identifier(_) => "Identifier",
            Pattern::Struct { .. } => "Struct",
            Pattern::EnumVariant { .. } => "EnumVariant",
            Pattern::Wildcard => "Wildcard",
            Pattern::EnumLiteral { .. } => "EnumLiteral",
            Pattern::Or(_) => "Or",
            Pattern::Tuple(_) => "Tuple",
            Pattern::Range { .. } => "Range",
            Pattern::Binding { .. } => "Binding",
            Pattern::Type { .. } => "Type",
            Pattern::Guard { .. } => "Guard",
        }
    }
}

impl AstFields for Pattern {
    fn ast_fields(&self) -> Vec<(&'static str, FieldValue)> {
        match self {
            Pattern::Literal(expr) => {
                vec![("value", FieldValue::expr(expr))]
            }
            Pattern::Identifier(name) => {
                vec![("name", FieldValue::String(name.clone()))]
            }
            Pattern::Struct { name, fields: fs } => vec![
                ("name", FieldValue::String(name.clone())),
                (
                    "fields",
                    FieldValue::Array(
                        fs.iter()
                            .map(|(n, p)| FieldValue::Struct {
                                name: "PatternField".to_string(),
                                fields: HashMap::from([
                                    ("name".to_string(), FieldValue::String(n.clone())),
                                    ("pattern".to_string(), FieldValue::pat(p)),
                                ]),
                            })
                            .collect(),
                    ),
                ),
            ],
            Pattern::EnumVariant {
                enum_name,
                variant,
                payload,
            } => vec![
                ("enum_name", FieldValue::String(enum_name.clone())),
                ("variant", FieldValue::String(variant.clone())),
                ("payload", FieldValue::opt_pattern(payload)),
            ],
            Pattern::Wildcard => vec![],
            Pattern::EnumLiteral { variant, payload } => vec![
                ("variant", FieldValue::String(variant.clone())),
                ("payload", FieldValue::opt_pattern(payload)),
            ],
            Pattern::Or(pats) => {
                vec![("patterns", FieldValue::pat_array(pats))]
            }
            Pattern::Tuple(pats) => {
                vec![("patterns", FieldValue::pat_array(pats))]
            }
            Pattern::Range {
                start,
                end,
                inclusive,
            } => vec![
                ("start", FieldValue::boxed_expr(start)),
                ("end", FieldValue::boxed_expr(end)),
                ("inclusive", FieldValue::Bool(*inclusive)),
            ],
            Pattern::Binding { name, pattern } => vec![
                ("name", FieldValue::String(name.clone())),
                ("pattern", FieldValue::Pat(pattern.clone())),
            ],
            Pattern::Type { type_name, binding } => vec![
                ("type_name", FieldValue::String(type_name.clone())),
                ("binding", FieldValue::opt_label(binding)),
            ],
            Pattern::Guard { pattern, condition } => vec![
                ("pattern", FieldValue::Pat(pattern.clone())),
                ("condition", FieldValue::boxed_expr(condition)),
            ],
        }
    }
}
