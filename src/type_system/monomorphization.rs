//! Monomorphization - Instantiate generic types with concrete type arguments
//!
//! This module transforms a program with generic types into a program where
//! all generics have been replaced with concrete instantiations.
//!
//! Pipeline: Parser → TypeChecker → TypeContext → Monomorphizer → Codegen
//!
//! The Monomorphizer:
//! - Uses TypeContext for type lookups (from typechecker)
//! - Uses TypeEnvironment to find generic definitions in the Program
//! - Uses TypeInstantiator to create concrete instantiations
//! - Collects all required instantiations by walking the AST
//! - Adds instantiated functions/structs/enums to the program

use super::{generate_instantiated_name, TypeEnvironment, TypeInstantiator};
use crate::ast::{AstType, Declaration, Expression, Program, Statement};
use crate::error::CompileError;
use crate::type_context::TypeContext;
use std::collections::HashSet;

/// Maximum recursion depth for AST traversal during monomorphization
const MAX_COLLECT_DEPTH: usize = 256;
/// Maximum number of generic instantiations to prevent unbounded growth
const MAX_INSTANTIATIONS: usize = 10_000;

/// Monomorphizer transforms generic code into concrete instantiations
pub struct Monomorphizer {
    /// Type information from the typechecker
    type_ctx: TypeContext,
    /// Pending instantiations to process: (base_name, type_args)
    pending: Vec<(String, Vec<AstType>)>,
    /// Already processed instantiations to avoid duplicates
    processed: HashSet<String>,
}

impl Monomorphizer {
    /// Create a new Monomorphizer with type information from the typechecker
    pub fn new(type_ctx: TypeContext) -> Self {
        Self {
            type_ctx,
            pending: Vec::new(),
            processed: HashSet::new(),
        }
    }

    /// Monomorphize a program - instantiate all generic types with concrete arguments
    pub fn monomorphize_program(&mut self, program: &Program) -> Result<Program, CompileError> {
        // Create TypeEnvironment to query generic definitions from the Program
        let type_env = TypeEnvironment::new(program);

        // Phase 1: Collect all generic instantiations needed
        self.collect_instantiations(program)?;

        // Phase 2: Process pending instantiations
        let mut new_declarations = Vec::new();

        while let Some((base_name, type_args)) = self.pending.pop() {
            let instantiated_name = generate_instantiated_name(&base_name, &type_args);

            // Skip if already processed
            if self.processed.contains(&instantiated_name) {
                continue;
            }
            self.processed.insert(instantiated_name.clone());

            // Try to instantiate as function, struct, or enum
            if let Some(generic_func) = type_env.get_generic_function(&base_name) {
                // Need mutable borrow for TypeInstantiator
                let mut type_env_mut = TypeEnvironment::new(program);
                let mut instantiator = TypeInstantiator::new(&mut type_env_mut);

                match instantiator.instantiate_function(generic_func, type_args.clone()) {
                    Ok(instantiated_func) => {
                        // Collect any nested instantiations from the instantiated function
                        self.collect_from_function(&instantiated_func, 0)?;
                        new_declarations.push(Declaration::Function(instantiated_func));
                    }
                    Err(e) => {
                        // Log but don't fail - some instantiations may not be valid
                        eprintln!("Warning: Failed to instantiate {}<...>: {}", base_name, e);
                    }
                }
            } else if let Some(generic_struct) = type_env.get_generic_struct(&base_name) {
                let mut type_env_mut = TypeEnvironment::new(program);
                let mut instantiator = TypeInstantiator::new(&mut type_env_mut);

                match instantiator.instantiate_struct(generic_struct, type_args.clone()) {
                    Ok(instantiated_struct) => {
                        new_declarations.push(Declaration::Struct(instantiated_struct));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to instantiate struct {}<...>: {}",
                            base_name, e
                        );
                    }
                }
            } else if let Some(generic_enum) = type_env.get_generic_enum(&base_name) {
                let mut type_env_mut = TypeEnvironment::new(program);
                let mut instantiator = TypeInstantiator::new(&mut type_env_mut);

                match instantiator.instantiate_enum(generic_enum, type_args.clone()) {
                    Ok(instantiated_enum) => {
                        new_declarations.push(Declaration::Enum(instantiated_enum));
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to instantiate enum {}<...>: {}",
                            base_name, e
                        );
                    }
                }
            }
        }

        // Phase 3: Build the output program with original + instantiated declarations
        let mut result_declarations = program.declarations.clone();
        result_declarations.extend(new_declarations);

        Ok(Program {
            declarations: result_declarations,
            statements: program.statements.clone(),
        })
    }

    /// Collect all generic instantiations needed from the program
    fn collect_instantiations(&mut self, program: &Program) -> Result<(), CompileError> {
        for decl in &program.declarations {
            self.collect_from_declaration(decl, 0)?;
        }
        for stmt in &program.statements {
            self.collect_from_statement(stmt, 0)?;
        }
        Ok(())
    }

    /// Collect instantiations from a declaration
    fn collect_from_declaration(
        &mut self,
        decl: &Declaration,
        depth: usize,
    ) -> Result<(), CompileError> {
        if depth > MAX_COLLECT_DEPTH {
            return Ok(());
        }
        match decl {
            Declaration::Function(func) => self.collect_from_function(func, depth)?,
            Declaration::Struct(struct_def) => {
                for method in &struct_def.methods {
                    self.collect_from_function(method, depth)?;
                }
            }
            Declaration::Enum(enum_def) => {
                for method in &enum_def.methods {
                    self.collect_from_function(method, depth)?;
                }
            }
            Declaration::ImplBlock(impl_block) => {
                for method in &impl_block.methods {
                    self.collect_from_function(method, depth)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Collect instantiations from a function
    fn collect_from_function(
        &mut self,
        func: &crate::ast::Function,
        depth: usize,
    ) -> Result<(), CompileError> {
        for stmt in &func.body {
            self.collect_from_statement(stmt, depth)?;
        }
        Ok(())
    }

    /// Collect instantiations from a statement
    fn collect_from_statement(
        &mut self,
        stmt: &Statement,
        depth: usize,
    ) -> Result<(), CompileError> {
        if depth > MAX_COLLECT_DEPTH {
            return Ok(());
        }
        match stmt {
            Statement::Expression { expr, .. } => self.collect_from_expression(expr, depth + 1)?,
            Statement::Return { expr, .. } => self.collect_from_expression(expr, depth + 1)?,
            Statement::VariableDeclaration {
                initializer, type_, ..
            } => {
                if let Some(init) = initializer {
                    self.collect_from_expression(init, depth + 1)?;
                }
                if let Some(ty) = type_ {
                    self.collect_from_type(ty, depth + 1)?;
                }
            }
            Statement::VariableAssignment { value, .. } => {
                self.collect_from_expression(value, depth + 1)?;
            }
            Statement::Loop { kind, body, .. } => {
                if let crate::ast::LoopKind::Condition(expr) = kind {
                    self.collect_from_expression(expr, depth + 1)?;
                }
                for stmt in body {
                    self.collect_from_statement(stmt, depth + 1)?;
                }
            }
            Statement::Break { .. } | Statement::Continue { .. } => {}
            _ => {}
        }
        Ok(())
    }

    /// Collect instantiations from an expression
    fn collect_from_expression(
        &mut self,
        expr: &Expression,
        depth: usize,
    ) -> Result<(), CompileError> {
        if depth > MAX_COLLECT_DEPTH {
            return Ok(());
        }
        match expr {
            Expression::FunctionCall {
                module,
                name,
                type_args,
                args,
                ..
            } => {
                // If explicit type args provided, queue for instantiation
                if !type_args.is_empty() {
                    let base_name = extract_base_name(name);
                    self.queue_instantiation(base_name, type_args.clone())?;
                }
                // Also check if module qualifier contains embedded type args like "Vec<i32>"
                else if let Some(m) = module {
                    if m.contains('<') {
                        if let Some((base, args)) = parse_embedded_type_args(m) {
                            self.queue_instantiation(base, args)?;
                        }
                    }
                }
                // Recurse into arguments
                for arg in args {
                    self.collect_from_expression(arg, depth + 1)?;
                }
            }
            Expression::MethodCall {
                object,
                type_args,
                args,
                ..
            } => {
                self.collect_from_expression(object, depth + 1)?;
                if !type_args.is_empty() {
                    for ty in type_args {
                        self.collect_from_type(ty, depth + 1)?;
                    }
                }
                for arg in args {
                    self.collect_from_expression(arg, depth + 1)?;
                }
            }
            Expression::StructLiteral { name, fields } => {
                if name.contains('<') {
                    if let Some((base, args)) = parse_embedded_type_args(name) {
                        self.queue_instantiation(base, args)?;
                    }
                }
                for (_, field_expr) in fields {
                    self.collect_from_expression(field_expr, depth + 1)?;
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.collect_from_expression(left, depth + 1)?;
                self.collect_from_expression(right, depth + 1)?;
            }
            Expression::QuestionMatch { scrutinee, arms } => {
                self.collect_from_expression(scrutinee, depth + 1)?;
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_from_expression(guard, depth + 1)?;
                    }
                    self.collect_from_expression(&arm.body, depth + 1)?;
                }
            }
            Expression::Conditional { scrutinee, arms } => {
                self.collect_from_expression(scrutinee, depth + 1)?;
                for arm in arms {
                    self.collect_from_expression(&arm.body, depth + 1)?;
                }
            }
            Expression::MemberAccess { object, .. }
            | Expression::StructField {
                struct_: object, ..
            } => {
                self.collect_from_expression(object, depth + 1)?;
            }
            Expression::ArrayLiteral(items) => {
                for item in items {
                    self.collect_from_expression(item, depth + 1)?;
                }
            }
            Expression::ArrayIndex { array, index } => {
                self.collect_from_expression(array, depth + 1)?;
                self.collect_from_expression(index, depth + 1)?;
            }
            Expression::Dereference(inner) | Expression::AddressOf(inner) => {
                self.collect_from_expression(inner, depth + 1)?;
            }
            Expression::VecConstructor {
                element_type,
                initial_values,
                ..
            } => {
                self.collect_from_type(element_type, depth + 1)?;
                if let Some(values) = initial_values {
                    for val in values {
                        self.collect_from_expression(val, depth + 1)?;
                    }
                }
            }
            Expression::DynVecConstructor {
                element_types,
                allocator,
                initial_capacity,
            } => {
                for ty in element_types {
                    self.collect_from_type(ty, depth + 1)?;
                }
                self.collect_from_expression(allocator, depth + 1)?;
                if let Some(cap) = initial_capacity {
                    self.collect_from_expression(cap, depth + 1)?;
                }
            }
            Expression::Closure { body, .. } => {
                self.collect_from_expression(body, depth + 1)?;
            }
            Expression::Block(statements) => {
                for stmt in statements {
                    self.collect_from_statement(stmt, depth + 1)?;
                }
            }
            // Literals and identifiers don't contain generic instantiations
            _ => {}
        }
        Ok(())
    }

    /// Collect instantiations from a type annotation
    fn collect_from_type(&mut self, ast_type: &AstType, depth: usize) -> Result<(), CompileError> {
        if depth > MAX_COLLECT_DEPTH {
            return Ok(());
        }
        match ast_type {
            AstType::Generic { name, type_args } if !type_args.is_empty() => {
                self.queue_instantiation(name.clone(), type_args.clone())?;
                for arg in type_args {
                    self.collect_from_type(arg, depth + 1)?;
                }
            }
            AstType::Slice(inner) | AstType::Ref(inner) => {
                self.collect_from_type(inner, depth + 1)?;
            }
            AstType::Function { args, return_type }
            | AstType::FunctionPointer {
                param_types: args,
                return_type,
            } => {
                for arg in args {
                    self.collect_from_type(arg, depth + 1)?;
                }
                self.collect_from_type(return_type, depth + 1)?;
            }
            t if t.is_ptr_type() => {
                if let Some(inner) = t.ptr_inner() {
                    self.collect_from_type(inner, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Queue a generic instantiation for processing
    fn queue_instantiation(
        &mut self,
        base_name: String,
        type_args: Vec<AstType>,
    ) -> Result<(), CompileError> {
        if type_args.is_empty() {
            return Ok(());
        }

        // Prevent unbounded growth of instantiations
        if self.processed.len() + self.pending.len() >= MAX_INSTANTIATIONS {
            return Err(CompileError::InternalError(
                format!(
                    "Monomorphization limit exceeded: more than {} generic instantiations \
                     generated (last type: '{}'); possible infinite recursion in generic types",
                    MAX_INSTANTIATIONS, base_name
                ),
                None,
            ));
        }

        let instantiated_name = generate_instantiated_name(&base_name, &type_args);

        // Skip if already processed or queued
        if self.processed.contains(&instantiated_name) {
            return Ok(());
        }

        // Check if already in pending queue
        let already_queued = self
            .pending
            .iter()
            .any(|(name, args)| *name == base_name && *args == type_args);

        if !already_queued {
            self.pending.push((base_name, type_args));
        }
        Ok(())
    }

    /// Get the TypeContext (for passing to codegen after monomorphization)
    pub fn into_type_context(self) -> TypeContext {
        self.type_ctx
    }
}

/// Extract base name from a potentially generic name like "Vec<i32>" -> "Vec"
fn extract_base_name(name: &str) -> String {
    crate::name_utils::strip_generics(name).to_string()
}

/// Parse embedded type arguments from a string like "Vec<i32>" -> Some(("Vec", [I32]))
fn parse_embedded_type_args(name: &str) -> Option<(String, Vec<AstType>)> {
    let pos = name.find('<')?;
    let base_name = crate::name_utils::strip_generics(name).to_string();
    let type_args_str = &name[pos + 1..name.len() - 1]; // Remove < and >

    let type_args = match crate::parser::parse_type_args_from_string(type_args_str) {
        Ok(args) => args,
        Err(e) => {
            eprintln!(
                "Warning: Failed to parse type arguments in '{}': {}",
                name, e
            );
            return None;
        }
    };

    if type_args.is_empty() {
        None
    } else {
        Some((base_name, type_args))
    }
}
