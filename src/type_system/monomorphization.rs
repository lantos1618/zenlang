use crate::ast::{Program, Declaration, Expression, AstType, Function, Statement};
use super::{TypeEnvironment, TypeInstantiator};
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

    pub fn monomorphize_program(&mut self, program: &Program) -> Result<Program, String> {
        let mut declarations = Vec::new();
        
        // First, type check the program to get type information
        self.type_checker.check_program(program).map_err(|e| e.to_string())?;
        
        for decl in &program.declarations {
            match decl {
                Declaration::Function(func) if !func.type_params.is_empty() => {
                    self.env.register_generic_function(func.clone());
                }
                Declaration::Struct(struct_def) if !struct_def.type_params.is_empty() => {
                    self.env.register_generic_struct(struct_def.clone());
                }
                Declaration::Enum(enum_def) if !enum_def.type_params.is_empty() => {
                    self.env.register_generic_enum(enum_def.clone());
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
                if self.processed_instantiations.contains(&(name.clone(), type_args.clone())) {
                    continue;
                }
                
                self.processed_instantiations.insert((name.clone(), type_args.clone()));
                
                if let Some(func) = self.env.get_generic_function(&name).cloned() {
                    let mut instantiator = TypeInstantiator::new(&mut self.env);
                    let instantiated = instantiator.instantiate_function(&func, type_args)?;
                    
                    self.collect_instantiations_from_function(&instantiated)?;
                    
                    declarations.push(Declaration::Function(instantiated.clone()));
                    self.instantiated_functions.insert(instantiated.name.clone(), instantiated);
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
        
        Ok(Program { declarations: transformed_declarations })
    }

    fn collect_instantiations_from_declaration(&mut self, decl: &Declaration) -> Result<(), String> {
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

    fn collect_instantiations_from_function(&mut self, func: &Function) -> Result<(), String> {
        for stmt in &func.body {
            self.collect_instantiations_from_statement(stmt)?;
        }
        Ok(())
    }

    fn collect_instantiations_from_statement(&mut self, stmt: &crate::ast::Statement) -> Result<(), String> {
        match stmt {
            crate::ast::Statement::Expression(expr) => {
                self.collect_instantiations_from_expression(expr)
            }
            crate::ast::Statement::Return(expr) => {
                self.collect_instantiations_from_expression(expr)
            }
            crate::ast::Statement::VariableDeclaration { initializer, type_, .. } => {
                if let Some(init) = initializer {
                    self.collect_instantiations_from_expression(init)?;
                }
                if let Some(ty) = type_ {
                    self.collect_instantiations_from_type(ty)?;
                }
                Ok(())
            }
            crate::ast::Statement::Loop { condition, body, .. } => {
                if let Some(cond) = condition {
                    self.collect_instantiations_from_expression(cond)?;
                }
                for stmt in body {
                    self.collect_instantiations_from_statement(stmt)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn collect_instantiations_from_expression(&mut self, expr: &Expression) -> Result<(), String> {
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
                if let Some(type_args) = extract_generic_struct_types(name) {
                    let base_name = extract_base_name(name);
                    if self.env.get_generic_struct(&base_name).is_some() {
                        self.pending_instantiations.push((base_name, type_args));
                    }
                }
                
                for (_, expr) in fields {
                    self.collect_instantiations_from_expression(expr)?;
                }
                Ok(())
            }
            Expression::BinaryOp { left, right, .. } => {
                self.collect_instantiations_from_expression(left)?;
                self.collect_instantiations_from_expression(right)
            }
            Expression::Conditional { scrutinee, arms } => {
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
            _ => Ok(()),
        }
    }

    fn collect_instantiations_from_type(&mut self, ast_type: &AstType) -> Result<(), String> {
        match ast_type {
            AstType::Generic { name, type_args } => {
                if !type_args.is_empty() {
                    if self.env.get_generic_struct(name).is_some() || 
                       self.env.get_generic_enum(name).is_some() {
                        self.pending_instantiations.push((name.clone(), type_args.clone()));
                    }
                }
                
                for arg in type_args {
                    self.collect_instantiations_from_type(arg)?;
                }
                Ok(())
            }
            AstType::Pointer(inner) | AstType::Array(inner) | 
            AstType::Option(inner) | AstType::Ref(inner) => {
                self.collect_instantiations_from_type(inner)
            }
            AstType::Result { ok_type, err_type } => {
                self.collect_instantiations_from_type(ok_type)?;
                self.collect_instantiations_from_type(err_type)
            }
            AstType::Function { args, return_type } => {
                for arg in args {
                    self.collect_instantiations_from_type(arg)?;
                }
                self.collect_instantiations_from_type(return_type)
            }
            _ => Ok(()),
        }
    }
    
    fn infer_type_arguments(&self, generic_func: &Function, args: &[Expression]) -> Result<Vec<AstType>, String> {
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
            AstType::Pointer(inner) | AstType::Array(inner) | 
            AstType::Option(inner) | AstType::Ref(inner) => {
                self.type_uses_parameter(inner, param_name)
            }
            _ => false,
        }
    }
    
    fn infer_expression_type(&self, expr: &Expression) -> Result<AstType, String> {
        match expr {
            Expression::Integer32(_) => Ok(AstType::I32),
            Expression::Integer64(_) => Ok(AstType::I64),
            Expression::Float32(_) => Ok(AstType::F32),
            Expression::Float64(_) => Ok(AstType::F64),
            Expression::Boolean(_) => Ok(AstType::Bool),
            Expression::String(_) => Ok(AstType::String),
            Expression::Identifier(name) => {
                // Would need access to variable types here
                // For now, return a placeholder
                Ok(AstType::I32)
            }
            _ => Ok(AstType::Void),
        }
    }
    
    fn transform_declarations(&mut self, declarations: Vec<Declaration>) -> Result<Vec<Declaration>, String> {
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
    
    fn transform_function(&mut self, mut func: Function) -> Result<Function, String> {
        func.body = self.transform_statements(func.body)?;
        Ok(func)
    }
    
    fn transform_statements(&mut self, statements: Vec<crate::ast::Statement>) -> Result<Vec<crate::ast::Statement>, String> {
        let mut result = Vec::new();
        
        for stmt in statements {
            result.push(self.transform_statement(stmt)?);
        }
        
        Ok(result)
    }
    
    fn transform_statement(&mut self, stmt: crate::ast::Statement) -> Result<crate::ast::Statement, String> {
        match stmt {
            crate::ast::Statement::Expression(expr) => {
                Ok(crate::ast::Statement::Expression(self.transform_expression(expr)?))
            }
            crate::ast::Statement::Return(expr) => {
                Ok(crate::ast::Statement::Return(self.transform_expression(expr)?))
            }
            crate::ast::Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } => {
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
                })
            }
            crate::ast::Statement::VariableAssignment { name, value } => {
                Ok(crate::ast::Statement::VariableAssignment {
                    name,
                    value: self.transform_expression(value)?,
                })
            }
            other => Ok(other),
        }
    }
    
    fn transform_expression(&mut self, expr: Expression) -> Result<Expression, String> {
        match expr {
            Expression::FunctionCall { name, args } => {
                // Check if this is a call to a generic function that has been monomorphized
                let base_name = extract_base_name(&name);
                
                // Transform the arguments first
                let transformed_args: Vec<Expression> = args.into_iter()
                    .map(|arg| self.transform_expression(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                
                // If this is a generic function, we need to determine which instantiation to use
                if self.env.get_generic_function(&base_name).is_some() {
                    // Infer the types of the arguments to determine the instantiation
                    let arg_types: Vec<AstType> = transformed_args.iter()
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
            Expression::BinaryOp { left, op, right } => {
                Ok(Expression::BinaryOp {
                    left: Box::new(self.transform_expression(*left)?),
                    op,
                    right: Box::new(self.transform_expression(*right)?),
                })
            }
            other => Ok(other),
        }
    }
}

fn generate_instantiated_name(base_name: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        return base_name.to_string();
    }
    
    let type_names: Vec<String> = type_args.iter().map(type_to_string).collect();
    format!("{}_{}", base_name, type_names.join("_"))
}

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
        AstType::String => "string".to_string(),
        AstType::Void => "void".to_string(),
        _ => "unknown".to_string(),
    }
}

fn extract_generic_struct_types(name: &str) -> Option<Vec<AstType>> {
    if name.contains('<') && name.contains('>') {
        // TODO: Parse type arguments from struct construction syntax
        None
    } else {
        None
    }
}

fn extract_base_name(name: &str) -> String {
    if let Some(idx) = name.find('<') {
        name[..idx].to_string()
    } else {
        name.to_string()
    }
}