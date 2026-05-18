use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum TargetYamlError {
    Io(std::io::Error),
    Parse(serde_yaml::Error),
    Schema(String),
    Json(serde_json::Error),
}

impl std::fmt::Display for TargetYamlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "failed to read target YAML: {err}"),
            Self::Parse(err) => write!(f, "invalid target YAML: {err}"),
            Self::Schema(message) => f.write_str(message),
            Self::Json(err) => write!(f, "json emit error: {err}"),
        }
    }
}

impl std::error::Error for TargetYamlError {}

impl From<std::io::Error> for TargetYamlError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_yaml::Error> for TargetYamlError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::Parse(err)
    }
}

impl From<serde_json::Error> for TargetYamlError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetYamlInput {
    triple: String,
    pointer_width: u32,
    endianness: Endianness,
    abi: String,
    #[serde(default)]
    layout: Option<serde_yaml::Value>,
    #[serde(default)]
    overrides: Option<serde_yaml::Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Endianness {
    Little,
    Big,
}

#[derive(Serialize)]
struct TargetJson {
    format: &'static str,
    semantic_status: &'static str,
    target: TargetJsonBody,
}

#[derive(Serialize)]
struct TargetJsonBody {
    triple: String,
    pointer_width: u32,
    endianness: Endianness,
    abi: String,
}

pub fn target_yaml_file_to_json(path: &Path) -> Result<String, TargetYamlError> {
    let source = fs::read_to_string(path)?;
    target_yaml_to_json(&source)
}

pub fn target_yaml_to_json(source: &str) -> Result<String, TargetYamlError> {
    let input: TargetYamlInput = serde_yaml::from_str(source)?;
    validate_target_yaml(&input)?;
    let json = TargetJson {
        format: "zen.target.v0",
        semantic_status: "validated",
        target: TargetJsonBody {
            triple: input.triple,
            pointer_width: input.pointer_width,
            endianness: input.endianness,
            abi: input.abi,
        },
    };
    Ok(serde_json::to_string_pretty(&json)?)
}

fn validate_target_yaml(input: &TargetYamlInput) -> Result<(), TargetYamlError> {
    if input.layout.is_some() || input.overrides.is_some() {
        return Err(TargetYamlError::Schema(
            "target YAML cannot override compiler-owned type layouts".into(),
        ));
    }
    if input.triple.trim().is_empty() {
        return Err(TargetYamlError::Schema(
            "target YAML `triple` cannot be empty".into(),
        ));
    }
    if !matches!(input.pointer_width, 32 | 64) {
        return Err(TargetYamlError::Schema(
            "target YAML `pointer_width` must be 32 or 64".into(),
        ));
    }
    if input.abi.trim().is_empty() {
        return Err(TargetYamlError::Schema(
            "target YAML `abi` cannot be empty".into(),
        ));
    }
    Ok(())
}
