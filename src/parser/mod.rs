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
mod impl_blocks;
mod import_declarations;
mod keywords;
mod lookahead;
mod navigation;
mod patterns;
mod precedence;
mod statements;
mod types;

use core::{Parser, StmtOrExpr};
use precedence::*;

pub fn parse(tokens: Vec<(Token, Span)>, file_id: FileId) -> Result<Program, Vec<CompileError>> {
    let mut parser = Parser::new(tokens);
    let decls = parser.parse_program();
    parser
        .errors
        .is_empty()
        .then_some(Program {
            declarations: decls,
            file_id,
        })
        .ok_or(parser.errors)
}
