//! The high-level compiler orchestrator.
//! This module ties the frontend (parser) and the backend (codegen) together.

use crate::ast::{Declaration, Program};
use crate::codegen::llvm::LLVMCompiler;
use crate::comptime::ComptimeInterpreter;
use crate::error::{CompileError, Result};
use crate::module_system::{resolver::ModuleResolver, ModuleSystem};
use crate::type_system::Monomorphizer;
use crate::typechecker::TypeChecker;
use inkwell::context::Context;
use inkwell::module::Module;

/// The main compiler structure.
#[allow(dead_code)]
pub struct Compiler<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> Compiler<'ctx> {
    #[allow(dead_code)]
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Core compilation pipeline - shared by compile_llvm and get_module
    #[allow(dead_code)]
    fn run_pipeline(&self, program: &Program) -> Result<LLVMCompiler<'ctx>> {
        // Process module imports (this loads stdlib modules)
        let mut module_system = ModuleSystem::new();
        let processed_program = self.process_imports_with_system(program, &mut module_system)?;

        // Execute comptime blocks and expressions
        let processed_program = self.execute_comptime(processed_program)?;

        // Resolve Self types in trait implementations
        let processed_program = self.resolve_self_types(processed_program)?;

        // Type check the program and get TypeContext
        // Pass loaded stdlib modules to TypeChecker so it can extract type info
        let mut typechecker = TypeChecker::new();
        typechecker.with_stdlib_modules(module_system.get_modules());
        let type_ctx = typechecker.check_program(&processed_program)?;

        // Monomorphize the program to resolve all generic types
        // Monomorphizer uses TypeContext for type lookups
        let mut monomorphizer = Monomorphizer::new(type_ctx);
        let monomorphized_program = monomorphizer.monomorphize_program(&processed_program)?;

        // Get TypeContext back from monomorphizer (possibly updated with new instantiations)
        let type_ctx = monomorphizer.into_type_context();

        // Pass TypeContext to codegen so it can look up types instead of re-inferring
        let mut llvm_compiler = LLVMCompiler::new(self.context, type_ctx);
        llvm_compiler.compile_program(&monomorphized_program)?;

        // Debug: Print LLVM IR before verification for debugging
        if std::env::var("DEBUG_LLVM").is_ok() {
            eprintln!("LLVM IR:\n{}", llvm_compiler.module.print_to_string());
        }

        if let Err(e) = llvm_compiler.module.verify() {
            return Err(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        Ok(llvm_compiler)
    }

    /// Compiles a program using the LLVM backend.
    /// Returns the LLVM IR as a string.
    #[allow(dead_code)]
    pub fn compile_llvm(&self, program: &Program) -> Result<String> {
        let llvm_compiler = self.run_pipeline(program)?;
        Ok(llvm_compiler.module.print_to_string().to_string())
    }

    /// Gets the LLVM module after compilation for execution engine creation.
    #[allow(dead_code)]
    pub fn get_module(&self, program: &Program) -> Result<Module<'ctx>> {
        let llvm_compiler = self.run_pipeline(program)?;
        Ok(llvm_compiler.module)
    }

    /// Process module imports and merge imported modules
    #[allow(dead_code)]
    fn process_imports(&self, program: &Program) -> Result<Program> {
        let mut module_system = ModuleSystem::new();
        self.process_imports_with_system(program, &mut module_system)
    }

    /// Process module imports with a provided ModuleSystem (allows reuse)
    fn process_imports_with_system(
        &self,
        program: &Program,
        module_system: &mut ModuleSystem,
    ) -> Result<Program> {
        let mut resolver = ModuleResolver::new();

        // Process all module imports
        for decl in &program.declarations {
            if let Declaration::ModuleImport {
                alias, module_path, ..
            } = decl
            {
                // Load the module (including @std modules - they'll load from stdlib files)
                module_system.load_module(module_path)?;

                // Register the import with the resolver
                resolver.add_import(alias.clone(), module_path.clone());

                // Extract and register exports
                if let Some(module) = module_system.get_modules().get(module_path) {
                    let exports = ModuleResolver::extract_exports(module);
                    resolver.add_exports(module_path.clone(), exports);
                }
            }
        }

        // Merge all modules into a single program
        let mut merged_program = module_system.merge_programs(program.clone());

        // Resolve module references
        resolver
            .resolve_program(&mut merged_program)
            .map_err(|e| CompileError::InternalError(e, None))?;

        Ok(merged_program)
    }

    /// Execute comptime blocks and expressions in the program
    #[allow(dead_code)]
    fn execute_comptime(&self, program: Program) -> Result<Program> {
        let mut interpreter = ComptimeInterpreter::new();
        let mut new_declarations = Vec::new();

        for decl in program.declarations {
            match decl {
                Declaration::ComptimeBlock(statements) => {
                    // Execute the comptime block
                    interpreter.execute_comptime_block(&statements)?;

                    // Get any generated declarations from the comptime execution
                    let mut generated = interpreter.get_generated_declarations();
                    new_declarations.append(&mut generated);
                }
                other => {
                    // Process comptime expressions within the declaration
                    let processed = self.process_declaration_comptime(other, &mut interpreter)?;
                    new_declarations.push(processed);
                }
            }
        }

        Ok(Program {
            declarations: new_declarations,
            statements: Vec::new(),
        })
    }

    /// Process comptime expressions within a declaration
    fn process_declaration_comptime(
        &self,
        decl: Declaration,
        interpreter: &mut ComptimeInterpreter,
    ) -> Result<Declaration> {
        match decl {
            Declaration::Function(mut func) => {
                // Process comptime expressions in function body
                func.body = self.process_statements_comptime(func.body, interpreter)?;
                Ok(Declaration::Function(func))
            }
            Declaration::Struct(mut struct_def) => {
                // Process default values in struct fields
                for field in &mut struct_def.fields {
                    if let Some(default) = &field.default_value {
                        field.default_value =
                            Some(self.process_expression_comptime(default.clone(), interpreter)?);
                    }
                }
                Ok(Declaration::Struct(struct_def))
            }
            other => Ok(other),
        }
    }

    /// Process comptime expressions within statements
    fn process_statements_comptime(
        &self,
        statements: Vec<crate::ast::Statement>,
        interpreter: &mut ComptimeInterpreter,
    ) -> Result<Vec<crate::ast::Statement>> {
        let mut processed = Vec::new();

        for stmt in statements {
            processed.push(self.process_statement_comptime(stmt, interpreter)?);
        }

        Ok(processed)
    }

    /// Process a single statement for comptime expressions
    fn process_statement_comptime(
        &self,
        stmt: crate::ast::Statement,
        interpreter: &mut ComptimeInterpreter,
    ) -> Result<crate::ast::Statement> {
        use crate::ast::{Expression, Statement};

        match stmt {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                is_mutable,
                declaration_type,
                span,
            } => {
                let processed_initializer = if let Some(init) = initializer {
                    Some(self.process_expression_comptime(init, interpreter)?)
                } else {
                    None
                };

                Ok(Statement::VariableDeclaration {
                    name,
                    type_,
                    initializer: processed_initializer,
                    is_mutable,
                    declaration_type,
                    span,
                })
            }
            Statement::VariableAssignment { name, value, span } => {
                Ok(Statement::VariableAssignment {
                    name,
                    value: self.process_expression_comptime(value, interpreter)?,
                    span,
                })
            }
            Statement::PointerAssignment {
                pointer,
                value,
                span,
            } => Ok(Statement::PointerAssignment {
                pointer: self.process_expression_comptime(pointer, interpreter)?,
                value: self.process_expression_comptime(value, interpreter)?,
                span,
            }),
            Statement::Return { expr, span } => Ok(Statement::Return {
                expr: self.process_expression_comptime(expr, interpreter)?,
                span,
            }),
            Statement::Expression { expr, span } => Ok(Statement::Expression {
                expr: self.process_expression_comptime(expr, interpreter)?,
                span,
            }),
            Statement::Loop {
                kind,
                label,
                body,
                span,
            } => Ok(Statement::Loop {
                kind,
                label,
                body: self.process_statements_comptime(body, interpreter)?,
                span,
            }),
            Statement::ComptimeBlock { statements, .. } => {
                // Execute the comptime block inline
                interpreter.execute_comptime_block(&statements)?;
                // Comptime blocks are elided at runtime - no span needed
                Ok(Statement::Expression {
                    expr: Expression::Integer32(0),
                    span: None,
                })
            }
            other => Ok(other),
        }
    }

    /// Process a single expression for comptime evaluation
    #[allow(clippy::only_used_in_recursion)]
    fn process_expression_comptime(
        &self,
        expr: crate::ast::Expression,
        interpreter: &mut ComptimeInterpreter,
    ) -> Result<crate::ast::Expression> {
        use crate::ast::Expression;

        match expr {
            Expression::Comptime(inner) => {
                // Evaluate the comptime expression
                let value = interpreter.evaluate_expression(&inner)?;
                // Convert the computed value back to an expression
                value.to_expression()
            }
            Expression::BinaryOp { left, op, right } => Ok(Expression::BinaryOp {
                left: Box::new(self.process_expression_comptime(*left, interpreter)?),
                op,
                right: Box::new(self.process_expression_comptime(*right, interpreter)?),
            }),
            Expression::FunctionCall {
                name,
                type_args,
                args,
            } => {
                let mut processed_args = Vec::new();
                for arg in args {
                    processed_args.push(self.process_expression_comptime(arg, interpreter)?);
                }
                Ok(Expression::FunctionCall {
                    name: name.clone(),
                    type_args: type_args.clone(),
                    args: processed_args,
                })
            }
            Expression::ArrayLiteral(elements) => {
                let mut processed = Vec::new();
                for elem in elements {
                    processed.push(self.process_expression_comptime(elem, interpreter)?);
                }
                Ok(Expression::ArrayLiteral(processed))
            }
            Expression::StructLiteral { name, fields } => {
                let mut processed_fields = Vec::new();
                for (field_name, field_expr) in fields {
                    processed_fields.push((
                        field_name,
                        self.process_expression_comptime(field_expr, interpreter)?,
                    ));
                }
                Ok(Expression::StructLiteral {
                    name,
                    fields: processed_fields,
                })
            }
            Expression::StringInterpolation { parts } => {
                let mut processed_parts = Vec::new();
                for part in parts {
                    use crate::ast::StringPart;
                    match part {
                        StringPart::Literal(s) => processed_parts.push(StringPart::Literal(s)),
                        StringPart::Interpolation(e) => {
                            processed_parts.push(StringPart::Interpolation(
                                self.process_expression_comptime(e, interpreter)?,
                            ));
                        }
                    }
                }
                Ok(Expression::StringInterpolation {
                    parts: processed_parts,
                })
            }
            other => Ok(other),
        }
    }

    /// Resolve Self types in trait implementations
    #[allow(dead_code)]
    fn resolve_self_types(&self, program: Program) -> Result<Program> {
        use crate::typechecker::self_resolution::transform_trait_impl_self_types;

        let mut new_declarations = Vec::new();

        for decl in program.declarations {
            match decl {
                Declaration::TraitImplementation(trait_impl) => {
                    // Transform Self types to concrete types
                    let transformed = transform_trait_impl_self_types(&trait_impl)?;
                    new_declarations.push(Declaration::TraitImplementation(transformed));
                }
                other => new_declarations.push(other),
            }
        }

        Ok(Program {
            declarations: new_declarations,
            statements: program.statements,
        })
    }

    /// Analyze a program and collect all errors without stopping at the first one.
    /// This is useful for LSP to show all diagnostics at once.
    #[allow(dead_code)]
    pub fn analyze_for_diagnostics(&self, program: &Program) -> Vec<CompileError> {
        let mut errors = Vec::new();

        // Try to process imports
        let processed_program = match self.process_imports(program) {
            Ok(p) => p,
            Err(err) => {
                errors.push(err);
                return errors; // Can't continue without imports
            }
        };

        // Try to execute comptime blocks
        let processed_program = match self.execute_comptime(processed_program) {
            Ok(p) => p,
            Err(err) => {
                errors.push(err);
                return errors; // Can't continue without comptime
            }
        };

        // Try to resolve Self types
        let processed_program = match self.resolve_self_types(processed_program) {
            Ok(p) => p,
            Err(err) => {
                errors.push(err);
                return errors; // Can't continue without Self resolution
            }
        };

        // Try to typecheck
        let mut typechecker = TypeChecker::new();
        let type_ctx = match typechecker.check_program(&processed_program) {
            Ok(ctx) => ctx,
            Err(err) => {
                errors.push(err);
                return errors; // Can't continue without type checking
            }
        };

        // Try to monomorphize (uses TypeContext for type lookups)
        let mut monomorphizer = Monomorphizer::new(type_ctx);
        let monomorphized_program = match monomorphizer.monomorphize_program(&processed_program) {
            Ok(p) => p,
            Err(err) => {
                errors.push(err);
                return errors; // Can't continue without monomorphization
            }
        };

        // Get TypeContext back from monomorphizer
        let type_ctx = monomorphizer.into_type_context();

        // Try to compile to LLVM
        let mut llvm_compiler = LLVMCompiler::new(self.context, type_ctx);
        if let Err(err) = llvm_compiler.compile_program(&monomorphized_program) {
            errors.push(err);
            return errors; // Compilation failed
        }

        // Try to verify LLVM module
        if let Err(e) = llvm_compiler.module.verify() {
            errors.push(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        errors
    }
}
