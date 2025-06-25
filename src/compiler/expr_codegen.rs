use super::*;

// All code related to expression codegen will be moved here.
// ... existing code from mod.rs for compile_expression and helpers ... 

impl<'ctx> Compiler<'ctx> {
    pub fn compile_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, CompileError> {
        match expr {
            Expression::Integer8(n) => self.compile_integer_literal(*n as i64),
            Expression::Integer16(n) => self.compile_integer_literal(*n as i64),
            Expression::Integer32(n) => self.compile_integer_literal(*n as i64),
            Expression::Integer64(n) => self.compile_integer_literal(*n),
            Expression::Float(n) => self.compile_float_literal(*n),
            Expression::String(val) => self.compile_string_literal(val),
            Expression::Identifier(name) => self.compile_identifier(name),
            Expression::BinaryOp { op, left, right } => self.compile_binary_operation(op, left, right),
            Expression::FunctionCall { name, args } => self.compile_function_call(name, args),
            Expression::Conditional { scrutinee, arms } => self.compile_conditional(scrutinee, arms),
            Expression::AddressOf(expr) => self.compile_address_of(expr),
            Expression::Dereference(expr) => self.compile_dereference(expr),
            Expression::PointerOffset { pointer, offset } => self.compile_pointer_offset(pointer, offset),
            Expression::StructLiteral { name, fields } => self.compile_struct_literal(name, fields),
            Expression::StructField { struct_, field } => self.compile_struct_field(struct_, field),
            Expression::StringLength(expr) => self.compile_string_length(expr),
        }
    }
} 