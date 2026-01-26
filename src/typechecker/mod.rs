pub mod behaviors;
pub mod declaration_checking;
pub mod function_checking;
pub mod inference;
pub mod intrinsics;
pub mod method_types;
pub mod scope;
pub mod self_resolution;
pub mod statement_checking;
pub mod type_resolution;
pub mod types;
pub mod validation;

use crate::ast::{AstType, Declaration, Expression, Function, Program, Statement};
use crate::error::{CompileError, Result, Span};
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
        while changed && iterations < 10 {
            changed = false;
            iterations += 1;

            let struct_names: Vec<String> = self.structs.keys().cloned().collect();
            for struct_name in struct_names {
                let resolved_fields: Vec<(String, AstType)> = {
                    let struct_info = self.structs.get(&struct_name).unwrap();
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

        // Fourth pass: type check function bodies
        for declaration in &program.declarations {
            self.check_declaration(declaration)?;
        }

        // Build TypeContext from collected type information
        Ok(self.build_type_context())
    }

    /// Build TypeContext from typechecker's collected information
    fn build_type_context(&self) -> TypeContext {
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
                // Common patterns: new, create, default, with_capacity, etc.
                let is_constructor = method.name == "new"
                    || method.name == "create"
                    || method.name == "default"
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

        // Register behavior implementations
        for (type_name, behavior_name) in self.behavior_resolver.implementations().keys() {
            ctx.register_behavior_impl(type_name, behavior_name);
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

    pub fn infer_expression_type(&mut self, expr: &Expression) -> Result<AstType> {
        match expr {
            Expression::Integer32(_) => Ok(AstType::I32),
            Expression::Integer64(_) => Ok(AstType::I64),
            Expression::Float32(_) => Ok(AstType::F32),
            Expression::Float64(_) => Ok(AstType::F64),
            Expression::Boolean(_) => Ok(AstType::Bool),
            Expression::Unit => Ok(AstType::Void),
            Expression::String(_) => Ok(AstType::StaticString), // String literals are static strings
            Expression::Identifier(name) => inference::infer_identifier_type(self, name),
            Expression::BinaryOp { left, op, right } => {
                inference::infer_binary_op_type(self, left, op, right)
            }
            Expression::FunctionCall {
                name,
                type_args,
                args,
            } => inference::infer_function_call_type(self, name, type_args, args),
            Expression::MemberAccess { object, member } => {
                // Check if accessing @std namespace
                if let Expression::Identifier(name) = &**object {
                    if name == "@std" {
                        // Resolve @std.module access
                        return Ok(AstType::Generic {
                            name: format!("StdModule::{}", member),
                            type_args: vec![],
                        });
                    }
                }
                let object_type = self.infer_expression_type(object)?;
                inference::infer_member_type(
                    &object_type,
                    member,
                    &self.structs,
                    &self.enums,
                    self.get_current_span(),
                )
            }
            Expression::Comptime(inner) => self.infer_expression_type(inner),
            Expression::Range { .. } => Ok(AstType::Range {
                start_type: Box::new(AstType::I32),
                end_type: Box::new(AstType::I32),
                inclusive: false,
            }),
            Expression::StructLiteral { name, .. } => {
                // For struct literals, return the struct type
                // Check if it's a known struct
                if let Some(struct_def) = self.structs.get(name) {
                    Ok(AstType::Struct {
                        name: name.clone(),
                        fields: struct_def.fields.clone(),
                    })
                } else {
                    // Check if it's a stdlib struct
                    if let Some(struct_info) = self.get_stdlib_struct(name) {
                        Ok(AstType::Struct {
                            name: name.clone(),
                            fields: struct_info.fields.clone(),
                        })
                    } else {
                        // It might be a generic struct that will be monomorphized
                        // For now, return a struct type with empty fields
                        Ok(AstType::Struct {
                            name: name.clone(),
                            fields: vec![],
                        })
                    }
                }
            }
            Expression::StdReference => {
                // Return a type representing @std
                Ok(AstType::Generic {
                    name: "Std".to_string(),
                    type_args: vec![],
                })
            }
            Expression::BuiltinReference => {
                // Return a type representing @builtin (raw compiler intrinsics)
                Ok(AstType::Generic {
                    name: "Builtin".to_string(),
                    type_args: vec![],
                })
            }
            Expression::ThisReference => {
                // Return a type representing @this
                Ok(AstType::Generic {
                    name: "This".to_string(),
                    type_args: vec![],
                })
            }
            Expression::StringInterpolation { .. } => {
                // String interpolation returns dynamic String (requires allocator)
                Ok(crate::ast::resolve_string_struct_type())
            }
            Expression::Closure {
                params,
                return_type,
                body,
            } => inference::infer_closure_type(self, params, return_type, body),
            Expression::ArrayIndex { array, .. } => {
                // Array indexing returns the element type
                let array_type = self.infer_expression_type(array)?;
                if let Some(elem_type) = array_type.ptr_inner() {
                    return Ok(elem_type.clone());
                }
                match array_type {
                    AstType::Slice(elem_type) => Ok(*elem_type),
                    AstType::FixedArray { element_type, .. } => Ok(*element_type),
                    _ => Err(CompileError::TypeError(
                        format!("Cannot index type {:?}", array_type),
                        None,
                    )),
                }
            }
            Expression::AddressOf(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                Ok(AstType::ptr(inner_type))
            }
            Expression::Dereference(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                if let Some(elem_type) = inner_type.ptr_inner() {
                    return Ok(elem_type.clone());
                }
                Err(CompileError::TypeError(
                    format!("Cannot dereference non-pointer type {:?}", inner_type),
                    None,
                ))
            }
            Expression::PointerOffset { pointer, .. } => {
                // Pointer offset returns the same pointer type
                self.infer_expression_type(pointer)
            }
            Expression::StructField { struct_, field } => {
                let struct_type = self.infer_expression_type(struct_)?;
                inference::infer_struct_field_type(
                    &struct_type,
                    field,
                    &self.structs,
                    &self.enums,
                    self.get_current_span(),
                )
            }
            Expression::Integer8(_) => Ok(AstType::I8),
            Expression::Integer16(_) => Ok(AstType::I16),
            Expression::Unsigned8(_) => Ok(AstType::U8),
            Expression::Unsigned16(_) => Ok(AstType::U16),
            Expression::Unsigned32(_) => Ok(AstType::U32),
            Expression::Unsigned64(_) => Ok(AstType::U64),
            Expression::ArrayLiteral(elements) => {
                // Infer type from first element - array literals produce slices
                if elements.is_empty() {
                    Ok(AstType::Slice(Box::new(AstType::Void)))
                } else {
                    let elem_type = self.infer_expression_type(&elements[0])?;
                    Ok(AstType::Slice(Box::new(elem_type)))
                }
            }
            Expression::TypeCast { target_type, .. } => Ok(target_type.clone()),
            Expression::QuestionMatch { scrutinee, arms } => {
                // QuestionMatch expression type is determined by the arms
                // All arms should have the same type

                // Infer the type of the scrutinee to properly type pattern bindings
                let scrutinee_type = self.infer_expression_type(scrutinee)?;

                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        // Enter a new scope for the pattern bindings
                        self.enter_scope();

                        // Extract pattern bindings and add them to the scope
                        // Pass the scrutinee type for proper typing
                        self.add_pattern_bindings_to_scope_with_type(
                            &arm.pattern,
                            &scrutinee_type,
                        )?;

                        // Special handling for blocks with early returns
                        // If the arm body is a block, we need to check if it actually
                        // produces a value or just has side effects before returning
                        let arm_type = if let Expression::Block(stmts) = &arm.body {
                            // Check if the block has any non-return statements before the return
                            let mut block_type = AstType::Void;
                            let has_early_return = false;

                            for (j, stmt) in stmts.iter().enumerate() {
                                match stmt {
                                    Statement::Return { .. } => {
                                        // Don't use return statement to determine block type
                                        break;
                                    }
                                    Statement::Expression { expr, .. } => {
                                        // If this is the last statement and there's no early return after it
                                        if j == stmts.len() - 1 && !has_early_return {
                                            block_type = self.infer_expression_type(expr)?;
                                        } else {
                                            // Still type-check intermediate expressions
                                            let _ = self.infer_expression_type(expr)?;
                                        }
                                    }
                                    _ => {
                                        self.check_statement(stmt)?;
                                    }
                                }
                            }
                            block_type
                        } else {
                            self.infer_expression_type(&arm.body)?
                        };

                        // The first non-void arm determines the type, or use first arm if all void
                        if i == 0
                            || (matches!(result_type, AstType::Void)
                                && !matches!(arm_type, AstType::Void))
                        {
                            result_type = arm_type;
                        }

                        // Exit the scope to remove the bindings
                        self.exit_scope();
                    }

                    Ok(result_type)
                }
            }
            Expression::PatternMatch { arms, .. } => {
                // Pattern match expression type is determined by the first arm
                // All arms should have the same type
                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        // Enter a new scope for the pattern bindings
                        self.enter_scope();

                        // Extract pattern bindings and add them to the scope
                        self.add_pattern_bindings_to_scope(&arm.pattern)?;

                        // Infer the type with bindings in scope
                        let arm_type = self.infer_expression_type(&arm.body)?;

                        // The first arm determines the type
                        if i == 0 {
                            result_type = arm_type;
                        }

                        // Exit the scope to remove the bindings
                        self.exit_scope();
                    }

                    Ok(result_type)
                }
            }
            Expression::Block(statements) => {
                // Enter a new scope for the block
                self.enter_scope();

                let mut block_type = AstType::Void;

                // Process all statements in the block
                for (i, stmt) in statements.iter().enumerate() {
                    match stmt {
                        Statement::Expression { expr, .. } => {
                            // The last expression determines the block's type
                            if i == statements.len() - 1 {
                                block_type = self.infer_expression_type(expr)?;
                            } else {
                                // Still type-check intermediate expressions
                                self.infer_expression_type(expr)?;
                            }
                        }
                        _ => {
                            // Process other statements (declarations, assignments, etc.)
                            self.check_statement(stmt)?;
                        }
                    }
                }

                // Exit the block's scope
                self.exit_scope();

                Ok(block_type)
            }
            Expression::Return(expr) => self.infer_expression_type(expr),
            Expression::EnumVariant {
                enum_name,
                variant,
                payload,
            } => inference::infer_enum_variant_type(self, enum_name, variant, payload),
            Expression::StringLength(_) => Ok(AstType::I64),
            Expression::MethodCall {
                object,
                method,
                type_args,
                args: _,
            } => inference::infer_method_call_type(self, object, method, type_args),
            Expression::Loop { body: _ } => {
                // Loop expressions return void for now
                Ok(AstType::Void)
            }
            Expression::Raise(expr) => inference::infer_raise_type(self, expr),
            Expression::Break { .. } | Expression::Continue { .. } => {
                // Break and continue don't return a value, they transfer control
                // For type checking purposes, they can be considered to return void
                Ok(AstType::Void)
            }
            Expression::EnumLiteral { variant, payload } => {
                inference::infer_enum_literal_type(self, variant, payload)
            }
            Expression::Conditional { scrutinee, arms } => {
                // Infer the type of the scrutinee to properly type pattern bindings
                let scrutinee_type = self.infer_expression_type(scrutinee)?;

                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        self.enter_scope();

                        // Extract pattern bindings and add them to the scope
                        self.add_pattern_bindings_to_scope_with_type(
                            &arm.pattern,
                            &scrutinee_type,
                        )?;

                        let arm_type = self.infer_expression_type(&arm.body)?;

                        // The first arm determines the type
                        if i == 0 {
                            result_type = arm_type;
                        }

                        self.exit_scope();
                    }

                    Ok(result_type)
                }
            }
            // Zen spec pointer operations
            Expression::PointerDereference(expr) => {
                // ptr.val -> T (if ptr is Ptr<T>, MutPtr<T>, or RawPtr<T>)
                let ptr_type = self.infer_expression_type(expr)?;
                if let Some(inner) = ptr_type.ptr_inner() {
                    Ok(inner.clone())
                } else {
                    Err(CompileError::TypeError(
                        format!("Cannot dereference non-pointer type: {:?}", ptr_type),
                        None,
                    ))
                }
            }
            Expression::PointerAddress(expr) => {
                // expr.addr -> RawPtr<T> (if expr is of type T)
                let expr_type = self.infer_expression_type(expr)?;
                Ok(AstType::raw_ptr(expr_type))
            }
            Expression::CreateReference(expr) => {
                // expr.ref() -> Ptr<T> (if expr is of type T)
                let expr_type = self.infer_expression_type(expr)?;
                Ok(AstType::ptr(expr_type))
            }
            Expression::CreateMutableReference(expr) => {
                // expr.mut_ref() -> MutPtr<T> (if expr is of type T)
                let expr_type = self.infer_expression_type(expr)?;
                Ok(AstType::mut_ptr(expr_type))
            }
            Expression::VecConstructor {
                element_type,
                size: _,
                initial_values: _,
            } => {
                // Vec<T>() -> Generic { name: "Vec", type_args: [T] }
                Ok(AstType::Generic {
                    name: "Vec".to_string(),
                    type_args: vec![element_type.clone()],
                })
            }
            Expression::DynVecConstructor {
                element_types,
                allocator: _,
                initial_capacity: _,
            } => {
                // DynVec<T>() -> Generic { name: "DynVec", type_args: [T, ...] }
                Ok(AstType::Generic {
                    name: "DynVec".to_string(),
                    type_args: element_types.clone(),
                })
            }
            Expression::ArrayConstructor { element_type } => {
                // Array<T>() -> Generic { name: "Array", type_args: [T] }
                // This matches the expected type format for generic types
                Ok(AstType::Generic {
                    name: "Array".to_string(),
                    type_args: vec![element_type.clone()],
                })
            }
            Expression::Some(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                Ok(AstType::Generic {
                    name: self
                        .well_known
                        .get_variant_parent_name(self.well_known.some_name())
                        .unwrap()
                        .to_string(),
                    type_args: vec![inner_type],
                })
            }
            Expression::None => Ok(AstType::Generic {
                name: self
                    .well_known
                    .get_variant_parent_name(self.well_known.none_name())
                    .unwrap()
                    .to_string(),
                type_args: vec![AstType::Void],
            }),
            Expression::CollectionLoop { .. } => {
                // collection.loop() returns unit/void
                Ok(AstType::Void)
            }
            Expression::Defer(_) => {
                // @this.defer() returns unit/void
                Ok(AstType::Void)
            }
        }
    }

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

    fn register_stdlib_module(&mut self, _alias: &str, _module_path: &str) -> Result<()> {
        // Stdlib modules are now loaded via with_stdlib_modules() from ModuleSystem
        // Types are extracted automatically when modules are loaded
        Ok(())
    }

    /// Initialize TypeChecker with already-loaded stdlib modules from ModuleSystem
    /// Extracts type information from the loaded modules
    pub fn with_stdlib_modules(&mut self, modules: &HashMap<String, Program>) {
        for (path, program) in modules {
            if path.starts_with("@std") || path.starts_with("std.") {
                self.extract_types_from_program(program, path);
                self.stdlib_modules.insert(path.clone(), program.clone());
            }
        }
    }

    /// Extract type information from a stdlib program
    fn extract_types_from_program(&mut self, program: &Program, module_path: &str) {
        for decl in &program.declarations {
            match decl {
                Declaration::Struct(def) => {
                    let fields: Vec<(String, AstType)> = def
                        .fields
                        .iter()
                        .map(|f| (f.name.clone(), f.type_.clone()))
                        .collect();
                    self.structs.insert(def.name.clone(), StructInfo { fields });
                }
                Declaration::Function(func) => {
                    if let Some((receiver, method)) = func.name.split_once('.') {
                        // Method: Type.method
                        let key = format!("{}::{}", receiver, method);
                        let sig = MethodSignature {
                            receiver_type: receiver.to_string(),
                            method_name: method.to_string(),
                            params: func.args.clone(),
                            return_type: func.return_type.clone(),
                            is_static: func
                                .args
                                .first()
                                .map(|(name, _)| name != "self")
                                .unwrap_or(true),
                        };
                        self.stdlib_methods.insert(key, sig);
                    } else {
                        // Standalone function
                        let key = format!("{}::{}", module_path, func.name);
                        let sig = FunctionSignature {
                            params: func.args.clone(),
                            return_type: func.return_type.clone(),
                            is_external: false,
                        };
                        self.stdlib_functions.insert(key, sig);
                    }
                }
                Declaration::Enum(def) => {
                    let variants: Vec<(String, Option<AstType>)> = def
                        .variants
                        .iter()
                        .map(|v| (v.name.clone(), v.payload.clone()))
                        .collect();
                    self.enums.insert(def.name.clone(), EnumInfo { variants });
                }
                Declaration::TraitImplementation(trait_impl) => {
                    for method in &trait_impl.methods {
                        let key = format!("{}::{}", trait_impl.type_name, method.name);
                        let sig = MethodSignature {
                            receiver_type: trait_impl.type_name.clone(),
                            method_name: method.name.clone(),
                            params: method.args.clone(),
                            return_type: method.return_type.clone(),
                            is_static: method
                                .args
                                .first()
                                .map(|(n, _)| n != "self")
                                .unwrap_or(true),
                        };
                        self.stdlib_methods.insert(key, sig);
                    }
                }
                _ => {}
            }
        }
    }

    /// Look up stdlib method return type (replaces stdlib_types().get_method_return_type)
    pub fn get_stdlib_method_type(&self, receiver: &str, method: &str) -> Option<&AstType> {
        let key = format!("{}::{}", receiver, method);
        self.stdlib_methods.get(&key).map(|sig| &sig.return_type)
    }

    /// Look up stdlib function return type (replaces stdlib_types().get_function_return_type)
    pub fn get_stdlib_function_type(&self, module: &str, func_name: &str) -> Option<&AstType> {
        let key = format!("{}::{}", module, func_name);
        self.stdlib_functions.get(&key).map(|sig| &sig.return_type)
    }

    /// Get stdlib struct definition (replaces stdlib_types().get_struct_definition)
    pub fn get_stdlib_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.get(name)
    }

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

    fn add_pattern_bindings_to_scope(&mut self, pattern: &crate::ast::Pattern) -> Result<()> {
        // Default to I32 when no type context is available (legacy behavior)
        self.add_pattern_bindings_to_scope_with_type(pattern, &AstType::I32)
    }

    /// Helper to resolve the payload type for an enum variant pattern match
    fn resolve_enum_payload_type(&self, variant: &str, scrutinee_type: &AstType) -> AstType {
        match scrutinee_type {
            AstType::Generic {
                name: enum_name,
                type_args,
            } => {
                if self.well_known.is_result(enum_name) && type_args.len() >= 2 {
                    if self.well_known.is_ok(variant) {
                        type_args[0].clone()
                    } else if self.well_known.is_err(variant) {
                        type_args[1].clone()
                    } else {
                        AstType::I32
                    }
                } else if self.well_known.is_option(enum_name) && !type_args.is_empty() {
                    if self.well_known.is_some(variant) {
                        type_args[0].clone()
                    } else {
                        AstType::Void
                    }
                } else {
                    scrutinee_type.clone()
                }
            }
            AstType::Enum {
                name: enum_name,
                variants,
            } => {
                if self.well_known.is_option(enum_name) || self.well_known.is_result(enum_name) {
                    if let Some(enum_variant) = variants.iter().find(|v| v.name == variant) {
                        if let Some(payload_ty) = &enum_variant.payload {
                            return payload_ty.clone();
                        }
                        return AstType::Void;
                    }
                }
                scrutinee_type.clone()
            }
            _ => scrutinee_type.clone(),
        }
    }

    /// Helper to unwrap primitive types from Generic wrapper
    fn unwrap_primitive_generic(&self, scrutinee_type: &AstType) -> AstType {
        if let AstType::Generic {
            name: type_name,
            type_args,
        } = scrutinee_type
        {
            if type_args.is_empty() {
                return match type_name.as_str() {
                    "i32" | "I32" => AstType::I32,
                    "i64" | "I64" => AstType::I64,
                    "f32" | "F32" => AstType::F32,
                    "f64" | "F64" => AstType::F64,
                    "bool" | "Bool" => AstType::Bool,
                    "string" => AstType::StaticString,
                    "String" => crate::ast::resolve_string_struct_type(),
                    _ => scrutinee_type.clone(),
                };
            }
        }
        scrutinee_type.clone()
    }

    fn add_pattern_bindings_to_scope_with_type(
        &mut self,
        pattern: &crate::ast::Pattern,
        scrutinee_type: &AstType,
    ) -> Result<()> {
        use crate::ast::Pattern;

        match pattern {
            Pattern::Identifier(name) => {
                let binding_type = self.unwrap_primitive_generic(scrutinee_type);
                self.declare_variable(name, binding_type, false)?;
            }
            Pattern::EnumLiteral { variant, payload } => {
                if let Some(payload_pattern) = payload {
                    let payload_type = self.resolve_enum_payload_type(variant, scrutinee_type);
                    self.add_pattern_bindings_to_scope_with_type(payload_pattern, &payload_type)?;
                }
            }
            Pattern::EnumVariant {
                variant, payload, ..
            } => {
                if let Some(payload_pattern) = payload {
                    let payload_type = self.resolve_enum_payload_type(variant, scrutinee_type);
                    self.add_pattern_bindings_to_scope_with_type(payload_pattern, &payload_type)?;
                }
            }
            Pattern::Binding { name, pattern } => {
                // Binding pattern: name @ pattern
                // Add the name as a variable with the scrutinee type
                self.declare_variable(name, scrutinee_type.clone(), false)?;
                // And recursively process the pattern
                self.add_pattern_bindings_to_scope_with_type(pattern, scrutinee_type)?;
            }
            Pattern::Or(patterns) => {
                // For or patterns, we need to ensure all alternatives bind the same names
                // For now, just process the first one
                if let Some(first) = patterns.first() {
                    self.add_pattern_bindings_to_scope_with_type(first, scrutinee_type)?;
                }
            }
            Pattern::Struct { fields, .. } => {
                // For struct patterns, add bindings for all fields
                // Uses scrutinee_type for all bindings; field-specific types need struct lookup
                for field in fields {
                    self.add_pattern_bindings_to_scope_with_type(&field.1, scrutinee_type)?;
                }
            }
            Pattern::Type { binding, .. } => {
                // Type pattern with optional binding
                if let Some(name) = binding {
                    self.declare_variable(name, scrutinee_type.clone(), false)?;
                }
            }
            // Tuple patterns - recursively bind patterns in the tuple
            Pattern::Tuple(patterns) => {
                for pattern in patterns {
                    self.add_pattern_bindings_to_scope_with_type(pattern, scrutinee_type)?;
                }
            }
            // Other patterns don't create bindings
            Pattern::Wildcard
            | Pattern::Literal(_)
            | Pattern::Range { .. }
            | Pattern::Guard { .. } => {}
        }
        Ok(())
    }

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
