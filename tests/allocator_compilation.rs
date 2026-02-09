//! Integration tests for allocator interface behavioral verification
//! Tests that stdlib allocator modules compile, run, and produce correct results

mod common;
use common::run_expecting_success;

#[test]
fn test_gpa_allocator_basic() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            alloc = compiler.raw_allocate(100)
            compiler.raw_deallocate(alloc, 100)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "GPA allocator basic test should return 0"
    );
}

#[test]
fn test_allocator_allocate_array() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            size = 10
            total = size * 8
            alloc = compiler.raw_allocate(total)
            compiler.raw_deallocate(alloc, total)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Allocator array test should return 0");
}

#[test]
fn test_allocator_reallocate() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr = compiler.raw_allocate(50)
            new_ptr = compiler.raw_reallocate(ptr, 50, 100)
            compiler.raw_deallocate(new_ptr, 100)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Allocator reallocate test should return 0"
    );
}

#[test]
fn test_allocator_with_null_check() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr = compiler.raw_allocate(100)
            is_null = compiler.is_null(ptr)
            compiler.raw_deallocate(ptr, 100)
            is_null ?
                | true { return 1 }
                | false { return 0 }
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Allocation should return non-null pointer (is_null=0)"
    );
}

#[test]
fn test_gpa_allocate_multiple() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            ptr1 = compiler.raw_allocate(100)
            ptr2 = compiler.raw_allocate(200)
            ptr3 = compiler.raw_allocate(50)

            compiler.raw_deallocate(ptr1, 100)
            compiler.raw_deallocate(ptr2, 200)
            compiler.raw_deallocate(ptr3, 50)

            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "GPA allocate multiple test should return 0"
    );
}

#[test]
fn test_allocator_with_pointer_arithmetic() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            base = compiler.raw_allocate(1000)
            offset = compiler.gep(base, 100)
            another_offset = compiler.gep(offset, 50)
            compiler.raw_deallocate(base, 1000)
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Allocator with pointer arithmetic should return 0"
    );
}

#[test]
fn test_allocator_loop_allocations() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            i:: i32 = 0
            loop {
                cond = i < 10 ?
                    | true { true }
                    | false { break }
                ptr = compiler.raw_allocate(50)
                compiler.raw_deallocate(ptr, 50)
                i = i + 1
            }
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(result.exit_code, 0, "Allocator in loop should return 0");
}

#[test]
fn test_allocator_conditional_allocation() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            size = 100
            ptr = size > 50 ?
                | true { compiler.raw_allocate(size) }
                | false { compiler.null_ptr() }

            ptr != compiler.null_ptr() ?
                | true { compiler.raw_deallocate(ptr, size) }
                | false { }

            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Allocator with conditional allocation should return 0"
    );
}

#[test]
fn test_allocator_overflow_check() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            count = 1000000
            item_size = 8
            total_size = count * item_size

            // Check for overflow
            overflow = total_size < count ?
                | true { 1 }
                | false { 0 }

            ptr = compiler.raw_allocate(total_size)
            compiler.raw_deallocate(ptr, total_size)

            return overflow
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "No overflow expected for 1000000*8, should return 0"
    );
}

#[test]
fn test_allocator_with_type_casting() {
    let code = r#"
        { compiler } = @std
        main = () i32 {
            // Allocate for i32 array
            count = 10
            item_size = 4
            total_size = count * item_size

            ptr = compiler.raw_allocate(total_size)
            typed_ptr = compiler.raw_ptr_cast(ptr)
            compiler.raw_deallocate(ptr, total_size)

            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Allocator with type casting should return 0"
    );
}

#[test]
fn test_allocator_string_usage() {
    // This documents how String would use the allocator
    let code = r#"
        main = () i32 {
            // String internally allocates memory
            s = "hello world"
            // String should deallocate when it goes out of scope
            return 0
        }
    "#;

    let result = run_expecting_success(code);
    assert_eq!(
        result.exit_code, 0,
        "Allocator with string usage should return 0"
    );
}
