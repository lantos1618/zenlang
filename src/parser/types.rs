use super::*;
use crate::ast::{BuiltinGenericTypeName, BuiltinTypeName};

impl Parser {
    // ── Types ─────────────────────────────────────────────────

    pub(super) fn parse_type(&mut self) -> Result<AstType, CompileError> {
        self.skip_newlines();
        let (tok, span) = self.advance();
        match tok {
            Token::Identifier(name) => self.resolve_type_name(&name),
            Token::LParen => {
                // Function type: `(i32, i32) i32`
                let mut params = Vec::new();
                loop {
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RParen) {
                        break;
                    }
                    params.push(self.parse_type()?);
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(&Token::RParen)?;
                let ret = Box::new(self.parse_type()?);
                Ok(AstType::Function { params, ret })
            }
            Token::LBracket => {
                // Array type: `[i32]` or `[i32; 10]`
                let elem = Box::new(self.parse_type()?);
                self.skip_newlines();
                let size = if matches!(self.peek(), Token::Semicolon) {
                    self.advance();
                    let (stok, sspan) = self.advance();
                    match stok {
                        Token::IntLiteral(n) => Some(n as usize),
                        _ => {
                            return Err(CompileError::Syntax(
                                "expected array size".into(),
                                Some(sspan),
                            ));
                        }
                    }
                } else {
                    None
                };
                self.expect(&Token::RBracket)?;
                Ok(AstType::Array { elem, size })
            }
            _ => Err(CompileError::Syntax(
                format!("expected type, found {:?}", tok),
                Some(span),
            )),
        }
    }

    fn resolve_type_name(&mut self, name: &str) -> Result<AstType, CompileError> {
        if let Ok(builtin) = name.parse::<BuiltinTypeName>() {
            return Ok(builtin.ast_type());
        }

        if matches!(self.peek(), Token::Lt) {
            let type_args = self.parse_type_arg_list()?;
            return Ok(Self::resolve_generic_type_name(name, type_args));
        }

        Ok(AstType::Named(name.to_string()))
    }

    fn resolve_generic_type_name(base: &str, type_args: Vec<AstType>) -> AstType {
        if let Ok(builtin) = base.parse::<BuiltinGenericTypeName>() {
            return builtin
                .ast_type(type_args)
                .unwrap_or_else(|type_args| AstType::Generic {
                    name: base.to_string(),
                    type_args,
                });
        }

        AstType::Generic {
            name: base.to_string(),
            type_args,
        }
    }
}
