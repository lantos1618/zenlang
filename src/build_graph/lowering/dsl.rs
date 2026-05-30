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
    Headers,
}

pub(in crate::build_graph) const BUILDER_IDENT: &str = "b";
pub(in crate::build_graph) const BUILDER_ADD_METHOD: &str = "add";
pub(in crate::build_graph) const BUILD_FUNCTION_NAME: &str = "build";
pub(in crate::build_graph) const ENV_METHOD: &str = "env";
pub(in crate::build_graph) const OS_FIELD: &str = "os";
pub(in crate::build_graph) const READ_FILE_METHOD: &str = "read_file";

const TARGET_KIND_SPELLINGS: &[(BuildTargetDslKind, &str)] = &[
    (BuildTargetDslKind::Executable, "Executable"),
    (BuildTargetDslKind::Test, "Test"),
    (BuildTargetDslKind::Library, "Library"),
];
pub(in crate::build_graph) const SUPPORTED_TARGET_KINDS: &str =
    "`Executable`, `Test`, and `Library`";

const TARGET_FIELD_SPELLINGS: &[(BuildTargetField, &str)] = &[
    (BuildTargetField::Name, "name"),
    (BuildTargetField::Main, "main"),
    (BuildTargetField::Root, "root"),
    (BuildTargetField::RootSourceFile, "root_source_file"),
    (BuildTargetField::OutDir, "out_dir"),
    (BuildTargetField::Dependencies, "dependencies"),
    (BuildTargetField::Features, "features"),
    (BuildTargetField::Exports, "exports"),
    (BuildTargetField::Packages, "packages"),
    (BuildTargetField::Link, "link"),
    (BuildTargetField::Headers, "headers"),
];

impl BuildTargetField {
    pub(in crate::build_graph) fn as_str(self) -> &'static str {
        crate::static_spelling::static_spelling(TARGET_FIELD_SPELLINGS, self)
    }
}

crate::static_spelling::impl_static_spelling_display!(
    BuildTargetDslKind,
    table = TARGET_KIND_SPELLINGS
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuildTargetDslKind,
    table = TARGET_KIND_SPELLINGS
);
crate::static_spelling::impl_static_spelling_display!(
    BuildTargetField,
    table = TARGET_FIELD_SPELLINGS
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuildTargetField,
    table = TARGET_FIELD_SPELLINGS
);
