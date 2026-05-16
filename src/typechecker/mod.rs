//! Typechecker — transforms untyped AST → TypedProgram.
//!
//! Pipeline:
//! 1. **Collect**: Register all struct/enum/function/behavior signatures
//! 2. **Resolve**: Resolve type references (Named("Foo") → Struct fields)
//! 3. **Check**: Type-check function bodies, produce TypedExpression
//!
//! The typechecker NEVER defaults unknown types to I32. If a type can't be
//! resolved, it's an error.

mod behavior_associations;
mod behavior_impl_support;
mod behavior_impl_validation;
mod behavior_ref_metadata;
mod closures;
mod declaration_collection;
mod environment;
mod expressions;
mod generic_type_validation;
mod monomorphize;
mod patterns;
mod program_checking;
mod resolve;
mod resolver_lookup;
mod resolver_validation;
mod scope_management;
mod self_type_validation;
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

include!("declaration_tasks.rs");
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
    behavior_refs_by_key: HashMap<String, BehaviorParentRef>,
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
            behavior_refs_by_key: HashMap::new(),
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
}

#[cfg(test)]
mod tests;
