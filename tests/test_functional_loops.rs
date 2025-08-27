// Tests for the new functional loop syntax
use test_utils::assert_output_contains;

#[test]
fn test_simple_loop_preserved() {
    let code = r#"
        main = () i32 {
            x ::= 5
            loop x > 0 {
                x = x - 1
            }
            return x
        }
    "#;
    
    assert_output_contains!(code, "0");
}

#[test]
fn test_infinite_loop_preserved() {
    let code = r#"
        main = () i32 {
            n ::= 0
            loop {
                n = n + 1
                n >= 3 ? | true => break | false => {}
            }
            return n
        }
    "#;
    
    assert_output_contains!(code, "3");
}

#[test]
fn test_loop_with_label() {
    let code = r#"
        main = () i32 {
            result ::= 0
            outer: loop {
                result = result + 1
                result >= 2 ? | true => break outer | false => {}
            }
            return result
        }
    "#;
    
    assert_output_contains!(code, "2");
}

#[test]
fn test_nested_loops() {
    let code = r#"
        main = () i32 {
            sum ::= 0
            i ::= 0
            outer: loop i < 3 {
                j ::= 0
                inner: loop j < 2 {
                    sum = sum + 1
                    j = j + 1
                }
                i = i + 1
            }
            return sum
        }
    "#;
    
    assert_output_contains!(code, "6");
}

#[test]
fn test_loop_with_continue() {
    let code = r#"
        main = () i32 {
            sum ::= 0
            i ::= 0
            loop i < 5 {
                i = i + 1
                i == 3 ? | true => continue | false => {}
                sum = sum + i
            }
            return sum
        }
    "#;
    
    assert_output_contains!(code, "12"); // 1 + 2 + 4 + 5 = 12
}

// Note: Range and iterator loop tests would require the stdlib to be loaded
// and the functional syntax to be fully integrated with the compiler.
// These tests verify that the basic loop constructs still work correctly.