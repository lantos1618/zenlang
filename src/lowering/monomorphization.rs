use super::{TypeEnvironment, TypeInstantiator};
use crate::ast::{AstType, Declaration, Expression, Function, Program};
use crate::error::CompileError;
use crate::stdlib_types::StdlibTypeRegistry;
use crate::typechecker::TypeChecker;
use std::collections::{HashMap, HashSet};

pub struct Monomorphizer {
    env: TypeEnvironment,
    instantiated_functions: HashMap<String, Function>,
    pending_instantiations: Vec<(String, Vec<AstType>)>,
    processed_instantiations: HashSet<(String, Vec<AstType>)>,
    type_checker: TypeChecker,
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
            instantiated_functions: HashMap::new(),
            pending_instantiations: Vec::new(),
            processed_instantiations: HashSet::new(),
            type_checker: TypeChecker::new(),
        }
    }

    pub fn monomorphize_program(&mut self, program: &Program) -> Result<Program, CompileError> {
        let mut declarations = Vec::new();

        self.type_checker.check_program(program)?;

        // After type checking, update functions with inferred return types
        let mut updated_functions = std::collections::HashMap::new();
        for (func_name, sig) in self.type_checker.get_function_signatures() {
            updated_functions.insert(func_name.clone(), sig.return_type.clone());
        }

        for decl in &program.declarations {
            match decl {
                Declaration::Function(func) => {
                    let mut updated_func = func.clone();
                    // If the function has Void return type and the type checker inferred a different type, update it
                    if func.return_type == AstType::Void {
                        if let Some(inferred_type) = updated_functions.get(&func.name) {
                            if *inferred_type != AstType::Void {
                                updated_func.return_type = inferred_type.clone();
                            }
                        }
                    }

                    if !func.type_params.is_empty() {
                        self.env.register_generic_function(updated_func.clone());
                        declarations.push(Declaration::Function(updated_func));
                    } else {
                        declarations.push(Declaration::Function(updated_func));
                    }
                }
                Declaration::Struct(struct_def) if !struct_def.type_params.is_empty() => {
                    self.env.register_generic_struct(struct_def.clone());
                    declarations.push(decl.clone());
                }
                Declaration::Enum(enum_def) if !enum_def.type_params.is_empty() => {
                    self.env.register_generic_enum(enum_def.clone());
                    // For now, also keep generic enums in declarations to ensure they're available
                    // This is a temporary fix - proper solution would be to infer instantiation from usage
                    declarations.push(decl.clone());
                }
                _ => declarations.push(decl.clone()),
            }
        }

        for decl in &program.declarations {
            self.collect_instantiations_from_declaration(decl)?;
        }

        while !self.pending_instantiations.is_empty() {
            let instantiations = std::mem::take(&mut self.pending_instantiations);

            for (name, type_args) in instantiations {
                if self
                    .processed_instantiations
                    .contains(&(name.clone(), type_args.clone()))
                {
                    continue;
                }

                self.processed_instantiations
                    .insert((name.clone(), type_args.clone()));

                if let Some(func) = self.env.get_generic_function(&name).cloned() {
                    let mut instantiator = TypeInstantiator::new(&mut self.env);
                    let instantiated = instantiator.instantiate_function(&func, type_args)?;

                    self.collect_instantiations_from_function(&instantiated)?;

                    declarations.push(Declaration::Function(instantiated.clone()));
                    self.instantiated_functions
                        .insert(instantiated.name.clone(), instantiated);
                } else if let Some(struct_def) = self.env.get_generic_struct(&name).cloned() {
                    let mut instantiator = TypeInstantiator::new(&mut self.env);
                    let instantiated = instantiator.instantiate_struct(&struct_def, type_args)?;
                    declarations.push(Declaration::Struct(instantiated));
                } else if let Some(enum_def) = self.env.get_generic_enum(&name).cloned() {
                    let mut instantiator = TypeInstantiator::new(&mut self.env);
                    let instantiated = instantiator.instantiate_enum(&enum_def, type_args)?;
                    declarations.push(Declaration::Enum(instantiated));
                }
            }
        }

        // Transform all function calls to use monomorphized names
        let transformed_declarations = self.transform_declarations(declarations)?;

        Ok(Program {
            declarations: transformed_declarations,
            statements: Vec::new(),
        })
    }

    fn collect_instantiations_from_declaration(
        &mut self,
        decl: &Declaration,
    ) -> Result<(), CompileError> {
        match decl {
            Declaration::Function(func) => self.collect_instantiations_from_function(func),
            Declaration::Struct(struct_def) => {
                for method in &struct_def.methods {
                    self.collect_instantiations_from_function(method)?;
                }
                Ok(())
            }
            Declaration::Enum(enum_def) => {
                for method in &enum_def.methods {
                    self.collect_instantiations_from_function(method)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn collect_instantiations_from_function(
        &mut self,
        func: &Function,
    ) -> Result<(), CompileError> {
        for stmt in &func.body {
            self.collect_instantiations_from_statement(stmt)?;
        }
        Ok(())
    }

    fn collect_instantiations_from_statement(
        &mut self,
        stmt: &crate::ast::Statement,
    ) -> Result<(), CompileError> {
        match stmt {
            crate::ast::Statement::Expression { expr, .. } => {
                self.collect_instantiations_from_expression(expr)
            }
            crate::ast::Statement::Return { expr, .. } => {
                self.collect_instantiations_from_expression(expr)
            }
            crate::ast::Statement::VariableDeclaration {
                initializer, type_, ..
            } => {
                if let Some(init) = initializer {
                    self.collect_instantiations_from_expression(init)?;
                }
                if let Some(ty) = type_ {
                    self.collect_instantiations_from_type(ty)?;
                }
                Ok(())
            }
            crate::ast::Statement::Loop { kind, body, .. } => {
                use crate::ast::LoopKind;
                match kind {
                    LoopKind::Condition(expr) => {
                        self.collect_instantiations_from_expression(expr)?;
                    }
                    LoopKind::Infinite => {}
                }
                for stmt in body {
                    self.collect_instantiations_from_statement(stmt)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn collect_instantiations_from_expression(
        &mut self,
        expr: &Expression,
    ) -> Result<(), CompileError> {
        match expr {
            Expression::FunctionCall { name, args } => {
                // Check if this is a generic function
                let base_name = extract_base_name(name);
                if let Some(generic_func) = self.env.get_generic_function(&base_name) {
                    // Infer type arguments from the call arguments
                    let type_args = self.infer_type_arguments(&generic_func, args)?;
                    if !type_args.is_empty() {
                        self.pending_instantiations.push((base_name, type_args));
                    }
                }

                for arg in args {
                    self.collect_instantiations_from_expression(arg)?;
                }
                Ok(())
            }
            Expression::StructLiteral { name, fields } => {
                // Check if this is a generic struct by checking if it exists in the environment
                let base_name = extract_base_name(name);
                if self.env.get_generic_struct(&base_name).is_some() {
                    // Infer type arguments from field values
                    let mut type_args = Vec::new();

                    for (_, field_expr) in fields {
                        // First collect instantiations from the field expression
                        self.collect_instantiations_from_expression(field_expr)?;

                        // Then try to infer its type for struct instantiation
                        if let Ok(field_type) = self.infer_expression_type(field_expr) {
                            // If this field's type helps determine a type parameter, add it
                            if !type_args.contains(&field_type)
                                && matches!(
                                    field_type,
                                    AstType::I32 | AstType::I64 | AstType::F32 | AstType::F64
                                )
                            {
                                type_args.push(field_type);
                                break; // For now, just take the first concrete type we find
                            }
                        }
                    }

                    if !type_args.is_empty() {
                        self.pending_instantiations.push((base_name, type_args));
                    }
                } else {
                    // Not a generic struct, just process field expressions
                    for (_, expr) in fields {
                        self.collect_instantiations_from_expression(expr)?;
                    }
                }
                Ok(())
            }
            Expression::BinaryOp { left, right, .. } => {
                self.collect_instantiations_from_expression(left)?;
                self.collect_instantiations_from_expression(right)
            }
            Expression::QuestionMatch { scrutinee, arms } => {
                self.collect_instantiations_from_expression(scrutinee)?;
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.collect_instantiations_from_expression(guard)?;
                    }
                    self.collect_instantiations_from_expression(&arm.body)?;
                }
                Ok(())
            }
            Expression::Dereference(expr) | Expression::AddressOf(expr) => {
                self.collect_instantiations_from_expression(expr)
            }
            Expression::ArrayLiteral(items) => {
                for item in items {
                    self.collect_instantiations_from_expression(item)?;
                }
                Ok(())
            }
            Expression::DynVecConstructor {
                element_types,
                allocator,
                initial_capacity,
            } => {
                // Process element types
                for element_type in element_types {
                    self.collect_instantiations_from_type(element_type)?;
                }
                // Process allocator expression
                self.collect_instantiations_from_expression(allocator)?;
                // Process initial capacity if provided
                if let Some(capacity) = initial_capacity {
                    self.collect_instantiations_from_expression(capacity)?;
                }
                Ok(())
            }
            Expression::VecConstructor {
                element_type,
                initial_values,
                ..
            } => {
                self.collect_instantiations_from_type(element_type)?;
                if let Some(values) = initial_values {
                    for value in values {
                        self.collect_instantiations_from_expression(value)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn collect_instantiations_from_type(&mut self, ast_type: &AstType) -> Result<(), CompileError> {
        match ast_type {
            AstType::Generic { name, type_args } => {
                if !type_args.is_empty()
                    && (self.env.get_generic_struct(name).is_some()
                        || self.env.get_generic_enum(name).is_some())
                {
                    self.pending_instantiations
                        .push((name.clone(), type_args.clone()));
                }

                for arg in type_args {
                    self.collect_instantiations_from_type(arg)?;
                }
                Ok(())
            }
            t if t.is_ptr_type() => {
                if let Some(inner) = t.ptr_inner() {
                    self.collect_instantiations_from_type(inner)
                } else {
                    Ok(())
                }
            }
            AstType::Array(inner) | AstType::Ref(inner) => {
                self.collect_instantiations_from_type(inner)
            }
            // Option and Result are now Generic types - handled in Generic match above
            AstType::Function { args, return_type } => {
                for arg in args {
                    self.collect_instantiations_from_type(arg)?;
                }
                self.collect_instantiations_from_type(return_type)
            }
            AstType::Vec { element_type, .. } => {
                self.collect_instantiations_from_type(element_type)
            }
            AstType::DynVec { element_types, .. } => {
                for elem_type in element_types {
                    self.collect_instantiations_from_type(elem_type)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn infer_type_arguments(
        &self,
        generic_func: &Function,
        args: &[Expression],
    ) -> Result<Vec<AstType>, CompileError> {
        let mut type_args = Vec::new();

        // For each type parameter in the generic function, try to infer it from the arguments
        for type_param in &generic_func.type_params {
            // Find the first parameter that uses this type parameter
            let mut inferred_type = None;

            for (i, (_param_name, param_type)) in generic_func.args.iter().enumerate() {
                if let Some(arg_expr) = args.get(i) {
                    // Check if this parameter uses the current type parameter
                    if self.type_uses_parameter(param_type, &type_param.name) {
                        // Infer the type from the argument expression
                        inferred_type = Some(self.infer_expression_type(arg_expr)?);
                        break;
                    }
                }
            }

            if let Some(ty) = inferred_type {
                type_args.push(ty);
            } else {
                // Couldn't infer this type parameter, default to i32 for now
                // In a real implementation, this would be an error
                type_args.push(AstType::I32);
            }
        }

        Ok(type_args)
    }

    fn type_uses_parameter(&self, ast_type: &AstType, param_name: &str) -> bool {
        match ast_type {
            AstType::Generic { name, .. } if name == param_name => true,
            t if t.is_ptr_type() => {
                if let Some(inner) = t.ptr_inner() {
                    self.type_uses_parameter(inner, param_name)
                } else {
                    false
                }
            }
            AstType::Array(inner) | AstType::Ref(inner) => {
                self.type_uses_parameter(inner, param_name)
            }
            // Option and Result are now Generic types - handled in Generic match above
            AstType::Vec { element_type, .. } => self.type_uses_parameter(element_type, param_name),
            AstType::DynVec { element_types, .. } => element_types
                .iter()
                .any(|t| self.type_uses_parameter(t, param_name)),
            _ => false,
        }
    }

    fn infer_expression_type(&self, expr: &Expression) -> Result<AstType, CompileError> {
        match expr {
            Expression::Integer32(_) => Ok(AstType::I32),
            Expression::Integer64(_) => Ok(AstType::I64),
            Expression::Float32(_) => Ok(AstType::F32),
            Expression::Float64(_) => Ok(AstType::F64),
            Expression::Boolean(_) => Ok(AstType::Bool),
            Expression::String(_) => Ok(crate::ast::resolve_string_struct_type()),
            Expression::Identifier(_name) => {
                // Would need access to variable types here
                // For now, return a placeholder
                Ok(AstType::I32)
            }
            _ => Ok(AstType::Void),
        }
    }

    fn transform_declarations(
        &mut self,
        declarations: Vec<Declaration>,
    ) -> Result<Vec<Declaration>, CompileError> {
        let mut result = Vec::new();

        for decl in declarations {
            match decl {
                Declaration::Function(func) => {
                    let transformed_func = self.transform_function(func)?;
                    result.push(Declaration::Function(transformed_func));
                }
                other => result.push(other),
            }
        }

        Ok(result)
    }

    fn transform_function(&mut self, mut func: Function) -> Result<Function, CompileError> {
        func.body = self.transform_statements(func.body)?;
        Ok(func)
    }

    fn transform_statements(
        &mut self,
        statements: Vec<crate::ast::Statement>,
    ) -> Result<Vec<crate::ast::Statement>, CompileError> {
        let mut result = Vec::new();

        for stmt in statements {
            result.push(self.transform_statement(stmt)?);
        }

        Ok(result)
    }

    fn transform_statement(
        &mut self,
        stmt: crate::ast::Statement,
    ) -> Result<crate::ast::Statement, CompileError> {
        match stmt {
            crate::ast::Statement::Expression { expr, span } => {
                Ok(crate::ast::Statement::Expression {
                    expr: self.transform_expression(expr)?,
                    span,
                })
            }
            crate::ast::Statement::Return { expr, span } => Ok(crate::ast::Statement::Return {
                expr: self.transform_expression(expr)?,
                span,
            }),
            crate::ast::Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                is_mutable,
                declaration_type,
                span,
            } => {
                let transformed_init = if let Some(init) = initializer {
                    Some(self.transform_expression(init)?)
                } else {
                    None
                };
                Ok(crate::ast::Statement::VariableDeclaration {
                    name,
                    type_,
                    initializer: transformed_init,
                    is_mutable,
                    declaration_type,
                    span,
                })
            }
            crate::ast::Statement::VariableAssignment { name, value, span } => {
                Ok(crate::ast::Statement::VariableAssignment {
                    name,
                    value: self.transform_expression(value)?,
                    span,
                })
            }
            other => Ok(other),
        }
    }

    fn transform_expression(&mut self, expr: Expression) -> Result<Expression, CompileError> {
        match expr {
            Expression::FunctionCall { name, args } => {
                // Check if this is a call to a generic function that has been monomorphized
                let base_name = extract_base_name(&name);

                // Transform the arguments first
                let transformed_args: Vec<Expression> = args
                    .into_iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                // If this is a generic function, we need to determine which instantiation to use
                if self.env.get_generic_function(&base_name).is_some() {
                    // Infer the types of the arguments to determine the instantiation
                    let arg_types: Vec<AstType> = transformed_args
                        .iter()
                        .map(|arg| self.infer_expression_type(arg))
                        .collect::<Result<Vec<_>, _>>()?;

                    // Generate the monomorphized name
                    let instantiated_name = generate_instantiated_name(&base_name, &arg_types);

                    Ok(Expression::FunctionCall {
                        name: instantiated_name,
                        args: transformed_args,
                    })
                } else {
                    Ok(Expression::FunctionCall {
                        name,
                        args: transformed_args,
                    })
                }
            }
            Expression::BinaryOp { left, op, right } => Ok(Expression::BinaryOp {
                left: Box::new(self.transform_expression(*left)?),
                op,
                right: Box::new(self.transform_expression(*right)?),
            }),
            Expression::StructLiteral { name, fields } => {
                // Transform field expressions
                let transformed_fields: Vec<(String, Expression)> = fields
                    .into_iter()
                    .map(|(field_name, field_expr)| {
                        self.transform_expression(field_expr)
                            .map(|expr| (field_name, expr))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                // Check if this is a generic struct that needs monomorphization
                let base_name = extract_base_name(&name);
                if self.env.get_generic_struct(&base_name).is_some() {
                    // Infer types from field values to determine the instantiation
                    // For now, we'll use a simplified approach that looks for specific patterns
                    // This should be enhanced with proper type inference

                    // Try to infer the type from the fields
                    if let Some(_struct_def) = self.env.get_generic_struct(&base_name).cloned() {
                        // Collect type arguments based on field types
                        let mut type_args = Vec::new();

                        // Simple heuristic: infer from field expressions
                        for (_field_name, field_expr) in &transformed_fields {
                            if let Ok(field_type) = self.infer_expression_type(field_expr) {
                                // If this field's type helps determine a type parameter, add it
                                if !type_args.contains(&field_type)
                                    && matches!(
                                        field_type,
                                        AstType::I32 | AstType::I64 | AstType::F32 | AstType::F64
                                    )
                                {
                                    type_args.push(field_type);
                                    break; // For now, just take the first concrete type we find
                                }
                            }
                        }

                        if !type_args.is_empty() {
                            let instantiated_name =
                                generate_instantiated_name(&base_name, &type_args);
                            return Ok(Expression::StructLiteral {
                                name: instantiated_name,
                                fields: transformed_fields,
                            });
                        }
                    }
                }

                Ok(Expression::StructLiteral {
                    name,
                    fields: transformed_fields,
                })
            }
            Expression::MemberAccess { object, member } => Ok(Expression::MemberAccess {
                object: Box::new(self.transform_expression(*object)?),
                member,
            }),
            other => Ok(other),
        }
    }
}

#[allow(dead_code)]
fn generate_instantiated_name(base_name: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        return base_name.to_string();
    }

    let type_names: Vec<String> = type_args.iter().map(type_to_string).collect();
    format!("{}_{}", base_name, type_names.join("_"))
}

#[allow(dead_code)]
fn type_to_string(ast_type: &AstType) -> String {
    match ast_type {
        AstType::I8 => "i8".to_string(),
        AstType::I16 => "i16".to_string(),
        AstType::I32 => "i32".to_string(),
        AstType::I64 => "i64".to_string(),
        AstType::U8 => "u8".to_string(),
        AstType::U16 => "u16".to_string(),
        AstType::U32 => "u32".to_string(),
        AstType::U64 => "u64".to_string(),
        AstType::F32 => "f32".to_string(),
        AstType::F64 => "f64".to_string(),
        AstType::Bool => "bool".to_string(),
        AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => "string".to_string(),
        AstType::Void => "void".to_string(),
        _ => "unknown".to_string(),
    }
}

#[allow(dead_code)]
fn extract_generic_struct_types(name: &str) -> Option<Vec<AstType>> {
    if name.contains('<') && name.contains('>') {
        // TODO: Parse type arguments from struct construction syntax
        None
    } else {
        None
    }
}

#[allow(dead_code)]
fn extract_base_name(name: &str) -> String {
    if let Some(idx) = name.find('<') {
        name[..idx].to_string()
    } else {
        name.to_string()
    }
}
