use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleRootPrefix {
    Std,
    AtStd,
}

impl ModuleRootPrefix {
    const STD: &'static str = "std";
    const AT_STD: &'static str = "@std";
    const ALL: &[ModuleRootPrefix] = &[ModuleRootPrefix::Std, ModuleRootPrefix::AtStd];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Std => Self::STD,
            Self::AtStd => Self::AT_STD,
        }
    }

    pub(super) const fn is_std(self) -> bool {
        matches!(self, Self::Std | Self::AtStd)
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
        crate::static_spelling::parse_static_spelling(Self::ALL, value, Self::as_str)
    }
}

pub(super) fn parse_module_root_prefix(value: &str) -> Option<ModuleRootPrefix> {
    value.parse().ok()
}
