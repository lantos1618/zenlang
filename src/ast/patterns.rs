//! Pattern matching constructs

use super::expressions::Expression;

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
