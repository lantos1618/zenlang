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
mod environment;
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

pub use environment::{BehaviorBound, BehaviorInfo, EnumInfo, FuncInfo, StructInfo};
pub(crate) use environment::{
    GenericFunctionTemplate, SourceModuleDependencies, TemplateDependencyEntry,
    TemplateDependencyState,
};

struct ResolverCallableSignature<'a> {
    parameter_names: &'a [String],
    parameter_types: &'a [AstType],
    return_type: &'a AstType,
}

struct ResolverTypeParameterMetadata<'a> {
    names: &'a [String],
    bound_refs: &'a [TypeParameterBoundRefMetadata],
}

enum ResolverCallableDeclarationMetadataTask<'a> {
    Function {
        name: &'a str,
        span: Span,
    },
    Method {
        type_name: &'a str,
        method_name: &'a str,
        span: Span,
    },
    TypeImpl {
        type_name: &'a str,
        methods: &'a [Declaration],
    },
}

enum CallableDeclarationTask<'a> {
    Function {
        name: &'a str,
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
    Method {
        type_name: &'a str,
        method_name: &'a str,
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
}

enum ResolverTypeDeclarationMetadataTask<'a> {
    Struct {
        name: &'a str,
        fields: &'a [StructField],
        span: Span,
    },
    Enum {
        name: &'a str,
        span: Span,
    },
}

enum AstTypeDeclarationTask<'a> {
    Struct {
        name: &'a str,
        type_params: &'a [ast::TypeParam],
        fields: &'a [StructField],
    },
    Enum {
        name: &'a str,
        type_params: &'a [ast::TypeParam],
        variants: &'a [EnumVariant],
    },
}

struct BehaviorDeclarationTask<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    methods: &'a [BehaviorMethod],
}

struct AstImportDeclarationTask<'a> {
    names: &'a [String],
    module_path: &'a [String],
}

#[derive(Default)]
struct AstDeclarationCollectionTasks<'a> {
    behaviors: Vec<BehaviorDeclarationTask<'a>>,
    types: Vec<AstTypeDeclarationTask<'a>>,
    callable: Vec<CallableDeclarationTask<'a>>,
    impl_blocks: Vec<ImplBlockDeclarationTask<'a>>,
    imports: Vec<AstImportDeclarationTask<'a>>,
    precollection_validations: AstPrecollectionValidationTasks<'a>,
}

struct AstStructFieldDefaultValidationTask<'a> {
    type_params: &'a [ast::TypeParam],
    fields: &'a [StructField],
}

enum AstTypeReferenceValidationTask<'a> {
    Struct {
        type_params: &'a [ast::TypeParam],
        fields: &'a [StructField],
    },
    Enum {
        type_params: &'a [ast::TypeParam],
        variants: &'a [EnumVariant],
    },
    Function {
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
    },
    Method {
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
    },
    Behavior {
        type_params: &'a [ast::TypeParam],
        methods: &'a [BehaviorMethod],
    },
    ImplBlock {
        methods: &'a [Declaration],
    },
    TopLevelExpr {
        expr: &'a Expression,
    },
}

enum SelfTypeContextValidationTask<'a> {
    Struct {
        fields: &'a [StructField],
    },
    Enum {
        variants: &'a [EnumVariant],
    },
    Function {
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
    Method {
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
    Behavior {
        methods: &'a [BehaviorMethod],
    },
    ImplBlock {
        behavior_type_args: &'a [AstType],
        methods: &'a [Declaration],
        span: Span,
    },
    Requires {
        behavior_type_args: &'a [AstType],
        span: Span,
    },
    BehaviorExtends {
        parent_type_args: &'a [AstType],
        span: Span,
    },
    TopLevelExpr {
        expr: &'a Expression,
    },
}

enum ResolverTypeReferenceValidationTask<'a> {
    Struct {
        name: &'a str,
        fields: &'a [StructField],
        span: Span,
    },
    Enum {
        name: &'a str,
        span: Span,
    },
    Function {
        name: &'a str,
        body: &'a Expression,
        span: Span,
    },
    Method {
        type_name: &'a str,
        method_name: &'a str,
        body: &'a Expression,
        span: Span,
    },
    Behavior {
        name: &'a str,
        methods: &'a [BehaviorMethod],
        span: Span,
    },
    ImplBlock {
        type_name: &'a str,
        methods: &'a [Declaration],
    },
    TopLevelExpr {
        expr: &'a Expression,
    },
}

struct ResolverBehaviorDeclarationMetadataTask<'a> {
    name: &'a str,
    span: Span,
}

#[derive(Default)]
struct BehaviorAssociationValidationTasks<'a> {
    extends: Vec<BehaviorExtendsValidationTask<'a>>,
    impls: Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    requires: Vec<BehaviorRequiresValidationTask<'a>>,
}

#[derive(Default)]
struct AstDeclarationValidationTasks<'a> {
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
    type_references: Vec<AstTypeReferenceValidationTask<'a>>,
    struct_field_defaults: Vec<AstStructFieldDefaultValidationTask<'a>>,
}

#[derive(Default)]
struct AstPrecollectionValidationTasks<'a> {
    self_type_contexts: Vec<SelfTypeContextValidationTask<'a>>,
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
}

trait BehaviorAssociationValidationTaskSource<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a>;
}

impl<'a> BehaviorAssociationValidationTaskSource<'a> for BehaviorAssociationValidationTasks<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        self
    }
}

impl<'a> BehaviorAssociationValidationTaskSource<'a> for AstDeclarationValidationTasks<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        &self.behavior_associations
    }
}

#[derive(Default)]
struct ResolverDeclarationMetadataTasks<'a> {
    callable: Vec<ResolverCallableDeclarationMetadataTask<'a>>,
    types: Vec<ResolverTypeDeclarationMetadataTask<'a>>,
    behaviors: Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
    type_references: Vec<ResolverTypeReferenceValidationTask<'a>>,
}

impl<'a> BehaviorAssociationValidationTaskSource<'a> for ResolverDeclarationMetadataTasks<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        &self.behavior_associations
    }
}

struct ResolverBehaviorImplBlockDeclarationTask<'a> {
    ast_type_name: &'a str,
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
    span: Span,
}

struct ResolverBehaviorImplBlockTask<'a> {
    ast_type_name: &'a str,
    restored_type_name: String,
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
}

struct ImplBlockDeclarationTask<'a> {
    type_name: &'a str,
    behavior: Option<&'a str>,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
}

struct BehaviorRequiresValidationTask<'a> {
    type_name: &'a str,
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    span: Span,
}

struct EffectiveBehaviorImplMethod<'a> {
    declaration: &'a Declaration,
    method_name: String,
}

struct BehaviorExtendsValidationTask<'a> {
    behavior: &'a str,
    parent: &'a str,
    parent_type_args: &'a [AstType],
    span: Span,
}

struct ResolverTypeBehaviorRefreshTask {
    restored_name: String,
}

struct ResolverTypeBehaviorAssociationListTask<'a> {
    symbol: &'a Symbol,
    name: &'a str,
    impl_edges: Vec<ExpectedBehaviorEdge>,
    required_edges: Vec<ExpectedBehaviorEdge>,
    span: Span,
}

struct ResolverBehaviorParentListTask<'a> {
    symbol: &'a Symbol,
    name: &'a str,
    parent_edges: Vec<ExpectedBehaviorEdge>,
    span: Span,
}

#[derive(Default)]
struct ResolverBehaviorAssociationListTasks<'a> {
    type_associations: Vec<ResolverTypeBehaviorAssociationListTask<'a>>,
    behavior_parents: Vec<ResolverBehaviorParentListTask<'a>>,
}

#[derive(Default)]
struct ResolverExpectedSymbolSets {
    declarations: HashSet<(Namespace, String)>,
    locals: HashSet<(String, u32)>,
    validate_imports: bool,
}

#[derive(Default)]
struct ResolverValidationReplayTasks<'a> {
    expected_symbols: ResolverExpectedSymbolSets,
    behavior_associations: ResolverBehaviorAssociationListTasks<'a>,
}

struct ResolverValidationBehaviorAssociationSource<'a> {
    name: &'a str,
    symbol: &'a Symbol,
    span: Span,
}

struct ResolverValidationReplayDeclarationTasks<'a> {
    expected_symbols: ResolverExpectedSymbolSets,
    expected_associations: ExpectedBehaviorAssociations,
    expected_parents: ExpectedBehaviorEdges,
    type_declarations: Vec<ResolverValidationBehaviorAssociationSource<'a>>,
    behavior_declarations: Vec<ResolverValidationBehaviorAssociationSource<'a>>,
}

impl Default for ResolverValidationReplayDeclarationTasks<'_> {
    fn default() -> Self {
        Self {
            expected_symbols: ResolverExpectedSymbolSets::default(),
            expected_associations: ExpectedBehaviorAssociations {
                impls: ExpectedBehaviorEdges::default(),
                required: ExpectedBehaviorEdges::default(),
            },
            expected_parents: ExpectedBehaviorEdges::default(),
            type_declarations: Vec::new(),
            behavior_declarations: Vec::new(),
        }
    }
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

trait AbsentMetadataValidation<const N: usize>: Copy {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; N];
}

impl AbsentMetadataValidation<4> for TypeParameterAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        TypeParameterAbsenceValidation::entries(self, symbol)
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

impl AbsentMetadataValidation<6> for ValueSignatureAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 6] {
        ValueSignatureAbsenceValidation::entries(self, symbol)
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

impl AbsentMetadataValidation<3> for FieldAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 3] {
        FieldAbsenceValidation::entries(self, symbol)
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

impl AbsentMetadataValidation<5> for VariantAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 5] {
        VariantAbsenceValidation::entries(self, symbol)
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

impl AbsentMetadataValidation<4> for BehaviorAssociationAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        BehaviorAssociationAbsenceValidation::entries(self, symbol)
    }
}

#[derive(Clone, Copy)]
struct BehaviorDeclarationAbsenceValidation {
    method_signature_code: &'static str,
    method_type_code: &'static str,
    parent_name_code: &'static str,
    parent_ref_code: &'static str,
}

impl AbsentMetadataValidation<4> for BehaviorDeclarationAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 4] {
        BehaviorDeclarationAbsenceValidation::entries(self, symbol)
    }
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

    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 1] {
        [AbsentMetadataEntry::new(
            symbol.is_mutable.is_some(),
            self.code,
            "mutability",
        )]
    }
}

impl AbsentMetadataValidation<1> for MutabilityAbsenceValidation {
    fn entries(self, symbol: &Symbol) -> [AbsentMetadataEntry; 1] {
        MutabilityAbsenceValidation::entries(self, symbol)
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
    resolver_type_parameter_metadata(symbol)
        .map(|metadata| type_param_bounds_from_resolver_refs(metadata.bound_refs))
        .unwrap_or_default()
}

fn resolver_type_param_names(symbol: &crate::resolver::Symbol) -> Vec<String> {
    resolver_type_parameter_metadata(symbol)
        .map(|metadata| metadata.names.to_vec())
        .unwrap_or_default()
}

fn resolver_type_parameter_metadata(
    symbol: &crate::resolver::Symbol,
) -> Option<ResolverTypeParameterMetadata<'_>> {
    Some(ResolverTypeParameterMetadata {
        names: symbol.type_parameter_names.as_deref()?,
        bound_refs: symbol.type_parameter_bound_refs.as_deref()?,
    })
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

#[derive(Clone)]
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
    #[cfg(test)]
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

    #[cfg(test)]
    fn edges_for(&self, owner: &str) -> &[ExpectedBehaviorEdge] {
        self.edges.get(owner).map(Vec::as_slice).unwrap_or(&[])
    }

    fn owned_edges_for(&self, owner: &str) -> Vec<ExpectedBehaviorEdge> {
        self.edges.get(owner).cloned().unwrap_or_default()
    }
}

struct ExpectedBehaviorAssociations {
    impls: ExpectedBehaviorEdges,
    required: ExpectedBehaviorEdges,
}

#[cfg(test)]
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
                    push_expected_behavior_impl_edge(
                        &mut expected,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    ..
                } => {
                    push_expected_behavior_required_edge(
                        &mut expected,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
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
        let tasks = Self::collect_ast_declaration_collection_tasks(decls);
        self.collect_behavior_declarations_from_tasks(&tasks.behaviors);
        self.validate_ast_precollection_tasks(&tasks.precollection_validations);
        if !self.resolver_backed_collection {
            self.collect_ast_type_declarations_from_tasks(&tasks.types);
        }
        self.collect_callable_declarations_from_tasks(&tasks.callable);
        self.collect_impl_block_declarations_from_tasks(&tasks.impl_blocks);
        self.collect_ast_import_declarations_from_tasks(&tasks.imports);
    }

    fn collect_ast_declaration_collection_tasks(
        decls: &[Declaration],
    ) -> AstDeclarationCollectionTasks<'_> {
        let mut tasks = AstDeclarationCollectionTasks::default();
        for decl in decls {
            Self::push_behavior_declaration_task(decl, &mut tasks.behaviors);
            Self::push_ast_type_declaration_task(decl, &mut tasks.types);
            Self::push_callable_declaration_task(decl, &mut tasks.callable);
            Self::push_impl_block_declaration_task(decl, &mut tasks.impl_blocks);
            Self::push_ast_import_declaration_task(decl, &mut tasks.imports);
            Self::push_self_type_context_validation_task(
                decl,
                &mut tasks.precollection_validations.self_type_contexts,
            );
            Self::push_behavior_extends_replay_task(
                decl,
                &mut tasks
                    .precollection_validations
                    .behavior_associations
                    .extends,
            );
        }
        tasks
    }

    #[cfg(test)]
    fn collect_ast_import_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<AstImportDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_import_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_import_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstImportDeclarationTask<'a>>,
    ) {
        if let Declaration::Import {
            names, module_path, ..
        } = decl
        {
            tasks.push(AstImportDeclarationTask { names, module_path });
        }
    }

    fn collect_ast_import_declarations_from_tasks(
        &mut self,
        tasks: &[AstImportDeclarationTask<'_>],
    ) {
        for task in tasks {
            for name in task.names {
                self.imports.insert(name.clone(), task.module_path.to_vec());
            }
        }
    }

    #[cfg(test)]
    fn collect_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ImplBlockDeclarationTask<'a>>,
    ) {
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            behavior_type_args,
            methods,
            ..
        } = decl
        {
            tasks.push(ImplBlockDeclarationTask {
                type_name,
                behavior: behavior.as_deref(),
                behavior_type_args,
                methods,
            });
        }
    }

    fn collect_impl_block_declarations_from_tasks(
        &mut self,
        tasks: &[ImplBlockDeclarationTask<'_>],
    ) {
        for task in tasks {
            if self.resolver_backed_collection {
                self.collect_resolver_backed_impl_block_templates(task.type_name, task.methods);
            } else {
                self.collect_ast_impl_block_declaration(
                    task.type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }
        }
    }

    fn collect_ast_impl_block_declaration(
        &mut self,
        type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
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

    fn collect_resolver_backed_impl_block_templates(
        &mut self,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            self.collect_resolver_backed_impl_method_template(type_name, method);
        }
    }

    #[cfg(test)]
    fn collect_callable_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<CallableDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_callable_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_callable_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<CallableDeclarationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(CallableDeclarationTask::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(CallableDeclarationTask::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            }),
            _ => {}
        }
    }

    fn collect_callable_declarations_from_tasks(&mut self, tasks: &[CallableDeclarationTask<'_>]) {
        for task in tasks {
            match task {
                CallableDeclarationTask::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    if self.resolver_backed_collection {
                        self.collect_resolver_backed_function_template(
                            name,
                            type_params,
                            params,
                            body,
                            *span,
                        );
                    } else {
                        self.collect_ast_function_declaration(
                            name,
                            type_params,
                            params,
                            return_type,
                            body,
                            *span,
                        );
                    }
                }
                CallableDeclarationTask::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    if self.resolver_backed_collection {
                        self.collect_resolver_backed_method_template(
                            type_name,
                            method_name,
                            type_params,
                            params,
                            body,
                            *span,
                        );
                    } else {
                        let key = Self::method_key(type_name, method_name);
                        self.collect_ast_method_declaration(
                            &key,
                            type_params,
                            params,
                            return_type,
                            body,
                            *span,
                        );
                    }
                }
            }
        }
    }

    fn collect_ast_function_declaration(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) {
        self.validate_generic_bounds(type_params);
        self.functions.insert(
            name.to_string(),
            func_info_from_ast_signature(name.to_string(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, span)
        {
            self.generic_functions.insert(name.to_string(), template);
        }
    }

    fn collect_ast_method_declaration(
        &mut self,
        key: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) {
        self.validate_generic_bounds(type_params);
        self.methods.insert(
            key.to_string(),
            func_info_from_ast_signature(key.to_string(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, span)
        {
            self.generic_methods.insert(key.to_string(), template);
        }
    }

    fn collect_resolver_backed_function_template(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        body: &Expression,
        span: Span,
    ) {
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, span)
        {
            self.generic_functions.insert(name.to_string(), template);
        }
    }

    fn collect_resolver_backed_method_template(
        &mut self,
        type_name: &str,
        method_name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        body: &Expression,
        span: Span,
    ) {
        if let Some(template) =
            generic_template_body_stub_from_type_params(type_params, params, body, span)
        {
            self.generic_methods
                .insert(Self::method_key(type_name, method_name), template);
        }
    }

    #[cfg(test)]
    fn collect_ast_type_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_type_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeDeclarationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeDeclarationTask::Struct {
                name,
                type_params,
                fields,
            }),
            Declaration::Enum {
                name,
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeDeclarationTask::Enum {
                name,
                type_params,
                variants,
            }),
            _ => {}
        }
    }

    fn collect_ast_type_declarations_from_tasks(&mut self, tasks: &[AstTypeDeclarationTask<'_>]) {
        for task in tasks {
            match task {
                AstTypeDeclarationTask::Struct {
                    name,
                    type_params,
                    fields,
                } => {
                    self.validate_generic_bounds(type_params);
                    self.structs.insert(
                        (*name).to_string(),
                        struct_info_from_ast_fields((*name).to_string(), type_params, fields),
                    );
                }
                AstTypeDeclarationTask::Enum {
                    name,
                    type_params,
                    variants,
                } => {
                    self.validate_generic_bounds(type_params);
                    self.enums.insert(
                        (*name).to_string(),
                        enum_info_from_ast_variants((*name).to_string(), type_params, variants),
                    );
                }
            }
        }
    }

    #[cfg(test)]
    fn collect_behavior_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<BehaviorDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_behavior_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<BehaviorDeclarationTask<'a>>,
    ) {
        if let Declaration::Behavior {
            name,
            type_params,
            methods,
            ..
        } = decl
        {
            tasks.push(BehaviorDeclarationTask {
                name,
                type_params,
                methods,
            });
        }
    }

    fn collect_behavior_declarations_from_tasks(&mut self, tasks: &[BehaviorDeclarationTask<'_>]) {
        let mut type_params_to_validate = Vec::new();

        for task in tasks {
            let BehaviorDeclarationTask {
                name,
                type_params,
                methods,
            } = task;

            if self.resolver_backed_collection {
                self.collect_resolver_backed_behavior_declaration_stub(name, methods);
            } else {
                self.collect_ast_behavior_declaration_signature(name, type_params, methods);
                type_params_to_validate.push(type_params);
            }
        }

        for type_params in type_params_to_validate {
            self.validate_generic_bounds(type_params);
        }
    }

    fn collect_ast_behavior_declaration_signature(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        methods: &[BehaviorMethod],
    ) {
        self.behaviors.insert(
            name.to_string(),
            behavior_info_from_ast_methods(name.to_string(), type_params, methods),
        );
    }

    fn collect_resolver_backed_behavior_declaration_stub(
        &mut self,
        name: &str,
        methods: &[BehaviorMethod],
    ) {
        self.behaviors.insert(
            name.to_string(),
            behavior_info_for_resolver_backed_stub(name.to_string(), methods),
        );
    }

    fn validate_ast_precollection_tasks(&mut self, tasks: &AstPrecollectionValidationTasks<'_>) {
        self.validate_self_type_context_tasks(&tasks.self_type_contexts);

        if self.resolver_backed_collection {
            return;
        }

        self.validate_ast_behavior_extends_tasks(&tasks.behavior_associations);
    }

    #[cfg(test)]
    fn collect_ast_precollection_validation_tasks(
        decls: &[Declaration],
    ) -> AstPrecollectionValidationTasks<'_> {
        let mut tasks = AstPrecollectionValidationTasks::default();
        for decl in decls {
            Self::push_self_type_context_validation_task(decl, &mut tasks.self_type_contexts);
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
        }
        tasks
    }

    fn validate_ast_behavior_extends_tasks(
        &mut self,
        tasks: &BehaviorAssociationValidationTasks<'_>,
    ) {
        self.validate_behavior_extends_tasks(tasks);
        self.validate_behavior_extends_cycles();
        self.validate_behavior_method_coherence();
    }

    fn push_behavior_extends_replay_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<BehaviorExtendsValidationTask<'a>>,
    ) -> bool {
        if let Declaration::BehaviorExtends {
            behavior,
            parent,
            parent_type_args,
            span,
        } = decl
        {
            tasks.push(BehaviorExtendsValidationTask {
                behavior,
                parent,
                parent_type_args,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    fn validate_behavior_extends_tasks(&mut self, tasks: &BehaviorAssociationValidationTasks<'_>) {
        for task in &tasks.extends {
            self.check_behavior_extends(
                task.behavior,
                task.parent,
                task.parent_type_args,
                task.span,
            );
        }
    }

    fn collect_declarations_with_symbols(&mut self, decls: &[Declaration], symbols: &SymbolTable) {
        self.with_resolver_backed_collection(|checker| checker.collect_declarations(decls));

        let tasks = Self::collect_resolver_declaration_metadata_tasks(decls);
        self.collect_resolver_declaration_metadata(symbols, &tasks);
        self.collect_resolver_behavior_impl_metadata(&tasks, symbols);
        self.validate_resolver_collected_declaration_semantics(symbols, &tasks);
        self.clear_resolver_behavior_ref_state();
        self.refresh_resolver_type_behavior_impls(&tasks, symbols);
    }

    fn collect_resolver_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationMetadataTasks<'_> {
        let mut tasks = ResolverDeclarationMetadataTasks::default();
        for decl in decls {
            let callable_handled = Self::push_resolver_callable_replay_tasks(
                decl,
                &mut tasks.callable,
                &mut tasks.type_references,
            );
            let type_handled = if callable_handled {
                false
            } else {
                Self::push_resolver_type_replay_tasks(
                    decl,
                    &mut tasks.types,
                    &mut tasks.type_references,
                )
            };
            let behavior_handled = if callable_handled || type_handled {
                false
            } else {
                Self::push_resolver_behavior_replay_tasks(
                    decl,
                    &mut tasks.behaviors,
                    &mut tasks.type_references,
                )
            };
            let behavior_impl_handled = if callable_handled || type_handled || behavior_handled {
                false
            } else {
                Self::push_resolver_behavior_impl_replay_tasks(
                    decl,
                    &mut tasks.behavior_associations.impls,
                    &mut tasks.type_references,
                )
            };
            if !callable_handled && !type_handled && !behavior_handled && !behavior_impl_handled {
                Self::push_resolver_type_reference_validation_task(
                    decl,
                    &mut tasks.type_references,
                );
            }
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
            Self::push_behavior_requires_replay_task(
                decl,
                &mut tasks.behavior_associations.requires,
            );
        }
        tasks
    }

    #[cfg(test)]
    fn collect_resolver_type_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_type_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
            }
            _ => {}
        }
    }

    fn push_resolver_type_replay_tasks<'a>(
        decl: &'a Declaration,
        type_tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                true
            }
            Declaration::Enum { name, span, .. } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn collect_resolver_behavior_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_behavior_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_behavior_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
    ) {
        if let Declaration::Behavior { name, span, .. } = decl {
            tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
        }
    }

    fn push_resolver_behavior_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Behavior {
            name,
            methods,
            span,
            ..
        } = decl
        {
            behavior_tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
            type_reference_tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                name,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    fn push_resolver_behavior_impl_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_impl_tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        let handled = Self::push_behavior_impl_block_declaration_task(decl, behavior_impl_tasks);
        if handled {
            let Declaration::ImplBlock {
                type_name, methods, ..
            } = decl
            else {
                return false;
            };
            type_reference_tasks
                .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
        }
        handled
    }

    #[cfg(test)]
    fn collect_resolver_behavior_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_behavior_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    ) -> bool {
        if let Declaration::ImplBlock {
            type_name,
            behavior: Some(behavior),
            behavior_type_args,
            methods,
            span,
            ..
        } = decl
        {
            tasks.push(ResolverBehaviorImplBlockDeclarationTask {
                ast_type_name: type_name,
                behavior,
                behavior_type_args,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    fn collect_resolver_callable_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverCallableDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_callable_declaration_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    #[cfg(test)]
    fn push_resolver_callable_declaration_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
    ) {
        match decl {
            Declaration::Function { name, span, .. } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
            }
            Declaration::Method {
                type_name,
                method_name,
                span,
                ..
            } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
            }
            _ => {}
        }
    }

    fn push_resolver_callable_replay_tasks<'a>(
        decl: &'a Declaration,
        callable_tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                callable_tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn collect_resolver_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_resolver_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
            }
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
            }
            Declaration::Behavior {
                name,
                methods,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span: *span,
                });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    fn collect_resolver_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        self.collect_resolver_callable_declaration_metadata(symbols, tasks);
        self.collect_resolver_type_declaration_metadata(symbols, tasks);
        self.collect_resolver_behavior_declaration_metadata_pass(symbols, tasks);
    }

    fn collect_resolver_behavior_declaration_metadata_pass(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.behaviors {
            self.collect_resolver_behavior_declaration(symbols, task.name, task.span);
        }
    }

    fn collect_resolver_type_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.types {
            match task {
                ResolverTypeDeclarationMetadataTask::Struct { name, fields, span } => {
                    self.collect_resolver_struct_declaration_metadata(symbols, name, fields, *span);
                }
                ResolverTypeDeclarationMetadataTask::Enum { name, span } => {
                    self.collect_resolver_enum_declaration_metadata(symbols, name, *span);
                }
            }
        }
    }

    fn collect_resolver_callable_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        for task in &tasks.callable {
            match task {
                ResolverCallableDeclarationMetadataTask::Function { name, span } => {
                    self.collect_resolver_function_signature(symbols, name, *span);
                }
                ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span,
                } => {
                    self.collect_resolver_method_signature(symbols, type_name, method_name, *span);
                }
                ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods } => {
                    self.collect_resolver_type_impl_declaration_metadata(
                        symbols, type_name, methods,
                    );
                }
            }
        }
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
        self.collect_resolver_type_declaration_metadata_for(
            symbols,
            name,
            span,
            |checker, name| {
                checker.collect_resolver_struct_fields(symbols, name, fields);
            },
        );
    }

    fn collect_resolver_enum_declaration_metadata(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
    ) {
        self.collect_resolver_type_declaration_metadata_for(
            symbols,
            name,
            span,
            |checker, name| {
                checker.collect_resolver_enum_variants(symbols, name);
            },
        );
    }

    fn collect_resolver_type_declaration_metadata_for(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
        collect: impl FnOnce(&mut Self, &str),
    ) {
        let restored_name =
            self.collect_resolver_type_behavior_refs_for_declaration(symbols, name, span);
        collect(self, &restored_name);
    }

    fn collect_resolver_behavior_impl_metadata(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let impl_tasks = self.resolver_behavior_impl_block_tasks(tasks, symbols);

        self.with_resolver_backed_collection(|checker| {
            for task in &impl_tasks {
                checker.collect_resolver_behavior_impl_method_signatures(
                    symbols,
                    task.ast_type_name,
                    &task.restored_type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }

            checker.validate_collected_behavior_extends_semantics();

            for task in &impl_tasks {
                checker.collect_behavior_default_method_signatures(
                    &task.restored_type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }
        });
    }

    fn validate_resolver_collected_declaration_semantics(
        &mut self,
        symbols: &SymbolTable,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
    ) {
        self.with_resolver_backed_collection(|checker| {
            checker.validate_behavior_association_tasks(tasks, Some(symbols));
            checker.validate_resolver_type_reference_tasks(tasks, Some(symbols));
            checker.validate_resolver_struct_field_default_tasks(tasks, Some(symbols));
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
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) {
        for task in self.resolver_type_behavior_refresh_tasks(tasks, symbols) {
            self.collect_resolver_type_behavior_impls(symbols, &task.restored_name);
        }
    }

    fn with_resolver_backed_collection(&mut self, collect: impl FnOnce(&mut Self)) {
        let previous = self.resolver_backed_collection;
        self.resolver_backed_collection = true;
        collect(self);
        self.resolver_backed_collection = previous;
    }

    fn resolver_behavior_impl_block_tasks<'a>(
        &self,
        tasks: &'a ResolverDeclarationMetadataTasks<'a>,
        symbols: &SymbolTable,
    ) -> Vec<ResolverBehaviorImplBlockTask<'a>> {
        let mut impl_tasks = Vec::new();
        for raw_task in &tasks.behavior_associations.impls {
            let restored_type_name = self.resolver_impl_type_name_for(
                symbols,
                raw_task.ast_type_name,
                raw_task.methods,
                Some((raw_task.behavior, raw_task.behavior_type_args)),
            );
            impl_tasks.push(ResolverBehaviorImplBlockTask {
                ast_type_name: raw_task.ast_type_name,
                restored_type_name,
                behavior: raw_task.behavior,
                behavior_type_args: raw_task.behavior_type_args,
                methods: raw_task.methods,
            });
        }
        impl_tasks
    }

    fn resolver_type_behavior_refresh_tasks(
        &self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: &SymbolTable,
    ) -> Vec<ResolverTypeBehaviorRefreshTask> {
        let mut refresh_tasks = Vec::new();
        for type_task in &tasks.types {
            match type_task {
                ResolverTypeDeclarationMetadataTask::Struct { name, span, .. }
                | ResolverTypeDeclarationMetadataTask::Enum { name, span } => {
                    let restored_name =
                        Self::resolver_symbol_name_for(symbols, Namespace::Type, name, *span);
                    refresh_tasks.push(ResolverTypeBehaviorRefreshTask { restored_name });
                }
            }
        }
        refresh_tasks
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
        if self.resolver_backed_collection {
            let tasks = Self::collect_resolver_semantic_validation_tasks(decls);
            self.validate_behavior_association_tasks(&tasks, symbols);
            self.validate_resolver_type_reference_tasks(&tasks, symbols);
            self.validate_resolver_struct_field_default_tasks(&tasks, symbols);
            return;
        }

        let tasks = Self::collect_ast_declaration_validation_tasks(decls);
        self.validate_behavior_association_tasks(&tasks.behavior_associations, symbols);
        self.validate_ast_type_reference_tasks(&tasks.type_references);
        self.validate_ast_struct_field_default_tasks(&tasks.struct_field_defaults);
    }

    fn collect_ast_declaration_validation_tasks(
        decls: &[Declaration],
    ) -> AstDeclarationValidationTasks<'_> {
        let mut tasks = AstDeclarationValidationTasks::default();
        for decl in decls {
            Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
            Self::push_behavior_impl_block_declaration_task(
                decl,
                &mut tasks.behavior_associations.impls,
            );
            Self::push_behavior_requires_replay_task(
                decl,
                &mut tasks.behavior_associations.requires,
            );
            Self::push_ast_type_reference_validation_task(decl, &mut tasks.type_references);
            Self::push_ast_struct_field_default_validation_task(
                decl,
                &mut tasks.struct_field_defaults,
            );
        }
        tasks
    }

    #[cfg(test)]
    fn collect_behavior_association_validation_tasks(
        decls: &[Declaration],
    ) -> BehaviorAssociationValidationTasks<'_> {
        let mut tasks = BehaviorAssociationValidationTasks::default();
        for decl in decls {
            Self::push_behavior_extends_replay_task(decl, &mut tasks.extends);
            Self::push_behavior_impl_block_declaration_task(decl, &mut tasks.impls);
            Self::push_behavior_requires_replay_task(decl, &mut tasks.requires);
        }
        tasks
    }

    fn collect_resolver_semantic_validation_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationMetadataTasks<'_> {
        Self::collect_resolver_declaration_metadata_tasks(decls)
    }

    fn validate_behavior_association_tasks<'a>(
        &mut self,
        tasks: &impl BehaviorAssociationValidationTaskSource<'a>,
        symbols: Option<&SymbolTable>,
    ) {
        let tasks = tasks.behavior_association_tasks();
        self.validate_behavior_impl_tasks(tasks, symbols);
        self.validate_behavior_requires_tasks(tasks, symbols);
    }

    fn validate_behavior_impl_tasks(
        &mut self,
        tasks: &BehaviorAssociationValidationTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.impls {
            self.validate_collected_behavior_impl_declaration(
                symbols,
                task.ast_type_name,
                task.behavior,
                task.behavior_type_args,
                task.methods,
                task.span,
            );
        }
    }

    fn push_behavior_requires_replay_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<BehaviorRequiresValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Requires {
            type_name,
            behavior,
            behavior_type_args,
            span,
        } = decl
        {
            tasks.push(BehaviorRequiresValidationTask {
                type_name,
                behavior,
                behavior_type_args,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    fn validate_behavior_requires_tasks(
        &mut self,
        tasks: &BehaviorAssociationValidationTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.requires {
            self.validate_collected_behavior_requires_declaration(
                symbols,
                task.type_name,
                task.behavior,
                task.behavior_type_args,
                task.span,
            );
        }
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

    #[cfg(test)]
    fn validate_struct_field_defaults(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            let tasks = Self::collect_resolver_declaration_metadata_tasks(decls);
            self.validate_resolver_struct_field_default_tasks(&tasks, symbols);
            return;
        }

        let tasks = Self::collect_ast_struct_field_default_validation_tasks(decls);
        self.validate_ast_struct_field_default_tasks(&tasks);
    }

    #[cfg(test)]
    fn collect_ast_struct_field_default_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstStructFieldDefaultValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_struct_field_default_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_struct_field_default_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstStructFieldDefaultValidationTask<'a>>,
    ) {
        if let Declaration::Struct {
            type_params,
            fields,
            ..
        } = decl
        {
            tasks.push(AstStructFieldDefaultValidationTask {
                type_params,
                fields,
            });
        }
    }

    fn validate_ast_struct_field_default_tasks(
        &mut self,
        tasks: &[AstStructFieldDefaultValidationTask<'_>],
    ) {
        for task in tasks {
            self.validate_ast_struct_field_defaults(!task.type_params.is_empty(), task.fields);
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

    fn validate_resolver_struct_field_default_tasks(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.types {
            if let ResolverTypeDeclarationMetadataTask::Struct { name, span, .. } = task {
                self.validate_resolver_struct_field_defaults(symbols, name, *span);
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
        let Some(signature) = Self::resolver_callable_signature_metadata(symbol) else {
            self.remove_callable_signature(name);
            return;
        };
        let info = func_info_from_resolver_signature(
            name.to_string(),
            symbol,
            signature.parameter_names,
            signature.parameter_types,
            signature.return_type,
        );
        self.insert_callable_signature(name, info);
        let type_parameter_names = resolver_type_param_names(symbol);
        self.collect_resolver_generic_template_signature(
            name,
            &type_parameter_names,
            signature.parameter_names,
            signature.parameter_types,
            signature.return_type,
        );
    }

    fn resolver_callable_signature_metadata(
        symbol: &Symbol,
    ) -> Option<ResolverCallableSignature<'_>> {
        Some(ResolverCallableSignature {
            parameter_names: symbol.parameter_names.as_deref()?,
            parameter_types: symbol.parameter_types.as_deref()?,
            return_type: symbol.return_type.as_ref()?,
        })
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
        let existing_params = template.params.clone();
        template.params = Self::resolver_params_from_metadata(
            &existing_params,
            parameter_names,
            parameter_types,
            template.span,
        );
        template.return_type = Self::resolver_optional_return_type(return_type);
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
                Self::resolver_struct_field_metadata(symbol)
            })
        else {
            self.structs.remove(name);
            return;
        };

        let (fields, field_defaults) =
            Self::resolver_struct_fields_from_metadata(field_types, ast_fields);
        self.structs.insert(
            name.to_string(),
            struct_info_from_resolver_fields(name.to_string(), symbol, fields, field_defaults),
        );
    }

    fn resolver_struct_field_metadata(symbol: &Symbol) -> Option<&[(String, AstType)]> {
        symbol.field_types.as_deref()
    }

    fn resolver_struct_fields_from_metadata(
        fields: &[(String, AstType)],
        ast_fields: &[StructField],
    ) -> (Vec<(String, AstType)>, HashMap<String, Expression>) {
        let field_defaults = ast_fields
            .iter()
            .zip(fields.iter())
            .filter_map(|(field, (restored_name, _))| {
                field
                    .default
                    .as_ref()
                    .map(|default| (restored_name.clone(), default.clone()))
            })
            .collect();
        (fields.to_vec(), field_defaults)
    }

    fn collect_resolver_enum_variants(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((symbol, variant_names)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Type, name, |symbol| {
                Self::resolver_enum_variant_name_metadata(symbol)
            })
        else {
            self.enums.remove(name);
            return;
        };

        let variants = Self::resolver_enum_variants_from_metadata(symbols, name, variant_names);
        self.enums.insert(
            name.to_string(),
            enum_info_from_resolver_variants(name.to_string(), symbol, variants),
        );
    }

    fn resolver_enum_variant_name_metadata(symbol: &Symbol) -> Option<&[String]> {
        symbol.variant_names.as_deref()
    }

    fn resolver_enum_variants_from_metadata(
        symbols: &SymbolTable,
        enum_name: &str,
        variant_names: &[String],
    ) -> Vec<(String, Option<AstType>)> {
        variant_names
            .iter()
            .map(|variant_name| {
                (
                    variant_name.clone(),
                    symbols
                        .lookup_variant(enum_name, variant_name)
                        .and_then(|variant| variant.variant_payload_type.clone()),
                )
            })
            .collect()
    }

    fn collect_resolver_behavior_methods(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((symbol, method_types)) =
            Self::resolver_symbol_metadata(symbols, Namespace::Behavior, name, |symbol| {
                Self::resolver_behavior_method_metadata(symbol)
            })
        else {
            self.behaviors.remove(name);
            return;
        };

        let Some(existing) = self.behaviors.get(name).cloned() else {
            return;
        };
        let methods = Self::resolver_behavior_methods_from_metadata(
            existing.methods,
            method_types,
            symbol.definition_span,
        );
        self.behaviors.insert(
            name.to_string(),
            behavior_info_from_resolver_methods(name.to_string(), symbol, methods),
        );
    }

    fn resolver_behavior_method_metadata(symbol: &Symbol) -> Option<&[BehaviorMethodTypeMetadata]> {
        symbol.behavior_method_types.as_deref()
    }

    fn resolver_behavior_methods_from_metadata(
        existing_methods: Vec<ast::BehaviorMethod>,
        method_types: &[BehaviorMethodTypeMetadata],
        span: Span,
    ) -> Vec<ast::BehaviorMethod> {
        let mut existing_methods: VecDeque<ast::BehaviorMethod> =
            existing_methods.into_iter().collect();
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
                span,
            ));
        }
        methods
    }

    fn resolver_behavior_method_from_metadata(
        existing_method: Option<&ast::BehaviorMethod>,
        metadata: BehaviorMethodTypeMetadata,
        span: Span,
    ) -> ast::BehaviorMethod {
        let params = Self::resolver_params_from_metadata(
            existing_method
                .map(|method| method.params.as_slice())
                .unwrap_or(&[]),
            &metadata.parameter_names,
            &metadata.parameter_types,
            Span::dummy(),
        );
        let return_type = Self::resolver_optional_return_type(&metadata.return_type);
        ast::BehaviorMethod {
            name: metadata.name,
            params,
            return_type,
            default_body: existing_method.and_then(|method| method.default_body.clone()),
            span: existing_method.map(|method| method.span).unwrap_or(span),
        }
    }

    fn resolver_params_from_metadata(
        existing_params: &[Param],
        parameter_names: &[String],
        parameter_types: &[AstType],
        default_span: Span,
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
                    span: default_span,
                },
            })
            .collect()
    }

    fn resolver_optional_return_type(return_type: &AstType) -> Option<AstType> {
        match return_type {
            AstType::Void => None,
            ty => Some(ty.clone()),
        }
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

        let parents = self.behavior_parent_refs_from_metadata(parent_refs);
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

        for implementation in self.behavior_impl_refs_from_metadata(name, impl_refs) {
            self.behavior_impls.insert(implementation);
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

    fn behavior_parent_refs_from_metadata(
        &self,
        metadata: &[BehaviorRefMetadata],
    ) -> Vec<BehaviorParentRef> {
        metadata
            .iter()
            .map(|parent| self.behavior_parent_ref_from_metadata(parent))
            .collect()
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

    fn behavior_impl_refs_from_metadata(
        &self,
        type_name: &str,
        metadata: &[BehaviorRefMetadata],
    ) -> Vec<(String, String)> {
        metadata
            .iter()
            .map(|behavior| {
                (
                    type_name.to_string(),
                    self.behavior_reference_key(&behavior.name, &behavior.type_args),
                )
            })
            .collect()
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
        self.resolver_behavior_ref_for(BehaviorRefRole::Required, type_name, behavior)
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
        let effective_methods = self.effective_behavior_impl_methods(
            symbols,
            type_name,
            methods,
            &mut unmatched_required,
        );

        for method in &effective_methods {
            if let Declaration::Function { span, .. } = method.declaration {
                if !required_methods
                    .iter()
                    .any(|required| required.name == method.method_name)
                {
                    self.diagnostics.push(Diagnostic::error(
                        "E6005",
                        format!(
                            "method `{}` is not declared by behavior `{}`",
                            method.method_name, behavior_key
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
                    .find_map(|method| match method.declaration {
                        Declaration::Function {
                            params,
                            return_type,
                            span,
                            ..
                        } if method.method_name == required.name => {
                            Some((params, return_type, *span))
                        }
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
        self.resolver_behavior_ref_for(BehaviorRefRole::Impl, type_name, behavior)
    }

    fn resolver_behavior_ref_for(
        &mut self,
        role: BehaviorRefRole,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        match role {
            BehaviorRefRole::Impl => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_impl_refs,
                type_name,
                behavior,
            ),
            BehaviorRefRole::Required => Self::pop_resolver_behavior_ref(
                self.resolver_backed_collection,
                &mut self.resolver_behavior_required_refs,
                type_name,
                behavior,
            ),
            BehaviorRefRole::Parent => None,
        }
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

    fn effective_behavior_impl_methods<'a>(
        &self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &'a [Declaration],
        unmatched_required: &mut VecDeque<String>,
    ) -> Vec<EffectiveBehaviorImplMethod<'a>> {
        methods
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
                let method_name = self.impl_effective_method_name(
                    unmatched_required,
                    ast_name,
                    resolver_owned_name,
                    type_name,
                );
                EffectiveBehaviorImplMethod {
                    declaration: method,
                    method_name,
                }
            })
            .collect()
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

    #[cfg(test)]
    fn collect_ast_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_ast_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Struct {
                type_params,
                fields,
            }),
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Enum {
                type_params,
                variants,
            }),
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Function {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Method {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Behavior {
                type_params,
                methods,
            }),
            Declaration::ImplBlock { methods, .. } => {
                tasks.push(AstTypeReferenceValidationTask::ImplBlock { methods });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(AstTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }

    fn validate_ast_type_reference_tasks(&mut self, tasks: &[AstTypeReferenceValidationTask<'_>]) {
        for task in tasks {
            match task {
                AstTypeReferenceValidationTask::Struct {
                    type_params,
                    fields,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for field in *fields {
                        self.validate_generic_type_ref_bounds(&field.ty, &scoped, field.span);
                        if let Some(default) = &field.default {
                            self.validate_generic_expr_type_references(default, &scoped);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Enum {
                    type_params,
                    variants,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_generic_type_ref_bounds(payload, &scoped, variant.span);
                        }
                    }
                }
                AstTypeReferenceValidationTask::Function {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Method {
                    type_params,
                    params,
                    return_type,
                    body,
                } => {
                    self.validate_ast_callable_type_references(
                        type_params,
                        params,
                        return_type,
                        body,
                        Span::dummy(),
                    );
                }
                AstTypeReferenceValidationTask::Behavior {
                    type_params,
                    methods,
                } => {
                    let scoped = type_param_name_set(type_params);
                    for method in *methods {
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
                AstTypeReferenceValidationTask::ImplBlock { methods } => {
                    for method in *methods {
                        if let Declaration::Function {
                            type_params,
                            params,
                            return_type,
                            body,
                            ..
                        } = method
                        {
                            self.validate_ast_callable_type_references(
                                type_params,
                                params,
                                return_type,
                                body,
                                method.span(),
                            );
                        }
                    }
                }
                AstTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_ast_callable_type_references(
        &mut self,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        return_span: Span,
    ) {
        let scoped = type_param_name_set(type_params);
        for param in params {
            self.validate_generic_type_ref_bounds(&param.ty, &scoped, param.span);
        }
        if let Some(return_type) = return_type {
            self.validate_generic_type_ref_bounds(return_type, &scoped, return_span);
        }
        self.validate_generic_expr_type_references(body, &scoped);
    }

    fn validate_resolver_type_reference_tasks(
        &mut self,
        tasks: &ResolverDeclarationMetadataTasks<'_>,
        symbols: Option<&SymbolTable>,
    ) {
        for task in &tasks.type_references {
            match task {
                ResolverTypeReferenceValidationTask::Struct { name, fields, span } => {
                    self.validate_resolver_struct_type_references(symbols, name, fields, *span);
                }
                ResolverTypeReferenceValidationTask::Enum { name, span } => {
                    self.validate_resolver_enum_type_references(symbols, name, *span);
                }
                ResolverTypeReferenceValidationTask::Function { name, body, span } => {
                    self.validate_resolver_function_type_references(symbols, name, body, *span);
                }
                ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span,
                } => {
                    let ast_key = Self::method_key(type_name, method_name);
                    self.validate_resolver_method_type_references(
                        symbols, &ast_key, type_name, body, *span,
                    );
                }
                ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span,
                } => {
                    self.validate_resolver_behavior_type_references(symbols, name, methods, *span);
                }
                ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods } => {
                    self.validate_resolver_impl_method_type_references(symbols, type_name, methods);
                }
                ResolverTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
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

    fn validate_resolver_impl_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &[Declaration],
    ) {
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

    fn validate_resolver_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_key = Self::validation_method_key(symbols, ast_key, type_name, span);
        self.validate_resolver_callable_type_references(&restored_key, body, span);
    }

    fn validate_resolver_function_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Value, name, span);
        self.validate_resolver_callable_type_references(&restored_name, body, span);
    }

    fn validate_resolver_callable_type_references(
        &mut self,
        restored_key: &str,
        body: &Expression,
        span: Span,
    ) {
        if let Some(scoped) = self.collected_value_type_param_scope(restored_key) {
            self.validate_collected_value_type_references(restored_key, &scoped, span);
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

    #[cfg(test)]
    fn collect_self_type_context_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<SelfTypeContextValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_self_type_context_validation_task(decl, &mut tasks);
        }
        tasks
    }

    fn push_self_type_context_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<SelfTypeContextValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct { fields, .. } => {
                tasks.push(SelfTypeContextValidationTask::Struct { fields });
            }
            Declaration::Enum { variants, .. } => {
                tasks.push(SelfTypeContextValidationTask::Enum { variants });
            }
            Declaration::Function {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Function {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Method {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Behavior { methods, .. } => {
                tasks.push(SelfTypeContextValidationTask::Behavior { methods });
            }
            Declaration::ImplBlock {
                behavior_type_args,
                methods,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::ImplBlock {
                behavior_type_args,
                methods,
                span: *span,
            }),
            Declaration::Requires {
                behavior_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Requires {
                behavior_type_args,
                span: *span,
            }),
            Declaration::BehaviorExtends {
                parent_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::BehaviorExtends {
                parent_type_args,
                span: *span,
            }),
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(SelfTypeContextValidationTask::TopLevelExpr { expr });
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
        }
    }

    fn validate_self_type_context_tasks(&mut self, tasks: &[SelfTypeContextValidationTask<'_>]) {
        for task in tasks {
            match task {
                SelfTypeContextValidationTask::Struct { fields } => {
                    for field in *fields {
                        self.validate_self_type_ref(&field.ty, field.span, false);
                        if let Some(default) = &field.default {
                            self.validate_self_type_expr(default, false);
                        }
                    }
                }
                SelfTypeContextValidationTask::Enum { variants } => {
                    for variant in *variants {
                        if let Some(payload) = &variant.payload {
                            self.validate_self_type_ref(payload, variant.span, false);
                        }
                    }
                }
                SelfTypeContextValidationTask::Function {
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    self.validate_self_type_callable(params, return_type, body, *span, false);
                }
                SelfTypeContextValidationTask::Method {
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    self.validate_self_type_callable(params, return_type, body, *span, true);
                }
                SelfTypeContextValidationTask::Behavior { methods } => {
                    for method in *methods {
                        let Some(default_body) = &method.default_body else {
                            self.validate_self_type_params(&method.params, true);
                            if let Some(return_type) = &method.return_type {
                                self.validate_self_type_ref(return_type, method.span, true);
                            }
                            continue;
                        };
                        self.validate_self_type_callable(
                            &method.params,
                            &method.return_type,
                            default_body,
                            method.span,
                            true,
                        );
                    }
                }
                SelfTypeContextValidationTask::ImplBlock {
                    behavior_type_args,
                    methods,
                    span,
                } => {
                    self.validate_self_type_refs(behavior_type_args, *span, false);
                    for method in *methods {
                        if let Declaration::Function {
                            params,
                            return_type,
                            body,
                            span,
                            ..
                        } = method
                        {
                            self.validate_self_type_callable(
                                params,
                                return_type,
                                body,
                                *span,
                                true,
                            );
                        }
                    }
                }
                SelfTypeContextValidationTask::Requires {
                    behavior_type_args,
                    span,
                } => {
                    self.validate_self_type_refs(behavior_type_args, *span, false);
                }
                SelfTypeContextValidationTask::BehaviorExtends {
                    parent_type_args,
                    span,
                } => {
                    self.validate_self_type_refs(parent_type_args, *span, false);
                }
                SelfTypeContextValidationTask::TopLevelExpr { expr } => {
                    self.validate_self_type_expr(expr, false);
                }
            }
        }
    }

    fn validate_self_type_callable(
        &mut self,
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
        allow_self_type: bool,
    ) {
        self.validate_self_type_params(params, allow_self_type);
        if let Some(return_type) = return_type {
            self.validate_self_type_ref(return_type, span, allow_self_type);
        }
        self.validate_self_type_expr(body, allow_self_type);
    }

    fn validate_self_type_params(&mut self, params: &[Param], allow_self_type: bool) {
        for param in params {
            self.validate_self_type_ref(&param.ty, param.span, allow_self_type);
        }
    }

    fn validate_self_type_refs(
        &mut self,
        ast_types: &[AstType],
        span: Span,
        allow_self_type: bool,
    ) {
        for ast_type in ast_types {
            self.validate_self_type_ref(ast_type, span, allow_self_type);
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
            | Expression::LoopControl { .. }
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

    fn validate_generic_type_arg_refs_allow_unknowns(&mut self, type_args: &[AstType], span: Span) {
        let scoped_type_params = HashSet::new();
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            &scoped_type_params,
            span,
            false,
        );
    }

    fn validate_generic_type_arg_refs(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
    ) {
        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            scoped_type_params,
            span,
            true,
        );
    }

    fn validate_generic_type_arg_refs_with_unknowns(
        &mut self,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        for type_arg in type_args {
            self.validate_generic_type_ref_bounds_with_unknowns(
                type_arg,
                scoped_type_params,
                span,
                reject_unknown,
            );
        }
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
                self.validate_generic_type_arg_refs_with_unknowns(
                    type_args,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );

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
                self.validate_generic_type_arg_refs_with_unknowns(
                    params,
                    scoped_type_params,
                    span,
                    reject_unknown,
                );
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
                self.validate_generic_type_arg_refs(type_args, scoped_type_params, *span);
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
            | Expression::LoopControl { .. }
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
                    self.require_resolver_callable_locals(symbols, params, body, &mut scope_cursor);
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
                    self.require_resolver_callable_locals(symbols, params, body, &mut scope_cursor);
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
                            self.require_resolver_scoped_expr_locals(
                                symbols,
                                default,
                                &mut scope_cursor,
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
                            self.require_resolver_callable_locals(
                                symbols,
                                &method.params,
                                default_body,
                                &mut scope_cursor,
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
                    self.validate_generic_type_arg_refs_allow_unknowns(behavior_type_args, *span);
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
                            self.require_resolver_callable_locals(
                                symbols,
                                params,
                                body,
                                &mut scope_cursor,
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
                    self.validate_generic_type_arg_refs_allow_unknowns(behavior_type_args, *span);
                }
                Declaration::BehaviorExtends {
                    behavior,
                    parent,
                    parent_type_args,
                    span,
                } => {
                    self.require_resolver_symbol(symbols, Namespace::Behavior, behavior, *span);
                    self.require_resolver_symbol(symbols, Namespace::Behavior, parent, *span);
                    self.validate_generic_type_arg_refs_allow_unknowns(parent_type_args, *span);
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
                    self.require_resolver_scoped_expr_locals(symbols, expr, &mut scope_cursor);
                }
                Declaration::Error { .. } => {}
            }
        }
        let replay_tasks = Self::collect_resolver_validation_replay_tasks(program, symbols);
        self.validate_no_extra_resolver_declaration_symbols(&replay_tasks, symbols);
        self.validate_no_extra_resolver_local_symbols(&replay_tasks, symbols);
        self.validate_resolver_behavior_association_lists(&replay_tasks);
        self.validate_stripped_resolver_import_symbols(&replay_tasks, symbols);
    }

    fn require_resolver_callable_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        body: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let mut locals = scope_cursor.new_scope();
        self.require_resolver_parameter_locals(symbols, params, &mut locals);
        self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut locals);
    }

    fn require_resolver_scoped_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
    ) {
        let mut locals = scope_cursor.new_scope();
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut locals);
    }

    fn validate_no_extra_resolver_declaration_symbols(
        &mut self,
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let expected = &tasks.expected_symbols;
        for symbol in symbols.symbols() {
            if !expected.validate_imports
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
            if !expected
                .declarations
                .contains(&(symbol.namespace, symbol.name.clone()))
            {
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
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        let expected = &tasks.expected_symbols;
        for symbol in symbols.symbols() {
            if symbol.namespace != Namespace::Local {
                continue;
            }
            if !expected
                .locals
                .contains(&(symbol.name.clone(), symbol.scope_id))
            {
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
        tasks: &ResolverValidationReplayTasks<'_>,
    ) {
        for task in &tasks.behavior_associations.type_associations {
            self.validate_resolver_behavior_impl_list(
                task.symbol,
                task.name,
                &task.impl_edges,
                task.span,
            );
            self.validate_resolver_behavior_required_list(
                task.symbol,
                task.name,
                &task.required_edges,
                task.span,
            );
        }

        for task in &tasks.behavior_associations.behavior_parents {
            self.validate_resolver_behavior_parent_list(
                task.symbol,
                task.name,
                &task.parent_edges,
                task.span,
            );
        }
    }

    #[cfg(test)]
    fn collect_resolver_behavior_association_list_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverBehaviorAssociationListTasks<'a> {
        Self::collect_resolver_validation_replay_tasks(program, symbols).behavior_associations
    }

    fn collect_resolver_validation_replay_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverValidationReplayTasks<'a> {
        let declaration_tasks =
            Self::collect_resolver_validation_replay_declaration_tasks(program, symbols);
        let mut tasks = ResolverValidationReplayTasks {
            expected_symbols: declaration_tasks.expected_symbols,
            behavior_associations: ResolverBehaviorAssociationListTasks::default(),
        };

        for source in declaration_tasks.type_declarations {
            Self::push_resolver_type_behavior_association_list_task(
                source,
                &declaration_tasks.expected_associations,
                &mut tasks.behavior_associations.type_associations,
            );
        }
        for source in declaration_tasks.behavior_declarations {
            Self::push_resolver_behavior_parent_list_task(
                source,
                &declaration_tasks.expected_parents,
                &mut tasks.behavior_associations.behavior_parents,
            );
        }

        tasks
    }

    fn push_resolver_type_behavior_association_list_task<'a>(
        source: ResolverValidationBehaviorAssociationSource<'a>,
        expected: &ExpectedBehaviorAssociations,
        tasks: &mut Vec<ResolverTypeBehaviorAssociationListTask<'a>>,
    ) {
        tasks.push(ResolverTypeBehaviorAssociationListTask {
            symbol: source.symbol,
            name: source.name,
            impl_edges: expected.impls.owned_edges_for(source.name),
            required_edges: expected.required.owned_edges_for(source.name),
            span: source.span,
        });
    }

    fn push_resolver_behavior_parent_list_task<'a>(
        source: ResolverValidationBehaviorAssociationSource<'a>,
        expected: &ExpectedBehaviorEdges,
        tasks: &mut Vec<ResolverBehaviorParentListTask<'a>>,
    ) {
        tasks.push(ResolverBehaviorParentListTask {
            symbol: source.symbol,
            name: source.name,
            parent_edges: expected.owned_edges_for(source.name),
            span: source.span,
        });
    }

    fn collect_resolver_validation_replay_declaration_tasks<'a>(
        program: &'a ast::Program,
        symbols: &'a SymbolTable,
    ) -> ResolverValidationReplayDeclarationTasks<'a> {
        let mut tasks = ResolverValidationReplayDeclarationTasks::default();
        let mut scope_cursor = ResolverScopeCursor::default();

        for decl in &program.declarations {
            match decl {
                Declaration::Function {
                    name, params, body, ..
                } => {
                    push_expected_resolver_callable_symbol(
                        name.clone(),
                        params,
                        body,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::Method {
                    type_name,
                    method_name,
                    params,
                    body,
                    ..
                } => {
                    push_expected_resolver_callable_symbol(
                        method_signature_key(type_name, method_name),
                        params,
                        body,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::Struct {
                    name, fields, span, ..
                } => {
                    for field in fields {
                        if let Some(default) = &field.default {
                            push_expected_resolver_scoped_expr_symbols(
                                default,
                                &mut scope_cursor,
                                &mut tasks.expected_symbols,
                            );
                        }
                    }
                    push_resolver_validation_association_source(
                        Namespace::Type,
                        name,
                        *span,
                        symbols,
                        &mut tasks.expected_symbols,
                        &mut tasks.type_declarations,
                    );
                }
                Declaration::Enum {
                    name,
                    variants,
                    span,
                    ..
                } => {
                    push_expected_resolver_variant_symbols(variants, &mut tasks.expected_symbols);
                    push_resolver_validation_association_source(
                        Namespace::Type,
                        name,
                        *span,
                        symbols,
                        &mut tasks.expected_symbols,
                        &mut tasks.type_declarations,
                    );
                }
                Declaration::Behavior {
                    name,
                    methods,
                    span,
                    ..
                } => {
                    for method in methods {
                        if let Some(default_body) = &method.default_body {
                            expected_resolver_callable_locals(
                                &method.params,
                                default_body,
                                &mut scope_cursor,
                                &mut tasks.expected_symbols.locals,
                            );
                        }
                    }
                    push_resolver_validation_association_source(
                        Namespace::Behavior,
                        name,
                        *span,
                        symbols,
                        &mut tasks.expected_symbols,
                        &mut tasks.behavior_declarations,
                    );
                }
                Declaration::Import {
                    names, module_path, ..
                } => {
                    push_expected_resolver_import_symbols(
                        names,
                        module_path,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::ImplBlock {
                    type_name,
                    behavior: Some(behavior),
                    methods,
                    behavior_type_args,
                    ..
                } => {
                    collect_expected_resolver_impl_method_symbols(
                        type_name,
                        methods,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                    push_expected_behavior_impl_edge(
                        &mut tasks.expected_associations,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                Declaration::ImplBlock {
                    type_name, methods, ..
                } => {
                    collect_expected_resolver_impl_method_symbols(
                        type_name,
                        methods,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    ..
                } => {
                    push_expected_behavior_required_edge(
                        &mut tasks.expected_associations,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                Declaration::BehaviorExtends {
                    behavior,
                    parent,
                    parent_type_args,
                    ..
                } => {
                    push_expected_behavior_parent_edge(
                        &mut tasks.expected_parents,
                        behavior,
                        parent,
                        parent_type_args,
                    );
                }
                Declaration::TopLevelExpr { expr, .. } => {
                    push_expected_resolver_scoped_expr_symbols(
                        expr,
                        &mut scope_cursor,
                        &mut tasks.expected_symbols,
                    );
                }
                _ => {}
            }
        }

        tasks
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
        tasks: &ResolverValidationReplayTasks<'_>,
        symbols: &SymbolTable,
    ) {
        if tasks.expected_symbols.validate_imports {
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

    fn require_resolver_child_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut child_locals = scope_cursor.child_scope(locals);
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut child_locals);
    }

    fn require_resolver_pattern_expr_locals(
        &mut self,
        symbols: &SymbolTable,
        pattern: &ast::Pattern,
        expr: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut pattern_locals = scope_cursor.child_scope(locals);
        self.require_resolver_pattern_locals(symbols, pattern, scope_cursor, &mut pattern_locals);
        self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut pattern_locals);
    }

    fn require_resolver_block_locals(
        &mut self,
        symbols: &SymbolTable,
        statements: &[ast::Statement],
        expr: Option<&Expression>,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
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
            self.require_resolver_expr_locals(symbols, expr, scope_cursor, &mut block_locals);
        }
    }

    fn require_resolver_closure_locals(
        &mut self,
        symbols: &SymbolTable,
        params: &[Param],
        body: &Expression,
        scope_cursor: &mut ResolverScopeCursor,
        locals: &ResolverLocalScope,
    ) {
        let mut closure_locals = scope_cursor.child_scope(locals);
        self.require_resolver_parameter_locals(symbols, params, &mut closure_locals);
        self.require_resolver_expr_locals(symbols, body, scope_cursor, &mut closure_locals);
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
                        self.require_resolver_pattern_expr_locals(
                            symbols,
                            &arm.pattern,
                            guard,
                            scope_cursor,
                            locals,
                        );
                    }
                    self.require_resolver_pattern_expr_locals(
                        symbols,
                        &arm.pattern,
                        &arm.body,
                        scope_cursor,
                        locals,
                    );
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                self.require_resolver_child_expr_locals(symbols, body, scope_cursor, locals);
            }
            Expression::Loop { body, .. } => {
                self.require_resolver_child_expr_locals(symbols, body, scope_cursor, locals);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.require_resolver_expr_locals(symbols, condition, scope_cursor, locals);
                self.require_resolver_child_expr_locals(symbols, then_body, scope_cursor, locals);
                if let Some(else_body) = else_body {
                    self.require_resolver_child_expr_locals(
                        symbols,
                        else_body,
                        scope_cursor,
                        locals,
                    );
                }
            }
            Expression::Block {
                statements, expr, ..
            } => {
                self.require_resolver_block_locals(
                    symbols,
                    statements,
                    expr.as_deref(),
                    scope_cursor,
                    locals,
                );
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.require_resolver_expr_locals(symbols, value, scope_cursor, locals);
                }
            }
            Expression::Closure { params, body, .. } => {
                self.require_resolver_closure_locals(symbols, params, body, scope_cursor, locals);
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
            | Expression::LoopControl { .. }
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
                if resolver_var_decl_binds_local(name, *mutable, *constant, locals) {
                    self.require_resolver_var_decl_local(symbols, name, *mutable, *span, locals);
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
                self.require_resolver_block_locals(symbols, stmts, None, scope_cursor, locals);
            }
        }
    }

    fn require_resolver_var_decl_local(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        mutable: bool,
        span: Span,
        locals: &mut ResolverLocalScope,
    ) {
        self.require_resolver_local_symbol(
            symbols,
            name,
            expected_local_symbol(mutable, locals.current_scope_id),
            span,
        );
        locals.insert(name.to_string(), mutable);
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
                self.require_resolver_pattern_binding(symbols, name, *span, locals);
            }
            ast::Pattern::Struct { fields, span, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.require_resolver_pattern_locals(symbols, nested, scope_cursor, locals);
                    } else {
                        self.require_resolver_pattern_binding(symbols, name, *span, locals);
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

    fn require_resolver_pattern_binding(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
        span: Span,
        locals: &mut ResolverLocalScope,
    ) {
        self.require_resolver_local_symbol(
            symbols,
            name,
            expected_local_symbol(false, locals.current_scope_id),
            span,
        );
        locals.insert(name.to_string(), false);
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
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_type_parameter_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: TypeParameterAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_field_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: FieldAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_variant_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: VariantAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_behavior_association_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorAssociationAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_behavior_declaration_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: BehaviorDeclarationAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
    }

    fn validate_resolver_absent_mutability_metadata(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: MutabilityAbsenceValidation,
        span: Span,
    ) {
        self.validate_resolver_absent_metadata(symbol, symbol_kind, name, validation, span);
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

    fn validate_resolver_absent_metadata<const N: usize>(
        &mut self,
        symbol: &crate::resolver::Symbol,
        symbol_kind: &str,
        name: &str,
        validation: impl AbsentMetadataValidation<N>,
        span: Span,
    ) {
        let entries = validation.entries(symbol);
        self.validate_resolver_absent_metadata_entries(symbol_kind, name, &entries, span);
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

        self.validate_resolver_metadata_list(
            symbol.type_parameter_names.as_deref(),
            &expected.names,
            format_type_parameter_names,
            validation.name_code,
            |actual, expected| validation.name_message(symbol_kind, name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_bounds.as_deref(),
            &expected.bounds,
            format_type_parameter_bounds,
            validation.bound_code,
            |actual, expected| validation.bound_message(symbol_kind, name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.type_parameter_bound_refs.as_deref(),
            &expected.bound_refs,
            format_type_parameter_bound_refs,
            validation.bound_ref_code,
            |actual, expected| validation.bound_ref_message(symbol_kind, name, actual, expected),
            span,
        );
    }

    fn validate_resolver_metadata_list<T: PartialEq>(
        &mut self,
        actual: Option<&[T]>,
        expected: &[T],
        display: impl Fn(Option<&[T]>) -> String,
        code: &'static str,
        message: impl Fn(&str, &str) -> String,
        span: Span,
    ) {
        if actual != Some(expected) {
            let actual_display = display(actual);
            let expected_display = display(Some(expected));
            self.diagnostics.push(Diagnostic::error(
                code,
                message(&actual_display, &expected_display),
                span,
            ));
        }
    }

    fn validate_resolver_metadata_value<T: PartialEq + ?Sized>(
        &mut self,
        actual: Option<&T>,
        expected: Option<&T>,
        display: impl Fn(Option<&T>) -> String,
        code: &'static str,
        message: impl Fn(&str, &str) -> String,
        span: Span,
    ) {
        if actual != expected {
            let actual_display = display(actual);
            let expected_display = display(expected);
            self.diagnostics.push(Diagnostic::error(
                code,
                message(&actual_display, &expected_display),
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
        self.validate_resolver_metadata_list(
            symbol.field_types.as_deref(),
            &expected.typed,
            format_field_types,
            validation.typed_code,
            |actual, expected| validation.typed_message(symbol_kind, name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_list(
            symbol.field_type_names.as_deref(),
            &expected.display,
            format_field_type_names,
            validation.display_code,
            |actual, expected| validation.display_message(symbol_kind, name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_names(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_variant_names: &[String],
        span: Span,
    ) {
        let validation = VariantNameValidation::resolver_code();
        self.validate_resolver_metadata_list(
            symbol.variant_names.as_deref(),
            expected_variant_names,
            format_variant_names,
            validation.code,
            |actual, expected| validation.message(name, actual, expected),
            span,
        );
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
        self.validate_resolver_metadata_value(
            symbol.variant_payload_type.as_ref(),
            expected.typed.as_ref(),
            |value| optional_ast_type_display(value, "none"),
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_value(
            symbol.variant_payload_type_name.as_deref(),
            expected.display.as_deref(),
            |value| resolver_metadata_display(value).to_string(),
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_variant_owner_name(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected_owner_name: &str,
        span: Span,
    ) {
        let validation = VariantOwnerValidation::resolver_code();
        self.validate_resolver_metadata_value(
            symbol.variant_owner_name.as_deref(),
            Some(expected_owner_name),
            |value| resolver_metadata_display(value).to_string(),
            validation.code,
            |actual, expected| validation.message(name, actual, expected),
            span,
        );
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
        self.validate_resolver_metadata_list(
            symbol.behavior_method_signatures.as_deref(),
            &expected.signatures,
            format_behavior_method_signatures,
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_list(
            symbol.behavior_method_types.as_deref(),
            &expected.typed,
            format_behavior_method_types,
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
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

        self.validate_resolver_metadata_list(
            symbol.parameter_names.as_deref(),
            &expected.names,
            format_parameter_names,
            validation.name_code,
            |actual, expected| validation.name_message(name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.parameter_type_names.as_deref(),
            &expected.display_types,
            format_parameter_type_names,
            validation.display_type_code,
            |actual, expected| validation.display_type_message(name, actual, expected),
            span,
        );

        self.validate_resolver_metadata_list(
            symbol.parameter_types.as_deref(),
            &expected.typed_types,
            format_ast_type_list,
            validation.typed_type_code,
            |actual, expected| validation.typed_type_message(name, actual, expected),
            span,
        );
    }

    fn validate_resolver_value_return_type(
        &mut self,
        symbol: &crate::resolver::Symbol,
        name: &str,
        expected: &ExpectedReturnMetadata,
        span: Span,
    ) {
        let validation = ReturnValidation::resolver_codes();

        self.validate_resolver_metadata_value(
            symbol.return_type_name.as_deref(),
            Some(expected.display.as_str()),
            |value| resolver_metadata_display(value).to_string(),
            validation.display_code,
            |actual, expected| validation.display_message(name, actual, expected),
            span,
        );
        self.validate_resolver_metadata_value(
            symbol.return_type.as_ref(),
            Some(&expected.typed),
            resolver_ast_type_metadata_display,
            validation.typed_code,
            |actual, expected| validation.typed_message(name, actual, expected),
            span,
        );
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

fn expected_behavior_edge(behavior: &str, type_args: &[AstType]) -> ExpectedBehaviorEdge {
    ExpectedBehaviorEdge::new(behavior, type_args)
}

fn push_expected_behavior_impl_edge(
    expected: &mut ExpectedBehaviorAssociations,
    type_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) {
    expected.impls.push(type_name, behavior, behavior_type_args);
}

fn push_expected_behavior_required_edge(
    expected: &mut ExpectedBehaviorAssociations,
    type_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) {
    expected
        .required
        .push(type_name, behavior, behavior_type_args);
}

fn push_expected_behavior_parent_edge(
    expected: &mut ExpectedBehaviorEdges,
    behavior: &str,
    parent: &str,
    parent_type_args: &[AstType],
) {
    expected.push(behavior, parent, parent_type_args);
}

fn collect_expected_resolver_impl_method_symbols(
    type_name: &str,
    methods: &[Declaration],
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut ResolverExpectedSymbolSets,
) {
    for method in methods {
        if let Declaration::Function {
            name, params, body, ..
        } = method
        {
            push_expected_resolver_callable_symbol(
                method_signature_key(type_name, name),
                params,
                body,
                scope_cursor,
                expected,
            );
        }
    }
}

fn push_resolver_validation_association_source<'a>(
    namespace: Namespace,
    name: &'a str,
    span: Span,
    symbols: &'a SymbolTable,
    expected: &mut ResolverExpectedSymbolSets,
    sources: &mut Vec<ResolverValidationBehaviorAssociationSource<'a>>,
) {
    expected.declarations.insert((namespace, name.to_string()));
    if let Some(symbol) = symbols.lookup(namespace, name) {
        sources.push(ResolverValidationBehaviorAssociationSource { name, symbol, span });
    }
}

fn push_expected_resolver_import_symbols(
    names: &[String],
    module_path: &[String],
    expected: &mut ResolverExpectedSymbolSets,
) {
    expected.validate_imports = true;
    expected
        .declarations
        .insert((Namespace::Module, module_path.join(".")));
    for name in names {
        expected
            .declarations
            .insert((Namespace::Import, name.clone()));
    }
}

fn push_expected_resolver_variant_symbols(
    variants: &[EnumVariant],
    expected: &mut ResolverExpectedSymbolSets,
) {
    for variant in variants {
        expected
            .declarations
            .insert((Namespace::Variant, variant.name.clone()));
    }
}

fn push_expected_resolver_scoped_expr_symbols(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut ResolverExpectedSymbolSets,
) {
    expected_resolver_scoped_expr_locals(expr, scope_cursor, &mut expected.locals);
}

fn push_expected_resolver_callable_symbol(
    name: String,
    params: &[Param],
    body: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut ResolverExpectedSymbolSets,
) {
    expected.declarations.insert((Namespace::Value, name));
    expected_resolver_callable_locals(params, body, scope_cursor, &mut expected.locals);
}

fn expected_resolver_callable_locals(
    params: &[Param],
    body: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut locals = scope_cursor.new_scope();
    expected_resolver_parameter_locals(params, &mut locals, expected);
    expected_resolver_expr_locals(body, scope_cursor, &mut locals, expected);
}

fn expected_resolver_scoped_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut locals = scope_cursor.new_scope();
    expected_resolver_expr_locals(expr, scope_cursor, &mut locals, expected);
}

fn expected_resolver_child_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut child_locals = scope_cursor.child_scope(locals);
    expected_resolver_expr_locals(expr, scope_cursor, &mut child_locals, expected);
}

fn expected_resolver_pattern_expr_locals(
    pattern: &ast::Pattern,
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut pattern_locals = scope_cursor.child_scope(locals);
    expected_resolver_pattern_locals(pattern, scope_cursor, &mut pattern_locals, expected);
    expected_resolver_expr_locals(expr, scope_cursor, &mut pattern_locals, expected);
}

fn expected_resolver_block_locals(
    statements: &[ast::Statement],
    expr: Option<&Expression>,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut block_locals = scope_cursor.child_scope(locals);
    for statement in statements {
        expected_resolver_statement_locals(statement, scope_cursor, &mut block_locals, expected);
    }
    if let Some(expr) = expr {
        expected_resolver_expr_locals(expr, scope_cursor, &mut block_locals, expected);
    }
}

fn expected_resolver_closure_locals(
    params: &[Param],
    body: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut closure_locals = scope_cursor.child_scope(locals);
    expected_resolver_parameter_locals(params, &mut closure_locals, expected);
    expected_resolver_expr_locals(body, scope_cursor, &mut closure_locals, expected);
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
                    expected_resolver_pattern_expr_locals(
                        &arm.pattern,
                        guard,
                        scope_cursor,
                        locals,
                        expected,
                    );
                }
                expected_resolver_pattern_expr_locals(
                    &arm.pattern,
                    &arm.body,
                    scope_cursor,
                    locals,
                    expected,
                );
            }
        }
        Expression::WhileLoop {
            condition, body, ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            expected_resolver_child_expr_locals(body, scope_cursor, locals, expected);
        }
        Expression::Loop { body, .. } => {
            expected_resolver_child_expr_locals(body, scope_cursor, locals, expected);
        }
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            expected_resolver_child_expr_locals(then_body, scope_cursor, locals, expected);
            if let Some(else_body) = else_body {
                expected_resolver_child_expr_locals(else_body, scope_cursor, locals, expected);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            expected_resolver_block_locals(
                statements,
                expr.as_deref(),
                scope_cursor,
                locals,
                expected,
            );
        }
        Expression::Return { value, .. } => {
            if let Some(value) = value {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::Closure { params, body, .. } => {
            expected_resolver_closure_locals(params, body, scope_cursor, locals, expected);
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
        | Expression::LoopControl { .. }
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
            if resolver_var_decl_binds_local(name, *mutable, *constant, locals) {
                expected_resolver_var_decl_local(name, *mutable, locals, expected);
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
            expected_resolver_block_locals(stmts, None, scope_cursor, locals, expected);
        }
    }
}

fn expected_resolver_var_decl_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected_resolver_local(name, mutable, locals, expected);
}

fn resolver_var_decl_binds_local(
    name: &str,
    mutable: bool,
    constant: bool,
    locals: &ResolverLocalScope,
) -> bool {
    constant || mutable || !locals.is_mutable(name)
}

fn expected_resolver_pattern_locals(
    pattern: &ast::Pattern,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match pattern {
        ast::Pattern::Identifier { name, .. } => {
            expected_resolver_pattern_binding(name, locals, expected);
        }
        ast::Pattern::Struct { fields, .. } => {
            for (name, nested) in fields {
                if let Some(nested) = nested {
                    expected_resolver_pattern_locals(nested, scope_cursor, locals, expected);
                } else {
                    expected_resolver_pattern_binding(name, locals, expected);
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

fn expected_resolver_pattern_binding(
    name: &str,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected_resolver_local(name, false, locals, expected);
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

#[cfg(test)]
mod tests;
