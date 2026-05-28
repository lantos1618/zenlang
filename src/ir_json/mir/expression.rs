use crate::ast::typed::{MatchKind, TypedExprKind, TypedExpression};

use super::mir_match_arm;
use super::schema::MirExpression;

pub(super) fn mir_expression(expr: &TypedExpression) -> MirExpression {
    let mut lowered = MirExpression {
        ty: expr.ty.display_name(),
        ..Default::default()
    };

    lowered.kind = match &expr.kind {
        TypedExprKind::IntLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
            "int"
        }
        TypedExprKind::FloatLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
            "float"
        }
        TypedExprKind::StringLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
            "static_string"
        }
        TypedExprKind::BoolLiteral(value) => {
            lowered.value = Some(serde_json::json!(value));
            "bool"
        }
        TypedExprKind::Variable(name) => {
            lowered.name = Some(name.clone());
            "local"
        }
        TypedExprKind::BinaryOp { op, left, right } => {
            lowered.op = Some(op.symbol());
            lowered.left = Some(Box::new(mir_expression(left)));
            lowered.right = Some(Box::new(mir_expression(right)));
            "binary"
        }
        TypedExprKind::UnaryOp { op, operand } => {
            lowered.op = Some(op.symbol());
            lowered.target = Some(Box::new(mir_expression(operand)));
            "unary"
        }
        TypedExprKind::FunctionCall { function, args } => {
            lowered.function = Some(function.clone());
            lowered.args = args.iter().map(mir_expression).collect();
            "call"
        }
        TypedExprKind::FieldAccess { object, field } => {
            lowered.target = Some(Box::new(mir_expression(object)));
            lowered.field = Some(field.clone());
            "field"
        }
        TypedExprKind::IndexAccess { object, index } => {
            lowered.target = Some(Box::new(mir_expression(object)));
            lowered.args = vec![mir_expression(index)];
            "index"
        }
        TypedExprKind::StructLiteral { type_name, fields } => {
            lowered.name = Some(type_name.clone());
            lowered.args = fields
                .iter()
                .map(|(_, value)| mir_expression(value))
                .collect();
            "struct"
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
            "enum_variant"
        }
        TypedExprKind::ArrayLiteral { elements } => {
            lowered.args = elements.iter().map(mir_expression).collect();
            "array"
        }
        TypedExprKind::Match {
            scrutinee,
            arms,
            kind,
        } => {
            lowered.target = Some(Box::new(mir_expression(scrutinee)));
            lowered.match_kind = Some(mir_match_kind(kind));
            lowered.arms = arms.iter().map(mir_match_arm).collect();
            "match"
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
            "cast"
        }
        TypedExprKind::Ref(inner) => {
            lowered.target = Some(Box::new(mir_expression(inner)));
            "ref"
        }
        TypedExprKind::MutRef(inner) => {
            lowered.target = Some(Box::new(mir_expression(inner)));
            "mut_ref"
        }
        TypedExprKind::Deref(inner) => {
            lowered.target = Some(Box::new(mir_expression(inner)));
            "deref"
        }
        TypedExprKind::StringInterpolation { parts } => {
            lowered.value = Some(serde_json::json!({ "parts": parts.len() }));
            "string_interpolation"
        }
        TypedExprKind::Assign { target, value } => {
            lowered.target = Some(Box::new(mir_expression(target)));
            lowered.value = Some(serde_json::to_value(mir_expression(value)).unwrap_or_default());
            "assign"
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
            "block"
        }
        TypedExprKind::LoopControl { action, label } => {
            lowered.op = Some(action.as_str());
            lowered.name = Some(label.clone());
            "loop_control"
        }
    };

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
