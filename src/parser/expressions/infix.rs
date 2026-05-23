use super::*;
use crate::error::REMOVED_INFIX_AS_CAST_MESSAGE;
use crate::parser::keywords::ParserPrefixKeyword;

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
        if matches!(
            self.peek(),
            Token::Identifier(name)
                if matches!(name.parse::<ParserPrefixKeyword>(), Ok(ParserPrefixKeyword::As))
        ) {
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

        let range = match self.peek() {
            Token::DotDot => Some(false),
            Token::DotDotEq => Some(true),
            _ => None,
        };
        let Some(inclusive) = range else {
            return Ok(InfixParse::Continue(lhs));
        };

        let (l_bp, r_bp) = (3, 4);
        if l_bp < min_bp {
            return Ok(InfixParse::Stop(lhs));
        }
        self.advance();
        let rhs = self.parse_expr_bp(r_bp)?;
        let span = lhs.span().merge(rhs.span());
        Ok(InfixParse::Parsed(Expression::Range {
            start: Box::new(lhs),
            end: Box::new(rhs),
            inclusive,
            span,
        }))
    }
}
