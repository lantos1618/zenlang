mod common;

use common::ExecutionHelper;
use zen::lexer::Lexer;
use zen::parser::Parser;

#[test]
fn test_mutable_loop_variable() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: string, ...) i64
        
        main = () i32 {
            sum :: i64 = 0
            i :: i64 = 0
            ten :: i64 = 10
            one :: i64 = 1
            loop (i < ten) {
                sum = sum + i
                printf("i=%lld, sum=%lld\n", i, sum)
                i = i + one
            }
            printf("Final sum: %lld\n", sum)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Compile and run
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    // Verify the loop executed correctly
    output.assert_stdout_contains("Final sum: 45");
    output.assert_success();
}

#[test]
fn test_mutable_variable_in_loop() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: string, ...) i64
        
        main = () i32 {
            counter :: i64 = 0
            limit :: i64 = 5
            one :: i64 = 1
            loop (counter < limit) {
                printf("Counter: %lld\n", counter)
                counter = counter + one
            }
            printf("Done! Counter = %lld\n", counter)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Compile and run
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    // Verify the loop executed 5 times
    output.assert_stdout_contains("Counter: 0");
    output.assert_stdout_contains("Counter: 4");
    output.assert_stdout_contains("Done! Counter = 5");
    output.assert_success();
}

#[test] 
fn test_nested_loops_with_mutation() {
    let helper = ExecutionHelper::new();
    
    let input = r#"
        extern printf = (format: string, ...) i64
        
        main = () i32 {
            total :: i64 = 0
            one :: i64 = 1
            i :: i64 = 0
            three :: i64 = 3
            loop (i < three) {
                j :: i64 = 0
                loop (j < three) {
                    total = total + one
                    printf("i=%lld, j=%lld, total=%lld\n", i, j, total)
                    j = j + one
                }
                i = i + one
            }
            printf("Total iterations: %lld\n", total)
            return 0
        }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().expect("Failed to parse program");
    
    // Compile and run
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    // Should have 9 iterations total (3x3)
    output.assert_stdout_contains("Total iterations: 9");
    output.assert_success();
}