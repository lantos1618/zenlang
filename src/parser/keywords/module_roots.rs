use super::ParserModuleRoot;
use crate::root_spelling::{AT_BUILTIN_ROOT, AT_STD_ROOT};

impl ParserModuleRoot {
    const ALL: &[ParserModuleRoot] = &[ParserModuleRoot::AtBuiltin, ParserModuleRoot::AtStd];

    pub(in crate::parser) fn as_str(self) -> &'static str {
        match self {
            Self::AtBuiltin => AT_BUILTIN_ROOT,
            Self::AtStd => AT_STD_ROOT,
        }
    }

    pub(in crate::parser) fn join_module_parts(self, parts: &[String]) -> String {
        if parts.is_empty() {
            self.as_str().to_string()
        } else {
            format!("{}.{}", self.as_str(), parts.join("."))
        }
    }
}

crate::static_spelling::impl_static_spelling_from_str!(
    ParserModuleRoot,
    variants = ParserModuleRoot::ALL,
    as_str = ParserModuleRoot::as_str
);
