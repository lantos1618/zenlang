use crate::ast::AstType;

pub(super) fn generic_impl_ast_types_compatible(
    expected: &AstType,
    actual: &AstType,
    type_name: &str,
    target_type_args: &[AstType],
) -> bool {
    let self_type = generic_impl_self_type(type_name, target_type_args);
    match (expected, actual) {
        (AstType::SelfType, AstType::SelfType) => true,
        (AstType::SelfType, actual) => actual == &self_type,
        (expected, AstType::SelfType) => expected == &self_type,
        (
            AstType::Generic {
                name,
                type_args: expected_args,
            },
            AstType::Generic {
                name: actual_name,
                type_args: actual_args,
            },
        ) => {
            name == actual_name
                && generic_impl_ast_type_slices_compatible(
                    expected_args,
                    actual_args,
                    type_name,
                    target_type_args,
                )
        }
        (
            AstType::Array { elem, size },
            AstType::Array {
                elem: actual,
                size: actual_size,
            },
        ) => {
            size == actual_size
                && generic_impl_ast_types_compatible(elem, actual, type_name, target_type_args)
        }
        (AstType::Slice(inner), AstType::Slice(actual))
        | (AstType::Ptr(inner), AstType::Ptr(actual))
        | (AstType::MutPtr(inner), AstType::MutPtr(actual))
        | (AstType::RawPtr(inner), AstType::RawPtr(actual)) => {
            generic_impl_ast_types_compatible(inner, actual, type_name, target_type_args)
        }
        (
            AstType::Function { params, ret },
            AstType::Function {
                params: actual,
                ret: actual_ret,
            },
        ) => {
            generic_impl_ast_type_slices_compatible(params, actual, type_name, target_type_args)
                && generic_impl_ast_types_compatible(ret, actual_ret, type_name, target_type_args)
        }
        _ => expected == actual,
    }
}

fn generic_impl_ast_type_slices_compatible(
    expected: &[AstType],
    actual: &[AstType],
    type_name: &str,
    target_type_args: &[AstType],
) -> bool {
    expected.len() == actual.len()
        && expected.iter().zip(actual).all(|(expected, actual)| {
            generic_impl_ast_types_compatible(expected, actual, type_name, target_type_args)
        })
}

fn generic_impl_self_type(type_name: &str, type_args: &[AstType]) -> AstType {
    if type_args.is_empty() {
        AstType::Named(type_name.to_string())
    } else {
        AstType::Generic {
            name: type_name.to_string(),
            type_args: type_args.to_vec(),
        }
    }
}

pub(super) fn generic_impl_type_display(
    ty: &AstType,
    type_name: &str,
    type_args: &[AstType],
) -> String {
    match ty {
        AstType::SelfType => generic_impl_self_type(type_name, type_args).display_name(),
        _ => ty.display_name(),
    }
}
