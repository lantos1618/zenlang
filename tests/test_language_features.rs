// Comprehensive integration tests for Zen language features
// Tests all major language constructs working together

mod common;

use common::ExecutionHelper;
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_fibonacci_recursive() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        fib = (n: i32) i32 {
            n <= 1 ? | true => { return n } | false => {}
            return fib(n - 1) + fib(n - 2)
        }
        
        main = () i32 {
            result := fib(10)
            printf("fib(10) = %d\n", result)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("fib(10) = 55");
    output.assert_success();
}

#[test]
fn test_factorial_iterative() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        factorial = (n: i32) i32 {
            result ::= 1
            i ::= 2
            loop i <= n {
                result = result * i
                i = i + 1
            }
            return result
        }
        
        main = () i32 {
            result := factorial(5)
            printf("5! = %d\n", result)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("5! = 120");
    output.assert_success();
}

#[test]
fn test_struct_with_methods() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        extern malloc = (size: i64) *void
        
        Point = {
            x: i32,
            y: i32,
        }
        
        point_new = (x: i32, y: i32) *Point {
            p := malloc(16) as *Point
            p.x = x
            p.y = y
            return p
        }
        
        point_distance_squared = (p: *Point) i32 {
            return p.x * p.x + p.y * p.y
        }
        
        main = () i32 {
            p := point_new(3, 4)
            dist_sq := point_distance_squared(p)
            printf("Point(3,4) distance squared: %d\n", dist_sq)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("Point(3,4) distance squared: 25");
    output.assert_success();
}

#[test]
fn test_array_operations() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        extern malloc = (size: i64) *void
        
        main = () i32 {
            // Allocate array of 5 integers
            arr := malloc(20) as *i32
            
            // Initialize array
            i ::= 0
            loop i < 5 {
                arr[i] = i * i
                i = i + 1
            }
            
            // Sum array elements
            sum ::= 0
            i = 0
            loop i < 5 {
                sum = sum + arr[i]
                i = i + 1
            }
            
            printf("Sum of squares 0-4: %d\n", sum)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("Sum of squares 0-4: 30");
    output.assert_success();
}

#[test]
fn test_string_operations() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        extern strlen = (s: *i8) i64
        
        main = () i32 {
            msg := "Hello, Zen!"
            len := strlen(msg)
            printf("Message: %s\n", msg)
            printf("Length: %ld\n", len)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("Message: Hello, Zen!");
    output.assert_stdout_contains("Length: 11");
    output.assert_success();
}

#[test]
fn test_nested_pattern_matching() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        classify = (x: i32, y: i32) i32 {
            result := x ? 
                | 0 => {
                    y ? | 0 => 0 | _ => 1
                }
                | _ => {
                    y ? | 0 => 2 | _ => 3
                }
            return result
        }
        
        main = () i32 {
            printf("(0,0) -> %d\n", classify(0, 0))
            printf("(0,1) -> %d\n", classify(0, 1))
            printf("(1,0) -> %d\n", classify(1, 0))
            printf("(1,1) -> %d\n", classify(1, 1))
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("(0,0) -> 0");
    output.assert_stdout_contains("(0,1) -> 1");
    output.assert_stdout_contains("(1,0) -> 2");
    output.assert_stdout_contains("(1,1) -> 3");
    output.assert_success();
}

#[test]
fn test_function_pointers() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        add = (a: i32, b: i32) i32 {
            return a + b
        }
        
        multiply = (a: i32, b: i32) i32 {
            return a * b
        }
        
        apply_op = (op: *(i32, i32) i32, x: i32, y: i32) i32 {
            return op(x, y)
        }
        
        main = () i32 {
            sum := apply_op(add, 5, 3)
            product := apply_op(multiply, 5, 3)
            
            printf("5 + 3 = %d\n", sum)
            printf("5 * 3 = %d\n", product)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("5 + 3 = 8");
    output.assert_stdout_contains("5 * 3 = 15");
    output.assert_success();
}

#[test]
fn test_multiple_return_values() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        divmod = (a: i32, b: i32) (i32, i32) {
            quotient := a / b
            remainder := a - (quotient * b)
            return (quotient, remainder)
        }
        
        main = () i32 {
            result := divmod(17, 5)
            printf("17 / 5 = %d remainder %d\n", result.0, result.1)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("17 / 5 = 3 remainder 2");
    output.assert_success();
}

#[test]
fn test_string_interpolation_complex() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        main = () i32 {
            x := 42
            y := 7
            msg := "The answer is $(x) and y is $(y), sum is $(x + y)"
            printf("%s\n", msg)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("The answer is 42 and y is 7, sum is 49");
    output.assert_success();
}

#[test] 
fn test_loop_with_range() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: *i8, ...) i32
        
        main = () i32 {
            sum ::= 0
            loop i in 1..5 {
                sum = sum + i
            }
            printf("Sum of 1..5: %d\n", sum)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse");
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run");
    
    output.assert_stdout_contains("Sum of 1..5: 10");
    output.assert_success();
}