use super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;
use inkwell::IntPredicate;

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_conditional(&mut self, scrutinee: &Expression, arms: &[(Expression, Expression)]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let parent_function = self.current_function
            .ok_or_else(|| CompileError::InternalError("No current function for conditional".to_string(), None))?;
        let cond_val = self.compile_expression(scrutinee)?;
        if !cond_val.is_int_value() || cond_val.into_int_value().get_type().get_bit_width() != 1 {
            return Err(CompileError::TypeMismatch {
                expected: "boolean (i1) for conditional expression".to_string(),
                found: format!("{:?}", cond_val.get_type()),
                span: None,
            });
        }
        let then_bb = self.context.append_basic_block(parent_function, "then");
        let else_bb = self.context.append_basic_block(parent_function, "else");
        let merge_bb = self.context.append_basic_block(parent_function, "ifcont");
        self.builder.build_conditional_branch(cond_val.into_int_value(), then_bb, else_bb)?;
        // Emit 'then' block
        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expression(&arms[0].1)?;
        self.builder.build_unconditional_branch(merge_bb)?;
        let then_bb_end = self.builder.get_insert_block().unwrap();
        // Emit 'else' block
        self.builder.position_at_end(else_bb);
        let else_val = self.compile_expression(&arms[1].1)?;
        self.builder.build_unconditional_branch(merge_bb)?;
        let else_bb_end = self.builder.get_insert_block().unwrap();
        // Emit 'merge' block
        self.builder.position_at_end(merge_bb);
        let phi = self.builder.build_phi(
            then_val.get_type(),
            "iftmp"
        )?;
        phi.add_incoming(&[(&then_val, then_bb_end), (&else_val, else_bb_end)]);
        Ok(phi.as_basic_value())
    }
} 