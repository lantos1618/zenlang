use super::BuildTargetDslIdent;
use std::{fmt, str::FromStr};

impl BuildTargetDslIdent {
    const ALL: &[BuildTargetDslIdent] = &[
        BuildTargetDslIdent::Builder,
        BuildTargetDslIdent::Add,
        BuildTargetDslIdent::Build,
        BuildTargetDslIdent::Env,
        BuildTargetDslIdent::Os,
        BuildTargetDslIdent::ReadFile,
    ];
    const BUILDER: &'static str = "b";
    const ADD: &'static str = "add";
    const BUILD: &'static str = "build";
    const ENV: &'static str = "env";
    const OS: &'static str = "os";
    const READ_FILE: &'static str = "read_file";

    pub(in crate::build_graph) fn as_str(self) -> &'static str {
        match self {
            Self::Builder => Self::BUILDER,
            Self::Add => Self::ADD,
            Self::Build => Self::BUILD,
            Self::Env => Self::ENV,
            Self::Os => Self::OS,
            Self::ReadFile => Self::READ_FILE,
        }
    }
}

impl fmt::Display for BuildTargetDslIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuildTargetDslIdent {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|ident| ident.as_str() == value)
            .ok_or(())
    }
}
