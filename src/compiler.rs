//! The high-level compiler orchestrator.
//! This module ties the frontend (parser) and the backend (codegen) together.

use crate::ast::{Program, Declaration};
use crate::codegen::llvm::LLVMCompiler;
use crate::comptime::ComptimeInterpreter;
use crate::error::{CompileError, Result};
use crate::module_system::{ModuleSystem, resolver::ModuleResolver};
use crate::type_system::Monomorphizer;
use inkwell::context::Context;
use inkwell::module::Module;

/// The main compiler structure.
pub struct Compiler<'ctx> {
    context: &'ctx Context,
}

impl<'ctx> Compiler<'ctx> {
    pub fn new(context: &'ctx Context) -> Self {
        Self { context }
    }

    /// Compiles a program using the LLVM backend.
    /// In the future, this could take a `target` enum.
    pub fn compile_llvm(&self, program: &Program) -> Result<String> {
        // Process module imports
        let processed_program = self.process_imports(program)?;
        
        // Execute comptime blocks and expressions
        let processed_program = self.execute_comptime(processed_program)?;
        
        // Monomorphize the program to resolve all generic types
        let mut monomorphizer = Monomorphizer::new();
        let monomorphized_program = monomorphizer.monomorphize_program(&processed_program)?;
        
        let mut llvm_compiler = LLVMCompiler::new(self.context);
        llvm_compiler.compile_program(&monomorphized_program)?;

        if let Err(e) = llvm_compiler.module.verify() {
            return Err(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        Ok(llvm_compiler.module.print_to_string().to_string())
    }

    /// Gets the LLVM module after compilation for execution engine creation.
    pub fn get_module(&self, program: &Program) -> Result<Module<'ctx>> {
        // Process module imports
        let processed_program = self.process_imports(program)?;
        
        // Execute comptime blocks and expressions
        let processed_program = self.execute_comptime(processed_program)?;
        
        // Monomorphize the program to resolve all generic types
        let mut monomorphizer = Monomorphizer::new();
        let monomorphized_program = monomorphizer.monomorphize_program(&processed_program)?;
        
        let mut llvm_compiler = LLVMCompiler::new(self.context);
        llvm_compiler.compile_program(&monomorphized_program)?;

        if let Err(e) = llvm_compiler.module.verify() {
            return Err(CompileError::InternalError(
                format!("LLVM verification error: {}", e.to_string()),
                None,
            ));
        }

        Ok(llvm_compiler.module)
    }
    
    /// Process module imports and merge imported modules
    fn process_imports(&self, program: &Program) -> Result<Program> {
        let mut module_system = ModuleSystem::new();
        let mut resolver = ModuleResolver::new();
        
        // Process all module imports
        for decl in &program.declarations {
            if let Declaration::ModuleImport { alias, module_path } = decl {
                // Load the module
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
        resolver.resolve_program(&mut merged_program)
            .map_err(|e| CompileError::InternalError(e, None))?;
        
        Ok(merged_program)
    }
    
    /// Execute comptime blocks and expressions in the program
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
        })
    }
    
    /// Process comptime expressions within a declaration
    fn process_declaration_comptime(&self, decl: Declaration, interpreter: &mut ComptimeInterpreter) -> Result<Declaration> {
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
                        field.default_value = Some(self.process_expression_comptime(default.clone(), interpreter)?);
                    }
                }
                Ok(Declaration::Struct(struct_def))
            }
            other => Ok(other),
        }
    }
    
    /// Process comptime expressions within statements
    fn process_statements_comptime(&self, statements: Vec<crate::ast::Statement>, interpreter: &mut ComptimeInterpreter) -> Result<Vec<crate::ast::Statement>> {
        let mut processed = Vec::new();
        
        for stmt in statements {
            processed.push(self.process_statement_comptime(stmt, interpreter)?);
        }
        
        Ok(processed)
    }
    
    /// Process a single statement for comptime expressions
    fn process_statement_comptime(&self, stmt: crate::ast::Statement, interpreter: &mut ComptimeInterpreter) -> Result<crate::ast::Statement> {
        use crate::ast::{Statement, Expression};
        
        match stmt {
            Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } => {
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
                })
            }
            Statement::VariableAssignment { name, value } => {
                Ok(Statement::VariableAssignment {
                    name,
                    value: self.process_expression_comptime(value, interpreter)?,
                })
            }
            Statement::PointerAssignment { pointer, value } => {
                Ok(Statement::PointerAssignment {
                    pointer: self.process_expression_comptime(pointer, interpreter)?,
                    value: self.process_expression_comptime(value, interpreter)?,
                })
            }
            Statement::Return(expr) => {
                Ok(Statement::Return(self.process_expression_comptime(expr, interpreter)?))
            }
            Statement::Expression(expr) => {
                Ok(Statement::Expression(self.process_expression_comptime(expr, interpreter)?))
            }
            Statement::Loop { kind, label, body } => {
                Ok(Statement::Loop {
                    kind,
                    label,
                    body: self.process_statements_comptime(body, interpreter)?,
                })
            }
            Statement::ComptimeBlock(statements) => {
                // Execute the comptime block inline
                interpreter.execute_comptime_block(&statements)?;
                // Comptime blocks don't produce runtime statements
                Ok(Statement::Expression(Expression::Integer32(0))) // placeholder
            }
            other => Ok(other),
        }
    }
    
    /// Process a single expression for comptime evaluation
    fn process_expression_comptime(&self, expr: crate::ast::Expression, interpreter: &mut ComptimeInterpreter) -> Result<crate::ast::Expression> {
        use crate::ast::Expression;
        
        match expr {
            Expression::Comptime(inner) => {
                // Evaluate the comptime expression
                let value = interpreter.evaluate_expression(&inner)?;
                // Convert the computed value back to an expression
                value.to_expression()
            }
            Expression::BinaryOp { left, op, right } => {
                Ok(Expression::BinaryOp {
                    left: Box::new(self.process_expression_comptime(*left, interpreter)?),
                    op,
                    right: Box::new(self.process_expression_comptime(*right, interpreter)?),
                })
            }
            Expression::FunctionCall { name, args } => {
                let mut processed_args = Vec::new();
                for arg in args {
                    processed_args.push(self.process_expression_comptime(arg, interpreter)?);
                }
                Ok(Expression::FunctionCall {
                    name,
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
                    processed_fields.push((field_name, self.process_expression_comptime(field_expr, interpreter)?));
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
                            processed_parts.push(StringPart::Interpolation(self.process_expression_comptime(e, interpreter)?));
                        }
                    }
                }
                Ok(Expression::StringInterpolation { parts: processed_parts })
            }
            other => Ok(other),
        }
    }
} 