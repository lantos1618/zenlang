use super::core::Compiler;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

impl<'ctx> Compiler<'ctx> {
    pub fn compile_string_length(&mut self, expr: &ast::Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let str_val = self.compile_expression(expr)?;
        if !str_val.is_pointer_value() {
            return Err(CompileError::TypeMismatch {
                expected: "string (i8*) for length operation".to_string(),
                found: format!("{:?}", str_val.get_type()),
                span: None,
            });
        }
        let str_ptr = str_val.into_pointer_value();
        let strlen_fn = match self.module.get_function("strlen") {
            Some(f) => f,
            None => {
                let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let fn_type = self.context.i64_type().fn_type(
                    &[i8_ptr_type.into()], 
                    false
                );
                self.module.add_function("strlen", fn_type, None)
            }
        };
        let len = {
            let call = self.builder.build_call(
                strlen_fn,
                &[str_ptr.into()],
                "strlen"
            )?;
            match call.try_as_basic_value() {
                inkwell::Either::Left(bv) => {
                    let int_val = bv.into_int_value();
                    let target_int_type = self.context.i64_type();
                    if int_val.get_type().get_bit_width() != target_int_type.get_bit_width() {
                        self.builder.build_int_z_extend(int_val, target_int_type, "strlen_ext")?.into()
                    } else {
                        int_val.into()
                    }
                },
                inkwell::Either::Right(_) => return Err(CompileError::InternalError(
                    "strlen did not return a basic value".to_string(),
                    None,
                )),
            }
        };
        Ok(len)
    }
} 