use zen::lexer::Lexer;
use zen::lexer::Token;

#[test]
fn test_lexer_identifier_integer_eof() {
    let input = "foo 123";
    let mut lexer = Lexer::new(input);
    let token1 = lexer.next_token();
    assert_eq!(token1, Token::Identifier("foo".to_string()));
    let token2 = lexer.next_token();
    assert_eq!(token2, Token::Integer("123".to_string()));
    let token3 = lexer.next_token();
    assert_eq!(token3, Token::Eof);
}

#[test]
fn test_lexer_keyword_and_symbol() {
    let input = "loop { }";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Keyword("loop".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('{'));
    assert_eq!(lexer.next_token(), Token::Symbol('}'));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_operator() {
    let input = "+ - * / == !=";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Operator("+".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("-".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("*".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("/".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("==".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("!=".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_string_literal() {
    let input = "\"hello world\"";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::StringLiteral("hello world".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_float_numbers() {
    let input = "3.14 2.0 0.5";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Float("3.14".to_string()));
    assert_eq!(lexer.next_token(), Token::Float("2.0".to_string()));
    assert_eq!(lexer.next_token(), Token::Float("0.5".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_more_keywords() {
    let input = "loop in comptime async await behavior impl";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Keyword("loop".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("in".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("comptime".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("async".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("await".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("behavior".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("impl".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_complex_expression() {
    let input = "result := (x + y) * 2;";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Identifier("result".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator(":=".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('('));
    assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("+".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("y".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol(')'));
    assert_eq!(lexer.next_token(), Token::Operator("*".to_string()));
    assert_eq!(lexer.next_token(), Token::Integer("2".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol(';'));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_zen_imports() {
    let input = "@std.core @std.build build.import";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Identifier("@std".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('.'));
    assert_eq!(lexer.next_token(), Token::Identifier("core".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("@std".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('.'));
    assert_eq!(lexer.next_token(), Token::Identifier("build".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("build".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('.'));
    assert_eq!(lexer.next_token(), Token::Identifier("import".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_conditional_arrow() {
    let input = "score -> s { | s >= 90 => \"A\" }";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Identifier("score".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("->".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("s".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('{'));
    assert_eq!(lexer.next_token(), Token::Symbol('|'));
    assert_eq!(lexer.next_token(), Token::Identifier("s".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator(">=".to_string()));
    assert_eq!(lexer.next_token(), Token::Integer("90".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("=>".to_string()));
    assert_eq!(lexer.next_token(), Token::StringLiteral("A".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('}'));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_zen_variable_declarations() {
    let input = "x := 42 y ::= 10 z: i32 = 5 w:: u64 = 100";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Identifier("x".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator(":=".to_string()));
    assert_eq!(lexer.next_token(), Token::Integer("42".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("y".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("::=".to_string()));
    assert_eq!(lexer.next_token(), Token::Integer("10".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("z".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol(':'));
    assert_eq!(lexer.next_token(), Token::Identifier("i32".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("=".to_string()));
    assert_eq!(lexer.next_token(), Token::Integer("5".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("w".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("::".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("u64".to_string()));
    assert_eq!(lexer.next_token(), Token::Operator("=".to_string()));
    assert_eq!(lexer.next_token(), Token::Integer("100".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_string_interpolation() {
    let input = "\"Hello $(name), your score is $(score)\"";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::StringLiteral("Hello $(name), your score is $(score)".to_string()));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_loop_with_in() {
    let input = "loop name in names { io.print(name) }";
    let mut lexer = Lexer::new(input);
    assert_eq!(lexer.next_token(), Token::Keyword("loop".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("name".to_string()));
    assert_eq!(lexer.next_token(), Token::Keyword("in".to_string()));
    assert_eq!(lexer.next_token(), Token::Identifier("names".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('{'));
    assert_eq!(lexer.next_token(), Token::Identifier("io".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('.'));
    assert_eq!(lexer.next_token(), Token::Identifier("print".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol('('));
    assert_eq!(lexer.next_token(), Token::Identifier("name".to_string()));
    assert_eq!(lexer.next_token(), Token::Symbol(')'));
    assert_eq!(lexer.next_token(), Token::Symbol('}'));
    assert_eq!(lexer.next_token(), Token::Eof);
}

#[test]
fn test_lexer_debug_variable_declarations() {
    let input = "x := 42 y ::= 10 z: i32 = 5 w:: u64 = 100";
    let mut lexer = Lexer::new(input);
    
    println!("Debugging token sequence for: '{}'", input);
    let mut token_count = 0;
    loop {
        let token = lexer.next_token();
        println!("Token {}: {:?}", token_count, token);
        token_count += 1;
        if token == Token::Eof {
            break;
        }
    }
}

#[test]
fn test_lexer_debug_simple() {
    let input = "w:: u64";
    let mut lexer = Lexer::new(input);
    
    println!("Debugging simple sequence: '{}'", input);
    let mut token_count = 0;
    loop {
        let token = lexer.next_token();
        println!("Token {}: {:?}", token_count, token);
        token_count += 1;
        if token == Token::Eof {
            break;
        }
    }
}

#[test]
fn test_lexer_debug_input_chars() {
    let input = "x := 42 y ::= 10 z: i32 = 5 w:: u64 = 100";
    println!("Input string: '{}'", input);
    println!("Input bytes: {:?}", input.as_bytes());
    println!("Input chars: {:?}", input.chars().collect::<Vec<_>>());
    
    // Find the position of "w:: u64"
    if let Some(pos) = input.find("w:: u64") {
        println!("Found 'w:: u64' at position {}", pos);
        let before = &input[..pos];
        let after = &input[pos + "w:: u64".len()..];
        println!("Before: '{}'", before);
        println!("After: '{}'", after);
        println!("After bytes: {:?}", after.as_bytes());
    }
}

#[test]
fn test_lexer_debug_step_by_step() {
    let input = "w:: u64";
    let mut lexer = Lexer::new(input);
    
    println!("Step-by-step debugging for: '{}'", input);
    println!("Initial position: {}, read_position: {}", lexer.position, lexer.read_position);
    println!("Initial current_char: {:?}", lexer.current_char);
    
    // Step 1: Get 'w'
    let token1 = lexer.next_token();
    println!("Token 1: {:?}", token1);
    println!("After token 1 - position: {}, read_position: {}, current_char: {:?}", 
             lexer.position, lexer.read_position, lexer.current_char);
    
    // Step 2: Get '::'
    let token2 = lexer.next_token();
    println!("Token 2: {:?}", token2);
    println!("After token 2 - position: {}, read_position: {}, current_char: {:?}", 
             lexer.position, lexer.read_position, lexer.current_char);
    
    // Step 3: Get 'u64'
    let token3 = lexer.next_token();
    println!("Token 3: {:?}", token3);
    println!("After token 3 - position: {}, read_position: {}, current_char: {:?}", 
             lexer.position, lexer.read_position, lexer.current_char);
} 