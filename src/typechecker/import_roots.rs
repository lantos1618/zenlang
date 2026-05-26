use std::fmt;

use crate::root_spelling::{AT_STD_ROOT, STD_ROOT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RootImportPath {
    Std,
    AtStd,
}

impl RootImportPath {
    const ALL: &[RootImportPath] = &[RootImportPath::Std, RootImportPath::AtStd];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Std => STD_ROOT,
            Self::AtStd => AT_STD_ROOT,
        }
    }

    fn matches_path(self, path: &[String]) -> bool {
        path.len() == 1 && path.first().is_some_and(|segment| segment == self.as_str())
    }
}

impl fmt::Display for RootImportPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub(super) fn parse_root_import_path(path: &[String]) -> Option<RootImportPath> {
    RootImportPath::ALL
        .iter()
        .copied()
        .find(|root| root.matches_path(path))
}
