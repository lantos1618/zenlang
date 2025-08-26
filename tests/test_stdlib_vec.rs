use zen::ast::{Declaration, AstType};
use zen::lexer::Lexer;
use zen::parser::Parser;
use zen::compiler::Compiler;
use inkwell::context::Context;

mod test_output_verification;
use test_output_verification::{ExecutionHelper, CapturedOutput};

#[test]
fn test_vec_implementation_parses() {
    let vec_code = r#"
// Test Vec struct and basic operations
comptime {
    core := @std.core
}

Vec<T> = {
    data: *T,
    len: i64,
    capacity: i64,
}

vec_new<T> = () Vec<T> {
    return Vec<T> {
        data: 0,
        len: 0,
        capacity: 0,
    }
}

main = () i32 {
    v := vec_new<i32>()
    return 0
}
"#;

    let lexer = Lexer::new(vec_code);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Vec code should parse: {:?}", result.err());
    
    let program = result.unwrap();
    
    // Verify we have the Vec struct
    let has_vec_struct = program.declarations.iter().any(|decl| {
        if let Declaration::Struct(s) = decl {
            s.name == "Vec"
        } else {
            false
        }
    });
    assert!(has_vec_struct, "Should have Vec struct declaration");
    
    // Verify vec_new function
    let has_vec_new = program.declarations.iter().any(|decl| {
        if let Declaration::Function(f) = decl {
            f.name == "vec_new"
        } else {
            false
        }
    });
    assert!(has_vec_new, "Should have vec_new function");
}

#[test]
fn test_vec_push_pop_operations() {
    let vec_ops = r#"
comptime {
    core := @std.core
}

Vec<T> = {
    data: *T,
    len: i64,
    capacity: i64,
}

vec_push<T> = (vec: *Vec<T>, value: T) void {
    // Simplified push for testing
    vec.len = vec.len + 1
}

vec_pop<T> = (vec: *Vec<T>) T {
    vec.len = vec.len - 1
    return vec.data[vec.len]
}

main = () i32 {
    return 0
}
"#;

    let lexer = Lexer::new(vec_ops);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Vec operations should parse: {:?}", result.err());
    
    let program = result.unwrap();
    
    // Check that push and pop functions exist
    let functions: Vec<&str> = program.declarations.iter()
        .filter_map(|decl| {
            if let Declaration::Function(f) = decl {
                Some(f.name.as_str())
            } else {
                None
            }
        })
        .collect();
    
    assert!(functions.contains(&"vec_push"), "Should have vec_push function");
    assert!(functions.contains(&"vec_pop"), "Should have vec_pop function");
}

#[test]
fn test_vec_memory_operations() {
    let mem_ops = r#"
comptime {
    core := @std.core
}

extern malloc = (size: i64) *void
extern free = (ptr: *void) void
extern memcpy = (dest: *void, src: *void, size: i64) void

Vec<T> = {
    data: *T,
    len: i64,  
    capacity: i64,
}

vec_grow<T> = (vec: *Vec<T>, new_capacity: i64) void {
    new_data := malloc(new_capacity * 8)  // Simplified sizeof
    
    vec.len > 0 ? | true => {
        memcpy(new_data, vec.data, vec.len * 8)
    } | false => {}
    
    vec.data != 0 ? | true => {
        free(vec.data)
    } | false => {}
    
    vec.data = new_data
    vec.capacity = new_capacity
}

main = () i32 {
    return 0
}
"#;

    let lexer = Lexer::new(mem_ops);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Memory operations should parse: {:?}", result.err());
}

#[test]
fn test_vec_with_pattern_matching() {
    let pattern_code = r#"
comptime {
    core := @std.core
}

Option<T> = 
    | Some(value: T)
    | None

Vec<T> = {
    data: *T,
    len: i64,
    capacity: i64,
}

vec_get<T> = (vec: *Vec<T>, index: i64) Option<T> {
    index >= 0 && index < vec.len ? 
        | true => return Option::Some(vec.data[index])
        | false => return Option::None
}

main = () i32 {
    return 0
}
"#;

    let lexer = Lexer::new(pattern_code);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Pattern matching with Vec should parse: {:?}", result.err());
}

#[test]
fn test_vec_iteration() {
    let iter_code = r#"
Vec<T> = {
    data: *T,
    len: i64,
    capacity: i64,
}

vec_iter<T> = (vec: *Vec<T>) void {
    i ::= 0
    loop i < vec.len {
        // Process vec.data[i]
        i = i + 1
    }
}

main = () i32 {
    return 0
}
"#;

    let lexer = Lexer::new(iter_code);
    let mut parser = Parser::new(lexer);
    let result = parser.parse_program();
    assert!(result.is_ok(), "Vec iteration should parse: {:?}", result.err());
}

#[test]
fn test_vec_compilation_basic() {
    use zen::ast::*;
    
    let helper = ExecutionHelper::new();
    
    // Create a simple struct and test
    let struct_decl = Declaration::Struct(StructDefinition {
        name: "SimpleVec".to_string(),
        type_params: vec![],
        fields: vec![
            StructField {
                name: "len".to_string(),
                type_: AstType::I32,
                is_mutable: false,
                default_value: None,
            },
        ],
        methods: vec![],
    });
    
    let printf_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "printf".to_string(),
        args: vec![AstType::Pointer(Box::new(AstType::I8))],
        return_type: AstType::I32,
        is_varargs: true,
    });
    
    let main_func = Declaration::Function(Function {
        type_params: vec![],
        is_async: false,
        name: "main".to_string(),
        args: vec![],
        return_type: AstType::I32,
        body: vec![
            Statement::VariableDeclaration {
                name: "v".to_string(),
                type_: Some(AstType::Struct {
                    name: "SimpleVec".to_string(),
                    fields: vec![("len".to_string(), AstType::I32)],
                }),
                initializer: Some(Expression::StructLiteral {
                    name: "SimpleVec".to_string(),
                    fields: vec![("len".to_string(), Expression::Integer32(0))],
                }),
                is_mutable: false,
                declaration_type: VariableDeclarationType::InferredImmutable,
            },
            Statement::Expression(Expression::FunctionCall {
                name: "printf".to_string(),
                args: vec![
                    Expression::String("Vec len: %d\n".to_string()),
                    Expression::StructField {
                        struct_: Box::new(Expression::Identifier("v".to_string())),
                        field: "len".to_string(),
                    },
                ],
            }),
            Statement::Return(Expression::Integer32(0)),
        ],
    });
    
    let program = Program {
        declarations: vec![struct_decl, printf_decl, main_func],
    };
    
    let output = helper.compile_ast_and_run(&program)
        .expect("Vec compilation and execution should succeed");
    
    // Verify the output contains the expected text
    output.assert_stdout_contains("Vec len: 0");
    output.assert_exit_code(0);
    
    println!("✓ Vec basic compilation and output verified!");
}