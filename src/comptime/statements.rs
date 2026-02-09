// Statement execution for the comptime interpreter

use crate::ast::{self, Statement};
use crate::error::{CompileError, Result};

use super::values::*;
use super::ComptimeInterpreter;
use super::Environment;

impl ComptimeInterpreter {
    /// Execute a compile-time block (public API — converts control flow signals to errors)
    pub fn execute_comptime_block(&mut self, statements: &[Statement]) -> Result<()> {
        for stmt in statements {
            match self.execute_statement(stmt) {
                Ok(_) => {}
                Err(ComptimeSignal::Error(e)) => return Err(e),
                Err(ComptimeSignal::Flow(cf)) => {
                    return Err(CompileError::ComptimeError(
                        format!("Unexpected {} outside of loop/function", cf),
                        None,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Execute a single statement. Returns `StmtResult` which can carry
    /// control flow signals (break/continue) without abusing the error type.
    pub fn execute_statement(&mut self, stmt: &Statement) -> StmtResult {
        match stmt {
            Statement::VariableDeclaration {
                name,
                initializer,
                is_mutable,
                span,
                ..
            } => {
                if let Some(init) = initializer {
                    let value = self
                        .evaluate_expression(init, span.clone())
                        .map_err(ComptimeSignal::Error)?;
                    self.env.define(name.clone(), value, *is_mutable);
                }
                Ok(None)
            }

            Statement::VariableAssignment { name, value, span } => {
                let val = self
                    .evaluate_expression(value, span.clone())
                    .map_err(ComptimeSignal::Error)?;
                self.env
                    .set(name, val, span.clone())
                    .map_err(ComptimeSignal::Error)?;
                Ok(None)
            }

            Statement::Expression { expr, span } => {
                let value = self
                    .evaluate_expression(expr, span.clone())
                    .map_err(ComptimeSignal::Error)?;
                Ok(Some(value))
            }

            Statement::Return { expr, span } => {
                let value = self
                    .evaluate_expression(expr, span.clone())
                    .map_err(ComptimeSignal::Error)?;
                Ok(Some(value))
            }

            Statement::ComptimeBlock {
                statements: stmts, ..
            } => {
                self.execute_comptime_block(stmts)
                    .map_err(ComptimeSignal::Error)?;
                Ok(None)
            }

            Statement::DestructuringImport { names, source, .. } => {
                let source_val = self
                    .evaluate_expression(source, None)
                    .map_err(ComptimeSignal::Error)?;
                if let ComptimeValue::Struct { fields, .. } = source_val {
                    for name in names {
                        if let Some(val) = fields.get(name) {
                            self.env.define(name.clone(), val.clone(), false);
                        } else {
                            return Err(ComptimeSignal::Error(CompileError::ComptimeError(
                                format!("Module has no member '{}'", name),
                                None,
                            )));
                        }
                    }
                }
                Ok(None)
            }

            Statement::Block { statements, .. } => {
                let mut result = None;
                for s in statements {
                    result = self.execute_statement(s)?;
                }
                Ok(result)
            }

            Statement::Loop { kind, body, .. } => self.execute_loop(kind, body),

            // Control flow signals — these are NOT errors
            Statement::Break { .. } => Err(ComptimeSignal::Flow(ComptimeControlFlow::Break)),
            Statement::Continue { .. } => Err(ComptimeSignal::Flow(ComptimeControlFlow::Continue)),

            _ => Err(ComptimeSignal::Error(CompileError::ComptimeError(
                format!("Statement type not supported in comptime: {:?}", stmt),
                None,
            ))),
        }
    }

    /// Execute a loop statement (loop condition { body })
    pub(super) fn execute_loop(&mut self, kind: &ast::LoopKind, body: &[Statement]) -> StmtResult {
        const MAX_ITERATIONS: usize = 100_000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(ComptimeSignal::Error(CompileError::ComptimeError(
                    format!(
                        "Compile-time loop exceeded {} iterations (infinite loop?)",
                        MAX_ITERATIONS
                    ),
                    None,
                )));
            }

            // Check loop condition
            if let ast::LoopKind::Condition(cond) = kind {
                let cond_val = self
                    .evaluate_expression(cond, None)
                    .map_err(ComptimeSignal::Error)?;
                match cond_val {
                    ComptimeValue::Bool(false) => return Ok(None),
                    ComptimeValue::Bool(true) => {}
                    _ => {
                        return Err(ComptimeSignal::Error(CompileError::ComptimeError(
                            "Loop condition must evaluate to a boolean".to_string(),
                            None,
                        )));
                    }
                }
            }

            // Execute body — intercept break/continue signals (both direct and tunneled)
            let mut should_break = false;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(Some(val)) => {
                        if matches!(stmt, Statement::Return { .. }) {
                            return Ok(Some(val));
                        }
                    }
                    Ok(None) => {}
                    // Direct signals from Statement::Break / Statement::Continue
                    Err(ComptimeSignal::Flow(ComptimeControlFlow::Break)) => {
                        should_break = true;
                        break;
                    }
                    Err(ComptimeSignal::Flow(ComptimeControlFlow::Continue)) => {
                        break;
                    }
                    // Tunneled signals from break/continue inside Expression::Block
                    Err(ComptimeSignal::Error(ref e))
                        if error_to_flow(e) == Some(ComptimeControlFlow::Break) =>
                    {
                        should_break = true;
                        break;
                    }
                    Err(ComptimeSignal::Error(ref e))
                        if error_to_flow(e) == Some(ComptimeControlFlow::Continue) =>
                    {
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }

            if should_break {
                return Ok(None);
            }
        }
    }

    /// Evaluate function calls — builtins via enum dispatch, user functions via env lookup
    pub(super) fn evaluate_function_call(
        &mut self,
        module: Option<&str>,
        name: &str,
        args: &[crate::ast::Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        use crate::ast::builtins::BuiltinFn;

        // Handle @builtin.* intrinsic calls (used in build.zen)
        if let Some(module) = module {
            if crate::intrinsics::is_intrinsic_module(module) {
                return self.call_build_intrinsic(name, args, span);
            }
        }

        if let Some(builtin) = BuiltinFn::from_name(name) {
            return self.call_builtin(builtin, args, span);
        }

        // User-defined function
        self.call_user_function(name, args, span)
    }

    /// Handle @builtin.* intrinsic calls in comptime (e.g., @builtin.import_std())
    fn call_build_intrinsic(
        &mut self,
        func: &str,
        args: &[crate::ast::Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        match func {
            "import_std" => {
                // @builtin.import_std() → a marker struct representing stdlib
                Ok(ComptimeValue::Struct {
                    name: "__package".to_string(),
                    fields: {
                        let mut f = std::collections::HashMap::new();
                        f.insert(
                            "kind".to_string(),
                            ComptimeValue::String("stdlib".to_string()),
                        );
                        f
                    },
                })
            }
            "import" => {
                // @builtin.import("url_or_path") → marker struct for remote/local package
                if args.is_empty() {
                    return Err(CompileError::ComptimeError(
                        format!(
                            "{}.import() requires a string argument",
                            crate::intrinsics::INTRINSIC_PREFIX
                        ),
                        span,
                    ));
                }
                let val = self.evaluate_expression(&args[0], span.clone())?;
                match val {
                    ComptimeValue::String(url) => {
                        let kind = if url.starts_with("./") || url.starts_with("../") {
                            "local"
                        } else {
                            "remote"
                        };
                        Ok(ComptimeValue::Struct {
                            name: "__package".to_string(),
                            fields: {
                                let mut f = std::collections::HashMap::new();
                                f.insert(
                                    "kind".to_string(),
                                    ComptimeValue::String(kind.to_string()),
                                );
                                f.insert("url".to_string(), ComptimeValue::String(url));
                                f
                            },
                        })
                    }
                    _ => Err(CompileError::ComptimeError(
                        format!(
                            "{}.import() expects a string argument",
                            crate::intrinsics::INTRINSIC_PREFIX
                        ),
                        span,
                    )),
                }
            }
            _ => Err(CompileError::ComptimeError(
                format!(
                    "Unknown {} intrinsic: {}",
                    crate::intrinsics::INTRINSIC_PREFIX,
                    func
                ),
                span,
            )),
        }
    }

    fn call_builtin(
        &mut self,
        builtin: crate::ast::builtins::BuiltinFn,
        args: &[crate::ast::Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        use crate::ast::builtins::BuiltinFn;
        use crate::ast::AstType;

        let val = self.require_one_arg(args, &builtin, span.clone())?;

        match builtin {
            BuiltinFn::Sizeof => {
                let arg_type = val.get_type();
                let size = arg_type.byte_size().unwrap_or(match &arg_type {
                    _ if arg_type.is_ptr_type() => 8,
                    AstType::Ref(_) => 8,
                    _ => 8,
                }) as i64;
                Ok(ComptimeValue::I64(size))
            }
            BuiltinFn::Typeof => Ok(ComptimeValue::Type(val.get_type())),
            BuiltinFn::Emit => match val {
                ComptimeValue::ASTNode(ref node) => {
                    if let ASTNodeValue::Declaration(decl) = node.as_ref() {
                        self.push_declaration(decl.clone());
                        Ok(ComptimeValue::Void)
                    } else {
                        Err(CompileError::ComptimeError(
                            "emit() expects a Declaration ASTNode".to_string(),
                            span,
                        ))
                    }
                }
                _ => Err(CompileError::ComptimeError(
                    "emit() expects an ASTNode argument".to_string(),
                    span,
                )),
            },
            BuiltinFn::ComptimeAssert => match val {
                ComptimeValue::Bool(true) => Ok(ComptimeValue::Void),
                ComptimeValue::Bool(false) => Err(CompileError::ComptimeError(
                    "Compile-time assertion failed".to_string(),
                    span,
                )),
                _ => Err(CompileError::ComptimeError(
                    "comptime_assert expects a boolean".to_string(),
                    span,
                )),
            },
        }
    }

    fn call_user_function(
        &mut self,
        name: &str,
        args: &[crate::ast::Expression],
        span: Option<crate::error::Span>,
    ) -> Result<ComptimeValue> {
        if let Some(ComptimeValue::Function {
            params,
            body,
            closure,
            ..
        }) = self.env.get(name)
        {
            let func_env = Environment::with_parent(closure);

            if args.len() != params.len() {
                return Err(CompileError::ComptimeError(
                    format!(
                        "Function {} expects {} arguments, got {}",
                        name,
                        params.len(),
                        args.len()
                    ),
                    span.clone(),
                ));
            }

            for (param, arg) in params.iter().zip(args) {
                let val = self.evaluate_expression(arg, span.clone())?;
                func_env.define(param.clone(), val, false);
            }

            let saved_env = std::mem::replace(&mut self.env, func_env);
            let mut result = ComptimeValue::Void;

            for stmt in &body {
                match self.execute_statement(stmt) {
                    Ok(Some(val)) => {
                        result = val;
                        break;
                    }
                    Ok(None) => {}
                    Err(ComptimeSignal::Error(e)) => {
                        self.env = saved_env;
                        return Err(e);
                    }
                    Err(ComptimeSignal::Flow(cf)) => {
                        self.env = saved_env;
                        return Err(CompileError::ComptimeError(
                            format!("{} is not allowed in function body at comptime", cf),
                            span,
                        ));
                    }
                }
            }

            self.env = saved_env;
            Ok(result)
        } else {
            Err(CompileError::ComptimeError(
                format!("Unknown function: {}", name),
                span,
            ))
        }
    }
}
