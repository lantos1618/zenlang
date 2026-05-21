use serde::Serialize;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TypeDeclarationKeyword {
    Impl,
    Implements,
    Requires,
    Extends,
    Derive,
}

impl TypeDeclarationKeyword {
    pub const ALL: &[TypeDeclarationKeyword] = &[
        TypeDeclarationKeyword::Impl,
        TypeDeclarationKeyword::Implements,
        TypeDeclarationKeyword::Requires,
        TypeDeclarationKeyword::Extends,
        TypeDeclarationKeyword::Derive,
    ];
    pub const IMPL: &'static str = "impl";
    pub const IMPLEMENTS: &'static str = "implements";
    pub const REQUIRES: &'static str = "requires";
    pub const EXTENDS: &'static str = "extends";
    pub const DERIVE: &'static str = "derive";

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Impl => Self::IMPL,
            Self::Implements => Self::IMPLEMENTS,
            Self::Requires => Self::REQUIRES,
            Self::Extends => Self::EXTENDS,
            Self::Derive => Self::DERIVE,
        }
    }
}

impl fmt::Display for TypeDeclarationKeyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for TypeDeclarationKeyword {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|keyword| keyword.as_str() == value)
            .ok_or(())
    }
}
