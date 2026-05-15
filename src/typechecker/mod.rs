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

use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::typed::*;
use crate::ast::{
    self, AstType, BehaviorMethod, Declaration, EnumVariant, Expression, Param, StructField,
};
use crate::error::{Diagnostic, Span};
use crate::module_system::{ResolvedModule, ResolvedModuleGraph};
use crate::resolver::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace, Symbol,
    SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

// ── Type Environment ──────────────────────────────────────────────

/// Information about a struct type.
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, AstType)>,
    pub field_defaults: HashMap<String, Expression>,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
}

/// Information about an enum type.
#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<(String, Option<AstType>)>,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
}

/// Information about a function signature.
#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorBound {
    pub behavior: String,
    pub type_args: Vec<AstType>,
}

#[derive(Debug, Clone)]
pub struct BehaviorInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub type_param_bounds: HashMap<String, BehaviorBound>,
    pub methods: Vec<ast::BehaviorMethod>,
}

#[derive(Debug, Clone)]
pub(crate) struct GenericFunctionTemplate {
    pub type_params: Vec<String>,
    pub params: Vec<Param>,
    pub return_type: Option<AstType>,
    pub body: Expression,
    pub span: Span,
    pub dependency_structs: HashMap<String, StructInfo>,
    pub dependency_enums: HashMap<String, EnumInfo>,
    pub dependency_functions: HashMap<String, FuncInfo>,
    pub dependency_generic_functions: HashMap<String, GenericFunctionTemplate>,
    pub dependency_methods: HashMap<String, FuncInfo>,
    pub dependency_generic_methods: HashMap<String, GenericFunctionTemplate>,
}

impl GenericFunctionTemplate {
    fn new(
        type_params: Vec<String>,
        params: Vec<Param>,
        return_type: Option<AstType>,
        body: Expression,
        span: Span,
    ) -> Self {
        Self {
            type_params,
            params,
            return_type,
            body,
            span,
            dependency_structs: HashMap::new(),
            dependency_enums: HashMap::new(),
            dependency_functions: HashMap::new(),
            dependency_generic_functions: HashMap::new(),
            dependency_methods: HashMap::new(),
            dependency_generic_methods: HashMap::new(),
        }
    }

    fn with_dependencies(
        mut self,
        dependency_structs: HashMap<String, StructInfo>,
        dependency_enums: HashMap<String, EnumInfo>,
        dependency_functions: HashMap<String, FuncInfo>,
        dependency_generic_functions: HashMap<String, GenericFunctionTemplate>,
        dependency_methods: HashMap<String, FuncInfo>,
        dependency_generic_methods: HashMap<String, GenericFunctionTemplate>,
    ) -> Self {
        self.dependency_structs = dependency_structs;
        self.dependency_enums = dependency_enums;
        self.dependency_functions = dependency_functions;
        self.dependency_generic_functions = dependency_generic_functions;
        self.dependency_methods = dependency_methods;
        self.dependency_generic_methods = dependency_generic_methods;
        self
    }

    fn with_source_dependencies(self, dependencies: SourceModuleDependencies) -> Self {
        self.with_dependencies(
            dependencies.structs,
            dependencies.enums,
            dependencies.functions,
            dependencies.generic_functions,
            dependencies.methods,
            dependencies.generic_methods,
        )
    }

    fn attach_source_dependencies(&mut self, dependencies: SourceModuleDependencies) {
        self.dependency_structs = dependencies.structs;
        self.dependency_enums = dependencies.enums;
        self.dependency_functions = dependencies.functions;
        self.dependency_generic_functions = dependencies.generic_functions;
        self.dependency_methods = dependencies.methods;
        self.dependency_generic_methods = dependencies.generic_methods;
    }
}

pub(crate) type TemplateStructDependencyState = Vec<(String, Option<StructInfo>)>;
pub(crate) type TemplateEnumDependencyState = Vec<(String, Option<EnumInfo>)>;
pub(crate) type TemplateFunctionDependencyState = Vec<(String, Option<FuncInfo>)>;
pub(crate) type TemplateGenericDependencyState = Vec<(String, Option<GenericFunctionTemplate>)>;
pub(crate) type TemplateMethodDependencyState = Vec<(String, Option<FuncInfo>)>;
pub(crate) type TemplateGenericMethodDependencyState =
    Vec<(String, Option<GenericFunctionTemplate>)>;

pub(crate) struct TemplateDependencyState {
    structs: TemplateStructDependencyState,
    enums: TemplateEnumDependencyState,
    functions: TemplateFunctionDependencyState,
    generic_functions: TemplateGenericDependencyState,
    methods: TemplateMethodDependencyState,
    generic_methods: TemplateGenericMethodDependencyState,
}

struct DefaultBehaviorMethod {
    name: String,
    params: Vec<Param>,
    return_type: Option<AstType>,
    body: Expression,
    span: Span,
}

struct ExpectedValueSignature {
    params: Vec<ExpectedParameter>,
    return_type: ExpectedReturnMetadata,
    type_params: Vec<ExpectedTypeParameter>,
}

impl ExpectedValueSignature {
    fn new(
        params: &[Param],
        return_type: &Option<AstType>,
        type_params: &[ast::TypeParam],
    ) -> Self {
        Self {
            params: expected_parameter_metadata(params),
            return_type: expected_return_metadata(return_type),
            type_params: expected_type_parameter_metadata(type_params),
        }
    }
}

struct ExpectedValueSymbol {
    signature: ExpectedValueSignature,
    is_public: bool,
}

impl ExpectedValueSymbol {
    fn new(
        params: &[Param],
        return_type: &Option<AstType>,
        type_params: &[ast::TypeParam],
        is_public: bool,
    ) -> Self {
        Self {
            signature: ExpectedValueSignature::new(params, return_type, type_params),
            is_public,
        }
    }
}

struct ExpectedParameter {
    name: String,
    typed: AstType,
    display: String,
}

impl ExpectedParameter {
    fn new(name: &str, ty: &AstType) -> Self {
        Self {
            name: name.to_string(),
            typed: ty.clone(),
            display: ty.display_name(),
        }
    }
}

struct ExpectedParameterMetadata {
    count: usize,
    names: Vec<String>,
    display_types: Vec<String>,
    typed_types: Vec<AstType>,
}

#[derive(Clone, Copy)]
struct ValueParameterValidation {
    name_code: &'static str,
    display_type_code: &'static str,
    typed_type_code: &'static str,
}

impl ValueParameterValidation {
    fn resolver_codes() -> Self {
        Self {
            name_code: "E0223",
            display_type_code: "E0216",
            typed_type_code: "E0356",
        }
    }

    fn name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has parameter names '{actual}', expected '{expected}'"
        )
    }

    fn display_type_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has parameter types '{actual}', expected '{expected}'"
        )
    }

    fn typed_type_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has typed parameter types '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedParameterMetadata {
    fn from_parameters(parameters: &[ExpectedParameter]) -> Self {
        Self {
            count: parameters.len(),
            names: parameters.iter().map(|param| param.name.clone()).collect(),
            display_types: parameters
                .iter()
                .map(|param| param.display.clone())
                .collect(),
            typed_types: parameters.iter().map(|param| param.typed.clone()).collect(),
        }
    }
}

struct ExpectedReturnMetadata {
    typed: AstType,
    display: String,
}

impl ExpectedReturnMetadata {
    fn new(return_type: &Option<AstType>) -> Self {
        let typed = return_type.clone().unwrap_or(AstType::Void);
        Self {
            display: typed.display_name(),
            typed,
        }
    }
}

#[derive(Clone, Copy)]
struct ReturnValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl ReturnValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0212",
            typed_code: "E0357",
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver value symbol '{name}' has return type '{actual}', expected '{expected}'")
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver value symbol '{name}' has typed return type '{actual}', expected '{expected}'"
        )
    }
}

struct ExpectedBehaviorMethod {
    signature: MethodSignatureMetadata,
    metadata: BehaviorMethodTypeMetadata,
}

impl ExpectedBehaviorMethod {
    fn new(method: &ast::BehaviorMethod) -> Self {
        let signature = expected_value_signature_metadata(&method.params, &method.return_type, &[]);
        let parameter_type_names: Vec<_> = signature
            .params
            .iter()
            .map(|param| param.display.clone())
            .collect();
        let parameter_names: Vec<_> = signature
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let parameter_types: Vec<_> = signature
            .params
            .into_iter()
            .map(|param| param.typed)
            .collect();

        Self {
            signature: (
                method.name.clone(),
                parameter_type_names,
                signature.return_type.display,
            ),
            metadata: BehaviorMethodTypeMetadata {
                name: method.name.clone(),
                parameter_names,
                parameter_types,
                return_type: signature.return_type.typed,
            },
        }
    }
}

struct ExpectedBehaviorMethodMetadata {
    signatures: Vec<MethodSignatureMetadata>,
    typed: Vec<BehaviorMethodTypeMetadata>,
}

#[derive(Clone, Copy)]
struct BehaviorMethodValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl BehaviorMethodValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0219",
            typed_code: "E0355",
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver behavior symbol '{name}' has methods '{actual}', expected '{expected}'")
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver behavior symbol '{name}' has typed methods '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedBehaviorMethodMetadata {
    fn from_methods(methods: &[ExpectedBehaviorMethod]) -> Self {
        Self {
            signatures: methods
                .iter()
                .map(|method| method.signature.clone())
                .collect(),
            typed: methods
                .iter()
                .map(|method| method.metadata.clone())
                .collect(),
        }
    }
}

struct ExpectedBehaviorSymbol {
    type_like: ExpectedTypeLikeSymbol,
    methods: Vec<ExpectedBehaviorMethod>,
}

impl ExpectedBehaviorSymbol {
    fn new(
        type_params: &[ast::TypeParam],
        methods: &[ast::BehaviorMethod],
        is_public: bool,
    ) -> Self {
        Self {
            type_like: ExpectedTypeLikeSymbol::new(type_params, Some(is_public)),
            methods: expected_behavior_method_metadata(methods),
        }
    }
}

struct ExpectedStructSymbol {
    type_like: ExpectedTypeLikeSymbol,
    fields: Vec<ExpectedField>,
}

impl ExpectedStructSymbol {
    fn new(type_params: &[ast::TypeParam], fields: &[StructField], is_public: bool) -> Self {
        Self {
            type_like: ExpectedTypeLikeSymbol::new(type_params, Some(is_public)),
            fields: expected_field_metadata(fields),
        }
    }
}

struct ExpectedEnumSymbol {
    type_like: ExpectedTypeLikeSymbol,
    variant_names: Vec<String>,
}

impl ExpectedEnumSymbol {
    fn new(type_params: &[ast::TypeParam], variants: &[EnumVariant], is_public: bool) -> Self {
        Self {
            type_like: ExpectedTypeLikeSymbol::new(type_params, Some(is_public)),
            variant_names: expected_variant_name_metadata(variants),
        }
    }
}

struct ExpectedVariantSymbol {
    owner_name: String,
    is_public: bool,
    payload: ExpectedVariantPayloadType,
}

impl ExpectedVariantSymbol {
    fn new(owner_name: &str, is_public: bool, payload: &Option<AstType>) -> Self {
        Self {
            owner_name: owner_name.to_string(),
            is_public,
            payload: ExpectedVariantPayloadType::new(payload),
        }
    }
}

struct ExpectedImportSymbol {
    source: String,
    is_public: bool,
}

impl ExpectedImportSymbol {
    fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            is_public: false,
        }
    }
}

struct ExpectedModuleSymbol {
    name: String,
    source: Option<String>,
    is_public: bool,
}

impl ExpectedModuleSymbol {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            source: None,
            is_public: false,
        }
    }
}

struct ExpectedLocalSymbol {
    scope_id: u32,
    is_mutable: bool,
    is_public: bool,
    source: Option<String>,
}

impl ExpectedLocalSymbol {
    fn new(is_mutable: bool, scope_id: u32) -> Self {
        Self {
            scope_id,
            is_mutable,
            is_public: false,
            source: None,
        }
    }
}

struct ExpectedTypeParameter {
    name: String,
    bound: Option<ExpectedTypeParameterBound>,
}

impl ExpectedTypeParameter {
    fn new(type_param: &ast::TypeParam) -> Self {
        Self {
            name: type_param.name.clone(),
            bound: ExpectedTypeParameterBound::new(type_param),
        }
    }
}

struct ExpectedTypeParameterBound {
    display: TypeParameterBoundMetadata,
    reference: TypeParameterBoundRefMetadata,
}

impl ExpectedTypeParameterBound {
    fn new(type_param: &ast::TypeParam) -> Option<Self> {
        let behavior = type_param.constraint.as_ref()?;
        let display = type_param_bound_display(type_param)?;
        Some(Self {
            display: (type_param.name.clone(), display),
            reference: TypeParameterBoundRefMetadata {
                type_parameter: type_param.name.clone(),
                behavior: behavior.clone(),
                type_args: type_param.constraint_type_args.clone(),
            },
        })
    }
}

struct ExpectedTypeParameterMetadata {
    count: usize,
    names: Vec<String>,
    bounds: Vec<TypeParameterBoundMetadata>,
    bound_refs: Vec<TypeParameterBoundRefMetadata>,
}

impl ExpectedTypeParameterMetadata {
    fn from_parameters(parameters: &[ExpectedTypeParameter]) -> Self {
        Self {
            count: parameters.len(),
            names: parameters.iter().map(|param| param.name.clone()).collect(),
            bounds: parameters
                .iter()
                .filter_map(|param| param.bound.as_ref().map(|bound| bound.display.clone()))
                .collect(),
            bound_refs: parameters
                .iter()
                .filter_map(|param| param.bound.as_ref().map(|bound| bound.reference.clone()))
                .collect(),
        }
    }
}

struct ExpectedTypeLikeSymbol {
    type_params: Vec<ExpectedTypeParameter>,
    is_public: Option<bool>,
}

impl ExpectedTypeLikeSymbol {
    fn new(type_params: &[ast::TypeParam], is_public: Option<bool>) -> Self {
        Self {
            type_params: expected_type_parameter_metadata(type_params),
            is_public,
        }
    }
}

#[derive(Clone, Copy)]
struct TypeParameterValidation {
    count_code: &'static str,
    name_code: &'static str,
    bound_code: &'static str,
    bound_ref_code: &'static str,
}

impl TypeParameterValidation {
    fn type_like_resolver_codes() -> Self {
        Self {
            count_code: "E0213",
            name_code: "E0346",
            bound_code: "E0222",
            bound_ref_code: "E0350",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            count_code: "E0220",
            name_code: "E0347",
            bound_code: "E0221",
            bound_ref_code: "E0351",
        }
    }

    fn count_validation(self) -> CountValidation {
        CountValidation {
            label: "type parameter count",
            code: self.count_code,
        }
    }

    fn name_message(self, symbol_kind: &str, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has type parameter names '{actual}', expected '{expected}'"
        )
    }

    fn bound_message(self, symbol_kind: &str, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has type parameter bounds '{actual}', expected '{expected}'"
        )
    }

    fn bound_ref_message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: &str,
        expected: &str,
    ) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has type parameter bound refs '{actual}', expected '{expected}'"
        )
    }
}

#[derive(Clone, Copy)]
struct TypeParameterAbsenceValidation {
    count_code: &'static str,
    name_code: &'static str,
    bound_code: &'static str,
    bound_ref_code: &'static str,
}

impl TypeParameterAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            count_code: "E0269",
            name_code: "E0348",
            bound_code: "E0270",
            bound_ref_code: "E0373",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            count_code: "E0285",
            name_code: "E0349",
            bound_code: "E0286",
            bound_ref_code: "E0364",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            count_code: "E0253",
            name_code: "E0350",
            bound_code: "E0254",
            bound_ref_code: "E0382",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            count_code: "E0334",
            name_code: "E0351",
            bound_code: "E0335",
            bound_ref_code: "E0391",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        [
            AbsentMetadataEntry::new(
                symbol.type_parameter_count.is_some(),
                self.count_code,
                "type parameter count",
            ),
            AbsentMetadataEntry::new(
                symbol.type_parameter_names.is_some(),
                self.name_code,
                "type parameter names",
            ),
            AbsentMetadataEntry::new(
                symbol.type_parameter_bounds.is_some(),
                self.bound_code,
                "type parameter bounds",
            ),
            AbsentMetadataEntry::new(
                symbol.type_parameter_bound_refs.is_some(),
                self.bound_ref_code,
                "typed type parameter bound refs",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct ValueSignatureAbsenceValidation {
    parameter_count_code: &'static str,
    parameter_name_code: &'static str,
    parameter_type_name_code: &'static str,
    parameter_type_code: &'static str,
    return_type_code: &'static str,
    typed_return_type_code: &'static str,
}

impl ValueSignatureAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0265",
            parameter_name_code: "E0267",
            parameter_type_name_code: "E0268",
            parameter_type_code: "E0371",
            return_type_code: "E0266",
            typed_return_type_code: "E0372",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0281",
            parameter_name_code: "E0283",
            parameter_type_name_code: "E0284",
            parameter_type_code: "E0362",
            return_type_code: "E0282",
            typed_return_type_code: "E0363",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0249",
            parameter_name_code: "E0251",
            parameter_type_name_code: "E0252",
            parameter_type_code: "E0380",
            return_type_code: "E0250",
            typed_return_type_code: "E0381",
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0310",
            parameter_name_code: "E0312",
            parameter_type_name_code: "E0313",
            parameter_type_code: "E0360",
            return_type_code: "E0311",
            typed_return_type_code: "E0361",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            parameter_count_code: "E0330",
            parameter_name_code: "E0332",
            parameter_type_name_code: "E0333",
            parameter_type_code: "E0389",
            return_type_code: "E0331",
            typed_return_type_code: "E0390",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 6] {
        [
            AbsentMetadataEntry::new(
                symbol.parameter_count.is_some(),
                self.parameter_count_code,
                "parameter count",
            ),
            AbsentMetadataEntry::new(
                symbol.parameter_names.is_some(),
                self.parameter_name_code,
                "parameter names",
            ),
            AbsentMetadataEntry::new(
                symbol.parameter_type_names.is_some(),
                self.parameter_type_name_code,
                "parameter types",
            ),
            AbsentMetadataEntry::new(
                symbol.parameter_types.is_some(),
                self.parameter_type_code,
                "typed parameter types",
            ),
            AbsentMetadataEntry::new(
                symbol.return_type_name.is_some(),
                self.return_type_code,
                "return type",
            ),
            AbsentMetadataEntry::new(
                symbol.return_type.is_some(),
                self.typed_return_type_code,
                "typed return type",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct FieldAbsenceValidation {
    count_code: &'static str,
    type_name_code: &'static str,
    typed_code: &'static str,
}

impl FieldAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            count_code: "E0271",
            type_name_code: "E0272",
            typed_code: "E0374",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            count_code: "E0287",
            type_name_code: "E0288",
            typed_code: "E0365",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            count_code: "E0255",
            type_name_code: "E0256",
            typed_code: "E0383",
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            count_code: "E0319",
            type_name_code: "E0320",
            typed_code: "E0398",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            count_code: "E0336",
            type_name_code: "E0337",
            typed_code: "E0392",
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            count_code: "E0321",
            type_name_code: "E0322",
            typed_code: "E0399",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            count_code: "E0298",
            type_name_code: "E0299",
            typed_code: "E0403",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 3] {
        [
            AbsentMetadataEntry::new(symbol.field_count.is_some(), self.count_code, "field count"),
            AbsentMetadataEntry::new(
                symbol.field_type_names.is_some(),
                self.type_name_code,
                "field types",
            ),
            AbsentMetadataEntry::new(
                symbol.field_types.is_some(),
                self.typed_code,
                "typed field types",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct VariantAbsenceValidation {
    names_code: &'static str,
    owner_code: &'static str,
    payload_count_code: &'static str,
    payload_type_name_code: &'static str,
    payload_type_code: &'static str,
}

impl VariantAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            names_code: "E0273",
            owner_code: "E0274",
            payload_count_code: "E0275",
            payload_type_name_code: "E0276",
            payload_type_code: "E0375",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            names_code: "E0289",
            owner_code: "E0290",
            payload_count_code: "E0291",
            payload_type_name_code: "E0292",
            payload_type_code: "E0366",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            names_code: "E0257",
            owner_code: "E0258",
            payload_count_code: "E0259",
            payload_type_name_code: "E0260",
            payload_type_code: "E0384",
        }
    }

    fn type_like_resolver_codes() -> Self {
        Self {
            names_code: "E0315",
            owner_code: "E0316",
            payload_count_code: "E0317",
            payload_type_name_code: "E0318",
            payload_type_code: "E0397",
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            names_code: "E0323",
            owner_code: "E0324",
            payload_count_code: "E0325",
            payload_type_name_code: "E0326",
            payload_type_code: "E0400",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            names_code: "E0300",
            owner_code: "E0301",
            payload_count_code: "E0302",
            payload_type_name_code: "E0303",
            payload_type_code: "E0404",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 5] {
        [
            AbsentMetadataEntry::new(
                symbol.variant_names.is_some(),
                self.names_code,
                "variant names",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_owner_name.is_some(),
                self.owner_code,
                "variant owner",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_payload_count.is_some(),
                self.payload_count_code,
                "variant payload count",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_payload_type_name.is_some(),
                self.payload_type_name_code,
                "variant payload type",
            ),
            AbsentMetadataEntry::new(
                symbol.variant_payload_type.is_some(),
                self.payload_type_code,
                "typed variant payload type",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct BehaviorAssociationAbsenceValidation {
    impl_name_code: &'static str,
    impl_ref_code: &'static str,
    required_name_code: &'static str,
    required_ref_code: &'static str,
}

impl BehaviorAssociationAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0279",
            impl_ref_code: "E0378",
            required_name_code: "E0280",
            required_ref_code: "E0379",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0295",
            impl_ref_code: "E0369",
            required_name_code: "E0296",
            required_ref_code: "E0370",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0263",
            impl_ref_code: "E0387",
            required_name_code: "E0264",
            required_ref_code: "E0388",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0341",
            impl_ref_code: "E0395",
            required_name_code: "E0342",
            required_ref_code: "E0396",
        }
    }

    fn behavior_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0327",
            impl_ref_code: "E0401",
            required_name_code: "E0328",
            required_ref_code: "E0402",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            impl_name_code: "E0306",
            impl_ref_code: "E0407",
            required_name_code: "E0307",
            required_ref_code: "E0408",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        [
            AbsentMetadataEntry::new(
                symbol.behavior_impl_names.is_some(),
                self.impl_name_code,
                "behavior impls",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_impl_refs.is_some(),
                self.impl_ref_code,
                "typed behavior impls",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_required_names.is_some(),
                self.required_name_code,
                "behavior requires",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_required_refs.is_some(),
                self.required_ref_code,
                "typed behavior requires",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct BehaviorDeclarationAbsenceValidation {
    method_signature_code: &'static str,
    method_type_code: &'static str,
    parent_name_code: &'static str,
    parent_ref_code: &'static str,
}

impl BehaviorDeclarationAbsenceValidation {
    fn module_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0277",
            method_type_code: "E0376",
            parent_name_code: "E0278",
            parent_ref_code: "E0377",
        }
    }

    fn import_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0293",
            method_type_code: "E0367",
            parent_name_code: "E0294",
            parent_ref_code: "E0368",
        }
    }

    fn local_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0261",
            method_type_code: "E0385",
            parent_name_code: "E0262",
            parent_ref_code: "E0386",
        }
    }

    fn variant_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0339",
            method_type_code: "E0393",
            parent_name_code: "E0340",
            parent_ref_code: "E0394",
        }
    }

    fn value_resolver_codes() -> Self {
        Self {
            method_signature_code: "E0304",
            method_type_code: "E0405",
            parent_name_code: "E0305",
            parent_ref_code: "E0406",
        }
    }

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        [
            AbsentMetadataEntry::new(
                symbol.behavior_method_signatures.is_some(),
                self.method_signature_code,
                "behavior methods",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_method_types.is_some(),
                self.method_type_code,
                "typed behavior methods",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_parent_names.is_some(),
                self.parent_name_code,
                "behavior parents",
            ),
            AbsentMetadataEntry::new(
                symbol.behavior_parent_refs.is_some(),
                self.parent_ref_code,
                "typed behavior parents",
            ),
        ]
    }
}

#[derive(Clone, Copy)]
struct MutabilityAbsenceValidation {
    code: &'static str,
}

impl MutabilityAbsenceValidation {
    fn module_resolver_code() -> Self {
        Self { code: "E0345" }
    }

    fn import_resolver_code() -> Self {
        Self { code: "E0344" }
    }

    fn type_like_resolver_code() -> Self {
        Self { code: "E0314" }
    }

    fn variant_resolver_code() -> Self {
        Self { code: "E0343" }
    }

    fn value_resolver_code() -> Self {
        Self { code: "E0308" }
    }

    fn entry(self, symbol: &Symbol) -> AbsentMetadataEntry {
        AbsentMetadataEntry::new(symbol.is_mutable.is_some(), self.code, "mutability")
    }
}

#[derive(Clone, Copy)]
struct MutabilityValidation {
    code: &'static str,
}

impl MutabilityValidation {
    fn resolver_code() -> Self {
        Self { code: "E0231" }
    }

    fn display(self, actual: Option<bool>, expected: bool) -> (&'static str, &'static str) {
        (mutability_name(actual), mutability_name(Some(expected)))
    }

    fn message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: Option<bool>,
        expected: bool,
    ) -> String {
        let (actual, expected) = self.display(actual, expected);
        format!(
            "resolver {symbol_kind} symbol '{name}' has mutability {actual}, expected {expected}"
        )
    }
}

#[derive(Clone, Copy)]
struct VisibilityValidation {
    code: &'static str,
}

impl VisibilityValidation {
    fn module_resolver_code() -> Self {
        Self { code: "E0229" }
    }

    fn import_resolver_code() -> Self {
        Self { code: "E0245" }
    }

    fn type_like_resolver_code() -> Self {
        Self { code: "E0225" }
    }

    fn variant_resolver_code() -> Self {
        Self { code: "E0226" }
    }

    fn value_resolver_code() -> Self {
        Self { code: "E0224" }
    }

    fn local_resolver_code() -> Self {
        Self { code: "E0247" }
    }

    fn display(self, actual: bool, expected: bool) -> (&'static str, &'static str) {
        (visibility_name(actual), visibility_name(expected))
    }

    fn message(self, symbol_kind: &str, name: &str, actual: bool, expected: bool) -> String {
        let (actual, expected) = self.display(actual, expected);
        format!(
            "resolver {symbol_kind} symbol '{name}' has visibility {actual}, expected {expected}"
        )
    }
}

#[derive(Clone, Copy)]
struct SourceAbsenceValidation {
    code: &'static str,
}

impl SourceAbsenceValidation {
    fn type_like_resolver_code() -> Self {
        Self { code: "E0309" }
    }

    fn variant_resolver_code() -> Self {
        Self { code: "E0329" }
    }

    fn value_resolver_code() -> Self {
        Self { code: "E0297" }
    }

    fn source_validation(self) -> SourceValidation {
        SourceValidation {
            code: self.code,
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }
}

#[derive(Clone, Copy)]
enum ResolverSymbolPresence {
    Extra,
    Missing,
}

#[derive(Clone, Copy)]
struct ResolverSymbolPresenceValidation {
    code: &'static str,
    presence: ResolverSymbolPresence,
}

impl ResolverSymbolPresenceValidation {
    fn missing_resolver_code() -> Self {
        Self {
            code: "E0210",
            presence: ResolverSymbolPresence::Missing,
        }
    }

    fn missing_local_resolver_code() -> Self {
        Self {
            code: "E0228",
            presence: ResolverSymbolPresence::Missing,
        }
    }

    fn extra_declaration_resolver_code() -> Self {
        Self {
            code: "E0243",
            presence: ResolverSymbolPresence::Extra,
        }
    }

    fn extra_local_resolver_code() -> Self {
        Self {
            code: "E0244",
            presence: ResolverSymbolPresence::Extra,
        }
    }

    fn message(self, symbol_kind: &str, name: &str) -> String {
        let verb = match self.presence {
            ResolverSymbolPresence::Extra => "has extra",
            ResolverSymbolPresence::Missing => "missing",
        };
        format!("resolver symbol table {verb} {symbol_kind} symbol '{name}'")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AbsentMetadataEntry {
    present: bool,
    code: &'static str,
    label: &'static str,
}

impl AbsentMetadataEntry {
    fn new(present: bool, code: &'static str, label: &'static str) -> Self {
        Self {
            present,
            code,
            label,
        }
    }

    fn message(self, symbol_kind: &str, name: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has {} metadata, expected none",
            self.label
        )
    }
}

#[derive(Clone, Copy)]
struct SourceValidation {
    code: &'static str,
    actual_missing: &'static str,
    expected_missing: &'static str,
    quote_expected: bool,
}

impl SourceValidation {
    fn module_resolver_code() -> Self {
        Self {
            code: "E0230",
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }

    fn stripped_import_resolver_code() -> Self {
        Self {
            code: "E0246",
            actual_missing: "unknown",
            expected_missing: "a module source",
            quote_expected: false,
        }
    }

    fn import_resolver_code() -> Self {
        Self {
            code: "E0227",
            actual_missing: "unknown",
            expected_missing: "none",
            quote_expected: true,
        }
    }

    fn local_resolver_code() -> Self {
        Self {
            code: "E0248",
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        }
    }

    fn message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: Option<&str>,
        expected: Option<&str>,
    ) -> String {
        let actual = actual.unwrap_or(self.actual_missing);
        let expected = expected.unwrap_or(self.expected_missing);
        let expected = if self.quote_expected {
            format!("'{expected}'")
        } else {
            expected.to_string()
        };
        format!("resolver {symbol_kind} symbol '{name}' has source '{actual}', expected {expected}")
    }
}

#[derive(Clone, Copy)]
struct CountValidation {
    label: &'static str,
    code: &'static str,
}

impl CountValidation {
    fn value_parameter_resolver_code() -> Self {
        Self {
            label: "parameter count",
            code: "E0211",
        }
    }

    fn field_resolver_code() -> Self {
        Self {
            label: "field count",
            code: "E0214",
        }
    }

    fn variant_payload_resolver_code() -> Self {
        Self {
            label: "payload count",
            code: "E0215",
        }
    }

    fn message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: Option<usize>,
        expected: usize,
    ) -> String {
        let actual = resolver_count_display(actual);
        format!(
            "resolver {symbol_kind} symbol '{name}' has {} {actual}, expected {expected}",
            self.label
        )
    }
}

struct ExpectedField {
    typed: (String, AstType),
    display: (String, String),
}

impl ExpectedField {
    fn new(name: &str, ty: &AstType) -> Self {
        Self {
            typed: (name.to_string(), ty.clone()),
            display: (name.to_string(), ty.display_name()),
        }
    }
}

struct ExpectedFieldMetadata {
    count: usize,
    typed: Vec<(String, AstType)>,
    display: Vec<(String, String)>,
}

#[derive(Clone, Copy)]
struct FieldValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl FieldValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0217",
            typed_code: "E0358",
        }
    }

    fn display_message(
        self,
        symbol_kind: &str,
        name: &str,
        actual: &str,
        expected: &str,
    ) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has fields '{actual}', expected '{expected}'"
        )
    }

    fn typed_message(self, symbol_kind: &str, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {symbol_kind} symbol '{name}' has typed fields '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedFieldMetadata {
    fn from_fields(fields: &[ExpectedField]) -> Self {
        Self {
            count: fields.len(),
            typed: fields.iter().map(|field| field.typed.clone()).collect(),
            display: fields.iter().map(|field| field.display.clone()).collect(),
        }
    }
}

struct ExpectedVariantPayloadType {
    typed: Option<AstType>,
    display: Option<String>,
}

impl ExpectedVariantPayloadType {
    fn new(payload: &Option<AstType>) -> Self {
        Self {
            typed: payload.clone(),
            display: payload.as_ref().map(AstType::display_name),
        }
    }
}

struct ExpectedVariantPayloadMetadata {
    count: usize,
    typed: Option<AstType>,
    display: Option<String>,
}

#[derive(Clone, Copy)]
struct VariantNameValidation {
    code: &'static str,
}

impl VariantNameValidation {
    fn resolver_code() -> Self {
        Self { code: "E0241" }
    }

    fn message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver type symbol '{name}' has variants '{actual}', expected '{expected}'")
    }
}

#[derive(Clone, Copy)]
struct VariantOwnerValidation {
    code: &'static str,
}

impl VariantOwnerValidation {
    fn resolver_code() -> Self {
        Self { code: "E0242" }
    }

    fn message(self, name: &str, actual: &str, expected: &str) -> String {
        format!("resolver variant symbol '{name}' has owner '{actual}', expected '{expected}'")
    }
}

#[derive(Clone, Copy)]
struct VariantPayloadValidation {
    display_code: &'static str,
    typed_code: &'static str,
}

impl VariantPayloadValidation {
    fn resolver_codes() -> Self {
        Self {
            display_code: "E0218",
            typed_code: "E0359",
        }
    }

    fn display_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver variant symbol '{name}' has payload type '{actual}', expected '{expected}'"
        )
    }

    fn typed_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver variant symbol '{name}' has typed payload type '{actual}', expected '{expected}'"
        )
    }
}

impl ExpectedVariantPayloadMetadata {
    fn from_payload(payload: ExpectedVariantPayloadType) -> Self {
        Self {
            count: usize::from(payload.typed.is_some()),
            typed: payload.typed,
            display: payload.display,
        }
    }
}

struct ImportedMethodSignature<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    params: &'a [Param],
    return_type: &'a Option<AstType>,
    body: &'a Expression,
    span: Span,
}

impl<'a> ImportedMethodSignature<'a> {
    fn from_function_declaration(name: &'a str, decl: &'a Declaration) -> Option<Self> {
        let Declaration::Function {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return None;
        };

        Some(Self {
            name,
            type_params,
            params,
            return_type,
            body,
            span: *span,
        })
    }

    fn from_method_declaration(name: &'a str, decl: &'a Declaration) -> Option<Self> {
        let Declaration::Method {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return None;
        };

        Some(Self {
            name,
            type_params,
            params,
            return_type,
            body,
            span: *span,
        })
    }

    fn func_info(&self, key: String) -> FuncInfo {
        func_info_from_ast_signature(key, self.type_params, self.params, self.return_type)
    }

    fn generic_template(&self) -> Option<GenericFunctionTemplate> {
        generic_template_from_type_params(
            self.type_params,
            self.params,
            self.return_type,
            self.body,
            self.span,
        )
    }
}

#[derive(Clone, Default)]
struct SourceModuleDependencies {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    generic_functions: HashMap<String, GenericFunctionTemplate>,
    methods: HashMap<String, FuncInfo>,
    generic_methods: HashMap<String, GenericFunctionTemplate>,
}

impl SourceModuleDependencies {
    fn apply_to_template(&self, template: GenericFunctionTemplate) -> GenericFunctionTemplate {
        template.with_source_dependencies(self.clone())
    }
}

#[derive(Debug, Clone)]
struct BehaviorParentRef {
    behavior: String,
    type_args: Vec<AstType>,
    key: String,
}

#[derive(Default)]
struct ResolverScopeCursor {
    next_scope_id: u32,
}

impl ResolverScopeCursor {
    fn new_scope(&mut self) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::new(self.next_scope_id)
    }

    fn child_scope(&mut self, parent: &ResolverLocalScope) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::with_parent(self.next_scope_id, parent)
    }
}

#[derive(Clone)]
struct ResolverLocalScope {
    current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ResolverLocalScope {
    fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    fn with_parent(current_scope_id: u32, parent: &ResolverLocalScope) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    fn insert(&mut self, name: String, mutable: bool) {
        self.visible_names.insert(name, mutable);
    }
}

/// Scope for variable types.
#[derive(Debug, Clone)]
struct Scope {
    vars: HashMap<String, VarInfo>,
}

#[derive(Debug, Clone)]
pub(crate) struct VarInfo {
    pub ty: Type,
    pub mutable: bool,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

fn type_param_bounds(type_params: &[ast::TypeParam]) -> HashMap<String, BehaviorBound> {
    type_params
        .iter()
        .filter_map(|param| {
            param.constraint.as_ref().map(|bound| {
                (
                    param.name.clone(),
                    BehaviorBound {
                        behavior: bound.clone(),
                        type_args: param.constraint_type_args.clone(),
                    },
                )
            })
        })
        .collect()
}

fn type_param_names(type_params: &[ast::TypeParam]) -> Vec<String> {
    type_params.iter().map(|param| param.name.clone()).collect()
}

fn generic_template_from_type_params(
    type_params: &[ast::TypeParam],
    params: &[Param],
    return_type: &Option<AstType>,
    body: &Expression,
    span: Span,
) -> Option<GenericFunctionTemplate> {
    let collected_type_params = type_param_names(type_params);
    if collected_type_params.is_empty() {
        return None;
    }

    Some(GenericFunctionTemplate::new(
        collected_type_params,
        params.to_vec(),
        return_type.clone(),
        body.clone(),
        span,
    ))
}

fn generic_template_body_stub_from_type_params(
    type_params: &[ast::TypeParam],
    params: &[Param],
    body: &Expression,
    span: Span,
) -> Option<GenericFunctionTemplate> {
    if type_params.is_empty() {
        return None;
    }

    let params = params
        .iter()
        .map(|param| Param {
            name: String::new(),
            ty: AstType::Void,
            mutable: param.mutable,
            span: param.span,
        })
        .collect();
    Some(GenericFunctionTemplate::new(
        Vec::new(),
        params,
        None,
        body.clone(),
        span,
    ))
}

fn func_info_from_ast_signature(
    name: String,
    type_params: &[ast::TypeParam],
    params: &[Param],
    return_type: &Option<AstType>,
) -> FuncInfo {
    FuncInfo {
        name,
        params: params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect(),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
    }
}

fn func_info_from_resolver_signature(
    name: String,
    symbol: &Symbol,
    parameter_names: &[String],
    parameter_types: &[AstType],
    return_type: &AstType,
) -> FuncInfo {
    FuncInfo {
        name,
        params: parameter_names
            .iter()
            .cloned()
            .zip(parameter_types.iter().cloned())
            .collect(),
        return_type: return_type.clone(),
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
    }
}

fn struct_info_from_ast_fields(
    name: String,
    type_params: &[ast::TypeParam],
    fields: &[StructField],
) -> StructInfo {
    StructInfo {
        name,
        fields: fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect(),
        field_defaults: fields
            .iter()
            .filter_map(|field| {
                field
                    .default
                    .as_ref()
                    .map(|default| (field.name.clone(), default.clone()))
            })
            .collect(),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
    }
}

fn enum_info_from_ast_variants(
    name: String,
    type_params: &[ast::TypeParam],
    variants: &[EnumVariant],
) -> EnumInfo {
    EnumInfo {
        name,
        variants: variants
            .iter()
            .map(|variant| (variant.name.clone(), variant.payload.clone()))
            .collect(),
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
    }
}

fn behavior_info_from_ast_methods(
    name: String,
    type_params: &[ast::TypeParam],
    methods: &[ast::BehaviorMethod],
) -> BehaviorInfo {
    BehaviorInfo {
        name,
        type_params: type_param_names(type_params),
        type_param_bounds: type_param_bounds(type_params),
        methods: methods.to_vec(),
    }
}

fn behavior_info_for_resolver_backed_stub(
    name: String,
    methods: &[ast::BehaviorMethod],
) -> BehaviorInfo {
    BehaviorInfo {
        name,
        type_params: Vec::new(),
        type_param_bounds: HashMap::new(),
        methods: methods.to_vec(),
    }
}

fn struct_info_from_resolver_fields(
    name: String,
    symbol: &Symbol,
    fields: Vec<(String, AstType)>,
    field_defaults: HashMap<String, Expression>,
) -> StructInfo {
    StructInfo {
        name,
        fields,
        field_defaults,
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
    }
}

fn enum_info_from_resolver_variants(
    name: String,
    symbol: &Symbol,
    variants: Vec<(String, Option<AstType>)>,
) -> EnumInfo {
    EnumInfo {
        name,
        variants,
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
    }
}

fn behavior_info_from_resolver_methods(
    name: String,
    symbol: &Symbol,
    methods: Vec<ast::BehaviorMethod>,
) -> BehaviorInfo {
    BehaviorInfo {
        name,
        type_params: resolver_type_param_names(symbol),
        type_param_bounds: resolver_type_param_bounds(symbol),
        methods,
    }
}

fn func_info_from_behavior_method(
    name: String,
    params: &[Param],
    return_type: &Option<AstType>,
) -> FuncInfo {
    FuncInfo {
        name,
        params: params
            .iter()
            .map(|param| (param.name.clone(), param.ty.clone()))
            .collect(),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        type_params: Vec::new(),
        type_param_bounds: HashMap::new(),
    }
}

fn type_param_bounds_from_resolver_refs(
    bounds: &[TypeParameterBoundRefMetadata],
) -> HashMap<String, BehaviorBound> {
    bounds
        .iter()
        .map(|bound| {
            (
                bound.type_parameter.clone(),
                BehaviorBound {
                    behavior: bound.behavior.clone(),
                    type_args: bound.type_args.clone(),
                },
            )
        })
        .collect()
}

fn resolver_type_param_bounds(symbol: &crate::resolver::Symbol) -> HashMap<String, BehaviorBound> {
    symbol
        .type_parameter_bound_refs
        .as_deref()
        .map(type_param_bounds_from_resolver_refs)
        .unwrap_or_default()
}

fn resolver_type_param_names(symbol: &crate::resolver::Symbol) -> Vec<String> {
    symbol.type_parameter_names.clone().unwrap_or_default()
}

fn method_signature_key(type_name: &str, method_name: &str) -> String {
    format!("{type_name}.{method_name}")
}

fn method_signature_key_parts(name: &str) -> Option<(&str, &str)> {
    name.split_once('.')
}

fn method_signature_receiver_name(name: &str) -> Option<&str> {
    method_signature_key_parts(name).map(|(receiver, _)| receiver)
}

fn method_signature_method_name_for_receiver<'a>(name: &'a str, receiver: &str) -> Option<&'a str> {
    method_signature_key_parts(name)
        .and_then(|(actual_receiver, method)| (actual_receiver == receiver).then_some(method))
}

fn is_method_signature_key(name: &str) -> bool {
    method_signature_key_parts(name).is_some()
}

fn type_param_bound_display(type_param: &ast::TypeParam) -> Option<String> {
    type_param.constraint.as_ref().map(|constraint| {
        if type_param.constraint_type_args.is_empty() {
            constraint.clone()
        } else {
            format!(
                "{}<{}>",
                constraint,
                type_param
                    .constraint_type_args
                    .iter()
                    .map(AstType::display_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    })
}

fn type_param_name_set(type_params: &[ast::TypeParam]) -> HashSet<String> {
    type_param_names(type_params).into_iter().collect()
}

fn ast_type_references_type_param(
    ast_type: &AstType,
    scoped_type_params: &HashSet<String>,
) -> bool {
    match ast_type {
        AstType::Named(name) => scoped_type_params.contains(name),
        AstType::Generic { type_args, .. } => type_args
            .iter()
            .any(|arg| ast_type_references_type_param(arg, scoped_type_params)),
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => {
            ast_type_references_type_param(inner, scoped_type_params)
        }
        AstType::Function { params, ret } => {
            params
                .iter()
                .any(|param| ast_type_references_type_param(param, scoped_type_params))
                || ast_type_references_type_param(ret, scoped_type_params)
        }
        _ => false,
    }
}

fn collect_ast_type_names(ast_type: &AstType, names: &mut HashSet<String>) {
    match ast_type {
        AstType::Named(name) => {
            names.insert(name.clone());
        }
        AstType::Generic { name, type_args } => {
            names.insert(name.clone());
            for type_arg in type_args {
                collect_ast_type_names(type_arg, names);
            }
        }
        AstType::Ptr(inner)
        | AstType::MutPtr(inner)
        | AstType::RawPtr(inner)
        | AstType::Slice(inner)
        | AstType::Array { elem: inner, .. } => collect_ast_type_names(inner, names),
        AstType::Function { params, ret } => {
            for param in params {
                collect_ast_type_names(param, names);
            }
            collect_ast_type_names(ret, names);
        }
        _ => {}
    }
}

fn concrete_self_ast_type(ast_type: &AstType, self_type_name: &str) -> AstType {
    match ast_type {
        AstType::SelfType => AstType::Named(self_type_name.to_string()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(concrete_self_ast_type(inner, self_type_name)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(concrete_self_ast_type(elem, self_type_name)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| concrete_self_ast_type(param, self_type_name))
                .collect(),
            ret: Box::new(concrete_self_ast_type(ret, self_type_name)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| concrete_self_ast_type(arg, self_type_name))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, AstType>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .cloned()
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => {
            AstType::Ptr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::MutPtr(inner) => {
            AstType::MutPtr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::RawPtr(inner) => {
            AstType::RawPtr(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::Slice(inner) => {
            AstType::Slice(Box::new(substitute_behavior_ast_type(inner, substitutions)))
        }
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_behavior_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_behavior_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_behavior_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: type_args
                .iter()
                .map(|arg| substitute_behavior_ast_type(arg, substitutions))
                .collect(),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_bound_ast_type(
    ast_type: &AstType,
    substitutions: &HashMap<String, Type>,
) -> AstType {
    match ast_type {
        AstType::Named(name) => substitutions
            .get(name)
            .map(monomorphize::type_to_ast)
            .unwrap_or_else(|| ast_type.clone()),
        AstType::Ptr(inner) => AstType::Ptr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::MutPtr(inner) => AstType::MutPtr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::RawPtr(inner) => AstType::RawPtr(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Slice(inner) => AstType::Slice(Box::new(substitute_behavior_bound_ast_type(
            inner,
            substitutions,
        ))),
        AstType::Array { elem, size } => AstType::Array {
            elem: Box::new(substitute_behavior_bound_ast_type(elem, substitutions)),
            size: *size,
        },
        AstType::Function { params, ret } => AstType::Function {
            params: params
                .iter()
                .map(|param| substitute_behavior_bound_ast_type(param, substitutions))
                .collect(),
            ret: Box::new(substitute_behavior_bound_ast_type(ret, substitutions)),
        },
        AstType::Generic { name, type_args } => AstType::Generic {
            name: name.clone(),
            type_args: substitute_behavior_bound_type_args(type_args, substitutions),
        },
        _ => ast_type.clone(),
    }
}

fn substitute_behavior_bound_type_args(
    type_args: &[AstType],
    substitutions: &HashMap<String, Type>,
) -> Vec<AstType> {
    type_args
        .iter()
        .map(|arg| substitute_behavior_bound_ast_type(arg, substitutions))
        .collect()
}

fn behavior_bound_display(bound: &BehaviorBound, substitutions: &HashMap<String, Type>) -> String {
    let type_args = substitute_behavior_bound_type_args(&bound.type_args, substitutions);
    if type_args.is_empty() {
        bound.behavior.clone()
    } else {
        format!(
            "{}<{}>",
            bound.behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        format!(
            "{}<{}>",
            behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn behavior_method_signatures_match(
    left: &ast::BehaviorMethod,
    right: &ast::BehaviorMethod,
) -> bool {
    left.return_type == right.return_type
        && left.params.len() == right.params.len()
        && left
            .params
            .iter()
            .zip(&right.params)
            .all(|(left, right)| left.mutable == right.mutable && left.ty == right.ty)
}

fn substituted_behavior_method_signature(
    method: &ast::BehaviorMethod,
    substitutions: &HashMap<String, AstType>,
) -> ast::BehaviorMethod {
    let mut method = method.clone();
    for param in &mut method.params {
        param.ty = substitute_behavior_ast_type(&param.ty, substitutions);
    }
    if let Some(return_type) = &mut method.return_type {
        *return_type = substitute_behavior_ast_type(return_type, substitutions);
    }
    method
}

#[derive(Clone, Copy)]
struct BehaviorRefValidation {
    symbol_kind: &'static str,
    name_label: &'static str,
    ref_label: &'static str,
    name_code: &'static str,
    ref_code: &'static str,
}

#[derive(Clone, Copy)]
enum BehaviorRefRole {
    Parent,
    Impl,
    Required,
}

#[derive(Clone, Copy)]
enum BehaviorRefCheck {
    Contains,
    List,
}

impl BehaviorRefValidation {
    fn for_role(role: BehaviorRefRole, check: BehaviorRefCheck) -> Self {
        let (symbol_kind, name_label, ref_label) = Self::role_labels(role);
        let (name_code, ref_code) = Self::codes_for(role, check);
        Self {
            symbol_kind,
            name_label,
            ref_label,
            name_code,
            ref_code,
        }
    }

    fn role_labels(role: BehaviorRefRole) -> (&'static str, &'static str, &'static str) {
        match role {
            BehaviorRefRole::Parent => ("behavior", "parents", "parent refs"),
            BehaviorRefRole::Impl => ("type", "behavior impls", "behavior impl refs"),
            BehaviorRefRole::Required => ("type", "behavior requires", "behavior requires refs"),
        }
    }

    fn codes_for(role: BehaviorRefRole, check: BehaviorRefCheck) -> (&'static str, &'static str) {
        match (role, check) {
            (BehaviorRefRole::Parent, BehaviorRefCheck::Contains) => ("E0235", "E0245"),
            (BehaviorRefRole::Parent, BehaviorRefCheck::List) => ("E0240", "E0246"),
            (BehaviorRefRole::Impl, BehaviorRefCheck::Contains) => ("E0236", "E0247"),
            (BehaviorRefRole::Impl, BehaviorRefCheck::List) => ("E0238", "E0248"),
            (BehaviorRefRole::Required, BehaviorRefCheck::Contains) => ("E0237", "E0249"),
            (BehaviorRefRole::Required, BehaviorRefCheck::List) => ("E0239", "E0250"),
        }
    }

    fn contains_name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected to include '{expected}'",
            self.symbol_kind, self.name_label
        )
    }

    fn contains_ref_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected to include '{expected}'",
            self.symbol_kind, self.ref_label
        )
    }

    fn list_name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected '{expected}'",
            self.symbol_kind, self.name_label
        )
    }

    fn list_ref_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected '{expected}'",
            self.symbol_kind, self.ref_label
        )
    }
}

struct BehaviorRefActual<'a> {
    names: Option<&'a [String]>,
    refs: Option<&'a [BehaviorRefMetadata]>,
}

impl<'a> BehaviorRefActual<'a> {
    fn for_role(symbol: &'a Symbol, role: BehaviorRefRole) -> Self {
        let (names, refs) = Self::metadata_for_role(symbol, role);
        Self { names, refs }
    }

    fn metadata_for_role(
        symbol: &'a Symbol,
        role: BehaviorRefRole,
    ) -> (Option<&'a [String]>, Option<&'a [BehaviorRefMetadata]>) {
        match role {
            BehaviorRefRole::Parent => (
                symbol.behavior_parent_names.as_deref(),
                symbol.behavior_parent_refs.as_deref(),
            ),
            BehaviorRefRole::Impl => (
                symbol.behavior_impl_names.as_deref(),
                symbol.behavior_impl_refs.as_deref(),
            ),
            BehaviorRefRole::Required => (
                symbol.behavior_required_names.as_deref(),
                symbol.behavior_required_refs.as_deref(),
            ),
        }
    }

    fn contains_display(&self, expected: &str) -> bool {
        self.names
            .is_some_and(|names| names.iter().any(|name| name == expected))
    }

    fn contains_metadata(&self, expected: &BehaviorRefMetadata) -> bool {
        self.refs
            .is_some_and(|refs| refs.iter().any(|behavior| behavior == expected))
    }

    fn names_match(&self, expected: &[String]) -> bool {
        behavior_ref_names_match(self.names, expected)
    }

    fn refs_match(&self, expected: &[BehaviorRefMetadata]) -> bool {
        behavior_refs_match(self.refs, expected)
    }
}

struct ExpectedBehaviorEdge {
    display: String,
    metadata: BehaviorRefMetadata,
}

impl ExpectedBehaviorEdge {
    fn new(behavior: &str, type_args: &[AstType]) -> Self {
        Self {
            display: behavior_ref_display(behavior, type_args),
            metadata: BehaviorRefMetadata {
                name: behavior.to_string(),
                type_args: type_args.to_vec(),
            },
        }
    }
}

struct ExpectedBehaviorEdgeMetadata {
    names: Vec<String>,
    refs: Vec<BehaviorRefMetadata>,
}

impl ExpectedBehaviorEdgeMetadata {
    fn from_edges(edges: &[ExpectedBehaviorEdge]) -> Self {
        Self {
            names: edges.iter().map(|edge| edge.display.clone()).collect(),
            refs: edges.iter().map(|edge| edge.metadata.clone()).collect(),
        }
    }
}

#[derive(Default)]
struct ExpectedBehaviorEdges {
    edges: HashMap<String, Vec<ExpectedBehaviorEdge>>,
}

impl ExpectedBehaviorEdges {
    fn parents_from(program: &ast::Program) -> Self {
        let mut expected = Self::default();
        for decl in &program.declarations {
            if let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                ..
            } = decl
            {
                expected.push(behavior, parent, parent_type_args);
            }
        }
        expected
    }

    fn push(&mut self, owner: &str, behavior: &str, type_args: &[AstType]) {
        self.edges
            .entry(owner.to_string())
            .or_default()
            .push(ExpectedBehaviorEdge::new(behavior, type_args));
    }

    fn edges_for(&self, owner: &str) -> &[ExpectedBehaviorEdge] {
        self.edges.get(owner).map(Vec::as_slice).unwrap_or(&[])
    }
}

struct ExpectedBehaviorAssociations {
    impls: ExpectedBehaviorEdges,
    required: ExpectedBehaviorEdges,
}

impl ExpectedBehaviorAssociations {
    fn new(program: &ast::Program) -> Self {
        let mut expected = Self {
            impls: ExpectedBehaviorEdges::default(),
            required: ExpectedBehaviorEdges::default(),
        };
        for decl in &program.declarations {
            match decl {
                Declaration::ImplBlock {
                    type_name,
                    behavior: Some(behavior),
                    behavior_type_args,
                    ..
                } => {
                    expected.impls.push(type_name, behavior, behavior_type_args);
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    ..
                } => {
                    expected
                        .required
                        .push(type_name, behavior, behavior_type_args);
                }
                _ => {}
            }
        }
        expected
    }
}

// ── TypeChecker ───────────────────────────────────────────────────

pub struct TypeChecker {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    methods: HashMap<String, FuncInfo>, // key: "TypeName.method_name"
    behaviors: HashMap<String, BehaviorInfo>,
    behavior_extends: HashMap<String, Vec<BehaviorParentRef>>,
    behavior_extends_spans: HashMap<String, Span>,
    behavior_impls: HashSet<(String, String)>,
    generic_functions: HashMap<String, GenericFunctionTemplate>,
    generic_methods: HashMap<String, GenericFunctionTemplate>,
    specialized_functions: Vec<TypedFunction>,
    specializations_seen: HashSet<String>,
    specialized_types: Vec<TypedTypeDef>,
    specialized_types_seen: HashSet<String>,
    type_substitutions: Vec<HashMap<String, Type>>,
    imports: HashMap<String, Vec<String>>, // imported name -> source module path
    scopes: Vec<Scope>,
    diagnostics: Vec<Diagnostic>,
    current_return_type: Option<Type>,
    current_self_type: Option<Type>,
    pending_defers: Vec<TypedExpression>,
    resolver_backed_collection: bool,
    resolver_behavior_impl_refs: HashMap<String, VecDeque<BehaviorRefMetadata>>,
    resolver_behavior_required_refs: HashMap<String, VecDeque<BehaviorRefMetadata>>,
    resolver_missing_behavior_impl_refs: HashSet<String>,
    resolver_missing_behavior_required_refs: HashSet<String>,
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
            behaviors: HashMap::new(),
            behavior_extends: HashMap::new(),
            behavior_extends_spans: HashMap::new(),
            behavior_impls: HashSet::new(),
            generic_functions: HashMap::new(),
            generic_methods: HashMap::new(),
            specialized_functions: Vec::new(),
            specializations_seen: HashSet::new(),
            specialized_types: Vec::new(),
            specialized_types_seen: HashSet::new(),
            type_substitutions: Vec::new(),
            imports: HashMap::new(),
            scopes: vec![Scope::new()], // global scope
            diagnostics: Vec::new(),
            current_return_type: None,
            current_self_type: None,
            pending_defers: Vec::new(),
            resolver_backed_collection: false,
            resolver_behavior_impl_refs: HashMap::new(),
            resolver_behavior_required_refs: HashMap::new(),
            resolver_missing_behavior_impl_refs: HashSet::new(),
            resolver_missing_behavior_required_refs: HashSet::new(),
        }
    }

    /// Type-check a program and produce a TypedProgram.
    pub fn check_program(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 1: Collect type definitions and function signatures
        self.collect_declarations(&program.declarations);
        self.validate_collected_declaration_semantics(&program.declarations, None);
        self.check_program_after_collection(program)
    }

    fn check_program_after_collection(
        &mut self,
        program: &ast::Program,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        // Phase 2: Check function bodies and produce typed AST
        let mut functions = Vec::new();
        let mut types = Vec::new();
        let mut globals = Vec::new();
        let mut entry_point = None;

        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
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
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
                    let full_name = Self::method_key(type_name, method_name);
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
                    name,
                    type_params,
                    fields,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
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
                    type_params,
                    variants,
                    span,
                    ..
                } => {
                    if !type_params.is_empty() {
                        continue;
                    }
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
                Declaration::Behavior { .. } => {}
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    ..
                } => {
                    for method in methods {
                        if let Declaration::Function {
                            name,
                            type_params,
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            if !type_params.is_empty() {
                                continue;
                            }
                            let full_name = Self::method_key(type_name, name);
                            self.current_self_type =
                                Some(self.resolve_type(&AstType::Named(type_name.clone())));
                            match self.check_function(&full_name, params, return_type, body, span) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
                            self.current_self_type = None;
                        }
                    }

                    if let Some(behavior) = behavior {
                        for default in self.behavior_default_methods_for_impl(
                            type_name,
                            behavior,
                            behavior_type_args,
                            methods,
                        ) {
                            let full_name = Self::method_key(type_name, &default.name);
                            self.current_self_type =
                                Some(self.resolve_type(&AstType::Named(type_name.clone())));
                            match self.check_function(
                                &full_name,
                                &default.params,
                                &default.return_type,
                                &default.body,
                                &default.span,
                            ) {
                                Ok(func) => functions.push(func),
                                Err(d) => self.diagnostics.push(d),
                            }
                            self.current_self_type = None;
                        }
                    }
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

        functions.append(&mut self.specialized_functions);
        types.append(&mut self.specialized_types);

        Ok(TypedProgram {
            functions,
            types,
            globals,
            entry_point,
        })
    }

    pub fn check_program_with_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.validate_resolver_symbols(program, symbols);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }
        self.collect_resolver_imports(symbols);
        self.collect_declarations_with_symbols(&program.declarations, symbols);
        self.check_program_after_collection(program)
    }

    pub fn check_module_graph_entry(
        &mut self,
        graph: &ResolvedModuleGraph,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        let Some(entry) = graph.module(graph.entry) else {
            self.diagnostics.push(Diagnostic::error(
                "E0232",
                format!("module graph missing entry module {:?}", graph.entry),
                Span::dummy(),
            ));
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        };

        let mut modules = graph.modules().values().collect::<Vec<_>>();
        modules.sort_by_key(|module| module.info.id.0);

        let mut dependency_programs = Vec::new();
        for module in modules {
            if module.info.id == graph.entry {
                continue;
            }

            let mut checker = TypeChecker::new();
            match checker.check_module_graph_module(graph, module) {
                Ok(typed) => dependency_programs.push(typed),
                Err(diags) => self.diagnostics.extend(diags),
            }
        }

        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }

        let mut typed = self.check_module_graph_module(graph, entry)?;
        for mut dependency in dependency_programs {
            typed.functions.append(&mut dependency.functions);
            typed.types.append(&mut dependency.types);
            typed.globals.append(&mut dependency.globals);
        }
        Ok(typed)
    }

    fn check_module_graph_module(
        &mut self,
        graph: &ResolvedModuleGraph,
        module: &ResolvedModule,
    ) -> Result<TypedProgram, Vec<Diagnostic>> {
        self.validate_resolver_symbols(&module.program, &module.symbols);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }

        self.collect_module_graph_imports(graph, module);
        if self.diagnostics.iter().any(|diag| diag.is_error()) {
            return Err(self
                .diagnostics
                .iter()
                .filter(|diag| diag.is_error())
                .cloned()
                .collect());
        }

        self.collect_declarations_with_symbols(&module.program.declarations, &module.symbols);
        self.check_program_after_collection(&module.program)
    }

    /// Get all diagnostics (errors + warnings).
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    // ── Phase 1: Collect ──────────────────────────────────────────

    fn collect_declarations(&mut self, decls: &[Declaration]) {
        self.collect_ast_behavior_declarations(decls);
        self.validate_ast_behavior_generic_bounds(decls);
        self.validate_ast_behavior_extends(decls);
        self.collect_type_declarations(decls);
        self.validate_ast_callable_generic_bounds(decls);
        self.collect_callable_declarations(decls);
        self.collect_impl_block_declarations(decls);
        self.collect_ast_import_declarations(decls);
    }

    fn collect_ast_import_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let Declaration::Import {
                names, module_path, ..
            } = decl
            else {
                continue;
            };

            for name in names {
                self.imports.insert(name.clone(), module_path.clone());
            }
        }
    }

    fn collect_impl_block_declarations(&mut self, decls: &[Declaration]) {
        if self.resolver_backed_collection {
            self.collect_resolver_backed_impl_block_templates(decls);
        } else {
            self.collect_ast_impl_block_declarations(decls);
        }
    }

    fn collect_ast_impl_block_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
                ..
            } = decl
            else {
                continue;
            };

            for method in methods {
                self.collect_impl_method_signature(type_name, method);
            }
            if let Some(behavior) = behavior {
                self.collect_behavior_default_method_signatures(
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                );
            }
        }
    }

    fn collect_resolver_backed_impl_block_templates(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let Declaration::ImplBlock {
                type_name, methods, ..
            } = decl
            else {
                continue;
            };

            for method in methods {
                self.collect_resolver_backed_impl_method_template(type_name, method);
            }
        }
    }

    fn validate_ast_callable_generic_bounds(&mut self, decls: &[Declaration]) {
        if self.resolver_backed_collection {
            return;
        }

        for decl in decls {
            match decl {
                Declaration::Function { type_params, .. }
                | Declaration::Method { type_params, .. } => {
                    self.validate_generic_bounds(type_params);
                }
                _ => {}
            }
        }
    }

    fn collect_callable_declarations(&mut self, decls: &[Declaration]) {
        if self.resolver_backed_collection {
            self.collect_resolver_backed_callable_templates(decls);
        } else {
            self.collect_ast_callable_declarations(decls);
        }
    }

    fn collect_ast_callable_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.functions.insert(
                        name.clone(),
                        func_info_from_ast_signature(
                            name.clone(),
                            type_params,
                            params,
                            return_type,
                        ),
                    );
                    if let Some(template) = generic_template_from_type_params(
                        type_params,
                        params,
                        return_type,
                        body,
                        *span,
                    ) {
                        self.generic_functions.insert(name.clone(), template);
                    }
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    let key = Self::method_key(type_name, method_name);
                    self.methods.insert(
                        key.clone(),
                        func_info_from_ast_signature(key.clone(), type_params, params, return_type),
                    );
                    if let Some(template) = generic_template_from_type_params(
                        type_params,
                        params,
                        return_type,
                        body,
                        *span,
                    ) {
                        self.generic_methods.insert(key, template);
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_resolver_backed_callable_templates(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Function {
                    name,
                    type_params,
                    params,
                    body,
                    span,
                    ..
                } => {
                    if let Some(template) = generic_template_body_stub_from_type_params(
                        type_params,
                        params,
                        body,
                        *span,
                    ) {
                        self.generic_functions.insert(name.clone(), template);
                    }
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    body,
                    span,
                    ..
                } => {
                    if let Some(template) = generic_template_body_stub_from_type_params(
                        type_params,
                        params,
                        body,
                        *span,
                    ) {
                        self.generic_methods
                            .insert(Self::method_key(type_name, method_name), template);
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_type_declarations(&mut self, decls: &[Declaration]) {
        if self.resolver_backed_collection {
            return;
        }

        self.validate_ast_type_generic_bounds(decls);
        self.collect_ast_type_declarations(decls);
    }

    fn validate_ast_type_generic_bounds(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Struct { type_params, .. } | Declaration::Enum { type_params, .. } => {
                    self.validate_generic_bounds(type_params);
                }
                _ => {}
            }
        }
    }

    fn collect_ast_type_declarations(&mut self, decls: &[Declaration]) {
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
                        struct_info_from_ast_fields(name.clone(), type_params, fields),
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
                        enum_info_from_ast_variants(name.clone(), type_params, variants),
                    );
                }
                _ => {}
            }
        }
    }

    fn collect_ast_behavior_declarations(&mut self, decls: &[Declaration]) {
        if self.resolver_backed_collection {
            self.collect_resolver_backed_behavior_declaration_stubs(decls);
        } else {
            self.collect_ast_behavior_declaration_signatures(decls);
        }
    }

    fn collect_ast_behavior_declaration_signatures(&mut self, decls: &[Declaration]) {
        for decl in decls {
            if let Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } = decl
            {
                self.behaviors.insert(
                    name.clone(),
                    behavior_info_from_ast_methods(name.clone(), type_params, methods),
                );
            }
        }
    }

    fn collect_resolver_backed_behavior_declaration_stubs(&mut self, decls: &[Declaration]) {
        for decl in decls {
            if let Declaration::Behavior { name, methods, .. } = decl {
                self.behaviors.insert(
                    name.clone(),
                    behavior_info_for_resolver_backed_stub(name.clone(), methods),
                );
            }
        }
    }

    fn validate_ast_behavior_generic_bounds(&mut self, decls: &[Declaration]) {
        if self.resolver_backed_collection {
            return;
        }

        for decl in decls {
            if let Declaration::Behavior { type_params, .. } = decl {
                self.validate_generic_bounds(type_params);
            }
        }
    }

    fn validate_ast_behavior_extends(&mut self, decls: &[Declaration]) {
        self.validate_self_type_contexts(decls);

        if self.resolver_backed_collection {
            return;
        }

        self.validate_ast_behavior_extends_declarations(decls);
    }

    fn validate_ast_behavior_extends_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            if let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } = decl
            {
                self.check_behavior_extends(behavior, parent, parent_type_args, *span);
            }
        }
        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();
    }

    fn collect_declarations_with_symbols(&mut self, decls: &[Declaration], symbols: &SymbolTable) {
        self.with_resolver_backed_collection(|checker| checker.collect_declarations(decls));

        self.collect_resolver_declaration_metadata(decls, symbols);
        self.collect_resolver_behavior_impl_metadata(decls, symbols);
        self.validate_resolver_collected_declaration_semantics(decls, symbols);
        self.clear_resolver_behavior_ref_state();
        self.refresh_resolver_type_behavior_impls(decls, symbols);
    }

    fn collect_resolver_declaration_metadata(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        self.collect_resolver_callable_declaration_metadata(decls, symbols);
        self.collect_resolver_type_declaration_metadata(decls, symbols);
        self.collect_resolver_behavior_declaration_metadata_pass(decls, symbols);
    }

    fn collect_resolver_behavior_declaration_metadata_pass(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        for decl in decls {
            if let Declaration::Behavior { name, span, .. } = decl {
                self.collect_resolver_behavior_declaration_metadata(symbols, name, *span);
            }
        }
    }

    fn collect_resolver_type_declaration_metadata(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        for decl in decls {
            match decl {
                Declaration::Struct {
                    name, fields, span, ..
                } => {
                    self.collect_resolver_struct_declaration_metadata(symbols, name, fields, *span);
                }
                Declaration::Enum { name, span, .. } => {
                    self.collect_resolver_enum_declaration_metadata(symbols, name, *span);
                }
                _ => {}
            }
        }
    }

    fn collect_resolver_callable_declaration_metadata(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        for decl in decls {
            match decl {
                Declaration::Function { name, span, .. } => {
                    self.collect_resolver_function_declaration_metadata(symbols, name, *span);
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    span,
                    ..
                } => {
                    self.collect_resolver_method_declaration_metadata(
                        symbols,
                        type_name,
                        method_name,
                        *span,
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: None,
                    methods,
                    ..
                } => {
                    self.collect_resolver_type_impl_declaration_metadata(
                        symbols, type_name, methods,
                    );
                }
                _ => {}
            }
        }
    }

    fn collect_resolver_function_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        self.collect_resolver_function_signature(symbols, name, span);
    }

    fn collect_resolver_method_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        method_name: &str,
        span: Span,
    ) {
        self.collect_resolver_method_signature(symbols, type_name, method_name, span);
    }

    fn collect_resolver_type_impl_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            if let Declaration::Function { name, span, .. } = method {
                self.collect_resolver_method_signature(symbols, type_name, name, *span);
            }
        }
    }

    fn collect_resolver_struct_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        let restored_name =
            self.collect_resolver_type_behavior_refs_for_declaration(symbols, name, span);
        self.collect_resolver_struct_fields(symbols, &restored_name, fields);
    }

    fn collect_resolver_enum_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        let restored_name =
            self.collect_resolver_type_behavior_refs_for_declaration(symbols, name, span);
        self.collect_resolver_enum_variants(symbols, &restored_name);
    }

    fn collect_resolver_behavior_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        self.collect_resolver_behavior_declaration(symbols, name, span);
    }

    fn collect_resolver_behavior_impl_metadata(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        self.with_resolver_backed_collection(|checker| {
            checker.for_each_resolver_behavior_impl_block(
                decls,
                symbols,
                |checker, ast_type_name, type_name, behavior, behavior_type_args, methods| {
                    checker.collect_resolver_behavior_impl_method_signatures(
                        symbols,
                        ast_type_name,
                        type_name,
                        behavior,
                        behavior_type_args,
                        methods,
                    );
                },
            );

            checker.validate_collected_behavior_extends_semantics();

            checker.for_each_resolver_behavior_impl_block(
                decls,
                symbols,
                |checker, _ast_type_name, type_name, behavior, behavior_type_args, methods| {
                    checker.collect_behavior_default_method_signatures(
                        type_name,
                        behavior,
                        behavior_type_args,
                        methods,
                    );
                },
            );
        });
    }

    fn validate_resolver_collected_declaration_semantics(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        self.with_resolver_backed_collection(|checker| {
            checker.validate_collected_declaration_semantics(decls, Some(symbols));
        });
    }

    fn clear_resolver_behavior_ref_state(&mut self) {
        self.resolver_behavior_impl_refs.clear();
        self.resolver_behavior_required_refs.clear();
        self.resolver_missing_behavior_impl_refs.clear();
        self.resolver_missing_behavior_required_refs.clear();
    }

    fn refresh_resolver_type_behavior_impls(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
    ) {
        self.for_each_resolver_type_declaration(decls, symbols, |checker, restored_name| {
            checker.collect_resolver_type_behavior_impls(symbols, restored_name);
        });
    }

    fn with_resolver_backed_collection(&mut self, collect: impl FnOnce(&mut Self)) {
        let previous = self.resolver_backed_collection;
        self.resolver_backed_collection = true;
        collect(self);
        self.resolver_backed_collection = previous;
    }

    fn for_each_resolver_behavior_impl_block(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
        mut visit: impl FnMut(&mut Self, &str, &str, &str, &[AstType], &[Declaration]),
    ) {
        for decl in decls {
            if let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                ..
            } = decl
            {
                let restored_type_name = self.resolver_impl_type_name_for(
                    symbols,
                    type_name,
                    methods,
                    Some((behavior, behavior_type_args)),
                );
                visit(
                    self,
                    type_name,
                    &restored_type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                );
            }
        }
    }

    fn for_each_resolver_type_declaration(
        &mut self,
        decls: &[Declaration],
        symbols: &SymbolTable,
        mut visit: impl FnMut(&mut Self, &str),
    ) {
        for decl in decls {
            match decl {
                Declaration::Struct { name, span, .. } | Declaration::Enum { name, span, .. } => {
                    let restored_name =
                        Self::resolver_symbol_name_for(symbols, Namespace::Type, name, *span);
                    visit(self, &restored_name);
                }
                _ => {}
            }
        }
    }

    fn collect_resolver_type_behavior_refs_for_declaration(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) -> String {
        let restored_name = Self::resolver_symbol_name_for(symbols, Namespace::Type, name, span);
        self.collect_resolver_type_behavior_impl_refs(symbols, &restored_name);
        self.collect_resolver_type_behavior_requires(symbols, &restored_name);
        restored_name
    }

    fn collect_resolver_behavior_declaration(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        let restored_name =
            Self::resolver_symbol_name_for(symbols, Namespace::Behavior, name, span);
        self.rekey_behavior_declaration(name, &restored_name);
        self.collect_resolver_behavior_methods(symbols, &restored_name);
        self.collect_resolver_behavior_parents(symbols, &restored_name);
    }

    fn rekey_behavior_declaration(&mut self, old_name: &str, new_name: &str) {
        if old_name == new_name {
            return;
        }
        if let Some(info) = self.behaviors.remove(old_name) {
            self.behaviors.insert(
                new_name.to_string(),
                BehaviorInfo {
                    name: new_name.to_string(),
                    ..info
                },
            );
        }
    }

    fn validate_collected_declaration_semantics(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        for decl in decls {
            if let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                span,
                ..
            } = decl
            {
                self.validate_collected_behavior_impl_declaration(
                    symbols,
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    *span,
                );
            }
        }

        for decl in decls {
            if let Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } = decl
            {
                self.validate_collected_behavior_requires_declaration(
                    symbols,
                    type_name,
                    behavior,
                    behavior_type_args,
                    *span,
                );
            }
        }

        self.validate_generic_type_references(decls, symbols);
        self.validate_struct_field_defaults(decls, symbols);
    }

    fn validate_collected_behavior_impl_declaration(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
    ) {
        let restored_type_name = symbols
            .map(|symbols| {
                self.resolver_impl_type_name_for(
                    symbols,
                    type_name,
                    methods,
                    Some((behavior, behavior_type_args)),
                )
            })
            .unwrap_or_else(|| type_name.to_string());
        self.check_behavior_impl(
            &restored_type_name,
            behavior,
            behavior_type_args,
            methods,
            span,
            symbols,
        );
    }

    fn validate_collected_behavior_requires_declaration(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        let type_name = symbols
            .map(|symbols| {
                self.resolver_required_type_name_for(
                    symbols,
                    type_name,
                    behavior,
                    behavior_type_args,
                )
            })
            .unwrap_or_else(|| type_name.to_string());
        self.check_behavior_requires(&type_name, behavior, behavior_type_args, span);
    }

    fn validate_struct_field_defaults(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            self.validate_resolver_struct_field_default_declarations(decls, symbols);
        } else {
            self.validate_ast_struct_field_default_declarations(decls);
        }
    }

    fn validate_resolver_struct_field_default_declarations(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        for decl in decls {
            let Declaration::Struct { name, span, .. } = decl else {
                continue;
            };

            self.validate_resolver_struct_field_defaults(symbols, name, *span);
        }
    }

    fn validate_ast_struct_field_default_declarations(&mut self, decls: &[Declaration]) {
        for decl in decls {
            let Declaration::Struct {
                type_params,
                fields,
                ..
            } = decl
            else {
                continue;
            };

            self.validate_ast_struct_field_defaults(!type_params.is_empty(), fields);
        }
    }

    fn validate_resolver_struct_field_defaults(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        let Some(info) = self.structs.get(&restored_name).cloned() else {
            return;
        };
        if !info.type_params.is_empty() {
            return;
        }
        for (field_name, expected) in &info.fields {
            if let Some(default) = info.field_defaults.get(field_name) {
                self.validate_struct_field_default(field_name, expected, default);
            }
        }
    }

    fn validate_ast_struct_field_defaults(
        &mut self,
        has_type_params: bool,
        fields: &[StructField],
    ) {
        if has_type_params {
            return;
        }
        for field in fields {
            if let Some(default) = &field.default {
                self.validate_struct_field_default(&field.name, &field.ty, default);
            }
        }
    }

    fn validate_struct_field_default(
        &mut self,
        field_name: &str,
        expected: &AstType,
        default: &Expression,
    ) {
        let expected = self.resolve_type(expected);
        self.push_scope();
        let actual = self.check_expr(default);
        self.pop_scope();

        let Ok(actual) = actual else {
            self.diagnostics.push(actual.expect_err("checked error"));
            return;
        };
        let actual_ty = if (expected.is_integer()
            && matches!(actual.kind, TypedExprKind::IntLiteral(_)))
            || (expected.is_float() && matches!(actual.kind, TypedExprKind::FloatLiteral(_)))
        {
            expected.clone()
        } else {
            actual.ty.clone()
        };

        if !self.types_compatible(&expected, &actual_ty) {
            self.diagnostics.push(Diagnostic::error(
                "E3073",
                format!(
                    "field `{}` default expects `{}`, found `{}`",
                    field_name,
                    expected.display_name(),
                    actual.ty.display_name()
                ),
                actual.span,
            ));
        }
    }

    fn validate_collected_behavior_extends_semantics(&mut self) {
        let behavior_extends: Vec<(String, Vec<BehaviorParentRef>, Span)> = self
            .behavior_extends
            .iter()
            .map(|(behavior, parents)| {
                (
                    behavior.clone(),
                    parents.clone(),
                    self.behavior_extends_spans
                        .get(behavior)
                        .copied()
                        .unwrap_or_else(Span::dummy),
                )
            })
            .collect();

        for (behavior, parents, span) in behavior_extends {
            let scoped_type_params: HashSet<String> = self
                .behaviors
                .get(&behavior)
                .map(|info| info.type_params.iter().cloned().collect())
                .unwrap_or_default();
            for parent in parents {
                self.behavior_type_arg_substitutions(
                    &parent.behavior,
                    &parent.type_args,
                    &scoped_type_params,
                    span,
                );
            }
        }

        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();
    }

    fn collect_resolver_value_signature(&mut self, symbols: &SymbolTable, name: &str) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            self.remove_callable_signature(name);
            return;
        };
        let (Some(parameter_names), Some(parameter_types), Some(return_type)) = (
            symbol.parameter_names.as_ref(),
            symbol.parameter_types.as_ref(),
            symbol.return_type.as_ref(),
        ) else {
            self.remove_callable_signature(name);
            return;
        };
        let info = func_info_from_resolver_signature(
            name.to_string(),
            symbol,
            parameter_names,
            parameter_types,
            return_type,
        );
        self.insert_callable_signature(name, info);
        self.collect_resolver_generic_template_signature(
            name,
            symbol.type_parameter_names.as_deref().unwrap_or(&[]),
            parameter_names,
            parameter_types,
            return_type,
        );
    }

    fn remove_callable_signature(&mut self, name: &str) {
        self.functions.remove(name);
        self.methods.remove(name);
        self.generic_functions.remove(name);
        self.generic_methods.remove(name);
    }

    fn insert_callable_signature(&mut self, name: &str, info: FuncInfo) {
        self.functions.remove(name);
        self.methods.remove(name);
        if is_method_signature_key(name) {
            self.methods.insert(name.to_string(), info);
        } else {
            self.functions.insert(name.to_string(), info);
        }
    }

    fn generic_callable_template_mut(
        &mut self,
        name: &str,
    ) -> Option<&mut GenericFunctionTemplate> {
        if is_method_signature_key(name) {
            self.generic_methods.get_mut(name)
        } else {
            self.generic_functions.get_mut(name)
        }
    }

    fn collect_resolver_generic_template_signature(
        &mut self,
        name: &str,
        type_parameter_names: &[String],
        parameter_names: &[String],
        parameter_types: &[AstType],
        return_type: &AstType,
    ) {
        let Some(template) = self.generic_callable_template_mut(name) else {
            return;
        };
        template.type_params = type_parameter_names.to_vec();
        template.params = parameter_names
            .iter()
            .cloned()
            .zip(parameter_types.iter().cloned())
            .enumerate()
            .map(|(index, (param_name, ty))| {
                let existing = template.params.get(index).cloned();
                Param {
                    name: param_name,
                    ty,
                    mutable: existing.as_ref().is_some_and(|param| param.mutable),
                    span: existing.map(|param| param.span).unwrap_or(template.span),
                }
            })
            .collect();
        template.return_type = match return_type {
            AstType::Void => None,
            ty => Some(ty.clone()),
        };
    }

    fn collect_resolver_method_signature(
        &mut self,
        symbols: &SymbolTable,
        type_name: &str,
        method_name: &str,
        span: Span,
    ) {
        let ast_key = Self::method_key(type_name, method_name);
        let restored_key =
            Self::resolver_method_signature_name_for(symbols, &ast_key, type_name, span);

        self.collect_resolver_callable_signature_for_key(symbols, &ast_key, &restored_key);
    }

    fn collect_resolver_function_signature(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::resolver_symbol_name_for(symbols, Namespace::Value, name, span);

        self.collect_resolver_callable_signature_for_key(symbols, name, &restored_name);
    }

    fn collect_resolver_callable_signature_for_key(
        &mut self,
        symbols: &SymbolTable,
        ast_key: &str,
        restored_key: &str,
    ) {
        if restored_key != ast_key {
            self.rekey_callable_template(ast_key, restored_key);
            self.remove_callable_signature(ast_key);
        }
        self.collect_resolver_value_signature(symbols, restored_key);
    }

    fn rekey_callable_template(&mut self, old_key: &str, new_key: &str) {
        let template = self
            .generic_functions
            .remove(old_key)
            .or_else(|| self.generic_methods.remove(old_key));

        if let Some(template) = template {
            if is_method_signature_key(new_key) {
                self.generic_methods.insert(new_key.to_string(), template);
            } else {
                self.generic_functions.insert(new_key.to_string(), template);
            }
        }
    }

    fn resolver_symbol_name_for(
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .lookup(namespace, name)
            .or_else(|| Self::resolver_symbol_by_span(symbols, namespace, span))
            .map(|symbol| symbol.name.clone())
            .unwrap_or_else(|| name.to_string())
    }

    fn resolver_method_signature_name_for(
        symbols: &SymbolTable,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .lookup(Namespace::Value, ast_key)
            .or_else(|| {
                let prefix = format!("{type_name}.");
                Self::resolver_symbol_by_span_matching(symbols, Namespace::Value, span, |symbol| {
                    symbol.name.starts_with(&prefix)
                })
            })
            .or_else(|| Self::resolver_method_signature_symbol_by_span(symbols, span))
            .map(|symbol| symbol.name.clone())
            .unwrap_or_else(|| ast_key.to_string())
    }

    fn resolver_symbol_by_span(
        symbols: &SymbolTable,
        namespace: Namespace,
        span: Span,
    ) -> Option<&crate::resolver::Symbol> {
        Self::resolver_symbol_by_span_matching(symbols, namespace, span, |_| true)
    }

    fn resolver_method_signature_symbol_by_span(
        symbols: &SymbolTable,
        span: Span,
    ) -> Option<&crate::resolver::Symbol> {
        Self::resolver_symbol_by_span_matching(symbols, Namespace::Value, span, |symbol| {
            is_method_signature_key(&symbol.name)
        })
    }

    fn resolver_symbol_by_span_matching(
        symbols: &SymbolTable,
        namespace: Namespace,
        span: Span,
        matches: impl Fn(&crate::resolver::Symbol) -> bool,
    ) -> Option<&crate::resolver::Symbol> {
        symbols.symbols().iter().find(|symbol| {
            symbol.namespace == namespace && symbol.definition_span == span && matches(symbol)
        })
    }

    fn resolver_impl_type_name_for(
        &self,
        symbols: &SymbolTable,
        type_name: &str,
        methods: &[Declaration],
        behavior_ref: Option<(&str, &[AstType])>,
    ) -> String {
        if symbols.lookup(Namespace::Type, type_name).is_some() {
            return type_name.to_string();
        }

        if let Some(type_name) = methods.iter().find_map(|method| {
            let Declaration::Function { span, .. } = method else {
                return None;
            };
            Self::resolver_method_signature_symbol_by_span(symbols, *span)
                .and_then(|symbol| method_signature_receiver_name(&symbol.name).map(str::to_string))
        }) {
            return type_name;
        }

        if let Some((behavior, behavior_type_args)) = behavior_ref {
            if let Some(candidate) = self.resolver_behavior_ref_owner_for(
                &self.resolver_behavior_impl_refs,
                &self.resolver_missing_behavior_impl_refs,
                behavior,
                behavior_type_args,
            ) {
                return candidate;
            }
        }

        type_name.to_string()
    }

    fn resolver_required_type_name_for(
        &self,
        symbols: &SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> String {
        if symbols.lookup(Namespace::Type, type_name).is_some() {
            return type_name.to_string();
        }

        if let Some(candidate) = self.resolver_behavior_ref_owner_for(
            &self.resolver_behavior_required_refs,
            &self.resolver_missing_behavior_required_refs,
            behavior,
            behavior_type_args,
        ) {
            return candidate;
        }

        type_name.to_string()
    }

    fn resolver_behavior_ref_owner_for(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        missing_refs: &HashSet<String>,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<String> {
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);
        self.unique_behavior_ref_owner_for_key(refs_by_type, &behavior_key)
            .or_else(|| self.unique_behavior_ref_owner(refs_by_type, |_| true))
            .or_else(|| Self::unique_owned_candidate(missing_refs.iter().cloned()))
    }

    fn unique_behavior_ref_owner_for_key(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        behavior_key: &str,
    ) -> Option<String> {
        self.unique_behavior_ref_owner(refs_by_type, |reference| {
            self.behavior_reference_matches_key(reference, behavior_key)
        })
    }

    fn behavior_reference_matches_key(
        &self,
        reference: &BehaviorRefMetadata,
        behavior_key: &str,
    ) -> bool {
        self.behavior_reference_key(&reference.name, &reference.type_args) == behavior_key
    }

    fn unique_behavior_ref_owner(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        matches: impl Fn(&BehaviorRefMetadata) -> bool,
    ) -> Option<String> {
        Self::unique_owned_candidate(refs_by_type.iter().filter_map(|(candidate_type, refs)| {
            refs.iter().any(&matches).then_some(candidate_type.clone())
        }))
    }

    fn unique_owned_candidate(mut candidates: impl Iterator<Item = String>) -> Option<String> {
        let candidate = candidates.next()?;
        candidates.next().is_none().then_some(candidate)
    }

    fn collect_resolver_struct_fields(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        ast_fields: &[StructField],
    ) {
        let Some((symbol, field_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                symbol.field_types.as_ref()
            })
        else {
            self.structs.remove(name);
            return;
        };

        self.structs.insert(
            name.to_string(),
            struct_info_from_resolver_fields(
                name.to_string(),
                symbol,
                field_types.to_vec(),
                Self::resolver_struct_field_defaults(field_types, ast_fields),
            ),
        );
    }

    fn resolver_struct_field_defaults(
        fields: &[(String, AstType)],
        ast_fields: &[StructField],
    ) -> HashMap<String, Expression> {
        ast_fields
            .iter()
            .zip(fields.iter())
            .filter_map(|(field, (restored_name, _))| {
                field
                    .default
                    .as_ref()
                    .map(|default| (restored_name.clone(), default.clone()))
            })
            .collect()
    }

    fn collect_resolver_enum_variants(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((symbol, variant_names)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                symbol.variant_names.as_ref()
            })
        else {
            self.enums.remove(name);
            return;
        };

        let variants = variant_names
            .iter()
            .map(|variant_name| {
                (
                    variant_name.clone(),
                    symbols
                        .lookup_variant(name, variant_name)
                        .and_then(|variant| variant.variant_payload_type.clone()),
                )
            })
            .collect();
        self.enums.insert(
            name.to_string(),
            enum_info_from_resolver_variants(name.to_string(), symbol, variants),
        );
    }

    fn collect_resolver_behavior_methods(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((symbol, method_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Behavior, name, |symbol| {
                symbol.behavior_method_types.as_ref()
            })
        else {
            self.behaviors.remove(name);
            return;
        };

        let Some(existing) = self.behaviors.get(name).cloned() else {
            return;
        };
        let mut existing_methods: VecDeque<ast::BehaviorMethod> =
            existing.methods.into_iter().collect();
        let mut methods = Vec::new();
        for (metadata_index, metadata) in method_types.iter().cloned().enumerate() {
            let future_method_names = method_types[metadata_index + 1..]
                .iter()
                .map(|metadata| metadata.name.as_str());
            let method = Self::named_queue_index_preserving_future_front(
                &existing_methods,
                &metadata.name,
                future_method_names,
                |method| method.name.as_str(),
            )
            .and_then(|index| existing_methods.remove(index));
            methods.push(Self::resolver_behavior_method_from_metadata(
                method.as_ref(),
                metadata,
                symbol.definition_span,
            ));
        }
        self.behaviors.insert(
            name.to_string(),
            behavior_info_from_resolver_methods(name.to_string(), symbol, methods),
        );
    }

    fn resolver_behavior_method_from_metadata(
        existing_method: Option<&ast::BehaviorMethod>,
        metadata: BehaviorMethodTypeMetadata,
        span: Span,
    ) -> ast::BehaviorMethod {
        let params = Self::resolver_behavior_method_params(
            existing_method
                .map(|method| method.params.as_slice())
                .unwrap_or(&[]),
            &metadata.parameter_names,
            &metadata.parameter_types,
        );
        let return_type = match metadata.return_type {
            AstType::Void => None,
            ty => Some(ty),
        };
        ast::BehaviorMethod {
            name: metadata.name,
            params,
            return_type,
            default_body: existing_method.and_then(|method| method.default_body.clone()),
            span: existing_method.map(|method| method.span).unwrap_or(span),
        }
    }

    fn resolver_behavior_method_params(
        existing_params: &[Param],
        parameter_names: &[String],
        parameter_types: &[AstType],
    ) -> Vec<Param> {
        parameter_types
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, ty)| match existing_params.get(index).cloned() {
                Some(mut param) => {
                    if let Some(name) = parameter_names.get(index) {
                        param.name = name.clone();
                    }
                    param.ty = ty;
                    param
                }
                None => Param {
                    name: parameter_names.get(index).cloned().unwrap_or_default(),
                    ty,
                    mutable: false,
                    span: Span::dummy(),
                },
            })
            .collect()
    }

    fn collect_resolver_behavior_parents(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((parent_refs, definition_span)) =
            Self::resolver_behavior_refs(symbols, Namespace::Behavior, name, |symbol| {
                &symbol.behavior_parent_refs
            })
            .map(|(refs, symbol)| (refs, symbol.definition_span))
        else {
            return;
        };

        let parents = parent_refs
            .iter()
            .map(|parent| self.behavior_parent_ref_from_metadata(parent))
            .collect();
        self.behavior_extends.insert(name.to_string(), parents);
        self.behavior_extends_spans
            .entry(name.to_string())
            .or_insert(definition_span);
    }

    fn collect_resolver_type_behavior_impls(&mut self, symbols: &SymbolTable, name: &str) {
        self.behavior_impls
            .retain(|(type_name, _)| type_name != name);
        let Some((impl_refs, _)) =
            Self::resolver_behavior_refs(symbols, Namespace::Type, name, |symbol| {
                &symbol.behavior_impl_refs
            })
        else {
            return;
        };

        for behavior in impl_refs {
            self.insert_behavior_impl_ref(name, &behavior.name, &behavior.type_args);
        }
    }

    fn collect_resolver_type_behavior_impl_refs(&mut self, symbols: &SymbolTable, name: &str) {
        Self::collect_resolver_type_behavior_refs(
            symbols,
            name,
            |symbol| &symbol.behavior_impl_refs,
            &mut self.resolver_behavior_impl_refs,
            &mut self.resolver_missing_behavior_impl_refs,
        );
    }

    fn collect_resolver_type_behavior_requires(&mut self, symbols: &SymbolTable, name: &str) {
        Self::collect_resolver_type_behavior_refs(
            symbols,
            name,
            |symbol| &symbol.behavior_required_refs,
            &mut self.resolver_behavior_required_refs,
            &mut self.resolver_missing_behavior_required_refs,
        );
    }

    fn collect_resolver_type_behavior_refs(
        symbols: &SymbolTable,
        name: &str,
        select_refs: impl Fn(&crate::resolver::Symbol) -> &Option<Vec<BehaviorRefMetadata>>,
        collected_refs: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        missing_refs: &mut HashSet<String>,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
            return;
        };

        if let Some(refs) = select_refs(symbol).as_deref() {
            collected_refs.insert(name.to_string(), refs.iter().cloned().collect());
        } else {
            missing_refs.insert(name.to_string());
        }
    }

    fn resolver_behavior_refs<'a>(
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        select_refs: impl Fn(&'a crate::resolver::Symbol) -> &'a Option<Vec<BehaviorRefMetadata>>,
    ) -> Option<(&'a [BehaviorRefMetadata], &'a crate::resolver::Symbol)> {
        let (symbol, refs) = Self::resolver_symbol_metadata(symbols, namespace, name, |symbol| {
            select_refs(symbol).as_deref()
        })?;

        Some((refs, symbol))
    }

    fn resolver_symbol_metadata<'a, T: ?Sized>(
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        select_metadata: impl Fn(&'a crate::resolver::Symbol) -> Option<&'a T>,
    ) -> Option<(&'a crate::resolver::Symbol, &'a T)> {
        let symbol = symbols.lookup(namespace, name)?;
        let metadata = select_metadata(symbol)?;
        Some((symbol, metadata))
    }

    fn behavior_parent_ref_from_metadata(
        &self,
        metadata: &BehaviorRefMetadata,
    ) -> BehaviorParentRef {
        self.behavior_parent_ref(&metadata.name, &metadata.type_args)
    }

    fn behavior_parent_ref(&self, behavior: &str, type_args: &[AstType]) -> BehaviorParentRef {
        BehaviorParentRef {
            behavior: behavior.to_string(),
            type_args: type_args.to_vec(),
            key: self.behavior_reference_key(behavior, type_args),
        }
    }

    fn collect_impl_method_signature(&mut self, type_name: &str, method: &Declaration) {
        let Declaration::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = method
        else {
            return;
        };

        self.validate_generic_bounds(type_params);
        let key = Self::method_key(type_name, name);
        self.methods.insert(
            key.clone(),
            func_info_from_ast_signature(key.clone(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, *span)
        {
            self.generic_methods.insert(key, template);
        }
    }

    fn collect_resolver_backed_impl_method_template(
        &mut self,
        type_name: &str,
        method: &Declaration,
    ) {
        let Declaration::Function {
            name,
            type_params,
            params,
            body,
            span,
            ..
        } = method
        else {
            return;
        };
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, *span)
        {
            self.generic_methods
                .insert(Self::method_key(type_name, name), template);
        }
    }

    fn collect_resolver_behavior_impl_method_signatures(
        &mut self,
        symbols: &SymbolTable,
        ast_type_name: &str,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
        let (behavior, behavior_type_args) =
            self.resolver_behavior_impl_ref_parts(type_name, behavior, behavior_type_args);
        let behavior_substitutions =
            self.behavior_type_param_substitutions(behavior, behavior_type_args);
        let mut required_methods: VecDeque<ast::BehaviorMethod> = self
            .behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new())
            .into_iter()
            .collect();

        for method in methods {
            let Declaration::Function { name, span, .. } = method else {
                continue;
            };
            let ast_key = Self::method_key(ast_type_name, name);
            let resolver_owned_key =
                self.resolver_backed_impl_method_key(Some(symbols), &ast_key, type_name, *span);
            let restored_name = self.resolver_backed_behavior_impl_method_signature_name(
                &mut required_methods,
                name,
                resolver_owned_key.as_deref(),
                type_name,
            );
            let Some(restored_name) = restored_name else {
                continue;
            };
            let restored_key = Self::method_key(type_name, &restored_name);
            self.collect_resolver_callable_signature_for_key(symbols, &ast_key, &restored_key);
        }
    }

    fn collect_behavior_default_method_signatures(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
        if self.should_skip_behavior_default_synthesis(type_name) {
            return;
        }
        let (behavior, behavior_type_args) =
            self.resolver_behavior_impl_ref_parts(type_name, behavior, behavior_type_args);
        for default in
            self.behavior_default_methods_for_impl(type_name, behavior, behavior_type_args, methods)
        {
            self.seed_behavior_default_method_signature(type_name, &default);
        }
    }

    fn should_skip_behavior_default_synthesis(&self, type_name: &str) -> bool {
        self.resolver_backed_collection
            && self.resolver_missing_behavior_impl_refs.contains(type_name)
    }

    fn behavior_reference_key(&self, behavior: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            behavior.to_string()
        } else {
            self.mangle_generic_type_name(behavior, type_args)
        }
    }

    fn insert_behavior_impl_ref(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) {
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);
        self.behavior_impls
            .insert((type_name.to_string(), behavior_key));
    }

    fn behavior_type_arg_substitutions(
        &mut self,
        behavior: &str,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) -> Option<HashMap<String, AstType>> {
        let Some(info) = self.behaviors.get(behavior).cloned() else {
            self.diagnostics.push(Diagnostic::error(
                "E6006",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return None;
        };

        if info.type_params.len() != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic behavior `{}` expects {} type arguments, found {}",
                    behavior,
                    info.type_params.len(),
                    type_args.len()
                ),
                span,
            ));
            return None;
        }

        let ast_substitutions: HashMap<String, AstType> = info
            .type_params
            .iter()
            .cloned()
            .zip(type_args.iter().cloned())
            .collect();
        let type_substitutions: HashMap<String, Type> = info
            .type_params
            .iter()
            .zip(type_args.iter())
            .filter_map(|(param, arg)| {
                if ast_type_references_type_param(arg, scoped_type_params) {
                    None
                } else {
                    Some((param.clone(), self.resolve_type(arg)))
                }
            })
            .collect();
        let error_count = self
            .diagnostics
            .iter()
            .filter(|diag| diag.is_error())
            .count();
        self.check_generic_bounds(&info.type_param_bounds, &type_substitutions, span);
        if self
            .diagnostics
            .iter()
            .filter(|diag| diag.is_error())
            .count()
            > error_count
        {
            return None;
        }

        Some(ast_substitutions)
    }

    fn check_behavior_requires(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        let resolver_required_ref = self.resolver_required_ref_for(type_name, behavior);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_required_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_required_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_required_ref.as_ref(), behavior, behavior_type_args);

        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6005",
                format!("undefined type `{}`", type_name),
                span,
            ));
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        let Some(_) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if !self.type_implements_behavior(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6007",
                format!(
                    "type `{}` does not implement required behavior `{}`",
                    type_name, behavior_key
                ),
                span,
            ));
        }
    }

    fn resolver_required_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        Self::pop_resolver_behavior_ref(
            self.resolver_backed_collection,
            &mut self.resolver_behavior_required_refs,
            type_name,
            behavior,
        )
    }

    fn check_behavior_extends(
        &mut self,
        behavior: &str,
        parent: &str,
        parent_type_args: &[AstType],
        span: Span,
    ) {
        if !self.behaviors.contains_key(behavior) {
            self.diagnostics.push(Diagnostic::error(
                "E6006",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return;
        }

        let scoped_type_params: HashSet<String> = self
            .behaviors
            .get(behavior)
            .map(|info| info.type_params.iter().cloned().collect())
            .unwrap_or_default();
        let Some(_) = self.behavior_type_arg_substitutions(
            parent,
            parent_type_args,
            &scoped_type_params,
            span,
        ) else {
            return;
        };

        let parent_ref = self.behavior_parent_ref(parent, parent_type_args);
        let parent_display = behavior_ref_display(parent, parent_type_args);
        let parents = self
            .behavior_extends
            .entry(behavior.to_string())
            .or_default();
        if parents
            .iter()
            .any(|existing| existing.key == parent_ref.key)
        {
            self.diagnostics.push(Diagnostic::error(
                "E6011",
                format!("duplicate behavior inheritance `{behavior}.extends({parent_display})`"),
                span,
            ));
            return;
        }

        parents.push(parent_ref);
        self.behavior_extends_spans
            .entry(behavior.to_string())
            .or_insert(span);
    }

    fn validate_behavior_extends_cycles(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        for behavior in behaviors {
            let mut visiting = HashSet::new();
            let mut visited = HashSet::new();
            if self.behavior_extends_has_cycle(&behavior, &mut visiting, &mut visited) {
                let span = self
                    .behavior_extends_spans
                    .get(&behavior)
                    .copied()
                    .unwrap_or_else(Span::dummy);
                self.diagnostics.push(Diagnostic::error(
                    "E6008",
                    format!("behavior inheritance cycle involving `{}`", behavior),
                    span,
                ));
            }
        }
    }

    fn behavior_extends_has_cycle(
        &self,
        behavior: &str,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visiting.contains(behavior) {
            return true;
        }
        if !visited.insert(behavior.to_string()) {
            return false;
        }

        visiting.insert(behavior.to_string());
        let has_cycle = self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents
                .iter()
                .any(|parent| self.behavior_extends_has_cycle(&parent.key, visiting, visited))
        });
        visiting.remove(behavior);
        has_cycle
    }

    fn validate_behavior_method_coherence(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        let mut diagnostics = Vec::new();

        for behavior in behaviors {
            let mut seen_behaviors = HashSet::new();
            let mut seen_methods = HashMap::new();
            self.collect_behavior_method_coherence_errors(
                &behavior,
                &behavior,
                &HashMap::new(),
                &mut seen_behaviors,
                &mut seen_methods,
                &mut diagnostics,
            );
        }

        self.diagnostics.extend(diagnostics);
    }

    fn collect_behavior_method_coherence_errors(
        &self,
        behavior: &str,
        root_behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen_behaviors: &mut HashSet<String>,
        seen_methods: &mut HashMap<String, ast::BehaviorMethod>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.mark_behavior_seen(behavior, substitutions, seen_behaviors) {
            return;
        }

        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_substitutions =
                    self.behavior_parent_type_param_substitutions(parent, substitutions);
                self.collect_behavior_method_coherence_errors(
                    &parent.behavior,
                    root_behavior,
                    &parent_substitutions,
                    seen_behaviors,
                    seen_methods,
                    diagnostics,
                );
            }
        }

        if let Some(info) = self.behaviors.get(behavior) {
            for method in &info.methods {
                let method = substituted_behavior_method_signature(method, substitutions);

                if let Some(previous) = seen_methods.get(&method.name) {
                    if !behavior_method_signatures_match(previous, &method) {
                        diagnostics.push(Diagnostic::error(
                            "E6009",
                            format!(
                                "conflicting behavior method `{}` inherited by `{}`",
                                method.name, root_behavior
                            ),
                            method.span,
                        ));
                    }
                } else {
                    seen_methods.insert(method.name.clone(), method);
                }
            }
        }
    }

    fn type_implements_behavior(&self, type_name: &str, behavior: &str) -> bool {
        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior.to_string()))
        {
            return true;
        }

        self.behavior_impls
            .iter()
            .any(|(implemented_type, implemented_behavior)| {
                implemented_type == type_name
                    && self.behavior_inherits_from(implemented_behavior, behavior)
            })
    }

    fn behavior_inherits_from(&self, behavior: &str, parent: &str) -> bool {
        self.behavior_inherits_from_inner(behavior, parent, &mut HashSet::new())
    }

    fn behavior_inherits_from_inner(
        &self,
        behavior: &str,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        if !seen.insert(behavior.to_string()) {
            return false;
        }

        self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents.iter().any(|candidate| {
                candidate.key == parent
                    || self.behavior_inherits_from_inner(&candidate.key, parent, seen)
            })
        })
    }

    fn check_behavior_impl(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
        symbols: Option<&SymbolTable>,
    ) {
        let resolver_impl_ref = self.resolver_impl_ref_for(type_name, behavior);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_impl_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_impl_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_impl_ref.as_ref(), behavior, behavior_type_args);

        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6005",
                format!("undefined type `{}`", type_name),
                span,
            ));
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        let Some(behavior_substitutions) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior_key.clone()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E6003",
                format!(
                    "duplicate implementation of behavior `{}` for type `{}`",
                    behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        if let Some(existing) = self.find_overlapping_behavior_impl(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6010",
                format!(
                    "overlapping implementations of behaviors `{}` and `{}` for type `{}`",
                    existing, behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        self.behavior_impls
            .insert((type_name.to_string(), behavior_key.clone()));
        let required_methods =
            self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new());
        let mut unmatched_required: VecDeque<String> = required_methods
            .iter()
            .map(|required| required.name.clone())
            .collect();
        let effective_methods: Vec<(&Declaration, String)> = methods
            .iter()
            .map(|method| {
                let ast_name = match method {
                    Declaration::Function { name, .. } => name.as_str(),
                    _ => "",
                };
                let ast_key = Self::method_key(type_name, ast_name);
                let resolver_owned_name = self.resolver_backed_impl_method_key(
                    symbols,
                    &ast_key,
                    type_name,
                    method.span(),
                );
                let effective_name = self.impl_effective_method_name(
                    &mut unmatched_required,
                    ast_name,
                    resolver_owned_name,
                    type_name,
                );
                (method, effective_name)
            })
            .collect();

        for (method, effective_name) in &effective_methods {
            if let Declaration::Function { span, .. } = method {
                if !required_methods
                    .iter()
                    .any(|required| required.name == *effective_name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E6005",
                        format!(
                            "method `{}` is not declared by behavior `{}`",
                            effective_name, behavior_key
                        ),
                        *span,
                    ));
                }
            }
        }

        for required in &required_methods {
            let Some(actual) =
                effective_methods
                    .iter()
                    .find_map(|(decl, effective_name)| match decl {
                        Declaration::Function {
                            params,
                            return_type,
                            span,
                            ..
                        } if effective_name == &required.name => Some((params, return_type, *span)),
                        _ => None,
                    })
            else {
                if required.default_body.is_some() {
                    continue;
                }
                self.diagnostics.push(Diagnostic::error(
                    "E6001",
                    format!(
                        "type `{}` implementation of `{}` is missing required method `{}`",
                        type_name, behavior_key, required.name
                    ),
                    span,
                ));
                continue;
            };

            let (actual_params, actual_return_type, actual_span) = actual;
            let collected_signature =
                self.resolver_backed_method_signature(type_name, &required.name);
            let actual_param_types: Vec<AstType> = collected_signature
                .map(|info| {
                    info.params
                        .iter()
                        .map(|(_, ty)| ty.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| actual_params.iter().map(|param| param.ty.clone()).collect());
            let actual_return = collected_signature
                .map(|info| info.return_type.clone())
                .unwrap_or_else(|| actual_return_type.clone().unwrap_or(AstType::Void));

            if actual_param_types.len() != required.params.len() {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects {} parameters, found {}",
                        required.name,
                        behavior_key,
                        required.params.len(),
                        actual_param_types.len()
                    ),
                    actual_span,
                ));
                continue;
            }

            for (idx, (expected, actual_ty)) in required
                .params
                .iter()
                .zip(actual_param_types.iter())
                .enumerate()
            {
                if !self.impl_ast_types_compatible(&expected.ty, actual_ty, type_name) {
                    self.diagnostics.push(Diagnostic::error(
                        "E6002",
                        format!(
                            "parameter {} for method `{}` in behavior `{}` expects `{}`, found `{}`",
                            idx + 1,
                            required.name,
                            behavior_key,
                            self.impl_type_display(&expected.ty, type_name),
                            actual_ty.display_name()
                        ),
                        actual_params
                            .get(idx)
                            .map(|param| param.span)
                            .unwrap_or(actual_span),
                    ));
                }
            }

            let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
            if !self.impl_ast_types_compatible(expected_return, &actual_return, type_name) {
                self.diagnostics.push(Diagnostic::error(
                    "E6002",
                    format!(
                        "method `{}` for behavior `{}` expects return `{}`, found `{}`",
                        required.name,
                        behavior_key,
                        self.impl_type_display(expected_return, type_name),
                        actual_return.display_name()
                    ),
                    actual_span,
                ));
            }
        }
    }

    fn resolver_impl_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        Self::pop_resolver_behavior_ref(
            self.resolver_backed_collection,
            &mut self.resolver_behavior_impl_refs,
            type_name,
            behavior,
        )
    }

    fn behavior_ref_parts<'a>(
        resolver_ref: Option<&'a BehaviorRefMetadata>,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        resolver_ref
            .map(|reference| (reference.name.as_str(), reference.type_args.as_slice()))
            .unwrap_or((behavior, behavior_type_args))
    }

    fn pop_resolver_behavior_ref(
        resolver_backed_collection: bool,
        refs_by_type: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get_mut(type_name)?;
        Self::pop_resolver_behavior_ref_from_queue(refs, behavior)
    }

    fn should_skip_missing_resolver_behavior_ref(
        &self,
        resolver_ref: Option<&BehaviorRefMetadata>,
        type_name: &str,
        missing_refs: &HashSet<String>,
    ) -> bool {
        self.resolver_backed_collection
            && resolver_ref.is_none()
            && missing_refs.contains(type_name)
    }

    fn pop_resolver_behavior_ref_from_queue(
        refs: &mut VecDeque<BehaviorRefMetadata>,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        let index = Self::resolver_behavior_ref_queue_index(refs, behavior)?;
        refs.remove(index)
    }

    fn resolver_behavior_impl_ref_for_peek(
        &self,
        type_name: &str,
        behavior: &str,
    ) -> Option<&BehaviorRefMetadata> {
        Self::peek_resolver_behavior_ref(
            self.resolver_backed_collection,
            &self.resolver_behavior_impl_refs,
            type_name,
            behavior,
        )
    }

    fn peek_resolver_behavior_ref<'a>(
        resolver_backed_collection: bool,
        refs_by_type: &'a HashMap<String, VecDeque<BehaviorRefMetadata>>,
        type_name: &str,
        behavior: &str,
    ) -> Option<&'a BehaviorRefMetadata> {
        if !resolver_backed_collection {
            return None;
        }

        let refs = refs_by_type.get(type_name)?;
        Self::resolver_behavior_ref_queue_index(refs, behavior).and_then(|index| refs.get(index))
    }

    fn resolver_behavior_ref_queue_index(
        refs: &VecDeque<BehaviorRefMetadata>,
        behavior: &str,
    ) -> Option<usize> {
        Self::named_queue_index(refs, behavior, |reference| reference.name.as_str())
    }

    fn named_queue_index<T>(
        items: &VecDeque<T>,
        name: &str,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        items
            .iter()
            .position(|item| item_name(item) == name)
            .or_else(|| (!items.is_empty()).then_some(0))
    }

    fn named_queue_index_preserving_future_front<'a, T>(
        items: &VecDeque<T>,
        name: &str,
        future_names: impl IntoIterator<Item = &'a str>,
        item_name: impl Fn(&T) -> &str,
    ) -> Option<usize> {
        if let Some(index) = items.iter().position(|item| item_name(item) == name) {
            return Some(index);
        }

        let front_name = item_name(items.front()?);
        (!future_names
            .into_iter()
            .any(|future_name| future_name == front_name))
        .then_some(0)
    }

    fn resolver_behavior_impl_ref_parts<'a>(
        &'a self,
        type_name: &str,
        behavior: &'a str,
        behavior_type_args: &'a [AstType],
    ) -> (&'a str, &'a [AstType]) {
        match self.resolver_behavior_impl_ref_for_peek(type_name, behavior) {
            Some(implementation) => (
                implementation.name.as_str(),
                implementation.type_args.as_slice(),
            ),
            None => (behavior, behavior_type_args),
        }
    }

    fn find_overlapping_behavior_impl(&self, type_name: &str, behavior: &str) -> Option<String> {
        self.behavior_impls
            .iter()
            .filter(|(implemented_type, _)| implemented_type == type_name)
            .map(|(_, implemented_behavior)| implemented_behavior)
            .find(|implemented_behavior| {
                self.behavior_inherits_from(implemented_behavior, behavior)
                    || self.behavior_inherits_from(behavior, implemented_behavior)
            })
            .cloned()
    }

    fn reject_unspecialized_generic_type(&mut self, type_name: &str, span: Span) -> bool {
        let type_param_count = self
            .structs
            .get(type_name)
            .map(|info| info.type_params.len())
            .or_else(|| self.enums.get(type_name).map(|info| info.type_params.len()))
            .unwrap_or(0);
        if type_param_count == 0 {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            "E6013",
            format!(
                "generic type `{}` expects {} type arguments, found 0",
                type_name, type_param_count
            ),
            span,
        ));
        true
    }

    fn behavior_default_methods_for_impl(
        &self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) -> Vec<DefaultBehaviorMethod> {
        let behavior_substitutions =
            self.behavior_type_param_substitutions(behavior, behavior_type_args);
        self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new())
            .iter()
            .filter(|required| {
                required.default_body.is_some()
                    && !self.impl_methods_include_behavior_method(
                        type_name,
                        methods,
                        &required.name,
                    )
            })
            .filter_map(|required| {
                let body = required.default_body.clone()?;
                Some(DefaultBehaviorMethod {
                    name: required.name.clone(),
                    params: required
                        .params
                        .iter()
                        .map(|param| Param {
                            name: param.name.clone(),
                            ty: concrete_self_ast_type(&param.ty, type_name),
                            mutable: param.mutable,
                            span: param.span,
                        })
                        .collect(),
                    return_type: required
                        .return_type
                        .as_ref()
                        .map(|ty| concrete_self_ast_type(ty, type_name)),
                    body,
                    span: required.span,
                })
            })
            .collect()
    }

    fn seed_behavior_default_method_signature(
        &mut self,
        type_name: &str,
        default: &DefaultBehaviorMethod,
    ) {
        let key = Self::method_key(type_name, &default.name);
        self.methods.insert(
            key.clone(),
            func_info_from_behavior_method(key, &default.params, &default.return_type),
        );
    }

    fn impl_methods_include_behavior_method(
        &self,
        type_name: &str,
        methods: &[Declaration],
        required_name: &str,
    ) -> bool {
        methods
            .iter()
            .any(|decl| matches!(decl, Declaration::Function { name, .. } if name == required_name))
            || (self.resolver_backed_collection
                && self
                    .resolver_backed_method_signature(type_name, required_name)
                    .is_some())
    }

    fn impl_effective_method_name(
        &self,
        unmatched_required: &mut VecDeque<String>,
        ast_name: &str,
        resolver_owned_key: Option<String>,
        type_name: &str,
    ) -> String {
        if let Some(resolver_owned_key) = resolver_owned_key {
            let resolver_owned_name =
                method_signature_method_name_for_receiver(&resolver_owned_key, type_name)
                    .unwrap_or(&resolver_owned_key)
                    .to_string();
            return Self::remove_named_queue_entry(unmatched_required, &resolver_owned_name)
                .unwrap_or(resolver_owned_name);
        }

        if let Some(name) = Self::remove_named_queue_entry(unmatched_required, ast_name) {
            return name;
        }

        if self.resolver_backed_collection {
            if let Some(index) = unmatched_required.iter().position(|required| {
                self.resolver_backed_method_signature(type_name, required)
                    .is_some()
            }) {
                return unmatched_required
                    .remove(index)
                    .unwrap_or_else(|| ast_name.to_string());
            }
        }

        ast_name.to_string()
    }

    fn resolver_backed_behavior_impl_method_signature_name(
        &self,
        required_methods: &mut VecDeque<ast::BehaviorMethod>,
        ast_name: &str,
        resolver_owned_key: Option<&str>,
        type_name: &str,
    ) -> Option<String> {
        if let Some(resolver_owned_key) = resolver_owned_key {
            let resolver_owned_name =
                method_signature_method_name_for_receiver(resolver_owned_key, type_name)
                    .unwrap_or(resolver_owned_key);
            if let Some(index) =
                Self::named_queue_index(required_methods, resolver_owned_name, |required| {
                    required.name.as_str()
                })
            {
                return required_methods.remove(index).map(|required| required.name);
            }
        }

        Self::named_queue_index(required_methods, ast_name, |required| {
            required.name.as_str()
        })
        .and_then(|index| required_methods.remove(index).map(|required| required.name))
    }

    fn resolver_backed_impl_method_key(
        &self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> Option<String> {
        self.resolver_backed_collection
            .then(|| Self::validation_method_key(symbols, ast_key, type_name, span))
    }

    fn resolver_backed_method_signature(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<&FuncInfo> {
        self.resolver_backed_collection
            .then(|| self.methods.get(&Self::method_key(type_name, method_name)))
            .flatten()
    }

    fn method_key(type_name: &str, method_name: &str) -> String {
        method_signature_key(type_name, method_name)
    }

    fn remove_named_queue_entry(items: &mut VecDeque<String>, name: &str) -> Option<String> {
        items
            .iter()
            .position(|item| item == name)
            .and_then(|index| items.remove(index))
    }

    fn behavior_methods_with_inherited_substituted(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        if !self.mark_behavior_seen(behavior, substitutions, seen) {
            return Vec::new();
        }

        let mut methods = Vec::new();
        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_substitutions =
                    self.behavior_parent_type_param_substitutions(parent, substitutions);
                methods.extend(self.behavior_methods_with_inherited_substituted(
                    &parent.behavior,
                    &parent_substitutions,
                    seen,
                ));
            }
        }
        if let Some(info) = self.behaviors.get(behavior) {
            methods.extend(
                info.methods
                    .iter()
                    .map(|method| substituted_behavior_method_signature(method, substitutions)),
            );
        }
        methods
    }

    fn behavior_seen_key(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
    ) -> String {
        let type_args = self
            .behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .filter_map(|param| substitutions.get(param).cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        behavior_ref_display(behavior, &type_args)
    }

    fn mark_behavior_seen(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> bool {
        let behavior_seen_key = self.behavior_seen_key(behavior, substitutions);
        seen.insert(behavior_seen_key)
    }

    fn behavior_methods_for_impl(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen: &mut HashSet<String>,
    ) -> Vec<ast::BehaviorMethod> {
        self.behavior_methods_with_inherited_substituted(behavior, substitutions, seen)
    }

    fn behavior_type_param_substitutions(
        &self,
        behavior: &str,
        type_args: &[AstType],
    ) -> HashMap<String, AstType> {
        self.behaviors
            .get(behavior)
            .map(|info| {
                info.type_params
                    .iter()
                    .cloned()
                    .zip(type_args.iter().cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn behavior_parent_type_param_substitutions(
        &self,
        parent: &BehaviorParentRef,
        substitutions: &HashMap<String, AstType>,
    ) -> HashMap<String, AstType> {
        let parent_type_args: Vec<AstType> = parent
            .type_args
            .iter()
            .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
            .collect();
        self.behavior_type_param_substitutions(&parent.behavior, &parent_type_args)
    }

    fn impl_ast_types_compatible(
        &self,
        expected: &AstType,
        actual: &AstType,
        self_type_name: &str,
    ) -> bool {
        match expected {
            AstType::SelfType => matches!(actual, AstType::Named(name) if name == self_type_name),
            _ => expected == actual,
        }
    }

    fn impl_type_display(&self, ty: &AstType, self_type_name: &str) -> String {
        match ty {
            AstType::SelfType => self_type_name.to_string(),
            _ => ty.display_name(),
        }
    }

    fn validate_generic_bounds(&mut self, type_params: &[ast::TypeParam]) {
        for param in type_params {
            if let Some(bound) = &param.constraint {
                if !self.behaviors.contains_key(bound) {
                    self.diagnostics.push(Diagnostic::error(
                        "E5002",
                        format!(
                            "generic bound `{}` on type parameter `{}` references undefined behavior",
                            bound, param.name
                        ),
                        param.span,
                    ));
                } else {
                    let expected = self
                        .behaviors
                        .get(bound)
                        .map(|info| info.type_params.len())
                        .unwrap_or(0);
                    let found = param.constraint_type_args.len();
                    if expected != found {
                        self.diagnostics.push(Diagnostic::error(
                            "E6012",
                            format!(
                                "generic behavior `{}` expects {} type arguments, found {}",
                                bound, expected, found
                            ),
                            param.span,
                        ));
                    }
                }
            }
        }
    }

    fn validate_generic_type_references(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            self.validate_resolver_backed_type_references(decls, symbols);
            return;
        }

        for decl in decls {
            match decl {
                Declaration::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for field in fields {
                        self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                        if let Some(default) = &field.default {
                            self.validate_generic_expr_type_references(default, &scoped);
                        }
                    }
                }
                Declaration::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for variant in variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                        }
                    }
                }
                Declaration::Function {
                    type_params,
                    params,
                    return_type,
                    body,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for param in params {
                        self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                    }
                    if let Some(return_type) = return_type {
                        self.validate_generic_type_ref_bounds(return_type, &scoped, Span::dummy());
                    }
                    self.validate_generic_expr_type_references(body, &scoped);
                }
                Declaration::Method {
                    type_params,
                    params,
                    return_type,
                    body,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for param in params {
                        self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                    }
                    if let Some(return_type) = return_type {
                        self.validate_generic_type_ref_bounds(return_type, &scoped, Span::dummy());
                    }
                    self.validate_generic_expr_type_references(body, &scoped);
                }
                Declaration::Behavior {
                    type_params,
                    methods,
                    ..
                } => {
                    let scoped = type_param_name_set(type_params);
                    for method in methods {
                        for param in &method.params {
                            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
                        }
                        if let Some(return_type) = &method.return_type {
                            self.validate_generic_type_ref_bounds(
                                return_type,
                                &scoped,
                                method.span,
                            );
                        }
                        if let Some(default_body) = &method.default_body {
                            self.validate_generic_expr_type_references(default_body, &scoped);
                        }
                    }
                }
                Declaration::ImplBlock { methods, .. } => {
                    for method in methods {
                        if let Declaration::Function {
                            type_params,
                            params,
                            return_type,
                            body,
                            ..
                        } = method
                        {
                            let scoped = type_param_name_set(type_params);
                            for param in params {
                                self.validate_generic_type_ref_bounds(
                                    &param.ty, &scoped, param.span,
                                );
                            }
                            if let Some(return_type) = return_type {
                                self.validate_generic_type_ref_bounds(
                                    return_type,
                                    &scoped,
                                    method.span(),
                                );
                            }
                            self.validate_generic_expr_type_references(body, &scoped);
                        }
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
                _ => {}
            }
        }
    }

    fn validate_resolver_backed_type_references(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        for decl in decls {
            match decl {
                Declaration::Struct {
                    name, fields, span, ..
                } => {
                    self.validate_resolver_struct_type_references(symbols, name, fields, *span);
                }
                Declaration::Enum { name, span, .. } => {
                    self.validate_resolver_enum_type_references(symbols, name, *span);
                }
                Declaration::Function {
                    name, body, span, ..
                } => {
                    self.validate_resolver_function_type_references(symbols, name, body, *span);
                }
                Declaration::Method {
                    body,
                    type_name,
                    method_name,
                    span,
                    ..
                } => {
                    let ast_key = Self::method_key(type_name, method_name);
                    self.validate_resolver_method_type_references(
                        symbols, &ast_key, type_name, body, *span,
                    );
                }
                Declaration::Behavior {
                    name,
                    methods,
                    span,
                    ..
                } => {
                    self.validate_resolver_behavior_type_references(symbols, name, methods, *span);
                }
                Declaration::ImplBlock {
                    type_name, methods, ..
                } => {
                    for method in methods {
                        if let Declaration::Function { name, body, .. } = method {
                            let ast_key = Self::method_key(type_name, name);
                            self.validate_resolver_method_type_references(
                                symbols,
                                &ast_key,
                                type_name,
                                body,
                                method.span(),
                            );
                        }
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
                _ => {}
            }
        }
    }

    fn validate_resolver_enum_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        if let Some(scoped) = self.collected_type_type_param_scope(&restored_name) {
            self.validate_collected_enum_type_references(&restored_name, &scoped, span);
        }
    }

    fn validate_resolver_struct_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        if let Some(scoped) = self.collected_type_type_param_scope(&restored_name) {
            self.validate_collected_struct_type_references(&restored_name, &scoped, span);
            for field in fields {
                if let Some(default) = &field.default {
                    self.validate_generic_expr_type_references(default, &scoped);
                }
            }
        }
    }

    fn validate_resolver_behavior_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        methods: &[BehaviorMethod],
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Behavior, name, span);
        if let Some(scoped) = self.collected_behavior_type_param_scope(&restored_name) {
            self.validate_collected_behavior_type_references(&restored_name, &scoped, span);
            for method in methods {
                if let Some(default_body) = &method.default_body {
                    self.validate_generic_expr_type_references(default_body, &scoped);
                }
            }
        }
    }

    fn validate_resolver_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_key = Self::validation_method_key(symbols, ast_key, type_name, span);
        if let Some(scoped) = self.collected_value_type_param_scope(&restored_key) {
            self.validate_collected_value_type_references(&restored_key, &scoped, span);
            self.validate_generic_expr_type_references(body, &scoped);
        }
    }

    fn validate_resolver_function_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Value, name, span);
        if let Some(scoped) = self.collected_value_type_param_scope(&restored_name) {
            self.validate_collected_value_type_references(&restored_name, &scoped, span);
            self.validate_generic_expr_type_references(body, &scoped);
        }
    }

    fn validation_symbol_name(
        symbols: Option<&SymbolTable>,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| Self::resolver_symbol_name_for(symbols, namespace, name, span))
            .unwrap_or_else(|| name.to_string())
    }

    fn validation_method_key(
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .map(|symbols| {
                Self::resolver_method_signature_name_for(symbols, ast_key, type_name, span)
            })
            .unwrap_or_else(|| ast_key.to_string())
    }

    fn collected_value_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .map(|info| info.type_params.iter().cloned().collect())
    }

    fn collected_type_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.structs
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
            .or_else(|| {
                self.enums
                    .get(name)
                    .map(|info| info.type_params.iter().cloned().collect())
            })
    }

    fn collected_behavior_type_param_scope(&self, name: &str) -> Option<HashSet<String>> {
        self.behaviors
            .get(name)
            .map(|info| info.type_params.iter().cloned().collect())
    }

    fn validate_collected_struct_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.structs.get(name).cloned() else {
            return;
        };
        for (_, ty) in &info.fields {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
    }

    fn validate_collected_enum_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.enums.get(name).cloned() else {
            return;
        };
        for (_, payload) in &info.variants {
            if let Some(payload) = payload {
                self.validate_generic_type_ref_bounds(payload, scoped, span);
            }
        }
    }

    fn validate_collected_behavior_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let Some(info) = self.behaviors.get(name).cloned() else {
            return;
        };
        for method in &info.methods {
            for param in &method.params {
                self.validate_generic_type_ref_bounds(&param.ty, scoped, span);
            }
            if let Some(return_type) = &method.return_type {
                self.validate_generic_type_ref_bounds(return_type, scoped, span);
            }
        }
    }

    fn validate_collected_value_type_references(
        &mut self,
        name: &str,
        scoped: &HashSet<String>,
        span: Span,
    ) {
        let info = self
            .functions
            .get(name)
            .or_else(|| self.methods.get(name))
            .cloned();
        let Some(info) = info else {
            return;
        };

        for (_, ty) in &info.params {
            self.validate_generic_type_ref_bounds(ty, scoped, span);
        }
        self.validate_generic_type_ref_bounds(&info.return_type, scoped, span);
    }

    fn validate_self_type_contexts(&mut self, decls: &[Declaration]) {
        for decl in decls {
            match decl {
                Declaration::Struct { fields, .. } => {
                    for field in fields {
                        self.validate_self_type_ref(&field.ty, field.span, false);
                        if let Some(default) = &field.default {
                            self.validate_self_type_expr(default, false);
                        }
                    }
                }
                Declaration::Enum { variants, .. } => {
                    for variant in variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_self_type_ref(payload, variant.span, false);
                        }
                    }
                }
                Declaration::Function {
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.validate_self_type_params(params, false);
                    if let Some(return_type) = return_type {
                        self.validate_self_type_ref(return_type, *span, false);
                    }
                    self.validate_self_type_expr(body, false);
                }
                Declaration::Method {
                    params,
                    return_type,
                    body,
                    span,
                    ..
                } => {
                    self.validate_self_type_params(params, true);
                    if let Some(return_type) = return_type {
                        self.validate_self_type_ref(return_type, *span, true);
                    }
                    self.validate_self_type_expr(body, true);
                }
                Declaration::Behavior { methods, .. } => {
                    for method in methods {
                        self.validate_self_type_params(&method.params, true);
                        if let Some(return_type) = &method.return_type {
                            self.validate_self_type_ref(return_type, method.span, true);
                        }
                        if let Some(default_body) = &method.default_body {
                            self.validate_self_type_expr(default_body, true);
                        }
                    }
                }
                Declaration::ImplBlock {
                    behavior_type_args,
                    methods,
                    span,
                    ..
                } => {
                    for type_arg in behavior_type_args {
                        self.validate_self_type_ref(type_arg, *span, false);
                    }
                    for method in methods {
                        if let Declaration::Function {
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            self.validate_self_type_params(params, true);
                            if let Some(return_type) = return_type {
                                self.validate_self_type_ref(return_type, *span, true);
                            }
                            self.validate_self_type_expr(body, true);
                        }
                    }
                }
                Declaration::Requires {
                    behavior_type_args,
                    span,
                    ..
                } => {
                    for type_arg in behavior_type_args {
                        self.validate_self_type_ref(type_arg, *span, false);
                    }
                }
                Declaration::BehaviorExtends {
                    parent_type_args,
                    span,
                    ..
                } => {
                    for type_arg in parent_type_args {
                        self.validate_self_type_ref(type_arg, *span, false);
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    self.validate_self_type_expr(expr, false);
                }
                Declaration::Import { .. } | Declaration::Error { .. } => {}
            }
        }
    }

    fn validate_self_type_params(&mut self, params: &[Param], allow_self_type: bool) {
        for param in params {
            self.validate_self_type_ref(&param.ty, param.span, allow_self_type);
        }
    }

    fn validate_self_type_ref(&mut self, ast_type: &AstType, span: Span, allow_self_type: bool) {
        match ast_type {
            AstType::SelfType => {
                if !allow_self_type {
                    self.diagnostics.push(Diagnostic::error(
                        "E0204",
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
            }
            AstType::Generic { type_args, .. } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, span, allow_self_type);
                }
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_self_type_ref(inner, span, allow_self_type);
            }
            AstType::Array { elem, .. } => {
                self.validate_self_type_ref(elem, span, allow_self_type);
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_self_type_ref(param, span, allow_self_type);
                }
                self.validate_self_type_ref(ret, span, allow_self_type);
            }
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::String
            | AstType::Named(_)
            | AstType::Inferred => {}
        }
    }

    fn validate_self_type_expr(&mut self, expr: &Expression, allow_self_type: bool) {
        match expr {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for arg in args {
                    self.validate_self_type_expr(arg, allow_self_type);
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_self_type_expr(receiver, allow_self_type);
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for arg in args {
                    self.validate_self_type_expr(arg, allow_self_type);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_self_type_expr(left, allow_self_type);
                self.validate_self_type_expr(right, allow_self_type);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_self_type_expr(operand, allow_self_type);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_self_type_expr(object, allow_self_type);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_self_type_expr(object, allow_self_type);
                self.validate_self_type_expr(index, allow_self_type);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                for (_, value) in fields {
                    self.validate_self_type_expr(value, allow_self_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload: None,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload: Some(payload),
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_self_type_ref(type_arg, *span, allow_self_type);
                }
                self.validate_self_type_expr(payload, allow_self_type);
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_self_type_expr(element, allow_self_type);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_self_type_expr(scrutinee, allow_self_type);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_self_type_expr(guard, allow_self_type);
                    }
                    self.validate_self_type_expr(&arm.body, allow_self_type);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.validate_self_type_expr(condition, allow_self_type);
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::Loop { body, .. } => {
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_self_type_expr(condition, allow_self_type);
                self.validate_self_type_expr(then_body, allow_self_type);
                if let Some(else_body) = else_body {
                    self.validate_self_type_expr(else_body, allow_self_type);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
                if let Some(expr) = expr {
                    self.validate_self_type_expr(expr, allow_self_type);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_self_type_expr(value, allow_self_type);
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                self.validate_self_type_params(params, allow_self_type);
                if let Some(return_type) = return_type {
                    self.validate_self_type_ref(return_type, *span, allow_self_type);
                }
                self.validate_self_type_expr(body, allow_self_type);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_self_type_expr(expr, allow_self_type);
                self.validate_self_type_ref(target_type, *span, allow_self_type);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.validate_self_type_expr(expr, allow_self_type);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_self_type_expr(start, allow_self_type);
                self.validate_self_type_expr(end, allow_self_type);
            }
            Expression::Defer { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_self_type_statement(&mut self, statement: &ast::Statement, allow_self_type: bool) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_self_type_ref(ty, *span, allow_self_type);
                }
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_self_type_expr(target, allow_self_type);
                self.validate_self_type_expr(value, allow_self_type);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_self_type_expr(expr, allow_self_type);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_self_type_statement(statement, allow_self_type);
                }
            }
        }
    }

    fn validate_generic_type_ref_bounds(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_ref_bounds_with_unknowns(
            ast_type,
            scoped_type_params,
            span,
            true,
        );
    }

    fn validate_generic_type_ref_bounds_allow_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_ref_bounds_with_unknowns(
            ast_type,
            scoped_type_params,
            span,
            false,
        );
    }

    fn validate_generic_type_ref_bounds_with_unknowns(
        &mut self,
        ast_type: &AstType,
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if scoped_type_params.contains(name) {
                    return;
                }

                if !self.is_known_named_type(name) {
                    if reject_unknown {
                        self.diagnostics.push(Diagnostic::error(
                            "E0201",
                            format!("unknown type symbol '{name}'"),
                            span,
                        ));
                    }
                    return;
                }

                let generic = self
                    .structs
                    .get(name)
                    .map(|info| ("struct", info.type_params.len()))
                    .or_else(|| {
                        self.enums
                            .get(name)
                            .map(|info| ("enum", info.type_params.len()))
                    });
                if let Some((kind, type_param_count)) = generic {
                    if type_param_count > 0 {
                        self.diagnostics.push(Diagnostic::error(
                            "E5001",
                            format!(
                                "generic {} `{}` expects {} type arguments, found 0",
                                kind, name, type_param_count
                            ),
                            span,
                        ));
                    }
                }
            }
            AstType::Generic { name, type_args } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds_with_unknowns(
                        type_arg,
                        scoped_type_params,
                        span,
                        reject_unknown,
                    );
                }

                if scoped_type_params.contains(name) {
                    return;
                }

                let (kind, type_params, type_param_bounds) =
                    if let Some(info) = self.structs.get(name) {
                        (
                            "struct",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else if let Some(info) = self.enums.get(name) {
                        (
                            "enum",
                            info.type_params.clone(),
                            info.type_param_bounds.clone(),
                        )
                    } else {
                        if reject_unknown && !self.imports.contains_key(name) {
                            self.diagnostics.push(Diagnostic::error(
                                "E0201",
                                format!("unknown type symbol '{name}'"),
                                span,
                            ));
                        }
                        return;
                    };

                if type_params.len() != type_args.len() {
                    self.diagnostics.push(Diagnostic::error(
                        "E5001",
                        format!(
                            "generic {} `{}` expects {} type arguments, found {}",
                            kind,
                            name,
                            type_params.len(),
                            type_args.len()
                        ),
                        span,
                    ));
                    return;
                }

                let substitutions: HashMap<String, Type> = type_params
                    .iter()
                    .zip(type_args.iter())
                    .filter_map(|(param, arg)| {
                        if ast_type_references_type_param(arg, scoped_type_params) {
                            None
                        } else {
                            Some((param.clone(), self.resolve_type(arg)))
                        }
                    })
                    .collect();
                self.check_generic_bounds(&type_param_bounds, &substitutions, span);
            }
            AstType::Ptr(inner)
            | AstType::MutPtr(inner)
            | AstType::RawPtr(inner)
            | AstType::Slice(inner) => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    inner,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Array { elem, .. } => {
                self.validate_generic_type_ref_bounds_with_unknowns(
                    elem,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_generic_type_ref_bounds_with_unknowns(
                        param,
                        scoped_type_params,
                        span,
                        reject_unknown,
                    );
                }
                self.validate_generic_type_ref_bounds_with_unknowns(
                    ret,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
            }
            _ => {}
        }
    }

    fn is_known_named_type(&self, name: &str) -> bool {
        self.structs.contains_key(name)
            || self.enums.contains_key(name)
            || self.imports.contains_key(name)
    }

    fn validate_generic_expr_type_references(
        &mut self,
        expr: &Expression,
        scoped_type_params: &HashSet<String>,
    ) {
        match expr {
            Expression::FunctionCall {
                type_args,
                args,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                for arg in args {
                    self.validate_generic_expr_type_references(arg, scoped_type_params);
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_generic_expr_type_references(receiver, scoped_type_params);
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                for arg in args {
                    self.validate_generic_expr_type_references(arg, scoped_type_params);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_generic_expr_type_references(left, scoped_type_params);
                self.validate_generic_expr_type_references(right, scoped_type_params);
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_generic_expr_type_references(operand, scoped_type_params);
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_generic_expr_type_references(object, scoped_type_params);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_generic_expr_type_references(object, scoped_type_params);
                self.validate_generic_expr_type_references(index, scoped_type_params);
            }
            Expression::StructLiteral {
                type_args,
                fields,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                for (_, value) in fields {
                    self.validate_generic_expr_type_references(value, scoped_type_params);
                }
            }
            Expression::EnumVariant {
                type_args,
                payload,
                span,
                ..
            } => {
                for type_arg in type_args {
                    self.validate_generic_type_ref_bounds(type_arg, scoped_type_params, *span);
                }
                if let Some(payload) = payload {
                    self.validate_generic_expr_type_references(payload, scoped_type_params);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_generic_expr_type_references(element, scoped_type_params);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_generic_expr_type_references(scrutinee, scoped_type_params);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        self.validate_generic_expr_type_references(guard, scoped_type_params);
                    }
                    self.validate_generic_expr_type_references(&arm.body, scoped_type_params);
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.validate_generic_expr_type_references(condition, scoped_type_params);
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::Loop { body, .. } => {
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_generic_expr_type_references(condition, scoped_type_params);
                self.validate_generic_expr_type_references(then_body, scoped_type_params);
                if let Some(else_body) = else_body {
                    self.validate_generic_expr_type_references(else_body, scoped_type_params);
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                for statement in statements {
                    self.validate_generic_statement_type_references(statement, scoped_type_params);
                }
                if let Some(expr) = expr {
                    self.validate_generic_expr_type_references(expr, scoped_type_params);
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_generic_expr_type_references(value, scoped_type_params);
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                for param in params {
                    self.validate_generic_type_ref_bounds(
                        &param.ty,
                        scoped_type_params,
                        param.span,
                    );
                }
                if let Some(return_type) = return_type {
                    self.validate_generic_type_ref_bounds(return_type, scoped_type_params, *span);
                }
                self.validate_generic_expr_type_references(body, scoped_type_params);
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
                self.validate_generic_type_ref_bounds(target_type, scoped_type_params, *span);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.validate_generic_expr_type_references(expr, scoped_type_params);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_generic_expr_type_references(start, scoped_type_params);
                self.validate_generic_expr_type_references(end, scoped_type_params);
            }
            Expression::Defer { expr, .. } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Identifier { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_generic_statement_type_references(
        &mut self,
        statement: &ast::Statement,
        scoped_type_params: &HashSet<String>,
    ) {
        match statement {
            ast::Statement::VarDecl {
                ty, value, span, ..
            } => {
                if let Some(ty) = ty {
                    self.validate_generic_type_ref_bounds(ty, scoped_type_params, *span);
                }
                self.validate_generic_expr_type_references(value, scoped_type_params);
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.validate_generic_expr_type_references(target, scoped_type_params);
                self.validate_generic_expr_type_references(value, scoped_type_params);
            }
            ast::Statement::Expression { expr, .. } => {
                self.validate_generic_expr_type_references(expr, scoped_type_params);
            }
            ast::Statement::Block { stmts, .. } => {
                for statement in stmts {
                    self.validate_generic_statement_type_references(statement, scoped_type_params);
                }
            }
        }
    }

    pub(crate) fn check_generic_bounds(
        &mut self,
        bounds: &HashMap<String, BehaviorBound>,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) {
        for (param, bound) in bounds {
            let Some(concrete) = substitutions.get(param) else {
                continue;
            };
            let behavior_key = self.behavior_bound_key(bound, substitutions);
            let behavior_display = behavior_bound_display(bound, substitutions);
            let Some(type_name) = self.behavior_bound_type_name(concrete) else {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        concrete.display_name(),
                        behavior_display,
                        param
                    ),
                    span,
                ));
                continue;
            };
            if !self.type_implements_behavior(&type_name, &behavior_key) {
                self.diagnostics.push(Diagnostic::error(
                    "E6004",
                    format!(
                        "type `{}` does not implement behavior `{}` required by `{}`",
                        type_name, behavior_display, param
                    ),
                    span,
                ));
            }
        }
    }

    fn behavior_bound_key(
        &self,
        bound: &BehaviorBound,
        substitutions: &HashMap<String, Type>,
    ) -> String {
        let type_args = substitute_behavior_bound_type_args(&bound.type_args, substitutions);
        self.behavior_reference_key(&bound.behavior, &type_args)
    }

    fn behavior_bound_type_name(&self, ty: &Type) -> Option<String> {
        match ty {
            Type::Named(name) | Type::Struct { name, .. } | Type::Enum { name, .. } => {
                Some(name.clone())
            }
            _ => None,
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
        self.define_var_with_mutability(name, ty, false);
    }

    pub(crate) fn define_var_with_mutability(&mut self, name: &str, ty: Type, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.vars.insert(name.to_string(), VarInfo { ty, mutable });
        }
    }

    pub(crate) fn lookup_var(&self, name: &str) -> Option<Type> {
        self.lookup_var_info(name).map(|info| info.ty.clone())
    }

    pub(crate) fn lookup_var_info(&self, name: &str) -> Option<&VarInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.vars.get(name) {
                return Some(info);
            }
        }
        None
    }

    pub(crate) fn is_import(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    pub(crate) fn is_root_std_import(&self, name: &str) -> bool {
        self.imports
            .get(name)
            .is_some_and(|path| path == &["std".to_string()] || path == &["@std".to_string()])
    }

    fn validate_resolver_symbols(&mut self, program: &ast::Program, symbols: &SymbolTable) {
        let mut scope_cursor = ResolverScopeCursor::default();
        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name,
                    params,
                    return_type,
                    type_params,
                    public,
                    span,
                    body,
                    ..
                } => {
                    self.require_resolver_value_symbol(
                        symbols,
                        name,
                        expected_value_symbol(params, return_type, type_params, *public),
                        *span,
                    );
                    let mut locals = scope_cursor.new_scope();
                    self.require_resolver_parameter_locals(symbols, params, &mut locals);
                    self.require_resolver_expr_locals(
                        symbols,
                        body,
                        &mut scope_cursor,
                        &mut locals,
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    params,
                    return_type,
                    type_params,
                    public,
                    span,
                    body,
                    ..
                } => {
                    self.require_resolver_symbol(symbols, Namespace::Type, type_name, *span);
                    self.require_resolver_value_symbol(
                        symbols,
                        &Self::method_key(type_name, method_name),
                        expected_value_symbol(params, return_type, type_params, *public),
                        *span,
                    );
                    let mut locals = scope_cursor.new_scope();
                    self.require_resolver_parameter_locals(symbols, params, &mut locals);
                    self.require_resolver_expr_locals(
                        symbols,
                        body,
                        &mut scope_cursor,
                        &mut locals,
                    );
                }
                Declaration::Struct {
                    name,
                    type_params,
                    fields,
                    public,
                    span,
                    ..
                } => {
                    if self
                        .require_resolver_struct_symbol(
                            symbols,
                            name,
                            expected_struct_symbol(type_params, fields, *public),
                            *span,
                        )
                        .is_none()
                    {
                        continue;
                    };
                    for field in fields {
                        if let Some(default) = &field.default {
                            let mut locals = scope_cursor.new_scope();
                            self.require_resolver_expr_locals(
                                symbols,
                                default,
                                &mut scope_cursor,
                                &mut locals,
                            );
                        }
                    }
                }
                Declaration::Enum {
                    name,
                    type_params,
                    variants,
                    public,
                    span,
                    ..
                } => {
                    self.require_resolver_enum_symbol(
                        symbols,
                        name,
                        expected_enum_symbol(type_params, variants, *public),
                        *span,
                    );
                    for variant in variants {
                        self.require_resolver_variant_symbol(
                            symbols,
                            &variant.name,
                            expected_variant_symbol(name, *public, &variant.payload),
                            variant.span,
                        );
                    }
                }
                Declaration::Behavior {
                    name,
                    type_params,
                    methods,
                    public,
                    span,
                    ..
                } => {
                    if self
                        .require_resolver_behavior_symbol(
                            symbols,
                            name,
                            expected_behavior_symbol(type_params, methods, *public),
                            *span,
                        )
                        .is_none()
                    {
                        continue;
                    };
                    for method in methods {
                        if let Some(default_body) = &method.default_body {
                            let mut locals = scope_cursor.new_scope();
                            self.require_resolver_parameter_locals(
                                symbols,
                                &method.params,
                                &mut locals,
                            );
                            self.require_resolver_expr_locals(
                                symbols,
                                default_body,
                                &mut scope_cursor,
                                &mut locals,
                            );
                        }
                    }
                }
                Declaration::Import {
                    names,
                    module_path,
                    span,
                } => {
                    self.require_resolver_module_symbol(
                        symbols,
                        expected_module_symbol(&module_path.join(".")),
                        *span,
                    );
                    for name in names {
                        self.require_resolver_import_symbol(
                            symbols,
                            name,
                            expected_import_symbol(&module_path.join(".")),
                            *span,
                        );
                    }
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    span,
                    ..
                } => {
                    let type_symbol = symbols.lookup(Namespace::Type, type_name);
                    if type_symbol.is_none() {
                        self.require_resolver_symbol(symbols, Namespace::Type, type_name, *span);
                    }
                    if let Some(behavior) = behavior {
                        self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                        if let Some(symbol) = type_symbol {
                            self.validate_resolver_behavior_impl_names(
                                symbol,
                                type_name,
                                expected_behavior_edge(behavior, behavior_type_args),
                                *span,
                            );
                        }
                    }
                    for type_arg in behavior_type_args {
                        self.validate_generic_type_ref_bounds_allow_unknowns(
                            type_arg,
                            &HashSet::new(),
                            *span,
                        );
                    }
                    for method in methods {
                        if let Declaration::Function {
                            name,
                            params,
                            return_type,
                            type_params,
                            public,
                            span,
                            body,
                            ..
                        } = method
                        {
                            self.require_resolver_value_symbol(
                                symbols,
                                &Self::method_key(type_name, name),
                                expected_value_symbol(params, return_type, type_params, *public),
                                *span,
                            );
                            let mut locals = scope_cursor.new_scope();
                            self.require_resolver_parameter_locals(symbols, params, &mut locals);
                            self.require_resolver_expr_locals(
                                symbols,
                                body,
                                &mut scope_cursor,
                                &mut locals,
                            );
                        }
                    }
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    span,
                } => {
                    let type_symbol = symbols.lookup(Namespace::Type, type_name);
                    if type_symbol.is_none() {
                        self.require_resolver_symbol(symbols, Namespace::Type, type_name, *span);
                    }
                    self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                    if let Some(symbol) = type_symbol {
                        self.validate_resolver_behavior_required_names(
                            symbol,
                            type_name,
                            expected_behavior_edge(behavior, behavior_type_args),
                            *span,
                        );
                    }
                    for type_arg in behavior_type_args {
                        self.validate_generic_type_ref_bounds_allow_unknowns(
                            type_arg,
                            &HashSet::new(),
                            *span,
                        );
                    }
                }
                Declaration::BehaviorExtends {
                    behavior,
                    parent,
                    parent_type_args,
                    span,
                } => {
                    self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                    self.require_resolver_symbol(symbols, Namespace::Behavior, parent, *span);
                    for type_arg in parent_type_args {
                        self.validate_generic_type_ref_bounds_allow_unknowns(
                            type_arg,
                            &HashSet::new(),
                            *span,
                        );
                    }
                    if let Some(symbol) = symbols.lookup(Namespace::Behavior, behavior) {
                        self.validate_resolver_behavior_parent_names(
                            symbol,
                            behavior,
                            expected_behavior_edge(parent, parent_type_args),
                            *span,
                        );
                    }
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    let mut locals = scope_cursor.new_scope();
                    self.require_resolver_expr_locals(
                        symbols,
                        expr,
                        &mut scope_cursor,
                        &mut locals,
                    );
                }
                Declaration::Error { .. } => {}
            }
        }
        self.validate_no_extra_resolver_declaration_symbols(program, symbols);
        self.validate_no_extra_resolver_local_symbols(program, symbols);
        self.validate_resolver_behavior_association_lists(program, symbols);
        self.validate_resolver_behavior_parent_lists(program, symbols);
        self.validate_stripped_resolver_import_symbols(program, symbols);
    }

    fn validate_no_extra_resolver_declaration_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected = expected_resolver_declaration_symbols(program);
        let validate_imports = program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Declaration::Import { .. }));
        for symbol in symbols.symbols() {
            if !validate_imports
                && matches!(symbol.namespace, Namespace::Module | Namespace::Import)
            {
                continue;
            }
            if !matches!(
                symbol.namespace,
                Namespace::Value
                    | Namespace::Type
                    | Namespace::Behavior
                    | Namespace::Variant
                    | Namespace::Module
                    | Namespace::Import
            ) {
                continue;
            }
            if !expected.contains(&(symbol.namespace, symbol.name.clone())) {
                self.validate_extra_resolver_symbol(
                    symbol.namespace.diagnostic_name(),
                    &symbol.name,
                    ResolverSymbolPresenceValidation::extra_declaration_resolver_code(),
                    symbol.definition_span,
                );
            }
        }
    }

    fn validate_no_extra_resolver_local_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected = expected_resolver_local_symbols(program);
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Local {
                continue;
            }
            if !expected.contains(&(symbol.name.clone(), symbol.scope_id)) {
                self.validate_extra_resolver_symbol(
                    "local",
                    &symbol.name,
                    ResolverSymbolPresenceValidation::extra_local_resolver_code(),
                    symbol.definition_span,
                );
            }
        }
    }

    fn validate_resolver_behavior_association_lists(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected = expected_behavior_associations(program);
        for decl in &program.declarations {
            let (Declaration::Struct { name, span, .. } | Declaration::Enum { name, span, .. }) =
                decl
            else {
                continue;
            };
            let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
                continue;
            };
            self.validate_resolver_behavior_impl_list(
                symbol,
                name,
                expected.impls.edges_for(name),
                *span,
            );
            self.validate_resolver_behavior_required_list(
                symbol,
                name,
                expected.required.edges_for(name),
                *span,
            );
        }
    }

    fn validate_resolver_behavior_parent_lists(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        let expected = expected_behavior_parent_associations(program);
        for decl in &program.declarations {
            let Declaration::Behavior { name, span, .. } = decl else {
                continue;
            };
            let Some(symbol) = symbols.lookup(Namespace::Behavior, name) else {
                continue;
            };
            self.validate_resolver_behavior_parent_list(
                symbol,
                name,
                expected.edges_for(name),
                *span,
            );
        }
    }

    fn require_resolver_module_symbol(
        &mut self,
        symbols: &SymbolTable,
        expected: ExpectedModuleSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Module, &expected.name) else {
            self.require_resolver_symbol(symbols, Namespace::Module, &expected.name, span);
            return;
        };

        self.validate_resolver_visibility(
            "module",
            &expected.name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "module",
            &expected.name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::module_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "module",
            &expected.name,
            ValueSignatureAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "module",
            &expected.name,
            TypeParameterAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "module",
            &expected.name,
            FieldAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "module",
            &expected.name,
            VariantAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorAssociationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "module",
            &expected.name,
            BehaviorDeclarationAbsenceValidation::module_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "module",
            &expected.name,
            MutabilityAbsenceValidation::module_resolver_code(),
            span,
        );
    }

    fn validate_stripped_resolver_import_symbols(
        &mut self,
        program: &ast::Program,
        symbols: &SymbolTable,
    ) {
        if program
            .declarations
            .iter()
            .any(|decl| matches!(decl, Declaration::Import { .. }))
        {
            return;
        }

        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            self.validate_resolver_visibility(
                "import",
                &symbol.name,
                symbol.is_public,
                false,
                VisibilityValidation::import_resolver_code(),
                symbol.definition_span,
            );
            if symbol.import_source.is_none() {
                self.validate_resolver_source(
                    "import",
                    &symbol.name,
                    symbol.import_source.as_deref(),
                    Some("a module source"),
                    SourceValidation::stripped_import_resolver_code(),
                    symbol.definition_span,
                );
            } else if let Some(source) = symbol.import_source.as_deref() {
                self.require_resolver_module_symbol(
                    symbols,
                    expected_module_symbol(source),
                    symbol.definition_span,
                );
            }
            self.validate_resolver_import_absent_declaration_metadata(
                symbol,
                &symbol.name,
                symbol.definition_span,
            );
        }
    }

    fn collect_resolver_imports(&mut self, symbols: &SymbolTable) {
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Import {
                continue;
            }
            let Some(source) = &symbol.import_source else {
                continue;
            };
            self.imports
                .entry(symbol.name.clone())
                .or_insert_with(|| source.split('.').map(str::to_string).collect());
        }
    }

    fn collect_module_graph_imports(
        &mut self,
        graph: &ResolvedModuleGraph,
        entry: &ResolvedModule,
    ) {
        for binding in &entry.imports {
            let Some(source_module) = graph.module(binding.source_module) else {
                self.diagnostics.push(Diagnostic::error(
                    "E0233",
                    format!(
                        "module graph import '{}' points at missing module {:?}",
                        binding.local_name, binding.source_module
                    ),
                    binding.span,
                ));
                continue;
            };

            let Some(decl) = source_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
            else {
                self.diagnostics.push(Diagnostic::error(
                    "E0234",
                    format!(
                        "module graph import '{}' points at missing symbol '{}'",
                        binding.local_name, binding.source_symbol
                    ),
                    binding.span,
                ));
                continue;
            };

            self.seed_module_graph_import(binding.local_name.as_str(), decl);
            self.seed_imported_callable_signature_type_dependencies(decl, source_module, graph);
            self.seed_imported_generic_function_dependencies(
                binding.local_name.as_str(),
                decl,
                source_module,
                graph,
            );
            if matches!(decl, Declaration::Behavior { .. }) {
                self.seed_behavior_extends_for_imported_behavior(
                    binding.local_name.as_str(),
                    binding.source_symbol.as_str(),
                    source_module,
                    graph,
                );
            }
            self.seed_public_methods_for_imported_type(
                binding.source_symbol.as_str(),
                source_module,
                graph,
            );
            self.seed_behavior_impls_for_imported_type(
                binding.local_name.as_str(),
                binding.source_symbol.as_str(),
                source_module,
                graph,
            );
        }
    }

    fn seed_imported_callable_signature_type_dependencies(
        &mut self,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let mut type_names = HashSet::new();
        match decl {
            Declaration::Function {
                params,
                return_type,
                ..
            }
            | Declaration::Method {
                params,
                return_type,
                ..
            } => {
                for param in params {
                    collect_ast_type_names(&param.ty, &mut type_names);
                }
                if let Some(return_type) = return_type {
                    collect_ast_type_names(return_type, &mut type_names);
                }
            }
            _ => return,
        }

        for type_name in type_names {
            self.seed_imported_type_dependency(&type_name, source_module, graph);
        }
    }

    fn seed_imported_type_dependency(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(type_decl) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(type_name))
        {
            if !matches!(
                type_decl,
                Declaration::Struct { public: true, .. } | Declaration::Enum { public: true, .. }
            ) {
                return;
            }
            self.seed_module_graph_import(type_name, type_decl);
            self.seed_public_methods_for_imported_type(type_name, source_module, graph);
            self.seed_behavior_impls_for_imported_type(type_name, type_name, source_module, graph);
            return;
        }

        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == type_name)
        else {
            return;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return;
        };
        let Some(type_decl) = imported_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
        else {
            return;
        };
        if !matches!(
            type_decl,
            Declaration::Struct { public: true, .. } | Declaration::Enum { public: true, .. }
        ) {
            return;
        }
        self.seed_module_graph_import(type_name, type_decl);
        self.seed_public_methods_for_imported_type(
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
        self.seed_behavior_impls_for_imported_type(
            type_name,
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
    }

    fn seed_imported_generic_function_dependencies(
        &mut self,
        local_name: &str,
        decl: &Declaration,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Declaration::Function { type_params, .. } = decl else {
            return;
        };
        if type_params.is_empty() {
            return;
        }
        let dependencies = Self::source_module_dependencies(source_module, graph);
        let Some(template) = self.generic_functions.get_mut(local_name) else {
            return;
        };
        Self::attach_template_dependencies(template, dependencies);
    }

    fn attach_template_dependencies(
        template: &mut GenericFunctionTemplate,
        dependencies: SourceModuleDependencies,
    ) {
        template.attach_source_dependencies(dependencies);
    }

    fn seed_module_graph_import(&mut self, local_name: &str, decl: &Declaration) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.structs.insert(
                    local_name.to_string(),
                    struct_info_from_ast_fields(local_name.to_string(), type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants),
                );
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.behaviors.insert(
                    local_name.to_string(),
                    behavior_info_from_ast_methods(local_name.to_string(), type_params, methods),
                );
            }
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.functions.insert(
                    local_name.to_string(),
                    func_info_from_ast_signature(
                        local_name.to_string(),
                        type_params,
                        params,
                        return_type,
                    ),
                );
                if let Some(template) =
                    generic_template_from_type_params(type_params, params, return_type, body, *span)
                {
                    self.generic_functions
                        .insert(local_name.to_string(), template);
                }
            }
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                let key = Self::method_key(type_name, method_name);
                self.methods.insert(
                    key.clone(),
                    func_info_from_ast_signature(key.clone(), type_params, params, return_type),
                );
                if let Some(template) =
                    generic_template_from_type_params(type_params, params, return_type, body, *span)
                {
                    self.generic_methods.insert(key, template);
                }
            }
            _ => {}
        }
    }

    fn seed_behavior_extends_for_imported_behavior(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        self.seed_behavior_extends_for_imported_behavior_inner(
            local_name,
            source_name,
            source_module,
            graph,
            &mut HashSet::new(),
        );
    }

    fn seed_behavior_extends_for_imported_behavior_inner(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        seen: &mut HashSet<String>,
    ) {
        if !seen.insert(source_name.to_string()) {
            return;
        }

        for decl in &source_module.program.declarations {
            let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } = decl
            else {
                continue;
            };
            if behavior != source_name {
                continue;
            }

            if let Some(parent_decl) = source_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(parent.as_str()))
            {
                self.seed_module_graph_import(parent, parent_decl);
                self.seed_behavior_extends_for_imported_behavior_inner(
                    parent,
                    parent,
                    source_module,
                    graph,
                    seen,
                );
            } else if let Some(binding) = source_module
                .imports
                .iter()
                .find(|binding| binding.local_name == *parent)
            {
                if let Some(parent_module) = graph.module(binding.source_module) {
                    if let Some(parent_decl) = parent_module
                        .program
                        .declarations
                        .iter()
                        .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
                    {
                        self.seed_module_graph_import(parent, parent_decl);
                        self.seed_behavior_extends_for_imported_behavior_inner(
                            parent,
                            binding.source_symbol.as_str(),
                            parent_module,
                            graph,
                            seen,
                        );
                    }
                }
            }

            let parent_ref = self.behavior_parent_ref(parent, parent_type_args);
            let parents = self
                .behavior_extends
                .entry(local_name.to_string())
                .or_default();
            if parents
                .iter()
                .any(|existing| existing.key == parent_ref.key)
            {
                continue;
            }

            parents.push(parent_ref);
            self.behavior_extends_spans
                .entry(local_name.to_string())
                .or_insert(*span);
        }
    }

    fn seed_public_methods_for_imported_type(
        &mut self,
        type_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let dependencies = Self::source_module_dependencies(source_module, graph);

        for decl in &source_module.program.declarations {
            let Declaration::Method {
                type_name: method_type,
                public,
                ..
            } = decl
            else {
                continue;
            };

            if method_type == type_name && *public {
                self.seed_imported_method_with_dependencies(type_name, decl, &dependencies);
            }
        }
        for decl in &source_module.program.declarations {
            let Declaration::ImplBlock {
                type_name: impl_type,
                behavior: None,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            if impl_type != type_name {
                continue;
            }
            for method in methods {
                self.seed_imported_impl_method(type_name, method, true, &dependencies);
            }
        }
    }

    fn seed_behavior_impls_for_imported_type(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        for decl in &source_module.program.declarations {
            let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                methods,
                ..
            } = decl
            else {
                continue;
            };
            if type_name != source_name {
                continue;
            }
            if !self.imported_behavior_impl_is_public(behavior, source_module, graph) {
                continue;
            }

            self.seed_behavior_decl_for_imported_impl(behavior, behavior, source_module, graph);
            self.seed_behavior_decl_for_imported_impl_from_imports(behavior, source_module, graph);

            self.insert_behavior_impl_ref(local_name, behavior, behavior_type_args);

            let dependencies = Self::source_module_dependencies(source_module, graph);
            for method in methods {
                self.seed_imported_impl_method(local_name, method, false, &dependencies);
            }
            for default in self.behavior_default_methods_for_impl(
                local_name,
                behavior,
                behavior_type_args,
                methods,
            ) {
                self.seed_behavior_default_method_signature(local_name, &default);
            }
        }
    }

    fn imported_behavior_impl_is_public(
        &self,
        behavior: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> bool {
        if let Some(Declaration::Behavior { public, .. }) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(behavior))
        {
            return *public;
        }

        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)
        else {
            return false;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return false;
        };
        matches!(
            imported_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str())),
            Some(Declaration::Behavior { public: true, .. })
        )
    }

    fn seed_behavior_decl_for_imported_impl(
        &mut self,
        local_name: &str,
        source_name: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        if let Some(behavior_decl) = source_module
            .program
            .declarations
            .iter()
            .find(|decl| decl.name() == Some(source_name))
        {
            self.seed_module_graph_import(local_name, behavior_decl);
            self.seed_behavior_extends_for_imported_behavior(
                local_name,
                source_name,
                source_module,
                graph,
            );
        }
    }

    fn seed_behavior_decl_for_imported_impl_from_imports(
        &mut self,
        behavior: &str,
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) {
        let Some(binding) = source_module
            .imports
            .iter()
            .find(|binding| binding.local_name == behavior)
        else {
            return;
        };
        let Some(imported_module) = graph.module(binding.source_module) else {
            return;
        };

        self.seed_behavior_decl_for_imported_impl(
            behavior,
            binding.source_symbol.as_str(),
            imported_module,
            graph,
        );
    }

    fn source_module_dependencies(
        source_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
    ) -> SourceModuleDependencies {
        let mut dependencies = SourceModuleDependencies::default();
        for binding in &source_module.imports {
            let Some(imported_module) = graph.module(binding.source_module) else {
                continue;
            };
            let Some(decl) = imported_module
                .program
                .declarations
                .iter()
                .find(|decl| decl.name() == Some(binding.source_symbol.as_str()))
            else {
                continue;
            };
            Self::insert_source_import_dependency(&binding.local_name, decl, &mut dependencies);
            if matches!(decl, Declaration::Struct { .. } | Declaration::Enum { .. }) {
                Self::insert_source_import_type_method_dependencies(
                    &binding.local_name,
                    binding.source_symbol.as_str(),
                    imported_module,
                    graph,
                    &mut dependencies,
                );
            } else if matches!(
                decl,
                Declaration::Function { type_params, .. } if !type_params.is_empty()
            ) {
                let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
                if let Some(template) = dependencies
                    .generic_functions
                    .get_mut(binding.local_name.as_str())
                {
                    Self::attach_template_dependencies(template, nested_dependencies);
                }
            }
        }

        for decl in &source_module.program.declarations {
            match decl {
                Declaration::Struct { name, .. } => {
                    Self::insert_source_type_dependency(name, decl, &mut dependencies);
                }
                Declaration::Enum { name, .. } => {
                    Self::insert_source_type_dependency(name, decl, &mut dependencies);
                }
                Declaration::Function { name, .. } => {
                    Self::insert_source_function_dependency(
                        name,
                        decl,
                        &mut dependencies.functions,
                        &mut dependencies.generic_functions,
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    ..
                } => {
                    Self::insert_source_method_dependency(
                        &Self::method_key(type_name, method_name),
                        decl,
                        &mut dependencies.methods,
                        &mut dependencies.generic_methods,
                    );
                }
                Declaration::ImplBlock {
                    type_name, methods, ..
                } => {
                    for method in methods {
                        if let Declaration::Function { name, .. } = method {
                            Self::insert_source_method_dependency(
                                &Self::method_key(type_name, name),
                                method,
                                &mut dependencies.methods,
                                &mut dependencies.generic_methods,
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        dependencies
    }

    fn insert_source_import_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
    ) {
        match decl {
            Declaration::Struct { .. } | Declaration::Enum { .. } => {
                Self::insert_source_type_dependency(local_name, decl, dependencies);
            }
            Declaration::Function { .. } => {
                Self::insert_source_function_dependency(
                    local_name,
                    decl,
                    &mut dependencies.functions,
                    &mut dependencies.generic_functions,
                );
            }
            _ => {}
        }
    }

    fn insert_source_type_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                dependencies.structs.insert(
                    local_name.to_string(),
                    struct_info_from_ast_fields(local_name.to_string(), type_params, fields),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                dependencies.enums.insert(
                    local_name.to_string(),
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants),
                );
            }
            _ => {}
        }
    }

    fn insert_source_import_type_method_dependencies(
        local_name: &str,
        source_name: &str,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
    ) {
        for decl in &imported_module.program.declarations {
            match decl {
                Declaration::Method {
                    type_name,
                    method_name,
                    public,
                    ..
                } if type_name == source_name && *public => {
                    Self::insert_source_imported_type_method_dependency(
                        &Self::method_key(local_name, method_name),
                        decl,
                        imported_module,
                        graph,
                        dependencies,
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: None,
                    methods,
                    ..
                } if type_name == source_name => {
                    for method in methods {
                        let Declaration::Function { name, public, .. } = method else {
                            continue;
                        };
                        if !*public {
                            continue;
                        }
                        Self::insert_source_imported_type_method_dependency(
                            &Self::method_key(local_name, name),
                            method,
                            imported_module,
                            graph,
                            dependencies,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn insert_source_imported_type_method_dependency(
        key: &str,
        decl: &Declaration,
        imported_module: &ResolvedModule,
        graph: &ResolvedModuleGraph,
        dependencies: &mut SourceModuleDependencies,
    ) {
        Self::insert_source_method_dependency(
            key,
            decl,
            &mut dependencies.methods,
            &mut dependencies.generic_methods,
        );
        if let Some(template) = dependencies.generic_methods.get_mut(key) {
            let nested_dependencies = Self::source_module_dependencies(imported_module, graph);
            Self::attach_template_dependencies(template, nested_dependencies);
        }
    }

    fn insert_source_function_dependency(
        key: &str,
        decl: &Declaration,
        functions: &mut HashMap<String, FuncInfo>,
        generic_functions: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        if let Some(signature) = ImportedMethodSignature::from_function_declaration(key, decl) {
            Self::insert_source_callable_dependency(signature, functions, generic_functions);
        }
    }

    fn insert_source_method_dependency(
        key: &str,
        decl: &Declaration,
        methods: &mut HashMap<String, FuncInfo>,
        generic_methods: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        if let Some(signature) = ImportedMethodSignature::from_function_declaration(key, decl)
            .or_else(|| ImportedMethodSignature::from_method_declaration(key, decl))
        {
            Self::insert_source_callable_dependency(signature, methods, generic_methods);
        }
    }

    fn insert_source_callable_dependency(
        signature: ImportedMethodSignature<'_>,
        callables: &mut HashMap<String, FuncInfo>,
        generic_callables: &mut HashMap<String, GenericFunctionTemplate>,
    ) {
        callables.insert(
            signature.name.to_string(),
            signature.func_info(signature.name.to_string()),
        );
        if let Some(template) = signature.generic_template() {
            generic_callables.insert(signature.name.to_string(), template);
        }
    }

    fn seed_imported_method_with_dependencies(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Method { method_name, .. } = method else {
            return;
        };
        let Some(signature) = ImportedMethodSignature::from_method_declaration(method_name, method)
        else {
            return;
        };

        self.seed_imported_method_signature(local_type_name, signature, dependencies);
    }

    fn seed_imported_impl_method(
        &mut self,
        local_type_name: &str,
        method: &Declaration,
        public_only: bool,
        dependencies: &SourceModuleDependencies,
    ) {
        let Declaration::Function { name, public, .. } = method else {
            return;
        };
        if public_only && !*public {
            return;
        }
        let Some(signature) = ImportedMethodSignature::from_function_declaration(name, method)
        else {
            return;
        };

        self.seed_imported_method_signature(local_type_name, signature, dependencies);
    }

    fn seed_imported_method_signature(
        &mut self,
        local_type_name: &str,
        signature: ImportedMethodSignature<'_>,
        dependencies: &SourceModuleDependencies,
    ) {
        let key = Self::method_key(local_type_name, signature.name);
        self.methods
            .insert(key.clone(), signature.func_info(key.clone()));
        if let Some(template) = signature.generic_template() {
            self.generic_methods
                .insert(key, dependencies.apply_to_template(template));
        }
    }

    fn require_resolver_symbol(
        &mut self,
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        let found = symbols.lookup(namespace, name).is_some()
            || matches!(namespace, Namespace::Type | Namespace::Behavior)
                && symbols.lookup(Namespace::Import, name).is_some();

        if !found {
            self.validate_missing_resolver_symbol(
                namespace.diagnostic_name(),
                name,
                ResolverSymbolPresenceValidation::missing_resolver_code(),
                span,
            );
        }
    }

    fn require_resolver_import_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedImportSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Import, name) else {
            self.require_resolver_symbol(symbols, Namespace::Import, name, span);
            return;
        };

        self.validate_resolver_visibility(
            "import",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::import_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "import",
            name,
            symbol.import_source.as_deref(),
            Some(expected.source.as_str()),
            SourceValidation::import_resolver_code(),
            span,
        );

        self.validate_resolver_import_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_import_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "import",
            name,
            ValueSignatureAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "import",
            name,
            TypeParameterAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "import",
            name,
            FieldAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "import",
            name,
            VariantAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "import",
            name,
            BehaviorAssociationAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "import",
            name,
            BehaviorDeclarationAbsenceValidation::import_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "import",
            name,
            MutabilityAbsenceValidation::import_resolver_code(),
            span,
        );
    }

    fn require_resolver_parameter_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        locals: &mut ResolverLocalScope,
    ) {
        for param in params {
            self.require_resolver_local_symbol(
                symbols,
                &param.name,
                expected_local_symbol(param.mutable, locals.current_scope_id),
                param.span,
            );
            locals.insert(param.name.clone(), param.mutable);
        }
    }

    fn require_resolver_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match expr {
            Expression::BinaryOp { left, right, .. } => {
                self.require_resolver_expr_locals(symbols, left, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, right, scope_cursor, locals);
            }
            Expression::UnaryOp { operand, .. } => {
                self.require_resolver_expr_locals(symbols, operand, scope_cursor, locals);
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg, scope_cursor, locals);
                }
            }
            Expression::MethodCall { receiver, args, .. } => {
                self.require_resolver_expr_locals(symbols, receiver, scope_cursor, locals);
                for arg in args {
                    self.require_resolver_expr_locals(symbols, arg, scope_cursor, locals);
                }
            }
            Expression::MemberAccess { object, .. } => {
                self.require_resolver_expr_locals(symbols, object, scope_cursor, locals);
            }
            Expression::IndexAccess { object, index, .. } => {
                self.require_resolver_expr_locals(symbols, object, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, index, scope_cursor, locals);
            }
            Expression::StructLiteral { fields, .. } => {
                for (_, value) in fields {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::EnumVariant { payload, .. } => {
                if let Some(payload) = payload {
                    self.require_resolver_expr_locals(symbols, payload, scope_cursor, locals);
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.require_resolver_expr_locals(symbols, element, scope_cursor, locals);
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.require_resolver_expr_locals(symbols, scrutinee, scope_cursor, locals);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        let mut guard_locals = scope_cursor.child_scope(locals);
                        self.require_resolver_pattern_locals(
                            symbols,
                            &arm.pattern,
                            scope_cursor,
                            &mut guard_locals,
                        );
                        self.require_resolver_expr_locals(
                            symbols,
                            guard,
                            scope_cursor,
                            &mut guard_locals,
                        );
                    }
                    let mut arm_locals = scope_cursor.child_scope(locals);
                    self.require_resolver_pattern_locals(
                        symbols,
                        &arm.pattern,
                        scope_cursor,
                        &mut arm_locals,
                    );
                    self.require_resolver_expr_locals(
                        symbols,
                        &arm.body,
                        scope_cursor,
                        &mut arm_locals,
                    );
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                let mut body_locals = scope_cursor.child_scope(locals);
                self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut body_locals);
            }
            Expression::Loop { body, .. } => {
                let mut body_locals = scope_cursor.child_scope(locals);
                self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut body_locals);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                let mut then_locals = scope_cursor.child_scope(locals);
                self.require_resolver_expr_locals(
                    symbols,
                    then_body,
                    scope_cursor,
                    &mut then_locals,
                );
                if let Some(else_body) = else_body {
                    let mut else_locals = scope_cursor.child_scope(locals);
                    self.require_resolver_expr_locals(
                        symbols,
                        else_body,
                        scope_cursor,
                        &mut else_locals,
                    );
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                let mut block_locals = scope_cursor.child_scope(locals);
                for statement in statements {
                    self.require_resolver_statement_locals(
                        symbols,
                        statement,
                        scope_cursor,
                        &mut block_locals,
                    );
                }
                if let Some(expr) = expr {
                    self.require_resolver_expr_locals(
                        symbols,
                        expr,
                        scope_cursor,
                        &mut block_locals,
                    );
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::Closure { params, body, .. } => {
                let mut closure_locals = scope_cursor.child_scope(locals);
                for param in params {
                    self.require_resolver_local_symbol(
                        symbols,
                        &param.name,
                        expected_local_symbol(false, closure_locals.current_scope_id),
                        param.span,
                    );
                    closure_locals.insert(param.name.clone(), false);
                }
                self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut closure_locals);
            }
            Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let ast::StringPart::Expr(expr) = part {
                        self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, end, scope_cursor, locals);
            }
            Expression::Identifier { .. }
            | Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn require_resolver_statement_locals(
        &mut self,
        symbols: &SymbolTable,
        statement: &ast::Statement,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match statement {
            ast::Statement::VarDecl {
                name,
                value,
                mutable,
                span,
                constant,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                if *constant || *mutable || !locals.is_mutable(name) {
                    self.require_resolver_local_symbol(
                        symbols,
                        name,
                        expected_local_symbol(*mutable, locals.current_scope_id),
                        *span,
                    );
                    locals.insert(name.clone(), *mutable);
                }
            }
            ast::Statement::Assignment { target, value, .. } => {
                self.require_resolver_expr_locals(symbols, target, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
            }
            ast::Statement::Expression { expr, .. } => {
                self.require_resolver_expr_locals(symbols, expr, scope_cursor, locals);
            }
            ast::Statement::Block { stmts, .. } => {
                let mut block_locals = scope_cursor.child_scope(locals);
                for statement in stmts {
                    self.require_resolver_statement_locals(
                        symbols,
                        statement,
                        scope_cursor,
                        &mut block_locals,
                    );
                }
            }
        }
    }

    fn require_resolver_pattern_locals(
        &mut self,
        symbols: &SymbolTable,
        pattern: &ast::Pattern,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &mut ResolverLocalScope,
    ) {
        match pattern {
            ast::Pattern::Identifier { name, span } => {
                self.require_resolver_local_symbol(
                    symbols,
                    name,
                    expected_local_symbol(false, locals.current_scope_id),
                    *span,
                );
                locals.insert(name.clone(), false);
            }
            ast::Pattern::Struct { fields, span, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.require_resolver_pattern_locals(symbols, nested, scope_cursor, locals);
                    } else {
                        self.require_resolver_local_symbol(
                            symbols,
                            name,
                            expected_local_symbol(false, locals.current_scope_id),
                            *span,
                        );
                        locals.insert(name.clone(), false);
                    }
                }
            }
            ast::Pattern::Enum {
                payload: Some(payload),
                ..
            } => {
                self.require_resolver_pattern_locals(symbols, payload, scope_cursor, locals);
            }
            ast::Pattern::Or { patterns, .. } => {
                for pattern in patterns {
                    self.require_resolver_pattern_locals(symbols, pattern, scope_cursor, locals);
                }
            }
            ast::Pattern::Literal { value, .. } => {
                self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
            }
            ast::Pattern::Range { start, end, .. } => {
                self.require_resolver_expr_locals(symbols, start, scope_cursor, locals);
                self.require_resolver_expr_locals(symbols, end, scope_cursor, locals);
            }
            ast::Pattern::Wildcard { .. }
            | ast::Pattern::Enum { payload: None, .. }
            | ast::Pattern::BoolTrue { .. }
            | ast::Pattern::BoolFalse { .. } => {}
        }
    }

    fn require_resolver_local_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedLocalSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup_in_scope(Namespace::Local, name, expected.scope_id)
        else {
            self.validate_missing_resolver_symbol(
                "local",
                name,
                ResolverSymbolPresenceValidation::missing_local_resolver_code(),
                span,
            );
            return;
        };

        self.validate_resolver_mutability(
            "local",
            name,
            symbol.is_mutable,
            expected.is_mutable,
            MutabilityValidation::resolver_code(),
            span,
        );

        self.validate_resolver_visibility(
            "local",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::local_resolver_code(),
            span,
        );

        self.validate_resolver_source(
            "local",
            name,
            symbol.import_source.as_deref(),
            expected.source.as_deref(),
            SourceValidation::local_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "local",
            name,
            ValueSignatureAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "local",
            name,
            TypeParameterAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "local",
            name,
            FieldAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "local",
            name,
            VariantAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "local",
            name,
            BehaviorAssociationAbsenceValidation::local_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "local",
            name,
            BehaviorDeclarationAbsenceValidation::local_resolver_codes(),
            span,
        );
    }

    fn require_resolver_type_like_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        expected: ExpectedTypeLikeSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let Some(symbol) = symbols.lookup(namespace, name) else {
            self.require_resolver_symbol(symbols, namespace, name, span);
            return None;
        };

        if let Some(expected_is_public) = expected.is_public {
            self.validate_resolver_visibility(
                namespace.diagnostic_name(),
                name,
                symbol.is_public,
                expected_is_public,
                VisibilityValidation::type_like_resolver_code(),
                span,
            );
        }

        self.validate_resolver_type_parameters(
            symbol,
            namespace.diagnostic_name(),
            name,
            &expected.type_params,
            TypeParameterValidation::type_like_resolver_codes(),
            span,
        );

        self.validate_resolver_type_like_absent_value_metadata(symbol, namespace, name, span);

        Some(symbol)
    }

    fn require_resolver_struct_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedStructSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Type,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_fields(symbol, Namespace::Type, name, &expected.fields, span);
        self.validate_resolver_struct_absent_enum_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_enum_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedEnumSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Type,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_variant_names(symbol, name, &expected.variant_names, span);
        self.validate_resolver_enum_absent_struct_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_variant_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedVariantSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let Some(symbol) = symbols.lookup_variant(&expected.owner_name, name) else {
            if let Some(symbol) = symbols.lookup(Namespace::Variant, name) {
                self.validate_resolver_variant_owner_name(symbol, name, &expected.owner_name, span);
                return None;
            }
            self.require_resolver_symbol(symbols, Namespace::Variant, name, span);
            return None;
        };

        self.validate_resolver_variant_owner_name(symbol, name, &expected.owner_name, span);
        self.validate_resolver_variant_visibility(symbol, name, expected.is_public, span);
        self.validate_resolver_variant_payload(symbol, name, expected.payload, span);
        self.validate_resolver_variant_absent_other_metadata(symbol, name, span);

        Some(symbol)
    }

    fn require_resolver_behavior_symbol<'a>(
        &mut self,
        symbols: &'a SymbolTable,
        name: &str,
        expected: ExpectedBehaviorSymbol,
        span: Span,
    ) -> Option<&'a crate::resolver::Symbol> {
        let symbol = self.require_resolver_type_like_symbol(
            symbols,
            Namespace::Behavior,
            name,
            expected.type_like,
            span,
        )?;

        self.validate_resolver_behavior_methods(symbol, name, &expected.methods, span);
        self.validate_resolver_behavior_absent_type_metadata(symbol, name, span);

        Some(symbol)
    }

    fn validate_resolver_absent_value_signature_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: ValueSignatureAbsenceValidation,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_type_parameter_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: TypeParameterAbsenceValidation,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_field_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: FieldAbsenceValidation,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_variant_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: VariantAbsenceValidation,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_behavior_association_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorAssociationAbsenceValidation,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_behavior_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorDeclarationAbsenceValidation,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
    }

    fn validate_resolver_absent_mutability_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: MutabilityAbsenceValidation,
        span: Span,
    ) {
        let entry = validation.entry(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &[entry], span);
    }

    fn validate_resolver_absent_source_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: SourceAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_source(
            symbol_kind,
            name,
            symbol.import_source.as_deref(),
            None,
            validation.source_validation(),
            span,
        );
    }

    fn validate_resolver_source(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<&str>,
        expected: Option<&str>,
        validation: SourceValidation,
        span: Span,
    ) {
        if actual != expected {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_mutability(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<bool>,
        expected: bool,
        validation: MutabilityValidation,
        span: Span,
    ) {
        if actual != Some(expected) {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_extra_resolver_symbol(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.validate_resolver_symbol_presence(symbol_kind, name, validation, span);
    }

    fn validate_missing_resolver_symbol(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.validate_resolver_symbol_presence(symbol_kind, name, validation, span);
    }

    fn validate_resolver_symbol_presence(
        &mut self,
        symbol_kind: &str,
        name: &str,
        validation: ResolverSymbolPresenceValidation,
        span: Span,
    ) {
        self.diagnostics.push(Diagnostic::error(
            validation.code,
            validation.message(symbol_kind, name),
            span,
        ));
    }

    fn validate_resolver_absent_metadata_entry(
        &mut self,
        symbol_kind: &str,
        name: &str,
        entry: AbsentMetadataEntry,
        span: Span,
    ) {
        if entry.present {
            self.diagnostics.push(Diagnostic::error(
                entry.code,
                entry.message(symbol_kind, name),
                span,
            ));
        }
    }

    fn validate_resolver_absent_metadata_entries(
        &mut self,
        symbol_kind: &str,
        name: &str,
        entries: &[AbsentMetadataEntry],
        span: Span,
    ) {
        for entry in entries {
            self.validate_resolver_absent_metadata_entry(symbol_kind, name, *entry, span);
        }
    }

    fn validate_resolver_visibility(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: bool,
        expected: bool,
        validation: VisibilityValidation,
        span: Span,
    ) {
        if actual != expected {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_count(
        &mut self,
        symbol_kind: &str,
        name: &str,
        actual: Option<usize>,
        expected: usize,
        validation: CountValidation,
        span: Span,
    ) {
        if actual != Some(expected) {
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(symbol_kind, name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_type_parameters(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        expected: &[ExpectedTypeParameter],
        validation: TypeParameterValidation,
        span: Span,
    ) {
        let expected = ExpectedTypeParameterMetadata::from_parameters(expected);
        self.validate_resolver_count(
            symbol_kind,
            name,
            symbol.type_parameter_count,
            expected.count,
            validation.count_validation(),
            span,
        );

        if symbol.type_parameter_names.as_deref() != Some(expected.names.as_slice()) {
            let actual = format_type_parameter_names(symbol.type_parameter_names.as_deref());
            let expected_names_display = format_type_parameter_names(Some(&expected.names));
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.name_message(symbol_kind, name, &actual, &expected_names_display),
                span,
            ));
        }

        if symbol.type_parameter_bounds.as_deref() != Some(expected.bounds.as_slice()) {
            let actual = format_type_parameter_bounds(symbol.type_parameter_bounds.as_deref());
            let expected_bounds_display = format_type_parameter_bounds(Some(&expected.bounds));
            self.diagnostics.push(Diagnostic::error(
                validation.bound_code,
                validation.bound_message(symbol_kind, name, &actual, &expected_bounds_display),
                span,
            ));
        }
        if symbol.type_parameter_bound_refs.as_deref() != Some(expected.bound_refs.as_slice()) {
            let actual =
                format_type_parameter_bound_refs(symbol.type_parameter_bound_refs.as_deref());
            let expected_refs = format_type_parameter_bound_refs(Some(&expected.bound_refs));
            self.diagnostics.push(Diagnostic::error(
                validation.bound_ref_code,
                validation.bound_ref_message(symbol_kind, name, &actual, &expected_refs),
                span,
            ));
        }
    }

    fn validate_resolver_type_like_absent_value_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            SourceAbsenceValidation::type_like_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            ValueSignatureAbsenceValidation::type_like_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            namespace.diagnostic_name(),
            name,
            MutabilityAbsenceValidation::type_like_resolver_code(),
            span,
        );
    }

    fn validate_resolver_fields(
        &mut self,
        symbol: &crate::resolver::Symbol,
        namespace: Namespace,
        name: &str,
        expected_fields: &[ExpectedField],
        span: Span,
    ) {
        let expected = ExpectedFieldMetadata::from_fields(expected_fields);
        self.validate_resolver_count(
            namespace.diagnostic_name(),
            name,
            symbol.field_count,
            expected.count,
            CountValidation::field_resolver_code(),
            span,
        );
        let validation = FieldValidation::resolver_codes();
        let symbol_kind = namespace.diagnostic_name();
        if symbol.field_types.as_deref() != Some(expected.typed.as_slice()) {
            let actual = format_field_types(symbol.field_types.as_deref());
            let expected = format_field_types(Some(&expected.typed));
            self.diagnostics.push(Diagnostic::error(
                validation.typed_code,
                validation.typed_message(symbol_kind, name, &actual, &expected),
                span,
            ));
        }
        if symbol.field_type_names.as_deref() != Some(expected.display.as_slice()) {
            let actual = format_field_type_names(symbol.field_type_names.as_deref());
            let expected = format_field_type_names(Some(&expected.display));
            self.diagnostics.push(Diagnostic::error(
                validation.display_code,
                validation.display_message(symbol_kind, name, &actual, &expected),
                span,
            ));
        }
    }

    fn validate_resolver_variant_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_variant_names: &[String],
        span: Span,
    ) {
        if symbol.variant_names.as_deref() != Some(expected_variant_names) {
            let validation = VariantNameValidation::resolver_code();
            let actual = format_variant_names(symbol.variant_names.as_deref());
            let expected = format_variant_names(Some(expected_variant_names));
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(name, &actual, &expected),
                span,
            ));
        }
    }

    fn validate_resolver_struct_absent_enum_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_variant_metadata(
            symbol,
            "type",
            name,
            VariantAbsenceValidation::type_like_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_enum_absent_struct_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_field_metadata(
            symbol,
            "type",
            name,
            FieldAbsenceValidation::type_like_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_variant_payload(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_payload: ExpectedVariantPayloadType,
        span: Span,
    ) {
        let expected = ExpectedVariantPayloadMetadata::from_payload(expected_payload);
        self.validate_resolver_count(
            "variant",
            name,
            symbol.variant_payload_count,
            expected.count,
            CountValidation::variant_payload_resolver_code(),
            span,
        );
        let validation = VariantPayloadValidation::resolver_codes();
        if symbol.variant_payload_type != expected.typed {
            let actual = optional_ast_type_display(symbol.variant_payload_type.as_ref(), "none");
            let expected = optional_ast_type_display(expected.typed.as_ref(), "none");
            self.diagnostics.push(Diagnostic::error(
                validation.typed_code,
                validation.typed_message(name, &actual, &expected),
                span,
            ));
        }
        if symbol.variant_payload_type_name != expected.display {
            let actual = resolver_metadata_display(symbol.variant_payload_type_name.as_deref());
            let expected = expected.display.as_deref().unwrap_or("none");
            self.diagnostics.push(Diagnostic::error(
                validation.display_code,
                validation.display_message(name, actual, expected),
                span,
            ));
        }
    }

    fn validate_resolver_variant_owner_name(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_owner_name: &str,
        span: Span,
    ) {
        if symbol.variant_owner_name.as_deref() != Some(expected_owner_name) {
            let validation = VariantOwnerValidation::resolver_code();
            let actual = resolver_metadata_display(symbol.variant_owner_name.as_deref());
            self.diagnostics.push(Diagnostic::error(
                validation.code,
                validation.message(name, actual, expected_owner_name),
                span,
            ));
        }
    }

    fn validate_resolver_variant_visibility(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_is_public: bool,
        span: Span,
    ) {
        self.validate_resolver_visibility(
            "variant",
            name,
            symbol.is_public,
            expected_is_public,
            VisibilityValidation::variant_resolver_code(),
            span,
        );
    }

    fn validate_resolver_variant_absent_other_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            "variant",
            name,
            SourceAbsenceValidation::variant_resolver_code(),
            span,
        );

        self.validate_resolver_absent_value_signature_metadata(
            symbol,
            "variant",
            name,
            ValueSignatureAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_type_parameter_metadata(
            symbol,
            "variant",
            name,
            TypeParameterAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "variant",
            name,
            FieldAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "variant",
            name,
            BehaviorAssociationAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "variant",
            name,
            BehaviorDeclarationAbsenceValidation::variant_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_metadata_entries(
            "variant",
            name,
            &[AbsentMetadataEntry::new(
                symbol.variant_names.is_some(),
                "E0338",
                "variant names",
            )],
            span,
        );
        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "variant",
            name,
            MutabilityAbsenceValidation::variant_resolver_code(),
            span,
        );
    }

    fn validate_resolver_behavior_methods(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_methods: &[ExpectedBehaviorMethod],
        span: Span,
    ) {
        let expected = ExpectedBehaviorMethodMetadata::from_methods(expected_methods);
        let validation = BehaviorMethodValidation::resolver_codes();
        if symbol.behavior_method_signatures.as_deref() != Some(expected.signatures.as_slice()) {
            let actual =
                format_behavior_method_signatures(symbol.behavior_method_signatures.as_deref());
            let expected = format_behavior_method_signatures(Some(&expected.signatures));
            self.diagnostics.push(Diagnostic::error(
                validation.display_code,
                validation.display_message(name, &actual, &expected),
                span,
            ));
        }
        if symbol.behavior_method_types.as_deref() != Some(expected.typed.as_slice()) {
            let actual = format_behavior_method_types(symbol.behavior_method_types.as_deref());
            let expected = format_behavior_method_types(Some(&expected.typed));
            self.diagnostics.push(Diagnostic::error(
                validation.typed_code,
                validation.typed_message(name, &actual, &expected),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_absent_type_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_field_metadata(
            symbol,
            "behavior",
            name,
            FieldAbsenceValidation::behavior_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "behavior",
            name,
            VariantAbsenceValidation::behavior_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "behavior",
            name,
            BehaviorAssociationAbsenceValidation::behavior_resolver_codes(),
            span,
        );
    }

    fn validate_resolver_behavior_parent_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Parent,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_parent_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Parent,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_impl_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Impl,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_impl_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Impl,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_required_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Required,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_required_list(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list_for_role(
            BehaviorRefRole::Required,
            symbol,
            name,
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_contains_for_role(
        &mut self,
        role: BehaviorRefRole,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_contains(
            BehaviorRefValidation::for_role(role, BehaviorRefCheck::Contains),
            name,
            BehaviorRefActual::for_role(symbol, role),
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_list_for_role(
        &mut self,
        role: BehaviorRefRole,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        self.validate_resolver_behavior_ref_list(
            BehaviorRefValidation::for_role(role, BehaviorRefCheck::List),
            name,
            BehaviorRefActual::for_role(symbol, role),
            expected,
            span,
        );
    }

    fn validate_resolver_behavior_ref_contains(
        &mut self,
        validation: BehaviorRefValidation,
        name: &str,
        actual: BehaviorRefActual<'_>,
        expected: ExpectedBehaviorEdge,
        span: Span,
    ) {
        if !actual.contains_display(&expected.display) {
            let actual = format_behavior_ref_names(actual.names);
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.contains_name_message(name, &actual, &expected.display),
                span,
            ));
        }
        if !actual.contains_metadata(&expected.metadata) {
            let actual = format_behavior_refs(actual.refs);
            let expected_ref =
                behavior_ref_display(&expected.metadata.name, &expected.metadata.type_args);
            self.diagnostics.push(Diagnostic::error(
                validation.ref_code,
                validation.contains_ref_message(name, &actual, &expected_ref),
                span,
            ));
        }
    }

    fn validate_resolver_behavior_ref_list(
        &mut self,
        validation: BehaviorRefValidation,
        name: &str,
        actual: BehaviorRefActual<'_>,
        expected: &[ExpectedBehaviorEdge],
        span: Span,
    ) {
        let expected = ExpectedBehaviorEdgeMetadata::from_edges(expected);
        if !actual.names_match(&expected.names) {
            let actual = format_behavior_ref_names(actual.names);
            let expected_names = format_behavior_ref_names(Some(&expected.names));
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.list_name_message(name, &actual, &expected_names),
                span,
            ));
        }
        if !actual.refs_match(&expected.refs) {
            let actual = format_behavior_refs(actual.refs);
            let expected_refs = format_behavior_refs(Some(&expected.refs));
            self.diagnostics.push(Diagnostic::error(
                validation.ref_code,
                validation.list_ref_message(name, &actual, &expected_refs),
                span,
            ));
        }
    }

    fn require_resolver_value_symbol(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        expected: ExpectedValueSymbol,
        span: Span,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Value, name) else {
            self.require_resolver_symbol(symbols, Namespace::Value, name, span);
            return;
        };

        self.validate_resolver_visibility(
            "value",
            name,
            symbol.is_public,
            expected.is_public,
            VisibilityValidation::value_resolver_code(),
            span,
        );

        self.validate_resolver_value_parameters(symbol, name, &expected.signature.params, span);
        self.validate_resolver_value_return_type(
            symbol,
            name,
            &expected.signature.return_type,
            span,
        );

        self.validate_resolver_type_parameters(
            symbol,
            "value",
            name,
            &expected.signature.type_params,
            TypeParameterValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_value_absent_declaration_metadata(symbol, name, span);
    }

    fn validate_resolver_value_parameters(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &[ExpectedParameter],
        span: Span,
    ) {
        let expected = ExpectedParameterMetadata::from_parameters(expected);
        self.validate_resolver_count(
            "value",
            name,
            symbol.parameter_count,
            expected.count,
            CountValidation::value_parameter_resolver_code(),
            span,
        );

        let validation = ValueParameterValidation::resolver_codes();

        if symbol.parameter_names.as_deref() != Some(expected.names.as_slice()) {
            let actual = format_parameter_names(symbol.parameter_names.as_deref());
            let expected_names_display = format_parameter_names(Some(&expected.names));
            self.diagnostics.push(Diagnostic::error(
                validation.name_code,
                validation.name_message(name, &actual, &expected_names_display),
                span,
            ));
        }

        if symbol.parameter_type_names.as_deref() != Some(expected.display_types.as_slice()) {
            let actual = format_parameter_type_names(symbol.parameter_type_names.as_deref());
            let expected_types = format_parameter_type_names(Some(&expected.display_types));
            self.diagnostics.push(Diagnostic::error(
                validation.display_type_code,
                validation.display_type_message(name, &actual, &expected_types),
                span,
            ));
        }
        if symbol.parameter_types.as_deref() != Some(expected.typed_types.as_slice()) {
            let actual = format_ast_type_list(symbol.parameter_types.as_deref());
            let expected_types = format_ast_type_list(Some(&expected.typed_types));
            self.diagnostics.push(Diagnostic::error(
                validation.typed_type_code,
                validation.typed_type_message(name, &actual, &expected_types),
                span,
            ));
        }
    }

    fn validate_resolver_value_return_type(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &ExpectedReturnMetadata,
        span: Span,
    ) {
        let validation = ReturnValidation::resolver_codes();

        if symbol.return_type_name.as_deref() != Some(expected.display.as_str()) {
            let actual = resolver_metadata_display(symbol.return_type_name.as_deref());
            self.diagnostics.push(Diagnostic::error(
                validation.display_code,
                validation.display_message(name, actual, &expected.display),
                span,
            ));
        }
        if symbol.return_type.as_ref() != Some(&expected.typed) {
            let actual = resolver_ast_type_metadata_display(symbol.return_type.as_ref());
            let expected = expected.typed.display_name();
            self.diagnostics.push(Diagnostic::error(
                validation.typed_code,
                validation.typed_message(name, &actual, &expected),
                span,
            ));
        }
    }

    fn validate_resolver_value_absent_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        span: Span,
    ) {
        self.validate_resolver_absent_source_metadata(
            symbol,
            "value",
            name,
            SourceAbsenceValidation::value_resolver_code(),
            span,
        );

        self.validate_resolver_absent_field_metadata(
            symbol,
            "value",
            name,
            FieldAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_variant_metadata(
            symbol,
            "value",
            name,
            VariantAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_association_metadata(
            symbol,
            "value",
            name,
            BehaviorAssociationAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_behavior_declaration_metadata(
            symbol,
            "value",
            name,
            BehaviorDeclarationAbsenceValidation::value_resolver_codes(),
            span,
        );

        self.validate_resolver_absent_mutability_metadata(
            symbol,
            "value",
            name,
            MutabilityAbsenceValidation::value_resolver_code(),
            span,
        );
    }
}

fn expected_return_metadata(return_type: &Option<AstType>) -> ExpectedReturnMetadata {
    ExpectedReturnMetadata::new(return_type)
}

fn visibility_name(is_public: bool) -> &'static str {
    if is_public {
        "public"
    } else {
        "private"
    }
}

fn mutability_name(is_mutable: Option<bool>) -> &'static str {
    match is_mutable {
        Some(true) => "mutable",
        Some(false) => "immutable",
        None => "unknown",
    }
}

fn resolver_count_display(count: Option<usize>) -> String {
    count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn resolver_metadata_display(value: Option<&str>) -> &str {
    value.unwrap_or("unknown")
}

fn resolver_ast_type_metadata_display(value: Option<&AstType>) -> String {
    optional_ast_type_display(value, "unknown")
}

fn optional_ast_type_display(value: Option<&AstType>, missing: &str) -> String {
    value
        .map(AstType::display_name)
        .unwrap_or_else(|| missing.to_string())
}

fn expected_parameter_metadata(params: &[Param]) -> Vec<ExpectedParameter> {
    let mut expected = Vec::new();
    for param in params {
        expected.push(ExpectedParameter::new(&param.name, &param.ty));
    }
    expected
}

fn expected_value_signature_metadata(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
) -> ExpectedValueSignature {
    ExpectedValueSignature::new(params, return_type, type_params)
}

fn expected_value_symbol(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
    is_public: bool,
) -> ExpectedValueSymbol {
    ExpectedValueSymbol::new(params, return_type, type_params, is_public)
}

fn expected_type_parameter_metadata(type_params: &[ast::TypeParam]) -> Vec<ExpectedTypeParameter> {
    let mut expected = Vec::new();
    for type_param in type_params {
        expected.push(ExpectedTypeParameter::new(type_param));
    }
    expected
}

fn expected_behavior_symbol(
    type_params: &[ast::TypeParam],
    methods: &[ast::BehaviorMethod],
    is_public: bool,
) -> ExpectedBehaviorSymbol {
    ExpectedBehaviorSymbol::new(type_params, methods, is_public)
}

fn expected_struct_symbol(
    type_params: &[ast::TypeParam],
    fields: &[StructField],
    is_public: bool,
) -> ExpectedStructSymbol {
    ExpectedStructSymbol::new(type_params, fields, is_public)
}

fn expected_enum_symbol(
    type_params: &[ast::TypeParam],
    variants: &[EnumVariant],
    is_public: bool,
) -> ExpectedEnumSymbol {
    ExpectedEnumSymbol::new(type_params, variants, is_public)
}

fn expected_variant_symbol(
    owner_name: &str,
    is_public: bool,
    payload: &Option<AstType>,
) -> ExpectedVariantSymbol {
    ExpectedVariantSymbol::new(owner_name, is_public, payload)
}

fn expected_import_symbol(source: &str) -> ExpectedImportSymbol {
    ExpectedImportSymbol::new(source)
}

fn expected_module_symbol(name: &str) -> ExpectedModuleSymbol {
    ExpectedModuleSymbol::new(name)
}

fn expected_local_symbol(is_mutable: bool, scope_id: u32) -> ExpectedLocalSymbol {
    ExpectedLocalSymbol::new(is_mutable, scope_id)
}

fn format_type_parameter_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_type_parameter_bounds(bounds: Option<&[TypeParameterBoundMetadata]>) -> String {
    format_resolver_display_list(bounds, |(name, behavior)| format!("{name}: {behavior}"))
}

fn format_type_parameter_bound_refs(bounds: Option<&[TypeParameterBoundRefMetadata]>) -> String {
    format_resolver_display_list(bounds, |bound| {
        format!(
            "{}: {}",
            bound.type_parameter,
            behavior_ref_display(&bound.behavior, &bound.type_args)
        )
    })
}

fn format_parameter_type_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_ast_type_list(types: Option<&[AstType]>) -> String {
    format_resolver_display_list(types, AstType::display_name)
}

fn format_parameter_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn expected_field_metadata(fields: &[StructField]) -> Vec<ExpectedField> {
    let mut expected = Vec::new();
    for field in fields {
        expected.push(ExpectedField::new(&field.name, &field.ty));
    }
    expected
}

fn format_field_types(fields: Option<&[(String, AstType)]>) -> String {
    format_resolver_named_list(fields, AstType::display_name)
}

fn format_field_type_names(fields: Option<&[(String, String)]>) -> String {
    format_resolver_named_list(fields, String::clone)
}

fn expected_variant_name_metadata(variants: &[EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

fn format_variant_names(variants: Option<&[String]>) -> String {
    format_resolver_string_list(variants)
}

fn format_resolver_string_list(values: Option<&[String]>) -> String {
    format_resolver_display_list(values, String::clone)
}

fn format_resolver_display_list<T>(
    values: Option<&[T]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    values
        .map(|values| format!("({})", join_resolver_display_values(values, display_value)))
        .unwrap_or_else(|| "unknown".to_string())
}

fn join_resolver_strings(values: &[String]) -> String {
    values.join(", ")
}

fn join_resolver_display_values<T>(values: &[T], display_value: impl Fn(&T) -> String) -> String {
    let entries = values.iter().map(display_value).collect::<Vec<_>>();
    join_resolver_strings(&entries)
}

fn format_resolver_named_list<T>(
    values: Option<&[(String, T)]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    values
        .map(|values| {
            let entries = values
                .iter()
                .map(|(name, value)| format!("{name}: {}", display_value(value)))
                .collect::<Vec<_>>();
            format!("({})", join_resolver_strings(&entries))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn expected_behavior_method_metadata(
    methods: &[ast::BehaviorMethod],
) -> Vec<ExpectedBehaviorMethod> {
    let mut expected = Vec::new();
    for method in methods {
        expected.push(ExpectedBehaviorMethod::new(method));
    }
    expected
}

fn expected_behavior_associations(program: &ast::Program) -> ExpectedBehaviorAssociations {
    ExpectedBehaviorAssociations::new(program)
}

fn expected_behavior_edge(behavior: &str, type_args: &[AstType]) -> ExpectedBehaviorEdge {
    ExpectedBehaviorEdge::new(behavior, type_args)
}

fn expected_behavior_parent_associations(program: &ast::Program) -> ExpectedBehaviorEdges {
    ExpectedBehaviorEdges::parents_from(program)
}

fn expected_resolver_declaration_symbols(program: &ast::Program) -> HashSet<(Namespace, String)> {
    let mut expected = HashSet::new();
    let validate_imports = program
        .declarations
        .iter()
        .any(|decl| matches!(decl, Declaration::Import { .. }));
    for decl in &program.declarations {
        match decl {
            Declaration::Function { name, .. } => {
                expected.insert((Namespace::Value, name.clone()));
            }
            Declaration::Method {
                type_name,
                method_name,
                ..
            } => {
                expected.insert((
                    Namespace::Value,
                    method_signature_key(type_name, method_name),
                ));
            }
            Declaration::Struct { name, .. } => {
                expected.insert((Namespace::Type, name.clone()));
            }
            Declaration::Enum { name, variants, .. } => {
                expected.insert((Namespace::Type, name.clone()));
                for variant in variants {
                    expected.insert((Namespace::Variant, variant.name.clone()));
                }
            }
            Declaration::Behavior { name, .. } => {
                expected.insert((Namespace::Behavior, name.clone()));
            }
            Declaration::Import {
                names, module_path, ..
            } if validate_imports => {
                expected.insert((Namespace::Module, module_path.join(".")));
                for name in names {
                    expected.insert((Namespace::Import, name.clone()));
                }
            }
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                for method in methods {
                    if let Declaration::Function { name, .. } = method {
                        expected.insert((Namespace::Value, method_signature_key(type_name, name)));
                    }
                }
            }
            Declaration::Import { .. }
            | Declaration::Requires { .. }
            | Declaration::BehaviorExtends { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
    }
    expected
}

fn expected_resolver_local_symbols(program: &ast::Program) -> HashSet<(String, u32)> {
    let mut expected = HashSet::new();
    let mut scope_cursor = ResolverScopeCursor::default();
    for decl in &program.declarations {
        match decl {
            Declaration::Function { params, body, .. }
            | Declaration::Method { params, body, .. } => {
                let mut locals = scope_cursor.new_scope();
                expected_resolver_parameter_locals(params, &mut locals, &mut expected);
                expected_resolver_expr_locals(body, &mut scope_cursor, &mut locals, &mut expected);
            }
            Declaration::Struct { fields, .. } => {
                for field in fields {
                    if let Some(default) = &field.default {
                        let mut locals = scope_cursor.new_scope();
                        expected_resolver_expr_locals(
                            default,
                            &mut scope_cursor,
                            &mut locals,
                            &mut expected,
                        );
                    }
                }
            }
            Declaration::Behavior { methods, .. } => {
                for method in methods {
                    if let Some(default_body) = &method.default_body {
                        let mut locals = scope_cursor.new_scope();
                        expected_resolver_parameter_locals(
                            &method.params,
                            &mut locals,
                            &mut expected,
                        );
                        expected_resolver_expr_locals(
                            default_body,
                            &mut scope_cursor,
                            &mut locals,
                            &mut expected,
                        );
                    }
                }
            }
            Declaration::ImplBlock { methods, .. } => {
                for method in methods {
                    if let Declaration::Function { params, body, .. } = method {
                        let mut locals = scope_cursor.new_scope();
                        expected_resolver_parameter_locals(params, &mut locals, &mut expected);
                        expected_resolver_expr_locals(
                            body,
                            &mut scope_cursor,
                            &mut locals,
                            &mut expected,
                        );
                    }
                }
            }
            Declaration::TopLevelExpr { expr, .. } => {
                let mut locals = scope_cursor.new_scope();
                expected_resolver_expr_locals(expr, &mut scope_cursor, &mut locals, &mut expected);
            }
            Declaration::Enum { .. }
            | Declaration::Import { .. }
            | Declaration::Requires { .. }
            | Declaration::BehaviorExtends { .. }
            | Declaration::Error { .. } => {}
        }
    }
    expected
}

fn expected_resolver_parameter_locals(
    params: &[Param],
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    for param in params {
        expected_resolver_local(&param.name, param.mutable, locals, expected);
    }
}

fn expected_resolver_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match expr {
        Expression::BinaryOp { left, right, .. } => {
            expected_resolver_expr_locals(left, scope_cursor, locals, expected);
            expected_resolver_expr_locals(right, scope_cursor, locals, expected);
        }
        Expression::UnaryOp { operand, .. } => {
            expected_resolver_expr_locals(operand, scope_cursor, locals, expected);
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MethodCall { receiver, args, .. } => {
            expected_resolver_expr_locals(receiver, scope_cursor, locals, expected);
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MemberAccess { object, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
        }
        Expression::IndexAccess { object, index, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
            expected_resolver_expr_locals(index, scope_cursor, locals, expected);
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::EnumVariant { payload, .. } => {
            if let Some(payload) = payload {
                expected_resolver_expr_locals(payload, scope_cursor, locals, expected);
            }
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                expected_resolver_expr_locals(element, scope_cursor, locals, expected);
            }
        }
        Expression::Match {
            scrutinee, arms, ..
        } => {
            expected_resolver_expr_locals(scrutinee, scope_cursor, locals, expected);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    let mut guard_locals = scope_cursor.child_scope(locals);
                    expected_resolver_pattern_locals(
                        &arm.pattern,
                        scope_cursor,
                        &mut guard_locals,
                        expected,
                    );
                    expected_resolver_expr_locals(guard, scope_cursor, &mut guard_locals, expected);
                }
                let mut arm_locals = scope_cursor.child_scope(locals);
                expected_resolver_pattern_locals(
                    &arm.pattern,
                    scope_cursor,
                    &mut arm_locals,
                    expected,
                );
                expected_resolver_expr_locals(&arm.body, scope_cursor, &mut arm_locals, expected);
            }
        }
        Expression::WhileLoop {
            condition, body, ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            let mut body_locals = scope_cursor.child_scope(locals);
            expected_resolver_expr_locals(body, scope_cursor, &mut body_locals, expected);
        }
        Expression::Loop { body, .. } => {
            let mut body_locals = scope_cursor.child_scope(locals);
            expected_resolver_expr_locals(body, scope_cursor, &mut body_locals, expected);
        }
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            let mut then_locals = scope_cursor.child_scope(locals);
            expected_resolver_expr_locals(then_body, scope_cursor, &mut then_locals, expected);
            if let Some(else_body) = else_body {
                let mut else_locals = scope_cursor.child_scope(locals);
                expected_resolver_expr_locals(else_body, scope_cursor, &mut else_locals, expected);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            let mut block_locals = scope_cursor.child_scope(locals);
            for statement in statements {
                expected_resolver_statement_locals(
                    statement,
                    scope_cursor,
                    &mut block_locals,
                    expected,
                );
            }
            if let Some(expr) = expr {
                expected_resolver_expr_locals(expr, scope_cursor, &mut block_locals, expected);
            }
        }
        Expression::Return { value, .. } => {
            if let Some(value) = value {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::Closure { params, body, .. } => {
            let mut closure_locals = scope_cursor.child_scope(locals);
            for param in params {
                expected_resolver_local(&param.name, false, &mut closure_locals, expected);
            }
            expected_resolver_expr_locals(body, scope_cursor, &mut closure_locals, expected);
        }
        Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        Expression::StringInterpolation { parts, .. } => {
            for part in parts {
                if let ast::StringPart::Expr(expr) = part {
                    expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
                }
            }
        }
        Expression::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        Expression::Identifier { .. }
        | Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::CharLiteral { .. }
        | Expression::Break { .. }
        | Expression::Continue { .. }
        | Expression::Error { .. } => {}
    }
}

fn expected_resolver_statement_locals(
    statement: &ast::Statement,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match statement {
        ast::Statement::VarDecl {
            name,
            value,
            mutable,
            constant,
            ..
        } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            if *constant || *mutable || !locals.is_mutable(name) {
                expected_resolver_local(name, *mutable, locals, expected);
            }
        }
        ast::Statement::Assignment { target, value, .. } => {
            expected_resolver_expr_locals(target, scope_cursor, locals, expected);
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Statement::Expression { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        ast::Statement::Block { stmts, .. } => {
            let mut block_locals = scope_cursor.child_scope(locals);
            for statement in stmts {
                expected_resolver_statement_locals(
                    statement,
                    scope_cursor,
                    &mut block_locals,
                    expected,
                );
            }
        }
    }
}

fn expected_resolver_pattern_locals(
    pattern: &ast::Pattern,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match pattern {
        ast::Pattern::Identifier { name, .. } => {
            expected_resolver_local(name, false, locals, expected);
        }
        ast::Pattern::Struct { fields, .. } => {
            for (name, nested) in fields {
                if let Some(nested) = nested {
                    expected_resolver_pattern_locals(nested, scope_cursor, locals, expected);
                } else {
                    expected_resolver_local(name, false, locals, expected);
                }
            }
        }
        ast::Pattern::Enum {
            payload: Some(payload),
            ..
        } => {
            expected_resolver_pattern_locals(payload, scope_cursor, locals, expected);
        }
        ast::Pattern::Or { patterns, .. } => {
            for pattern in patterns {
                expected_resolver_pattern_locals(pattern, scope_cursor, locals, expected);
            }
        }
        ast::Pattern::Literal { value, .. } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Pattern::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        ast::Pattern::Wildcard { .. }
        | ast::Pattern::Enum { payload: None, .. }
        | ast::Pattern::BoolTrue { .. }
        | ast::Pattern::BoolFalse { .. } => {}
    }
}

fn expected_resolver_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected.insert((name.to_string(), locals.current_scope_id));
    locals.insert(name.to_string(), mutable);
}

fn format_behavior_method_signatures(methods: Option<&[MethodSignatureMetadata]>) -> String {
    format_resolver_display_list(methods, |(name, params, return_type)| {
        format!("{name}({}) {return_type}", params.join(", "))
    })
}

fn format_behavior_method_types(methods: Option<&[BehaviorMethodTypeMetadata]>) -> String {
    format_resolver_display_list(methods, |method| {
        let params = method
            .parameter_types
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                let name = method
                    .parameter_names
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("_");
                format!("{name}: {}", ty.display_name())
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}({}) {}",
            method.name,
            params,
            method.return_type.display_name()
        )
    })
}

fn format_behavior_ref_names(parents: Option<&[String]>) -> String {
    format_resolver_nonempty_joined_list(parents, String::clone)
}

fn format_behavior_refs(refs: Option<&[BehaviorRefMetadata]>) -> String {
    format_resolver_nonempty_joined_list(refs, |behavior| {
        behavior_ref_display(&behavior.name, &behavior.type_args)
    })
}

fn format_resolver_nonempty_joined_list<T>(
    values: Option<&[T]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    match values {
        Some(values) if !values.is_empty() => join_resolver_display_values(values, display_value),
        _ => "none".to_string(),
    }
}

fn behavior_ref_names_match(actual: Option<&[String]>, expected: &[String]) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}

fn behavior_refs_match(
    actual: Option<&[BehaviorRefMetadata]>,
    expected: &[BehaviorRefMetadata],
) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::declarations::StructField;
    use crate::ast::expressions::BinaryOp;
    use crate::error::Span;

    fn parse_program(src: &str) -> ast::Program {
        let mut files = crate::error::FileTable::new();
        let file_id = files.add_file("test.zen".to_string(), src.to_string());
        let tokens = crate::lexer::tokenize(src, file_id).expect("tokenize");
        crate::parser::parse(tokens, file_id).expect("parse")
    }

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
    fn method_signature_key_helpers_share_receiver_parsing() {
        assert_eq!(method_signature_key("Point", "get"), "Point.get");
        assert_eq!(
            method_signature_key_parts("Point.get"),
            Some(("Point", "get"))
        );
        assert_eq!(method_signature_receiver_name("Point.get"), Some("Point"));
        assert_eq!(
            method_signature_method_name_for_receiver("Point.get", "Point"),
            Some("get")
        );
        assert_eq!(
            method_signature_method_name_for_receiver("Other.get", "Point"),
            None
        );
        assert!(is_method_signature_key("Point.get"));
        assert_eq!(method_signature_key_parts("plain"), None);
        assert_eq!(method_signature_receiver_name("plain"), None);
        assert!(!is_method_signature_key("plain"));
    }

    #[test]
    fn resolver_symbol_lookup_helpers_share_definition_span_fallbacks() {
        let program = parse_program(
            r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { return self.x }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let Declaration::Method { span, .. } = &program.declarations[1] else {
            panic!("expected method declaration");
        };
        let span = *span;

        assert_eq!(
            TypeChecker::resolver_symbol_name_for(
                &symbols,
                Namespace::Value,
                "Point.missing",
                span
            ),
            "Point.get"
        );
        assert_eq!(
            TypeChecker::resolver_method_signature_name_for(
                &symbols,
                "Missing.missing",
                "Missing",
                span
            ),
            "Point.get"
        );
        assert_eq!(
            TypeChecker::resolver_method_signature_symbol_by_span(&symbols, span)
                .map(|symbol| symbol.name.as_str()),
            Some("Point.get")
        );
    }

    #[test]
    fn resolver_count_display_formats_known_and_missing_counts() {
        assert_eq!(resolver_count_display(Some(2)), "2");
        assert_eq!(resolver_count_display(None), "unknown");
    }

    #[test]
    fn count_validation_formats_message() {
        let validation = CountValidation {
            label: "parameter count",
            code: "COUNT",
        };

        assert_eq!(validation.code, "COUNT");
        assert_eq!(
            validation.message("value", "add", Some(1), 2),
            "resolver value symbol 'add' has parameter count 1, expected 2"
        );
        assert_eq!(
            validation.message("variant", "Some", None, 1),
            "resolver variant symbol 'Some' has parameter count unknown, expected 1"
        );
    }

    #[test]
    fn count_validation_uses_value_parameter_resolver_code() {
        let validation = CountValidation::value_parameter_resolver_code();

        assert_eq!(validation.label, "parameter count");
        assert_eq!(validation.code, "E0211");
    }

    #[test]
    fn count_validation_uses_field_resolver_code() {
        let validation = CountValidation::field_resolver_code();

        assert_eq!(validation.label, "field count");
        assert_eq!(validation.code, "E0214");
    }

    #[test]
    fn count_validation_uses_variant_payload_resolver_code() {
        let validation = CountValidation::variant_payload_resolver_code();

        assert_eq!(validation.label, "payload count");
        assert_eq!(validation.code, "E0215");
    }

    #[test]
    fn type_parameter_validation_formats_messages() {
        let validation = TypeParameterValidation {
            count_code: "COUNT",
            name_code: "NAMES",
            bound_code: "BOUNDS",
            bound_ref_code: "BOUND_REFS",
        };

        assert_eq!(validation.name_code, "NAMES");
        assert_eq!(
            validation.name_message("value", "identity", "(U)", "(T)"),
            "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
        );
        assert_eq!(
            validation.bound_message("type", "Box", "(T: Other)", "(T: Json)"),
            "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
        );
        assert_eq!(
            validation.bound_ref_message("behavior", "Serializable", "(T: Json<i32>)", "(T: Json<T>)"),
            "resolver behavior symbol 'Serializable' has type parameter bound refs '(T: Json<i32>)', expected '(T: Json<T>)'"
        );
    }

    #[test]
    fn type_parameter_validation_uses_type_like_resolver_codes() {
        let validation = TypeParameterValidation::type_like_resolver_codes();

        assert_eq!(validation.count_code, "E0213");
        assert_eq!(validation.name_code, "E0346");
        assert_eq!(validation.bound_code, "E0222");
        assert_eq!(validation.bound_ref_code, "E0350");
    }

    #[test]
    fn type_parameter_validation_uses_value_resolver_codes() {
        let validation = TypeParameterValidation::value_resolver_codes();

        assert_eq!(validation.count_code, "E0220");
        assert_eq!(validation.name_code, "E0347");
        assert_eq!(validation.bound_code, "E0221");
        assert_eq!(validation.bound_ref_code, "E0351");
    }

    #[test]
    fn type_parameter_validation_builds_count_validation() {
        let validation = TypeParameterValidation {
            count_code: "COUNT",
            name_code: "NAMES",
            bound_code: "BOUNDS",
            bound_ref_code: "BOUND_REFS",
        }
        .count_validation();

        assert_eq!(validation.label, "type parameter count");
        assert_eq!(validation.code, "COUNT");
    }

    #[test]
    fn value_parameter_validation_formats_messages() {
        let validation = ValueParameterValidation {
            name_code: "NAMES",
            display_type_code: "TYPES",
            typed_type_code: "TYPED_TYPES",
        };

        assert_eq!(validation.name_code, "NAMES");
        assert_eq!(
            validation.name_message("add", "(a, other)", "(a, b)"),
            "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
        );
        assert_eq!(
            validation.display_type_message("add", "(i32, i32)", "(i32, f64)"),
            "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
        );
        assert_eq!(
            validation.typed_type_message("apply", "(i32)", "((i32) i32)"),
            "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
        );
    }

    #[test]
    fn value_parameter_validation_uses_resolver_codes() {
        let validation = ValueParameterValidation::resolver_codes();

        assert_eq!(validation.name_code, "E0223");
        assert_eq!(validation.display_type_code, "E0216");
        assert_eq!(validation.typed_type_code, "E0356");
    }

    #[test]
    fn return_validation_formats_messages() {
        let validation = ReturnValidation {
            display_code: "RETURN",
            typed_code: "TYPED_RETURN",
        };

        assert_eq!(validation.display_code, "RETURN");
        assert_eq!(
            validation.display_message("main", "bool", "i32"),
            "resolver value symbol 'main' has return type 'bool', expected 'i32'"
        );
        assert_eq!(
            validation.typed_message("apply", "i32", "(i32) i32"),
            "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
        );
    }

    #[test]
    fn return_validation_uses_resolver_codes() {
        let validation = ReturnValidation::resolver_codes();

        assert_eq!(validation.display_code, "E0212");
        assert_eq!(validation.typed_code, "E0357");
    }

    #[test]
    fn behavior_method_validation_formats_messages() {
        let validation = BehaviorMethodValidation {
            display_code: "METHODS",
            typed_code: "TYPED_METHODS",
        };

        assert_eq!(validation.display_code, "METHODS");
        assert_eq!(
            validation.display_message("Serializable", "(encode(Self) bool)", "(encode(Self) str)"),
            "resolver behavior symbol 'Serializable' has methods '(encode(Self) bool)', expected '(encode(Self) str)'"
        );
        assert_eq!(
            validation.typed_message(
                "Mapper",
                "(map(__arg0: Self, __arg1: i32) i32)",
                "(map(__arg0: Self, __arg1: (i32) i32) i32)"
            ),
            "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) i32)'"
        );
    }

    #[test]
    fn behavior_method_validation_uses_resolver_codes() {
        let validation = BehaviorMethodValidation::resolver_codes();

        assert_eq!(validation.display_code, "E0219");
        assert_eq!(validation.typed_code, "E0355");
    }

    #[test]
    fn field_validation_formats_messages() {
        let validation = FieldValidation {
            display_code: "FIELDS",
            typed_code: "TYPED_FIELDS",
        };

        assert_eq!(validation.display_code, "FIELDS");
        assert_eq!(
            validation.display_message("type", "Point", "(x: i32)", "(x: f64)"),
            "resolver type symbol 'Point' has fields '(x: i32)', expected '(x: f64)'"
        );
        assert_eq!(
            validation.typed_message("type", "Pipeline", "(callback: i32)", "(callback: (i32) i32)"),
            "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
        );
    }

    #[test]
    fn field_validation_uses_resolver_codes() {
        let validation = FieldValidation::resolver_codes();

        assert_eq!(validation.display_code, "E0217");
        assert_eq!(validation.typed_code, "E0358");
    }

    #[test]
    fn variant_payload_validation_formats_messages() {
        let validation = VariantPayloadValidation {
            display_code: "PAYLOAD",
            typed_code: "TYPED_PAYLOAD",
        };

        assert_eq!(validation.display_code, "PAYLOAD");
        assert_eq!(
            validation.display_message("Some", "bool", "i32"),
            "resolver variant symbol 'Some' has payload type 'bool', expected 'i32'"
        );
        assert_eq!(
            validation.typed_message("Wrap", "i32", "(i32) i32"),
            "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
        );
    }

    #[test]
    fn variant_payload_validation_uses_resolver_codes() {
        let validation = VariantPayloadValidation::resolver_codes();

        assert_eq!(validation.display_code, "E0218");
        assert_eq!(validation.typed_code, "E0359");
    }

    #[test]
    fn variant_owner_validation_formats_message() {
        let validation = VariantOwnerValidation { code: "OWNER" };

        assert_eq!(validation.code, "OWNER");
        assert_eq!(
            validation.message("Some", "Result", "Option"),
            "resolver variant symbol 'Some' has owner 'Result', expected 'Option'"
        );
    }

    #[test]
    fn variant_owner_validation_uses_resolver_code() {
        let validation = VariantOwnerValidation::resolver_code();

        assert_eq!(validation.code, "E0242");
    }

    #[test]
    fn variant_name_validation_formats_message() {
        let validation = VariantNameValidation { code: "VARIANTS" };

        assert_eq!(validation.code, "VARIANTS");
        assert_eq!(
            validation.message("Option", "(Some)", "(Some, None)"),
            "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'"
        );
    }

    #[test]
    fn variant_name_validation_uses_resolver_code() {
        let validation = VariantNameValidation::resolver_code();

        assert_eq!(validation.code, "E0241");
    }

    #[test]
    fn resolver_metadata_display_formats_known_and_missing_values() {
        assert_eq!(resolver_metadata_display(Some("Point")), "Point");
        assert_eq!(resolver_metadata_display(None), "unknown");
        assert_eq!(
            resolver_ast_type_metadata_display(Some(&AstType::I32)),
            "i32"
        );
        assert_eq!(resolver_ast_type_metadata_display(None), "unknown");
        assert_eq!(
            optional_ast_type_display(Some(&AstType::Bool), "none"),
            "bool"
        );
        assert_eq!(optional_ast_type_display(None, "none"), "none");
    }

    #[test]
    fn resolver_string_list_display_formats_known_and_missing_lists() {
        let names = vec!["T".to_string(), "U".to_string()];
        assert_eq!(join_resolver_strings(&names), "T, U");
        assert_eq!(
            join_resolver_display_values(&[AstType::I32, AstType::Bool], AstType::display_name),
            "i32, bool"
        );
        assert_eq!(format_resolver_string_list(Some(&names)), "(T, U)");
        assert_eq!(format_resolver_string_list(None), "unknown");
    }

    #[test]
    fn resolver_display_list_formats_mapped_known_and_missing_items() {
        let types = vec![AstType::I32, AstType::Bool];
        assert_eq!(format_ast_type_list(Some(&types)), "(i32, bool)");
        assert_eq!(format_ast_type_list(None), "unknown");

        let bounds = vec![("T".to_string(), "Display".to_string())];
        assert_eq!(format_type_parameter_bounds(Some(&bounds)), "(T: Display)");
        assert_eq!(format_type_parameter_bounds(None), "unknown");

        let bound_refs = vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Display".to_string(),
            type_args: vec![AstType::I32],
        }];
        assert_eq!(
            format_type_parameter_bound_refs(Some(&bound_refs)),
            "(T: Display<i32>)"
        );
        assert_eq!(format_type_parameter_bound_refs(None), "unknown");
    }

    #[test]
    fn resolver_nonempty_joined_list_formats_present_empty_and_missing_items() {
        let names = vec!["Json".to_string(), "Debug".to_string()];
        assert_eq!(format_behavior_ref_names(Some(&names)), "Json, Debug");
        assert_eq!(format_behavior_ref_names(Some(&[])), "none");
        assert_eq!(format_behavior_ref_names(None), "none");

        let refs = vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }];
        assert_eq!(format_behavior_refs(Some(&refs)), "Json<i32>");
        assert_eq!(format_behavior_refs(Some(&[])), "none");
        assert_eq!(format_behavior_refs(None), "none");
    }

    #[test]
    fn resolver_behavior_ref_helpers_share_pop_and_peek_selection() {
        let refs = VecDeque::from(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]);
        let mut refs_by_type = HashMap::from([("Point".to_string(), refs.clone())]);

        assert_eq!(
            TypeChecker::peek_resolver_behavior_ref(true, &refs_by_type, "Point", "Debug")
                .map(|reference| reference.name.as_str()),
            Some("Debug")
        );
        assert_eq!(
            TypeChecker::pop_resolver_behavior_ref(true, &mut refs_by_type, "Point", "Debug")
                .map(|reference| reference.name),
            Some("Debug".to_string())
        );

        let mut refs_by_type = HashMap::from([("Point".to_string(), refs)]);
        assert_eq!(
            TypeChecker::peek_resolver_behavior_ref(true, &refs_by_type, "Point", "Missing")
                .map(|reference| reference.name.as_str()),
            Some("Json")
        );
        assert_eq!(
            TypeChecker::pop_resolver_behavior_ref(true, &mut refs_by_type, "Point", "Missing")
                .map(|reference| reference.name),
            Some("Json".to_string())
        );
        assert!(
            TypeChecker::peek_resolver_behavior_ref(false, &refs_by_type, "Point", "Debug")
                .is_none()
        );
        assert!(
            TypeChecker::pop_resolver_behavior_ref(false, &mut refs_by_type, "Point", "Debug")
                .is_none()
        );
    }

    #[test]
    fn resolver_behavior_ref_queue_selection_prefers_exact_then_front() {
        let refs = VecDeque::from(vec![
            BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            },
            BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            },
        ]);

        assert_eq!(
            TypeChecker::resolver_behavior_ref_queue_index(&refs, "Debug"),
            Some(1)
        );
        assert_eq!(
            TypeChecker::resolver_behavior_ref_queue_index(&refs, "Missing"),
            Some(0)
        );
        assert_eq!(
            TypeChecker::resolver_behavior_ref_queue_index(&VecDeque::new(), "Missing"),
            None
        );
    }

    #[test]
    fn named_queue_selection_prefers_exact_then_front() {
        let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

        assert_eq!(
            TypeChecker::named_queue_index(&items, "Debug", String::as_str),
            Some(1)
        );
        assert_eq!(
            TypeChecker::named_queue_index(&items, "Missing", String::as_str),
            Some(0)
        );
        assert_eq!(
            TypeChecker::named_queue_index(&VecDeque::<String>::new(), "Missing", String::as_str),
            None
        );
    }

    #[test]
    fn named_queue_selection_can_preserve_front_for_future_match() {
        let items = VecDeque::from(["Json".to_string(), "Debug".to_string()]);

        assert_eq!(
            TypeChecker::named_queue_index_preserving_future_front(
                &items,
                "Debug",
                Vec::<&str>::new(),
                String::as_str,
            ),
            Some(1)
        );
        assert_eq!(
            TypeChecker::named_queue_index_preserving_future_front(
                &items,
                "Missing",
                ["Json"],
                String::as_str,
            ),
            None
        );
        assert_eq!(
            TypeChecker::named_queue_index_preserving_future_front(
                &items,
                "Missing",
                ["Other"],
                String::as_str,
            ),
            Some(0)
        );
    }

    #[test]
    fn impl_effective_method_name_prefers_resolver_then_ast_then_collected_signature() {
        let mut tc = TypeChecker::new();
        tc.resolver_backed_collection = true;
        tc.methods.insert(
            "Point.describe".to_string(),
            FuncInfo {
                name: "Point.describe".to_string(),
                params: Vec::new(),
                return_type: AstType::Void,
                type_params: Vec::new(),
                type_param_bounds: HashMap::new(),
            },
        );
        let mut unmatched = VecDeque::from([
            "encode".to_string(),
            "debug".to_string(),
            "describe".to_string(),
        ]);

        assert_eq!(
            tc.impl_effective_method_name(
                &mut unmatched,
                "stale",
                Some("Point.encode".to_string()),
                "Point",
            ),
            "encode"
        );
        assert_eq!(
            tc.impl_effective_method_name(&mut unmatched, "debug", None, "Point"),
            "debug"
        );
        assert_eq!(
            tc.impl_effective_method_name(&mut unmatched, "missing", None, "Point"),
            "describe"
        );
        assert!(unmatched.is_empty());
    }

    #[test]
    fn resolver_backed_impl_method_key_requires_resolver_collection() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let (span, ast_key) = if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function { name, span, .. } = &mut methods[0] {
                *name = "missing".to_string();
                (*span, TypeChecker::method_key(type_name, name))
            } else {
                panic!("expected impl method");
            }
        } else {
            panic!("expected impl block");
        };
        let mut tc = TypeChecker::new();

        assert_eq!(
            tc.resolver_backed_impl_method_key(Some(&symbols), &ast_key, "Missing", span),
            None
        );
        tc.resolver_backed_collection = true;
        assert_eq!(
            tc.resolver_backed_impl_method_key(Some(&symbols), &ast_key, "Missing", span),
            Some("Point.encode".to_string())
        );
    }

    #[test]
    fn resolver_backed_behavior_impl_method_signature_name_prefers_resolver_key() {
        let tc = TypeChecker::new();
        let mut required = VecDeque::from([
            ast::BehaviorMethod {
                name: "encode".to_string(),
                params: Vec::new(),
                return_type: Some(AstType::Str),
                default_body: None,
                span: Span::dummy(),
            },
            ast::BehaviorMethod {
                name: "debug".to_string(),
                params: Vec::new(),
                return_type: Some(AstType::Str),
                default_body: None,
                span: Span::dummy(),
            },
        ]);

        assert_eq!(
            tc.resolver_backed_behavior_impl_method_signature_name(
                &mut required,
                "stale",
                Some("Point.encode"),
                "Point",
            ),
            Some("encode".to_string())
        );
        assert_eq!(
            tc.resolver_backed_behavior_impl_method_signature_name(
                &mut required,
                "debug",
                None,
                "Point",
            ),
            Some("debug".to_string())
        );
        assert!(required.is_empty());
    }

    #[test]
    fn resolver_backed_method_signature_requires_resolver_collection() {
        let mut tc = TypeChecker::new();
        tc.methods.insert(
            "Point.encode".to_string(),
            FuncInfo {
                name: "Point.encode".to_string(),
                params: Vec::new(),
                return_type: AstType::Str,
                type_params: Vec::new(),
                type_param_bounds: HashMap::new(),
            },
        );

        assert!(tc
            .resolver_backed_method_signature("Point", "encode")
            .is_none());
        tc.resolver_backed_collection = true;
        assert_eq!(
            tc.resolver_backed_method_signature("Point", "encode")
                .map(|info| info.return_type.clone()),
            Some(AstType::Str)
        );
    }

    #[test]
    fn behavior_default_synthesis_skip_requires_resolver_collection_and_missing_impl_ref() {
        let mut tc = TypeChecker::new();
        tc.resolver_missing_behavior_impl_refs
            .insert("Point".to_string());

        assert!(!tc.should_skip_behavior_default_synthesis("Point"));
        tc.resolver_backed_collection = true;
        assert!(tc.should_skip_behavior_default_synthesis("Point"));
        assert!(!tc.should_skip_behavior_default_synthesis("Other"));
    }

    #[test]
    fn resolver_backed_behavior_collection_defers_generic_metadata_to_resolver() {
        let program = parse_program(
            r#"
Json<T: Json<T>>: behavior {
    encode: (Self) T {
        return 1
    }
}
"#,
        );
        let mut tc = TypeChecker::new();

        tc.with_resolver_backed_collection(|checker| {
            checker.collect_declarations(&program.declarations);
        });

        let behavior = tc.behaviors.get("Json").expect("behavior stub");
        assert!(
            behavior.type_params.is_empty(),
            "resolver-backed behavior collection should not keep AST generic names before resolver metadata"
        );
        assert!(
            behavior.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST generic bounds before resolver metadata"
        );
        assert!(
            behavior.methods[0].default_body.is_some(),
            "resolver-backed behavior collection should still keep default bodies for later resolver metadata restoration"
        );
    }

    #[test]
    fn method_key_formats_type_qualified_method_name() {
        assert_eq!(TypeChecker::method_key("Point", "encode"), "Point.encode");
    }

    #[test]
    fn resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks() {
        let tc = TypeChecker::new();
        let mut refs_by_type = HashMap::from([
            (
                "Point".to_string(),
                VecDeque::from(vec![BehaviorRefMetadata {
                    name: "Json".to_string(),
                    type_args: vec![AstType::I32],
                }]),
            ),
            (
                "Label".to_string(),
                VecDeque::from(vec![BehaviorRefMetadata {
                    name: "Debug".to_string(),
                    type_args: vec![],
                }]),
            ),
        ]);
        let missing_refs = HashSet::new();

        assert_eq!(
            tc.resolver_behavior_ref_owner_for(
                &refs_by_type,
                &missing_refs,
                "Json",
                &[AstType::I32]
            ),
            Some("Point".to_string())
        );
        assert_eq!(
            tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
            None
        );

        refs_by_type.remove("Label");
        assert_eq!(
            tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
            Some("Point".to_string())
        );

        refs_by_type.clear();
        let missing_refs = HashSet::from(["Recovered".to_string()]);
        assert_eq!(
            tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
            Some("Recovered".to_string())
        );
    }

    #[test]
    fn resolver_symbol_metadata_helper_requires_symbol_and_selected_metadata() {
        let program = parse_program(
            r#"
Point: { x: i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");

        assert_eq!(
            TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| {
                symbol.field_types.as_ref()
            })
            .map(|(_, fields)| fields[0].0.as_str()),
            Some("x")
        );
        symbols.set_field_types_for_test(Namespace::Type, "Point", None);
        assert!(TypeChecker::resolver_symbol_metadata(
            &symbols,
            Namespace::Type,
            "Point",
            |symbol| symbol.field_types.as_ref()
        )
        .is_none());
        assert!(TypeChecker::resolver_symbol_metadata(
            &symbols,
            Namespace::Type,
            "Missing",
            |symbol| symbol.field_types.as_ref()
        )
        .is_none());
    }

    #[test]
    fn callable_signature_insert_routes_function_and_method_keys() {
        let mut tc = TypeChecker::new();
        let function = FuncInfo {
            name: "make".to_string(),
            params: vec![],
            return_type: AstType::I32,
            type_params: vec![],
            type_param_bounds: HashMap::new(),
        };
        let method = FuncInfo {
            name: "Point.get".to_string(),
            params: vec![("self".to_string(), AstType::Named("Point".to_string()))],
            return_type: AstType::I32,
            type_params: vec![],
            type_param_bounds: HashMap::new(),
        };

        tc.insert_callable_signature("make", function);
        tc.insert_callable_signature("Point.get", method);

        assert!(tc.functions.contains_key("make"));
        assert!(!tc.methods.contains_key("make"));
        assert!(tc.methods.contains_key("Point.get"));
        assert!(!tc.functions.contains_key("Point.get"));
    }

    #[test]
    fn generic_callable_template_mut_routes_function_and_method_keys() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T { return value }

Box.get<T> = (self: Box<T>) T { return self.value }
"#,
        );
        let ast::Declaration::Function {
            type_params: function_type_params,
            params: function_params,
            return_type: function_return_type,
            body: function_body,
            span: function_span,
            ..
        } = &program.declarations[1]
        else {
            panic!("expected generic function");
        };
        let ast::Declaration::Method {
            type_params: method_type_params,
            params: method_params,
            return_type: method_return_type,
            body: method_body,
            span: method_span,
            ..
        } = &program.declarations[2]
        else {
            panic!("expected generic method");
        };
        let mut tc = TypeChecker::new();
        tc.generic_functions.insert(
            "identity".to_string(),
            generic_template_from_type_params(
                function_type_params,
                function_params,
                function_return_type,
                function_body,
                *function_span,
            )
            .expect("generic function template"),
        );
        tc.generic_methods.insert(
            "Box.get".to_string(),
            generic_template_from_type_params(
                method_type_params,
                method_params,
                method_return_type,
                method_body,
                *method_span,
            )
            .expect("generic method template"),
        );

        tc.generic_callable_template_mut("identity")
            .expect("function template")
            .return_type = Some(AstType::I32);
        tc.generic_callable_template_mut("Box.get")
            .expect("method template")
            .return_type = Some(AstType::Bool);

        assert_eq!(
            tc.generic_functions
                .get("identity")
                .and_then(|template| template.return_type.as_ref()),
            Some(&AstType::I32)
        );
        assert_eq!(
            tc.generic_methods
                .get("Box.get")
                .and_then(|template| template.return_type.as_ref()),
            Some(&AstType::Bool)
        );
        assert!(!tc.generic_methods.contains_key("identity"));
        assert!(!tc.generic_functions.contains_key("Box.get"));
    }

    #[test]
    fn callable_template_rekey_routes_function_and_method_keys() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

identity<T> = (value: T) T { return value }

Box.get<T> = (self: Box<T>) T { return self.value }
"#,
        );
        let ast::Declaration::Function {
            type_params: function_type_params,
            params: function_params,
            return_type: function_return_type,
            body: function_body,
            span: function_span,
            ..
        } = &program.declarations[1]
        else {
            panic!("expected generic function");
        };
        let ast::Declaration::Method {
            type_params: method_type_params,
            params: method_params,
            return_type: method_return_type,
            body: method_body,
            span: method_span,
            ..
        } = &program.declarations[2]
        else {
            panic!("expected generic method");
        };
        let mut tc = TypeChecker::new();
        tc.generic_functions.insert(
            "identity".to_string(),
            generic_template_from_type_params(
                function_type_params,
                function_params,
                function_return_type,
                function_body,
                *function_span,
            )
            .expect("generic function template"),
        );
        tc.generic_methods.insert(
            "Box.get".to_string(),
            generic_template_from_type_params(
                method_type_params,
                method_params,
                method_return_type,
                method_body,
                *method_span,
            )
            .expect("generic method template"),
        );

        tc.rekey_callable_template("identity", "renamed");
        tc.rekey_callable_template("Box.get", "Box.fetch");

        assert!(tc.generic_functions.contains_key("renamed"));
        assert!(!tc.generic_functions.contains_key("identity"));
        assert!(tc.generic_methods.contains_key("Box.fetch"));
        assert!(!tc.generic_methods.contains_key("Box.get"));
    }

    #[test]
    fn resolver_backed_callable_template_collection_defers_signature_metadata_to_resolver() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

identity<T> = (mut value: T) T { return value }

Box.get<T> = (self: Box<T>, mut fallback: T) T { return fallback }
"#,
        );
        let mut tc = TypeChecker::new();

        tc.with_resolver_backed_collection(|checker| {
            checker.collect_declarations(&program.declarations);
        });

        let function_template = tc
            .generic_functions
            .get("identity")
            .expect("function template stub");
        assert!(
            function_template.type_params.is_empty(),
            "resolver-backed generic function templates should not keep AST generic names before resolver metadata"
        );
        assert_eq!(function_template.params.len(), 1);
        assert_eq!(function_template.params[0].name, "");
        assert_eq!(function_template.params[0].ty, AstType::Void);
        assert!(function_template.params[0].mutable);
        assert_eq!(function_template.return_type, None);

        let method_template = tc
            .generic_methods
            .get("Box.get")
            .expect("method template stub");
        assert!(
            method_template.type_params.is_empty(),
            "resolver-backed generic method templates should not keep AST generic names before resolver metadata"
        );
        assert_eq!(method_template.params.len(), 2);
        assert_eq!(method_template.params[1].name, "");
        assert_eq!(method_template.params[1].ty, AstType::Void);
        assert!(method_template.params[1].mutable);
        assert_eq!(method_template.return_type, None);
    }

    #[test]
    fn behavior_ref_validation_maps_role_and_check_diagnostics() {
        let cases = [
            (
                BehaviorRefRole::Parent,
                BehaviorRefCheck::Contains,
                ("behavior", "parents", "parent refs", "E0235", "E0245"),
            ),
            (
                BehaviorRefRole::Parent,
                BehaviorRefCheck::List,
                ("behavior", "parents", "parent refs", "E0240", "E0246"),
            ),
            (
                BehaviorRefRole::Impl,
                BehaviorRefCheck::Contains,
                (
                    "type",
                    "behavior impls",
                    "behavior impl refs",
                    "E0236",
                    "E0247",
                ),
            ),
            (
                BehaviorRefRole::Impl,
                BehaviorRefCheck::List,
                (
                    "type",
                    "behavior impls",
                    "behavior impl refs",
                    "E0238",
                    "E0248",
                ),
            ),
            (
                BehaviorRefRole::Required,
                BehaviorRefCheck::Contains,
                (
                    "type",
                    "behavior requires",
                    "behavior requires refs",
                    "E0237",
                    "E0249",
                ),
            ),
            (
                BehaviorRefRole::Required,
                BehaviorRefCheck::List,
                (
                    "type",
                    "behavior requires",
                    "behavior requires refs",
                    "E0239",
                    "E0250",
                ),
            ),
        ];

        for (role, check, expected) in cases {
            let validation = BehaviorRefValidation::for_role(role, check);
            assert_eq!(
                (
                    validation.symbol_kind,
                    validation.name_label,
                    validation.ref_label,
                    validation.name_code,
                    validation.ref_code,
                ),
                expected
            );
        }

        let contains =
            BehaviorRefValidation::for_role(BehaviorRefRole::Impl, BehaviorRefCheck::Contains);
        assert_eq!(
            contains.contains_name_message("Point", "PrettyJson", "Json<str>"),
            "resolver type symbol 'Point' has behavior impls 'PrettyJson', expected to include 'Json<str>'"
        );
        assert_eq!(
            contains.contains_ref_message("Point", "PrettyJson", "Json<str>"),
            "resolver type symbol 'Point' has behavior impl refs 'PrettyJson', expected to include 'Json<str>'"
        );

        let list = BehaviorRefValidation::for_role(BehaviorRefRole::Parent, BehaviorRefCheck::List);
        assert_eq!(
            list.list_name_message("PrettyJson", "Json, Debug", "Json"),
            "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'"
        );
        assert_eq!(
            list.list_ref_message("PrettyJson", "Json, Debug", "Json"),
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'"
        );
    }

    #[test]
    fn behavior_ref_validation_separates_role_labels_from_check_codes() {
        let parent = BehaviorRefValidation::role_labels(BehaviorRefRole::Parent);
        let implementation = BehaviorRefValidation::role_labels(BehaviorRefRole::Impl);
        let required = BehaviorRefValidation::role_labels(BehaviorRefRole::Required);
        let parent_contains =
            BehaviorRefValidation::codes_for(BehaviorRefRole::Parent, BehaviorRefCheck::Contains);
        let parent_list =
            BehaviorRefValidation::codes_for(BehaviorRefRole::Parent, BehaviorRefCheck::List);

        assert_eq!(parent, ("behavior", "parents", "parent refs"));
        assert_eq!(
            implementation,
            ("type", "behavior impls", "behavior impl refs")
        );
        assert_eq!(
            required,
            ("type", "behavior requires", "behavior requires refs")
        );
        assert_eq!(parent_contains, ("E0235", "E0245"));
        assert_eq!(parent_list, ("E0240", "E0246"));
    }

    #[test]
    fn behavior_ref_actual_selects_role_metadata() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let behavior = symbols
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol");
        let ty = symbols
            .lookup(Namespace::Type, "Point")
            .expect("type symbol");

        let parent = BehaviorRefActual::for_role(behavior, BehaviorRefRole::Parent);
        assert_eq!(format_behavior_ref_names(parent.names), "Json<str>");
        assert_eq!(format_behavior_refs(parent.refs), "Json<str>");

        let implementation = BehaviorRefActual::for_role(ty, BehaviorRefRole::Impl);
        assert_eq!(
            format_behavior_ref_names(implementation.names),
            "PrettyJson"
        );
        assert_eq!(format_behavior_refs(implementation.refs), "PrettyJson");

        let required = BehaviorRefActual::for_role(ty, BehaviorRefRole::Required);
        assert_eq!(format_behavior_ref_names(required.names), "Json<str>");
        assert_eq!(format_behavior_refs(required.refs), "Json<str>");
    }

    #[test]
    fn behavior_ref_actual_exposes_role_metadata_selection() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let behavior = symbols
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol");
        let ty = symbols
            .lookup(Namespace::Type, "Point")
            .expect("type symbol");

        let (parent_names, parent_refs) =
            BehaviorRefActual::metadata_for_role(behavior, BehaviorRefRole::Parent);
        let (impl_names, impl_refs) =
            BehaviorRefActual::metadata_for_role(ty, BehaviorRefRole::Impl);
        let (required_names, required_refs) =
            BehaviorRefActual::metadata_for_role(ty, BehaviorRefRole::Required);

        assert_eq!(format_behavior_ref_names(parent_names), "Json<str>");
        assert_eq!(format_behavior_refs(parent_refs), "Json<str>");
        assert_eq!(format_behavior_ref_names(impl_names), "PrettyJson");
        assert_eq!(format_behavior_refs(impl_refs), "PrettyJson");
        assert_eq!(format_behavior_ref_names(required_names), "Json<str>");
        assert_eq!(format_behavior_refs(required_refs), "Json<str>");
    }

    #[test]
    fn behavior_ref_actual_matches_expected_edges() {
        let names = vec!["Json<i32>".to_string()];
        let refs = vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: vec![AstType::I32],
        }];
        let actual = BehaviorRefActual {
            names: Some(&names),
            refs: Some(&refs),
        };
        let expected = expected_behavior_edge("Json", &[AstType::I32]);
        let mismatch = expected_behavior_edge("Debug", &[]);
        let expected_list =
            ExpectedBehaviorEdgeMetadata::from_edges(std::slice::from_ref(&expected));

        assert!(actual.contains_display(&expected.display));
        assert!(actual.contains_metadata(&expected.metadata));
        assert!(!actual.contains_display(&mismatch.display));
        assert!(!actual.contains_metadata(&mismatch.metadata));
        assert!(actual.names_match(&expected_list.names));
        assert!(actual.refs_match(&expected_list.refs));
    }

    #[test]
    fn expected_parameter_builds_name_display_and_type_together() {
        let parameter = ExpectedParameter::new(
            "mapper",
            &AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            },
        );

        assert_eq!(parameter.name, "mapper");
        assert_eq!(parameter.display, "(i32) str");
        assert_eq!(
            parameter.typed,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            }
        );
    }

    #[test]
    fn expected_return_metadata_defaults_and_displays_together() {
        let explicit = ExpectedReturnMetadata::new(&Some(AstType::Named("Point".to_string())));
        let implicit = ExpectedReturnMetadata::new(&None);

        assert_eq!(explicit.display, "Point");
        assert_eq!(explicit.typed, AstType::Named("Point".to_string()));
        assert_eq!(implicit.display, "void");
        assert_eq!(implicit.typed, AstType::Void);
    }

    #[test]
    fn expected_type_parameter_builds_bound_display_and_ref_together() {
        let type_param = ast::TypeParam {
            name: "T".to_string(),
            constraint: Some("Json".to_string()),
            constraint_type_args: vec![AstType::Named("T".to_string())],
            span: Span::dummy(),
        };

        let expected = ExpectedTypeParameter::new(&type_param);
        let bound = expected.bound.expect("expected bound");

        assert_eq!(expected.name, "T");
        assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
        assert_eq!(bound.reference.type_parameter, "T");
        assert_eq!(bound.reference.behavior, "Json");
        assert_eq!(
            bound.reference.type_args,
            vec![AstType::Named("T".to_string())]
        );
    }

    #[test]
    fn expected_field_builds_display_and_type_together() {
        let field = ExpectedField::new(
            "mapper",
            &AstType::Function {
                params: vec![AstType::Named("Input".to_string())],
                ret: Box::new(AstType::Named("Output".to_string())),
            },
        );

        assert_eq!(
            field.display,
            ("mapper".to_string(), "(Input) Output".to_string())
        );
        assert_eq!(
            field.typed,
            (
                "mapper".to_string(),
                AstType::Function {
                    params: vec![AstType::Named("Input".to_string())],
                    ret: Box::new(AstType::Named("Output".to_string())),
                }
            )
        );
    }

    #[test]
    fn expected_variant_payload_builds_display_and_type_together() {
        let payload = ExpectedVariantPayloadType::new(&Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Bool),
        }));
        let empty_payload = ExpectedVariantPayloadType::new(&None);

        assert_eq!(payload.display, Some("(i32) bool".to_string()));
        assert_eq!(
            payload.typed,
            Some(AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Bool),
            })
        );
        assert_eq!(empty_payload.display, None);
        assert_eq!(empty_payload.typed, None);
    }

    #[test]
    fn expected_behavior_method_builds_signature_and_metadata_together() {
        let method = ast::BehaviorMethod {
            name: "map".to_string(),
            params: vec![Param {
                name: "mapper".to_string(),
                ty: AstType::Function {
                    params: vec![AstType::I32],
                    ret: Box::new(AstType::Str),
                },
                mutable: false,
                span: Span::dummy(),
            }],
            return_type: Some(AstType::Str),
            default_body: None,
            span: Span::dummy(),
        };

        let expected = ExpectedBehaviorMethod::new(&method);

        assert_eq!(
            expected.signature,
            (
                "map".to_string(),
                vec!["(i32) str".to_string()],
                "str".to_string(),
            )
        );
        assert_eq!(expected.metadata.name, "map");
        assert_eq!(
            expected.metadata.parameter_names,
            vec!["mapper".to_string()]
        );
        assert_eq!(
            expected.metadata.parameter_types,
            vec![AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Str),
            }]
        );
        assert_eq!(expected.metadata.return_type, AstType::Str);
    }

    #[test]
    fn expected_value_signature_builds_components_together() {
        let params = vec![Param {
            name: "value".to_string(),
            ty: AstType::Named("T".to_string()),
            mutable: false,
            span: Span::dummy(),
        }];
        let return_type = Some(AstType::Named("T".to_string()));
        let type_params = vec![ast::TypeParam {
            name: "T".to_string(),
            constraint: Some("Json".to_string()),
            constraint_type_args: vec![AstType::Named("T".to_string())],
            span: Span::dummy(),
        }];

        let signature = ExpectedValueSignature::new(&params, &return_type, &type_params);

        assert_eq!(signature.params[0].name, "value");
        assert_eq!(signature.params[0].display, "T");
        assert_eq!(signature.params[0].typed, AstType::Named("T".to_string()));
        assert_eq!(signature.return_type.display, "T");
        assert_eq!(signature.return_type.typed, AstType::Named("T".to_string()));
        assert_eq!(signature.type_params[0].name, "T");
        let bound = signature.type_params[0]
            .bound
            .as_ref()
            .expect("expected bound");
        assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
        assert_eq!(bound.reference.behavior, "Json");
        assert_eq!(
            bound.reference.type_args,
            vec![AstType::Named("T".to_string())]
        );
    }

    #[test]
    fn expected_value_symbol_builds_signature_and_visibility_together() {
        let params = vec![Param {
            name: "value".to_string(),
            ty: AstType::I32,
            mutable: false,
            span: Span::dummy(),
        }];
        let return_type = Some(AstType::Bool);

        let symbol = ExpectedValueSymbol::new(&params, &return_type, &[], true);

        assert!(symbol.is_public);
        assert_eq!(symbol.signature.params[0].name, "value");
        assert_eq!(symbol.signature.params[0].display, "i32");
        assert_eq!(symbol.signature.params[0].typed, AstType::I32);
        assert_eq!(symbol.signature.return_type.display, "bool");
        assert_eq!(symbol.signature.return_type.typed, AstType::Bool);
        assert!(symbol.signature.type_params.is_empty());
    }

    #[test]
    fn expected_type_like_symbol_builds_type_params_and_visibility_together() {
        let type_params = vec![ast::TypeParam {
            name: "T".to_string(),
            constraint: Some("Json".to_string()),
            constraint_type_args: vec![AstType::Named("T".to_string())],
            span: Span::dummy(),
        }];

        let symbol = ExpectedTypeLikeSymbol::new(&type_params, Some(true));

        assert_eq!(symbol.is_public, Some(true));
        assert_eq!(symbol.type_params[0].name, "T");
        let bound = symbol.type_params[0]
            .bound
            .as_ref()
            .expect("expected bound");
        assert_eq!(bound.display, ("T".to_string(), "Json<T>".to_string()));
        assert_eq!(bound.reference.type_parameter, "T");
        assert_eq!(bound.reference.behavior, "Json");
        assert_eq!(
            bound.reference.type_args,
            vec![AstType::Named("T".to_string())]
        );
    }

    #[test]
    fn expected_behavior_symbol_builds_type_like_and_methods_together() {
        let type_params = vec![ast::TypeParam {
            name: "T".to_string(),
            constraint: None,
            constraint_type_args: vec![],
            span: Span::dummy(),
        }];
        let methods = vec![ast::BehaviorMethod {
            name: "encode".to_string(),
            params: vec![Param {
                name: "value".to_string(),
                ty: AstType::Named("Self".to_string()),
                mutable: false,
                span: Span::dummy(),
            }],
            return_type: Some(AstType::Named("T".to_string())),
            default_body: None,
            span: Span::dummy(),
        }];

        let symbol = ExpectedBehaviorSymbol::new(&type_params, &methods, true);

        assert_eq!(symbol.type_like.is_public, Some(true));
        assert_eq!(symbol.type_like.type_params[0].name, "T");
        assert_eq!(symbol.methods[0].signature.0, "encode");
        assert_eq!(symbol.methods[0].signature.1, vec!["Self".to_string()]);
        assert_eq!(symbol.methods[0].signature.2, "T");
        assert_eq!(symbol.methods[0].metadata.name, "encode");
        assert_eq!(
            symbol.methods[0].metadata.parameter_names,
            vec!["value".to_string()]
        );
        assert_eq!(
            symbol.methods[0].metadata.return_type,
            AstType::Named("T".to_string())
        );
    }

    #[test]
    fn expected_struct_symbol_builds_type_like_and_fields_together() {
        let type_params = vec![ast::TypeParam {
            name: "T".to_string(),
            constraint: None,
            constraint_type_args: vec![],
            span: Span::dummy(),
        }];
        let fields = vec![StructField {
            name: "value".to_string(),
            ty: AstType::Named("T".to_string()),
            default: None,
            mutable: false,
            span: Span::dummy(),
        }];

        let symbol = ExpectedStructSymbol::new(&type_params, &fields, true);

        assert_eq!(symbol.type_like.is_public, Some(true));
        assert_eq!(symbol.type_like.type_params[0].name, "T");
        assert_eq!(
            symbol.fields[0].display,
            ("value".to_string(), "T".to_string())
        );
        assert_eq!(
            symbol.fields[0].typed,
            ("value".to_string(), AstType::Named("T".to_string()))
        );
    }

    #[test]
    fn expected_enum_symbol_builds_type_like_and_variants_together() {
        let type_params = vec![ast::TypeParam {
            name: "T".to_string(),
            constraint: None,
            constraint_type_args: vec![],
            span: Span::dummy(),
        }];
        let variants = vec![
            EnumVariant {
                name: "Some".to_string(),
                payload: Some(AstType::Named("T".to_string())),
                span: Span::dummy(),
            },
            EnumVariant {
                name: "None".to_string(),
                payload: None,
                span: Span::dummy(),
            },
        ];

        let symbol = ExpectedEnumSymbol::new(&type_params, &variants, true);

        assert_eq!(symbol.type_like.is_public, Some(true));
        assert_eq!(symbol.type_like.type_params[0].name, "T");
        assert_eq!(
            symbol.variant_names,
            vec!["Some".to_string(), "None".to_string()]
        );
    }

    #[test]
    fn expected_variant_symbol_builds_owner_visibility_and_payload_together() {
        let payload = Some(AstType::Function {
            params: vec![AstType::I32],
            ret: Box::new(AstType::Bool),
        });

        let symbol = ExpectedVariantSymbol::new("Result", true, &payload);

        assert_eq!(symbol.owner_name, "Result");
        assert!(symbol.is_public);
        assert_eq!(symbol.payload.display, Some("(i32) bool".to_string()));
        assert_eq!(
            symbol.payload.typed,
            Some(AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::Bool),
            })
        );
    }

    #[test]
    fn expected_import_symbol_builds_source_and_visibility_together() {
        let symbol = ExpectedImportSymbol::new("std.io");

        assert_eq!(symbol.source, "std.io");
        assert!(!symbol.is_public);
    }

    #[test]
    fn expected_module_symbol_builds_name_source_and_visibility_together() {
        let symbol = ExpectedModuleSymbol::new("std.io");

        assert_eq!(symbol.name, "std.io");
        assert_eq!(symbol.source, None);
        assert!(!symbol.is_public);
    }

    #[test]
    fn expected_local_symbol_builds_scope_mutability_source_and_visibility_together() {
        let symbol = ExpectedLocalSymbol::new(true, 42);

        assert_eq!(symbol.scope_id, 42);
        assert!(symbol.is_mutable);
        assert_eq!(symbol.source, None);
        assert!(!symbol.is_public);
    }

    #[test]
    fn expected_behavior_edge_builds_display_and_metadata_together() {
        let edge = ExpectedBehaviorEdge::new("Json", &[AstType::I32]);

        assert_eq!(edge.display, "Json<i32>");
        assert_eq!(edge.metadata.name, "Json");
        assert_eq!(edge.metadata.type_args, vec![AstType::I32]);
    }

    #[test]
    fn expected_behavior_associations_build_impl_and_required_edges_together() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );

        let expected = ExpectedBehaviorAssociations::new(&program);
        let impl_edge = &expected.impls.edges_for("Point")[0];
        let required_edge = &expected.required.edges_for("Point")[0];

        assert_eq!(impl_edge.display, "Json<str>");
        assert_eq!(impl_edge.metadata.name, "Json");
        assert_eq!(impl_edge.metadata.type_args, vec![AstType::Str]);
        assert_eq!(required_edge.display, "Json<str>");
        assert_eq!(required_edge.metadata.name, "Json");
        assert_eq!(required_edge.metadata.type_args, vec![AstType::Str]);
    }

    #[test]
    fn expected_behavior_edges_build_parent_edges_from_extends_together() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );

        let expected = ExpectedBehaviorEdges::parents_from(&program);
        let edge = &expected.edges_for("PrettyJson")[0];

        assert_eq!(edge.display, "Json");
        assert_eq!(edge.metadata.name, "Json");
        assert_eq!(edge.metadata.type_args, Vec::<AstType>::new());
    }

    #[test]
    fn behavior_ref_role_validation_emits_selected_contains_diagnostics() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let ty = symbols
            .lookup(Namespace::Type, "Point")
            .expect("type symbol");
        let mut tc = TypeChecker::new();

        tc.validate_resolver_behavior_ref_contains_for_role(
            BehaviorRefRole::Impl,
            ty,
            "Point",
            expected_behavior_edge("Json", &[AstType::Str]),
            Span::dummy(),
        );

        assert!(tc.diagnostics.iter().any(|d| d.code == "E0236" && d.message.contains(
            "resolver type symbol 'Point' has behavior impls 'PrettyJson', expected to include 'Json<str>'"
        )));
        assert!(tc.diagnostics.iter().any(|d| d.code == "E0247" && d.message.contains(
            "resolver type symbol 'Point' has behavior impl refs 'PrettyJson', expected to include 'Json<str>'"
        )));
    }

    #[test]
    fn behavior_association_absence_validation_builds_entries() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup(Namespace::Type, "Point")
            .expect("type symbol");
        let entries = BehaviorAssociationAbsenceValidation {
            impl_name_code: "IMPL_NAMES",
            impl_ref_code: "IMPL_REFS",
            required_name_code: "REQUIRED_NAMES",
            required_ref_code: "REQUIRED_REFS",
        }
        .entries(symbol);

        assert_eq!(
            entries,
            [
                AbsentMetadataEntry::new(true, "IMPL_NAMES", "behavior impls"),
                AbsentMetadataEntry::new(true, "IMPL_REFS", "typed behavior impls"),
                AbsentMetadataEntry::new(true, "REQUIRED_NAMES", "behavior requires"),
                AbsentMetadataEntry::new(true, "REQUIRED_REFS", "typed behavior requires"),
            ]
        );
    }

    #[test]
    fn behavior_association_absence_validation_uses_module_resolver_codes() {
        let validation = BehaviorAssociationAbsenceValidation::module_resolver_codes();

        assert_eq!(validation.impl_name_code, "E0279");
        assert_eq!(validation.impl_ref_code, "E0378");
        assert_eq!(validation.required_name_code, "E0280");
        assert_eq!(validation.required_ref_code, "E0379");
    }

    #[test]
    fn behavior_association_absence_validation_uses_import_resolver_codes() {
        let validation = BehaviorAssociationAbsenceValidation::import_resolver_codes();

        assert_eq!(validation.impl_name_code, "E0295");
        assert_eq!(validation.impl_ref_code, "E0369");
        assert_eq!(validation.required_name_code, "E0296");
        assert_eq!(validation.required_ref_code, "E0370");
    }

    #[test]
    fn behavior_association_absence_validation_uses_local_resolver_codes() {
        let validation = BehaviorAssociationAbsenceValidation::local_resolver_codes();

        assert_eq!(validation.impl_name_code, "E0263");
        assert_eq!(validation.impl_ref_code, "E0387");
        assert_eq!(validation.required_name_code, "E0264");
        assert_eq!(validation.required_ref_code, "E0388");
    }

    #[test]
    fn behavior_association_absence_validation_uses_variant_resolver_codes() {
        let validation = BehaviorAssociationAbsenceValidation::variant_resolver_codes();

        assert_eq!(validation.impl_name_code, "E0341");
        assert_eq!(validation.impl_ref_code, "E0395");
        assert_eq!(validation.required_name_code, "E0342");
        assert_eq!(validation.required_ref_code, "E0396");
    }

    #[test]
    fn behavior_association_absence_validation_uses_behavior_resolver_codes() {
        let validation = BehaviorAssociationAbsenceValidation::behavior_resolver_codes();

        assert_eq!(validation.impl_name_code, "E0327");
        assert_eq!(validation.impl_ref_code, "E0401");
        assert_eq!(validation.required_name_code, "E0328");
        assert_eq!(validation.required_ref_code, "E0402");
    }

    #[test]
    fn behavior_association_absence_validation_uses_value_resolver_codes() {
        let validation = BehaviorAssociationAbsenceValidation::value_resolver_codes();

        assert_eq!(validation.impl_name_code, "E0306");
        assert_eq!(validation.impl_ref_code, "E0407");
        assert_eq!(validation.required_name_code, "E0307");
        assert_eq!(validation.required_ref_code, "E0408");
    }

    #[test]
    fn behavior_declaration_absence_validation_builds_entries() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup(Namespace::Behavior, "PrettyJson")
            .expect("behavior symbol");
        let entries = BehaviorDeclarationAbsenceValidation {
            method_signature_code: "METHODS",
            method_type_code: "TYPED_METHODS",
            parent_name_code: "PARENTS",
            parent_ref_code: "TYPED_PARENTS",
        }
        .entries(symbol);

        assert_eq!(
            entries,
            [
                AbsentMetadataEntry::new(true, "METHODS", "behavior methods"),
                AbsentMetadataEntry::new(true, "TYPED_METHODS", "typed behavior methods"),
                AbsentMetadataEntry::new(true, "PARENTS", "behavior parents"),
                AbsentMetadataEntry::new(true, "TYPED_PARENTS", "typed behavior parents"),
            ]
        );
    }

    #[test]
    fn behavior_declaration_absence_validation_uses_module_resolver_codes() {
        let validation = BehaviorDeclarationAbsenceValidation::module_resolver_codes();

        assert_eq!(validation.method_signature_code, "E0277");
        assert_eq!(validation.method_type_code, "E0376");
        assert_eq!(validation.parent_name_code, "E0278");
        assert_eq!(validation.parent_ref_code, "E0377");
    }

    #[test]
    fn behavior_declaration_absence_validation_uses_import_resolver_codes() {
        let validation = BehaviorDeclarationAbsenceValidation::import_resolver_codes();

        assert_eq!(validation.method_signature_code, "E0293");
        assert_eq!(validation.method_type_code, "E0367");
        assert_eq!(validation.parent_name_code, "E0294");
        assert_eq!(validation.parent_ref_code, "E0368");
    }

    #[test]
    fn behavior_declaration_absence_validation_uses_local_resolver_codes() {
        let validation = BehaviorDeclarationAbsenceValidation::local_resolver_codes();

        assert_eq!(validation.method_signature_code, "E0261");
        assert_eq!(validation.method_type_code, "E0385");
        assert_eq!(validation.parent_name_code, "E0262");
        assert_eq!(validation.parent_ref_code, "E0386");
    }

    #[test]
    fn behavior_declaration_absence_validation_uses_variant_resolver_codes() {
        let validation = BehaviorDeclarationAbsenceValidation::variant_resolver_codes();

        assert_eq!(validation.method_signature_code, "E0339");
        assert_eq!(validation.method_type_code, "E0393");
        assert_eq!(validation.parent_name_code, "E0340");
        assert_eq!(validation.parent_ref_code, "E0394");
    }

    #[test]
    fn behavior_declaration_absence_validation_uses_value_resolver_codes() {
        let validation = BehaviorDeclarationAbsenceValidation::value_resolver_codes();

        assert_eq!(validation.method_signature_code, "E0304");
        assert_eq!(validation.method_type_code, "E0405");
        assert_eq!(validation.parent_name_code, "E0305");
        assert_eq!(validation.parent_ref_code, "E0406");
    }

    #[test]
    fn value_signature_absence_validation_builds_entries() {
        let program = parse_program(
            r#"
add = (left: i32, right: i32) i32 { return left + right }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup(Namespace::Value, "add")
            .expect("value symbol");
        let entries = ValueSignatureAbsenceValidation {
            parameter_count_code: "PARAM_COUNT",
            parameter_name_code: "PARAM_NAMES",
            parameter_type_name_code: "PARAM_TYPES",
            parameter_type_code: "TYPED_PARAM_TYPES",
            return_type_code: "RETURN_TYPE",
            typed_return_type_code: "TYPED_RETURN_TYPE",
        }
        .entries(symbol);

        assert!(entries.iter().all(|entry| entry.present));
        assert_eq!(
            entries.map(|entry| entry.message("value", "add")),
            [
                "resolver value symbol 'add' has parameter count metadata, expected none",
                "resolver value symbol 'add' has parameter names metadata, expected none",
                "resolver value symbol 'add' has parameter types metadata, expected none",
                "resolver value symbol 'add' has typed parameter types metadata, expected none",
                "resolver value symbol 'add' has return type metadata, expected none",
                "resolver value symbol 'add' has typed return type metadata, expected none",
            ]
        );
    }

    #[test]
    fn value_signature_absence_validation_uses_module_resolver_codes() {
        let validation = ValueSignatureAbsenceValidation::module_resolver_codes();

        assert_eq!(validation.parameter_count_code, "E0265");
        assert_eq!(validation.parameter_name_code, "E0267");
        assert_eq!(validation.parameter_type_name_code, "E0268");
        assert_eq!(validation.parameter_type_code, "E0371");
        assert_eq!(validation.return_type_code, "E0266");
        assert_eq!(validation.typed_return_type_code, "E0372");
    }

    #[test]
    fn value_signature_absence_validation_uses_import_resolver_codes() {
        let validation = ValueSignatureAbsenceValidation::import_resolver_codes();

        assert_eq!(validation.parameter_count_code, "E0281");
        assert_eq!(validation.parameter_name_code, "E0283");
        assert_eq!(validation.parameter_type_name_code, "E0284");
        assert_eq!(validation.parameter_type_code, "E0362");
        assert_eq!(validation.return_type_code, "E0282");
        assert_eq!(validation.typed_return_type_code, "E0363");
    }

    #[test]
    fn value_signature_absence_validation_uses_local_resolver_codes() {
        let validation = ValueSignatureAbsenceValidation::local_resolver_codes();

        assert_eq!(validation.parameter_count_code, "E0249");
        assert_eq!(validation.parameter_name_code, "E0251");
        assert_eq!(validation.parameter_type_name_code, "E0252");
        assert_eq!(validation.parameter_type_code, "E0380");
        assert_eq!(validation.return_type_code, "E0250");
        assert_eq!(validation.typed_return_type_code, "E0381");
    }

    #[test]
    fn value_signature_absence_validation_uses_type_like_resolver_codes() {
        let validation = ValueSignatureAbsenceValidation::type_like_resolver_codes();

        assert_eq!(validation.parameter_count_code, "E0310");
        assert_eq!(validation.parameter_name_code, "E0312");
        assert_eq!(validation.parameter_type_name_code, "E0313");
        assert_eq!(validation.parameter_type_code, "E0360");
        assert_eq!(validation.return_type_code, "E0311");
        assert_eq!(validation.typed_return_type_code, "E0361");
    }

    #[test]
    fn value_signature_absence_validation_uses_variant_resolver_codes() {
        let validation = ValueSignatureAbsenceValidation::variant_resolver_codes();

        assert_eq!(validation.parameter_count_code, "E0330");
        assert_eq!(validation.parameter_name_code, "E0332");
        assert_eq!(validation.parameter_type_name_code, "E0333");
        assert_eq!(validation.parameter_type_code, "E0389");
        assert_eq!(validation.return_type_code, "E0331");
        assert_eq!(validation.typed_return_type_code, "E0390");
    }

    #[test]
    fn type_parameter_absence_validation_builds_entries() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

identity<T: Json> = (value: T) T { return value }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup(Namespace::Value, "identity")
            .expect("value symbol");
        let entries = TypeParameterAbsenceValidation {
            count_code: "COUNT",
            name_code: "NAMES",
            bound_code: "BOUNDS",
            bound_ref_code: "BOUND_REFS",
        }
        .entries(symbol);

        assert_eq!(
            entries,
            [
                AbsentMetadataEntry::new(true, "COUNT", "type parameter count"),
                AbsentMetadataEntry::new(true, "NAMES", "type parameter names"),
                AbsentMetadataEntry::new(true, "BOUNDS", "type parameter bounds"),
                AbsentMetadataEntry::new(true, "BOUND_REFS", "typed type parameter bound refs"),
            ]
        );
    }

    #[test]
    fn type_parameter_absence_validation_uses_module_resolver_codes() {
        let validation = TypeParameterAbsenceValidation::module_resolver_codes();

        assert_eq!(validation.count_code, "E0269");
        assert_eq!(validation.name_code, "E0348");
        assert_eq!(validation.bound_code, "E0270");
        assert_eq!(validation.bound_ref_code, "E0373");
    }

    #[test]
    fn type_parameter_absence_validation_uses_import_resolver_codes() {
        let validation = TypeParameterAbsenceValidation::import_resolver_codes();

        assert_eq!(validation.count_code, "E0285");
        assert_eq!(validation.name_code, "E0349");
        assert_eq!(validation.bound_code, "E0286");
        assert_eq!(validation.bound_ref_code, "E0364");
    }

    #[test]
    fn type_parameter_absence_validation_uses_local_resolver_codes() {
        let validation = TypeParameterAbsenceValidation::local_resolver_codes();

        assert_eq!(validation.count_code, "E0253");
        assert_eq!(validation.name_code, "E0350");
        assert_eq!(validation.bound_code, "E0254");
        assert_eq!(validation.bound_ref_code, "E0382");
    }

    #[test]
    fn type_parameter_absence_validation_uses_variant_resolver_codes() {
        let validation = TypeParameterAbsenceValidation::variant_resolver_codes();

        assert_eq!(validation.count_code, "E0334");
        assert_eq!(validation.name_code, "E0351");
        assert_eq!(validation.bound_code, "E0335");
        assert_eq!(validation.bound_ref_code, "E0391");
    }

    #[test]
    fn field_absence_validation_builds_entries() {
        let program = parse_program(
            r#"
Point: { x: i32, y: i32 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup(Namespace::Type, "Point")
            .expect("type symbol");
        let entries = FieldAbsenceValidation {
            count_code: "COUNT",
            type_name_code: "FIELD_TYPES",
            typed_code: "TYPED_FIELDS",
        }
        .entries(symbol);

        assert_eq!(
            entries,
            [
                AbsentMetadataEntry::new(true, "COUNT", "field count"),
                AbsentMetadataEntry::new(true, "FIELD_TYPES", "field types"),
                AbsentMetadataEntry::new(true, "TYPED_FIELDS", "typed field types"),
            ]
        );
    }

    #[test]
    fn field_absence_validation_uses_module_resolver_codes() {
        let validation = FieldAbsenceValidation::module_resolver_codes();

        assert_eq!(validation.count_code, "E0271");
        assert_eq!(validation.type_name_code, "E0272");
        assert_eq!(validation.typed_code, "E0374");
    }

    #[test]
    fn field_absence_validation_uses_import_resolver_codes() {
        let validation = FieldAbsenceValidation::import_resolver_codes();

        assert_eq!(validation.count_code, "E0287");
        assert_eq!(validation.type_name_code, "E0288");
        assert_eq!(validation.typed_code, "E0365");
    }

    #[test]
    fn field_absence_validation_uses_local_resolver_codes() {
        let validation = FieldAbsenceValidation::local_resolver_codes();

        assert_eq!(validation.count_code, "E0255");
        assert_eq!(validation.type_name_code, "E0256");
        assert_eq!(validation.typed_code, "E0383");
    }

    #[test]
    fn field_absence_validation_uses_type_like_resolver_codes() {
        let validation = FieldAbsenceValidation::type_like_resolver_codes();

        assert_eq!(validation.count_code, "E0319");
        assert_eq!(validation.type_name_code, "E0320");
        assert_eq!(validation.typed_code, "E0398");
    }

    #[test]
    fn field_absence_validation_uses_variant_resolver_codes() {
        let validation = FieldAbsenceValidation::variant_resolver_codes();

        assert_eq!(validation.count_code, "E0336");
        assert_eq!(validation.type_name_code, "E0337");
        assert_eq!(validation.typed_code, "E0392");
    }

    #[test]
    fn field_absence_validation_uses_behavior_resolver_codes() {
        let validation = FieldAbsenceValidation::behavior_resolver_codes();

        assert_eq!(validation.count_code, "E0321");
        assert_eq!(validation.type_name_code, "E0322");
        assert_eq!(validation.typed_code, "E0399");
    }

    #[test]
    fn field_absence_validation_uses_value_resolver_codes() {
        let validation = FieldAbsenceValidation::value_resolver_codes();

        assert_eq!(validation.count_code, "E0298");
        assert_eq!(validation.type_name_code, "E0299");
        assert_eq!(validation.typed_code, "E0403");
    }

    #[test]
    fn variant_absence_validation_builds_entries() {
        let program = parse_program(
            r#"
Option<T>: Some(T), None
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup_variant("Option", "Some")
            .expect("variant symbol");
        let entries = VariantAbsenceValidation {
            names_code: "NAMES",
            owner_code: "OWNER",
            payload_count_code: "PAYLOAD_COUNT",
            payload_type_name_code: "PAYLOAD_TYPE",
            payload_type_code: "TYPED_PAYLOAD",
        }
        .entries(symbol);

        assert_eq!(
            entries,
            [
                AbsentMetadataEntry::new(false, "NAMES", "variant names"),
                AbsentMetadataEntry::new(true, "OWNER", "variant owner"),
                AbsentMetadataEntry::new(true, "PAYLOAD_COUNT", "variant payload count"),
                AbsentMetadataEntry::new(true, "PAYLOAD_TYPE", "variant payload type"),
                AbsentMetadataEntry::new(true, "TYPED_PAYLOAD", "typed variant payload type"),
            ]
        );
    }

    #[test]
    fn variant_absence_validation_uses_module_resolver_codes() {
        let validation = VariantAbsenceValidation::module_resolver_codes();

        assert_eq!(validation.names_code, "E0273");
        assert_eq!(validation.owner_code, "E0274");
        assert_eq!(validation.payload_count_code, "E0275");
        assert_eq!(validation.payload_type_name_code, "E0276");
        assert_eq!(validation.payload_type_code, "E0375");
    }

    #[test]
    fn variant_absence_validation_uses_import_resolver_codes() {
        let validation = VariantAbsenceValidation::import_resolver_codes();

        assert_eq!(validation.names_code, "E0289");
        assert_eq!(validation.owner_code, "E0290");
        assert_eq!(validation.payload_count_code, "E0291");
        assert_eq!(validation.payload_type_name_code, "E0292");
        assert_eq!(validation.payload_type_code, "E0366");
    }

    #[test]
    fn variant_absence_validation_uses_local_resolver_codes() {
        let validation = VariantAbsenceValidation::local_resolver_codes();

        assert_eq!(validation.names_code, "E0257");
        assert_eq!(validation.owner_code, "E0258");
        assert_eq!(validation.payload_count_code, "E0259");
        assert_eq!(validation.payload_type_name_code, "E0260");
        assert_eq!(validation.payload_type_code, "E0384");
    }

    #[test]
    fn variant_absence_validation_uses_type_like_resolver_codes() {
        let validation = VariantAbsenceValidation::type_like_resolver_codes();

        assert_eq!(validation.names_code, "E0315");
        assert_eq!(validation.owner_code, "E0316");
        assert_eq!(validation.payload_count_code, "E0317");
        assert_eq!(validation.payload_type_name_code, "E0318");
        assert_eq!(validation.payload_type_code, "E0397");
    }

    #[test]
    fn variant_absence_validation_uses_behavior_resolver_codes() {
        let validation = VariantAbsenceValidation::behavior_resolver_codes();

        assert_eq!(validation.names_code, "E0323");
        assert_eq!(validation.owner_code, "E0324");
        assert_eq!(validation.payload_count_code, "E0325");
        assert_eq!(validation.payload_type_name_code, "E0326");
        assert_eq!(validation.payload_type_code, "E0400");
    }

    #[test]
    fn variant_absence_validation_uses_value_resolver_codes() {
        let validation = VariantAbsenceValidation::value_resolver_codes();

        assert_eq!(validation.names_code, "E0300");
        assert_eq!(validation.owner_code, "E0301");
        assert_eq!(validation.payload_count_code, "E0302");
        assert_eq!(validation.payload_type_name_code, "E0303");
        assert_eq!(validation.payload_type_code, "E0404");
    }

    #[test]
    fn mutability_absence_validation_builds_entry() {
        let program = parse_program(
            r#"
main = (mut input: i32) i32 {
    value ::= input
    return value
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let symbol = symbols
            .lookup_scoped(Namespace::Local, "input")
            .expect("local symbol");
        let entry = MutabilityAbsenceValidation { code: "MUTABLE" }.entry(symbol);

        assert_eq!(
            entry,
            AbsentMetadataEntry::new(true, "MUTABLE", "mutability")
        );
    }

    #[test]
    fn mutability_absence_validation_uses_module_resolver_code() {
        let validation = MutabilityAbsenceValidation::module_resolver_code();

        assert_eq!(validation.code, "E0345");
    }

    #[test]
    fn mutability_absence_validation_uses_import_resolver_code() {
        let validation = MutabilityAbsenceValidation::import_resolver_code();

        assert_eq!(validation.code, "E0344");
    }

    #[test]
    fn mutability_absence_validation_uses_type_like_resolver_code() {
        let validation = MutabilityAbsenceValidation::type_like_resolver_code();

        assert_eq!(validation.code, "E0314");
    }

    #[test]
    fn mutability_absence_validation_uses_variant_resolver_code() {
        let validation = MutabilityAbsenceValidation::variant_resolver_code();

        assert_eq!(validation.code, "E0343");
    }

    #[test]
    fn mutability_absence_validation_uses_value_resolver_code() {
        let validation = MutabilityAbsenceValidation::value_resolver_code();

        assert_eq!(validation.code, "E0308");
    }

    #[test]
    fn mutability_validation_formats_actual_and_expected() {
        let validation = MutabilityValidation { code: "MUTABLE" };

        assert_eq!(validation.code, "MUTABLE");
        assert_eq!(
            validation.display(Some(false), true),
            ("immutable", "mutable")
        );
        assert_eq!(validation.display(None, false), ("unknown", "immutable"));
        assert_eq!(
            validation.message("local", "value", Some(false), true),
            "resolver local symbol 'value' has mutability immutable, expected mutable"
        );
    }

    #[test]
    fn mutability_validation_uses_resolver_code() {
        let validation = MutabilityValidation::resolver_code();

        assert_eq!(validation.code, "E0231");
    }

    #[test]
    fn visibility_validation_formats_actual_and_expected() {
        let validation = VisibilityValidation { code: "VISIBLE" };

        assert_eq!(validation.code, "VISIBLE");
        assert_eq!(validation.display(true, false), ("public", "private"));
        assert_eq!(validation.display(false, true), ("private", "public"));
        assert_eq!(
            validation.message("import", "io", true, false),
            "resolver import symbol 'io' has visibility public, expected private"
        );
    }

    #[test]
    fn visibility_validation_uses_local_resolver_code() {
        let validation = VisibilityValidation::local_resolver_code();

        assert_eq!(validation.code, "E0247");
    }

    #[test]
    fn visibility_validation_uses_module_resolver_code() {
        let validation = VisibilityValidation::module_resolver_code();

        assert_eq!(validation.code, "E0229");
    }

    #[test]
    fn visibility_validation_uses_import_resolver_code() {
        let validation = VisibilityValidation::import_resolver_code();

        assert_eq!(validation.code, "E0245");
    }

    #[test]
    fn visibility_validation_uses_type_like_resolver_code() {
        let validation = VisibilityValidation::type_like_resolver_code();

        assert_eq!(validation.code, "E0225");
    }

    #[test]
    fn visibility_validation_uses_variant_resolver_code() {
        let validation = VisibilityValidation::variant_resolver_code();

        assert_eq!(validation.code, "E0226");
    }

    #[test]
    fn visibility_validation_uses_value_resolver_code() {
        let validation = VisibilityValidation::value_resolver_code();

        assert_eq!(validation.code, "E0224");
    }

    #[test]
    fn resolver_symbol_presence_validation_formats_messages() {
        let extra = ResolverSymbolPresenceValidation {
            code: "EXTRA",
            presence: ResolverSymbolPresence::Extra,
        };
        let missing = ResolverSymbolPresenceValidation {
            code: "MISSING",
            presence: ResolverSymbolPresence::Missing,
        };

        assert_eq!(extra.code, "EXTRA");
        assert_eq!(
            extra.message("value", "main"),
            "resolver symbol table has extra value symbol 'main'"
        );
        assert_eq!(missing.code, "MISSING");
        assert_eq!(
            missing.message("local", "value"),
            "resolver symbol table missing local symbol 'value'"
        );
    }

    #[test]
    fn resolver_symbol_presence_validation_uses_resolver_codes() {
        let missing = ResolverSymbolPresenceValidation::missing_resolver_code();
        let missing_local = ResolverSymbolPresenceValidation::missing_local_resolver_code();
        let extra_declaration = ResolverSymbolPresenceValidation::extra_declaration_resolver_code();
        let extra_local = ResolverSymbolPresenceValidation::extra_local_resolver_code();

        assert_eq!(missing.code, "E0210");
        assert!(matches!(missing.presence, ResolverSymbolPresence::Missing));
        assert_eq!(missing_local.code, "E0228");
        assert!(matches!(
            missing_local.presence,
            ResolverSymbolPresence::Missing
        ));
        assert_eq!(extra_declaration.code, "E0243");
        assert!(matches!(
            extra_declaration.presence,
            ResolverSymbolPresence::Extra
        ));
        assert_eq!(extra_local.code, "E0244");
        assert!(matches!(
            extra_local.presence,
            ResolverSymbolPresence::Extra
        ));
    }

    #[test]
    fn resolver_symbol_presence_validation_pushes_diagnostic() {
        let mut tc = TypeChecker::new();

        tc.validate_resolver_symbol_presence(
            "value",
            "main",
            ResolverSymbolPresenceValidation {
                code: "EXTRA",
                presence: ResolverSymbolPresence::Extra,
            },
            Span::dummy(),
        );

        assert_eq!(tc.diagnostics.len(), 1);
        assert_eq!(tc.diagnostics[0].code, "EXTRA");
        assert_eq!(
            tc.diagnostics[0].message,
            "resolver symbol table has extra value symbol 'main'"
        );
    }

    #[test]
    fn source_absence_validation_builds_source_validation() {
        let validation = SourceAbsenceValidation { code: "SOURCE" }.source_validation();

        assert_eq!(validation.code, "SOURCE");
        assert_eq!(validation.actual_missing, "none");
        assert_eq!(validation.expected_missing, "none");
        assert!(!validation.quote_expected);
    }

    #[test]
    fn source_absence_validation_uses_type_like_resolver_code() {
        let validation = SourceAbsenceValidation::type_like_resolver_code();

        assert_eq!(validation.code, "E0309");
    }

    #[test]
    fn source_absence_validation_uses_variant_resolver_code() {
        let validation = SourceAbsenceValidation::variant_resolver_code();

        assert_eq!(validation.code, "E0329");
    }

    #[test]
    fn source_absence_validation_uses_value_resolver_code() {
        let validation = SourceAbsenceValidation::value_resolver_code();

        assert_eq!(validation.code, "E0297");
    }

    #[test]
    fn source_validation_formats_message() {
        let quoted = SourceValidation {
            code: "SOURCE",
            actual_missing: "unknown",
            expected_missing: "none",
            quote_expected: true,
        };
        let unquoted = SourceValidation {
            code: "SOURCE",
            actual_missing: "none",
            expected_missing: "none",
            quote_expected: false,
        };

        assert_eq!(
            quoted.message("import", "io", Some("other"), Some("std")),
            "resolver import symbol 'io' has source 'other', expected 'std'"
        );
        assert_eq!(
            unquoted.message("value", "main", Some("std"), None),
            "resolver value symbol 'main' has source 'std', expected none"
        );
    }

    #[test]
    fn source_validation_uses_resolver_codes() {
        let module = SourceValidation::module_resolver_code();
        let stripped_import = SourceValidation::stripped_import_resolver_code();
        let import = SourceValidation::import_resolver_code();
        let local = SourceValidation::local_resolver_code();

        assert_eq!(module.code, "E0230");
        assert_eq!(module.actual_missing, "none");
        assert_eq!(module.expected_missing, "none");
        assert!(!module.quote_expected);
        assert_eq!(stripped_import.code, "E0246");
        assert_eq!(stripped_import.actual_missing, "unknown");
        assert_eq!(stripped_import.expected_missing, "a module source");
        assert!(!stripped_import.quote_expected);
        assert_eq!(import.code, "E0227");
        assert_eq!(import.actual_missing, "unknown");
        assert_eq!(import.expected_missing, "none");
        assert!(import.quote_expected);
        assert_eq!(local.code, "E0248");
        assert_eq!(local.actual_missing, "none");
        assert_eq!(local.expected_missing, "none");
        assert!(!local.quote_expected);
    }

    #[test]
    fn absent_metadata_entry_formats_message() {
        let entry = AbsentMetadataEntry {
            present: true,
            code: "ABSENT",
            label: "parameter count",
        };

        assert_eq!(entry.code, "ABSENT");
        assert_eq!(
            entry.message("value", "main"),
            "resolver value symbol 'main' has parameter count metadata, expected none"
        );
    }

    #[test]
    fn resolver_named_list_display_formats_known_and_missing_items() {
        let fields = vec![("value".to_string(), "i32".to_string())];
        assert_eq!(
            format_resolver_named_list(Some(&fields), |ty: &String| ty.clone()),
            "(value: i32)"
        );
        assert_eq!(
            format_resolver_named_list::<String>(None, |ty: &String| ty.clone()),
            "unknown"
        );
    }

    #[test]
    fn check_program_rejects_self_type_outside_method_or_behavior() {
        let program = parse_program(
            r#"
main = (value: Self) i32 { return 0 }
"#,
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program(&program)
            .expect_err("Self should require a method or behavior context");

        assert!(
            err.iter()
                .any(|d| d.message.contains("Self type is only valid")),
            "expected invalid Self type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_rejects_unknown_type_references() {
        let program = parse_program(
            r#"
main = (value: Missing, items: Bag<i32>) i32 { return 0 }
"#,
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program(&program)
            .expect_err("unknown type reference should fail");

        assert!(
            err.iter()
                .any(|d| d.message.contains("unknown type symbol 'Missing'")),
            "expected unknown type diagnostic, got {err:?}"
        );
        assert!(
            err.iter()
                .any(|d| d.message.contains("unknown type symbol 'Bag'")),
            "expected unknown generic type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_rejects_unknown_type_references_in_struct_field_defaults() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T = {
        same: Missing = 1
        same
    }
}
"#,
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program(&program)
            .expect_err("unknown struct field default type reference should fail");

        assert!(
            err.iter()
                .any(|d| d.message.contains("unknown type symbol 'Missing'")),
            "expected unknown field default type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_rejects_struct_field_default_type_mismatch() {
        let program = parse_program(
            r#"
Point: { x: i32 = "bad" }
"#,
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program(&program)
            .expect_err("struct field default type mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("field `x` default expects `i32`, found `str`")),
            "expected field default type mismatch diagnostic, got {err:?}"
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
    fn collect_enum_info() {
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Enum {
            name: "OptionI32".into(),
            type_params: Vec::new(),
            variants: vec![
                EnumVariant {
                    name: "Some".into(),
                    payload: Some(AstType::I32),
                    span: Span::dummy(),
                },
                EnumVariant {
                    name: "None".into(),
                    payload: None,
                    span: Span::dummy(),
                },
            ],
            public: false,
            span: Span::dummy(),
        }];
        tc.collect_declarations(&decls);
        assert!(tc.enums.contains_key("OptionI32"));
        assert_eq!(tc.enums["OptionI32"].variants.len(), 2);
    }

    #[test]
    fn collect_import_info() {
        let program = parse_program(
            r#"
{ io, fmt } = std

main = () i32 {
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.collect_declarations(&program.declarations);
        assert_eq!(tc.imports.get("io"), Some(&vec!["std".to_string()]));
        assert_eq!(tc.imports.get("fmt"), Some(&vec!["std".to_string()]));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_function_type_metadata() {
        let mut program = parse_program(
            r#"
apply = (callback: (i32) i32) (i32) i32 {
    return callback
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            params[0].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.functions.get("apply").expect("function info");
        assert_eq!(
            info.params[0].1,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
        assert_eq!(
            info.return_type,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_function_signature() {
        let mut program = parse_program(
            r#"
main = (value: i32) i32 { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "main", None);
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should not keep AST-only function metadata when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_function_signature_after_name_restore() {
        let mut program = parse_program(
            r#"
main = (value: i32) i32 { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "main", None);
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST function signature key after resolver name restoration"
        );
        assert!(
            !tc.functions.contains_key("main"),
            "resolver-backed collection should clear the restored function signature key when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_function_template()
    {
        let mut program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "identity", None);
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should not keep AST-only generic function templates when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_validate_stale_generic_function_body_refs_when_signature_incomplete(
    ) {
        let mut program = parse_program(
            r#"
identity<T> = (value: T) T {
    same: T = value
    return same
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "identity", None);
        if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
            type_params[0].name = "Stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should remove generic template when resolver signature metadata is incomplete"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST generic body refs when resolver signature metadata is incomplete: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_function_signature_for_type_refs() {
        let mut program = parse_program(
            r#"
main = (value: i32) i32 { return value }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function {
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            params[0].ty = AstType::Named("Missing".to_string());
            *return_type = Some(AstType::Named("AlsoMissing".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_method_signature_for_type_refs() {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.get = (self: Box, value: i32) i32 { return value }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method {
            params,
            return_type,
            ..
        } = &mut program.declarations[1]
        {
            params[1].ty = AstType::Named("Missing".to_string());
            *return_type = Some(AstType::Named("AlsoMissing".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored method signature metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_method_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { return self.x }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method { method_name, .. } = &mut program.declarations[1] {
            *method_name = "missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.get"));
        assert!(!tc.methods.contains_key("Point.missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_method_target_and_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { return self.x }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method {
            type_name,
            method_name,
            ..
        } = &mut program.declarations[1]
        {
            *type_name = "Missing".to_string();
            *method_name = "missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.get"));
        assert!(!tc.methods.contains_key("Missing.missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_method_signature_after_key_restore() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Point.get = (self: Point) i32 { return self.x }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
        if let Declaration::Method {
            type_name,
            method_name,
            params,
            return_type,
            ..
        } = &mut program.declarations[1]
        {
            *type_name = "Missing".to_string();
            *method_name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST method signature key after resolver key restoration"
        );
        assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should clear the restored method signature key when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_function_name_metadata() {
        let mut program = parse_program(
            r#"
main = () i32 { return 1 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { name, .. } = &mut program.declarations[0] {
            *name = "missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.functions.contains_key("main"));
        assert!(!tc.functions.contains_key("missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_function_template_name_metadata() {
        let mut program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { name, .. } = &mut program.declarations[0] {
            *name = "missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.generic_functions.contains_key("identity"));
        assert!(!tc.generic_functions.contains_key("missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_generic_function_template_after_name_restore()
    {
        let mut program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "identity", None);
        if let Declaration::Function {
            name,
            params,
            return_type,
            ..
        } = &mut program.declarations[0]
        {
            *name = "missing".to_string();
            params[0].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_functions.contains_key("missing"),
            "resolver-backed collection should clear the stale AST generic function template key after resolver name restoration"
        );
        assert!(
            !tc.generic_functions.contains_key("identity"),
            "resolver-backed collection should clear the restored generic function template key when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_function_type_params_for_type_refs() {
        let mut program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { type_params, .. } = &mut program.declarations[0] {
            type_params[0].name = "Stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_metadata_for_type_refs() {
        let mut program = parse_program(
            r#"
Box<T>: { value: T }
Option<T>: Some(T), None
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
            type_params[0].name = "StaleBox".to_string();
        }
        if let Declaration::Enum { type_params, .. } = &mut program.declarations[1] {
            type_params[0].name = "StaleOption".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored type metadata should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_type_params_for_type_refs() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
            type_params[0].name = "Stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior type parameters should avoid stale AST type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_function_bounds_for_validation() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { return value }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { type_params, .. } = &mut program.declarations[1] {
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored function bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_bounds_for_validation() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
            type_params[0].constraint = Some("MissingBox".to_string());
            type_params[0].constraint_type_args.clear();
        }
        if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
            type_params[0].constraint = Some("MissingOption".to_string());
            type_params[0].constraint_type_args.clear();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored type bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_type_bounds() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Box<T: Json<T>>: { value: T }
Option<T: Json<T>>: Some(T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Box", None);
        symbols.set_type_parameter_bound_refs_for_test(Namespace::Type, "Option", None);
        if let Declaration::Struct { type_params, .. } = &mut program.declarations[1] {
            type_params[0].constraint = Some("MissingBox".to_string());
            type_params[0].constraint_type_args.clear();
        }
        if let Declaration::Enum { type_params, .. } = &mut program.declarations[2] {
            type_params[0].constraint = Some("MissingOption".to_string());
            type_params[0].constraint_type_args.clear();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.structs
                .get("Box")
                .expect("struct info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed struct collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
        assert!(
            tc.enums
                .get("Option")
                .expect("enum info")
                .type_param_bounds
                .is_empty(),
            "resolver-backed enum collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_bounds_for_validation() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_bounds() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bound_refs_for_test(Namespace::Behavior, "Serializable", None);
        if let Declaration::Behavior { type_params, .. } = &mut program.declarations[1] {
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Serializable").expect("behavior info");
        assert!(
            info.type_param_bounds.is_empty(),
            "resolver-backed behavior collection should not keep AST-only bounds when resolver bound metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_impl_method_bounds_for_validation() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Box: { value: i32 }
Box.impl = {
    keep<T: Json<T>> = (self: Box, value: T) T { return value }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function { type_params, .. } = &mut methods[0] {
                type_params[0].constraint = Some("Missing".to_string());
                type_params[0].constraint_type_args.clear();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method bounds should avoid stale AST generic-bound diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_method_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { return self.x }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function { name, .. } = &mut methods[0] {
                *name = "missing".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.get"));
        assert!(!tc.methods.contains_key("Point.missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_target_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { return self.x }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[1] {
            *type_name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.get"));
        assert!(!tc.methods.contains_key("Missing.get"));
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_impl_method_signature() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { return self.x }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Point.get", None);
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function {
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                params[0].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.methods.contains_key("Point.get"),
            "resolver-backed collection should not keep AST-only impl method metadata when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_name_metadata(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { return value }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params.pop();
                *return_type = None;
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.keep")
            .expect("generic impl method template");
        assert!(!tc.generic_methods.contains_key("Box.missing"));
        assert_eq!(template.params.len(), 2);
        assert_eq!(template.params[0].name, "self");
        assert_eq!(template.params[1].name, "value");
        assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
        assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_metadata()
    {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }

Box.impl = {
    apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
        return callback
    }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[3] {
            if let Declaration::Function {
                type_params,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                type_params[0].name = "Stale".to_string();
                type_params[0].constraint = Some("Debug".to_string());
                type_params[0].constraint_type_args.clear();
                params[1].ty = AstType::I32;
                *return_type = Some(AstType::I32);
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.apply")
            .expect("generic impl method template");
        assert_eq!(template.type_params, vec!["U".to_string()]);
        assert_eq!(
            tc.methods
                .get("Box.apply")
                .expect("impl method info")
                .type_param_bounds
                .get("U"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("U".to_string())],
            })
        );
        assert_eq!(
            template.params[1].ty,
            AstType::Function {
                params: vec![AstType::Named("U".to_string())],
                ret: Box::new(AstType::Named("U".to_string())),
            }
        );
        assert_eq!(
            template.return_type,
            Some(AstType::Function {
                params: vec![AstType::Named("U".to_string())],
                ret: Box::new(AstType::Named("U".to_string())),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_return_presence(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        return value
    }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function { return_type, .. } = &mut methods[0] {
                *return_type = None;
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.keep")
            .expect("generic impl method template");
        assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_parameter_count(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, right: T) T {
        return left
    }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function { params, .. } = &mut methods[0] {
                params.pop();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.choose")
            .expect("generic impl method template");
        assert_eq!(template.params.len(), 3);
        assert_eq!(template.params[0].name, "self");
        assert_eq!(template.params[1].name, "left");
        assert_eq!(template.params[2].name, "right");
        assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
        assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
        assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
    }

    #[test]
    fn collect_declarations_with_symbols_preserves_type_impl_generic_template_param_mutability_by_position(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, mut value: T) T {
        value = value
        return value
    }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function { params, .. } = &mut methods[0] {
                params[1].name = "stale".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.keep")
            .expect("generic impl method template");
        assert_eq!(template.params[1].name, "value");
        assert!(
            template.params[1].mutable,
            "resolver-restored impl method parameter name should preserve positional mutability"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_ignores_stale_type_impl_generic_template_param_names_for_mutability(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    choose<T> = (self: Box, left: T, mut right: T) T {
        right = right
        return right
    }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function { params, .. } = &mut methods[0] {
                params.swap(1, 2);
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.choose")
            .expect("generic impl method template");
        assert_eq!(template.params[1].name, "left");
        assert_eq!(template.params[2].name, "right");
        assert!(
            template.params[1].mutable,
            "resolver-restored first non-self impl parameter should keep first AST position mutability"
        );
        assert!(
            !template.params[2].mutable,
            "resolver-restored second non-self impl parameter should keep second AST position mutability"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_impl_method_template(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { return value }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function {
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                params[1].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should not keep AST-only generic impl method templates when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_generic_impl_method_template_after_key_restore(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { return value }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[1]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params[1].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST generic impl method template key after resolver key restoration"
        );
        assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should clear the restored generic impl method template key when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_template_target_and_name_metadata(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T { return value }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[1]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params.pop();
                *return_type = None;
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.keep")
            .expect("generic impl method template");
        assert!(!tc.generic_methods.contains_key("Missing.missing"));
        assert_eq!(template.params.len(), 2);
        assert_eq!(template.params[0].name, "self");
        assert_eq!(template.params[1].name, "value");
        assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
        assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_type_impl_generic_method_name_for_body_type_refs(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }

Box.impl = {
    keep<T> = (self: Box, value: T) T {
        same: T = value
        return same
    }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[1] {
            if let Declaration::Function {
                name, type_params, ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                type_params[0].name = "Stale".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic impl method name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_function_template_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
apply<T: Json<T>> = (callback: (T) T) (T) T {
    return callback
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function {
            type_params,
            params,
            return_type,
            ..
        } = &mut program.declarations[2]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[0].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc.generic_functions.get("apply").expect("generic template");
        assert_eq!(template.type_params, vec!["T".to_string()]);
        assert_eq!(
            tc.functions
                .get("apply")
                .expect("function info")
                .type_param_bounds
                .get("T"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            })
        );
        assert_eq!(
            template.params[0].ty,
            AstType::Function {
                params: vec![AstType::Named("T".to_string())],
                ret: Box::new(AstType::Named("T".to_string())),
            }
        );
        assert_eq!(
            template.return_type,
            Some(AstType::Function {
                params: vec![AstType::Named("T".to_string())],
                ret: Box::new(AstType::Named("T".to_string())),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_function_template_return_presence() {
        let mut program = parse_program(
            r#"
identity<T> = (value: T) T {
    return value
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { return_type, .. } = &mut program.declarations[0] {
            *return_type = None;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_functions
            .get("identity")
            .expect("generic template");
        assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_function_template_parameter_count() {
        let mut program = parse_program(
            r#"
choose<T> = (left: T, right: T) T {
    return left
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { params, .. } = &mut program.declarations[0] {
            params.pop();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_functions
            .get("choose")
            .expect("generic template");
        assert_eq!(template.params.len(), 2);
        assert_eq!(template.params[0].name, "left");
        assert_eq!(template.params[1].name, "right");
        assert_eq!(template.params[0].ty, AstType::Named("T".to_string()));
        assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
    }

    #[test]
    fn collect_declarations_with_symbols_preserves_generic_template_param_mutability_by_position() {
        let mut program = parse_program(
            r#"
keep<T> = (mut value: T) T {
    value = value
    return value
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { params, .. } = &mut program.declarations[0] {
            params[0].name = "stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc.generic_functions.get("keep").expect("generic template");
        assert_eq!(template.params[0].name, "value");
        assert!(
            template.params[0].mutable,
            "resolver-restored parameter name should preserve positional mutability"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_ignores_stale_generic_template_param_names_for_mutability()
    {
        let mut program = parse_program(
            r#"
choose<T> = (left: T, mut right: T) T {
    right = right
    return right
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Function { params, .. } = &mut program.declarations[0] {
            params.swap(0, 1);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_functions
            .get("choose")
            .expect("generic template");
        assert_eq!(template.params[0].name, "left");
        assert_eq!(template.params[1].name, "right");
        assert!(
            template.params[0].mutable,
            "resolver-restored first parameter should keep first AST position mutability"
        );
        assert!(
            !template.params[1].mutable,
            "resolver-restored second parameter should keep second AST position mutability"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_method_template_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Box: { value: i32 }
Box.apply<U: Json<U>> = (self: Box, callback: (U) U) (U) U {
    return callback
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method {
            type_params,
            params,
            return_type,
            ..
        } = &mut program.declarations[3]
        {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            params[1].ty = AstType::I32;
            *return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.apply")
            .expect("generic method template");
        assert_eq!(template.type_params, vec!["U".to_string()]);
        assert_eq!(
            tc.methods
                .get("Box.apply")
                .expect("method info")
                .type_param_bounds
                .get("U"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("U".to_string())],
            })
        );
        assert_eq!(
            template.params[1].ty,
            AstType::Function {
                params: vec![AstType::Named("U".to_string())],
                ret: Box::new(AstType::Named("U".to_string())),
            }
        );
        assert_eq!(
            template.return_type,
            Some(AstType::Function {
                params: vec![AstType::Named("U".to_string())],
                ret: Box::new(AstType::Named("U".to_string())),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_method_template_return_presence() {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T {
    return value
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method { return_type, .. } = &mut program.declarations[1] {
            *return_type = None;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.keep")
            .expect("generic method template");
        assert_eq!(template.return_type, Some(AstType::Named("T".to_string())));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_method_template_parameter_count() {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, right: T) T {
    return left
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method { params, .. } = &mut program.declarations[1] {
            params.pop();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.choose")
            .expect("generic method template");
        assert_eq!(template.params.len(), 3);
        assert_eq!(template.params[0].name, "self");
        assert_eq!(template.params[1].name, "left");
        assert_eq!(template.params[2].name, "right");
        assert_eq!(template.params[0].ty, AstType::Named("Box".to_string()));
        assert_eq!(template.params[1].ty, AstType::Named("T".to_string()));
        assert_eq!(template.params[2].ty, AstType::Named("T".to_string()));
    }

    #[test]
    fn collect_declarations_with_symbols_preserves_generic_method_template_param_mutability_by_position(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, mut value: T) T {
    value = value
    return value
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method { params, .. } = &mut program.declarations[1] {
            params[1].name = "stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.keep")
            .expect("generic method template");
        assert_eq!(template.params[1].name, "value");
        assert!(
            template.params[1].mutable,
            "resolver-restored method parameter name should preserve positional mutability"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_ignores_stale_generic_method_template_param_names_for_mutability(
    ) {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.choose<T> = (self: Box, left: T, mut right: T) T {
    right = right
    return right
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method { params, .. } = &mut program.declarations[1] {
            params.swap(1, 2);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Box.choose")
            .expect("generic method template");
        assert_eq!(template.params[1].name, "left");
        assert_eq!(template.params[2].name, "right");
        assert!(
            template.params[1].mutable,
            "resolver-restored first non-self method parameter should keep first AST position mutability"
        );
        assert!(
            !template.params[2].mutable,
            "resolver-restored second non-self method parameter should keep second AST position mutability"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_generic_method_template() {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
        if let Declaration::Method {
            params,
            return_type,
            ..
        } = &mut program.declarations[1]
        {
            params[1].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should not keep AST-only generic method templates when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_generic_method_template_after_key_restore() {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Box.keep", None);
        if let Declaration::Method {
            type_name,
            method_name,
            params,
            return_type,
            ..
        } = &mut program.declarations[1]
        {
            *type_name = "Missing".to_string();
            *method_name = "missing".to_string();
            params[1].ty = AstType::Named("Stale".to_string());
            *return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed collection should clear the stale AST generic method template key after resolver key restoration"
        );
        assert!(
            !tc.generic_methods.contains_key("Box.keep"),
            "resolver-backed collection should clear the restored generic method template key when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_generic_method_name_for_body_type_refs() {
        let mut program = parse_program(
            r#"
Box: { value: i32 }
Box.keep<T> = (self: Box, value: T) T {
    same: T = value
    return same
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Method {
            method_name,
            type_params,
            ..
        } = &mut program.declarations[1]
        {
            *method_name = "missing".to_string();
            type_params[0].name = "Stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored generic method name and type parameters should avoid stale AST body type-ref diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_struct_field_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Pipeline<T: Json<T>>: { callback: (i32) i32 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Struct {
            type_params,
            fields,
            ..
        } = &mut program.declarations[2]
        {
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            fields[0].ty = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.structs.get("Pipeline").expect("struct info");
        assert_eq!(
            info.type_param_bounds.get("T"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            })
        );
        assert_eq!(
            info.fields[0].1,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_struct_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Struct { name, .. } = &mut program.declarations[0] {
            *name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.structs.contains_key("Point"));
        assert!(!tc.structs.contains_key("Missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_struct_field_names_for_defaults() {
        let mut program = parse_program(
            r#"
Point: { x: i32 = true }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
            fields[0].name = "stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.iter().any(|diag| {
                diag.code == "E3073"
                    && diag
                        .message
                        .contains("field `x` default expects `i32`, found `bool`")
            }),
            "resolver-backed default validation should use resolver-restored field names: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_struct_fields() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_types_for_test(Namespace::Type, "Point", None);
        if let Declaration::Struct { fields, .. } = &mut program.declarations[0] {
            fields[0].ty = AstType::Named("Stale".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should not keep AST-only struct fields when resolver field metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_validate_stale_struct_field_default_refs_when_fields_incomplete(
    ) {
        let mut program = parse_program(
            r#"
Box<T>: {
    value: T = {
        same: T = 1
        same
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_types_for_test(Namespace::Type, "Box", None);
        if let Declaration::Struct { type_params, .. } = &mut program.declarations[0] {
            type_params[0].name = "Stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.structs.contains_key("Box"),
            "resolver-backed collection should remove struct fields when resolver field metadata is incomplete"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST struct field default refs when resolver field metadata is incomplete: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_struct_fields_after_name_restore() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_types_for_test(Namespace::Type, "Point", None);
        if let Declaration::Struct { name, fields, .. } = &mut program.declarations[0] {
            *name = "Missing".to_string();
            fields[0].ty = AstType::Named("Stale".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.structs.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST struct key after resolver name restoration"
        );
        assert!(
            !tc.structs.contains_key("Point"),
            "resolver-backed collection should clear the restored struct key when resolver field metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_enum_payload_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug: behavior {
    debug: (Self) str
}
Callback<T: Json<T>>: Wrap((i32) i32), None
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Enum {
            type_params,
            variants,
            ..
        } = &mut program.declarations[2]
        {
            type_params[0].constraint = Some("Debug".to_string());
            type_params[0].constraint_type_args.clear();
            variants[0].payload = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.enums.get("Callback").expect("enum info");
        assert_eq!(
            info.type_param_bounds.get("T"),
            Some(&BehaviorBound {
                behavior: "Json".to_string(),
                type_args: vec![AstType::Named("T".to_string())],
            })
        );
        assert_eq!(
            info.variants[0].1,
            Some(AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_enum_name_metadata() {
        let mut program = parse_program(
            r#"
Option<T>: Some(T), None
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Enum { name, .. } = &mut program.declarations[0] {
            *name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.enums.contains_key("Option"));
        assert!(!tc.enums.contains_key("Missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_enum_variants() {
        let mut program = parse_program(
            r#"
Option<T>: Some(T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
        if let Declaration::Enum { variants, .. } = &mut program.declarations[0] {
            variants[0].payload = Some(AstType::Named("Stale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should not keep AST-only enum variants when resolver variant metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_enum_variants_after_name_restore() {
        let mut program = parse_program(
            r#"
Option<T>: Some(T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
        if let Declaration::Enum { name, variants, .. } = &mut program.declarations[0] {
            *name = "Missing".to_string();
            variants[0].payload = Some(AstType::Named("Stale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.enums.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST enum key after resolver name restoration"
        );
        assert!(
            !tc.enums.contains_key("Option"),
            "resolver-backed collection should clear the restored enum key when resolver variant metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_metadata() {
        let mut program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
            methods[0].params[1].ty = AstType::I32;
            methods[0].return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Mapper").expect("behavior info");
        assert_eq!(
            info.methods[0].params[1].ty,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
        assert_eq!(
            info.methods[0].return_type,
            Some(AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            })
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_name_metadata() {
        let mut program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { name, .. } = &mut program.declarations[0] {
            *name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.behaviors.contains_key("Json"));
        assert!(!tc.behaviors.contains_key("Missing"));
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_methods() {
        let mut program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[0] {
            methods[0].params[1].ty = AstType::Named("Stale".to_string());
            methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should not keep AST-only behavior methods when resolver method metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_validate_stale_behavior_default_body_refs_when_methods_incomplete(
    ) {
        let mut program = parse_program(
            r#"
Mapper<T>: behavior {
    map: (Self, value: T) T {
        same: T = value
        return same
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
        if let Declaration::Behavior { type_params, .. } = &mut program.declarations[0] {
            type_params[0].name = "Stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should remove behavior methods when resolver method metadata is incomplete"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST behavior default body refs when resolver method metadata is incomplete: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_behavior_methods_after_name_restore() {
        let mut program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, i32) i32
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_types_for_test(Namespace::Behavior, "Mapper", None);
        if let Declaration::Behavior { name, methods, .. } = &mut program.declarations[0] {
            *name = "Missing".to_string();
            methods[0].params[1].ty = AstType::Named("Stale".to_string());
            methods[0].return_type = Some(AstType::Named("AlsoStale".to_string()));
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.behaviors.contains_key("Missing"),
            "resolver-backed collection should clear the stale AST behavior key after resolver name restoration"
        );
        assert!(
            !tc.behaviors.contains_key("Mapper"),
            "resolver-backed collection should clear the restored behavior key when resolver method metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].name = "missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Json").expect("behavior info");
        assert_eq!(info.methods[0].name, "encode");
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method name metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_return_presence_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].return_type = None;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Json").expect("behavior info");
        assert_eq!(info.methods[0].return_type, Some(AstType::Str));
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method return metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_count() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params.push(Param {
                name: "stale".to_string(),
                ty: AstType::I32,
                mutable: false,
                span: Span::dummy(),
            });
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Json").expect("behavior info");
        assert_eq!(info.methods[0].params.len(), 1);
        assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_missing_parameter_count() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (Self, i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params.pop();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Mapper").expect("behavior info");
        assert_eq!(info.methods[0].params.len(), 2);
        assert_eq!(info.methods[0].params[0].name, "__arg0");
        assert_eq!(info.methods[0].params[1].name, "__arg1");
        assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
        assert_eq!(info.methods[0].params[1].ty, AstType::I32);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method params should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_parameter_names() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params[0].name = "stale".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Json").expect("behavior info");
        assert_eq!(info.methods[0].params[0].name, "value");
        assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_ignores_stale_behavior_method_parameter_order() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params.swap(0, 1);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Mapper").expect("behavior info");
        assert_eq!(info.methods[0].params[0].name, "value");
        assert_eq!(info.methods[0].params[1].name, "input");
        assert_eq!(info.methods[0].params[0].ty, AstType::SelfType);
        assert_eq!(info.methods[0].params[1].ty, AstType::I32);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_method_count() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
    describe: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
    describe = (value: Point) str { return "desc" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods.pop();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.behaviors.get("Json").expect("behavior info");
        assert_eq!(info.methods.len(), 2);
        assert_eq!(info.methods[0].name, "encode");
        assert_eq!(info.methods[1].name, "describe");
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior methods should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32 { return callback }
}

Point.implements(Mapper) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params[1].ty = AstType::I32;
            methods[0].return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.methods.get("Point.map").expect("default method info");
        assert_eq!(
            info.params[1].1,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
        assert_eq!(
            info.return_type,
            AstType::Function {
                params: vec![AstType::I32],
                ret: Box::new(AstType::I32),
            }
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_default_method_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { return "default" }
}

Point.implements(Json) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].name = "missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc
            .methods
            .get("Point.encode")
            .expect("resolver-restored default method");
        assert_eq!(info.params[0].0, "self");
        assert_eq!(info.return_type, AstType::Str);
        assert!(
            !tc.methods.contains_key("Point.missing"),
            "stale AST-only behavior default method name should not be synthesized"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior default method name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_skips_default_when_resolver_restores_impl_method_name() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { return "default" }
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function { name, .. } = &mut methods[0] {
                *name = "missing".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let method = tc
            .methods
            .get("Point.encode")
            .expect("restored impl method");
        assert_eq!(
            method.params[0].0, "value",
            "resolver-restored explicit impl method should not be overwritten by the behavior default"
        );
        assert!(
            !tc.methods.contains_key("Point.missing"),
            "stale AST-only impl method key should be removed"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method name should suppress default insertion: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_impl_behavior_for_defaults() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { return "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[3] {
            *behavior = Some("Debug".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let method = tc
            .methods
            .get("Point.encode")
            .expect("resolver-restored behavior default");
        assert_eq!(method.params[0].0, "self");
        assert_eq!(method.return_type, AstType::Str);
        assert!(
            !tc.methods.contains_key("Point.describe"),
            "stale AST-only behavior default should not be synthesized"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl metadata should drive default synthesis: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_for_defaults() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { return "default" }
}

Point.implements(Json) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
            *type_name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.encode"));
        assert!(!tc.methods.contains_key("Missing.encode"));
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl target should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_for_defaults()
    {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { return "default" }
}
Debug: behavior {
    describe: (Self) str
}

Point.implements(Json) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            ..
        } = &mut program.declarations[3]
        {
            *type_name = "Missing".to_string();
            *behavior = Some("Debug".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.encode"));
        assert!(!tc.methods.contains_key("Missing.encode"));
        assert!(
            !tc.methods.contains_key("Point.describe"),
            "stale AST-only behavior default should not be synthesized"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl target and name should drive omitted default synthesis: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_defers_impl_checks_until_resolver_metadata_is_collected() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { return callback }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { methods, .. } = &mut program.declarations[1] {
            methods[0].params[1].ty = AstType::I32;
            methods[0].return_type = Some(AstType::I32);
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_impl_method_metadata_for_impl_checks() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (self: Self, callback: (i32) i32) (i32) i32
}

Point.implements(Mapper) {
    map = (self: Point, callback: (i32) i32) (i32) i32 { return callback }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function {
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                params[1].ty = AstType::I32;
                *return_type = Some(AstType::I32);
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_impl_method_name_metadata_for_impl_checks() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function { name, .. } = &mut methods[0] {
                *name = "missing".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method name metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_let_stale_ast_name_hide_extra_impl_method() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    extra = (value: Point) str { return "extra" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function { name, .. } = &mut methods[0] {
                *name = "encode".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages
                .iter()
                .any(|message| *message == "method `extra` is not declared by behavior `Json`"),
            "resolver-owned extra impl method should not be hidden by stale AST required name: {:?}",
            messages
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_impl_method_parameter_names_for_impl_checks()
    {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (value: Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function { params, .. } = &mut methods[0] {
                params[0].name = "stale".to_string();
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.methods.get("Point.encode").expect("impl method info");
        assert_eq!(info.params[0].0, "value");
        assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter names should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_ignores_stale_impl_method_parameter_order_for_impl_checks()
    {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Mapper: behavior {
    map: (value: Self, input: i32) str
}

Point.implements(Mapper) {
    map = (value: Point, input: i32) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function { params, .. } = &mut methods[0] {
                params.swap(0, 1);
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.methods.get("Point.map").expect("impl method info");
        assert_eq!(info.params[0].0, "value");
        assert_eq!(info.params[1].0, "input");
        assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
        assert_eq!(info.params[1].1, AstType::I32);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl method parameter order should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_name_metadata() {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { type_name, .. } = &mut program.declarations[2] {
            *type_name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(tc.methods.contains_key("Point.encode"));
        assert!(!tc.methods.contains_key("Missing.encode"));
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl target should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn resolver_declaration_metadata_skips_behavior_impl_methods_until_behavior_impl_pass() {
        let program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.impl = {
    get = (self: Point) i32 { return self.x }
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        tc.with_resolver_backed_collection(|checker| {
            checker.collect_declarations(&program.declarations);
        });
        tc.collect_resolver_declaration_metadata(&program.declarations, &symbols);

        assert!(
            tc.methods.contains_key("Point.get"),
            "non-behavior impl methods should still be refreshed by declaration metadata"
        );
        assert!(
            !tc.methods.contains_key("Point.encode"),
            "behavior impl method signatures should be owned by the behavior impl metadata pass"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_method_signature(
    ) {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
        if let Declaration::ImplBlock { methods, .. } = &mut program.declarations[2] {
            if let Declaration::Function {
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                params[0].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should not keep AST-only method metadata when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_behavior_impl_method_signature_after_key_restore(
    ) {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params[0].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.methods.contains_key("Missing.missing"),
            "resolver-backed behavior impl collection should not keep stale AST method keys"
        );
        assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored method keys when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_method_signature_target_and_name_metadata(
    ) {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params[0].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let info = tc.methods.get("Point.encode").expect("impl method info");
        assert!(!tc.methods.contains_key("Missing.missing"));
        assert_eq!(info.params[0].0, "value");
        assert_eq!(info.params[0].1, AstType::Named("Point".to_string()));
        assert_eq!(info.return_type, AstType::Str);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl method signature should avoid stale AST diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_generic_method_template_target_and_name_metadata(
    ) {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params.pop();
                *return_type = None;
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let template = tc
            .generic_methods
            .get("Point.encode")
            .expect("generic behavior impl method template");
        assert!(!tc.generic_methods.contains_key("Missing.missing"));
        assert!(!tc.generic_methods.contains_key("Point.missing"));
        assert_eq!(template.type_params, vec!["T".to_string()]);
        assert_eq!(template.params.len(), 1);
        assert_eq!(template.params[0].name, "value");
        assert_eq!(template.params[0].ty, AstType::Named("Point".to_string()));
        assert_eq!(template.return_type, Some(AstType::Str));
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior impl generic template should avoid stale AST diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_clears_stale_behavior_impl_generic_method_template_after_key_restore(
    ) {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode<T> = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_for_test(Namespace::Value, "Point.encode", None);
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            if let Declaration::Function {
                name,
                params,
                return_type,
                ..
            } = &mut methods[0]
            {
                *name = "missing".to_string();
                params[0].ty = AstType::Named("Stale".to_string());
                *return_type = Some(AstType::Named("AlsoStale".to_string()));
            }
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.generic_methods.contains_key("Missing.missing"),
            "resolver-backed behavior impl collection should clear stale AST generic method templates"
        );
        assert!(
            !tc.generic_methods.contains_key("Point.encode"),
            "resolver-backed behavior impl collection should clear restored generic method templates when resolver signature metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_parent_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::BehaviorExtends {
            parent,
            parent_type_args,
            ..
        } = &mut program.declarations[2]
        {
            *parent = "Missing".to_string();
            parent_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let parents = tc
            .behavior_extends
            .get("PrettyJson")
            .expect("behavior parents");
        assert_eq!(parents[0].behavior, "Json");
        assert_eq!(parents[0].type_args, vec![AstType::Str]);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_parent_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_refs_for_test(Namespace::Behavior, "PrettyJson", None);
        if let Declaration::BehaviorExtends {
            parent_type_args, ..
        } = &mut program.declarations[2]
        {
            parent_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.behavior_extends.contains_key("PrettyJson"),
            "resolver-backed collection should not keep AST-only behavior parent refs when resolver parent metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_parent_type_args() {
        let mut program = parse_program(
            r#"
Marker<T>: behavior {
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Marker<str>)
PrettyJson.extends(Marker<i32>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let second_parent = program
            .declarations
            .iter_mut()
            .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
            .nth(1)
            .expect("second parent declaration");
        if let Declaration::BehaviorExtends {
            parent_type_args, ..
        } = second_parent
        {
            parent_type_args[0] = AstType::Str;
        } else {
            panic!("expected behavior extends declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let parents = tc
            .behavior_extends
            .get("PrettyJson")
            .expect("behavior parents");
        let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
        assert_eq!(parent_keys, vec!["Marker_str", "Marker_i32"]);
        assert!(
            tc.diagnostics.iter().all(|diagnostic| !diagnostic
                .message
                .contains("duplicate behavior inheritance")),
            "resolver-restored parent type args should avoid false duplicate diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_parent_and_type_param_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Behavior { type_params, .. } = &mut program.declarations[2] {
            type_params[0].name = "Stale".to_string();
            type_params[0].constraint = Some("Missing".to_string());
            type_params[0].constraint_type_args.clear();
        }
        if let Declaration::BehaviorExtends {
            parent,
            parent_type_args,
            ..
        } = &mut program.declarations[3]
        {
            *parent = "Missing".to_string();
            parent_type_args[0] = AstType::Named("Stale".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let parents = tc.behavior_extends.get("Pretty").expect("behavior parents");
        assert_eq!(parents[0].behavior, "Serializable");
        assert_eq!(parents[0].type_args, vec![AstType::Named("T".to_string())]);
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored behavior parent and type-parameter metadata should avoid stale AST extends diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_reports_resolver_restored_behavior_parent_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::BehaviorExtends {
            parent,
            parent_type_args,
            ..
        } = &mut program.declarations[3]
        {
            *parent = "Missing".to_string();
            parent_type_args[0] = AstType::I32;
        } else {
            panic!("expected behavior extends declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `PrettyJson` is missing required method `encode`"
            }),
            "resolver-restored parent metadata should report the inherited missing method, got {:?}",
            messages
        );
        assert!(
            messages.iter().all(|message| !message.contains("Missing")),
            "stale AST-only behavior parent names should not leak into diagnostics: {:?}",
            messages
        );
    }

    #[test]
    fn collect_declarations_with_symbols_reports_conflict_from_restored_parent_type_args() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Debug<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Debug<i32>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let second_parent = program
            .declarations
            .iter_mut()
            .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
            .nth(1)
            .expect("second parent declaration");
        if let Declaration::BehaviorExtends {
            parent_type_args, ..
        } = second_parent
        {
            parent_type_args[0] = AstType::Str;
        } else {
            panic!("expected behavior extends declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages.iter().any(|message| {
                *message == "conflicting behavior method `encode` inherited by `PrettyJson`"
            }),
            "resolver-restored parent type args should drive inherited method coherence diagnostics, got {:?}",
            messages
        );
        let parents = tc
            .behavior_extends
            .get("PrettyJson")
            .expect("behavior parents");
        let parent_keys: Vec<_> = parents.iter().map(|parent| parent.key.as_str()).collect();
        assert_eq!(parent_keys, vec!["Json_str", "Debug_i32"]);
    }

    #[test]
    fn collect_declarations_with_symbols_reports_cycle_from_restored_parent_refs() {
        let mut program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
PrettyJson: behavior {
    pretty: (Self) str
}
Debug: behavior {
    debug: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let second_parent = program
            .declarations
            .iter_mut()
            .filter(|declaration| matches!(declaration, Declaration::BehaviorExtends { .. }))
            .nth(1)
            .expect("second parent declaration");
        if let Declaration::BehaviorExtends { parent, .. } = second_parent {
            *parent = "Debug".to_string();
        } else {
            panic!("expected behavior extends declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages
                .iter()
                .any(|message| message.contains("behavior inheritance cycle")),
            "resolver-restored parent refs should drive cycle diagnostics, got {:?}",
            messages
        );
        let parents = tc
            .behavior_extends
            .get("PrettyJson")
            .expect("behavior parents");
        assert_eq!(parents[0].behavior, "Json");
    }

    #[test]
    fn collect_declarations_with_symbols_synthesizes_defaults_from_restored_behavior_parent() {
        let mut program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str { return "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::BehaviorExtends { parent, .. } = &mut program.declarations[3] {
            *parent = "Missing".to_string();
        } else {
            panic!("expected behavior extends declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.methods.contains_key("Point.encode"),
            "resolver-restored parent metadata should synthesize inherited default method"
        );
        assert!(
            !tc.methods.contains_key("Point.Missing"),
            "stale AST-only parent names should not synthesize default methods"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_synthesizes_generic_defaults_from_restored_parent_args() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T { return "json" }
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::BehaviorExtends {
            parent,
            parent_type_args,
            ..
        } = &mut program.declarations[3]
        {
            *parent = "Missing".to_string();
            parent_type_args[0] = AstType::I32;
        } else {
            panic!("expected behavior extends declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let encode = tc
            .methods
            .get("Point.encode")
            .expect("resolver-restored parent should synthesize inherited default");
        assert_eq!(
            encode.return_type,
            AstType::Str,
            "resolver-restored parent type args should drive inherited default return type"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            behavior_type_args, ..
        } = &mut program.declarations[2]
        {
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.behavior_impls
                .contains(&("Point".to_string(), "Json_str".to_string())),
            "resolver metadata should restore the validated Json<str> impl"
        );
        assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
            "AST-only Json<i32> impl drift should not remain after resolver collection"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_impl_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
        if let Declaration::ImplBlock {
            behavior_type_args, ..
        } = &mut program.declarations[2]
        {
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json_i32".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_synthesize_stale_impl_defaults_after_target_restore(
    ) {
        let mut program = parse_program(
            r#"
Point: { x: i32 }
Json: behavior {
    encode: (self: Self) str { return "default" }
}

Point.implements(Json) {
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_refs_for_test(Namespace::Type, "Point", None);
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            *behavior = Some("AlsoMissing".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            !tc.behavior_impls
                .contains(&("Point".to_string(), "Json".to_string())),
            "resolver-backed collection should not keep AST-only behavior impl refs when resolver impl metadata is incomplete"
        );
        assert!(
            !tc.methods.contains_key("Missing.encode"),
            "resolver-backed default synthesis should not keep stale AST target method keys"
        );
        assert!(
            !tc.methods.contains_key("Point.encode"),
            "resolver-backed default synthesis should not synthesize behavior defaults when resolver impl metadata is incomplete"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only impl refs after target restoration when resolver impl metadata is incomplete: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_name_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock { behavior, .. } = &mut program.declarations[2] {
            *behavior = Some("Missing".to_string());
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl name metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_impl_target_and_name_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            behavior_type_args,
            ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            *behavior = Some("AlsoMissing".to_string());
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.behavior_impls
                .contains(&("Point".to_string(), "Json_str".to_string())),
            "resolver metadata should restore the validated Point implements Json<str> association"
        );
        assert!(
            !tc.behavior_impls
                .contains(&("Missing".to_string(), "AlsoMissing_i32".to_string())),
            "stale AST-only impl target and behavior metadata should not remain after resolver collection"
        );
        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored impl target and behavior metadata should avoid stale AST impl diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_reports_resolver_restored_impl_target_and_name() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            behavior_type_args,
            ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            *behavior = Some("AlsoMissing".to_string());
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages.iter().any(|message| {
                *message == "type `Point` implementation of `Json_str` is missing required method `encode`"
            }),
            "resolver-restored impl metadata should report the validated missing method, got {:?}",
            messages
        );
        assert!(
            messages
                .iter()
                .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
            "stale AST-only impl names should not leak into diagnostics: {:?}",
            messages
        );
    }

    #[test]
    fn collect_declarations_with_symbols_reports_overlap_from_restored_impl_type_args() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let first_impl = program
            .declarations
            .iter_mut()
            .find(|declaration| {
                matches!(
                    declaration,
                    Declaration::ImplBlock {
                        behavior: Some(behavior),
                        ..
                    } if behavior == "Json"
                )
            })
            .expect("Json impl declaration");
        if let Declaration::ImplBlock {
            behavior_type_args, ..
        } = first_impl
        {
            behavior_type_args[0] = AstType::I32;
        } else {
            panic!("expected Json impl declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages.iter().any(|message| {
                *message
                    == "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
            }),
            "resolver-restored impl type args should drive overlap diagnostics, got {:?}",
            messages
        );
        assert!(
            messages.iter().all(|message| !message.contains("Json_i32")),
            "stale AST-only impl type args should not leak into overlap diagnostics: {:?}",
            messages
        );
    }

    #[test]
    fn collect_declarations_with_symbols_avoids_false_duplicate_from_restored_impl_type_args() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T { return "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let second_impl = program
            .declarations
            .iter_mut()
            .filter(|declaration| {
                matches!(
                    declaration,
                    Declaration::ImplBlock {
                        behavior: Some(behavior),
                        ..
                    } if behavior == "Json"
                )
            })
            .nth(1)
            .expect("second Json impl declaration");
        if let Declaration::ImplBlock {
            behavior_type_args, ..
        } = second_impl
        {
            behavior_type_args[0] = AstType::Str;
        } else {
            panic!("expected second Json impl declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages
                .iter()
                .all(|message| !message.contains("duplicate implementation")),
            "resolver-restored impl type args should avoid false duplicate diagnostics, got {:?}",
            messages
        );
        assert!(
            tc.behavior_impls
                .contains(&("Point".to_string(), "Json_str".to_string()))
                && tc
                    .behavior_impls
                    .contains(&("Point".to_string(), "Json_i32".to_string())),
            "resolver-restored impl type args should keep distinct impl specializations: {:?}",
            tc.behavior_impls
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_required_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Requires {
            behavior_type_args, ..
        } = &mut program.declarations[3]
        {
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Requires { type_name, .. } = &mut program.declarations[3] {
            *type_name = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_required_target_and_name_metadata()
    {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Requires {
            type_name,
            behavior,
            behavior_type_args,
            ..
        } = &mut program.declarations[3]
        {
            *type_name = "Missing".to_string();
            *behavior = "AlsoMissing".to_string();
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires target and behavior metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_reports_resolver_restored_required_target_and_name() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Requires {
            type_name,
            behavior,
            behavior_type_args,
            ..
        } = &mut program.declarations[2]
        {
            *type_name = "Missing".to_string();
            *behavior = "AlsoMissing".to_string();
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        let messages: Vec<_> = tc
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert!(
            messages
                .iter()
                .any(|message| *message == "type `Point` does not implement required behavior `Json_str`"),
            "resolver-restored requires metadata should report the validated missing impl, got {:?}",
            messages
        );
        assert!(
            messages
                .iter()
                .all(|message| !message.contains("Missing") && !message.contains("AlsoMissing")),
            "stale AST-only requires names should not leak into diagnostics: {:?}",
            messages
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_restored_requires_ref_for_inherited_impl() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
PrettyJson: behavior {
    pretty: (Self) str
}
Point: { x: i32 }

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let requires = program
            .declarations
            .iter_mut()
            .find(|declaration| matches!(declaration, Declaration::Requires { .. }))
            .expect("requires declaration");
        if let Declaration::Requires {
            behavior,
            behavior_type_args,
            ..
        } = requires
        {
            *behavior = "Missing".to_string();
            behavior_type_args[0] = AstType::I32;
        } else {
            panic!("expected requires declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires ref should be satisfied by inherited child impl: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_distinct_restored_requires_type_args() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T { return "default" }
}
Point: { x: i32 }

Point.implements(Json<str>) {
}

Point.implements(Json<i32>) {
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let second_requires = program
            .declarations
            .iter_mut()
            .filter(|declaration| matches!(declaration, Declaration::Requires { .. }))
            .nth(1)
            .expect("second requires declaration");
        if let Declaration::Requires {
            behavior_type_args, ..
        } = second_requires
        {
            behavior_type_args[0] = AstType::Str;
        } else {
            panic!("expected requires declaration");
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.iter().all(|diagnostic| !diagnostic
                .message
                .contains("does not implement required behavior")),
            "resolver-restored requires type args should keep distinct satisfied specializations: {:?}",
            tc.diagnostics
        );
        assert!(
            tc.behavior_impls
                .contains(&("Point".to_string(), "Json_str".to_string()))
                && tc
                    .behavior_impls
                    .contains(&("Point".to_string(), "Json_i32".to_string())),
            "resolver-restored impl refs should keep both required specializations available: {:?}",
            tc.behavior_impls
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_fallback_to_stale_ast_behavior_required_metadata()
    {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
        if let Declaration::Requires {
            behavior_type_args, ..
        } = &mut program.declarations[3]
        {
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_does_not_validate_stale_requires_after_target_restore() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_refs_for_test(Namespace::Type, "Point", None);
        if let Declaration::Requires {
            type_name,
            behavior,
            behavior_type_args,
            ..
        } = &mut program.declarations[3]
        {
            *type_name = "Missing".to_string();
            *behavior = "AlsoMissing".to_string();
            behavior_type_args[0] = AstType::I32;
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-backed collection should not validate stale AST-only requires refs after target restoration when resolver required metadata is incomplete: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn collect_declarations_with_symbols_uses_resolver_behavior_required_name_metadata() {
        let mut program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        if let Declaration::Requires { behavior, .. } = &mut program.declarations[3] {
            *behavior = "Missing".to_string();
        }
        let mut tc = TypeChecker::new();

        tc.collect_declarations_with_symbols(&program.declarations, &symbols);

        assert!(
            tc.diagnostics.is_empty(),
            "resolver-restored requires name metadata should avoid stale AST requires diagnostics: {:?}",
            tc.diagnostics
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_declarations() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let empty_symbols = SymbolTable::default();
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &empty_symbols)
            .expect_err("missing resolver symbols should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing value symbol 'main'")),
            "expected missing resolver symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_declarations() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let symbols_program = parse_program(
            r#"
main = () i32 { return 0 }
extra = () i32 { return 1 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver declarations should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra value symbol 'extra'")),
            "expected extra resolver symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_imports_when_ast_imports_are_present() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let symbols_program = parse_program(
            r#"
{ io, math } = std
main = () i32 { return 0 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver imports should fail when AST imports are present");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra import symbol 'math'")),
            "expected extra resolver import diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_modules_when_ast_imports_are_present() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let symbols_program = parse_program(
            r#"
{ io } = std
{ helper } = other
main = () i32 { return 0 }
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver modules should fail when AST imports are present");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra module symbol 'other'")),
            "expected extra resolver module diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_method_receiver_type() {
        let program = parse_program(
            r#"
Point: { x: i32 }
Point.label = () str { return "point" }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Type, "Point");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing receiver type resolver symbol should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing type symbol 'Point'")),
            "expected missing method receiver type symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_method_signature() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    return self.value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "Box.get",
            Some(vec!["Box<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.get' has parameter types '(Box<i32>)', expected '(Box<T>)'"
            )),
            "expected resolver method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_method_function_type_signature() {
        let program = parse_program(
            r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    return callback
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "Box.map",
            Some(vec!["Box<T>".to_string(), "T".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver method function type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.map' has parameter types '(Box<T>, T)', expected '(Box<T>, (T) T)'"
            )),
            "expected resolver method function type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_uses_resolver_import_bindings() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 {
    io.println("ok")
    return 0
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));

        let mut tc = TypeChecker::new();
        tc.check_program_with_symbols(&program, &symbols)
            .expect("resolver import symbols should seed typechecker imports");

        assert!(tc.is_root_std_import("io"));
    }

    #[test]
    fn check_program_with_symbols_validates_stripped_resolver_import_sources() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Import, "io", None);
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("stripped resolver imports without sources should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver import symbol 'io' has source 'unknown', expected a module source"
            )),
            "expected stripped resolver import source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_stripped_resolver_import_visibility() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Import, "io", true);
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("stripped resolver import visibility should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has visibility public, expected private")),
            "expected stripped resolver import visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_stripped_resolver_import_modules() {
        let mut program = parse_program(
            r#"
{ io } = std
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Module, "std");
        program
            .declarations
            .retain(|decl| !matches!(decl, Declaration::Import { .. }));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("stripped resolver imports should require source module symbols");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing module symbol 'std'")),
            "expected stripped resolver import module diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_module_graph_entry_uses_graph_import_bindings() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let math_path = tmp.path().join("math.zen");
        std::fs::write(
            &math_path,
            "pub add = (a: i32, b: i32) i32 { return a + b }\n",
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            "{ add } = math\n\nmain = () i32 { return add(1, 2) }\n",
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");
        let entry = graph.module(graph.entry).expect("entry module");
        assert!(
            !entry
                .program
                .declarations
                .iter()
                .any(|decl| decl.name() == Some("add")),
            "graph entry should not merge imported declarations"
        );

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("graph import bindings should seed imported signatures");

        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "main"));
        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "add"));
    }

    #[test]
    fn check_module_graph_entry_seeds_imported_function_type_signatures() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let callbacks_path = tmp.path().join("callbacks.zen");
        std::fs::write(
            &callbacks_path,
            "pub apply = (callback: (i32) i32, value: i32) i32 { return value }\n",
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ apply } = callbacks

main = () i32 {
    callback = (value: i32) i32 { return value }
    return apply(callback, 1)
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        tc.check_module_graph_entry(&graph)
            .expect("graph import bindings should seed function-typed signatures");
    }

    #[test]
    fn check_module_graph_entry_specializes_imported_generic_functions() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let identity_path = tmp.path().join("identity.zen");
        std::fs::write(
            &identity_path,
            "pub id<T> = (value: T) T { return value }\n",
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            "{ id } = identity\n\nmain = () i32 { return id<i32>(1) }\n",
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("graph import bindings should seed generic templates");

        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "id_i32"));
    }

    #[test]
    fn check_module_graph_entry_specializes_imported_generic_enums() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let option_path = tmp.path().join("option.zen");
        std::fs::write(
            &option_path,
            r#"pub Option<T>:
    None,
    Some(T)

pub Result<T, E>:
    Ok(T),
    Err(E)
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Option, Result } = option

main = () i32 {
    maybe = Option<i32>.Some(7)
    result = Result<i32, str>.Ok(9)
    return 0
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("graph import bindings should seed generic enum templates");

        assert!(typed.types.iter().any(|ty| ty.name == "Option_i32"));
        assert!(typed.types.iter().any(|ty| ty.name == "Result_i32_str"));
    }

    #[test]
    fn check_module_graph_entry_seeds_public_methods_for_imported_types() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let geometry_path = tmp.path().join("geometry.zen");
        std::fs::write(
            &geometry_path,
            r#"pub Point: { x: i32 }

pub Point.value = (self: Point) i32 {
    return self.x
}
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    return point.value()
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        tc.check_module_graph_entry(&graph)
            .expect("imported public type should seed its public methods");
    }

    #[test]
    fn check_module_graph_entry_does_not_seed_private_methods_for_imported_types() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let geometry_path = tmp.path().join("geometry.zen");
        std::fs::write(
            &geometry_path,
            r#"pub Point: { x: i32 }

Point.value = (self: Point) i32 {
    return self.x
}
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    return point.value()
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let err = TypeChecker::new()
            .check_module_graph_entry(&graph)
            .expect_err("private imported methods should not be seeded");

        assert!(
            err.iter()
                .any(|d| d.message.contains("type `Point` has no method `value`")),
            "expected private imported method diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_module_graph_entry_specializes_public_generic_methods_for_imported_types() {
        let tmp = tempfile::tempdir().expect("temp dir");
        let geometry_path = tmp.path().join("geometry.zen");
        std::fs::write(
            &geometry_path,
            r#"pub Point: { x: i32 }

pub Point.keep<T> = (self: Point, value: T) T {
    return value
}
"#,
        )
        .expect("write imported module");

        let main_path = tmp.path().join("main.zen");
        std::fs::write(
            &main_path,
            r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    return point.keep<i32>(1)
}
"#,
        )
        .expect("write entry module");

        let mut files = crate::error::FileTable::new();
        let mut modules = crate::module_system::ModuleSystem::new();
        let graph = modules
            .load_module_graph(&main_path, &mut files)
            .expect("module graph");

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_module_graph_entry(&graph)
            .expect("imported public type should seed public generic method templates");

        assert!(typed
            .functions
            .iter()
            .any(|function| function.name == "Point.keep_i32"));
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_sources() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Import, "io", Some("other".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import source mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has source 'other', expected 'std'")),
            "expected resolver import source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_visibility() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Import, "io", true);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has visibility public, expected private")),
            "expected resolver import visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_absent_declaration_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Import, "io", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import declaration metadata should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver import symbol 'io' has parameter count metadata, expected none"
            )),
            "expected resolver import parameter metadata diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver import symbol 'io' has return type metadata, expected none")),
            "expected resolver import return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_absent_type_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(Namespace::Import, "io", Some(vec!["x".to_string()]));
        symbols.set_parameter_type_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Import, "io", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Import, "io", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Import,
            "io",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Import,
            "io",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Import,
            "io",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(Namespace::Import, "io", Some(vec!["Some".to_string()]));
        symbols.set_variant_owner_name_for_test(
            Namespace::Import,
            "io",
            Some("Option".to_string()),
        );
        symbols.set_variant_payload_count_for_test(Namespace::Import, "io", Some(1));
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Import,
            "io",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Import, "io", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Import,
            "io",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_names: vec!["self".to_string()],
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Import,
            "io",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Import,
            "io",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import type metadata should fail");

        for expected in [
            "resolver import symbol 'io' has parameter names metadata, expected none",
            "resolver import symbol 'io' has parameter types metadata, expected none",
            "resolver import symbol 'io' has typed parameter types metadata, expected none",
            "resolver import symbol 'io' has typed return type metadata, expected none",
            "resolver import symbol 'io' has type parameter count metadata, expected none",
            "resolver import symbol 'io' has type parameter names metadata, expected none",
            "resolver import symbol 'io' has type parameter bounds metadata, expected none",
            "resolver import symbol 'io' has typed type parameter bound refs metadata, expected none",
            "resolver import symbol 'io' has field count metadata, expected none",
            "resolver import symbol 'io' has field types metadata, expected none",
            "resolver import symbol 'io' has typed field types metadata, expected none",
            "resolver import symbol 'io' has variant names metadata, expected none",
            "resolver import symbol 'io' has variant owner metadata, expected none",
            "resolver import symbol 'io' has variant payload count metadata, expected none",
            "resolver import symbol 'io' has variant payload type metadata, expected none",
            "resolver import symbol 'io' has typed variant payload type metadata, expected none",
            "resolver import symbol 'io' has behavior methods metadata, expected none",
            "resolver import symbol 'io' has typed behavior methods metadata, expected none",
            "resolver import symbol 'io' has behavior parents metadata, expected none",
            "resolver import symbol 'io' has typed behavior parents metadata, expected none",
            "resolver import symbol 'io' has behavior impls metadata, expected none",
            "resolver import symbol 'io' has typed behavior impls metadata, expected none",
            "resolver import symbol 'io' has behavior requires metadata, expected none",
            "resolver import symbol 'io' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver import metadata diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_import_and_module_absent_mutability() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_mutability_for_test(Namespace::Import, "io", Some(true));
        symbols.set_mutability_for_test(Namespace::Module, "std", Some(false));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver import/module mutability metadata should fail");

        for expected in [
            "resolver import symbol 'io' has mutability metadata, expected none",
            "resolver module symbol 'std' has mutability metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver import/module mutability diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_module_symbols() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Module, "std", true);
        symbols.set_import_source_for_test(Namespace::Module, "std", Some("other".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver module metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver module symbol 'std' has visibility public, expected private")),
            "expected resolver module visibility diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver module symbol 'std' has source 'other', expected none")),
            "expected resolver module source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Module, "std", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver module declaration metadata should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver module symbol 'std' has parameter count metadata, expected none"
            )),
            "expected resolver module parameter metadata diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver module symbol 'std' has return type metadata, expected none")),
            "expected resolver module return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_module_absent_type_metadata() {
        let program = parse_program(
            r#"
{ io } = std
main = () i32 {
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(Namespace::Module, "std", Some(vec!["x".to_string()]));
        symbols.set_parameter_type_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Module, "std", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Module, "std", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Module,
            "std",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Module,
            "std",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Module,
            "std",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_owner_name_for_test(
            Namespace::Module,
            "std",
            Some("Option".to_string()),
        );
        symbols.set_variant_payload_count_for_test(Namespace::Module, "std", Some(1));
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Module,
            "std",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Module, "std", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Module,
            "std",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_names: vec!["self".to_string()],
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Module,
            "std",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Module,
            "std",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver module type metadata should fail");

        for expected in [
            "resolver module symbol 'std' has parameter names metadata, expected none",
            "resolver module symbol 'std' has parameter types metadata, expected none",
            "resolver module symbol 'std' has typed parameter types metadata, expected none",
            "resolver module symbol 'std' has typed return type metadata, expected none",
            "resolver module symbol 'std' has type parameter count metadata, expected none",
            "resolver module symbol 'std' has type parameter names metadata, expected none",
            "resolver module symbol 'std' has type parameter bounds metadata, expected none",
            "resolver module symbol 'std' has typed type parameter bound refs metadata, expected none",
            "resolver module symbol 'std' has field count metadata, expected none",
            "resolver module symbol 'std' has field types metadata, expected none",
            "resolver module symbol 'std' has typed field types metadata, expected none",
            "resolver module symbol 'std' has variant names metadata, expected none",
            "resolver module symbol 'std' has variant owner metadata, expected none",
            "resolver module symbol 'std' has variant payload count metadata, expected none",
            "resolver module symbol 'std' has variant payload type metadata, expected none",
            "resolver module symbol 'std' has typed variant payload type metadata, expected none",
            "resolver module symbol 'std' has behavior methods metadata, expected none",
            "resolver module symbol 'std' has typed behavior methods metadata, expected none",
            "resolver module symbol 'std' has behavior parents metadata, expected none",
            "resolver module symbol 'std' has typed behavior parents metadata, expected none",
            "resolver module symbol 'std' has behavior impls metadata, expected none",
            "resolver module symbol 'std' has typed behavior impls metadata, expected none",
            "resolver module symbol 'std' has behavior requires metadata, expected none",
            "resolver module symbol 'std' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver module metadata diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_impl_methods() {
        let program = parse_program(
            r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { return "point" }
}
"#,
        );
        let symbols = SymbolTable::default();
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing impl method resolver symbols should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing value symbol 'Point.stringify'")),
            "expected missing impl method symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_impl_method_signature() {
        let program = parse_program(
            r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(
            Namespace::Value,
            "Point.stringify",
            Some("i32".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver impl method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Point.stringify' has return type 'i32', expected 'str'"
            )),
            "expected resolver impl method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_impl_function_type_signature() {
        let program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}

Point: { x: i32 }

Point.implements(Mapper) {
    map = (value: Point, callback: (i32) i32) (i32) i32 {
        return callback
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(
            Namespace::Value,
            "Point.map",
            Some("i32".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver impl method function type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Point.map' has return type 'i32', expected '(i32) i32'"
            )),
            "expected resolver impl method function type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_impl_method_body_locals() {
        let program = parse_program(
            r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    stringify = (value: Point) str {
        label = "point"
        return label
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "label");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver impl method body local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'label'")),
            "expected missing resolver impl method body local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_enum_variants() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Variant, "Some");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver enum variant symbols should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing variant symbol 'Some'")),
            "expected missing enum variant symbol diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_arity() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Value, "add", Some(1));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function arity mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver value symbol 'add' has parameter count 1, expected 2")),
            "expected resolver function arity diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_parameter_types() {
        let program = parse_program(
            r#"
add = (a: i32, b: f64) f64 { return b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "add",
            Some(vec!["i32".to_string(), "i32".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function parameter type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'add' has parameter types '(i32, i32)', expected '(i32, f64)'"
            )),
            "expected resolver function parameter type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_metadata() {
        let program = parse_program(
            r#"
apply = (callback: (i32) i32, value: i32) i32 { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_type_names_for_test(
            Namespace::Value,
            "apply",
            Some(vec!["i32".to_string(), "i32".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function type parameter metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has parameter types '(i32, i32)', expected '((i32) i32, i32)'"
            )),
            "expected resolver function type parameter metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_parameter_names() {
        let program = parse_program(
            r#"
add = (a: i32, b: f64) f64 { return b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(
            Namespace::Value,
            "add",
            Some(vec!["a".to_string(), "other".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function parameter name mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'add' has parameter names '(a, other)', expected '(a, b)'"
            )),
            "expected resolver function parameter name diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_parameter_locals() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "a");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver parameter local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'a'")),
            "expected missing resolver parameter local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_parameter_local_mutability() {
        let program = parse_program(
            r#"
add = (mut a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_local_mutability_for_test("a", Some(false));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver parameter local mutability mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has mutability immutable, expected mutable")),
            "expected resolver parameter local mutability diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_visibility_and_source() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Local, "a", true);
        symbols.set_import_source_for_test(Namespace::Local, "a", Some("std".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver local visibility/source mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has visibility public, expected private")),
            "expected resolver local visibility diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has source 'std', expected none")),
            "expected resolver local source diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_absent_declaration_metadata() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver local declaration metadata should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has parameter count metadata, expected none")),
            "expected resolver local parameter metadata diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver local symbol 'a' has return type metadata, expected none")),
            "expected resolver local return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_absent_type_metadata() {
        let program = parse_program(
            r#"
add = (a: i32, b: i32) i32 { return a + b }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_names_for_test(Namespace::Local, "a", Some(vec!["x".to_string()]));
        symbols.set_parameter_type_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Local, "a", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Local, "a", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Local,
            "a",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Local,
            "a",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Local,
            "a",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(Namespace::Local, "a", Some(vec!["Some".to_string()]));
        symbols.set_variant_owner_name_for_test(Namespace::Local, "a", Some("Option".to_string()));
        symbols.set_variant_payload_count_for_test(Namespace::Local, "a", Some(1));
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Local,
            "a",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Local, "a", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Local,
            "a",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_names: vec!["self".to_string()],
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Local,
            "a",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Local,
            "a",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver local type metadata should fail");

        for expected in [
            "resolver local symbol 'a' has parameter names metadata, expected none",
            "resolver local symbol 'a' has parameter types metadata, expected none",
            "resolver local symbol 'a' has typed parameter types metadata, expected none",
            "resolver local symbol 'a' has typed return type metadata, expected none",
            "resolver local symbol 'a' has type parameter count metadata, expected none",
            "resolver local symbol 'a' has type parameter names metadata, expected none",
            "resolver local symbol 'a' has type parameter bounds metadata, expected none",
            "resolver local symbol 'a' has typed type parameter bound refs metadata, expected none",
            "resolver local symbol 'a' has field count metadata, expected none",
            "resolver local symbol 'a' has field types metadata, expected none",
            "resolver local symbol 'a' has typed field types metadata, expected none",
            "resolver local symbol 'a' has variant names metadata, expected none",
            "resolver local symbol 'a' has variant owner metadata, expected none",
            "resolver local symbol 'a' has variant payload count metadata, expected none",
            "resolver local symbol 'a' has variant payload type metadata, expected none",
            "resolver local symbol 'a' has typed variant payload type metadata, expected none",
            "resolver local symbol 'a' has behavior methods metadata, expected none",
            "resolver local symbol 'a' has typed behavior methods metadata, expected none",
            "resolver local symbol 'a' has behavior parents metadata, expected none",
            "resolver local symbol 'a' has typed behavior parents metadata, expected none",
            "resolver local symbol 'a' has behavior impls metadata, expected none",
            "resolver local symbol 'a' has typed behavior impls metadata, expected none",
            "resolver local symbol 'a' has behavior requires metadata, expected none",
            "resolver local symbol 'a' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver local metadata diagnostic `{expected}`, got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_var_decl_locals() {
        let program = parse_program(
            r#"
main = () i32 {
    value = 1
    return value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver var local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver var local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_var_decl_local_mutability() {
        let program = parse_program(
            r#"
main = () i32 {
    value ::= 1
    return value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_local_mutability_for_test("value", Some(false));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver var local mutability mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver local symbol 'value' has mutability immutable, expected mutable"
            )),
            "expected resolver var local mutability diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_locals() {
        let program = parse_program(
            r#"
main = () i32 {
    return 0
}
"#,
        );
        let symbols_program = parse_program(
            r#"
main = () i32 {
    value = 1
    return 0
}
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&symbols_program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table has extra local symbol 'value'")),
            "expected extra resolver local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_local_mutability_by_scope() {
        let program = parse_program(
            r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    return value
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let inner_scope = symbols
            .symbols()
            .iter()
            .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
            .map(|symbol| symbol.scope_id)
            .max()
            .expect("inner value local");
        symbols.set_local_mutability_in_scope_for_test("value", inner_scope, Some(true));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver scoped local mutability mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver local symbol 'value' has mutability mutable, expected immutable"
            )),
            "expected scoped resolver local mutability diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_pattern_locals() {
        let program = parse_program(
            r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    return value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "inner");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver pattern local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'inner'")),
            "expected missing resolver pattern local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_top_level_expr_locals() {
        let program = parse_program(
            r#"
value := 1
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver top-level expr local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver top-level expr local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_closure_locals() {
        let program = parse_program(
            r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    return 0
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "inner");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver closure local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'inner'")),
            "expected missing resolver closure local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_struct_field_default_locals() {
        let program = parse_program(
            r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver struct field default local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver struct field default local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_requires_resolver_behavior_default_locals() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str {
        value = "{}"
        value
    }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.remove_for_test(Namespace::Local, "value");
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("missing resolver behavior default local should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver symbol table missing local symbol 'value'")),
            "expected missing resolver behavior default local diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_visibility() {
        let program = parse_program(
            r#"
pub exported = () i32 { return 1 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Value, "exported", false);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'exported' has visibility private, expected public"
            )),
            "expected resolver function visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_return_type() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(Namespace::Value, "main", Some("bool".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function return mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver value symbol 'main' has return type 'bool', expected 'i32'")),
            "expected resolver function return diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_return_metadata() {
        let program = parse_program(
            r#"
factory = () (i32) i32 {
    return (value: i32) i32 { value }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_return_type_name_for_test(Namespace::Value, "factory", Some("i32".to_string()));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function type return metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'factory' has return type 'i32', expected '(i32) i32'"
            )),
            "expected resolver function type return metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_typed_signature_metadata() {
        let program = parse_program(
            r#"
apply = (callback: (i32) i32) (i32) i32 {
    return callback
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_parameter_types_for_test(Namespace::Value, "apply", Some(vec![AstType::I32]));
        symbols.set_return_type_for_test(Namespace::Value, "apply", Some(AstType::I32));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed function signature metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
            )),
            "expected resolver typed parameter diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
            )),
            "expected resolver typed return diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_counts() {
        let program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_count_for_test(Namespace::Value, "identity", Some(0));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic arity mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'identity' has type parameter count 0, expected 1"
            )),
            "expected resolver function generic arity diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_names() {
        let program = parse_program(
            r#"
identity<T> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_names_for_test(
            Namespace::Value,
            "identity",
            Some(vec!["U".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic parameter name mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver function generic parameter name diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_bounds() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
encode<T: Json> = (value: T) str { return "encoded" }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Value,
            "encode",
            Some(vec![("T".to_string(), "Other".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic bound mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'encode' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver function generic bound diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { return value }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Value,
            "identity",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: vec![AstType::Str],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function generic bound ref mismatch should fail");

        let expected = "resolver value symbol 'identity' has type parameter bound refs '(T: Json<str>)', expected '(T: Json<T>)'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver function generic bound ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_function_absent_declaration_metadata() {
        let program = parse_program(
            r#"
main = () i32 { return 0 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Value, "main", Some("std".to_string()));
        symbols.set_field_count_for_test(Namespace::Value, "main", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Value,
            "main",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Value,
            "main",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Value,
            "main",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Value, "main", Some(AstType::I32));
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Value,
            "main",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_names: vec!["self".to_string()],
                parameter_types: vec![AstType::Named("Self".to_string())],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Value,
            "main",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Value,
            "main",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver function declaration metadata should fail");

        for expected in [
            "resolver value symbol 'main' has source 'std', expected none",
            "resolver value symbol 'main' has field count metadata, expected none",
            "resolver value symbol 'main' has field types metadata, expected none",
            "resolver value symbol 'main' has typed field types metadata, expected none",
            "resolver value symbol 'main' has variant names metadata, expected none",
            "resolver value symbol 'main' has variant payload type metadata, expected none",
            "resolver value symbol 'main' has typed variant payload type metadata, expected none",
            "resolver value symbol 'main' has behavior methods metadata, expected none",
            "resolver value symbol 'main' has typed behavior methods metadata, expected none",
            "resolver value symbol 'main' has behavior parents metadata, expected none",
            "resolver value symbol 'main' has typed behavior parents metadata, expected none",
            "resolver value symbol 'main' has behavior impls metadata, expected none",
            "resolver value symbol 'main' has typed behavior impls metadata, expected none",
            "resolver value symbol 'main' has behavior requires metadata, expected none",
            "resolver value symbol 'main' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver function declaration metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_parameter_counts() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_count_for_test(Namespace::Type, "Box", Some(0));
        symbols.set_type_parameter_count_for_test(Namespace::Behavior, "Serializable", Some(0));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic arity mismatches should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver type symbol 'Box' has type parameter count 0, expected 1")),
            "expected resolver type generic arity diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter count 0, expected 1"
            )),
            "expected resolver behavior generic arity diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_parameter_names() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_names_for_test(
            Namespace::Type,
            "Box",
            Some(vec!["U".to_string()]),
        );
        symbols.set_type_parameter_names_for_test(
            Namespace::Behavior,
            "Serializable",
            Some(vec!["U".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic parameter name mismatches should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver type generic parameter name diagnostic, got {err:?}"
        );
        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver behavior generic parameter name diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_visibility() {
        let program = parse_program(
            r#"
pub Box<T>: { value: T }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Type, "Box", false);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver type visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver type symbol 'Box' has visibility private, expected public")),
            "expected resolver type visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_visibility() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Behavior, "Json", true);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Json' has visibility public, expected private"
            )),
            "expected resolver behavior visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_parameter_bounds() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
Box<T: Json>: { value: T }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Type,
            "Box",
            Some(vec![("T".to_string(), "Other".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver type generic bound mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver type generic bound diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Behavior,
            "Serializable",
            Some(vec![("T".to_string(), "Json<i32>".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior generic bound mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter bounds '(T: Json<i32>)', expected '(T: Json<T>)'"
            )),
            "expected resolver behavior generic bound diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_type_like_absent_value_metadata() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Type, "Box", Some("std".to_string()));
        symbols.set_parameter_count_for_test(Namespace::Type, "Box", Some(1));
        symbols.set_return_type_name_for_test(Namespace::Type, "Box", Some("i32".to_string()));
        symbols.set_return_type_for_test(Namespace::Type, "Box", Some(AstType::I32));
        symbols.set_import_source_for_test(Namespace::Behavior, "Json", Some("std".to_string()));
        symbols.set_parameter_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["value".to_string()]),
        );
        symbols.set_parameter_type_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Self".to_string()]),
        );
        symbols.set_parameter_types_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![AstType::SelfType]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver type-like value metadata should fail");

        for expected in [
            "resolver type symbol 'Box' has source 'std', expected none",
            "resolver type symbol 'Box' has parameter count metadata, expected none",
            "resolver type symbol 'Box' has return type metadata, expected none",
            "resolver type symbol 'Box' has typed return type metadata, expected none",
            "resolver behavior symbol 'Json' has source 'std', expected none",
            "resolver behavior symbol 'Json' has parameter names metadata, expected none",
            "resolver behavior symbol 'Json' has parameter types metadata, expected none",
            "resolver behavior symbol 'Json' has typed parameter types metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver type-like value metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_method_signatures() {
        let program = parse_program(
            r#"
Serializable: behavior {
    encode: (Self, i32) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Serializable",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string(), "bool".to_string()],
                "str".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has methods '(encode(Self, bool) str)', expected '(encode(Self, i32) str)'"
            )),
            "expected resolver behavior method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_function_type_method_signatures() {
        let program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Mapper",
            Some(vec![(
                "map".to_string(),
                vec!["Self".to_string(), "i32".to_string()],
                "(i32) i32".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior function type method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, i32) (i32) i32)', expected '(map(Self, (i32) i32) (i32) i32)'"
            )),
            "expected resolver behavior function type method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_method_types() {
        let program = parse_program(
            r#"
Mapper: behavior {
    map: (Self, (i32) i32) (i32) i32
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_types_for_test(
            Namespace::Behavior,
            "Mapper",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "map".to_string(),
                parameter_names: vec!["__arg0".to_string(), "__arg1".to_string()],
                parameter_types: vec![AstType::SelfType, AstType::I32],
                return_type: AstType::Function {
                    params: vec![AstType::I32],
                    ret: Box::new(AstType::I32),
                },
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed behavior method metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has typed methods '(map(__arg0: Self, __arg1: i32) (i32) i32)', expected '(map(__arg0: Self, __arg1: (i32) i32) (i32) i32)'"
            )),
            "expected resolver typed behavior method diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_method_signatures() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior method signature mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Json' has methods '(encode(Self) str)', expected '(encode(Self) T)'"
            )),
            "expected resolver generic behavior method signature diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_function_type_method_signatures(
    ) {
        let program = parse_program(
            r#"
Mapper<T>: behavior {
    map: (Self, (T) T) (T) T
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Behavior,
            "Mapper",
            Some(vec![(
                "map".to_string(),
                vec!["Self".to_string(), "T".to_string()],
                "(T) T".to_string(),
            )]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior function type method mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Mapper' has methods '(map(Self, T) (T) T)', expected '(map(Self, (T) T) (T) T)'"
            )),
            "expected resolver generic behavior function type method diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_absent_type_metadata() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_count_for_test(Namespace::Behavior, "Json", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Behavior,
            "Json",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Behavior, "Json", Some(AstType::I32));
        symbols.set_behavior_impl_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Debug".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec!["Debug".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Behavior,
            "Json",
            Some(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior type metadata should fail");

        for expected in [
            "resolver behavior symbol 'Json' has field count metadata, expected none",
            "resolver behavior symbol 'Json' has field types metadata, expected none",
            "resolver behavior symbol 'Json' has typed field types metadata, expected none",
            "resolver behavior symbol 'Json' has variant names metadata, expected none",
            "resolver behavior symbol 'Json' has variant payload type metadata, expected none",
            "resolver behavior symbol 'Json' has typed variant payload type metadata, expected none",
            "resolver behavior symbol 'Json' has behavior impls metadata, expected none",
            "resolver behavior symbol 'Json' has typed behavior impls metadata, expected none",
            "resolver behavior symbol 'Json' has behavior requires metadata, expected none",
            "resolver behavior symbol 'Json' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver behavior type metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_parent_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_names_for_test(Namespace::Behavior, "PrettyJson", None);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior parent metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'none', expected to include 'Json'"
            )),
            "expected resolver behavior parent metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_parent_names() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_names_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec!["Json<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior parent metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'PrettyJson' has parents 'Json<i32>', expected to include 'Json<str>'"
            )),
            "expected resolver generic behavior parent metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_parent_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior parent ref mismatch should fail");

        let expected =
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior parent ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_accepts_resolver_behavior_parent_child_type_param_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
        );
        let symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        let mut tc = TypeChecker::new();

        tc.check_program_with_symbols(&program, &symbols)
            .expect("resolver parent type arg using child type parameter should validate");
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_names_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec!["Json".to_string(), "Debug".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior parent metadata should fail");

        let expected =
            "resolver behavior symbol 'PrettyJson' has parents 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior parent metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_parent_refs() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Behavior,
            "PrettyJson",
            Some(vec![
                BehaviorRefMetadata {
                    name: "Json".to_string(),
                    type_args: vec![],
                },
                BehaviorRefMetadata {
                    name: "Debug".to_string(),
                    type_args: vec![],
                },
            ]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior parent ref metadata should fail");

        let expected =
            "resolver behavior symbol 'PrettyJson' has parent refs 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior parent ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_impl_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_names_for_test(Namespace::Type, "Point", None);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior impl metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impls 'none', expected to include 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior impl metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_impl_names() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior impl metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impls 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior impl metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_impl_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Type,
            "Point",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior impl ref mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impl refs 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior impl ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_behavior_required_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_names_for_test(Namespace::Type, "Point", None);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver behavior requires metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires 'none', expected to include 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior requires metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_required_names() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json<i32>".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior requires metadata mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior requires metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_behavior_required_refs() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Point: { x: i32 }

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_refs_for_test(
            Namespace::Type,
            "Point",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic behavior requires ref mismatch should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires refs 'Json<i32>', expected to include 'Json<str>'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver generic behavior requires ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json".to_string(), "Debug".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior impl metadata should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impls 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior impl metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_impl_refs() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Type,
            "Point",
            Some(vec![
                BehaviorRefMetadata {
                    name: "Json".to_string(),
                    type_args: vec![],
                },
                BehaviorRefMetadata {
                    name: "Debug".to_string(),
                    type_args: vec![],
                },
            ]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior impl ref metadata should fail");

        let expected =
            "resolver type symbol 'Point' has behavior impl refs 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior impl ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_required_names() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Json".to_string(), "Debug".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior requires metadata should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior requires metadata diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_rejects_extra_resolver_behavior_required_refs() {
        let program = parse_program(
            r#"
Json: behavior {
    encode: (Self) str
}

Debug: behavior {
    debug: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_behavior_required_refs_for_test(
            Namespace::Type,
            "Point",
            Some(vec![
                BehaviorRefMetadata {
                    name: "Json".to_string(),
                    type_args: vec![],
                },
                BehaviorRefMetadata {
                    name: "Debug".to_string(),
                    type_args: vec![],
                },
            ]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("extra resolver behavior requires ref metadata should fail");

        let expected =
            "resolver type symbol 'Point' has behavior requires refs 'Json, Debug', expected 'Json'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected extra resolver behavior requires ref diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_field_counts() {
        let program = parse_program(
            r#"
Point: { x: i32, y: i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_count_for_test(Namespace::Type, "Point", Some(1));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct field count mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver type symbol 'Point' has field count 1, expected 2")),
            "expected resolver struct field count diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_field_types() {
        let program = parse_program(
            r#"
Point: { x: i32, y: f64 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec![
                ("x".to_string(), "i32".to_string()),
                ("y".to_string(), "i32".to_string()),
            ]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct field type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Point' has fields '(x: i32, y: i32)', expected '(x: i32, y: f64)'"
            )),
            "expected resolver struct field type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_function_type_fields() {
        let program = parse_program(
            r#"
Pipeline: { callback: (i32) i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Pipeline",
            Some(vec![("callback".to_string(), "i32".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct function type field mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver struct function type field diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_typed_field_metadata() {
        let program = parse_program(
            r#"
Pipeline: { callback: (i32) i32 }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_types_for_test(
            Namespace::Type,
            "Pipeline",
            Some(vec![("callback".to_string(), AstType::I32)]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed struct field metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver typed struct field diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_struct_field_types() {
        let program = parse_program(
            r#"
Box<T>: { value: T }
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Box",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic struct field mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has fields '(value: i32)', expected '(value: T)'"
            )),
            "expected resolver generic struct field diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata() {
        let program = parse_program(
            r#"
Point: { x: i32 }
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_names_for_test(
            Namespace::Type,
            "Point",
            Some(vec!["Some".to_string()]),
        );
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Type,
            "Point",
            Some("i32".to_string()),
        );
        symbols.set_variant_payload_type_for_test(Namespace::Type, "Point", Some(AstType::I32));
        symbols.set_field_count_for_test(Namespace::Type, "Option", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Type,
            "Option",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Type,
            "Option",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver struct/enum kind metadata should fail");

        for expected in [
            "resolver type symbol 'Point' has variant names metadata, expected none",
            "resolver type symbol 'Point' has variant payload type metadata, expected none",
            "resolver type symbol 'Point' has typed variant payload type metadata, expected none",
            "resolver type symbol 'Option' has field count metadata, expected none",
            "resolver type symbol 'Option' has field types metadata, expected none",
            "resolver type symbol 'Option' has typed field types metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver struct/enum kind metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_payload_counts() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_count_for_test(Namespace::Variant, "Some", Some(0));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant payload count mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver variant symbol 'Some' has payload count 0, expected 1")),
            "expected resolver enum variant payload count diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_visibility() {
        let program = parse_program(
            r#"
pub Option<T>: Some(T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_public_for_test(Namespace::Variant, "Some", false);
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant visibility mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Some' has visibility private, expected public"
            )),
            "expected resolver enum variant visibility diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_payload_types() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Some",
            Some("bool".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant payload type mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Some' has payload type 'bool', expected 'i32'"
            )),
            "expected resolver enum variant payload type diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_function_type_payloads() {
        let program = parse_program(
            r#"
Callback: Wrap((i32) i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Wrap",
            Some("i32".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum function type payload mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Wrap' has payload type 'i32', expected '(i32) i32'"
            )),
            "expected resolver enum function type payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_typed_payload_metadata() {
        let program = parse_program(
            r#"
Callback: Wrap((i32) i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_for_test(Namespace::Variant, "Wrap", Some(AstType::I32));
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver typed enum payload metadata mismatch should fail");

        assert!(
            err.iter().any(|d| d.message.contains(
                "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
            )),
            "expected resolver typed enum payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads() {
        let program = parse_program(
            r#"
Callback<T>: Wrap((T) T), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Wrap",
            Some("T".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic enum function type payload mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver variant symbol 'Wrap' has payload type 'T', expected '(T) T'")),
            "expected resolver generic enum function type payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_generic_enum_payload_types() {
        let program = parse_program(
            r#"
Result<T, E>: Ok(T), Err(E)
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_payload_type_name_for_test(
            Namespace::Variant,
            "Err",
            Some("T".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver generic enum payload mismatch should fail");

        assert!(
            err.iter().any(|d| d
                .message
                .contains("resolver variant symbol 'Err' has payload type 'T', expected 'E'")),
            "expected resolver generic enum payload diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_variant_absent_other_metadata() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_import_source_for_test(Namespace::Variant, "Some", Some("std".to_string()));
        symbols.set_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
        symbols.set_parameter_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["value".to_string()]),
        );
        symbols.set_parameter_type_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["i32".to_string()]),
        );
        symbols.set_parameter_types_for_test(Namespace::Variant, "Some", Some(vec![AstType::I32]));
        symbols.set_return_type_name_for_test(Namespace::Variant, "Some", Some("i32".to_string()));
        symbols.set_return_type_for_test(Namespace::Variant, "Some", Some(AstType::I32));
        symbols.set_type_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
        symbols.set_type_parameter_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["T".to_string()]),
        );
        symbols.set_type_parameter_bounds_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![("T".to_string(), "Json".to_string())]),
        );
        symbols.set_type_parameter_bound_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![TypeParameterBoundRefMetadata {
                type_parameter: "T".to_string(),
                behavior: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_field_count_for_test(Namespace::Variant, "Some", Some(1));
        symbols.set_field_type_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![("value".to_string(), "i32".to_string())]),
        );
        symbols.set_field_types_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![("value".to_string(), AstType::I32)]),
        );
        symbols.set_variant_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Other".to_string()]),
        );
        symbols.set_behavior_method_signatures_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![(
                "encode".to_string(),
                vec!["Self".to_string()],
                "str".to_string(),
            )]),
        );
        symbols.set_behavior_method_types_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorMethodTypeMetadata {
                name: "encode".to_string(),
                parameter_names: vec!["self".to_string()],
                parameter_types: vec![AstType::SelfType],
                return_type: AstType::Str,
            }]),
        );
        symbols.set_behavior_parent_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_parent_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_impl_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_impl_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        symbols.set_behavior_required_names_for_test(
            Namespace::Variant,
            "Some",
            Some(vec!["Json".to_string()]),
        );
        symbols.set_behavior_required_refs_for_test(
            Namespace::Variant,
            "Some",
            Some(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: Vec::new(),
            }]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver variant non-variant metadata should fail");

        for expected in [
            "resolver variant symbol 'Some' has source 'std', expected none",
            "resolver variant symbol 'Some' has parameter count metadata, expected none",
            "resolver variant symbol 'Some' has parameter names metadata, expected none",
            "resolver variant symbol 'Some' has parameter types metadata, expected none",
            "resolver variant symbol 'Some' has typed parameter types metadata, expected none",
            "resolver variant symbol 'Some' has return type metadata, expected none",
            "resolver variant symbol 'Some' has typed return type metadata, expected none",
            "resolver variant symbol 'Some' has type parameter count metadata, expected none",
            "resolver variant symbol 'Some' has type parameter names metadata, expected none",
            "resolver variant symbol 'Some' has type parameter bounds metadata, expected none",
            "resolver variant symbol 'Some' has typed type parameter bound refs metadata, expected none",
            "resolver variant symbol 'Some' has field count metadata, expected none",
            "resolver variant symbol 'Some' has field types metadata, expected none",
            "resolver variant symbol 'Some' has typed field types metadata, expected none",
            "resolver variant symbol 'Some' has variant names metadata, expected none",
            "resolver variant symbol 'Some' has behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has behavior requires metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver variant metadata diagnostic '{expected}', got {err:?}"
            );
        }
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_names() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_names_for_test(
            Namespace::Type,
            "Option",
            Some(vec!["Some".to_string()]),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant names mismatch should fail");

        let expected =
            "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver enum variant names diagnostic, got {err:?}"
        );
    }

    #[test]
    fn check_program_with_symbols_validates_resolver_enum_variant_owner_names() {
        let program = parse_program(
            r#"
Option: Some(i32), None
"#,
        );
        let mut symbols = crate::resolver::Resolver::new()
            .resolve_program(&program)
            .expect("resolver succeeds");
        symbols.set_variant_owner_name_for_test(
            Namespace::Variant,
            "Some",
            Some("Result".to_string()),
        );
        let mut tc = TypeChecker::new();

        let err = tc
            .check_program_with_symbols(&program, &symbols)
            .expect_err("resolver enum variant owner mismatch should fail");

        let expected = "resolver variant symbol 'Some' has owner 'Result', expected 'Option'";
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver enum variant owner diagnostic, got {err:?}"
        );
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
    fn binary_op_mixed_numeric_width_requires_cast() {
        let tc = TypeChecker::new();
        let err = tc
            .check_binary_op(BinaryOp::Add, &Type::I32, &Type::I64, &Span::dummy())
            .expect_err("mixed integer arithmetic should fail");
        assert!(
            err.message
                .contains("arithmetic operands must have the same type"),
            "expected mixed numeric diagnostic, got {err:?}"
        );

        let err = tc
            .check_binary_op(BinaryOp::Mul, &Type::F32, &Type::F64, &Span::dummy())
            .expect_err("mixed float arithmetic should fail");
        assert!(
            err.message
                .contains("arithmetic operands must have the same type"),
            "expected mixed numeric diagnostic, got {err:?}"
        );
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
    fn struct_literal_uses_default_for_omitted_field() {
        use crate::ast::declarations::StructField;
        use crate::ast::typed::{TypedExprKind, TypedStatementKind};
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
                            default: Some(Expression::IntLiteral {
                                value: 2,
                                span: Span::dummy(),
                            }),
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

        let typed = tc
            .check_program(&program)
            .expect("defaulted struct field may be omitted");
        let TypedStatementKind::VarDecl { value, .. } = &typed.functions[0].body.statements[0].kind
        else {
            panic!("expected var decl");
        };
        let TypedExprKind::StructLiteral { fields, .. } = &value.kind else {
            panic!("expected struct literal");
        };
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[1].0, "y");
        assert!(matches!(fields[1].1.kind, TypedExprKind::IntLiteral(2)));
    }

    #[test]
    fn generic_struct_literal_uses_substituted_default_for_omitted_field() {
        use crate::ast::declarations::{StructField, TypeParam};
        use crate::ast::typed::{TypedExprKind, TypedStatementKind};
        use crate::ast::{Expression, Program, Statement};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![
                Declaration::Struct {
                    name: "Box".into(),
                    type_params: vec![TypeParam {
                        name: "T".into(),
                        constraint: None,
                        constraint_type_args: Vec::new(),
                        span: Span::dummy(),
                    }],
                    fields: vec![StructField {
                        name: "value".into(),
                        ty: AstType::Named("T".into()),
                        default: Some(Expression::Block {
                            statements: vec![Statement::VarDecl {
                                name: "same".into(),
                                ty: Some(AstType::Named("T".into())),
                                value: Expression::StringLiteral {
                                    value: "fallback".into(),
                                    span: Span::dummy(),
                                },
                                mutable: false,
                                constant: false,
                                span: Span::dummy(),
                            }],
                            expr: Some(Box::new(Expression::Identifier {
                                name: "same".into(),
                                span: Span::dummy(),
                            })),
                            span: Span::dummy(),
                        }),
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
                            name: "box".into(),
                            ty: None,
                            value: Expression::StructLiteral {
                                name: "Box".into(),
                                type_args: vec![AstType::Str],
                                fields: Vec::new(),
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

        let typed = tc
            .check_program(&program)
            .expect("generic defaulted struct field may be omitted");
        let TypedStatementKind::VarDecl { value, .. } = &typed.functions[0].body.statements[0].kind
        else {
            panic!("expected var decl");
        };
        let TypedExprKind::StructLiteral { fields, .. } = &value.kind else {
            panic!("expected struct literal");
        };
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "value");
        assert_eq!(fields[0].1.ty, Type::Str);
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
    fn enum_variant_unknown_variant_is_error() {
        let program = parse_program(
            r#"
Status: Ok, Err

main = () void {
    value = Status.Pending
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown enum variant should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
            "expected unknown variant diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_variant_payload_type_mismatch_is_error() {
        let program = parse_program(
            r#"
Maybe: Some(i32), None

main = () void {
    value = Maybe.Some("bad")
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("enum payload type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("payload for enum variant `Maybe.Some` expects `i32`, found `str`")),
            "expected payload type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn assignment_to_immutable_binding_is_error() {
        let program = parse_program(
            r#"
main = () void {
    x = 1
    x = 2
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("immutable assignment should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("cannot assign to immutable variable `x`")),
            "expected immutable assignment diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn assignment_type_mismatch_is_error() {
        let program = parse_program(
            r#"
main = () void {
    x ::= 1
    x = "bad"
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("assignment type mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("assignment to `x` expects `i32`, found `str`")),
            "expected assignment type diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn invalid_field_access_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

main = () void {
    p = Point { x: 1 }
    y = p.y
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("invalid field access should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("type `Point` has no field `y`")),
            "expected invalid field diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn implicit_integer_width_conversion_is_error() {
        let program = parse_program(
            r#"
take_i64 = (value: i64) void {}

main = () void {
    x: i32 = 1
    take_i64(x)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("implicit integer conversion should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("argument 1 for `take_i64` expects `i64`, found `i32`")),
            "expected integer conversion diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn implicit_float_width_conversion_is_error() {
        use crate::ast::{Expression, Program};
        let mut tc = TypeChecker::new();
        let program = Program {
            declarations: vec![Declaration::Function {
                name: "take_f32".into(),
                type_params: Vec::new(),
                params: vec![ast::Param {
                    name: "value".into(),
                    ty: AstType::F32,
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
            }],
            file_id: 0,
        };
        tc.collect_declarations(&program.declarations);

        let expected = tc.functions["take_f32"].params[0].1.clone();
        assert!(!tc.types_compatible(&tc.resolve_type(&expected), &Type::F64));
    }

    #[test]
    fn unknown_root_std_module_call_is_error() {
        let program = parse_program(
            r#"
{ io } = std

main = () void {
    io.nope("bad")
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown std module function should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("undefined module function `io.nope`")),
            "expected undefined module function diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn known_root_std_runtime_standins_remain_allowed() {
        let program = parse_program(
            r#"
{ io } = std

main = () void {
    io.print("hello")
    io.println("world")
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("temporary root std io stand-ins should typecheck");
    }

    #[test]
    fn non_void_function_without_return_is_error() {
        let program = parse_program(
            r#"
missing = () i32 {
    x = 1
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-void fallthrough should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("function `missing` must return `i32` on all non-error paths")),
            "expected missing return diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_missing_variant_is_error() {
        let program = parse_program(
            r#"
Color: Red, Green, Blue

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-exhaustive enum match should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("non-exhaustive match on `Color`: missing `Blue`")),
            "expected non-exhaustive enum diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_duplicate_variant_is_error() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Red { "again" }
        | Green { "green" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("duplicate enum match arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("duplicate match arm for `Color.Red`")),
            "expected duplicate enum arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_unknown_variant_is_error() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Blue { "blue" }
        | Green { "green" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown enum match arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("enum `Color` has no variant `Blue`")),
            "expected unknown enum arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_payload_shape_is_checked() {
        let program = parse_program(
            r#"
Maybe: Some(i32), None

describe = (m: Maybe) StaticString {
    m ?
        | Some { "some" }
        | None(value) { "none" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("enum match payload shape should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("match arm `Maybe.Some` requires a payload")),
            "expected missing payload diagnostic, got {errors:?}"
        );
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("match arm `Maybe.None` does not accept a payload")),
            "expected forbidden payload diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_wildcard_after_all_variants_is_redundant() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | Red { "red" }
        | Green { "green" }
        | _ { "fallback" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("redundant enum wildcard arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("redundant wildcard match arm")),
            "expected redundant wildcard diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn enum_match_variant_after_wildcard_is_redundant() {
        let program = parse_program(
            r#"
Color: Red, Green

describe = (c: Color) StaticString {
    c ?
        | _ { "fallback" }
        | Red { "red" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("enum variant after wildcard should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("redundant match arm for `Color.Red`")),
            "expected redundant enum arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn bool_match_missing_arm_is_error_for_value_match() {
        let program = parse_program(
            r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-exhaustive boolean value match should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("non-exhaustive bool match: missing `false`")),
            "expected non-exhaustive bool diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn bool_match_duplicate_arm_is_error() {
        let program = parse_program(
            r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { "yes" }
        | true { "again" }
        | false { "no" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("duplicate boolean match arm should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("duplicate match arm for `true`")),
            "expected duplicate bool arm diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn match_arm_return_does_not_force_never_result_type() {
        let program = parse_program(
            r#"
describe = (flag: bool) StaticString {
    flag ?
        | true { return "early" }
        | false { "late" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let typed = tc
            .check_program(&program)
            .expect("returning arm should not force match type to never");
        let body = &typed.functions[0].body;
        assert_eq!(body.ty, Type::Str);
    }

    #[test]
    fn types_compatible_basics() {
        let tc = TypeChecker::new();
        // Same types
        assert!(tc.types_compatible(&Type::I32, &Type::I32));
        // Numeric conversions require explicit casts except literal coercion.
        assert!(!tc.types_compatible(&Type::I64, &Type::I32));
        assert!(!tc.types_compatible(&Type::F32, &Type::F64));
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
    fn substitute_type_covers_all_composite_type_shapes() {
        let tc = TypeChecker::new();
        let mut subs = HashMap::new();
        subs.insert("T".to_string(), Type::I32);

        assert_eq!(
            tc.substitute_type(
                &AstType::RawPtr(Box::new(AstType::Named("T".into()))),
                &subs
            ),
            Type::RawPtr(Box::new(Type::I32))
        );
        assert_eq!(
            tc.substitute_type(
                &AstType::MutPtr(Box::new(AstType::Named("T".into()))),
                &subs
            ),
            Type::MutPtr(Box::new(Type::I32))
        );
        assert_eq!(
            tc.substitute_type(&AstType::Slice(Box::new(AstType::Named("T".into()))), &subs),
            Type::Slice(Box::new(Type::I32))
        );
        assert_eq!(
            tc.substitute_type(
                &AstType::Array {
                    elem: Box::new(AstType::Named("T".into())),
                    size: Some(3),
                },
                &subs,
            ),
            Type::Array {
                elem: Box::new(Type::I32),
                size: Some(3),
            }
        );
        assert_eq!(
            tc.substitute_type(
                &AstType::Function {
                    params: vec![AstType::Named("T".into())],
                    ret: Box::new(AstType::Named("T".into())),
                },
                &subs,
            ),
            Type::Function {
                params: vec![Type::I32],
                ret: Box::new(Type::I32),
            }
        );
    }

    #[test]
    fn substitute_type_preserves_function_type_arguments_in_nested_generics() {
        let mut tc = TypeChecker::new();
        tc.structs.insert(
            "Box".to_string(),
            StructInfo {
                name: "Box".to_string(),
                fields: vec![("value".to_string(), AstType::Named("T".to_string()))],
                field_defaults: HashMap::new(),
                type_params: vec!["T".to_string()],
                type_param_bounds: HashMap::new(),
            },
        );
        let function_type = Type::Function {
            params: vec![Type::I32],
            ret: Box::new(Type::I32),
        };
        let mut subs = HashMap::new();
        subs.insert("T".to_string(), function_type.clone());

        assert_eq!(
            tc.substitute_type(
                &AstType::Generic {
                    name: "Box".to_string(),
                    type_args: vec![AstType::Named("T".to_string())],
                },
                &subs,
            ),
            Type::Struct {
                name: "Box_fn_i32_ret_i32".to_string(),
                fields: vec![("value".to_string(), function_type)],
            }
        );
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
                constraint_type_args: Vec::new(),
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
    fn generic_method_collection() {
        use crate::ast::Expression;
        let mut tc = TypeChecker::new();
        let decls = vec![Declaration::Method {
            type_name: "Box".into(),
            method_name: "get".into(),
            type_params: vec![crate::ast::declarations::TypeParam {
                name: "T".into(),
                constraint: None,
                constraint_type_args: Vec::new(),
                span: Span::dummy(),
            }],
            params: vec![crate::ast::Param {
                name: "value".into(),
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
        let info = tc.methods.get("Box.get").unwrap();
        assert_eq!(info.type_params, vec!["T".to_string()]);
        assert!(tc.generic_methods.contains_key("Box.get"));
    }

    #[test]
    fn type_impl_method_collection() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 {
        return self.x
    }
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.collect_declarations(&program.declarations);
        let info = tc.methods.get("Point.get").unwrap();
        assert_eq!(info.params.len(), 1);
        assert_eq!(info.return_type, AstType::I32);
    }

    #[test]
    fn behavior_declaration_collection() {
        let program = parse_program(
            r#"
Serializable: behavior {
    to_json: (Self) String
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.collect_declarations(&program.declarations);
        let info = tc.behaviors.get("Serializable").unwrap();
        assert_eq!(info.name, "Serializable");
        assert_eq!(info.methods.len(), 1);
        assert_eq!(info.methods[0].name, "to_json");
    }

    #[test]
    fn behavior_impl_with_required_method_passes() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("valid behavior impl should typecheck");
    }

    #[test]
    fn behavior_impl_missing_required_method_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("missing behavior method should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` implementation of `Json` is missing required method `to_json`"
            )),
            "expected missing behavior method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_can_omit_default_method() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str { return "{}" }
}

Point.implements(Json) {
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("behavior impl may omit a method with a default body");
    }

    #[test]
    fn behavior_impl_duplicate_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("duplicate behavior impl should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("duplicate implementation of behavior `Json` for type `Point`")),
            "expected duplicate behavior impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_without_type_args_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.implements(Json) {
    encode = (value: Point) str { return "point" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior impl without type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")),
            "expected generic behavior impl arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_with_type_args_passes_requires() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.requires(Json<str>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior impl should satisfy matching generic requires");
    }

    #[test]
    fn behavior_impl_generic_behavior_type_arg_bound_failure_is_error() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { return value }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior type argument bound should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` does not implement behavior `Json<Point>` required by `T`"
            )),
            "expected generic behavior type argument bound diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_type_arg_bound_passes_when_satisfied() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Point: { x: i32 }

Point.implements(Json<Point>) {
    encode = (value: Point) Point { return value }
}

Point.implements(Serializable<Point>) {
    serialize = (value: Point) Point { return value }
}
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior type argument bound should pass when satisfied");
    }

    #[test]
    fn behavior_requires_generic_behavior_type_arg_arity_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.requires(Json<i32, str>)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior requires arity mismatch should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 2")),
            "expected generic behavior requires arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_generic_behavior_substitutes_method_signature() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) i32 { return 1 }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior impl return mismatch should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "method `encode` for behavior `Json_str` expects return `str`, found `i32`"
            )),
            "expected substituted behavior method return diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_overlapping_inherited_behavior_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

Point.implements(PrettyJson) {
    to_json = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("overlapping inherited behavior impl should fail");
        assert!(
            errors.iter().any(|d| {
                d.message.contains(
                    "overlapping implementations of behaviors `Json` and `PrettyJson` for type `Point`",
                )
            }),
            "expected overlapping behavior impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_requires_passes_when_impl_exists() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

Point.requires(Json)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("requires should pass when behavior impl exists");
    }

    #[test]
    fn behavior_requires_rejects_missing_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.requires(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("requires should fail without behavior impl");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("type `Point` does not implement required behavior `Json`")),
            "expected requires missing impl diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_requires_generic_behavior_without_type_args_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) str
}

Point.requires(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior requires without type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")),
            "expected generic behavior requires arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_requires_parent_methods() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("extended behavior should require parent methods");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` implementation of `PrettyJson` is missing required method `to_json`"
            )),
            "expected inherited missing method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_impl_satisfies_parent_requires() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

Point.requires(Json)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("implementation of child behavior should satisfy parent requires");
    }

    #[test]
    fn behavior_extends_generic_parent_requires_substituted_methods() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic parent method should be required with substituted signature");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` implementation of `PrettyJson` is missing required method `encode`"
            )),
            "expected inherited generic parent missing method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_generic_parent_satisfies_specialized_requires() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

Point.requires(Json<str>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("child behavior impl should satisfy specialized generic parent requires");
    }

    #[test]
    fn behavior_extends_generic_parent_accepts_child_type_parameter_arg() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}

Pretty<T: Json<T>>: behavior {
    pretty: (Self) T
}

Pretty.extends(Serializable<T>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior parent should accept child type parameter args");
    }

    #[test]
    fn behavior_impl_generic_parent_overlap_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.implements(PrettyJson) {
    encode = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("specialized parent and child behavior impls should overlap");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "overlapping implementations of behaviors `Json_str` and `PrettyJson` for type `Point`"
            )),
            "expected specialized behavior impl overlap diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_distinct_generic_specializations_do_not_overlap() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

Point.implements(Json<i32>) {
    encode = (value: Point) i32 { return value.x }
}

Point.requires(Json<str>)
Point.requires(Json<i32>)
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("distinct behavior specializations should not overlap");
    }

    #[test]
    fn behavior_extends_cycle_is_error() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

Json.extends(PrettyJson)
PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("cyclic behavior inheritance should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("behavior inheritance cycle")),
            "expected behavior inheritance cycle diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_duplicate_parent_is_error() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("duplicate behavior inheritance edge should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("duplicate behavior inheritance `PrettyJson.extends(Json)`")
            }),
            "expected duplicate behavior inheritance diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_duplicate_generic_parent_is_error() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json<str>)
PrettyJson.extends(Json<str>)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("duplicate specialized behavior inheritance edge should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("duplicate behavior inheritance `PrettyJson.extends(Json<str>)`")
            }),
            "expected duplicate generic behavior inheritance diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_generic_parent_without_type_args_is_error() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    encode: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior extends parent without type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic behavior `Json` expects 1 type arguments, found 0")),
            "expected generic behavior extends parent arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_extends_conflicting_method_signature_is_error() {
        let program = parse_program(
            r#"
Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    to_json: (Self) i32
}

PrettyJson.extends(Json)
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("conflicting inherited behavior method should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("conflicting behavior method `to_json` inherited by `PrettyJson`")
            }),
            "expected conflicting inherited behavior method diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_impl_signature_mismatch_is_error() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: i32) i32 { return value }
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("behavior impl signature mismatch should fail");
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("parameter 1 for method `to_json`")),
            "expected behavior parameter mismatch diagnostic, got {errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|d| d.message.contains("expects return `str`, found `i32`")),
            "expected behavior return mismatch diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_function_explicit_type_arg_arity_is_error() {
        let program = parse_program(
            r#"
identity<T> = (value: T) T {
    return value
}

main = () i32 {
    return identity<i32, str>(1)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("wrong generic type-argument arity should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("generic function `identity` expects 1 type arguments, found 2")),
            "expected generic arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn nongeneric_function_explicit_type_args_are_error() {
        let program = parse_program(
            r#"
id = (value: i32) i32 {
    return value
}

main = () i32 {
    return id<i32>(1)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("non-generic function type arguments should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("non-generic function `id` does not accept type arguments")),
            "expected non-generic type-argument diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_function_inference_failure_is_error() {
        let program = parse_program(
            r#"
make_default<T> = () T {
    return 0
}

main = () i32 {
    return make_default()
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("uninferred generic type argument should fail");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("cannot infer type argument `T` for generic function `make_default`")),
            "expected generic inference diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_bound_references_unknown_behavior_is_error() {
        let program = parse_program(
            r#"
show<T: Display> = (value: T) T {
    return value
}

main = () i32 {
    return show(1)
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown generic behavior bounds should fail");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "generic bound `Display` on type parameter `T` references undefined behavior"
            )),
            "expected generic bound diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_bound_rejects_unspecialized_generic_behavior() {
        let program = parse_program(
            r#"
Json<T>: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("generic behavior bound without type arguments should fail");
        assert!(
            errors.iter().any(|d| {
                d.message
                    .contains("generic behavior `Json` expects 1 type arguments, found 0")
            }),
            "expected generic behavior bound arity diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_behavior_bound_with_type_args_accepts_matching_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<Point>) {
    encode = (value: Point) Point { return value }
}

identity<T: Json<T>> = (value: T) T {
    return value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    return same.x
}
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("generic behavior bound type argument should substitute at call site");
    }

    #[test]
    fn generic_behavior_bound_with_type_args_rejects_mismatched_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { return "point" }
}

identity<T: Json<T>> = (value: T) T {
    return value
}

main = () i32 {
    p = Point { x: 1 }
    same = identity(p)
    return same.x
}
"#,
        );

        let errors = TypeChecker::new()
            .check_program(&program)
            .expect_err("generic behavior bound should require matching behavior type args");
        assert!(
            errors.iter().any(|d| d.message.contains(
                "type `Point` does not implement behavior `Json<Point>` required by `T`"
            )),
            "expected generic behavior bound type argument diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn behavior_generic_bound_accepts_later_behavior_declaration() {
        let program = parse_program(
            r#"
Serializable<T: Json>: behavior {
    encode: (Self) str
}

Json: behavior {
    to_json: (Self) str
}

main = () i32 {
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("behavior generic bounds should be independent of declaration order");
    }

    #[test]
    fn behavior_generic_bound_unknown_behavior_reports_once() {
        let program = parse_program(
            r#"
Serializable<T: Missing>: behavior {
    encode: (Self) str
}

main = () i32 {
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("unknown behavior generic bound should fail");
        let count = errors
            .iter()
            .filter(|d| {
                d.message.contains(
                    "generic bound `Missing` on type parameter `T` references undefined behavior",
                )
            })
            .count();
        assert_eq!(
            count, 1,
            "expected one behavior generic bound diagnostic, got {errors:?}"
        );
    }

    #[test]
    fn generic_behavior_bound_accepts_type_with_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

Point.implements(Json) {
    to_json = (value: Point) str { return "point" }
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        tc.check_program(&program)
            .expect("type with behavior impl should satisfy generic bound");
    }

    #[test]
    fn generic_behavior_bound_accepts_inherited_behavior_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)

Point.implements(PrettyJson) {
    to_json = (value: Point) str { return "point" }
    pretty = (value: Point) str { return "pretty" }
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    return 0
}
"#,
        );

        TypeChecker::new()
            .check_program(&program)
            .expect("child behavior impl should satisfy inherited generic bound");
    }

    #[test]
    fn generic_behavior_bound_rejects_type_without_impl() {
        let program = parse_program(
            r#"
Point: { x: i32 }

Json: behavior {
    to_json: (Self) str
}

encode<T: Json> = (value: T) str {
    return "encoded"
}

main = () i32 {
    p = Point { x: 1 }
    encoded = encode(p)
    return 0
}
"#,
        );

        let mut tc = TypeChecker::new();
        let errors = tc
            .check_program(&program)
            .expect_err("type without behavior impl should not satisfy generic bound");
        assert!(
            errors.iter().any(|d| d
                .message
                .contains("type `Point` does not implement behavior `Json`")),
            "expected missing generic bound impl diagnostic, got {errors:?}"
        );
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
