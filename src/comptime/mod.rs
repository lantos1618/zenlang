// Compile-time execution framework for Zen
// This module provides an interpreter that executes Zen code during compilation
//
// Split into components:
//   values.rs      - ComptimeValue, ASTNodeValue, control flow types
//   environment.rs - Variable scoping
//   expressions.rs - Expression evaluation, binary ops, pattern matching
//   statements.rs  - Statement execution, loops, function calls
//   methods.rs     - Method dispatch (AST nodes, arrays, strings, meta)
//   meta.rs        - AST introspection primitives

use crate::ast::{Declaration, Expression};
use crate::error::{CompileError, Result};
use std::collections::HashMap;
use std::rc::Rc;

pub mod meta;

mod environment;
mod expressions;
mod methods;
mod statements;
pub mod values;

// Re-export public types
pub use environment::Environment;
pub use values::{ASTNodeValue, ComptimeControlFlow, ComptimeSignal, ComptimeValue, StmtResult};

/// The compile-time interpreter.
pub struct ComptimeInterpreter {
    pub(crate) env: Environment,
    generated_declarations: Vec<Declaration>,
    pub(crate) modules: HashMap<String, ComptimeValue>,
}

impl Default for ComptimeInterpreter {
    fn default() -> Self {
        let mut interpreter = ComptimeInterpreter {
            env: Environment::new(),
            generated_declarations: Vec::new(),
            modules: HashMap::new(),
        };
        interpreter.init_builtins();
        interpreter
    }
}

impl ComptimeInterpreter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_variable(&mut self, name: String, value: ComptimeValue) {
        self.env.variables.borrow_mut().insert(name, (value, true));
    }

    pub fn get_variable(&self, name: &str) -> Option<ComptimeValue> {
        self.env
            .variables
            .borrow()
            .get(name)
            .map(|(v, _)| v.clone())
    }

    /// Execute a closure in a child scope. The environment is always restored,
    /// even if the closure returns an error.
    pub(crate) fn with_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let child_env = Environment::with_parent(self.env.clone());
        let saved = std::mem::replace(&mut self.env, child_env);
        let result = f(self);
        self.env = saved;
        result
    }

    /// Filter program declarations by a predicate.
    pub(crate) fn filter_program_declarations(
        node: &ASTNodeValue,
        pred: fn(&Declaration) -> bool,
        method_name: &str,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        if let ASTNodeValue::Program(prog) = node {
            Ok(ComptimeValue::Array(
                prog.declarations
                    .iter()
                    .filter(|d| pred(d))
                    .map(|d| ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(d.clone()))))
                    .collect(),
            ))
        } else {
            Err(CompileError::ComptimeError(
                format!("{}() only works on Program nodes", method_name),
                span,
            ))
        }
    }

    /// Find a single named declaration in a Program node.
    pub(crate) fn find_program_declaration(
        node: &ASTNodeValue,
        target_name: &str,
        extract_name: fn(&Declaration) -> Option<&str>,
        method_name: &str,
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        if let ASTNodeValue::Program(prog) = node {
            for d in &prog.declarations {
                if extract_name(d) == Some(target_name) {
                    return Ok(ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
                        d.clone(),
                    ))));
                }
            }
            Ok(ComptimeValue::Null)
        } else {
            Err(CompileError::ComptimeError(
                format!("{}() only works on Program nodes", method_name),
                span,
            ))
        }
    }

    /// Helper: evaluate an argument as a string, or return an error.
    pub(crate) fn eval_string_arg(
        &mut self,
        args: &[Expression],
        method_name: &str,
        span: Option<crate::error::Span>,
    ) -> Result<String> {
        if args.len() != 1 {
            return Err(CompileError::ComptimeError(
                format!("{}() expects 1 argument (name)", method_name),
                span,
            ));
        }
        let val = self.evaluate_expression(&args[0], span.clone())?;
        match val {
            ComptimeValue::String(s) => Ok(s),
            _ => Err(CompileError::ComptimeError(
                format!("{}() expects a string argument", method_name),
                span,
            )),
        }
    }

    /// Helper: extract an integer index from a ComptimeValue.
    pub(crate) fn value_to_index(val: &ComptimeValue) -> Option<usize> {
        match val {
            ComptimeValue::I32(i) => Some(*i as usize),
            ComptimeValue::I64(i) => Some(*i as usize),
            ComptimeValue::U32(i) => Some(*i as usize),
            ComptimeValue::U64(i) => Some(*i as usize),
            _ => None,
        }
    }

    pub fn get_generated_declarations(&mut self) -> Vec<Declaration> {
        std::mem::take(&mut self.generated_declarations)
    }

    /// Push a declaration to be injected into the program after comptime execution.
    /// This is the bridge between comptime code generation and the compiler pipeline.
    pub fn push_declaration(&mut self, decl: Declaration) {
        self.generated_declarations.push(decl);
    }

    fn init_builtins(&mut self) {
        self.modules.insert(
            "@std".to_string(),
            ComptimeValue::Struct {
                name: "@std".to_string(),
                fields: {
                    let mut fields = HashMap::new();

                    // Stdlib module stubs — these match real .zen files in stdlib/.
                    // They provide empty namespace objects so comptime code can reference
                    // them without crashing, even though comptime can't call runtime functions.
                    for ns in &[
                        "core",
                        "io",
                        "collections",
                        "memory",
                        "math",
                        "sys",
                        "concurrency",
                    ] {
                        fields.insert(
                            ns.to_string(),
                            ComptimeValue::Struct {
                                name: ns.to_string(),
                                fields: HashMap::new(),
                            },
                        );
                    }

                    // @std.meta — the real comptime module (AST introspection)
                    fields.insert(
                        "meta".to_string(),
                        ComptimeValue::Struct {
                            name: "meta".to_string(),
                            fields: {
                                let mut meta_fields = HashMap::new();
                                meta_fields
                                    .insert("Expression".to_string(), meta::expression_variants());
                                meta_fields
                                    .insert("Statement".to_string(), meta::statement_variants());
                                meta_fields.insert(
                                    "Declaration".to_string(),
                                    meta::declaration_variants(),
                                );
                                meta_fields.insert("AstType".to_string(), meta::type_variants());
                                meta_fields.insert("Pattern".to_string(), meta::pattern_variants());
                                meta_fields
                            },
                        },
                    );

                    // @std.build — build system integration (matches stdlib/build.zen)
                    fields.insert(
                        "build".to_string(),
                        ComptimeValue::Struct {
                            name: "build".to_string(),
                            fields: {
                                let mut build_fields = HashMap::new();
                                build_fields.insert(
                                    "import".to_string(),
                                    ComptimeValue::Function {
                                        name: "import".to_string(),
                                        params: vec!["module_name".to_string()],
                                        body: vec![],
                                        closure: Environment::new(),
                                    },
                                );
                                build_fields
                            },
                        },
                    );

                    fields
                },
            },
        );

        // Register "std" as an alias for "@std" so that both
        // `{ meta } = @std` and `{ meta } = std` work in comptime code
        if let Some(std_val) = self.modules.get("@std").cloned() {
            self.modules.insert("std".to_string(), std_val.clone());
            // Also make "std" available as a variable for expression evaluation
            self.env.define("std".to_string(), std_val, false);
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::ast::{self, Expression, Pattern, Statement};

    fn parse_and_get_program(source: &str) -> ast::Program {
        let lexer = crate::lexer::Lexer::new(source);
        let mut parser = crate::parser::Parser::new(lexer);
        parser.parse_program().unwrap()
    }

    #[test]
    fn test_meta_parse_and_variant_name() {
        let mut interp = ComptimeInterpreter::new();
        let source_expr = Expression::String("x = 42".to_string());
        let meta_val = interp
            .evaluate_member_access(interp.modules.get("@std").unwrap().clone(), "meta", None)
            .unwrap();
        let result = interp
            .evaluate_method_call(meta_val.clone(), "parse", &[source_expr], None)
            .unwrap();
        assert!(matches!(result, ComptimeValue::ASTNode(_)));
        if let ComptimeValue::ASTNode(ref node) = result {
            assert_eq!(meta::variant_name(node), "Program");
        }
    }

    #[test]
    fn test_meta_type_info_on_parsed_code() {
        let program = parse_and_get_program("add = (a: i32, b: i32) i32 { return a + b }");
        let node = ASTNodeValue::Program(program);
        let info = meta::type_info(&node).unwrap();
        if let ComptimeValue::Struct { fields, .. } = &info {
            assert_eq!(
                fields.get("kind").unwrap().clone(),
                ComptimeValue::String("Program".to_string())
            );
        } else {
            panic!("Expected TypeInfo struct");
        }
    }

    #[test]
    fn test_meta_walk_function_declaration() {
        let program = parse_and_get_program("add = (a: i32, b: i32) i32 { return a + b }");
        assert!(!program.declarations.is_empty());
        let func_node = ASTNodeValue::Declaration(program.declarations[0].clone());
        assert_eq!(meta::variant_name(&func_node), "Function");
        let flds = meta::fields(&func_node).unwrap();
        let field_names: Vec<String> = flds
            .iter()
            .filter_map(|f| {
                if let ComptimeValue::Struct { fields, .. } = f {
                    if let Some(ComptimeValue::String(n)) = fields.get("name") {
                        return Some(n.clone());
                    }
                }
                None
            })
            .collect();
        assert!(field_names.contains(&"name".to_string()));
        assert!(field_names.contains(&"args".to_string()));
    }

    #[test]
    fn test_meta_walk_binary_expression() {
        let program = parse_and_get_program("main = () i32 { return 2 + 3 }");
        let func_decl = &program.declarations[0];
        if let Declaration::Function(f) = func_decl {
            let return_stmt = &f.body[0];
            if let Statement::Return { expr, .. } = return_stmt {
                let expr_node = ASTNodeValue::Expression(expr.clone());
                assert_eq!(meta::variant_name(&expr_node), "BinaryOp");
            }
        }
    }

    #[test]
    fn test_meta_parse_intrinsic_via_interpreter() {
        let mut interp = ComptimeInterpreter::new();
        let std_val = interp.modules.get("@std").unwrap().clone();
        if let ComptimeValue::Struct { fields, .. } = std_val {
            if let Some(meta_val) = fields.get("meta") {
                interp
                    .env
                    .define("meta".to_string(), meta_val.clone(), false);
            }
        }
        let meta_obj = interp.env.get("meta").unwrap();
        let result = interp
            .evaluate_method_call(
                meta_obj,
                "parse",
                &[Expression::String("x = 42".to_string())],
                None,
            )
            .unwrap();
        assert!(matches!(result, ComptimeValue::ASTNode(_)));
    }

    #[test]
    fn test_meta_field_access_on_ast_node() {
        let mut interp = ComptimeInterpreter::new();
        let program = parse_and_get_program("greet = (name: StringLiteral) void {}");
        let func_node = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Declaration(
            program.declarations[0].clone(),
        )));
        interp.env.define("func".to_string(), func_node, false);
        let func_val = interp.env.get("func").unwrap();
        let name = interp
            .evaluate_member_access(func_val, "name", None)
            .unwrap();
        assert_eq!(name, ComptimeValue::String("greet".to_string()));
    }

    #[test]
    fn test_meta_field_access_on_binary_op() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Integer32(10)),
            op: ast::BinaryOperator::Multiply,
            right: Box::new(Expression::Integer32(5)),
        };
        let node = ComptimeValue::ASTNode(Rc::new(ASTNodeValue::Expression(expr)));
        let mut interp = ComptimeInterpreter::new();
        interp.env.define("expr".to_string(), node, false);
        let expr_val = interp.env.get("expr").unwrap();
        let op = interp
            .evaluate_member_access(expr_val.clone(), "op", None)
            .unwrap();
        assert_eq!(op, ComptimeValue::String("*".to_string()));
    }

    #[test]
    fn test_destructuring_import_std_meta() {
        let mut interp = ComptimeInterpreter::new();
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();
        let meta_val = interp.env.get("meta");
        assert!(meta_val.is_some());
    }

    #[test]
    fn test_array_len_method() {
        let mut interp = ComptimeInterpreter::new();
        let arr = ComptimeValue::Array(vec![
            ComptimeValue::I32(1),
            ComptimeValue::I32(2),
            ComptimeValue::I32(3),
        ]);
        interp.env.define("arr".to_string(), arr, false);
        let arr_val = interp.env.get("arr").unwrap();
        let len = interp
            .evaluate_method_call(arr_val, "len", &[], None)
            .unwrap();
        assert_eq!(len, ComptimeValue::I64(3));
    }

    #[test]
    fn test_string_append_method() {
        let mut interp = ComptimeInterpreter::new();
        let s = ComptimeValue::String("hello ".to_string());
        let result = interp
            .evaluate_method_call(
                s,
                "append",
                &[Expression::String("world".to_string())],
                None,
            )
            .unwrap();
        assert_eq!(result, ComptimeValue::String("hello world".to_string()));
    }

    #[test]
    fn test_full_meta_pipeline_parse_walk_introspect() {
        let mut interp = ComptimeInterpreter::new();
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();
        let meta_val = interp.env.get("meta").unwrap();
        let ast_node = interp
            .evaluate_method_call(
                meta_val.clone(),
                "parse",
                &[Expression::String(
                    "add = (a: i32, b: i32) i32 { return a + b }".to_string(),
                )],
                None,
            )
            .unwrap();
        interp
            .env
            .define("program".to_string(), ast_node.clone(), false);

        let decls = interp
            .evaluate_member_access(ast_node, "declarations", None)
            .unwrap();
        if let ComptimeValue::Array(items) = &decls {
            assert_eq!(items.len(), 1);
            let func_name = interp
                .evaluate_member_access(items[0].clone(), "name", None)
                .unwrap();
            assert_eq!(func_name, ComptimeValue::String("add".to_string()));
        } else {
            panic!("Expected array of declarations");
        }
    }

    #[test]
    fn test_question_match_literal_patterns() {
        let mut interp = ComptimeInterpreter::new();
        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Integer32(42)),
            arms: vec![
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::Integer32(42)),
                    guard: None,
                    body: Expression::String("matched".to_string()),
                },
                ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::String("no match".to_string()),
                },
            ],
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("matched".to_string()));
    }

    #[test]
    fn test_question_match_wildcard_fallthrough() {
        let mut interp = ComptimeInterpreter::new();
        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Integer32(99)),
            arms: vec![
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::Integer32(42)),
                    guard: None,
                    body: Expression::String("forty-two".to_string()),
                },
                ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::String("other".to_string()),
                },
            ],
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("other".to_string()));
    }

    #[test]
    fn test_question_match_string_patterns() {
        let mut interp = ComptimeInterpreter::new();
        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::String("Function".to_string())),
            arms: vec![
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::String("Function".to_string())),
                    guard: None,
                    body: Expression::String("is func".to_string()),
                },
                ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::String("unknown".to_string()),
                },
            ],
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("is func".to_string()));
    }

    #[test]
    fn test_question_match_boolean_patterns() {
        let mut interp = ComptimeInterpreter::new();
        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Boolean(true)),
            arms: vec![
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::Boolean(true)),
                    guard: None,
                    body: Expression::String("yes".to_string()),
                },
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::Boolean(false)),
                    guard: None,
                    body: Expression::String("no".to_string()),
                },
            ],
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("yes".to_string()));
    }

    #[test]
    fn test_question_match_with_binding() {
        let mut interp = ComptimeInterpreter::new();
        let expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Integer32(42)),
            arms: vec![ast::MatchArm {
                pattern: Pattern::Identifier("n".to_string()),
                guard: None,
                body: Expression::Identifier("n".to_string()),
            }],
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::I32(42));
    }

    #[test]
    fn test_loop_with_condition() {
        let mut interp = ComptimeInterpreter::new();
        interp
            .env
            .define("i".to_string(), ComptimeValue::I32(0), true);
        let loop_stmt = Statement::Loop {
            kind: ast::LoopKind::Condition(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: ast::BinaryOperator::LessThan,
                right: Box::new(Expression::Integer32(5)),
            }),
            label: None,
            body: vec![Statement::VariableAssignment {
                name: "i".to_string(),
                value: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("i".to_string())),
                    op: ast::BinaryOperator::Add,
                    right: Box::new(Expression::Integer32(1)),
                },
                span: None,
            }],
            span: None,
        };
        interp.execute_statement(&loop_stmt).unwrap();
        assert_eq!(interp.env.get("i").unwrap(), ComptimeValue::I32(5));
    }

    #[test]
    fn test_loop_with_break() {
        let mut interp = ComptimeInterpreter::new();
        interp
            .env
            .define("i".to_string(), ComptimeValue::I32(0), true);
        let loop_stmt = Statement::Loop {
            kind: ast::LoopKind::Infinite,
            label: None,
            body: vec![
                Statement::VariableAssignment {
                    name: "i".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::Integer32(1)),
                    },
                    span: None,
                },
                Statement::Expression {
                    expr: Expression::QuestionMatch {
                        scrutinee: Box::new(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("i".to_string())),
                            op: ast::BinaryOperator::Equals,
                            right: Box::new(Expression::Integer32(3)),
                        }),
                        arms: vec![
                            ast::MatchArm {
                                pattern: Pattern::Literal(Expression::Boolean(true)),
                                guard: None,
                                body: Expression::Block(vec![Statement::Break {
                                    label: None,
                                    span: None,
                                }]),
                            },
                            ast::MatchArm {
                                pattern: Pattern::Wildcard,
                                guard: None,
                                body: Expression::Integer32(0),
                            },
                        ],
                    },
                    span: None,
                },
            ],
            span: None,
        };
        interp.execute_statement(&loop_stmt).unwrap();
        assert_eq!(interp.env.get("i").unwrap(), ComptimeValue::I32(3));
    }

    #[test]
    fn test_array_index_expression() {
        let mut interp = ComptimeInterpreter::new();
        interp.env.define(
            "arr".to_string(),
            ComptimeValue::Array(vec![
                ComptimeValue::I32(10),
                ComptimeValue::I32(20),
                ComptimeValue::I32(30),
            ]),
            false,
        );
        let expr = Expression::ArrayIndex {
            array: Box::new(Expression::Identifier("arr".to_string())),
            index: Box::new(Expression::Integer32(1)),
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::I32(20));
    }

    #[test]
    fn test_array_index_out_of_bounds() {
        let mut interp = ComptimeInterpreter::new();
        interp.env.define(
            "arr".to_string(),
            ComptimeValue::Array(vec![ComptimeValue::I32(1)]),
            false,
        );
        let expr = Expression::ArrayIndex {
            array: Box::new(Expression::Identifier("arr".to_string())),
            index: Box::new(Expression::Integer32(5)),
        };
        assert!(interp.evaluate_expression(&expr, None).is_err());
    }

    #[test]
    fn test_string_concatenation() {
        let mut interp = ComptimeInterpreter::new();
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::String("hello ".to_string())),
            op: ast::BinaryOperator::Add,
            right: Box::new(Expression::String("world".to_string())),
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("hello world".to_string()));
    }

    #[test]
    fn test_string_interpolation() {
        let mut interp = ComptimeInterpreter::new();
        interp.env.define(
            "name".to_string(),
            ComptimeValue::String("Zen".to_string()),
            false,
        );
        interp
            .env
            .define("ver".to_string(), ComptimeValue::I32(7), false);
        let expr = Expression::StringInterpolation {
            parts: vec![
                ast::StringPart::Interpolation(Expression::Identifier("name".to_string())),
                ast::StringPart::Literal(" v".to_string()),
                ast::StringPart::Interpolation(Expression::Identifier("ver".to_string())),
            ],
        };
        let result = interp.evaluate_expression(&expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("Zen v7".to_string()));
    }

    #[test]
    fn test_string_building_with_loop() {
        let mut interp = ComptimeInterpreter::new();
        interp.env.define(
            "result".to_string(),
            ComptimeValue::String("".to_string()),
            true,
        );
        interp
            .env
            .define("i".to_string(), ComptimeValue::I32(0), true);
        let loop_stmt = Statement::Loop {
            kind: ast::LoopKind::Condition(Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: ast::BinaryOperator::LessThan,
                right: Box::new(Expression::Integer32(5)),
            }),
            label: None,
            body: vec![
                Statement::VariableAssignment {
                    name: "result".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("result".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::StringInterpolation {
                            parts: vec![
                                ast::StringPart::Interpolation(Expression::Identifier(
                                    "i".to_string(),
                                )),
                                ast::StringPart::Literal(",".to_string()),
                            ],
                        }),
                    },
                    span: None,
                },
                Statement::VariableAssignment {
                    name: "i".to_string(),
                    value: Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::Integer32(1)),
                    },
                    span: None,
                },
            ],
            span: None,
        };
        interp.execute_statement(&loop_stmt).unwrap();
        assert_eq!(
            interp.env.get("result").unwrap(),
            ComptimeValue::String("0,1,2,3,4,".to_string())
        );
    }

    #[test]
    fn test_meta_ast_walk_with_pattern_match() {
        let mut interp = ComptimeInterpreter::new();
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();
        let meta_val = interp.env.get("meta").unwrap();
        let ast_node = interp
            .evaluate_method_call(
                meta_val,
                "parse",
                &[Expression::String(
                    "add = (a: i32, b: i32) i32 { return a + b }".to_string(),
                )],
                None,
            )
            .unwrap();
        interp.env.define("program".to_string(), ast_node, false);
        let prog = interp.env.get("program").unwrap();
        let func = interp
            .evaluate_method_call(
                prog,
                "find_function",
                &[Expression::String("add".to_string())],
                None,
            )
            .unwrap();
        let vname = interp
            .evaluate_method_call(func, "variant_name", &[], None)
            .unwrap();
        interp.env.define("vname".to_string(), vname, false);
        let match_expr = Expression::QuestionMatch {
            scrutinee: Box::new(Expression::Identifier("vname".to_string())),
            arms: vec![
                ast::MatchArm {
                    pattern: Pattern::Literal(Expression::String("Function".to_string())),
                    guard: None,
                    body: Expression::String("found function!".to_string()),
                },
                ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expression::String("other".to_string()),
                },
            ],
        };
        let result = interp.evaluate_expression(&match_expr, None).unwrap();
        assert_eq!(result, ComptimeValue::String("found function!".to_string()));
    }

    #[test]
    fn test_block_expression() {
        let mut interp = ComptimeInterpreter::new();
        let block = Expression::Block(vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                type_: None,
                initializer: Some(Expression::Integer32(10)),
                is_mutable: false,
                declaration_type: ast::VariableDeclarationType::InferredImmutable,
                span: None,
            },
            Statement::Expression {
                expr: Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: ast::BinaryOperator::Add,
                    right: Box::new(Expression::Integer32(5)),
                },
                span: None,
            },
        ]);
        let result = interp.evaluate_expression(&block, None).unwrap();
        assert_eq!(result, ComptimeValue::I32(15));
    }

    #[test]
    fn test_helper_functions_find() {
        let mut interp = ComptimeInterpreter::new();
        let import_stmt = Statement::DestructuringImport {
            names: vec!["meta".to_string()],
            source: Expression::StdReference,
            span: None,
        };
        interp.execute_statement(&import_stmt).unwrap();
        let meta_val = interp.env.get("meta").unwrap();
        let ast_node = interp
            .evaluate_method_call(meta_val, "parse",
                &[Expression::String("add = (a: i32, b: i32) i32 { return a + b }\nsub = (a: i32, b: i32) i32 { return a - b }".to_string())],
                None,
            )
            .unwrap();
        let funcs = interp
            .evaluate_method_call(ast_node.clone(), "functions", &[], None)
            .unwrap();
        if let ComptimeValue::Array(items) = &funcs {
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected array from functions()");
        }
        let sub_fn = interp
            .evaluate_method_call(
                ast_node.clone(),
                "find_function",
                &[Expression::String("sub".to_string())],
                None,
            )
            .unwrap();
        assert!(!matches!(sub_fn, ComptimeValue::Null));
        let none_fn = interp
            .evaluate_method_call(
                ast_node,
                "find_function",
                &[Expression::String("nonexistent".to_string())],
                None,
            )
            .unwrap();
        assert!(matches!(none_fn, ComptimeValue::Null));
    }
}
