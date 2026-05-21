use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::parser) enum ParserModuleRoot {
    AtBuiltin,
    AtStd,
}

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

impl FromStr for ParserModuleRoot {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|root| root.as_str() == value)
            .ok_or(())
    }
}
