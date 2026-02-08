//! Integration tests for pointer operations and pattern matching
//! Tests compile AND run code to verify correct behavior at runtime

mod common;
use common::run_expecting_success;

#[test]
#[ignore = "SIGSEGV at runtime — compiler.raw_deallocate causes segfault"]
fn test_raw_pointer_allocation() {
    // Test raw pointer allocation and deallocation using compiler intrinsics
    let code = r#"
        main = () i32 {
            ptr = compiler.raw_allocate(64)
            null_ptr = compiler.null_ptr()
            is_valid = ptr != null_ptr ?
                | true { 1 }
                | false { 0 }
            compiler.raw_deallocate(ptr, 64)
            return is_valid
        }
    "#;

    // Allocation should succeed, so is_valid = 1
    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 1,
        "Raw pointer allocation: expected is_valid=1"
    );
}

#[test]
#[ignore = "SIGSEGV at runtime — compiler.raw_deallocate causes segfault"]
fn test_pointer_comparison() {
    // Test pointer comparison
    let code = r#"
        main = () i32 {
            ptr1 = compiler.raw_allocate(8)
            ptr2 = compiler.raw_allocate(8)
            null_ptr = compiler.null_ptr()

            // Compare pointers
            same = ptr1 == ptr2 ?
                | true { 1 }
                | false { 0 }

            is_null = ptr1 == null_ptr ?
                | true { 1 }
                | false { 0 }

            compiler.raw_deallocate(ptr1, 8)
            compiler.raw_deallocate(ptr2, 8)
            return same + is_null
        }
    "#;

    // Two different allocations should not be same (0), ptr1 should not be null (0)
    // same=0 + is_null=0 = 0
    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Pointer comparison: expected same=0, is_null=0"
    );
}

#[test]
fn test_option_pattern_matching() {
    // Test Option type pattern matching (builtin type)
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

    // Option.Some(42) matched -> returns 42
    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 42, "Option pattern matching: expected 42");
}

#[test]
fn test_option_none() {
    // Test Option.None creation and matching
    let code = r#"
        main = () i32 {
            none_val = Option.None
            none_val ?
                | .Some(_) { return 1 }
                | .None { return 0 }
        }
    "#;

    // Option.None matched -> returns 0
    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Option.None: expected 0");
}

#[test]
fn test_result_pattern_matching() {
    // Test Result type pattern matching (builtin type)
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

    // Result.Ok(100) matched -> returns 100
    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 100,
        "Result pattern matching: expected 100"
    );
}

#[test]
fn test_result_error_simple() {
    // Test Result.Err creation and matching with simple returns
    let code = r#"
        main = () i32 {
            err_val = Result.Err(42)
            err_val ?
                | .Ok(_) { return 0 }
                | .Err(_e) { return 1 }
        }
    "#;

    // Result.Err(42) matched by .Err(_e) -> returns 1
    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 1, "Result.Err: expected 1");
}

#[test]
fn test_custom_enum_pattern() {
    // Test custom enum with pattern matching
    let code = r#"
        // Define a custom enum type
        MyPtr<T>:
            Valid: i64,
            Invalid

        // Test pattern matching on custom enum
        main = () i32 {
            ptr = MyPtr.Valid(12345)
            ptr ?
                | .Valid(addr) { return 1 }
                | .Invalid { return 0 }
        }
    "#;

    // MyPtr.Valid matched -> returns 1
    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 1, "Custom enum pattern: expected 1");
}

#[test]
fn test_enum_with_instance_methods() {
    // Test enum type with instance methods - now works with type tracking fix!
    let code = r#"
        // Define enum type
        SafePtr<T>:
            Some: i64,
            None

        // Instance method - doesn't require static method resolution
        SafePtr<T>.is_valid = (self: SafePtr<T>) bool {
            self ?
                | .Some(_) { true }
                | .None { false }
        }

        main = () i32 {
            // Create using direct enum variant syntax instead of static method
            ptr = SafePtr.Some(12345)
            ptr.is_valid() ?
                | true { return 1 }
                | false { return 0 }
        }
    "#;

    // SafePtr.Some(12345).is_valid() = true -> returns 1
    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 1,
        "Enum with instance methods: expected 1"
    );
}

#[test]
fn test_pointer_arithmetic() {
    // Test pointer offset calculations using compiler intrinsics
    let code = r#"
        main = () i32 {
            // Allocate memory for 4 i32 values
            ptr = compiler.raw_allocate(32)

            // Calculate offset - gep(ptr, byte_offset)
            offset_ptr = compiler.gep(ptr, 8)

            // Clean up
            compiler.raw_deallocate(ptr, 32)
            return 0
        }
    "#;

    // Returns 0
    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Pointer arithmetic: expected 0");
}

#[test]
fn test_sizeof_intrinsic() {
    // Test sizeof intrinsic
    let code = r#"
        main = () i32 {
            size = compiler.sizeof<i32>()
            // sizeof i32 should be 4
            return size
        }
    "#;

    // sizeof(i32) = 4
    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 4, "sizeof i32: expected 4");
}

#[test]
fn test_ptr_to_int_and_back() {
    // Test pointer to integer conversion
    let code = r#"
        main = () i32 {
            ptr = compiler.raw_allocate(8)
            addr = compiler.ptr_to_int(ptr)
            back = compiler.int_to_ptr(addr)

            // Compare original and converted back
            same = ptr == back ?
                | true { 1 }
                | false { 0 }

            compiler.raw_deallocate(ptr, 8)
            return same
        }
    "#;

    // Round-trip should preserve pointer, same=1
    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 1,
        "ptr_to_int/int_to_ptr round-trip: expected 1"
    );
}
