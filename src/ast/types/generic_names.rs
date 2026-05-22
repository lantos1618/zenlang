use super::AstType;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinGenericTypeName {
    Ptr,
    MutPtr,
    RawPtr,
    Slice,
}

impl BuiltinGenericTypeName {
    pub const ALL: &[BuiltinGenericTypeName] = &[
        BuiltinGenericTypeName::Ptr,
        BuiltinGenericTypeName::MutPtr,
        BuiltinGenericTypeName::RawPtr,
        BuiltinGenericTypeName::Slice,
    ];
    const PTR: &'static str = "Ptr";
    const MUT_PTR: &'static str = "MutPtr";
    const RAW_PTR: &'static str = "RawPtr";
    const SLICE: &'static str = "Slice";

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ptr => Self::PTR,
            Self::MutPtr => Self::MUT_PTR,
            Self::RawPtr => Self::RAW_PTR,
            Self::Slice => Self::SLICE,
        }
    }

    pub fn ast_type(self, mut type_args: Vec<AstType>) -> Result<AstType, Vec<AstType>> {
        if type_args.len() != 1 {
            return Err(type_args);
        }
        let ty = Box::new(type_args.remove(0));
        Ok(match self {
            Self::Ptr => AstType::Ptr(ty),
            Self::MutPtr => AstType::MutPtr(ty),
            Self::RawPtr => AstType::RawPtr(ty),
            Self::Slice => AstType::Slice(ty),
        })
    }
}

impl fmt::Display for BuiltinGenericTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BuiltinGenericTypeName {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .copied()
            .find(|name| name.as_str() == value)
            .ok_or(())
    }
}
