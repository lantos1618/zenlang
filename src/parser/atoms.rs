use super::*;
use crate::error::{REMOVED_AS_CAST_MESSAGE, REMOVED_RETURN_KEYWORD_MESSAGE};
use crate::parser::keywords::THIS_DEFER_METHOD;

mod forms;
mod module_roots;

impl Parser {
    pub(super) fn parse_prefix(&mut self) -> Result<Expression, CompileError> {
        self.skip_newlines();
        match self.peek().clone() {
            Token::Minus => self.parse_unary_prefix(UnaryOp::Neg),
            Token::Not => self.parse_unary_prefix(UnaryOp::Not),
            Token::Tilde => self.parse_unary_prefix(UnaryOp::BitNot),

            Token::IntLiteral(value) => {
                let (_, span) = self.advance();
                Ok(Expression::IntLiteral { value, span })
            }
            Token::FloatLiteral(value) => {
                let (_, span) = self.advance();
                Ok(Expression::FloatLiteral { value, span })
            }
            Token::StringLiteral(value) => {
                let (_, span) = self.advance();
                Ok(Expression::StringLiteral { value, span })
            }

            Token::StringChunk(_) | Token::InterpolationStart => self.parse_string_interpolation(),

            Token::Identifier(ref name) => {
                let name = name.clone();
                let (_, span) = self.advance();
                match name.as_str() {
                    "true" => Ok(Expression::BoolLiteral { value: true, span }),
                    "false" => Ok(Expression::BoolLiteral { value: false, span }),
                    "return" => Err(CompileError::Syntax(
                        REMOVED_RETURN_KEYWORD_MESSAGE.into(),
                        Some(span),
                    )),
                    "as" => Err(CompileError::Syntax(
                        REMOVED_AS_CAST_MESSAGE.into(),
                        Some(span),
                    )),
                    "loop" => self.parse_loop(span),
                    "cast" => self.parse_cast(span),
                    _ => Ok(Expression::Identifier { name, span }),
                }
            }

            Token::AtThis => {
                let (_, span) = self.advance();
                if matches!(self.peek(), Token::Dot) {
                    self.advance();
                    let (method, _) = self.expect_identifier()?;
                    if method == THIS_DEFER_METHOD {
                        self.expect(&Token::LParen)?;
                        let expr = self.parse_expression()?;
                        let end = self.expect(&Token::RParen)?;
                        return Ok(Expression::Defer {
                            expr: Box::new(expr),
                            span: span.merge(end),
                        });
                    }
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

            Token::AtBuiltin => self.parse_builtin_module_call_expr(),
            Token::AtStd => self.parse_std_module_root_expr(),

            Token::LParen => {
                if self.is_closure() {
                    self.parse_closure()
                } else {
                    self.advance();
                    let expr = self.parse_expression()?;
                    self.expect(&Token::RParen)?;
                    Ok(expr)
                }
            }

            Token::LBrace => self.parse_block_expression(),

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
                    self.consume_comma();
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

    fn parse_unary_prefix(&mut self, op: UnaryOp) -> Result<Expression, CompileError> {
        let (_, op_span) = self.advance();
        let operand = self.parse_expr_bp(PREFIX_BP)?;
        let span = op_span.merge(operand.span());
        Ok(Expression::UnaryOp {
            op,
            operand: Box::new(operand),
            span,
        })
    }
}
