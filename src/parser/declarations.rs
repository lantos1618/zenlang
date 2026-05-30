use super::*;
use crate::parser::keywords::{BEHAVIOR_KEYWORD, MUT_KEYWORD};

mod function_forms;
mod generic;

impl Parser {
    pub(super) fn consume_mutability_keyword(&mut self) -> bool {
        if let Token::Identifier(ref name) = self.peek() {
            if name == MUT_KEYWORD {
                self.advance();
                return true;
            }
        }
        false
    }

    pub(super) fn parse_declaration(&mut self) -> Result<Declaration, CompileError> {
        self.skip_newlines();

        if matches!(self.peek(), Token::AtExport) {
            return self.parse_export_manifest();
        }

        let public = if matches!(self.peek(), Token::Pub) {
            self.advance();
            self.skip_newlines();
            true
        } else {
            false
        };

        if matches!(self.peek(), Token::Extern) {
            let extern_span = self.peek_span();
            self.advance();
            self.skip_newlines();
            return self.parse_extern_function(public, extern_span);
        }

        if matches!(self.peek(), Token::LBrace) && self.is_import() {
            return self.parse_import();
        }

        let (name, name_span) = self.expect_identifier()?;

        self.skip_newlines();

        match self.peek() {
            Token::Colon if self.colon_is_followed_by_identifier(BEHAVIOR_KEYWORD) => {
                self.parse_behavior_def(name, Vec::new(), public, name_span)
            }

            Token::Colon if self.is_struct_def() => {
                self.parse_struct_def_with_params(name, Vec::new(), public, name_span)
            }

            Token::Colon if self.is_enum_def() => {
                self.parse_enum_def_with_params(name, Vec::new(), public, name_span)
            }

            Token::Lt => {
                let type_params = self.parse_type_params()?;
                self.parse_generic_declaration(name, type_params, public, name_span)
            }

            Token::Dot => self.parse_dot_declaration(name, Vec::new(), public, name_span),

            Token::Assign => self.parse_function_def(name, Vec::new(), public, name_span),

            Token::ConstAssign => self.parse_top_level_var_decl(name, name_span, false, true),

            Token::DeclareAssign => self.parse_top_level_var_decl(name, name_span, true, false),

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

    /// `@export({ Name, other })` — the module's public surface. The brace group
    /// is a bare identifier list (comma- and/or newline-separated), not a struct
    /// literal.
    fn parse_export_manifest(&mut self) -> Result<Declaration, CompileError> {
        let start = self.peek_span();
        self.expect(&Token::AtExport)?;
        self.expect(&Token::LParen)?;
        self.skip_newlines();
        self.expect(&Token::LBrace)?;

        let mut names = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Token::RBrace) {
                break;
            }
            let (name, _) = self.expect_identifier()?;
            names.push(name);
            self.skip_newlines();
            if !self.consume_comma() {
                break;
            }
        }

        self.skip_newlines();
        self.expect(&Token::RBrace)?;
        self.expect(&Token::RParen)?;
        let span = start.merge(self.prev_span());
        Ok(Declaration::Export { names, span })
    }
}
