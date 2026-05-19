use serde::Serialize;

#[derive(Serialize)]
pub(super) struct MirJsonProgram {
    pub(super) format: &'static str,
    pub(super) schema_version: u32,
    pub(super) semantic_status: &'static str,
    pub(super) lowering_status: &'static str,
    pub(super) functions: Vec<MirFunction>,
}

#[derive(Serialize)]
pub(super) struct MirFunction {
    pub(super) name: String,
    pub(super) params: Vec<MirParam>,
    pub(super) return_type: String,
    pub(super) blocks: Vec<MirBlock>,
}

#[derive(Serialize)]
pub(super) struct MirParam {
    pub(super) name: String,
    pub(super) r#type: String,
}

#[derive(Serialize)]
pub(super) struct MirBlock {
    pub(super) label: &'static str,
    pub(super) statements: Vec<MirStatement>,
    pub(super) terminator: MirTerminator,
}

#[derive(Serialize)]
pub(super) struct MirStatement {
    pub(super) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub(super) ty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) mutable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<MirExpression>,
}

#[derive(Serialize)]
pub(super) struct MirTerminator {
    pub(super) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<MirExpression>,
}

#[derive(Serialize)]
pub(super) struct MirExpression {
    pub(super) kind: &'static str,
    #[serde(rename = "type")]
    pub(super) ty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) op: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) left: Option<Box<MirExpression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) right: Option<Box<MirExpression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) target: Option<Box<MirExpression>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) match_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) args: Vec<MirExpression>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) arms: Vec<MirMatchArm>,
}

#[derive(Serialize)]
pub(super) struct MirMatchArm {
    pub(super) pattern: MirPattern,
    pub(super) body: MirBlock,
}

#[derive(Serialize)]
pub(super) struct MirPattern {
    pub(super) kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) value: Option<serde_json::Value>,
    pub(super) bindings: Vec<MirPatternBinding>,
}

#[derive(Serialize)]
pub(super) struct MirPatternBinding {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) ty: String,
}
