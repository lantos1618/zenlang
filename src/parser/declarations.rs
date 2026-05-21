use super::*;
use crate::parser::keywords::ParserBehaviorKeyword;

mod function_forms;
mod suffix_forms;

impl Parser {
    pub(super) fn consume_mutability_keyword(&mut self) -> bool {
        use crate::parser::keywords::ParserMutabilityKeyword;

        if let Token::Identifier(ref name) = self.peek() {
            if name.parse::<ParserMutabilityKeyword>().is_ok() {
                self.advance();
                return true;
            }
        }
        false
    }

    // ── Declarations ──────────────────────────────────────────

    pub(super) fn parse_declaration(&mut self) -> Result<Declaration, CompileError> {
        self.skip_newlines();

        // pub prefix
        let public = if matches!(self.peek(), Token::Pub) {
            self.advance();
            self.skip_newlines();
            true
        } else {
            false
        };

        // Import: `{ names } = module.path`
        if matches!(self.peek(), Token::LBrace) && self.is_import() {
            return self.parse_import();
        }

        // Must be identifier-led
        let (name, name_span) = self.expect_identifier()?;

        self.skip_newlines();

        match self.peek() {
            // Behavior: `Name: behavior { method: (Self) Return }`
            Token::Colon
                if self
                    .colon_is_followed_by_identifier(ParserBehaviorKeyword::Behavior.as_str()) =>
            {
                self.parse_behavior_def(name, Vec::new(), public, name_span)
            }

            // Struct: `Name: { fields }`
            Token::Colon if self.is_struct_def() => self.parse_struct_def(name, public, name_span),

            // Enum: `Name: Variant1, Variant2` OR `Name:\n  Variant1,\n  Variant2`
            Token::Colon if self.is_enum_def() => self.parse_enum_def(name, public, name_span),

            // Generic type/function: `Name<T>...`
            Token::Lt => self.parse_generic_declaration_suffix(name, public, name_span),

            // Method: `Type.method = ...`
            Token::Dot => self.parse_type_suffix_declaration(name, public, name_span),

            // Function: `name = (params) ret { body }` or const `name = expr`
            Token::Assign => self.parse_function_def(name, Vec::new(), public, name_span),

            // Const assignment at top level: `name := expr`
            Token::ConstAssign => self.parse_const_decl(name, name_span),

            // Mutable decl assignment at top level: `name ::= expr`
            Token::DeclareAssign => self.parse_var_decl_toplevel(name, name_span),

            _ => Err(CompileError::Syntax(
                format!(
                    "unexpected token {:?} after identifier '{}'",
                    self.peek(),
                    name
                ),
                Some(self.peek_span()),
            )),
        }
    }
}
