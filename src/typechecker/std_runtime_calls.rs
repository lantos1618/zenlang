use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StdRuntimeCall {
    IoPrint,
    IoPrintln,
}

impl StdRuntimeCall {
    const IO: &'static str = "io";
    const PRINT: &'static str = "print";
    const PRINTLN: &'static str = "println";
    const ALL: &[StdRuntimeCall] = &[StdRuntimeCall::IoPrint, StdRuntimeCall::IoPrintln];

    const fn module(self) -> &'static str {
        match self {
            Self::IoPrint | Self::IoPrintln => Self::IO,
        }
    }

    const fn function(self) -> &'static str {
        match self {
            Self::IoPrint => Self::PRINT,
            Self::IoPrintln => Self::PRINTLN,
        }
    }
}

impl fmt::Display for StdRuntimeCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.module(), self.function())
    }
}

pub(super) fn parse_std_runtime_call(module: &str, function: &str) -> Option<StdRuntimeCall> {
    StdRuntimeCall::ALL
        .iter()
        .copied()
        .find(|call| call.module() == module && call.function() == function)
}
