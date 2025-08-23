// This file is intentionally empty to satisfy the Cargo.toml target requirement.
// You can add actual test utilities here as needed. 

use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use zen::compiler::Compiler;
use zen::codegen::llvm::LLVMCompiler;
use zen::error::CompileError;
use std::ops::{Deref, DerefMut};

/// A test context that manages the LLVM context and compiler state
/// for running tests. This ensures all compilation and execution use the same
/// context.
pub struct TestContext<'ctx> {
    context: &'ctx Context,
    compiler: Compiler<'ctx>,
    llvm_compiler: Option<LLVMCompiler<'ctx>>,
}

impl<'ctx> TestContext<'ctx> {
    /// Creates a new test context with a fresh compiler.
    pub fn new(context: &'ctx Context) -> Self {
        let compiler = Compiler::new(context);
        Self {
            context,
            compiler,
            llvm_compiler: None,
        }
    }

    /// Compiles a program and returns the LLVM IR as a string.
    pub fn compile(&mut self, program: &ast::Program) -> Result<String, CompileError> {
        // Create a new LLVM compiler and compile the program
        let mut llvm_compiler = LLVMCompiler::new(self.context);
        llvm_compiler.compile_program(program)?;
        
        // Store the compiler for later use
        self.llvm_compiler = Some(llvm_compiler);
        
        // Return the IR as a string
        Ok(self.llvm_compiler.as_ref().unwrap().module.print_to_string().to_string())
    }

    /// Gets the LLVM IR as a string.
    pub fn get_ir(&self) -> Result<String, CompileError> {
        if let Some(ref llvm_compiler) = self.llvm_compiler {
            Ok(llvm_compiler.module.print_to_string().to_string())
        } else {
            Err(CompileError::InternalError("No LLVM compiler available".to_string(), None))
        }
    }

    /// Runs a compiled program and returns the result.
    pub fn run(&self) -> Result<i64, CompileError> {
        if let Some(ref llvm_compiler) = self.llvm_compiler {
            let execution_engine = llvm_compiler.module
                .create_jit_execution_engine(OptimizationLevel::None)
                .map_err(|e| CompileError::InternalError(format!("Failed to create execution engine: {}", e), None))?;

            let main_function: JitFunction<unsafe extern "C" fn() -> i64> = unsafe {
                execution_engine
                    .get_function("main")
                    .map_err(|e| CompileError::InternalError(format!("Failed to get main function: {}", e), None))?
            };

            unsafe {
                Ok(main_function.call())
            }
        } else {
            Err(CompileError::InternalError("No LLVM compiler available".to_string(), None))
        }
    }

    /// Gets access to the LLVM module for testing purposes.
    pub fn module(&self) -> Result<&inkwell::module::Module<'ctx>, CompileError> {
        if let Some(ref llvm_compiler) = self.llvm_compiler {
            Ok(&llvm_compiler.module)
        } else {
            Err(CompileError::InternalError("No LLVM compiler available".to_string(), None))
        }
    }

    /// Gets mutable access to the LLVM compiler for testing purposes.
    pub fn llvm_compiler_mut(&mut self) -> Result<&mut LLVMCompiler<'ctx>, CompileError> {
        if let Some(ref mut llvm_compiler) = self.llvm_compiler {
            Ok(llvm_compiler)
        } else {
            Err(CompileError::InternalError("No LLVM compiler available".to_string(), None))
        }
    }

    /// Creates a simple test program that returns a constant value.
    pub fn create_simple_program(value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            type_params: vec![],
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Integer64(value))],
            is_async: false,
        }])
    }

    /// Creates a test program that performs a binary operation.
    pub fn create_binary_op_program(
        left: i64,
        op: BinaryOperator,
        right: i64,
    ) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            type_params: vec![],
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Integer64(left)),
                op,
                right: Box::new(Expression::Integer64(right)),
            })],
            is_async: false,
        }])
    }

    /// Creates a test program that declares and returns a variable.
    pub fn create_variable_program(name: &str, value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            type_params: vec![],
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration {
                    name: name.to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(value)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Identifier(name.to_string())),
            ],
            is_async: false,
        }])
    }

    /// Creates a test program that calls a function.
    pub fn create_function_call_program(
        func_name: &str,
        args: Vec<Expression>,
        return_type: AstType,
    ) -> ast::Program {
        ast::Program::from_functions(vec![
            ast::Function {
                name: func_name.to_string(),
                type_params: vec![],
                args: vec![("arg".to_string(), return_type.clone())],
                return_type: return_type.clone(),
                body: vec![Statement::Return(Expression::Identifier("arg".to_string()))],
                is_async: false,
            },
            ast::Function {
                name: "main".to_string(),
                type_params: vec![],
                args: vec![],
                return_type: return_type,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: func_name.to_string(),
                    args,
                })],
                is_async: false,
            },
        ])
    }

    /// Creates a test program with a function that returns a specific type.
    pub fn create_function_program(name: &str, return_type: AstType) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: name.to_string(),
            type_params: vec![],
            args: vec![],
            return_type: return_type,
            body: vec![Statement::Return(Expression::Integer64(42))], // Default return value
            is_async: false,
        }])
    }
}

impl<'ctx> Deref for TestContext<'ctx> {
    type Target = Compiler<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.compiler
    }
}

impl<'ctx> DerefMut for TestContext<'ctx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.compiler
    }
}

/// Macro to create a test context and run a test
#[macro_export]
macro_rules! test_context {
    (|$test_context:ident: &mut TestContext| $body:block) => {
        let context = Context::create();
        let mut $test_context = TestContext::new(&context);
        $body
    };
} 