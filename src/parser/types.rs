use super::*;

impl Parser {
    // ── Types ─────────────────────────────────────────────────

    pub(super) fn parse_type(&mut self) -> Result<AstType, CompileError> {
        self.skip_newlines();
        let (tok, span) = self.advance();
        match tok {
            Token::Identifier(name) => self.resolve_type_name(&name, span),
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

    fn resolve_type_name(&mut self, name: &str, _span: Span) -> Result<AstType, CompileError> {
        let base = match name {
            "i8" => return Ok(AstType::I8),
            "i16" => return Ok(AstType::I16),
            "i32" => return Ok(AstType::I32),
            "i64" => return Ok(AstType::I64),
            "u8" => return Ok(AstType::U8),
            "u16" => return Ok(AstType::U16),
            "u32" => return Ok(AstType::U32),
            "u64" => return Ok(AstType::U64),
            "usize" => return Ok(AstType::Usize),
            "f32" => return Ok(AstType::F32),
            "f64" => return Ok(AstType::F64),
            "bool" => return Ok(AstType::Bool),
            "void" => return Ok(AstType::Void),
            "str" => return Ok(AstType::Str),
            "String" | "StaticString" => return Ok(AstType::Str),
            "Self" => return Ok(AstType::SelfType),
            _ => name.to_string(),
        };

        // Check for generic args: Name<T, U>
        if matches!(self.peek(), Token::Lt) {
            self.advance(); // consume <
            let mut type_args = Vec::new();
            loop {
                self.skip_newlines();
                if matches!(self.peek(), Token::Gt) {
                    break;
                }
                type_args.push(self.parse_type()?);
                self.skip_newlines();
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                }
            }
            self.expect(&Token::Gt)?;

            // Handle well-known generic types
            match base.as_str() {
                "Ptr" if type_args.len() == 1 => Ok(AstType::Ptr(Box::new(type_args.remove(0)))),
                "MutPtr" if type_args.len() == 1 => {
                    Ok(AstType::MutPtr(Box::new(type_args.remove(0))))
                }
                "RawPtr" if type_args.len() == 1 => {
                    Ok(AstType::RawPtr(Box::new(type_args.remove(0))))
                }
                _ => Ok(AstType::Generic {
                    name: base,
                    type_args,
                }),
            }
        } else {
            Ok(AstType::Named(base))
        }
    }
}
