pub mod calls;
pub mod collections;
pub mod control;
pub mod enums;
pub mod inference;
pub mod literals;
pub mod operations;
pub mod patterns;
pub mod structs;
pub mod utils;

use super::LLVMCompiler;
use crate::ast::Expression;
use crate::error::CompileError;
use inkwell::values::BasicValueEnum;

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            // Literals
            Expression::Integer8(_)
            | Expression::Integer16(_)
            | Expression::Integer32(_)
            | Expression::Integer64(_)
            | Expression::Unsigned8(_)
            | Expression::Unsigned16(_)
            | Expression::Unsigned32(_)
            | Expression::Unsigned64(_)
            | Expression::Float32(_)
            | Expression::Float64(_)
            | Expression::Boolean(_)
            | Expression::Unit
            | Expression::String(_) => literals::compile_literal(self, expr),

            Expression::Identifier(_) => literals::compile_identifier(self, expr),

            // String interpolation
            Expression::StringInterpolation { parts } => {
                use crate::ast::StringPart;
                let parts_vec: Vec<StringPart> = parts.to_vec();
                self.compile_string_interpolation(&parts_vec)
            }

            // Operations
            Expression::BinaryOp { .. } => operations::compile_binary_operation(self, expr),

            // Function calls
            Expression::FunctionCall { .. } => calls::compile_function_call(self, expr),
            Expression::MethodCall { .. } => calls::compile_method_call(self, expr),

            // Structs
            Expression::StructLiteral { .. } => structs::compile_struct_literal(self, expr),
            Expression::StructField { .. } => structs::compile_struct_field(self, expr),
            Expression::MemberAccess { .. } => structs::compile_member_access(self, expr),

            // Enums
            Expression::EnumVariant {
                enum_name,
                variant,
                payload,
            } => enums::compile_enum_variant(self, enum_name, variant, payload),
            Expression::EnumLiteral { .. } => enums::compile_enum_literal(self, expr),
            Expression::Some(value) => enums::compile_some(self, value),
            Expression::None => enums::compile_none(self),

            // Collections - deprecated syntax, all use stdlib/vec.zen now
            Expression::ArrayLiteral(_)
            | Expression::ArrayIndex { .. }
            | Expression::VecConstructor { .. }
            | Expression::DynVecConstructor { .. }
            | Expression::ArrayConstructor { .. } => Err(CompileError::InternalError(
                "Array/Vec syntax is deprecated. Use Vec.new(allocator) from stdlib/vec.zen"
                    .to_string(),
                self.get_current_span(),
            )),

            // Control flow
            Expression::Loop { .. } => control::compile_loop(self, expr),
            Expression::Break { .. } => control::compile_break(self, expr),
            Expression::Continue { .. } => control::compile_continue(self, expr),
            Expression::Return(_) => control::compile_return(self, expr),
            Expression::Range { .. } => control::compile_range_expression(self, expr),

            // Pattern matching
            Expression::QuestionMatch { .. } | Expression::Conditional { .. } => {
                patterns::compile_pattern_match(self, expr)
            }

            // Block expressions - compile statements and return last expression value
            Expression::Block(statements) => compile_block_expression(self, statements),
            Expression::Closure { .. } => calls::compile_closure(self, expr),
            Expression::Comptime(_) => utils::compile_comptime_expression(self, expr),
            Expression::Raise(_) => utils::compile_raise_expression(self, expr),

            // Pointers - delegate to pointers.rs
            Expression::AddressOf(inner) => self.compile_address_of(inner),
            Expression::Dereference(inner) => self.compile_dereference(inner),
            Expression::PointerOffset { pointer, offset } => {
                self.compile_pointer_offset(pointer, offset)
            }
            Expression::PointerDereference(inner) => self.compile_dereference(inner),
            Expression::PointerAddress(inner) => self.compile_pointer_to_int(inner),
            Expression::CreateReference(inner) => self.compile_address_of(inner),
            Expression::CreateMutableReference(inner) => self.compile_address_of(inner),

            _ => Err(CompileError::InternalError(
                format!("Unhandled expression type: {:?}", expr),
                None,
            )),
        }
    }

    pub fn infer_expression_type(
        &self,
        expr: &Expression,
    ) -> Result<crate::ast::AstType, CompileError> {
        inference::infer_expression_type(self, expr)
    }

    pub fn infer_closure_return_type(
        &self,
        body: &Expression,
    ) -> Result<crate::ast::AstType, CompileError> {
        inference::infer_closure_return_type(self, body)
    }

    pub fn compile_enum_variant(
        &mut self,
        enum_name: &str,
        variant: &str,
        payload: &Option<Box<Expression>>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        enums::compile_enum_variant(self, enum_name, variant, payload)
    }

    pub fn compile_array_index_address(
        &mut self,
        array: &Expression,
        index: &Expression,
    ) -> Result<inkwell::values::PointerValue<'ctx>, CompileError> {
        collections::compile_array_index_address(self, array, index)
    }
}

/// Compile a block expression - executes statements and returns the last expression's value
fn compile_block_expression<'ctx>(
    compiler: &mut LLVMCompiler<'ctx>,
    statements: &[crate::ast::Statement],
) -> Result<BasicValueEnum<'ctx>, CompileError> {
    use crate::ast::Statement;

    if statements.is_empty() {
        // Empty block returns void/unit (i32 0 as placeholder)
        return Ok(compiler.context.i32_type().const_int(0, false).into());
    }

    // Compile all statements except the last one
    for stmt in &statements[..statements.len() - 1] {
        compiler.compile_statement(stmt)?;
    }

    // Handle the last statement specially - if it's an expression, return its value
    let last_stmt = &statements[statements.len() - 1];
    match last_stmt {
        Statement::Expression { expr, .. } => {
            // The last expression is the block's return value
            compiler.compile_expression(expr)
        }
        Statement::Return { .. } => {
            // If the last statement is a return, compile it normally
            // The actual return happens in the statement, but we need to return something here
            compiler.compile_statement(last_stmt)?;
            // Return a dummy value since the return already happened
            Ok(compiler.context.i32_type().const_int(0, false).into())
        }
        _ => {
            // Other statements don't produce a value, compile and return void
            compiler.compile_statement(last_stmt)?;
            Ok(compiler.context.i32_type().const_int(0, false).into())
        }
    }
}
