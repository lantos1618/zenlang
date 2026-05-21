use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleRootPrefix {
    Std,
    AtStd,
    Builtin,
}

impl ModuleRootPrefix {
    const STD: &'static str = "std";
    const AT_STD: &'static str = "@std";
    const AT_BUILTIN: &'static str = "@builtin";
    const ALL: &[ModuleRootPrefix] = &[
        ModuleRootPrefix::Std,
        ModuleRootPrefix::AtStd,
        ModuleRootPrefix::Builtin,
    ];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Std => Self::STD,
            Self::AtStd => Self::AT_STD,
            Self::Builtin => Self::AT_BUILTIN,
        }
    }

    pub(super) const fn is_std(self) -> bool {
        matches!(self, Self::Std | Self::AtStd)
    }

    pub(super) const fn is_builtin(self) -> bool {
        matches!(self, Self::Builtin)
    }
}

impl fmt::Display for ModuleRootPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ModuleRootPrefix {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|prefix| prefix.as_str() == value)
            .ok_or(())
    }
}

pub(super) fn parse_module_root_prefix(value: &str) -> Option<ModuleRootPrefix> {
    value.parse().ok()
}
