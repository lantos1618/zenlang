use super::*;

impl TypeChecker {
    pub(crate) fn check_expr(&mut self, expr: &Expression) -> Result<TypedExpression, Diagnostic> {
        match expr {
            Expression::IntLiteral { value, span } => self.check_int_literal_expr(*value, *span),

            Expression::FloatLiteral { value, span } => {
                self.check_float_literal_expr(*value, *span)
            }

            Expression::StringLiteral { value, span } => {
                self.check_static_string_literal_expr(value, *span)
            }

            Expression::BoolLiteral { value, span } => self.check_bool_literal_expr(*value, *span),

            Expression::Identifier { name, span } => self.check_identifier_expr(name, *span),

            Expression::BinaryOp {
                op,
                left,
                right,
                span,
            } => self.check_binary_expr(*op, left, right, *span),

            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => self.check_function_call_expr(name, module, type_args, args, *span),

            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                span,
            } => self.check_method_call_expr(receiver, method, type_args, args, *span),

            Expression::MemberAccess {
                object,
                field,
                span,
            } => self.check_member_access_expr(object, field, *span),

            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => self.check_struct_literal_expr(name, type_args, fields, *span),

            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => self.check_enum_variant_expr(enum_name, type_args, variant, payload, *span),

            Expression::ArrayLiteral { elements, span } => {
                self.check_array_literal_expr(elements, *span)
            }

            Expression::Block {
                statements,
                expr,
                span,
            } => self.check_block_expr(statements, expr, *span),

            Expression::Break { span } => self.check_break_expr(*span),

            Expression::Continue { span } => self.check_continue_expr(*span),

            Expression::Match {
                scrutinee,
                arms,
                span,
            } => self.check_match_expr(scrutinee, arms, *span),

            Expression::If {
                condition,
                then_body,
                else_body,
                span,
            } => self.check_if_expr(condition, then_body, else_body, *span),

            Expression::WhileLoop {
                condition,
                body,
                span,
            } => self.check_while_loop_expr(condition, body, *span),

            Expression::Loop {
                body,
                control_label,
                span,
            } => self.check_loop_expr(body, control_label, *span),

            Expression::LoopControl {
                action,
                target_label,
                span,
            } => self.check_loop_control_expr(*action, target_label, *span),

            Expression::Cast {
                expr,
                target_type,
                span,
            } => self.check_cast_expr(expr, target_type, *span),

            Expression::StringInterpolation { parts, span } => {
                self.check_string_interpolation_expr(parts, *span)
            }

            Expression::Defer { expr, span } => self.check_defer_expr(expr, *span),

            Expression::IndexAccess {
                object,
                index,
                span,
            } => self.check_index_access_expr(object, index, *span),

            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => self.check_closure_expr(params, return_type, body, *span),

            Expression::UnaryOp { op, operand, span } => self.check_unary_expr(*op, operand, *span),

            Expression::Range {
                start, end, span, ..
            } => self.check_range_expr(start, end, *span),

            Expression::Error { span } => self.check_error_expr(*span),
        }
    }
}
