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
