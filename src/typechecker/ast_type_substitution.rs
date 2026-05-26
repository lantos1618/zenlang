use crate::ast::AstType;

pub(in crate::typechecker) fn substitute_ast_type_names<F>(
    ast_type: &AstType,
    substitute_name: &F,
) -> AstType
where
    F: Fn(&str) -> Option<AstType> + ?Sized,
{
    match ast_type {
        AstType::Named(name) => substitute_name(name).unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(substitute_ast_type_names(inner, substitute_name)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_ast_type_names(inner, substitute_name)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(substitute_ast_type_names(inner, substitute_name)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_ast_type_names(inner, substitute_name)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_ast_type_names(elem, substitute_name)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: substitute_ast_type_args_names(params, substitute_name),
            ret: Box::new(substitute_ast_type_names(ret, substitute_name)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: substitute_ast_type_args_names(type_args, substitute_name),
        },
        _ => ast_type.clone(),
    }
}

pub(in crate::typechecker) fn substitute_ast_type_args_names<F>(
    type_args: &[AstType],
    substitute_name: &F,
) -> Vec<AstType>
where
    F: Fn(&str) -> Option<AstType> + ?Sized,
{
    type_args
        .iter()
        .map(|arg| substitute_ast_type_names(arg, substitute_name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::substitute_ast_type_names;
    use crate::ast::AstType;
    use std::collections::HashMap;

    #[test]
    fn ast_type_name_substitution_covers_recursive_shapes() {
        let ast_type = AstType::Function {
            params: vec![
                AstType::Ptr(Box::new(AstType::Named("T".into()))),
                AstType::Generic {
                    name: "Result".into(),
                    type_args: vec![
                        AstType::Array {
                            elem: Box::new(AstType::RawPtr(Box::new(AstType::Named("E".into())))),
                            size: Some(2),
                        },
                        AstType::MutPtr(Box::new(AstType::Named("Missing".into()))),
                    ],
                },
            ],
            ret: Box::new(AstType::Slice(Box::new(AstType::Named("T".into())))),
        };
        let substitutions = HashMap::from([
            ("T".to_string(), AstType::I32),
            ("E".to_string(), AstType::Str),
        ]);

        let substituted =
            substitute_ast_type_names(&ast_type, &|name| substitutions.get(name).cloned());

        assert_eq!(
            substituted,
            AstType::Function {
                params: vec![
                    AstType::Ptr(Box::new(AstType::I32)),
                    AstType::Generic {
                        name: "Result".into(),
                        type_args: vec![
                            AstType::Array {
                                elem: Box::new(AstType::RawPtr(Box::new(AstType::Str))),
                                size: Some(2),
                            },
                            AstType::MutPtr(Box::new(AstType::Named("Missing".into()))),
                        ],
                    },
                ],
                ret: Box::new(AstType::Slice(Box::new(AstType::I32))),
            }
        );
    }
}
