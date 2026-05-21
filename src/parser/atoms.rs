use super::*;
use crate::error::{REMOVED_AS_CAST_MESSAGE, REMOVED_RETURN_KEYWORD_MESSAGE};
use crate::parser::keywords::{ParserPrefixKeyword, ParserThisMethod};

mod forms;
mod module_roots;

impl Parser {
    /// Parse prefix / atom expressions.
    pub(super) fn parse_prefix(&mut self) -> Result<Expression, CompileError> {
        self.skip_newlines();
        match self.peek().clone() {
            // Unary operators
            Token::Minus => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: s,
                })
            }
            Token::Not => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: s,
                })
            }
            Token::Tilde => {
                let (_, span) = self.advance();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let s = span.merge(operand.span());
                Ok(Expression::UnaryOp {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                    span: s,
                })
            }

            // Literals
            Token::IntLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::IntLiteral(v) = tok {
                    Ok(Expression::IntLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }
            Token::FloatLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::FloatLiteral(v) = tok {
                    Ok(Expression::FloatLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }
            Token::StringLiteral(_) => {
                let (tok, span) = self.advance();
                if let Token::StringLiteral(v) = tok {
                    Ok(Expression::StringLiteral { value: v, span })
                } else {
                    unreachable!()
                }
            }

            // String interpolation
            Token::StringChunk(_) | Token::InterpolationStart => self.parse_string_interpolation(),

            // Identifier or parser-owned prefix keyword.
            Token::Identifier(ref name) => {
                if let Ok(keyword) = name.parse::<ParserPrefixKeyword>() {
                    let (_, span) = self.advance();
                    match keyword {
                        ParserPrefixKeyword::True => {
                            Ok(Expression::BoolLiteral { value: true, span })
                        }
                        ParserPrefixKeyword::False => {
                            Ok(Expression::BoolLiteral { value: false, span })
                        }
                        ParserPrefixKeyword::Return => Err(CompileError::Syntax(
                            REMOVED_RETURN_KEYWORD_MESSAGE.into(),
                            Some(span),
                        )),
                        ParserPrefixKeyword::As => Err(CompileError::Syntax(
                            REMOVED_AS_CAST_MESSAGE.into(),
                            Some(span),
                        )),
                        ParserPrefixKeyword::Break => Ok(Expression::Break { span }),
                        ParserPrefixKeyword::Continue => Ok(Expression::Continue { span }),
                        ParserPrefixKeyword::Loop => self.parse_loop(span),
                        ParserPrefixKeyword::Cast => self.parse_cast(span),
                    }
                } else {
                    let name = name.clone();
                    let (_, span) = self.advance();
                    Ok(Expression::Identifier { name, span })
                }
            }

            // @this.defer(expr) or other @ tokens
            Token::AtThis => {
                let (_, span) = self.advance();
                // @this.defer(expr)
                if matches!(self.peek(), Token::Dot) {
                    self.advance(); // consume .
                    let (method, _) = self.expect_identifier()?;
                    if let Ok(method) = method.parse::<ParserThisMethod>() {
                        match method {
                            ParserThisMethod::Defer => {
                                self.expect(&Token::LParen)?;
                                let expr = self.parse_expression()?;
                                let end = self.expect(&Token::RParen)?;
                                return Ok(Expression::Defer {
                                    expr: Box::new(expr),
                                    span: span.merge(end),
                                });
                            }
                        }
                    }
                    // Other @this.method calls
                    return Err(CompileError::Syntax(
                        format!("unknown @this method: {}", method),
                        Some(span),
                    ));
                }
                Ok(Expression::Identifier {
                    name: "@this".to_string(),
                    span,
                })
            }

            Token::Dot => self.parse_shorthand_enum_variant_expr(),

            // Module-qualified: @builtin.func(args), @std.mod.func(args)
            Token::AtBuiltin => self.parse_builtin_module_call_expr(),
            Token::AtStd => self.parse_std_module_root_expr(),

            // Parenthesized expression or closure
            Token::LParen => {
                // Try to detect if this is a closure: `(params) ret? { body }`
                // vs a grouped expression: `(expr)`
                if self.is_closure() {
                    self.parse_closure()
                } else {
                    self.advance(); // consume (
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen)?;
                    Ok(expr)
                }
            }

            // Block expression
            Token::LBrace => self.parse_block_expression(),

            // Array literal
            Token::LBracket => {
                let (_, span) = self.advance();
                let mut elements = Vec::new();
                loop {
                    self.skip_newlines();
                    if matches!(self.peek(), Token::RBracket) {
                        break;
                    }
                    elements.push(self.parse_expression()?);
                    self.skip_newlines();
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                let end = self.expect(&Token::RBracket)?;
                Ok(Expression::ArrayLiteral {
                    elements,
                    span: span.merge(end),
                })
            }

            _ => {
                let (tok, span) = self.advance();
                Err(CompileError::Syntax(
                    format!("unexpected token {:?} in expression", tok),
                    Some(span),
                ))
            }
        }
    }
}
