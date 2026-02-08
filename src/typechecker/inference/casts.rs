use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result};
use crate::typechecker::validation::is_type_parameter_name;

pub fn infer_cast_type(args: &[Expression], span: Option<crate::error::Span>) -> Result<AstType> {
    if args.len() == 2 {
        if let Expression::Identifier(type_name) = &args[1] {
            if is_type_parameter_name(type_name) {
                return Ok(AstType::Generic {
                    name: type_name.clone(),
                    type_args: vec![],
                });
            }

            return match crate::parser::parse_type_from_string(type_name) {
                Ok(ast_type) => {
                    if ast_type.is_numeric()
                        || ast_type.is_ptr_type()
                        || matches!(ast_type, AstType::FunctionPointer { .. })
                    {
                        Ok(ast_type)
                    } else {
                        Err(CompileError::TypeError(
                            format!(
                                "cast() target type '{}' is not a valid numeric, pointer, or function pointer type. Supported types: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, Ptr<T>, MutPtr<T>, RawPtr<T>, (params) ReturnType",
                                type_name
                            ),
                            span.clone(),
                        ))
                    }
                }
                Err(e) => Err(CompileError::TypeError(
                    format!(
                        "cast() target type '{}' is not a valid type: {}",
                        type_name, e
                    ),
                    span.clone(),
                )),
            };
        }
    }

    Err(CompileError::TypeError(
        format!(
            "cast() expects 2 arguments: cast(value, type), but got {} argument(s). Example: cast(42, i64)",
            args.len()
        ),
        span,
    ))
}
