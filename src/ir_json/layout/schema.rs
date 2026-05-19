use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Serialize)]
pub(super) struct LayoutJsonProgram {
    pub(super) format: &'static str,
    pub(super) schema_version: u32,
    pub(super) semantic_status: &'static str,
    pub(super) target: LayoutJsonTarget,
    pub(super) layouts: BTreeMap<String, LayoutJsonType>,
}

#[derive(Serialize)]
pub(super) struct LayoutJsonTarget {
    pub(super) pointer_size: u32,
    pub(super) pointer_alignment: u32,
    pub(super) usize_size: u32,
}

#[derive(Clone, Serialize)]
pub(super) struct LayoutJsonType {
    pub(super) kind: &'static str,
    pub(super) size: u32,
    pub(super) alignment: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) fields: Vec<LayoutJsonField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) variants: Vec<LayoutJsonVariant>,
}

#[derive(Clone, Serialize)]
pub(super) struct LayoutJsonField {
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) offset: u32,
}

#[derive(Clone, Serialize)]
pub(super) struct LayoutJsonVariant {
    pub(super) name: String,
    pub(super) tag: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) payload_fields: Vec<LayoutJsonField>,
}
