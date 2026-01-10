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

pub struct TypeChecker {
    scopes: Vec<HashMap<String, VariableInfo>>,
    functions: HashMap<String, FunctionSignature>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    behavior_resolver: BehaviorResolver,
    module_imports: HashMap<String, String>,
    current_impl_type: Option<String>,
    current_span: Option<Span>,
    pub well_known: WellKnownTypes,
    /// Shared type context that flows to later compilation phases
    pub type_context: TypeContext,
}

#[derive(Clone, Debug)]
pub struct FunctionSignature {
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_external: bool,
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
    fn parse_generic_type_string(type_str: &str) -> (String, Vec<AstType>) {
        if let Some(angle_pos) = type_str.find('<') {
            let base_type = type_str[..angle_pos].to_string();
            let args_str = &type_str[angle_pos + 1..type_str.len() - 1]; // Remove < and >

            // Simple parsing - split by comma and trim
            let type_args: Vec<AstType> = args_str
                .split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    match trimmed {
                        "i32" => AstType::I32,
                        "i64" => AstType::I64,
                        "f32" => AstType::F32,
                        "f64" => AstType::F64,
                        "bool" => AstType::Bool,
                        "string" => AstType::StaticString, // lowercase string maps to StaticString
                        "StaticString" => AstType::StaticString, // explicit static string type
                        "String" => crate::ast::resolve_string_struct_type(), // String is a struct from stdlib/string.zen
                        _ => {
                            // Check if it's another generic type
                            if trimmed.contains('<') {
                                let (inner_base, inner_args) =
                                    Self::parse_generic_type_string(trimmed);
                                AstType::Generic {
                                    name: inner_base,
                                    type_args: inner_args,
                                }
                            } else {
                                // Unknown type, treat as identifier
                                AstType::Generic {
                                    name: trimmed.to_string(),
                                    type_args: vec![],
                                }
                            }
                        }
                    }
                })
                .collect();

            (base_type, type_args)
        } else {
            (type_str.to_string(), vec![])
        }
    }

    pub fn new() -> Self {
        let enums = HashMap::new();

        // Register Option<T> and Result<T, E> as fallback until stdlib preloading is implemented
        // TODO: These should be loaded from stdlib/core/option.zen and stdlib/core/result.zen
        // For now, keep as fallback to ensure Option/Result work even without explicit imports
        // Option and Result are now defined in stdlib/core/option.zen and stdlib/core/result.zen
        // They will be loaded when the stdlib is imported - no hardcoding needed

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
            well_known: WellKnownTypes::new(),
            type_context: TypeContext::new(),
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
                    match self.infer_function_return_type(func) {
                        Ok(inferred_type) => {
                            if let Some(sig) = self.functions.get_mut(&func.name) {
                                sig.return_type = inferred_type;
                            }
                        }
                        Err(_) => {}
                    }
                }
            }
        }

        // Fourth pass: type check function bodies
        for declaration in &program.declarations {
            self.check_declaration(declaration)?;
        }

        // Build and return the TypeContext with all collected type information
        self.build_type_context();
        Ok(self.type_context.clone())
    }

    /// Build the TypeContext from collected type information
    fn build_type_context(&mut self) {
        // Copy function signatures to TypeContext
        for (name, sig) in &self.functions {
            self.type_context.register_function(
                name.clone(),
                sig.params.clone(),
                sig.return_type.clone(),
                sig.is_external,
            );
        }

        // Copy struct definitions to TypeContext
        for (name, info) in &self.structs {
            self.type_context.register_struct(name.clone(), info.fields.clone());
        }

        // Copy enum definitions to TypeContext
        for (name, info) in &self.enums {
            self.type_context.register_enum(name.clone(), info.variants.clone());
        }
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

    pub fn infer_expression_type(&mut self, expr: &Expression) -> Result<AstType> {
        // eprintln!("DEBUG TypeChecker: infer_expression_type called for expr type: {}",
        //     match expr {
        //         Expression::Integer8(_) => "Integer8",
        //         Expression::Integer16(_) => "Integer16",
        //         Expression::Integer32(_) => "Integer32",
        //         Expression::Integer64(_) => "Integer64",
        //         Expression::Identifier(_) => "Identifier",
        //         Expression::Conditional { .. } => "Conditional",
        //         Expression::PatternMatch { .. } => "PatternMatch",
        //         Expression::QuestionMatch { .. } => "QuestionMatch",
        //         Expression::Some(_) => "Some",
        //         Expression::None => "None",
        //         Expression::String(_) => "String",
        //         Expression::Boolean(_) => "Boolean",
        //         Expression::Unit => "Unit",
        //         _ => "Other"
        //     }
        // );
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
            Expression::FunctionCall { name, args } => {
                inference::infer_function_call_type(self, name, args)
            }
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
                    let registry = crate::stdlib_types::stdlib_types();
                    if let Some(struct_def) = registry.get_struct_definition(name) {
                        let fields: Vec<(String, AstType)> = struct_def.fields
                            .iter()
                            .map(|f| (f.name.clone(), f.type_.clone()))
                            .collect();
                        Ok(AstType::Struct {
                            name: name.clone(),
                            fields,
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
                    AstType::Array(elem_type) => Ok(*elem_type),
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
                // Infer type from first element
                if elements.is_empty() {
                    Ok(AstType::Array(Box::new(AstType::Void)))
                } else {
                    let elem_type = self.infer_expression_type(&elements[0])?;
                    Ok(AstType::Array(Box::new(elem_type)))
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
                payload: _,
            } => inference::infer_enum_variant_type(enum_name, variant, &self.enums),
            Expression::StringLength(_) => Ok(AstType::I64),
            Expression::MethodCall {
                object,
                method,
                args: _,
            } => inference::infer_method_call_type(self, object, method),
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
                // eprintln!("DEBUG TypeChecker: Processing conditional with {} arms", arms.len());
                // Conditional expression type is determined by the first arm
                // All arms should have the same type (checked during type checking)

                // Infer the type of the scrutinee to properly type pattern bindings
                let scrutinee_type = self.infer_expression_type(scrutinee)?;

                if arms.is_empty() {
                    Ok(AstType::Void)
                } else {
                    let mut result_type = AstType::Void;

                    // Process each arm with its own pattern bindings
                    for (i, arm) in arms.iter().enumerate() {
                        // eprintln!("DEBUG TypeChecker: Processing arm {} pattern: {:?}", i, arm.pattern);

                        // Enter a new scope for the pattern bindings
                        self.enter_scope();
                        // eprintln!("DEBUG TypeChecker: Entered scope for arm {}", i);

                        // Extract pattern bindings and add them to the scope
                        self.add_pattern_bindings_to_scope_with_type(
                            &arm.pattern,
                            &scrutinee_type,
                        )?;
                        // eprintln!("DEBUG TypeChecker: Added pattern bindings for arm {}", i);

                        // Infer the type with bindings in scope
                        // eprintln!("DEBUG TypeChecker: Inferring type for arm {} body", i);
                        let arm_type = self.infer_expression_type(&arm.body)?;
                        // eprintln!("DEBUG TypeChecker: Arm {} type: {:?}", i, arm_type);

                        // The first arm determines the type
                        if i == 0 {
                            result_type = arm_type;
                        }

                        // Exit the scope to remove the bindings
                        self.exit_scope();
                        // eprintln!("DEBUG TypeChecker: Exited scope for arm {}", i);
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
                size,
                initial_values: _,
            } => {
                // Vec<T, size>() -> Vec<T, size>
                Ok(AstType::Vec {
                    element_type: Box::new(element_type.clone()),
                    size: *size,
                })
            }
            Expression::DynVecConstructor {
                element_types,
                allocator: _,
                initial_capacity: _,
            } => {
                // DynVec<T>() or DynVec<T1, T2, ...>() -> DynVec<T, ...>
                Ok(AstType::DynVec {
                    element_types: element_types.clone(),
                    allocator_type: None, // Allocator type inferred from constructor arg
                })
            }
            Expression::ArrayConstructor { element_type } => {
                // Array<T>() -> Generic { name: "Array", type_args: [T] }
                // TODO: "Array" should come from TypeContext, not hardcoded
                Ok(AstType::Generic {
                    name: "Array".to_string(),
                    type_args: vec![element_type.clone()],
                })
            }
            Expression::Some(inner) => {
                let inner_type = self.infer_expression_type(inner)?;
                Ok(AstType::Generic {
                    name: self.well_known.get_variant_parent_name(self.well_known.some_name()).unwrap().to_string(),
                    type_args: vec![inner_type],
                })
            }
            Expression::None => {
                Ok(AstType::Generic {
                    name: self.well_known.get_variant_parent_name(self.well_known.none_name()).unwrap().to_string(),
                    type_args: vec![AstType::Void],
                })
            }
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
        self.behavior_resolver.resolve_method(type_name, method_name)
    }

    fn register_stdlib_module(&mut self, _alias: &str, _module_path: &str) -> Result<()> {
        // Stdlib function signatures are looked up dynamically via stdlib_types.rs
        // which parses the actual .zen files. No need for hardcoded registration.
        Ok(())
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
            AstType::Generic { name: enum_name, type_args } => {
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
            AstType::Enum { name: enum_name, variants } => {
                if self.well_known.is_option(enum_name) || self.well_known.is_result(enum_name) {
                    if let Some(enum_variant) = variants.iter().find(|v| &v.name == variant) {
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
        if let AstType::Generic { name: type_name, type_args } = scrutinee_type {
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
            Pattern::EnumVariant { variant, payload, .. } => {
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
                for field in fields {
                    // field is (String, Pattern)
                    // TODO: Should extract field type from struct type
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    #[test]
    fn test_basic_type_checking() {
        let input = "main: () void = {
            x = 42
            y : i32 = 100
            z = x + y
        }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut type_checker = TypeChecker::new();
        assert!(type_checker.check_program(&program).is_ok());
    }

    #[test]
    fn test_type_mismatch_error() {
        let input = "main: () void = {
            x : i32 = \"hello\"
        }";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        let mut type_checker = TypeChecker::new();
        let result = type_checker.check_program(&program);
        assert!(result.is_err());
        if let Err(CompileError::TypeError(msg, _)) = result {
            assert!(msg.contains("Type mismatch"));
        }
    }
}
