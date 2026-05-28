use super::*;
use crate::error::REMOVED_INFIX_AS_CAST_MESSAGE;

pub(super) enum InfixParse {
    Parsed(Expression),
    Stop(Expression),
    Continue(Expression),
}

impl Parser {
    const REMOVED_AS_CAST_L_BP: u8 = 1;

    pub(super) fn parse_infix_or_range_expr(
        &mut self,
        lhs: Expression,
        min_bp: u8,
    ) -> Result<InfixParse, CompileError> {
        if matches!(self.peek(), Token::Identifier(name) if name == "as") {
            if Self::REMOVED_AS_CAST_L_BP < min_bp {
                return Ok(InfixParse::Stop(lhs));
            }
            self.advance();
            self.parse_type()?;
            return Err(CompileError::Syntax(
                REMOVED_INFIX_AS_CAST_MESSAGE.into(),
                Some(lhs.span().merge(self.prev_span())),
            ));
        }

        if let Some(operator) = infix_operator(self.peek()) {
            if operator.l_bp < min_bp {
                return Ok(InfixParse::Stop(lhs));
            }

            self.advance();
            let rhs = self.parse_expr_bp(operator.r_bp)?;
            let span = lhs.span().merge(rhs.span());
            return Ok(InfixParse::Parsed(Expression::BinaryOp {
                op: operator.op,
                left: Box::new(lhs),
                right: Box::new(rhs),
                span,
            }));
        }

        if !matches!(self.peek(), Token::DotDot | Token::DotDotEq) {
            return Ok(InfixParse::Continue(lhs));
        }

        let (l_bp, r_bp) = (3, 4);
        if l_bp < min_bp {
            return Ok(InfixParse::Stop(lhs));
        }
        self.advance();
        let rhs = self.parse_expr_bp(r_bp)?;
        let span = lhs.span().merge(rhs.span());
        Err(CompileError::Syntax(
            "range expressions are not implemented; range typing remains gated".into(),
            Some(span),
        ))
    }
}
