//! Program-level parsing - exports, imports, and declaration detection
//! Extracted from statements.rs to reduce file size

use super::core::Parser;
use crate::ast::{Declaration, Statement};
use crate::error::{CompileError, Result};
use crate::lexer::Token;
impl<'a> Parser<'a> {
    /// Parse an @export declaration
    pub fn parse_export(&mut self) -> Result<Declaration> {
        self.next_token();

        // Check for @export * (export all public symbols)
        if self.current_token == Token::Operator("*".to_string()) {
            self.next_token();
            return Ok(Declaration::Export {
                symbols: vec!["*".to_string()], // Special marker for "export all"
            });
        }

        if self.current_token != Token::Symbol('{') {
            return Err(CompileError::SyntaxError(
                "Expected '{' or '*' after @export".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token(); // consume '{'

        let exported_symbols = self.parse_identifier_list('}', "@export list")?;

        Ok(Declaration::Export {
            symbols: exported_symbols,
        })
    }

    /// Parse a destructuring import: { name, name } = @std
    pub fn parse_destructuring_import_declaration(&mut self) -> Result<Vec<Declaration>> {
        self.next_token(); // consume '{'
        let imported_names = self.parse_identifier_list('}', "destructuring import")?;

        // Expect '=' operator
        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' after destructuring pattern".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();

        // Expect @std or @std.module reference
        if self.current_token == Token::AtStd {
            let mut module_path = "@std".to_string();
            self.next_token();

            // Handle @std.module syntax
            while self.current_token == Token::Symbol('.') {
                self.next_token();
                if let Token::Identifier(member) = &self.current_token {
                    module_path.push('.');
                    module_path.push_str(member);
                    self.next_token();
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected identifier after '.'".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
            }

            // Create imports from the specified module
            // For destructuring imports like { Range } = @std.core.iterator,
            // we load the entire module, not module.name
            let mut declarations = vec![];
            for name in imported_names {
                let actual_module_path = if module_path == "@std" {
                    // Convention: { name } = @std loads @std.{name}
                    // Module system handles actual file resolution
                    format!("@std.{}", name)
                } else {
                    // Import from specific module like @std.core.iterator
                    // Load the whole module, not module.name
                    module_path.clone()
                };
                declarations.push(Declaration::ModuleImport {
                    alias: name.clone(),
                    module_path: actual_module_path,
                    span: Some(self.current_span.clone()),
                });
            }
            Ok(declarations)
        } else if let Token::Identifier(module) = &self.current_token {
            // Handle both @std.module and package-name.module patterns (e.g., std.io, http.server)
            let mut module_path = module.clone();
            self.next_token();

            // Handle dotted paths
            while self.current_token == Token::Symbol('.') {
                self.next_token();
                if let Token::Identifier(member) = &self.current_token {
                    module_path.push('.');
                    module_path.push_str(member);
                    self.next_token();
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected identifier after '.'".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
            }

            let mut declarations = vec![];
            for name in imported_names {
                let actual_module_path = if !module_path.contains('.') {
                    // Single identifier like `std`: expand each name to `std.name`
                    // This matches the @std expansion behavior above
                    format!("{}.{}", module_path, name)
                } else {
                    // Dotted path like `std.math.math`: import from specific module
                    module_path.clone()
                };
                declarations.push(Declaration::ModuleImport {
                    alias: name.clone(),
                    module_path: actual_module_path,
                    span: Some(self.current_span.clone()),
                });
            }
            Ok(declarations)
        } else {
            Err(CompileError::SyntaxError(
                "Expected module reference after '=' in destructuring import".to_string(),
                Some(self.current_span.clone()),
            ))
        }
    }

    /// Check if the current position represents a module import after :=
    /// Returns true if this is @std, @std.xxx, or build.import pattern
    pub fn is_module_import_after_colon_assign(&mut self) -> bool {
        if self.current_token == Token::AtStd {
            return true;
        }

        if let Token::Identifier(id) = &self.current_token {
            if id.starts_with("@std") {
                return true;
            }
            if id == "build" {
                let saved_state = self.lexer.save_state();
                let saved_current = self.current_token.clone();
                let saved_peek = self.peek_token.clone();

                self.next_token();
                let is_import = self.current_token == Token::Symbol('.') && {
                    self.next_token();
                    matches!(&self.current_token, Token::Identifier(name) if name == "import")
                };

                self.lexer.restore_state(saved_state);
                self.current_token = saved_current;
                self.peek_token = saved_peek;

                return is_import;
            }
        }
        false
    }

    /// Parse a module import after := has been consumed
    /// Handles both @std.module and build.import("module") patterns
    pub fn parse_module_import_after_colon_assign(&mut self, alias: String) -> Result<Declaration> {
        if let Token::Identifier(id) = &self.current_token {
            if id == "build" {
                self.next_token();
                if self.current_token != Token::Symbol('.') {
                    return Err(CompileError::SyntaxError(
                        "Expected '.' after 'build'".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();

                if !matches!(&self.current_token, Token::Identifier(name) if name == "import") {
                    return Err(CompileError::SyntaxError(
                        "Expected 'import' after 'build.'".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();

                if self.current_token != Token::Symbol('(') {
                    return Err(CompileError::SyntaxError(
                        "Expected '(' after 'build.import'".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();

                let module_name = if let Token::StringLiteral(name) = &self.current_token {
                    name.clone()
                } else {
                    return Err(CompileError::SyntaxError(
                        "Expected string literal for module name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                };
                self.next_token();

                if self.current_token != Token::Symbol(')') {
                    return Err(CompileError::SyntaxError(
                        "Expected ')' after module name".to_string(),
                        Some(self.current_span.clone()),
                    ));
                }
                self.next_token();

                Ok(Declaration::ModuleImport {
                    alias,
                    module_path: format!("std.{}", module_name),
                    span: Some(self.current_span.clone()),
                })
            } else {
                let module_path = id.clone();
                self.next_token();

                let full_path = if self.current_token == Token::Symbol('.') {
                    let mut path = module_path;
                    while self.current_token == Token::Symbol('.') {
                        self.next_token();
                        if let Token::Identifier(member) = &self.current_token {
                            path.push('.');
                            path.push_str(member);
                            self.next_token();
                        } else {
                            break;
                        }
                    }
                    path
                } else {
                    module_path
                };

                Ok(Declaration::ModuleImport {
                    alias,
                    module_path: full_path,
                    span: Some(self.current_span.clone()),
                })
            }
        } else {
            Err(CompileError::SyntaxError(
                "Expected module path after :=".to_string(),
                Some(self.current_span.clone()),
            ))
        }
    }

    /// Parse a constant declaration from a statement
    pub fn parse_constant_from_statement(&mut self) -> Result<Declaration> {
        let stmt = self.parse_statement()?;
        if let Statement::VariableDeclaration {
            name,
            type_,
            initializer,
            ..
        } = stmt
        {
            if let Some(init) = initializer {
                Ok(Declaration::Constant {
                    name,
                    type_,
                    value: init,
                    span: Some(self.current_span.clone()),
                })
            } else {
                Err(CompileError::SyntaxError(
                    "Constant declaration requires an initializer".to_string(),
                    Some(self.current_span.clone()),
                ))
            }
        } else {
            Err(CompileError::SyntaxError(
                "Expected variable declaration".to_string(),
                Some(self.current_span.clone()),
            ))
        }
    }

    /// Parse a top-level mutable variable declaration: name :: Type = value
    pub fn parse_top_level_mutable_var(&mut self, name: String) -> Result<Declaration> {
        self.next_token();
        let type_ = self.parse_type()?;

        if self.current_token != Token::Operator("=".to_string()) {
            return Err(CompileError::SyntaxError(
                "Expected '=' after type in mutable variable declaration".to_string(),
                Some(self.current_span.clone()),
            ));
        }
        self.next_token();

        let value = self.parse_expression()?;

        if self.current_token == Token::Symbol(';') {
            self.next_token();
        }

        Ok(Declaration::Constant {
            name,
            value,
            type_: Some(type_),
            span: Some(self.current_span.clone()),
        })
    }
}
