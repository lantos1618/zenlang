pub mod behaviors;
pub mod declaration_checking;
pub mod expression_inference;
pub mod function_checking;
pub mod inference;
pub mod intrinsics;
pub mod method_types;
pub mod pattern_binding;
pub mod scope;
pub mod self_resolution;
pub mod statement_checking;
pub mod stdlib_loading;
pub mod type_resolution;
pub mod types;
pub mod validation;

use crate::ast::primitives;
use crate::ast::{AstType, Declaration, Function, Program, Statement};
use crate::error::{Result, Span};
use crate::type_context::TypeContext;
use crate::well_known::WellKnownTypes;
use behaviors::BehaviorResolver;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct VariableInfo {
    pub type_: AstType,
    pub is_mutable: bool,
    pub is_initialized: bool,
}

#[allow(dead_code)]
pub struct TypeChecker {
    scopes: Vec<HashMap<String, VariableInfo>>,
    functions: HashMap<String, FunctionSignature>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    behavior_resolver: BehaviorResolver,
    module_imports: HashMap<String, String>,
    current_impl_type: Option<String>,
    current_span: Option<Span>,
    /// Expected return type for the current function being checked
    current_function_return_type: Option<AstType>,
    pub well_known: WellKnownTypes,
    // Cache of loaded stdlib modules for type lookup
    stdlib_modules: HashMap<String, Program>,
    // Extracted stdlib method signatures: "Type::method" -> signature
    stdlib_methods: HashMap<String, MethodSignature>,
    // Extracted stdlib function signatures: "module::function" -> signature
    stdlib_functions: HashMap<String, FunctionSignature>,
    // Type aliases: "CompletionFn" -> (u64, i64) void
    type_aliases: HashMap<String, AstType>,
    // Collected variable types for TypeContext: "function_name::var_name" -> type
    // Variables are collected during checking because scopes get popped
    collected_variables: HashMap<String, AstType>,
    // Current function name for scoping variables
    current_function_name: Option<String>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct FunctionSignature {
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_external: bool,
}

#[derive(Clone, Debug)]
pub struct MethodSignature {
    pub receiver_type: String,
    pub method_name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_static: bool,
}

#[derive(Clone, Debug)]
pub struct StructInfo {
    pub fields: Vec<(String, AstType)>,
}

#[derive(Clone, Debug)]
pub struct EnumInfo {
    pub variants: Vec<(String, Option<AstType>)>,
}

impl TypeChecker {
    /// Resolve Generic types to Struct types if they're known structs
    /// This handles the case where the parser represents struct types as Generic
    /// Recursively resolves nested Generic types in fields
    /// Uses a visited set to prevent infinite recursion on circular references
    fn resolve_generic_to_struct(&self, ast_type: &AstType) -> AstType {
        type_resolution::resolve_generic_to_struct(self, ast_type)
    }

    /// Get the inferred function signatures
    pub fn get_function_signatures(&self) -> &HashMap<String, FunctionSignature> {
        &self.functions
    }

    /// Look up a UFC method in function signatures by type name and method name.
    ///
    /// Handles generic type names: if `"TypeName.method"` isn't found directly,
    /// also matches `"TypeName<...>.method"` keys (since the parser stores generic
    /// params in function names, e.g., `"SafePtr<T>.is_valid"`).
    pub fn find_ufc_method(&self, type_name: &str, method: &str) -> Option<&FunctionSignature> {
        // Fast path: exact match (e.g., "EnemyArray.get_state")
        let exact_key = format!("{}.{}", type_name, method);
        if let Some(sig) = self.functions.get(&exact_key) {
            return Some(sig);
        }
        // Slow path: generic match (e.g., "SafePtr<T>.is_valid" when type_name is "SafePtr")
        let suffix = format!(".{}", method);
        for (key, sig) in &self.functions {
            if key.ends_with(&suffix) && key.starts_with(type_name) {
                let between = &key[type_name.len()..key.len() - suffix.len()];
                if between.starts_with('<') && between.ends_with('>') {
                    return Some(sig);
                }
            }
        }
        None
    }

    /// Resolve a type alias (e.g., "CompletionFn" -> function type)
    pub fn resolve_type_alias(&self, name: &str) -> Option<AstType> {
        self.type_aliases.get(name).cloned()
    }

    /// Parse type arguments from a generic type string like "HashMap<i32, i32>"
    /// Delegates to the unified parser implementation.
    pub fn parse_generic_type_string(type_str: &str) -> (String, Vec<AstType>) {
        crate::parser::parse_generic_type_string(type_str)
    }

    pub fn new() -> Self {
        let enums = HashMap::new();
        // Option and Result are loaded from stdlib/core/ when imported

        let mut functions = HashMap::new();

        // Register builtin math functions
        functions.insert(
            "min".to_string(),
            FunctionSignature {
                params: vec![
                    ("a".to_string(), AstType::I32),
                    ("b".to_string(), AstType::I32),
                ],
                return_type: AstType::I32,
                is_external: false,
            },
        );
        functions.insert(
            "max".to_string(),
            FunctionSignature {
                params: vec![
                    ("a".to_string(), AstType::I32),
                    ("b".to_string(), AstType::I32),
                ],
                return_type: AstType::I32,
                is_external: false,
            },
        );
        functions.insert(
            "abs".to_string(),
            FunctionSignature {
                params: vec![("x".to_string(), AstType::I32)],
                return_type: AstType::I32,
                is_external: false,
            },
        );

        Self {
            scopes: vec![HashMap::new()],
            functions,
            structs: HashMap::new(),
            enums,
            behavior_resolver: BehaviorResolver::new(),
            module_imports: HashMap::new(),
            current_impl_type: None,
            current_span: None,
            current_function_return_type: None,
            well_known: WellKnownTypes::new(),
            stdlib_modules: HashMap::new(),
            stdlib_methods: HashMap::new(),
            stdlib_functions: HashMap::new(),
            type_aliases: HashMap::new(),
            collected_variables: HashMap::new(),
            current_function_name: None,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<TypeContext> {
        // First pass: collect all type definitions and function signatures
        for declaration in program.declarations.iter() {
            self.collect_declaration_types(declaration)?;
        }

        // Second pass: resolve Generic types to Struct types in struct fields
        // This handles forward references - all structs are now registered
        // We do multiple passes until no more changes occur (to handle nested dependencies)
        let mut changed = true;
        let mut iterations = 0;
        const MAX_RESOLUTION_ITERATIONS: usize = 100;
        while changed && iterations < MAX_RESOLUTION_ITERATIONS {
            changed = false;
            iterations += 1;

            let struct_names: Vec<String> = self.structs.keys().cloned().collect();
            for struct_name in struct_names {
                let resolved_fields: Vec<(String, AstType)> = {
                    let struct_info = match self.structs.get(&struct_name) {
                        Some(info) => info,
                        None => continue, // struct was removed during iteration
                    };
                    struct_info
                        .fields
                        .iter()
                        .map(|(name, field_type)| {
                            let resolved = self.resolve_generic_to_struct(field_type);
                            if &resolved != field_type {
                                changed = true;
                            }
                            (name.clone(), resolved)
                        })
                        .collect()
                };
                if let Some(struct_info) = self.structs.get_mut(&struct_name) {
                    struct_info.fields = resolved_fields;
                }
            }
        }
        if iterations >= MAX_RESOLUTION_ITERATIONS && changed {
            eprintln!("Warning: struct field type resolution did not converge after {} iterations (possible circular dependency)", MAX_RESOLUTION_ITERATIONS);
        }

        // Third pass: infer return types for functions with Void return type
        for declaration in &program.declarations {
            if let Declaration::Function(func) = declaration {
                if func.return_type == AstType::Void && !func.body.is_empty() {
                    if let Ok(inferred_type) = self.infer_function_return_type(func) {
                        if let Some(sig) = self.functions.get_mut(&func.name) {
                            sig.return_type = inferred_type;
                        }
                    }
                }
            }
        }

        // Fourth pass: type check function bodies
        for declaration in &program.declarations {
            self.check_declaration(declaration)?;
        }

        // Build TypeContext from collected type information
        Ok(self.build_type_context())
    }

    /// Check a program, returning partial TypeContext even on error.
    /// Returns (TypeContext, Option<CompileError>).
    /// The TypeContext will contain everything collected before the error occurred.
    pub fn check_program_tolerant(
        &mut self,
        program: &Program,
    ) -> (TypeContext, Option<crate::error::CompileError>) {
        let (ctx, errors) = self.check_program_collect_errors(program);
        (ctx, errors.into_iter().next())
    }

    /// Like check_program_tolerant but collects ALL errors instead of just the first.
    /// Continues past declaration errors, body errors — everything possible.
    pub fn check_program_collect_errors(
        &mut self,
        program: &Program,
    ) -> (TypeContext, Vec<crate::error::CompileError>) {
        let mut errors: Vec<crate::error::CompileError> = Vec::new();

        // First pass: collect all type definitions and function signatures
        // Continue past declaration errors to collect as much as possible
        for declaration in program.declarations.iter() {
            if let Err(e) = self.collect_declaration_types(declaration) {
                errors.push(e);
            }
        }

        // Second pass: resolve Generic types to Struct types in struct fields
        let mut changed = true;
        let mut iterations = 0;
        const MAX_RESOLUTION_ITERATIONS: usize = 100;
        while changed && iterations < MAX_RESOLUTION_ITERATIONS {
            changed = false;
            iterations += 1;

            let struct_names: Vec<String> = self.structs.keys().cloned().collect();
            for struct_name in struct_names {
                let resolved_fields: Vec<(String, AstType)> = {
                    let struct_info = match self.structs.get(&struct_name) {
                        Some(info) => info,
                        None => continue,
                    };
                    struct_info
                        .fields
                        .iter()
                        .map(|(name, field_type)| {
                            let resolved = self.resolve_generic_to_struct(field_type);
                            if &resolved != field_type {
                                changed = true;
                            }
                            (name.clone(), resolved)
                        })
                        .collect()
                };
                if let Some(struct_info) = self.structs.get_mut(&struct_name) {
                    struct_info.fields = resolved_fields;
                }
            }
        }

        // Third pass: infer return types for functions with Void return type
        for declaration in &program.declarations {
            if let Declaration::Function(func) = declaration {
                if func.return_type == AstType::Void && !func.body.is_empty() {
                    if let Ok(inferred_type) = self.infer_function_return_type(func) {
                        if let Some(sig) = self.functions.get_mut(&func.name) {
                            sig.return_type = inferred_type;
                        }
                    }
                }
            }
        }

        // Fourth pass: type check function bodies — collect all errors
        for declaration in &program.declarations {
            if let Err(e) = self.check_declaration(declaration) {
                errors.push(e);
            }
        }

        (self.build_type_context(), errors)
    }

    /// Build TypeContext from typechecker's collected information
    pub fn build_type_context(&self) -> TypeContext {
        let mut ctx = TypeContext::new();

        // Register functions
        for (name, sig) in &self.functions {
            ctx.register_function(
                name.clone(),
                sig.params.clone(),
                sig.return_type.clone(),
                sig.is_external,
            );
        }

        // Register structs
        for (name, info) in &self.structs {
            ctx.register_struct(name.clone(), info.fields.clone());
        }

        // Register enums
        for (name, info) in &self.enums {
            ctx.register_enum(name.clone(), info.variants.clone());
        }

        // Register methods from behavior resolver (inherent methods - impl blocks without trait)
        for (type_name, methods) in &self.behavior_resolver.inherent_methods {
            for method in methods {
                // Convert param_types to named params (using index-based names)
                let params: Vec<(String, AstType)> = method
                    .param_types
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (format!("arg{}", i), t.clone()))
                    .collect();
                ctx.register_method_with_params(
                    type_name,
                    &method.name,
                    params,
                    method.return_type.clone(),
                );

                // Register constructors (methods that return the type itself)
                // Use centralized constructor method definitions
                let is_constructor = primitives::is_constructor_method(&method.name)
                    || method.name.starts_with("with_")
                    || method.name.starts_with("from_");

                if is_constructor {
                    // The return type should resolve to an instance of the type
                    let constructor_return = match &method.return_type {
                        AstType::Generic { name, type_args } if name == type_name => {
                            // Method returns Self or the implementing type
                            AstType::Generic {
                                name: type_name.clone(),
                                type_args: type_args.clone(),
                            }
                        }
                        other => other.clone(),
                    };
                    ctx.register_constructor(type_name, &method.name, constructor_return);
                }
            }
        }

        // Register behavior implementations and their methods
        for ((type_name, behavior_name), impl_info) in self.behavior_resolver.implementations() {
            ctx.register_behavior_impl(type_name, behavior_name);

            // Also register the actual method signatures from the trait implementation
            for (method_name, method_info) in &impl_info.methods {
                let params: Vec<(String, AstType)> = method_info
                    .param_types
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (format!("arg{}", i), t.clone()))
                    .collect();
                ctx.register_method_with_params(
                    type_name,
                    method_name,
                    params,
                    method_info.return_type.clone(),
                );
            }
        }

        // Register type aliases (for function type aliases like CompletionFn)
        for (name, aliased_type) in &self.type_aliases {
            ctx.type_aliases.insert(name.clone(), aliased_type.clone());
        }

        // Register collected variables (scope::var_name -> type)
        for (key, var_type) in &self.collected_variables {
            ctx.variables.insert(key.clone(), var_type.clone());
        }

        // Register module imports (alias -> module_path)
        for (alias, module_path) in &self.module_imports {
            ctx.module_imports
                .insert(alias.clone(), module_path.clone());
        }

        ctx
    }

    fn collect_declaration_types(&mut self, declaration: &Declaration) -> Result<()> {
        declaration_checking::collect_declaration_types(self, declaration)
    }

    fn check_declaration(&mut self, declaration: &Declaration) -> Result<()> {
        declaration_checking::check_declaration(self, declaration)
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<()> {
        statement_checking::check_statement(self, statement)
    }

    pub fn set_current_span(&mut self, span: Option<Span>) {
        self.current_span = span;
    }

    pub fn get_current_span(&self) -> Option<Span> {
        self.current_span.clone()
    }

    /// Set the expected return type for the current function being checked
    pub fn set_function_return_type(&mut self, return_type: Option<AstType>) {
        self.current_function_return_type = return_type;
    }

    /// Get the expected return type for the current function
    pub fn get_function_return_type(&self) -> Option<&AstType> {
        self.current_function_return_type.as_ref()
    }

    // infer_expression_type moved to expression_inference.rs

    fn types_compatible(&self, expected: &AstType, actual: &AstType) -> bool {
        validation::types_compatible(expected, actual)
    }

    /// Resolve a trait/behavior method for a type
    /// Returns the method info if the type implements a trait with this method
    pub fn resolve_trait_method(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<behaviors::MethodInfo> {
        self.behavior_resolver
            .resolve_method(type_name, method_name)
    }

    // Stdlib loading methods moved to stdlib_loading.rs

    fn enter_scope(&mut self) {
        scope::enter_scope(self)
    }

    fn exit_scope(&mut self) {
        scope::exit_scope(self)
    }

    fn declare_variable(&mut self, name: &str, type_: AstType, is_mutable: bool) -> Result<()> {
        scope::declare_variable(self, name, type_, is_mutable, None)
    }

    fn declare_variable_with_init(
        &mut self,
        name: &str,
        type_: AstType,
        is_mutable: bool,
        is_initialized: bool,
    ) -> Result<()> {
        // Collect variable for TypeContext (used by LSP hover, inlay hints, etc.)
        if let Some(func_name) = &self.current_function_name {
            let key = format!("{}::{}", func_name, name);
            self.collected_variables.insert(key, type_.clone());
        }
        scope::declare_variable_with_init(self, name, type_, is_mutable, is_initialized, None)
    }

    fn declare_variable_with_init_and_span(
        &mut self,
        name: &str,
        type_: AstType,
        is_mutable: bool,
        is_initialized: bool,
        span: Option<Span>,
    ) -> Result<()> {
        // Collect variable for TypeContext (used by LSP hover, inlay hints, etc.)
        if let Some(func_name) = &self.current_function_name {
            let key = format!("{}::{}", func_name, name);
            self.collected_variables.insert(key, type_.clone());
        }
        scope::declare_variable_with_init(self, name, type_, is_mutable, is_initialized, span)
    }

    fn mark_variable_initialized(&mut self, name: &str) -> Result<()> {
        scope::mark_variable_initialized(self, name)
    }

    /// Infer the return type of a function from its body
    fn infer_function_return_type(&mut self, func: &Function) -> Result<AstType> {
        // Create a temporary scope for the function
        self.enter_scope();

        // Add function parameters to scope
        for (param_name, param_type) in &func.args {
            self.declare_variable(param_name, param_type.clone(), false)?;
        }

        // Analyze the body to find the return type
        let return_type = if let Some(last_stmt) = func.body.last() {
            match last_stmt {
                Statement::Expression { expr, .. } => {
                    // The last expression is the return value
                    self.infer_expression_type(expr)?
                }
                Statement::Return { expr, .. } => {
                    // Explicit return statement
                    self.infer_expression_type(expr)?
                }
                _ => {
                    // Other statements don't produce a return value
                    AstType::Void
                }
            }
        } else {
            // Empty body returns void
            AstType::Void
        };

        self.exit_scope();
        Ok(return_type)
    }

    fn get_variable_type(&self, name: &str) -> Result<AstType> {
        scope::get_variable_type(self, name, &self.enums)
    }

    fn get_variable_info(&self, name: &str) -> Result<VariableInfo> {
        scope::get_variable_info(self, name)
    }

    // Pattern binding methods moved to pattern_binding.rs

    fn variable_exists(&self, name: &str) -> bool {
        scope::variable_exists(self, name)
    }

    fn variable_exists_in_current_scope(&self, name: &str) -> bool {
        scope::variable_exists_in_current_scope(self, name)
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::AstType;
    use crate::error::CompileError;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::typechecker::TypeChecker;

    /// Helper to create a TypeChecker and parse + check a program
    fn check_program(input: &str) -> Result<TypeChecker, CompileError> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser
            .parse_program()
            .map_err(|e| CompileError::SyntaxError(format!("Parse error: {:?}", e), None))?;
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)?;
        Ok(type_checker)
    }

    // ========================================================================
    // Basic Type Checking Tests
    // ========================================================================

    #[test]
    fn test_basic_type_checking() {
        let input = "main: () void = {
            x = 42
            y : i32 = 100
            z = x + y
        }";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_type_mismatch_error() {
        let input = "main: () void = {
            x : i32 = \"hello\"
        }";
        let result = check_program(input);
        assert!(result.is_err());
        if let Err(CompileError::TypeError(msg, _)) = result {
            assert!(msg.contains("Type mismatch"));
        }
    }

    // ========================================================================
    // Binary Operations Type Inference Tests
    // ========================================================================

    #[test]
    fn test_integer_arithmetic() {
        let input = "main: () void = {
            a: i32 = 10
            b: i32 = 20
            c = a + b
            d = a - b
            e = a * b
            f = a / b
        }";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_integer_promotion() {
        // i32 + i64 should work (promote to i64)
        let input = "main: () void = {
            a: i32 = 10
            b: i64 = 20
            c = a + b
        }";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_float_arithmetic() {
        let input = "main: () void = {
            a: f64 = 1.5
            b: f64 = 2.5
            c = a + b
            d = a * b
        }";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_comparison_operators() {
        let input = "main: () void = {
            a: i32 = 10
            b: i32 = 20
            c = a < b
            d = a > b
            e = a == b
            f = a != b
            g = a <= b
            h = a >= b
        }";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_boolean_operators() {
        let input = "main: () void = {
            a: bool = true
            b: bool = false
            c = a && b
            d = a || b
        }";
        assert!(check_program(input).is_ok());
    }

    // ========================================================================
    // Function Call Type Inference Tests
    // ========================================================================

    #[test]
    fn test_function_return_type() {
        let input = "
            add = (a: i32, b: i32) i32 { return a + b }
            main: () void = {
                result: i32 = add(1, 2)
            }
        ";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_function_wrong_return_type() {
        let input = "
            add = (a: i32, b: i32) i32 { return a + b }
            main: () void = {
                result: string = add(1, 2)
            }
        ";
        let result = check_program(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_void_function() {
        let input = "
            do_nothing = () void { }
            main: () void = {
                do_nothing()
            }
        ";
        assert!(check_program(input).is_ok());
    }

    // ========================================================================
    // Struct Type Inference Tests
    // ========================================================================

    #[test]
    fn test_struct_literal() {
        let input = "
            Point: { x: i32, y: i32 }
            main: () void = {
                p = Point { x: 10, y: 20 }
            }
        ";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_struct_field_access() {
        let input = "
            Point: { x: i32, y: i32 }
            main: () void = {
                p = Point { x: 10, y: 20 }
                a: i32 = p.x
                b: i32 = p.y
            }
        ";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_struct_field_wrong_type() {
        let input = "
            Point: { x: i32, y: i32 }
            main: () void = {
                p = Point { x: 10, y: 20 }
                a: string = p.x
            }
        ";
        let result = check_program(input);
        assert!(result.is_err());
    }

    // ========================================================================
    // Enum Type Inference Tests
    // ========================================================================

    #[test]
    fn test_enum_variant_literal() {
        // Zen enum syntax: Name: Variant1, Variant2, ...
        // Use qualified syntax Status.Active for non-generic enums
        let input = "
            Status:
                Active,
                Inactive,
                Pending

            main = () void {
                s: Status = Status.Active
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_enum_with_payload() {
        // Zen generic enum syntax
        let input = "
            MyOption<T>:
                Some: T,
                None

            main = () void {
                x: MyOption<i32> = .Some(42)
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    // ========================================================================
    // Control Flow Type Inference Tests
    // ========================================================================

    #[test]
    fn test_conditional_same_types() {
        let input = "
            main: () void = {
                x = true
                y = x ?
                    | true { 1 }
                    | false { 2 }
            }
        ";
        assert!(check_program(input).is_ok());
    }

    #[test]
    fn test_loop_type() {
        // Zen uses '::=' for mutable bindings
        let input = "
            main = () void {
                i ::= 0
                loop i < 10 {
                    i = i + 1
                }
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    // ========================================================================
    // Type Inference Helper Tests
    // ========================================================================

    #[test]
    fn test_binary_op_type_promotion() {
        use crate::typechecker::inference::promote_numeric_types;

        // Same types - no promotion
        let result = promote_numeric_types(&AstType::I32, &AstType::I32, None);
        assert_eq!(result.unwrap(), AstType::I32);

        // i32 + i64 -> i64
        let result = promote_numeric_types(&AstType::I32, &AstType::I64, None);
        assert_eq!(result.unwrap(), AstType::I64);

        // f32 + f64 -> f64
        let result = promote_numeric_types(&AstType::F32, &AstType::F64, None);
        assert_eq!(result.unwrap(), AstType::F64);

        // int + float -> float
        let result = promote_numeric_types(&AstType::I32, &AstType::F64, None);
        assert_eq!(result.unwrap(), AstType::F64);
    }

    #[test]
    fn test_types_comparable() {
        use crate::typechecker::inference::types_comparable;

        // Same types are comparable
        assert!(types_comparable(&AstType::I32, &AstType::I32));
        assert!(types_comparable(&AstType::Bool, &AstType::Bool));

        // Numeric types are comparable
        assert!(types_comparable(&AstType::I32, &AstType::I64));
        assert!(types_comparable(&AstType::F32, &AstType::F64));

        // Different categories are not comparable
        assert!(!types_comparable(&AstType::I32, &AstType::Bool));
        assert!(!types_comparable(&AstType::I32, &AstType::StaticString));
    }

    // ========================================================================
    // Generic Type Inference Tests
    // ========================================================================

    #[test]
    fn test_generic_struct() {
        // Generic struct with inferred type
        let input = "
            Container<T>: { value: T }
            main = () void {
                c = Container<i32> { value: 42 }
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_generic_function() {
        let input = "
            identity<T> = (x: T) T { return x }
            main = () void {
                a = identity<i32>(42)
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_type_mismatch() {
        // Function returns i32 but declares string return type
        let input = "
            bad_return = () string {
                return 42
            }
        ";
        let result = check_program(input);
        assert!(result.is_err());
        if let Err(CompileError::TypeError(msg, _)) = result {
            assert!(msg.contains("Return type mismatch"));
        }
    }

    #[test]
    fn test_return_type_correct() {
        // Function returns correct type
        let input = "
            good_return = () i32 {
                return 42
            }
        ";
        assert!(check_program(input).is_ok());
    }

    // ========================================================================
    // Nested Struct Tests
    // ========================================================================

    #[test]
    fn test_nested_struct_access() {
        let input = "
            Inner: { value: i32 }
            Outer: { inner: Inner }
            main = () void {
                o = Outer { inner: Inner { value: 42 } }
                x: i32 = o.inner.value
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_struct_wrong_type() {
        let input = "
            Inner: { value: i32 }
            Outer: { inner: Inner }
            main = () void {
                o = Outer { inner: Inner { value: 42 } }
                x: string = o.inner.value
            }
        ";
        assert!(check_program(input).is_err());
    }

    // ========================================================================
    // Unsigned Integer Tests
    // ========================================================================

    #[test]
    fn test_unsigned_integer_cast() {
        // Unsigned integers need explicit casting: cast(value, type)
        let input = "
            main = () void {
                a: u32 = cast(255, u32)
                b: u64 = cast(1000000, u64)
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_signed_unsigned_comparison() {
        // Comparing signed and unsigned should work
        let input = "
            main = () void {
                a: i32 = 10
                b: u32 = 20
                c = a < b
            }
        ";
        let result = check_program(input);
        // This may or may not be an error depending on language semantics
        // For now, just verify it doesn't crash
        let _ = result;
    }

    // ========================================================================
    // Array Tests
    // ========================================================================

    #[test]
    fn test_array_literal() {
        let input = "
            main = () void {
                arr = [1, 2, 3, 4, 5]
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_array_index() {
        let input = "
            main = () void {
                arr = [10, 20, 30]
                x = arr[0]
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    // ========================================================================
    // Multiple Function Tests
    // ========================================================================

    #[test]
    fn test_function_calling_function() {
        let input = "
            double = (x: i32) i32 { return x * 2 }
            quadruple = (x: i32) i32 { return double(double(x)) }
            main = () void {
                result = quadruple(5)
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_recursive_function() {
        let input = "
            factorial = (n: i32) i32 {
                n <= 1 ?
                    | true { return 1 }
                    | false { return n * factorial(n - 1) }
            }
            main = () void {
                result = factorial(5)
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    // ========================================================================
    // Variable Shadowing Tests
    // ========================================================================

    #[test]
    fn test_local_variable_shadows_outer() {
        // Local variable should be allowed to shadow outer scope variable
        let input = "
            ptr = 42
            get_ptr = () i32 {
                ptr = 100
                return ptr
            }
            main = () void {
                result = get_ptr()
            }
        ";
        let result = check_program(input);
        if let Err(ref e) = result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_no_reassign_immutable_in_same_scope() {
        // Should NOT allow reassigning immutable variable in same scope
        let input = "
            main = () void {
                x = 10
                x = 20
            }
        ";
        let result = check_program(input);
        assert!(result.is_err());
    }
}
