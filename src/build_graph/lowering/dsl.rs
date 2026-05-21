use std::{fmt, str::FromStr};

#[path = "dsl/target_fields.rs"]
mod target_fields;

pub(super) use target_fields::BuildTargetField;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuildTargetDslKind {
    Executable,
    Test,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuildTargetDslIdent {
    Builder,
    Add,
    Build,
    Env,
    Os,
    ReadFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostEffectResultVariant {
    Ok,
    Err,
}

impl BuildTargetDslKind {
    const ALL: &[BuildTargetDslKind] = &[
        BuildTargetDslKind::Executable,
        BuildTargetDslKind::Test,
        BuildTargetDslKind::Library,
    ];
    const EXECUTABLE: &'static str = "Executable";
    const TEST: &'static str = "Test";
    const LIBRARY: &'static str = "Library";

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Executable => Self::EXECUTABLE,
            Self::Test => Self::TEST,
            Self::Library => Self::LIBRARY,
        }
    }

    pub(super) fn supported_display_list() -> String {
        let names = Self::ALL
            .iter()
            .map(|kind| format!("`{kind}`"))
            .collect::<Vec<_>>();
        let Some((last, rest)) = names.split_last() else {
            return String::new();
        };
        if rest.is_empty() {
            return last.clone();
        }
        format!("{}, and {last}", rest.join(", "))
    }
}

impl HostEffectResultVariant {
    const ALL: &[HostEffectResultVariant] =
        &[HostEffectResultVariant::Ok, HostEffectResultVariant::Err];
    const OK: &'static str = "Ok";
    const ERR: &'static str = "Err";

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Ok => Self::OK,
            Self::Err => Self::ERR,
        }
    }
}

impl fmt::Display for HostEffectResultVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HostEffectResultVariant {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, <Self as FromStr>::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|variant| variant.as_str() == value)
            .ok_or(())
    }
}

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

    pub(super) fn as_str(self) -> &'static str {
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

impl fmt::Display for BuildTargetDslKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuildTargetDslKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.as_str() == value)
            .ok_or(())
    }
}
