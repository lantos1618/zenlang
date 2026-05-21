use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::build_graph) enum BuildTargetField {
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

impl BuildTargetField {
    const ALL: &[BuildTargetField] = &[
        BuildTargetField::Name,
        BuildTargetField::Main,
        BuildTargetField::Root,
        BuildTargetField::RootSourceFile,
        BuildTargetField::OutDir,
        BuildTargetField::Dependencies,
        BuildTargetField::Features,
        BuildTargetField::Exports,
        BuildTargetField::Packages,
        BuildTargetField::Link,
    ];
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

    pub(in crate::build_graph) fn as_str(self) -> &'static str {
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

    pub(in crate::build_graph) fn is_package_link_semantics(self) -> bool {
        matches!(self, Self::Packages | Self::Link)
    }
}

impl FromStr for BuildTargetField {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|field| field.as_str() == value)
            .ok_or(())
    }
}

impl fmt::Display for BuildTargetField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
