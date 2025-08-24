use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::compiler::Compiler;
use inkwell::context::Context;

#[test]
fn test_non_generic_function_still_works() {
    let input = r#"
        add :: (a: i32, b: i32) -> i32 {
            a + b
        }
        
        main :: () -> i32 {
            result := add(10, 20);
            result
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // This should work even with monomorphization since there are no generics
    let llvm_ir = compiler.compile_llvm(&program).expect("Failed to compile to LLVM");
    
    // Check that the LLVM IR contains our functions
    assert!(llvm_ir.contains("@add"));
    assert!(llvm_ir.contains("@main"));
}

#[test]
fn test_simple_main_function() {
    let input = r#"
        main :: () -> i32 {
            42
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    let llvm_ir = compiler.compile_llvm(&program).expect("Failed to compile to LLVM");
    
    // Check that main is defined
    assert!(llvm_ir.contains("define i32 @main"));
}