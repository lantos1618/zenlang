use std::fs;
use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};

mod error;
mod validation;

pub use error::TargetYamlError;
use validation::validate_target_yaml;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetYamlInput {
    triple: String,
    pointer_width: u32,
    endianness: Endianness,
    abi: String,
    #[serde(default)]
    backend: Option<TargetBackendInput>,
    #[serde(default)]
    layout: Option<serde_yaml::Value>,
    #[serde(default)]
    overrides: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetBackendInput {
    codegen: TargetBackendCodegen,
    #[serde(default)]
    c_compiler: Option<String>,
    #[serde(default)]
    c_flags: Vec<String>,
}

#[derive(Debug)]
enum TargetBackendCodegen {
    C,
    Unsupported,
}

impl TargetBackendCodegen {
    const C_SPELLING: &'static str = "c";
}

impl<'de> Deserialize<'de> for TargetBackendCodegen {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let spelling = String::deserialize(deserializer)?;
        Ok(match spelling.as_str() {
            Self::C_SPELLING => Self::C,
            _ => Self::Unsupported,
        })
    }
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
    schema_version: u32,
    semantic_status: &'static str,
    target: TargetJsonBody,
}

#[derive(Serialize)]
struct TargetJsonBody {
    triple: String,
    pointer_width: u32,
    endianness: Endianness,
    abi: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    backend: Option<TargetBackendJson>,
}

#[derive(Serialize)]
struct TargetBackendJson {
    codegen: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    c_compiler: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    c_flags: Vec<String>,
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
        schema_version: 0,
        semantic_status: "validated",
        target: TargetJsonBody {
            triple: input.triple,
            pointer_width: input.pointer_width,
            endianness: input.endianness,
            abi: input.abi,
            backend: input.backend.map(TargetBackendJson::from),
        },
    };
    Ok(serde_json::to_string_pretty(&json)?)
}

impl From<TargetBackendInput> for TargetBackendJson {
    fn from(input: TargetBackendInput) -> Self {
        let codegen = match input.codegen {
            TargetBackendCodegen::C => TargetBackendCodegen::C_SPELLING,
            TargetBackendCodegen::Unsupported => {
                unreachable!("target backend codegen is validated before JSON emission")
            }
        };
        Self {
            codegen,
            c_compiler: input.c_compiler,
            c_flags: input.c_flags,
        }
    }
}
