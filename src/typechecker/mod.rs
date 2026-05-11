//! Typechecker — transforms untyped AST → TypedProgram.
//!
//! Pipeline:
//! 1. **Collect**: Register all struct/enum/function/behavior signatures
//! 2. **Resolve**: Resolve type references (Named("Foo") → Struct fields)
//! 3. **Check**: Type-check function bodies, produce TypedExpression
//!
//! The typechecker NEVER defaults unknown types to I32. If a type can't be
//! resolved, it's an error.

mod closures;
mod expressions;
mod monomorphize;
mod patterns;
mod resolve;
mod statements;

use std::collections::{HashMap, HashSet};

use crate::ast::typed::*;
use crate::ast::{self, AstType, Declaration};
use crate::error::Diagnostic;

// ── Type Environment ──────────────────────────────────────────────

/// Information about a struct type.
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, AstType)>,
    pub type_params: Vec<String>,
}

/// Information about an enum type.
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<(String, Option<AstType>)>,
    pub type_params: Vec<String>,
}

/// Information about a function signature.
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub type_params: Vec<String>,
}

/// Scope for variable types.
#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, Type>,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

// ── TypeChecker ───────────────────────────────────────────────────

pub struct TypeChecker {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    methods: HashMap<String, FuncInfo>, // key: "TypeName.method_name"
    imports: HashSet<String>,           // imported module names (e.g. "io")
    scopes: Vec<Scope>,
    diagnostics: Vec<Diagnostic>,
    current_return_type: Option<Type>,
    current_self_type: Option<Type>,
    pending_defers: Vec<TypedExpression>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            methods: HashMap::new(),
            imports: HashSet::new(),
            scopes: vec![Scope::new()], // global scope
            diagnostics: Vec::new(),
            current_return_type: None,
            current_self_type: None,
            pending_defers: Vec::new(),
        }
    }

    /// Type-check a program and produce a TypedProgram.
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 1: Collect type definitions and function signatures
        self.collect_declarations(&program.declarations);

        // Phase 2: Check function bodies and produce typed AST
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut globals = Vec::new();
        let mut entry_point = None;

        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if name == "main" {
                        entry_point = Some(name.clone());
                    }
                    match self.check_function(name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    let full_name = format!("{}.{}", type_name, method_name);
                    // Set Self type for method body
                    self.current_self_type =
                        Some(self.resolve_type(&AstType::Named(type_name.clone())));
                    match self.check_function(&full_name, params, return_type, body, span) {
                        Ok(func) => functions.push(func),
                        Err(d) => self.diagnostics.push(d),
                    }
                    self.current_self_type = None;
                }
                Declaration::Struct {
                    name, fields, span, ..
                } => {
                    let resolved_fields: Vec<(String, Type)> = fields
                        .iter()
                        .map(|f| (f.name.clone(), self.resolve_type(&f.ty)))
                        .collect();
                    types.push(TypedTypeDef {
                        name: name.clone(),
                        kind: TypeDefKind::Struct {
                            fields: resolved_fields,
                        },
                        methods: Vec::new(),
                        span: *span,
                    });
                }
                Declaration::Enum {
                    name,
                    variants,
                    span,
                    ..
                } => {
                    let typed_variants: Vec<TypedVariant> = variants
                        .iter()
                        .enumerate()
                        .map(|(i, v)| TypedVariant {
                            name: v.name.clone(),
                            tag: i as u32,
                            payload: v
                                .payload
                                .as_ref()
                                .map(|ty| vec![("payload".to_string(), self.resolve_type(ty))]),
                        })
                        .collect();
                    types.push(TypedTypeDef {
                        name: name.clone(),
                        kind: TypeDefKind::Enum {
                            variants: typed_variants,
                        },
                        methods: Vec::new(),
                        span: *span,
                    });
                }
                Declaration::TopLevelExpr { expr, span } => {
                    // Top-level expressions like main() call
                    match self.check_expr(expr) {
                        Ok(typed_expr) => {
                            globals.push(TypedGlobal {
                                name: "__top_level__".into(),
                                ty: typed_expr.ty.clone(),
                                value: typed_expr,
                                mutable: false,
                                span: *span,
                            });
                        }
                        Err(d) => self.diagnostics.push(d),
                    }
                }
                Declaration::Import { .. } => {
                    // Imports are handled by the module system, not the typechecker
                }
                _ => {}
            }
        }

        let errors: Vec<_> = self
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .cloned()
            .collect();
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(TypedProgram {
            functions,
            types,
            globals,
            entry_point,
        })
    }

    /// Get all diagnostics (errors + warnings).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    // ── Phase 1: Collect ──────────────────────────────────────────

    fn collect_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    ..
                } => {
                    self.structs.insert(
                        name.clone(),
                        StructInfo {
                            name: name.clone(),
                            fields: fields
                                .iter()
                                .map(|f| (f.name.clone(), f.ty.clone()))
                                .collect(),
                            type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                        },
                    );
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    ..
                } => {
                    self.enums.insert(
                        name.clone(),
                        EnumInfo {
                            name: name.clone(),
                            variants: variants
                                .iter()
                                .map(|v| (v.name.clone(), v.payload.clone()))
                                .collect(),
                            type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                        },
                    );
                }
                Declaration::Import { names, .. } => {
                    for name in names {
                        self.imports.insert(name.clone());
                    }
                }
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    ..
                } => {
                    let ret = return_type.clone().unwrap_or(AstType::Void);
                    self.functions.insert(
                        name.clone(),
                        FuncInfo {
                            name: name.clone(),
                            params: params
                                .iter()
                                .map(|p| (p.name.clone(), p.ty.clone()))
                                .collect(),
                            return_type: ret,
                            type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                        },
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    ..
                } => {
                    let key = format!("{}.{}", type_name, method_name);
                    let ret = return_type.clone().unwrap_or(AstType::Void);
                    self.methods.insert(
                        key.clone(),
                        FuncInfo {
                            name: key,
                            params: params
                                .iter()
                                .map(|p| (p.name.clone(), p.ty.clone()))
                                .collect(),
                            return_type: ret,
                            type_params: type_params.iter().map(|tp| tp.name.clone()).collect(),
                        },
                    );
                }
                _ => {}
            }
        }
    }

    // ── Scope Management ──────────────────────────────────────────

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn define_var(&mut self, name: &str, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(name.to_string(), ty);
        }
    }

    pub(crate) fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.vars.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    pub(crate) fn is_import(&self, name: &str) -> bool {
        self.imports.contains(name)
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::declarations::StructField;
    use crate::ast::expressions::BinaryOp;
    use crate::error::Span;

    #[test]
    fn resolve_primitive_types() {
        let tc = TypeChecker::new();
        assert_eq!(tc.resolve_type(&AstType::I32), Type::I32);
        assert_eq!(tc.resolve_type(&AstType::F64), Type::F64);
        assert_eq!(tc.resolve_type(&AstType::Bool), Type::Bool);
        assert_eq!(tc.resolve_type(&AstType::Void), Type::Void);
        assert_eq!(tc.resolve_type(&AstType::Str), Type::Str);
    }

    #[test]
    fn resolve_pointer_types() {
        let tc = TypeChecker::new();
        assert_eq!(
            tc.resolve_type(&AstType::Ptr(Box::new(AstType::I32))),
            Type::Ptr(Box::new(Type::I32))
        );
    }

    #[test]
    fn scope_variable_lookup() {
        let mut tc = TypeChecker::new();
        tc.define_var("x", Type::I32);
        assert_eq!(tc.lookup_var("x"), Some(Type::I32));

        tc.push_scope();
        tc.define_var("y", Type::Bool);
        assert_eq!(tc.lookup_var("y"), Some(Type::Bool));
        assert_eq!(tc.lookup_var("x"), Some(Type::I32)); // parent scope

        tc.pop_scope();
        assert_eq!(tc.lookup_var("y"), None); // out of scope
    }

    #[test]
    fn collect_struct_info() {
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Struct {
            name: "Point".into(),
            type_params: Vec::new(),
            fields: vec![
                StructField {
                    name: "x".into(),
                    ty: AstType::F64,
                    default: None,
                    mutable: false,
                    span: Span::dummy(),
                },
                StructField {
                    name: "y".into(),
                    ty: AstType::F64,
                    default: None,
                    mutable: false,
                    span: Span::dummy(),
                },
            ],
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        assert!(tc.structs.contains_key("Point"));
        assert_eq!(tc.structs["Point"].fields.len(), 2);
    }

    #[test]
    fn binary_op_types() {
        let tc = TypeChecker::new();
        assert_eq!(
            tc.check_binary_op(BinaryOp::Add, &Type::I32, &Type::I32, &Span::dummy())
                .unwrap(),
            Type::I32
        );
        assert_eq!(
            tc.check_binary_op(BinaryOp::Eq, &Type::I32, &Type::I32, &Span::dummy())
                .unwrap(),
            Type::Bool
        );
        assert_eq!(
            tc.check_binary_op(BinaryOp::And, &Type::Bool, &Type::Bool, &Span::dummy())
                .unwrap(),
            Type::Bool
        );
    }

    #[test]
    fn binary_op_type_mismatch() {
        let tc = TypeChecker::new();
        // Arithmetic on non-numeric type
        assert!(tc
            .check_binary_op(BinaryOp::Add, &Type::I32, &Type::Str, &Span::dummy())
            .is_err());
        assert!(tc
            .check_binary_op(BinaryOp::Add, &Type::Bool, &Type::I32, &Span::dummy())
            .is_err());
        // Logical op on non-bool
        assert!(tc
            .check_binary_op(BinaryOp::And, &Type::I32, &Type::Bool, &Span::dummy())
            .is_err());
        // Unknown is permissive (error recovery)
        assert!(tc
            .check_binary_op(BinaryOp::Add, &Type::Unknown, &Type::Str, &Span::dummy())
            .is_ok());
    }

    #[test]
    fn unknown_function_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "nonexistent".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: Vec::new(),
                            span: Span::dummy(),
                        },
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        let result = tc.check_program(&program);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|d| d.message.contains("undefined function")));
    }

    #[test]
    fn return_type_mismatch_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "foo".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::I32),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: Some(Box::new(Expression::Return {
                        value: Some(Box::new(Expression::StringLiteral {
                            value: "hello".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        let result = tc.check_program(&program);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|d| d.message.contains("return type mismatch")));
    }

    #[test]
    fn function_call_wrong_arity_is_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Function {
                    name: "add".into(),
                    type_params: Vec::new(),
                    params: vec![
                        ast::Param {
                            name: "a".into(),
                            ty: AstType::I32,
                            mutable: false,
                            span: Span::dummy(),
                        },
                        ast::Param {
                            name: "b".into(),
                            ty: AstType::I32,
                            mutable: false,
                            span: Span::dummy(),
                        },
                    ],
                    return_type: Some(AstType::I32),
                    body: Expression::Block {
                        statements: Vec::new(),
                        expr: Some(Box::new(Expression::Return {
                            value: Some(Box::new(Expression::Identifier {
                                name: "a".into(),
                                span: Span::dummy(),
                            })),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![ast::Statement::Expression {
                            expr: Expression::FunctionCall {
                                name: "add".into(),
                                module: None,
                                type_args: Vec::new(),
                                args: vec![Expression::IntLiteral {
                                    value: 1,
                                    span: Span::dummy(),
                                }],
                                span: Span::dummy(),
                            },
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("wrong arity should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("function `add` expects 2 arguments, found 1")),
            "expected arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn function_call_argument_type_mismatch_is_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Function {
                    name: "takes_i32".into(),
                    type_params: Vec::new(),
                    params: vec![ast::Param {
                        name: "value".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    }],
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: Vec::new(),
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![ast::Statement::Expression {
                            expr: Expression::FunctionCall {
                                name: "takes_i32".into(),
                                module: None,
                                type_args: Vec::new(),
                                args: vec![Expression::StringLiteral {
                                    value: "bad".into(),
                                    span: Span::dummy(),
                                }],
                                span: Span::dummy(),
                            },
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("argument type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("argument 1 for `takes_i32` expects `i32`, found `str`")),
            "expected argument type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn struct_literal_missing_field_is_error() {
        use crate::ast::declarations::StructField;
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Struct {
                    name: "Point".into(),
                    type_params: Vec::new(),
                    fields: vec![
                        StructField {
                            name: "x".into(),
                            ty: AstType::I32,
                            default: None,
                            mutable: false,
                            span: Span::dummy(),
                        },
                        StructField {
                            name: "y".into(),
                            ty: AstType::I32,
                            default: None,
                            mutable: false,
                            span: Span::dummy(),
                        },
                    ],
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![Statement::VarDecl {
                            name: "p".into(),
                            ty: None,
                            value: Expression::StructLiteral {
                                name: "Point".into(),
                                type_args: Vec::new(),
                                fields: vec![(
                                    "x".into(),
                                    Expression::IntLiteral {
                                        value: 1,
                                        span: Span::dummy(),
                                    },
                                )],
                                span: Span::dummy(),
                            },
                            mutable: false,
                            constant: false,
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("missing struct field should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("missing field `y` for struct `Point`")),
            "expected missing field diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn struct_literal_field_type_mismatch_is_error() {
        use crate::ast::declarations::StructField;
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Struct {
                    name: "Point".into(),
                    type_params: Vec::new(),
                    fields: vec![StructField {
                        name: "x".into(),
                        ty: AstType::I32,
                        default: None,
                        mutable: false,
                        span: Span::dummy(),
                    }],
                    public: false,
                    span: Span::dummy(),
                },
                Declaration::Function {
                    name: "main".into(),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    return_type: Some(AstType::Void),
                    body: Expression::Block {
                        statements: vec![Statement::VarDecl {
                            name: "p".into(),
                            ty: None,
                            value: Expression::StructLiteral {
                                name: "Point".into(),
                                type_args: Vec::new(),
                                fields: vec![(
                                    "x".into(),
                                    Expression::StringLiteral {
                                        value: "bad".into(),
                                        span: Span::dummy(),
                                    },
                                )],
                                span: Span::dummy(),
                            },
                            mutable: false,
                            constant: false,
                            span: Span::dummy(),
                        }],
                        expr: None,
                        span: Span::dummy(),
                    },
                    public: false,
                    span: Span::dummy(),
                },
            ],
            file_id: 0,
        };

        let errors = tc
            .check_program(&program)
            .expect_err("struct field type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("field `x` for struct `Point` expects `i32`, found `str`")),
            "expected field type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn types_compatible_basics() {
        let tc = TypeChecker::new();
        // Same types
        assert!(tc.types_compatible(&Type::I32, &Type::I32));
        // Integer coercion
        assert!(tc.types_compatible(&Type::I64, &Type::I32));
        // Float coercion
        assert!(tc.types_compatible(&Type::F32, &Type::F64));
        // Unknown is permissive
        assert!(tc.types_compatible(&Type::I32, &Type::Unknown));
        // Named types are nominal and do not match unrelated concrete types.
        assert!(tc.types_compatible(&Type::Named("UserId".into()), &Type::Named("UserId".into())));
        assert!(!tc.types_compatible(
            &Type::Named("UserId".into()),
            &Type::Named("OrderId".into())
        ));
        assert!(!tc.types_compatible(&Type::Str, &Type::Named("StaticString".into())));
        // Clear mismatch
        assert!(!tc.types_compatible(&Type::I32, &Type::Str));
        assert!(!tc.types_compatible(&Type::Bool, &Type::I32));
    }

    #[test]
    fn literal_coercion_in_var_decl() {
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![Statement::VarDecl {
                        name: "x".into(),
                        ty: Some(AstType::I64),
                        value: Expression::IntLiteral {
                            value: 42,
                            span: Span::dummy(),
                        },
                        mutable: false,
                        constant: false,
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            }],
            file_id: 0,
        };
        let result = tc.check_program(&program).unwrap();
        // The variable should have type I64 (coerced from I32 literal)
        let body = &result.functions[0].body;
        match &body.statements[0].kind {
            TypedStatementKind::VarDecl { ty, .. } => assert_eq!(*ty, Type::I64),
            _ => panic!("expected VarDecl"),
        }
    }

    #[test]
    fn resolve_string_type() {
        let tc = TypeChecker::new();
        // "String" as a named type should resolve to Type::String
        assert_eq!(
            tc.resolve_type(&AstType::Named("String".into())),
            Type::String
        );
    }

    #[test]
    fn resolve_slice_type() {
        let tc = TypeChecker::new();
        assert_eq!(
            tc.resolve_type(&AstType::Slice(Box::new(AstType::I32))),
            Type::Slice(Box::new(Type::I32))
        );
    }

    #[test]
    fn infer_type_args_basic() {
        let tc = TypeChecker::new();
        // Generic function: identity<T>(x: T) -> T
        let type_params = vec!["T".to_string()];
        let params = vec![("x".to_string(), AstType::Named("T".into()))];
        let arg_types = vec![Type::I32];
        let subs = tc.infer_type_args(&type_params, &params, &arg_types);
        assert_eq!(subs.get("T"), Some(&Type::I32));
    }

    #[test]
    fn substitute_type_basic() {
        let tc = TypeChecker::new();
        let mut subs = HashMap::new();
        subs.insert("T".to_string(), Type::I32);
        // T → I32
        assert_eq!(
            tc.substitute_type(&AstType::Named("T".into()), &subs),
            Type::I32
        );
        // Ptr<T> → Ptr<I32>
        assert_eq!(
            tc.substitute_type(&AstType::Ptr(Box::new(AstType::Named("T".into()))), &subs),
            Type::Ptr(Box::new(Type::I32))
        );
        // Non-generic type unchanged
        assert_eq!(tc.substitute_type(&AstType::Bool, &subs), Type::Bool);
    }

    #[test]
    fn generic_function_collection() {
        use crate::ast::Expression;
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Function {
            name: "identity".into(),
            type_params: vec![crate::ast::declarations::TypeParam {
                name: "T".into(),
                constraint: None,
                span: Span::dummy(),
            }],
            params: vec![crate::ast::Param {
                name: "x".into(),
                ty: AstType::Named("T".into()),
                mutable: false,
                span: Span::dummy(),
            }],
            return_type: Some(AstType::Named("T".into())),
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        let info = tc.functions.get("identity").unwrap();
        assert_eq!(info.type_params, vec!["T".to_string()]);
    }

    #[test]
    fn func_info_non_generic_has_empty_type_params() {
        use crate::ast::Expression;
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Function {
            name: "add".into(),
            type_params: Vec::new(),
            params: vec![
                crate::ast::Param {
                    name: "a".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                },
                crate::ast::Param {
                    name: "b".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                },
            ],
            return_type: Some(AstType::I32),
            body: Expression::Block {
                statements: Vec::new(),
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        let info = tc.functions.get("add").unwrap();
        assert!(info.type_params.is_empty());
    }
}
