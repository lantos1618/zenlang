use super::BuildTargetDslKind;

impl BuildTargetDslKind {
    const ALL: &[BuildTargetDslKind] = &[
        BuildTargetDslKind::Executable,
        BuildTargetDslKind::Test,
        BuildTargetDslKind::Library,
    ];
    const EXECUTABLE: &'static str = "Executable";
    const TEST: &'static str = "Test";
    const LIBRARY: &'static str = "Library";

    pub(in crate::build_graph) fn as_str(self) -> &'static str {
        match self {
            Self::Executable => Self::EXECUTABLE,
            Self::Test => Self::TEST,
            Self::Library => Self::LIBRARY,
        }
    }

    pub(in crate::build_graph) fn supported_display_list() -> String {
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

crate::static_spelling::impl_static_spelling_display!(
    BuildTargetDslKind,
    as_str = BuildTargetDslKind::as_str
);
crate::static_spelling::impl_static_spelling_from_str!(
    BuildTargetDslKind,
    variants = BuildTargetDslKind::ALL,
    as_str = BuildTargetDslKind::as_str
);
