use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    And,
    Or,

    BitAnd,
    BitOr,
    BitXor,
    ShiftLeft,
    ShiftRight,
}

impl BinaryOp {
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

const LOOP_CONTROL_ACTION_SPELLINGS: &[(LoopControlAction, &str)] = &[
    (LoopControlAction::Done, "done"),
    (LoopControlAction::Next, "next"),
];

impl LoopControlAction {
    pub fn as_str(self) -> &'static str {
        crate::static_spelling::static_spelling(LOOP_CONTROL_ACTION_SPELLINGS, self)
    }
}

crate::static_spelling::impl_static_spelling_display!(
    LoopControlAction,
    table = LOOP_CONTROL_ACTION_SPELLINGS
);
crate::static_spelling::impl_static_spelling_from_str!(
    LoopControlAction,
    table = LOOP_CONTROL_ACTION_SPELLINGS
);
