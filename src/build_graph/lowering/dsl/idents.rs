use super::BuildTargetDslIdent;

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

crate::static_spelling::impl_static_spelling_display!(
    BuildTargetDslIdent,
    as_str = BuildTargetDslIdent::as_str
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuildTargetDslIdent,
    variants = BuildTargetDslIdent::ALL,
    as_str = BuildTargetDslIdent::as_str
);
