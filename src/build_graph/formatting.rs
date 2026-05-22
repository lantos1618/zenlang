use std::fmt;

use super::{BuildTargetKind, HostEffect};

impl BuildTargetKind {
    pub fn diagnostic_name(&self) -> &'static str {
        match self {
            Self::Executable { .. } => "executable",
            Self::Test { .. } => "test",
            Self::Library { .. } => "library",
        }
    }
}

impl fmt::Display for BuildTargetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.diagnostic_name())
    }
}

impl fmt::Display for HostEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadEnv(name) => write!(f, "read env `{name}`"),
            Self::ReadFile(path) => write!(f, "read file `{path}`"),
        }
    }
}
