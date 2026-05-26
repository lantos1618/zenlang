use super::BuildTargetField;

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

crate::static_spelling::impl_static_spelling_display!(
    BuildTargetField,
    as_str = BuildTargetField::as_str
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuildTargetField,
    variants = BuildTargetField::ALL,
    as_str = BuildTargetField::as_str
);
