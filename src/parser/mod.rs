use crate::ast::declarations::{
    Declaration, EnumVariant, StructField, TypeDeclarationKeyword, TypeParam,
};
use crate::ast::expressions::{
    BinaryOp, Expression, LoopControlAction, MatchArm, StringPart, UnaryOp,
};
use crate::ast::patterns::Pattern;
use crate::ast::statements::Statement;
use crate::ast::types::{AstType, Param};
use crate::ast::Program;
use crate::error::{CompileError, FileId, Span};
use crate::lexer::Token;

mod atoms;
mod behavior_declarations;
mod block_helpers;
mod core;
mod declaration_types;
mod declarations;
mod expressions;
mod import_declarations;
mod patterns;
mod precedence;
mod statements;
mod types;

#[cfg(test)]
mod tests;

use core::{Parser, StmtOrExpr};
use precedence::*;

// ── Public API ────────────────────────────────────────────────────

/// Parse a token stream into a Program (list of declarations).
pub fn parse(tokens: Vec<(Token, Span)>, file_id: FileId) -> Result<Program, Vec<CompileError>> {
    let mut parser = Parser::new(tokens, file_id);
    let decls = parser.parse_program();
    if parser.errors.is_empty() {
        Ok(Program {
            declarations: decls,
            file_id,
        })
    } else {
        Err(parser.errors)
    }
}
