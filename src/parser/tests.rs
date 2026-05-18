use super::*;
use crate::lexer;

fn parse_str(src: &str) -> Result<Program, Vec<CompileError>> {
    let tokens = lexer::tokenize(src, 0).map_err(|e| vec![e])?;
    parse(tokens, 0)
}

fn parse_ok(src: &str) -> Program {
    parse_str(src).unwrap_or_else(|errs| {
        for e in &errs {
            eprintln!("{:?}", e);
        }
        panic!("parse failed with {} errors", errs.len());
    })
}

fn parse_err(src: &str) -> Vec<CompileError> {
    parse_str(src).expect_err("expected parse to fail")
}

mod behaviors;
mod declarations;
mod examples_types_errors;
mod expressions;
mod misc_syntax;
