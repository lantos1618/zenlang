use crate::ast::typed::{
    TypedBlock, TypedFunction, TypedMatchArm, TypedPattern, TypedProgram, TypedStatement,
    TypedStatementKind,
};

mod expression;
mod schema;

use expression::mir_expression;
use schema::{
    MirBlock, MirFunction, MirJsonProgram, MirMatchArm, MirNamedType, MirPattern, MirStatement,
    MirTerminator,
};

pub(super) fn program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let graph = MirJsonProgram {
        format: "zen.mir.v0",
        schema_version: 0,
        semantic_status: "checked",
        lowering_status: "minimal",
        functions: program.functions.iter().map(mir_function).collect(),
    };

    serde_json::to_string_pretty(&graph)
}

fn mir_function(function: &TypedFunction) -> MirFunction {
    MirFunction {
        name: function.name.clone(),
        params: function
            .params
            .iter()
            .map(|param| MirNamedType {
                name: param.name.clone(),
                r#type: param.ty.display_name(),
            })
            .collect(),
        return_type: function.return_type.display_name(),
        blocks: vec![mir_entry_block(&function.body)],
    }
}

fn mir_entry_block(body: &TypedBlock) -> MirBlock {
    MirBlock {
        label: "entry",
        statements: body.statements.iter().map(mir_statement).collect(),
        terminator: match &body.expr {
            Some(expr) => MirTerminator {
                kind: "return",
                value: Some(mir_expression(expr)),
            },
            None => MirTerminator {
                kind: "return_void",
                ..Default::default()
            },
        },
    }
}

fn mir_statement(statement: &TypedStatement) -> MirStatement {
    match &statement.kind {
        TypedStatementKind::VarDecl {
            name,
            ty,
            value,
            mutable,
        } => MirStatement {
            kind: "let",
            name: Some(name.clone()),
            ty: Some(ty.display_name()),
            mutable: Some(*mutable),
            value: Some(mir_expression(value)),
        },
        TypedStatementKind::Expression(expr) => MirStatement {
            kind: "expr",
            value: Some(mir_expression(expr)),
            ..Default::default()
        },
    }
}

pub(in crate::ir_json::mir) fn mir_match_arm(arm: &TypedMatchArm) -> MirMatchArm {
    MirMatchArm {
        pattern: mir_pattern(&arm.pattern),
        body: mir_entry_block(&arm.body),
    }
}

fn mir_pattern(pattern: &TypedPattern) -> MirPattern {
    match pattern {
        TypedPattern::Bool(value) => MirPattern {
            kind: "bool",
            value: Some(serde_json::json!(value)),
            ..Default::default()
        },
        TypedPattern::EnumVariant {
            type_name,
            variant,
            bindings,
        } => MirPattern {
            kind: "enum_variant",
            name: Some(format!("{type_name}.{variant}")),
            bindings: bindings
                .iter()
                .map(|(name, ty)| MirNamedType {
                    name: name.clone(),
                    r#type: ty.display_name(),
                })
                .collect(),
            ..Default::default()
        },
        TypedPattern::Wildcard => MirPattern {
            kind: "wildcard",
            ..Default::default()
        },
        TypedPattern::Value(value) => MirPattern {
            kind: "value",
            value: Some(serde_json::to_value(mir_expression(value)).unwrap_or_default()),
            ..Default::default()
        },
    }
}
