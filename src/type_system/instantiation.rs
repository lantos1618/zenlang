use crate::ast::{AstType, Function, StructDefinition, EnumDefinition, Statement, Expression, TypeParameter};
use super::{TypeEnvironment, TypeSubstitution};

pub struct TypeInstantiator<'a> {
    env: &'a mut TypeEnvironment,
}

impl<'a> TypeInstantiator<'a> {
    pub fn new(env: &'a mut TypeEnvironment) -> Self {
        Self { env }
    }

    pub fn instantiate_function(
        &mut self,
        func: &Function,
        type_args: Vec<AstType>,
    ) -> Result<Function, String> {
        self.env.validate_type_args(&func.type_params, &type_args)?;
        
        let mut substitution = TypeSubstitution::new();
        for (param, arg) in func.type_params.iter().zip(type_args.iter()) {
            substitution.add(param.name.clone(), arg.clone());
        }
        
        let instantiated_name = generate_instantiated_name(&func.name, &type_args);
        
        let instantiated_args: Vec<(String, AstType)> = func.args
            .iter()
            .map(|(name, ty)| (name.clone(), substitution.apply(ty)))
            .collect();
        
        let instantiated_return = substitution.apply(&func.return_type);
        
        let instantiated_body = self.instantiate_statements(&func.body, &substitution)?;
        
        Ok(Function {
            name: instantiated_name,
            type_params: Vec::new(),
            args: instantiated_args,
            return_type: instantiated_return,
            body: instantiated_body,
            is_async: func.is_async,
        })
    }

    pub fn instantiate_struct(
        &mut self,
        struct_def: &StructDefinition,
        type_args: Vec<AstType>,
    ) -> Result<StructDefinition, String> {
        self.env.validate_type_args(&struct_def.type_params, &type_args)?;
        
        let mut substitution = TypeSubstitution::new();
        for (param, arg) in struct_def.type_params.iter().zip(type_args.iter()) {
            substitution.add(param.name.clone(), arg.clone());
        }
        
        let instantiated_name = generate_instantiated_name(&struct_def.name, &type_args);
        
        let instantiated_fields = struct_def.fields
            .iter()
            .map(|field| crate::ast::StructField {
                name: field.name.clone(),
                type_: substitution.apply(&field.type_),
                is_mutable: field.is_mutable,
                default_value: field.default_value.clone(),
            })
            .collect();
        
        let instantiated_methods = struct_def.methods
            .iter()
            .map(|method| self.instantiate_method(method, &substitution))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(StructDefinition {
            name: instantiated_name,
            type_params: Vec::new(),
            fields: instantiated_fields,
            methods: instantiated_methods,
        })
    }

    pub fn instantiate_enum(
        &mut self,
        enum_def: &EnumDefinition,
        type_args: Vec<AstType>,
    ) -> Result<EnumDefinition, String> {
        self.env.validate_type_args(&enum_def.type_params, &type_args)?;
        
        let mut substitution = TypeSubstitution::new();
        for (param, arg) in enum_def.type_params.iter().zip(type_args.iter()) {
            substitution.add(param.name.clone(), arg.clone());
        }
        
        let instantiated_name = generate_instantiated_name(&enum_def.name, &type_args);
        
        let instantiated_variants = enum_def.variants
            .iter()
            .map(|variant| crate::ast::EnumVariant {
                name: variant.name.clone(),
                payload: variant.payload.as_ref().map(|ty| substitution.apply(ty)),
            })
            .collect();
        
        let instantiated_methods = enum_def.methods
            .iter()
            .map(|method| self.instantiate_method(method, &substitution))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(EnumDefinition {
            name: instantiated_name,
            type_params: Vec::new(),
            variants: instantiated_variants,
            methods: instantiated_methods,
        })
    }

    fn instantiate_method(
        &mut self,
        method: &Function,
        substitution: &TypeSubstitution,
    ) -> Result<Function, String> {
        let instantiated_args: Vec<(String, AstType)> = method.args
            .iter()
            .map(|(name, ty)| (name.clone(), substitution.apply(ty)))
            .collect();
        
        let instantiated_return = substitution.apply(&method.return_type);
        let instantiated_body = self.instantiate_statements(&method.body, substitution)?;
        
        Ok(Function {
            name: method.name.clone(),
            type_params: Vec::new(),
            args: instantiated_args,
            return_type: instantiated_return,
            body: instantiated_body,
            is_async: method.is_async,
        })
    }

    fn instantiate_statements(
        &mut self,
        statements: &[Statement],
        substitution: &TypeSubstitution,
    ) -> Result<Vec<Statement>, String> {
        statements.iter()
            .map(|stmt| self.instantiate_statement(stmt, substitution))
            .collect()
    }

    fn instantiate_statement(
        &mut self,
        statement: &Statement,
        substitution: &TypeSubstitution,
    ) -> Result<Statement, String> {
        match statement {
            Statement::VariableDeclaration { name, type_, initializer, is_mutable, declaration_type } => {
                Ok(Statement::VariableDeclaration {
                    name: name.clone(),
                    type_: type_.as_ref().map(|t| substitution.apply(t)),
                    initializer: initializer.as_ref().map(|e| self.instantiate_expression(e, substitution)),
                    is_mutable: *is_mutable,
                    declaration_type: declaration_type.clone(),
                })
            }
            Statement::Expression(expr) => {
                Ok(Statement::Expression(self.instantiate_expression(expr, substitution)))
            }
            Statement::Return(expr) => {
                Ok(Statement::Return(self.instantiate_expression(expr, substitution)))
            }
            Statement::Loop { condition, label, body } => {
                Ok(Statement::Loop {
                    condition: condition.as_ref().map(|c| self.instantiate_expression(c, substitution)),
                    label: label.clone(),
                    body: self.instantiate_statements(body, substitution)?,
                })
            }
            _ => Ok(statement.clone()),
        }
    }

    fn instantiate_expression(&mut self, expr: &Expression, substitution: &TypeSubstitution) -> Expression {
        match expr {
            Expression::FunctionCall { name, args } => {
                Expression::FunctionCall {
                    name: name.clone(),
                    args: args.iter().map(|a| self.instantiate_expression(a, substitution)).collect(),
                }
            }
            Expression::BinaryOp { left, op, right } => {
                Expression::BinaryOp {
                    left: Box::new(self.instantiate_expression(left, substitution)),
                    op: op.clone(),
                    right: Box::new(self.instantiate_expression(right, substitution)),
                }
            }
            Expression::StructLiteral { name, fields } => {
                Expression::StructLiteral {
                    name: name.clone(),
                    fields: fields.iter().map(|(n, e)| (n.clone(), self.instantiate_expression(e, substitution))).collect(),
                }
            }
            Expression::MemberAccess { object, member } => {
                Expression::MemberAccess {
                    object: Box::new(self.instantiate_expression(object, substitution)),
                    member: member.clone(),
                }
            }
            Expression::ArrayLiteral(items) => {
                Expression::ArrayLiteral(items.iter().map(|e| self.instantiate_expression(e, substitution)).collect())
            }
            Expression::Dereference(expr) => {
                Expression::Dereference(Box::new(self.instantiate_expression(expr, substitution)))
            }
            Expression::AddressOf(expr) => {
                Expression::AddressOf(Box::new(self.instantiate_expression(expr, substitution)))
            }
            Expression::Conditional { scrutinee, arms } => {
                Expression::Conditional {
                    scrutinee: Box::new(self.instantiate_expression(scrutinee, substitution)),
                    arms: arms.iter().map(|arm| crate::ast::ConditionalArm {
                        pattern: arm.pattern.clone(),
                        guard: arm.guard.as_ref().map(|g| self.instantiate_expression(g, substitution)),
                        body: self.instantiate_expression(&arm.body, substitution),
                    }).collect(),
                }
            }
            _ => expr.clone(),
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
        AstType::Pointer(inner) => format!("ptr_{}", type_to_string(inner)),
        AstType::Array(inner) => format!("arr_{}", type_to_string(inner)),
        AstType::Generic { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args: Vec<String> = type_args.iter().map(type_to_string).collect();
                format!("{}_{}", name, args.join("_"))
            }
        }
        _ => "unknown".to_string(),
    }
}