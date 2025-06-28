use super::core::Parser;
use crate::error::Result;

impl<'a> Parser<'a> {
    pub fn parse_comptime(&mut self) -> Result<()> {
        // TODO: Implement compile-time parsing
        unimplemented!("Compile-time parsing not yet implemented");
    }
}
