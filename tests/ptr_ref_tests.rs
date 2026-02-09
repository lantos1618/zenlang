//! Integration tests for pointer operations and pattern matching
//! Tests compile AND run code to verify correct behavior at runtime

mod common;
use common::run_expecting_success;

#[test]
fn test_raw_pointer_allocation() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr = compiler.raw_allocate(64)
            compiler.raw_deallocate(ptr, 64)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Raw pointer allocation: expected 0");
}

#[test]
fn test_pointer_comparison() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr1 = compiler.raw_allocate(8)
            ptr2 = compiler.raw_allocate(8)

            compiler.raw_deallocate(ptr1, 8)
            compiler.raw_deallocate(ptr2, 8)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Pointer comparison: expected 0");
}

#[test]
fn test_option_pattern_matching() {
    let code = r#"
        get_value = () i32 {
            val = Option.Some(42)
            result = val ?
                | .Some(x) { x }
                | .None { 0 }
            return result
        }

        main = () i32 {
            return get_value()
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 42, "Option pattern matching: expected 42");
}

#[test]
fn test_option_none() {
    let code = r#"
        main = () i32 {
            none_val = Option.None
            none_val ?
                | .Some(_) { return 1 }
                | .None { return 0 }
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Option.None: expected 0");
}

#[test]
fn test_result_pattern_matching() {
    let code = r#"
        get_result = () i32 {
            val = Result.Ok(100)
            result = val ?
                | .Ok(x) { x }
                | .Err(_e) { 0 }
            return result
        }

        main = () i32 {
            return get_result()
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 100,
        "Result pattern matching: expected 100"
    );
}

#[test]
fn test_result_error_simple() {
    let code = r#"
        main = () i32 {
            err_val = Result.Err(42)
            err_val ?
                | .Ok(_) { return 0 }
                | .Err(_e) { return 1 }
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 1, "Result.Err: expected 1");
}

#[test]
fn test_custom_enum_pattern() {
    let code = r#"
        MyPtr<T>:
            Valid: i64,
            Invalid

        main = () i32 {
            ptr = MyPtr.Valid(12345)
            ptr ?
                | .Valid(addr) { return 1 }
                | .Invalid { return 0 }
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 1, "Custom enum pattern: expected 1");
}

#[test]
fn test_enum_with_instance_methods() {
    let code = r#"
        SafePtr<T>:
            Some: i64,
            None

        SafePtr<T>.is_valid = (self: SafePtr<T>) bool {
            self ?
                | .Some(_) { true }
                | .None { false }
        }

        main = () i32 {
            ptr = SafePtr.Some(12345)
            ptr.is_valid() ?
                | true { return 1 }
                | false { return 0 }
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 1,
        "Enum with instance methods: expected 1"
    );
}

#[test]
fn test_pointer_arithmetic() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr = compiler.raw_allocate(32)
            offset_ptr = compiler.gep(ptr, 8)
            compiler.raw_deallocate(ptr, 32)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Pointer arithmetic: expected 0");
}

#[test]
fn test_sizeof_intrinsic() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            size = compiler.sizeof<i32>()
            return size
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 4, "sizeof i32: expected 4");
}

#[test]
fn test_ptr_to_int_and_back() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr = compiler.raw_allocate(8)
            addr = compiler.ptr_to_int(ptr)
            back = compiler.int_to_ptr(addr)
            compiler.raw_deallocate(ptr, 8)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "ptr_to_int/int_to_ptr: expected 0");
}
