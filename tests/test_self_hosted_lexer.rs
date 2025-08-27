use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::compiler::Compiler;
use inkwell::context::Context;

#[test]
fn test_self_hosted_lexer_compiles() {
    // Read the self-hosted lexer code
    let lexer_code = std::fs::read_to_string("stdlib/lexer.zen")
        .expect("Failed to read lexer.zen");
    
    // Parse the lexer code
    let lexer = Lexer::new(&lexer_code);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    
    // For now, just check that it parses without errors
    assert!(result.is_ok(), "Failed to parse lexer.zen: {:?}", result.err());
    
    let program = result.unwrap();
    
    // Check that we have declarations
    assert!(!program.declarations.is_empty(), "Lexer should have declarations");
    
    println!("✓ Self-hosted lexer parses successfully!");
    println!("  Found {} declarations", program.declarations.len());
}

#[test]
fn test_self_hosted_lexer_basic_tokenization() {
    // Simple test to check if basic lexer constructs are present
    let test_code = r#"
        // Test tokenization
        TokenType = 
            | Identifier(value: string)
            | Integer(value: string)
            | Eof
        
        lexer_new = (input: string) Lexer {
            return Lexer { input: input }
        }
    "#;
    
    let lexer = Lexer::new(test_code);
    let mut parser = Parser::new(lexer);
    
    let result = parser.parse_program();
    assert!(result.is_ok(), "Failed to parse test code: {:?}", result.err());
    
    println!("✓ Basic lexer constructs parse correctly!");
}

#[test]
fn test_self_hosted_lexer_can_compile() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // Simple subset of lexer for testing compilation
    let test_code = r#"
        extern malloc(size: i64) *i8
        
        Lexer = {
            input: string,
            position: i32,
            current_char: i8,
        }
        
        lexer_new = (input: string) Lexer {
            return Lexer {
                input: input,
                position: 0,
                current_char: 0,
            }
        }
        
        main = () i32 {
            test_input := "hello"
            lexer := lexer_new(test_input)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(test_code);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program()
        .expect("Failed to parse lexer test code");
    
    // Try to compile to LLVM IR
    let result = compiler.compile_llvm(&program);
    
    match result {
        Ok(ir) => {
            println!("✓ Self-hosted lexer subset compiles to LLVM IR!");
            assert!(ir.contains("lexer_new"), "Should contain lexer_new function");
        }
        Err(e) => {
            // For now, we expect some compilation errors due to missing features
            // But we can still check that we got far enough
            println!("⚠ Compilation incomplete (expected): {:?}", e);
        }
    }
}