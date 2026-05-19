use serde::Serialize;
use std::fmt;
use std::str::FromStr;

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    And,
    Or,

    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl BinaryOp {
    /// Returns the operator symbol for display/error messages.
    pub fn symbol(&self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
            BinaryOp::BitAnd => "&",
            BinaryOp::BitOr => "|",
            BinaryOp::BitXor => "^",
            BinaryOp::ShiftLeft => "<<",
            BinaryOp::ShiftRight => ">>",
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum UnaryOp {
    Neg,    // -x
    Not,    // !x
    BitNot, // ~x
}

impl UnaryOp {
    pub fn symbol(&self) -> &'static str {
        match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
            UnaryOp::BitNot => "~",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LoopControlAction {
    Done,
    Next,
}

impl LoopControlAction {
    pub const DONE: &'static str = "done";
    pub const NEXT: &'static str = "next";

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Done => Self::DONE,
            Self::Next => Self::NEXT,
        }
    }
}

impl fmt::Display for LoopControlAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for LoopControlAction {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == Self::Done.as_str() {
            Ok(Self::Done)
        } else if value == Self::Next.as_str() {
            Ok(Self::Next)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_control_action_owns_text_spelling() {
        assert_eq!(LoopControlAction::Done.as_str(), "done");
        assert_eq!(LoopControlAction::Next.as_str(), "next");
        assert_eq!(
            "done".parse::<LoopControlAction>(),
            Ok(LoopControlAction::Done)
        );
        assert_eq!(
            "next".parse::<LoopControlAction>(),
            Ok(LoopControlAction::Next)
        );
        assert!("stop".parse::<LoopControlAction>().is_err());
        assert_eq!(LoopControlAction::Done.to_string(), "done");
        assert_eq!(LoopControlAction::Next.to_string(), "next");
    }
}
