use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuildTargetDslKind {
    Executable,
    Test,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum BuildTargetField {
    Name,
    Main,
    Root,
    RootSourceFile,
    OutDir,
    Dependencies,
    Features,
    Exports,
    Packages,
    Link,
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
    const ALL: [Self; 3] = [Self::Executable, Self::Test, Self::Library];
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

impl BuildTargetField {
    const NAME: &'static str = "name";
    const MAIN: &'static str = "main";
    const ROOT: &'static str = "root";
    const ROOT_SOURCE_FILE: &'static str = "root_source_file";
    const OUT_DIR: &'static str = "out_dir";
    const DEPENDENCIES: &'static str = "dependencies";
    const FEATURES: &'static str = "features";
    const EXPORTS: &'static str = "exports";
    const PACKAGES: &'static str = "packages";
    const LINK: &'static str = "link";

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Name => Self::NAME,
            Self::Main => Self::MAIN,
            Self::Root => Self::ROOT,
            Self::RootSourceFile => Self::ROOT_SOURCE_FILE,
            Self::OutDir => Self::OUT_DIR,
            Self::Dependencies => Self::DEPENDENCIES,
            Self::Features => Self::FEATURES,
            Self::Exports => Self::EXPORTS,
            Self::Packages => Self::PACKAGES,
            Self::Link => Self::LINK,
        }
    }

    pub(super) fn is_package_link_semantics(self) -> bool {
        matches!(self, Self::Packages | Self::Link)
    }
}

impl FromStr for BuildTargetField {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            Self::NAME => Ok(Self::Name),
            Self::MAIN => Ok(Self::Main),
            Self::ROOT => Ok(Self::Root),
            Self::ROOT_SOURCE_FILE => Ok(Self::RootSourceFile),
            Self::OUT_DIR => Ok(Self::OutDir),
            Self::DEPENDENCIES => Ok(Self::Dependencies),
            Self::FEATURES => Ok(Self::Features),
            Self::EXPORTS => Ok(Self::Exports),
            Self::PACKAGES => Ok(Self::Packages),
            Self::LINK => Ok(Self::Link),
            _ => Err(()),
        }
    }
}

impl HostEffectResultVariant {
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
        match value {
            Self::OK => Ok(Self::Ok),
            Self::ERR => Ok(Self::Err),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BuildTargetField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl BuildTargetDslIdent {
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

impl fmt::Display for BuildTargetDslKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuildTargetDslKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            Self::EXECUTABLE => Ok(Self::Executable),
            Self::TEST => Ok(Self::Test),
            Self::LIBRARY => Ok(Self::Library),
            _ => Err(()),
        }
    }
}
