use inkwell::context::Context;
use inkwell::execution_engine::JitFunction;
use inkwell::OptimizationLevel;
use zen::ast;
use zen::compiler::Compiler;

#[test]
fn test_simple_pattern_match_codegen() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // Create a simple pattern matching function
    let program = ast::Program {
        declarations: vec![
            ast::Declaration::Function(ast::Function {
                type_params: vec![],
                name: "test_match".to_string(),
                args: vec![("x".to_string(), ast::AstType::I32)],
                return_type: ast::AstType::I32,
                body: vec![
                    ast::Statement::Return(
                        ast::Expression::Conditional { 
                            scrutinee: Box::new(ast::Expression::Identifier("x".to_string())),
                            arms: vec![
                                ast::ConditionalArm {
                                    pattern: ast::Pattern::Literal(ast::Expression::Integer32(0)),
                                    guard: None,
                                    body: ast::Expression::Integer32(100),
                                },
                                ast::ConditionalArm {
                                    pattern: ast::Pattern::Literal(ast::Expression::Integer32(1)),
                                    guard: None,
                                    body: ast::Expression::Integer32(200),
                                },
                                ast::ConditionalArm {
                                    pattern: ast::Pattern::Wildcard,
                                    guard: None,
                                    body: ast::Expression::Integer32(300),
                                },
                            ],
                        }
                    ),
                ],
                is_async: false,
            }),
        ],
    };
    
    // Compile the program
    let module = compiler.get_module(&program).expect("Failed to compile program");
    
    // Create execution engine
    let execution_engine = module
        .create_jit_execution_engine(OptimizationLevel::None)
        .expect("Failed to create execution engine");
    
    // Get the function
    let test_fn: JitFunction<unsafe extern "C" fn(i32) -> i32> = unsafe {
        execution_engine.get_function("test_match").expect("Failed to get function")
    };
    
    // Test the pattern matching
    unsafe {
        assert_eq!(test_fn.call(0), 100, "Pattern match for 0 should return 100");
        assert_eq!(test_fn.call(1), 200, "Pattern match for 1 should return 200");
        assert_eq!(test_fn.call(2), 300, "Pattern match for 2 should return 300 (wildcard)");
        assert_eq!(test_fn.call(-1), 300, "Pattern match for -1 should return 300 (wildcard)");
        assert_eq!(test_fn.call(999), 300, "Pattern match for 999 should return 300 (wildcard)");
    }
}
