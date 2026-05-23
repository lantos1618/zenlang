use crate::root_spelling::{AT_STD_ROOT, STD_ROOT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleRootPrefix {
    Std,
    AtStd,
}

impl ModuleRootPrefix {
    const ALL: &[ModuleRootPrefix] = &[ModuleRootPrefix::Std, ModuleRootPrefix::AtStd];

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Std => STD_ROOT,
            Self::AtStd => AT_STD_ROOT,
        }
    }

    pub(super) const fn is_std(self) -> bool {
        matches!(self, Self::Std | Self::AtStd)
    }
}

crate::static_spelling::impl_static_spelling_display!(
    ModuleRootPrefix,
    as_str = ModuleRootPrefix::as_str
);
crate::static_spelling::impl_static_spelling_from_str!(
    ModuleRootPrefix,
    variants = ModuleRootPrefix::ALL,
    as_str = ModuleRootPrefix::as_str
);

pub(super) fn parse_module_root_prefix(value: &str) -> Option<ModuleRootPrefix> {
    value.parse().ok()
}
