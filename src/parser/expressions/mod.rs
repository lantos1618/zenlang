pub mod blocks;
pub mod calls;
pub mod collections;
pub mod control_flow;
pub mod literals;
pub mod operators;
pub mod patterns;
pub mod primary;
pub mod structs;

use super::core::Parser;
use crate::ast::Expression;
use crate::error::Result;

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.enter_recursion()?;
        let result = operators::parse_binary_expression(self, 0);
        self.exit_recursion();
        result
    }

    /// Parse expression in pattern context - doesn't allow `|` as bitwise OR
    /// since `|` is used as pattern alternative separator
    pub fn parse_pattern_expression(&mut self) -> Result<Expression> {
        operators::parse_pattern_expression(self)
    }
}
