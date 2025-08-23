use crate::ast::{Program, Declaration, Expression, AstType, Function};
use super::{TypeEnvironment, TypeInstantiator};
use std::collections::{HashMap, HashSet};

pub struct Monomorphizer {
    env: TypeEnvironment,
    instantiated_functions: HashMap<String, Function>,
    pending_instantiations: Vec<(String, Vec<AstType>)>,
    processed_instantiations: HashSet<(String, Vec<AstType>)>,
}

impl Monomorphizer {
    pub fn new() -> Self {
        Self {
            env: TypeEnvironment::new(),
            instantiated_functions: HashMap::new(),
            pending_instantiations: Vec::new(),
            processed_instantiations: HashSet::new(),
        }
    }

    pub fn monomorphize_program(&mut self, program: &Program) -> Result<Program, String> {
        let mut declarations = Vec::new();
        
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
        
        Ok(Program { declarations })
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
                if let Some(type_args) = extract_generic_call_types(name, args) {
                    let base_name = extract_base_name(name);
                    if self.env.get_generic_function(&base_name).is_some() {
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
}

fn extract_generic_call_types(name: &str, _args: &[Expression]) -> Option<Vec<AstType>> {
    if name.contains('<') && name.contains('>') {
        // TODO: Parse type arguments from function call syntax
        None
    } else {
        None
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