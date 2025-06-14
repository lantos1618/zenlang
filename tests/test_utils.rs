use inkwell::{
    context::Context,
    execution_engine::JitFunction,
    module::Module,
    OptimizationLevel,
};
use lynlang::{
    ast::{self, AstType, Expression, Statement},
    compiler::Compiler,
    error::CompileError,
};
use std::ops::{Deref, DerefMut};

/// A test context that manages the LLVM context, module, and compiler state
/// for running tests. This ensures all compilation and execution use the same
/// module and context.
pub struct TestContext<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    compiler: Compiler<'ctx>,
}

impl<'ctx> TestContext<'ctx> {
    /// Creates a new test context with a fresh module and compiler.
    pub fn new(context: &'ctx Context) -> Self {
        let module = context.create_module("test");
        let compiler = Compiler::new(context);
        Self {
            context,
            module,
            compiler,
        }
    }

    /// Compiles a program into the test context's module.
    pub fn compile(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        // Create a new module for this compilation
        self.module = self.context.create_module("test");
        self.compiler.module = self.module.clone();
        self.compiler.compile_program(program)
    }

    /// Runs a compiled program and returns its result.
    /// The program must have a 'main' function that returns an i64.
    pub fn run(&self) -> Result<i64, String> {
        let execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create JIT engine: {}", e))?;

        let jit_function: JitFunction<unsafe extern "C" fn() -> i64> = unsafe {
            execution_engine
                .get_function("main")
                .map_err(|e| format!("Failed to get main function: {}", e))?
        };

        Ok(unsafe { jit_function.call() })
    }

    /// Gets the IR string for the current module.
    pub fn get_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Creates a simple test program that returns a constant value.
    pub fn create_simple_program(value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![Statement::Return(Expression::Integer64(value))],
        }])
    }

    /// Creates a test program that performs a binary operation.
    pub fn create_binary_op_program(
        left: i64,
        op: ast::BinaryOperator,
        right: i64,
    ) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Integer64(left)),
                op,
                right: Box::new(Expression::Integer64(right)),
            })],
        }])
    }

    /// Creates a test program that declares and returns a variable.
    pub fn create_variable_program(name: &str, value: i64) -> ast::Program {
        ast::Program::from_functions(vec![ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::Int64,
            body: vec![
                Statement::VariableDeclaration {
                    name: name.to_string(),
                    type_: AstType::Int64,
                    initializer: Some(Expression::Integer64(value)),
                },
                Statement::Return(Expression::Identifier(name.to_string())),
            ],
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
                args: vec![("arg".to_string(), return_type.clone())],
                return_type: return_type.clone(),
                body: vec![Statement::Return(Expression::Identifier("arg".to_string()))],
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: func_name.to_string(),
                    args,
                })],
            },
        ])
    }

    /// Creates a test program that declares and returns a function.
    pub fn create_function_program(name: &str, return_type: AstType) -> ast::Program {
        ast::Program::from_functions(vec![
            ast::Function {
                name: name.to_string(),
                args: vec![("arg".to_string(), return_type.clone())],
                return_type: return_type.clone(),
                body: vec![Statement::Return(Expression::Identifier("arg".to_string()))],
            },
            ast::Function {
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::Int64,
                body: vec![Statement::Return(Expression::Integer64(0))],
            },
        ])
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

/// Helper macro to create a test context and run a test.
#[macro_export]
macro_rules! test_context {
    ($body:expr) => {{
        let context = inkwell::context::Context::create();
        let mut test_context = $crate::test_utils::TestContext::new(&context);
        $body(&mut test_context)
    }};
} 