use std::collections::HashSet;

use super::*;
use crate::parser::keywords::{BEHAVIOR_KEYWORD, MUT_KEYWORD};

mod function_forms;
mod generic;

/// Desugar `@export({ ... })` manifests: set the `public` flag on the named
/// declarations (exporting a type also exports its methods and impl methods),
/// then drop the `Export` nodes. After this the AST is identical to one written
/// with per-declaration `pub`, so the whole downstream pipeline (resolver,
/// import-seeding, codegen, goldens) is unchanged. An exported name that matches
/// no declaration is an error.
pub(super) fn apply_export_manifests(
    decls: &mut Vec<Declaration>,
    errors: &mut Vec<CompileError>,
) {
    let mut exported: HashSet<String> = HashSet::new();
    let mut export_spans: Vec<(String, Span)> = Vec::new();
    for decl in decls.iter() {
        if let Declaration::Export { names, span } = decl {
            for name in names {
                exported.insert(name.clone());
                export_spans.push((name.clone(), *span));
            }
        }
    }

    if exported.is_empty() {
        return;
    }

    let mut matched: HashSet<String> = HashSet::new();
    for decl in decls.iter_mut() {
        match decl {
            Declaration::Function { name, public, .. }
            | Declaration::Struct { name, public, .. }
            | Declaration::Enum { name, public, .. }
            | Declaration::Behavior { name, public, .. } => {
                if exported.contains(name) {
                    *public = true;
                    matched.insert(name.clone());
                }
            }
            // A method is exported by its dotted name: `@export({ Box.get })`.
            Declaration::Method {
                type_name,
                method_name,
                public,
                ..
            } => {
                let key = format!("{type_name}.{method_name}");
                if exported.contains(&key) {
                    *public = true;
                    matched.insert(key);
                }
            }
            // Methods inside an impl block are exported the same way, by
            // `Type.method`. Privacy is per-method: an unlisted method stays
            // private even when its type is exported.
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                for method in methods.iter_mut() {
                    let Some(mname) = method.name().map(str::to_string) else {
                        continue;
                    };
                    let key = format!("{type_name}.{mname}");
                    if exported.contains(&key) {
                        matched.insert(key);
                        match method {
                            Declaration::Function { public, .. }
                            | Declaration::Method { public, .. } => *public = true,
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for (name, span) in export_spans {
        if !matched.contains(&name) {
            errors.push(CompileError::Resolution(
                format!("exported name `{name}` is not defined in this module"),
                Some(span),
            ));
        }
    }

    decls.retain(|decl| !matches!(decl, Declaration::Export { .. }));
}

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

        // Everything is private by default; visibility is granted only by an
        // `@export({ ... })` manifest (desugared in apply_export_manifests).
        let public = false;

        if matches!(self.peek(), Token::AtExtern) {
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
            // An export entry is a bare name (`Point`) or a dotted method ref
            // (`Box.get`) — methods are exported individually.
            let (mut name, _) = self.expect_identifier()?;
            if matches!(self.peek(), Token::Dot) {
                self.advance();
                let (method, _) = self.expect_identifier()?;
                name = format!("{name}.{method}");
            }
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
