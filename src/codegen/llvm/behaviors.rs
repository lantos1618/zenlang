use super::LLVMCompiler;
use crate::ast::{AstType, Expression, TraitImplementation};
use crate::error::CompileError;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType};
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use std::collections::HashMap;

// ============================================================================
// BehaviorCodegen - VTable and method dispatch management
// ============================================================================

#[allow(dead_code)]
#[derive(Default)]
pub struct BehaviorCodegen<'ctx> {
    vtables: HashMap<(String, String), PointerValue<'ctx>>,
    pub method_impls: HashMap<(String, String), FunctionValue<'ctx>>,
    /// Track return types of trait methods for proper void handling
    pub method_return_types: HashMap<(String, String), AstType>,
}

impl<'ctx> BehaviorCodegen<'ctx> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register the return type of a trait method
    pub fn register_method_return_type(
        &mut self,
        type_name: &str,
        method_name: &str,
        return_type: AstType,
    ) {
        self.method_return_types.insert(
            (type_name.to_string(), method_name.to_string()),
            return_type,
        );
    }

    /// Look up the return type of a trait method
    pub fn get_method_return_type(&self, type_name: &str, method_name: &str) -> Option<&AstType> {
        self.method_return_types
            .get(&(type_name.to_string(), method_name.to_string()))
    }

    pub fn generate_vtable(
        &mut self,
        context: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
        type_name: &str,
        behavior_name: &str,
        methods: &[(&str, FunctionValue<'ctx>)],
    ) -> Result<PointerValue<'ctx>, CompileError> {
        let fn_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let field_types: Vec<_> = (0..methods.len()).map(|_| fn_ptr_type.into()).collect();
        let vtable_type = context.struct_type(&field_types, false);

        let vtable_name = format!("vtable_{}_{}", type_name, behavior_name);
        let vtable_global = module.add_global(vtable_type, None, &vtable_name);

        let method_values: Vec<BasicValueEnum> = methods
            .iter()
            .map(|(_, func)| {
                func.as_global_value()
                    .as_pointer_value()
                    .const_cast(fn_ptr_type)
                    .into()
            })
            .collect();

        vtable_global.set_initializer(&vtable_type.const_named_struct(&method_values));
        let vtable_ptr = vtable_global.as_pointer_value();
        self.vtables.insert(
            (type_name.to_string(), behavior_name.to_string()),
            vtable_ptr,
        );
        Ok(vtable_ptr)
    }

    pub fn register_method(
        &mut self,
        type_name: &str,
        method_name: &str,
        function: FunctionValue<'ctx>,
    ) {
        self.method_impls
            .insert((type_name.to_string(), method_name.to_string()), function);
    }

    pub fn resolve_method(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        self.method_impls
            .get(&(type_name.to_string(), method_name.to_string()))
            .copied()
    }
}

// ============================================================================
// Helpers for impl/trait compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    /// Create function type from return type and parameter types
    fn create_fn_type(
        &self,
        return_type: &super::Type<'ctx>,
        param_types: &[BasicMetadataTypeEnum<'ctx>],
    ) -> Result<FunctionType<'ctx>, CompileError> {
        Ok(match return_type {
            super::Type::Void => self.context.void_type().fn_type(param_types, false),
            super::Type::Basic(basic) => match basic {
                BasicTypeEnum::IntType(t) => t.fn_type(param_types, false),
                BasicTypeEnum::FloatType(t) => t.fn_type(param_types, false),
                BasicTypeEnum::PointerType(t) => t.fn_type(param_types, false),
                BasicTypeEnum::StructType(t) => t.fn_type(param_types, false),
                BasicTypeEnum::ArrayType(t) => t.fn_type(param_types, false),
                BasicTypeEnum::VectorType(t) => t.fn_type(param_types, false),
                BasicTypeEnum::ScalableVectorType(t) => t.fn_type(param_types, false),
            },
            super::Type::Struct(st) => st.fn_type(param_types, false),
            _ => {
                return Err(CompileError::UnsupportedFeature(
                    format!("Unsupported return type: {:?}", return_type),
                    None,
                ))
            }
        })
    }

    /// Resolve Self type to concrete type
    fn resolve_self_type(
        &self,
        param_type: &AstType,
        type_name: &str,
        _type_params: &[crate::ast::TypeParameter],
    ) -> AstType {
        match param_type {
            AstType::Generic { name, .. } if name == "Self" || name.starts_with("Self_") => {
                if let Some(struct_info) = self.struct_types.get(type_name) {
                    let fields: Vec<_> = struct_info
                        .fields
                        .iter()
                        .map(|(n, (_, t))| (n.clone(), t.clone()))
                        .collect();
                    AstType::Struct {
                        name: type_name.to_string(),
                        fields,
                    }
                } else {
                    AstType::Struct {
                        name: type_name.to_string(),
                        fields: vec![],
                    }
                }
            }
            _ => param_type.clone(),
        }
    }

    /// Convert parameter to LLVM metadata type, handling structs by pointer
    fn param_to_metadata(
        &self,
        llvm_type: super::Type<'ctx>,
    ) -> Result<BasicMetadataTypeEnum<'ctx>, CompileError> {
        Ok(match llvm_type {
            super::Type::Basic(basic) => basic.into(),
            super::Type::Struct(_) => self
                .context
                .ptr_type(inkwell::AddressSpace::default())
                .into(),
            _ => {
                return Err(CompileError::UnsupportedFeature(
                    "Unsupported parameter type in method".to_string(),
                    None,
                ))
            }
        })
    }

    /// Extract type name from AstType
    fn type_name_from_ast(ast_type: &AstType) -> Option<String> {
        match ast_type {
            // Vec, DynVec, HashMap etc. are now Generic types from stdlib
            AstType::Struct { name, .. }
            | AstType::Generic { name, .. }
            | AstType::Enum { name, .. }
            | AstType::EnumType { name } => Some(name.clone()),
            _ => None,
        }
    }
}

// ============================================================================
// Impl Block Compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_impl_block(
        &mut self,
        impl_block: &crate::ast::ImplBlock,
    ) -> Result<(), CompileError> {
        let type_name = &impl_block.type_name;
        self.current_impl_type = Some(type_name.clone());

        for method in &impl_block.methods {
            let mangled_name = format!("{}_{}", type_name, method.name);
            let llvm_return_type = self.to_llvm_type(&method.return_type)?;

            let mut param_types = Vec::new();
            for (param_name, param_type) in &method.args {
                let resolved = if param_name == "self" {
                    if param_type.is_ptr_type() {
                        param_type.clone()
                    } else {
                        AstType::ptr(AstType::Generic {
                            name: type_name.clone(),
                            type_args: impl_block
                                .type_params
                                .iter()
                                .map(|tp| AstType::Generic {
                                    name: tp.name.clone(),
                                    type_args: vec![],
                                })
                                .collect(),
                        })
                    }
                } else {
                    self.resolve_self_type(param_type, type_name, &impl_block.type_params)
                };

                let llvm_type = self.to_llvm_type(&resolved)?;
                param_types.push(self.param_to_metadata(llvm_type)?);
            }

            let fn_type = self.create_fn_type(&llvm_return_type, &param_types)?;
            let function = self.module.add_function(&mangled_name, fn_type, None);

            if let Some(ref mut bc) = self.behavior_codegen {
                bc.method_impls
                    .insert((type_name.clone(), method.name.clone()), function);
            }
        }

        Ok(())
    }
}

// ============================================================================
// Trait Implementation Compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_trait_implementation(
        &mut self,
        trait_impl: &TraitImplementation,
    ) -> Result<(), CompileError> {
        let type_name = &trait_impl.type_name;
        let trait_name = &trait_impl.trait_name;
        self.current_impl_type = Some(type_name.clone());

        for method in &trait_impl.methods {
            let mangled_name = format!("{}_{}_{}", type_name, trait_name, method.name);
            let llvm_return_type = self.to_llvm_type(&method.return_type)?;

            let mut param_types = Vec::new();
            for (param_name, param_type) in &method.args {
                let actual_type = if param_name == "self" {
                    self.resolve_self_type(param_type, type_name, &[])
                } else {
                    param_type.clone()
                };

                let llvm_param =
                    if param_name == "self" || matches!(actual_type, AstType::Struct { .. }) {
                        let st = self.to_llvm_type(&actual_type)?;
                        if matches!(st, super::Type::Struct(_)) {
                            super::Type::Basic(
                                self.context
                                    .ptr_type(inkwell::AddressSpace::default())
                                    .into(),
                            )
                        } else {
                            st
                        }
                    } else {
                        self.to_llvm_type(&actual_type)?
                    };

                if let Ok(meta) = self.param_to_metadata(llvm_param) {
                    param_types.push(meta);
                }
            }

            let fn_type = self.create_fn_type(&llvm_return_type, &param_types)?;
            let function = self.module.add_function(&mangled_name, fn_type, None);

            // Set up function body
            let entry = self.context.append_basic_block(function, "entry");
            self.builder.position_at_end(entry);
            let prev_function = self.current_function;
            self.current_function = Some(function);

            self.symbols.enter_scope();

            // Add module imports to variables (so trait impl methods can access them)
            for (name, marker) in self.module_imports.clone() {
                let alloca = self.builder.build_alloca(self.context.i64_type(), &name)?;
                self.builder
                    .build_store(alloca, self.context.i64_type().const_int(marker, false))?;
                self.variables.insert(
                    name.clone(),
                    super::VariableInfo {
                        pointer: alloca,
                        ast_type: AstType::StdModule,
                        is_mutable: false,
                        is_initialized: true,
                        definition_span: self.get_current_span(),
                    },
                );
            }

            for (i, (param_name, param_type)) in method.args.iter().enumerate() {
                if i < function.count_params() as usize {
                    let param_value = function.get_nth_param(i as u32).ok_or_else(|| {
                        CompileError::InternalError(
                            format!("Missing parameter {} in method", i),
                            self.get_current_span(),
                        )
                    })?;
                    let alloca = self
                        .builder
                        .build_alloca(param_value.get_type(), param_name)?;
                    self.builder.build_store(alloca, param_value)?;
                    self.symbols
                        .insert(param_name.clone(), super::symbols::Symbol::Variable(alloca));

                    let actual_type = if param_name == "self" {
                        let resolved = self.resolve_self_type(param_type, type_name, &[]);
                        AstType::ptr(resolved)
                    } else {
                        param_type.clone()
                    };

                    self.variables.insert(
                        param_name.clone(),
                        super::VariableInfo {
                            pointer: alloca,
                            ast_type: actual_type,
                            is_mutable: false,
                            is_initialized: true,
                            definition_span: self.get_current_span(),
                        },
                    );
                }
            }

            for stmt in &method.body {
                self.compile_statement(stmt)?;
            }

            if matches!(llvm_return_type, super::Type::Void) {
                if let Ok(block) = self.current_block() {
                    if block.get_terminator().is_none() {
                        self.builder.build_return(None)?;
                    }
                }
            }

            self.symbols.exit_scope();
            self.variables.clear();
            self.current_function = prev_function;

            if self.behavior_codegen.is_none() {
                self.behavior_codegen = Some(BehaviorCodegen::new());
            }
            if let Some(ref mut bc) = self.behavior_codegen {
                bc.register_method(type_name, &method.name, function);
                // Register return type for proper void handling
                bc.register_method_return_type(type_name, &method.name, method.return_type.clone());
            }
        }

        // Generate vtable
        let methods: Vec<_> = trait_impl
            .methods
            .iter()
            .filter_map(|m| {
                let name = format!("{}_{}_{}", type_name, trait_name, m.name);
                self.module
                    .get_function(&name)
                    .map(|f| (m.name.as_str(), f))
            })
            .collect();

        if let Some(ref mut bc) = self.behavior_codegen {
            bc.generate_vtable(self.context, &self.module, type_name, trait_name, &methods)?;
        }

        self.current_impl_type = None;
        Ok(())
    }
}

// ============================================================================
// Method Call Compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    /// Compile method call with explicit type arguments from AST
    pub fn compile_method_call_with_type_args(
        &mut self,
        object: &Expression,
        method_name: &str,
        type_args: &[AstType],
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Handle @std reference
        if matches!(object, Expression::StdReference) {
            let method_with_generics = Self::format_method_with_type_args(method_name, type_args);
            return self.compile_std_method_call(&method_with_generics, args);
        }

        // Handle module imports
        if let Expression::Identifier(name) = object {
            if let Some(var_info) = self.variables.get(name) {
                if matches!(var_info.ast_type, AstType::StdModule) {
                    let method_with_generics =
                        Self::format_method_with_type_args(method_name, type_args);
                    return super::functions::calls::compile_function_call(
                        self,
                        Some(name),
                        &method_with_generics,
                        args,
                    );
                }
            }
        }

        // Delegate to the existing method for other cases
        self.compile_method_call(object, method_name, args)
    }

    /// Format method name with type arguments (e.g., "load" + [I32] -> "load<i32>")
    fn format_method_with_type_args(method_name: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            method_name.to_string()
        } else {
            let type_args_str = type_args
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", method_name, type_args_str)
        }
    }

    pub fn compile_method_call(
        &mut self,
        object: &Expression,
        method_name: &str,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // Handle @std reference
        if matches!(object, Expression::StdReference) {
            return self.compile_std_method_call(method_name, args);
        }

        // Handle module imports
        if let Expression::Identifier(name) = object {
            if let Some(var_info) = self.variables.get(name) {
                if matches!(var_info.ast_type, AstType::StdModule) {
                    return super::functions::calls::compile_function_call(
                        self,
                        Some(name),
                        method_name,
                        args,
                    );
                }
            }
        }

        // NOTE: Range constructors and methods are now in stdlib/core/iterator.zen
        // HashMap methods use stdlib Zen implementation via normal resolution

        // Try behavior codegen dispatch
        let type_name = self.infer_type_name(object)?;
        if let Some(result) = self.try_behavior_dispatch(object, &type_name, method_name, args)? {
            return Ok(result);
        }

        // Try qualified method name
        if let Some(result) =
            self.try_qualified_method_call(object, &type_name, method_name, args)?
        {
            return Ok(result);
        }

        // Fallback to UFC
        if let Some(result) = self.try_ufc_call(object, method_name, args)? {
            return Ok(result);
        }

        Err(CompileError::UndeclaredFunction(
            format!("{}.{}", type_name, method_name),
            self.get_current_span(),
        ))
    }

    fn compile_std_method_call(
        &mut self,
        method_name: &str,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if let Ok(result) =
            super::functions::calls::compile_function_call(self, None, method_name, args)
        {
            return Ok(result);
        }
        super::functions::calls::compile_function_call(self, Some("Std"), method_name, args)
    }

    // NOTE: HashMap and Range methods now use stdlib Zen implementations
    // See stdlib/collections/hashmap.zen and stdlib/core/iterator.zen

    fn try_behavior_dispatch(
        &mut self,
        object: &Expression,
        type_name: &str,
        method_name: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
        // First check if method exists - need to check before borrowing bc mutably
        let (function, is_void_method) = {
            let Some(ref bc) = self.behavior_codegen else {
                return Ok(None);
            };
            let Some(function) = bc.resolve_method(type_name, method_name) else {
                return Ok(None);
            };
            // Check if method returns void using registered return type
            let is_void = bc
                .get_method_return_type(type_name, method_name)
                .map(|rt| matches!(rt, AstType::Void))
                .unwrap_or(false);
            (function, is_void)
        };

        let self_value = match object {
            Expression::Identifier(name) => {
                if let Some(var_info) = self.variables.get(name) {
                    var_info.pointer.into()
                } else {
                    self.compile_expression(object)?
                }
            }
            _ => {
                let value = self.compile_expression(object)?;
                let alloca = self.builder.build_alloca(value.get_type(), "self_temp")?;
                self.builder.build_store(alloca, value)?;
                alloca.into()
            }
        };

        let mut compiled_args = vec![self_value];
        for arg in args {
            compiled_args.push(self.compile_expression(arg)?);
        }

        let args_meta: Vec<_> = compiled_args
            .iter()
            .map(|a| inkwell::values::BasicMetadataValueEnum::from(*a))
            .collect();

        let call = self
            .builder
            .build_call(function, &args_meta, "method_call")?;

        // Handle return value based on registered return type
        if is_void_method {
            // Void-returning method (like deallocate) - return a unit/dummy value
            // The caller should check context to know if result matters
            // This allows void methods to work as statements via trait dispatch
            let dummy = self.context.i32_type().const_int(0, false);
            Ok(Some(dummy.into()))
        } else {
            // Non-void method - return the actual value
            match call.try_as_basic_value().left() {
                Some(value) => Ok(Some(value)),
                None => Err(CompileError::InternalError(
                    format!(
                        "Trait method {}.{} should return a value but LLVM call returned void",
                        type_name, method_name
                    ),
                    self.get_current_span(),
                )),
            }
        }
    }

    fn try_qualified_method_call(
        &mut self,
        object: &Expression,
        type_name: &str,
        method_name: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
        let qualified = format!("{}.{}", type_name, method_name);
        let qualified_generic = format!("{}<T>.{}", type_name, method_name);

        let method_to_use = if self.function_types.contains_key(&qualified)
            || self.module.get_function(&qualified).is_some()
        {
            Some(qualified)
        } else if self.function_types.contains_key(&qualified_generic)
            || self.module.get_function(&qualified_generic).is_some()
        {
            Some(qualified_generic)
        } else {
            None
        };

        if method_to_use.is_some() {
            // Check if object is a type name (not a variable) - for static method calls
            let is_static_call = if let Expression::Identifier(id) = object {
                // It's static if the identifier is a type name, not a variable
                !self.variables.contains_key(id)
                    && (self.type_ctx.has_struct(id)
                        || self.type_ctx.has_enum(id)
                        || self.struct_types.contains_key(id))
            } else {
                false
            };

            let call_args: Vec<Expression> = if is_static_call {
                // Static call - don't pass the type as first arg
                args.to_vec()
            } else {
                // Instance method - pass object as first arg (UFC)
                let mut ufc_args = vec![object.clone()];
                ufc_args.extend_from_slice(args);
                ufc_args
            };

            if let Ok(result) = super::functions::calls::compile_function_call(
                self,
                Some(type_name),
                method_name,
                &call_args,
            ) {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    fn try_ufc_call(
        &mut self,
        object: &Expression,
        method_name: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
        if self.function_types.contains_key(method_name) {
            let mut ufc_args = vec![object.clone()];
            ufc_args.extend_from_slice(args);
            if let Ok(result) =
                super::functions::calls::compile_function_call(self, None, method_name, &ufc_args)
            {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    fn infer_type_name(&self, expr: &Expression) -> Result<String, CompileError> {
        match expr {
            Expression::Identifier(name) => {
                if let Some(var_info) = self.variables.get(name) {
                    // First try the actual type (important for Ptr<T>, MutPtr<T> methods)
                    if let Some(n) = Self::type_name_from_ast(&var_info.ast_type) {
                        return Ok(n);
                    }
                    // Fall back to inner type for pointer field access
                    if let Some(inner) = var_info.ast_type.ptr_inner() {
                        if let Some(n) = Self::type_name_from_ast(inner) {
                            return Ok(n);
                        }
                    }
                }
                // If identifier looks like a type expression (e.g., \"Ptr<i32>\"), parse it once
                if name.contains('<') {
                    if let Ok(parsed_type) = crate::parser::parse_type_from_string(name) {
                        // First try the actual type
                        if let Some(n) = Self::type_name_from_ast(&parsed_type) {
                            return Ok(n);
                        }
                        // Fall back to inner type
                        if let Some(inner) = parsed_type.ptr_inner() {
                            if let Some(n) = Self::type_name_from_ast(inner) {
                                return Ok(n);
                            }
                        }
                    }
                }
                if crate::intrinsics::well_known().get_type(name).is_some() {
                    return Ok(name.clone());
                }
                // Check if this is a known struct type (for static method calls like MyType.new())
                if self.type_ctx.has_struct(name) || self.struct_types.contains_key(name) {
                    return Ok(name.clone());
                }
                // Check if this is a known enum type
                if self.type_ctx.has_enum(name) {
                    return Ok(name.clone());
                }
                if let Ok(ast_type) = self.infer_expression_type(expr) {
                    // First try the actual type
                    if let Some(n) = Self::type_name_from_ast(&ast_type) {
                        return Ok(n);
                    }
                    // Fall back to inner type
                    if let Some(inner) = ast_type.ptr_inner() {
                        if let Some(n) = Self::type_name_from_ast(inner) {
                            return Ok(n);
                        }
                    }
                }
                Ok("UnknownType".to_string())
            }
            Expression::StructLiteral { name, .. } => Ok(name.clone()),
            _ => {
                if let Ok(ast_type) = self.infer_expression_type(expr) {
                    // First try the actual type
                    if let Some(n) = Self::type_name_from_ast(&ast_type) {
                        return Ok(n);
                    }
                    // Fall back to inner type
                    if let Some(inner) = ast_type.ptr_inner() {
                        if let Some(n) = Self::type_name_from_ast(inner) {
                            return Ok(n);
                        }
                    }
                }
                Ok("UnknownType".to_string())
            }
        }
    }
}
