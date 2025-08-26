use crate::ast::{AstType, Expression, Pattern, EnumVariant};

/// Result<T, E> type for error handling
pub fn create_result_type() -> AstType {
    AstType::Enum {
        name: "Result".to_string(),
        variants: vec![
            EnumVariant {
                name: "Ok".to_string(),
                payload: Some(AstType::Generic {
                    name: "T".to_string(),
                    type_args: vec![],
                }),
            },
            EnumVariant {
                name: "Err".to_string(),
                payload: Some(AstType::Generic {
                    name: "E".to_string(),
                    type_args: vec![],
                }),
            },
        ],
    }
}

/// Option<T> type for nullable values
pub fn create_option_type() -> AstType {
    AstType::Enum {
        name: "Option".to_string(),
        variants: vec![
            EnumVariant {
                name: "Some".to_string(),
                payload: Some(AstType::Generic {
                    name: "T".to_string(),
                    type_args: vec![],
                }),
            },
            EnumVariant {
                name: "None".to_string(),
                payload: None,
            },
        ],
    }
}

/// Helper to create an Ok variant
pub fn ok_value(value: Expression) -> Expression {
    Expression::EnumVariant {
        enum_name: "Result".to_string(),
        variant: "Ok".to_string(),
        payload: Some(Box::new(value)),
    }
}

/// Helper to create an Err variant
pub fn err_value(error: Expression) -> Expression {
    Expression::EnumVariant {
        enum_name: "Result".to_string(),
        variant: "Err".to_string(),
        payload: Some(Box::new(error)),
    }
}

/// Helper to create a Some variant
pub fn some_value(value: Expression) -> Expression {
    Expression::EnumVariant {
        enum_name: "Option".to_string(),
        variant: "Some".to_string(),
        payload: Some(Box::new(value)),
    }
}

/// Helper to create a None variant
pub fn none_value() -> Expression {
    Expression::EnumVariant {
        enum_name: "Option".to_string(),
        variant: "None".to_string(),
        payload: None,
    }
}

/// Pattern for matching Ok(value)
pub fn ok_pattern(binding: Option<String>) -> Pattern {
    Pattern::EnumVariant {
        enum_name: "Result".to_string(),
        variant: "Ok".to_string(),
        payload: binding.map(|name| Box::new(Pattern::Identifier(name))),
    }
}

/// Pattern for matching Err(error)
pub fn err_pattern(binding: Option<String>) -> Pattern {
    Pattern::EnumVariant {
        enum_name: "Result".to_string(),
        variant: "Err".to_string(),
        payload: binding.map(|name| Box::new(Pattern::Identifier(name))),
    }
}

/// Pattern for matching Some(value)
pub fn some_pattern(binding: Option<String>) -> Pattern {
    Pattern::EnumVariant {
        enum_name: "Option".to_string(),
        variant: "Some".to_string(),
        payload: binding.map(|name| Box::new(Pattern::Identifier(name))),
    }
}

/// Pattern for matching None
pub fn none_pattern() -> Pattern {
    Pattern::EnumVariant {
        enum_name: "Option".to_string(),
        variant: "None".to_string(),
        payload: None,
    }
}