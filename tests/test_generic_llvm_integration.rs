use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::compiler::Compiler;
use inkwell::context::Context;

#[test]
fn test_generic_function_monomorphization_and_llvm() {
    let input = r#"
        identity<T> = (x: T) T {
            x
        }
        
        main = () i32 {
            a := identity(42);
            b := identity(3.14);
            a
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // This should monomorphize identity into identity_i32 and identity_f64
    // then compile to LLVM IR
    let llvm_ir = compiler.compile_llvm(&program).expect("Failed to compile to LLVM");
    
    // Check that the LLVM IR contains monomorphized versions
    assert!(llvm_ir.contains("identity_i32") || llvm_ir.contains("define i32 @identity"));
    assert!(!llvm_ir.contains("identity<T>"));
}

#[test]
fn test_generic_struct_monomorphization_and_llvm() {
    let input = r#"
        struct Box<T> {
            value: T
        }
        
        main = () i32 {
            box1 := Box { value: 42 };
            box2 := Box { value: 3.14 };
            box1.value
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // This should monomorphize Box into Box_i32 and Box_f64
    let llvm_ir = compiler.compile_llvm(&program).expect("Failed to compile to LLVM");
    
    // Check that the LLVM IR contains monomorphized struct types
    assert!(llvm_ir.contains("Box_i32") || llvm_ir.contains("%Box"));
    assert!(!llvm_ir.contains("Box<T>"));
}

#[test]
fn test_nested_generic_monomorphization() {
    let input = r#"
        pair<T, U> = (a: T, b: U) T {
            a
        }
        
        main = () i32 {
            result := pair(10, 20.5);
            result
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // Should monomorphize pair<i32, f64>
    let llvm_ir = compiler.compile_llvm(&program).expect("Failed to compile to LLVM");
    
    // Check that monomorphization happened
    assert!(llvm_ir.contains("pair_i32_f64") || llvm_ir.contains("define i32 @pair"));
}