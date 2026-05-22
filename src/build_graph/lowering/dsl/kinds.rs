use super::BuildTargetDslKind;
use std::{fmt, str::FromStr};

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
