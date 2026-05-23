use super::ParserModuleRoot;

impl ParserModuleRoot {
    const ALL: &[ParserModuleRoot] = &[ParserModuleRoot::AtBuiltin, ParserModuleRoot::AtStd];

    const AT_BUILTIN: &'static str = "@builtin";
    const AT_STD: &'static str = "@std";

    pub(in crate::parser) fn as_str(self) -> &'static str {
        match self {
            Self::AtBuiltin => Self::AT_BUILTIN,
            Self::AtStd => Self::AT_STD,
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
