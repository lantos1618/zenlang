use crate::ast::typed::{MatchKind, TypedExprKind, TypedExpression};

use super::mir_match_arm;
use super::schema::MirExpression;

pub(super) fn mir_expression(expr: &TypedExpression) -> MirExpression {
    let mut lowered = MirExpression {
        kind: mir_expression_kind(expr),
        ty: expr.ty.display_name(),
        name: None,
        value: None,
        op: None,
        left: None,
        right: None,
        target: None,
        field: None,
        function: None,
        match_kind: None,
        args: Vec::new(),
        arms: Vec::new(),
    };

    match &expr.kind {
        TypedExprKind::IntLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
        }
        TypedExprKind::FloatLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
        }
        TypedExprKind::StringLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
        }
        TypedExprKind::BoolLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
        }
        TypedExprKind::Variable(name) => {
            lowered.name = Some(name.clone());
        }
        TypedExprKind::BinaryOp { op, left, right } => {
            lowered.op = Some(op.symbol());
            lowered.left = Some(Box::new(mir_expression(left)));
            lowered.right = Some(Box::new(mir_expression(right)));
        }
        TypedExprKind::UnaryOp { op, operand } => {
            lowered.op = Some(op.symbol());
            lowered.target = Some(Box::new(mir_expression(operand)));
        }
        TypedExprKind::FunctionCall { function, args } => {
            lowered.function = Some(function.clone());
            lowered.args = args.iter().map(mir_expression).collect();
        }
        TypedExprKind::FieldAccess { object, field } => {
            lowered.target = Some(Box::new(mir_expression(object)));
            lowered.field = Some(field.clone());
        }
        TypedExprKind::IndexAccess { object, index } => {
            lowered.target = Some(Box::new(mir_expression(object)));
            lowered.args = vec![mir_expression(index)];
        }
        TypedExprKind::StructLiteral { type_name, fields } => {
            lowered.name = Some(type_name.clone());
            lowered.args = fields
                .iter()
                .map(|(_, value)| mir_expression(value))
                .collect();
        }
        TypedExprKind::EnumVariant {
            type_name,
            variant,
            payload,
        } => {
            lowered.name = Some(format!("{type_name}.{variant}"));
            if let Some(payload) = payload {
                lowered.args = vec![mir_expression(payload)];
            }
        }
        TypedExprKind::ArrayLiteral { elements } => {
            lowered.args = elements.iter().map(mir_expression).collect();
        }
        TypedExprKind::Match {
            scrutinee,
            arms,
            kind,
        } => {
            lowered.target = Some(Box::new(mir_expression(scrutinee)));
            lowered.match_kind = Some(mir_match_kind(kind));
            lowered.arms = arms.iter().map(mir_match_arm).collect();
        }
        TypedExprKind::Cast {
            expr,
            from_type,
            to_type,
        } => {
            lowered.target = Some(Box::new(mir_expression(expr)));
            lowered.name = Some(format!(
                "{}->{}",
                from_type.display_name(),
                to_type.display_name()
            ));
        }
        TypedExprKind::Ref(inner) | TypedExprKind::MutRef(inner) | TypedExprKind::Deref(inner) => {
            lowered.target = Some(Box::new(mir_expression(inner)));
        }
        TypedExprKind::Closure { fn_name, .. } => {
            lowered.function = Some(fn_name.clone());
        }
        TypedExprKind::StringInterpolation { parts } => {
            lowered.value = Some(serde_json::json!({ "parts": parts.len() }));
        }
        TypedExprKind::Intrinsic { name, args } => {
            lowered.function = Some(name.clone());
            lowered.args = args.iter().map(mir_expression).collect();
        }
        TypedExprKind::Assign { target, value } => {
            lowered.target = Some(Box::new(mir_expression(target)));
            lowered.value = Some(serde_json::to_value(mir_expression(value)).unwrap_or_default());
        }
        TypedExprKind::Block(block) => {
            let result = block
                .expr
                .as_ref()
                .map(|expr| serde_json::to_value(mir_expression(expr)).unwrap_or_default());
            lowered.value = Some(serde_json::json!({
                "statement_count": block.statements.len(),
                "has_result": block.expr.is_some(),
                "result": result,
            }));
        }
        TypedExprKind::LoopControl { action, label } => {
            lowered.op = Some(action.as_str());
            lowered.name = Some(label.clone());
        }
        TypedExprKind::Break | TypedExprKind::Continue | TypedExprKind::Error => {}
    }

    lowered
}

fn mir_match_kind(kind: &MatchKind) -> &'static str {
    match kind {
        MatchKind::ConditionalElse => "conditional_else",
        MatchKind::Conditional => "conditional",
        MatchKind::WhileLoop => "while_loop",
        MatchKind::ControlledLoop { .. } => "controlled_loop",
        MatchKind::EnumMatch => "enum",
        MatchKind::ValueMatch => "value",
    }
}

fn mir_expression_kind(expr: &TypedExpression) -> &'static str {
    match &expr.kind {
        TypedExprKind::IntLiteral(_) => "int",
        TypedExprKind::FloatLiteral(_) => "float",
        TypedExprKind::StringLiteral(_) => "static_string",
        TypedExprKind::BoolLiteral(_) => "bool",
        TypedExprKind::Variable(_) => "local",
        TypedExprKind::BinaryOp { .. } => "binary",
        TypedExprKind::UnaryOp { .. } => "unary",
        TypedExprKind::FunctionCall { .. } => "call",
        TypedExprKind::FieldAccess { .. } => "field",
        TypedExprKind::IndexAccess { .. } => "index",
        TypedExprKind::StructLiteral { .. } => "struct",
        TypedExprKind::EnumVariant { .. } => "enum_variant",
        TypedExprKind::ArrayLiteral { .. } => "array",
        TypedExprKind::Match { .. } => "match",
        TypedExprKind::Cast { .. } => "cast",
        TypedExprKind::Ref(_) => "ref",
        TypedExprKind::MutRef(_) => "mut_ref",
        TypedExprKind::Deref(_) => "deref",
        TypedExprKind::Closure { .. } => "closure",
        TypedExprKind::StringInterpolation { .. } => "string_interpolation",
        TypedExprKind::Intrinsic { .. } => "intrinsic",
        TypedExprKind::Assign { .. } => "assign",
        TypedExprKind::Block(_) => "block",
        TypedExprKind::Break => "break",
        TypedExprKind::Continue => "continue",
        TypedExprKind::LoopControl { .. } => "loop_control",
        TypedExprKind::Error => "error",
    }
}
