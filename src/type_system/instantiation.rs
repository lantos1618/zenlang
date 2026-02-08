use super::{generate_instantiated_name, TypeEnvironment, TypeSubstitution};
use crate::ast::{AstType, EnumDefinition, Expression, Function, Statement, StructDefinition};
use crate::error::CompileError;

pub struct TypeInstantiator<'a, 'prog> {
    env: &'a mut TypeEnvironment<'prog>,
}

impl<'a, 'prog> TypeInstantiator<'a, 'prog> {
    pub fn new(env: &'a mut TypeEnvironment<'prog>) -> Self {
        Self { env }
    }

    pub fn instantiate_function(
        &mut self,
        func: &Function,
        type_args: Vec<AstType>,
    ) -> Result<Function, CompileError> {
        self.env.validate_type_args(&func.type_params, &type_args)?;

        let mut substitution = TypeSubstitution::new();
        for (param, arg) in func.type_params.iter().zip(type_args.iter()) {
            substitution.add(param.name.clone(), arg.clone());
        }

        let instantiated_name = generate_instantiated_name(&func.name, &type_args);

        let instantiated_args: Vec<(String, AstType)> = func
            .args
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
            is_varargs: func.is_varargs,
            is_public: func.is_public,
        })
    }

    pub fn instantiate_struct(
        &mut self,
        struct_def: &StructDefinition,
        type_args: Vec<AstType>,
    ) -> Result<StructDefinition, CompileError> {
        self.env
            .validate_type_args(&struct_def.type_params, &type_args)?;

        let mut substitution = TypeSubstitution::new();
        for (param, arg) in struct_def.type_params.iter().zip(type_args.iter()) {
            substitution.add(param.name.clone(), arg.clone());
        }

        let instantiated_name = generate_instantiated_name(&struct_def.name, &type_args);

        let instantiated_fields = struct_def
            .fields
            .iter()
            .map(|field| crate::ast::StructField {
                name: field.name.clone(),
                type_: substitution.apply(&field.type_),
                is_mutable: field.is_mutable,
                default_value: field.default_value.clone(),
            })
            .collect();

        let instantiated_methods = struct_def
            .methods
            .iter()
            .map(|method| self.instantiate_method(method, &substitution))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(StructDefinition {
            name: instantiated_name,
            type_params: Vec::new(),
            fields: instantiated_fields,
            methods: instantiated_methods,
            span: struct_def.span.clone(), // Preserve original span
        })
    }

    pub fn instantiate_enum(
        &mut self,
        enum_def: &EnumDefinition,
        type_args: Vec<AstType>,
    ) -> Result<EnumDefinition, CompileError> {
        self.env
            .validate_type_args(&enum_def.type_params, &type_args)?;

        let mut substitution = TypeSubstitution::new();
        for (param, arg) in enum_def.type_params.iter().zip(type_args.iter()) {
            substitution.add(param.name.clone(), arg.clone());
        }

        let instantiated_name = generate_instantiated_name(&enum_def.name, &type_args);

        let instantiated_variants = enum_def
            .variants
            .iter()
            .map(|variant| crate::ast::EnumVariant {
                name: variant.name.clone(),
                payload: variant.payload.as_ref().map(|ty| substitution.apply(ty)),
            })
            .collect();

        let instantiated_methods = enum_def
            .methods
            .iter()
            .map(|method| self.instantiate_method(method, &substitution))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(EnumDefinition {
            name: instantiated_name,
            type_params: Vec::new(),
            variants: instantiated_variants,
            methods: instantiated_methods,
            required_traits: enum_def.required_traits.clone(),
            span: enum_def.span.clone(), // Preserve original span
        })
    }

    fn instantiate_method(
        &mut self,
        method: &Function,
        substitution: &TypeSubstitution,
    ) -> Result<Function, CompileError> {
        let instantiated_args: Vec<(String, AstType)> = method
            .args
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
            is_varargs: method.is_varargs,
            is_public: method.is_public,
        })
    }

    fn instantiate_statements(
        &mut self,
        statements: &[Statement],
        substitution: &TypeSubstitution,
    ) -> Result<Vec<Statement>, CompileError> {
        statements
            .iter()
            .map(|stmt| self.instantiate_statement(stmt, substitution))
            .collect()
    }

    fn instantiate_statement(
        &mut self,
        statement: &Statement,
        substitution: &TypeSubstitution,
    ) -> Result<Statement, CompileError> {
        match statement {
            Statement::VariableDeclaration {
                name,
                type_,
                initializer,
                is_mutable,
                declaration_type,
                span,
            } => Ok(Statement::VariableDeclaration {
                name: name.clone(),
                type_: type_.as_ref().map(|t| substitution.apply(t)),
                initializer: initializer
                    .as_ref()
                    .map(|e| self.instantiate_expression(e, substitution)),
                is_mutable: *is_mutable,
                declaration_type: declaration_type.clone(),
                span: span.clone(),
            }),
            Statement::Expression { expr, span } => Ok(Statement::Expression {
                expr: self.instantiate_expression(expr, substitution),
                span: span.clone(),
            }),
            Statement::Return { expr, span } => Ok(Statement::Return {
                expr: self.instantiate_expression(expr, substitution),
                span: span.clone(),
            }),
            Statement::Loop {
                kind,
                label,
                body,
                span,
            } => {
                use crate::ast::LoopKind;
                let new_kind = match kind {
                    LoopKind::Infinite => LoopKind::Infinite,
                    LoopKind::Condition(expr) => {
                        LoopKind::Condition(self.instantiate_expression(expr, substitution))
                    }
                };
                Ok(Statement::Loop {
                    kind: new_kind,
                    label: label.clone(),
                    body: self.instantiate_statements(body, substitution)?,
                    span: span.clone(),
                })
            }
            _ => Ok(statement.clone()),
        }
    }

    // Note: substitution is passed through for future use in type variable replacement
    #[allow(clippy::only_used_in_recursion)]
    fn instantiate_expression(
        &mut self,
        expr: &Expression,
        substitution: &TypeSubstitution,
    ) -> Expression {
        match expr {
            Expression::FunctionCall {
                name,
                type_args,
                args,
                span,
            } => Expression::FunctionCall {
                name: name.clone(),
                type_args: type_args.clone(),
                args: args
                    .iter()
                    .map(|a| self.instantiate_expression(a, substitution))
                    .collect(),
                span: span.clone(),
            },
            Expression::BinaryOp { left, op, right } => Expression::BinaryOp {
                left: Box::new(self.instantiate_expression(left, substitution)),
                op: op.clone(),
                right: Box::new(self.instantiate_expression(right, substitution)),
            },
            Expression::StructLiteral { name, fields } => Expression::StructLiteral {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|(n, e)| (n.clone(), self.instantiate_expression(e, substitution)))
                    .collect(),
            },
            Expression::MemberAccess { object, member } => Expression::MemberAccess {
                object: Box::new(self.instantiate_expression(object, substitution)),
                member: member.clone(),
            },
            Expression::ArrayLiteral(items) => Expression::ArrayLiteral(
                items
                    .iter()
                    .map(|e| self.instantiate_expression(e, substitution))
                    .collect(),
            ),
            Expression::Dereference(expr) => {
                Expression::Dereference(Box::new(self.instantiate_expression(expr, substitution)))
            }
            Expression::AddressOf(expr) => {
                Expression::AddressOf(Box::new(self.instantiate_expression(expr, substitution)))
            }
            Expression::QuestionMatch { scrutinee, arms } => Expression::QuestionMatch {
                scrutinee: Box::new(self.instantiate_expression(scrutinee, substitution)),
                arms: arms
                    .iter()
                    .map(|arm| crate::ast::MatchArm {
                        pattern: arm.pattern.clone(),
                        guard: arm
                            .guard
                            .as_ref()
                            .map(|g| self.instantiate_expression(g, substitution)),
                        body: self.instantiate_expression(&arm.body, substitution),
                    })
                    .collect(),
            },
            _ => expr.clone(),
        }
    }
}

// generate_instantiated_name and type_to_string moved to mod.rs
