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
}

impl<'ctx> BehaviorCodegen<'ctx> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
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
            .map(|(_, func)| func.as_global_value().as_pointer_value().const_cast(fn_ptr_type).into())
            .collect();

        vtable_global.set_initializer(&vtable_type.const_named_struct(&method_values));
        let vtable_ptr = vtable_global.as_pointer_value();
        self.vtables.insert((type_name.to_string(), behavior_name.to_string()), vtable_ptr);
        Ok(vtable_ptr)
    }

    pub fn register_method(&mut self, type_name: &str, method_name: &str, function: FunctionValue<'ctx>) {
        self.method_impls.insert((type_name.to_string(), method_name.to_string()), function);
    }

    pub fn resolve_method(&self, type_name: &str, method_name: &str) -> Option<FunctionValue<'ctx>> {
        self.method_impls.get(&(type_name.to_string(), method_name.to_string())).copied()
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
            _ => return Err(CompileError::UnsupportedFeature(
                format!("Unsupported return type: {:?}", return_type), None
            )),
        })
    }

    /// Resolve Self type to concrete type
    fn resolve_self_type(&self, param_type: &AstType, type_name: &str, _type_params: &[crate::ast::TypeParameter]) -> AstType {
        match param_type {
            AstType::Generic { name, .. } if name == "Self" || name.starts_with("Self_") => {
                if let Some(struct_info) = self.struct_types.get(type_name) {
                    let fields: Vec<_> = struct_info.fields.iter()
                        .map(|(n, (_, t))| (n.clone(), t.clone()))
                        .collect();
                    AstType::Struct { name: type_name.to_string(), fields }
                } else {
                    AstType::Struct { name: type_name.to_string(), fields: vec![] }
                }
            }
            _ => param_type.clone(),
        }
    }

    /// Convert parameter to LLVM metadata type, handling structs by pointer
    fn param_to_metadata(&self, llvm_type: super::Type<'ctx>) -> Result<BasicMetadataTypeEnum<'ctx>, CompileError> {
        Ok(match llvm_type {
            super::Type::Basic(basic) => basic.into(),
            super::Type::Struct(_) => self.context.ptr_type(inkwell::AddressSpace::default()).into(),
            _ => return Err(CompileError::UnsupportedFeature(
                "Unsupported parameter type in method".to_string(), None
            )),
        })
    }

    /// Extract type name from AstType
    fn type_name_from_ast(ast_type: &AstType) -> Option<String> {
        match ast_type {
            AstType::Struct { name, .. } | AstType::Generic { name, .. }
            | AstType::Enum { name, .. } | AstType::EnumType { name } => Some(name.clone()),
            // TODO: These names should come from parsing stdlib, not hardcoded
            AstType::DynVec { .. } => Some("DynVec".to_string()),
            AstType::Vec { .. } => Some("Vec".to_string()),
            _ => None,
        }
    }
}

// ============================================================================
// Impl Block Compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_impl_block(&mut self, impl_block: &crate::ast::ImplBlock) -> Result<(), CompileError> {
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
                            type_args: impl_block.type_params.iter()
                                .map(|tp| AstType::Generic { name: tp.name.clone(), type_args: vec![] })
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
                bc.method_impls.insert((type_name.clone(), method.name.clone()), function);
            }
        }

        Ok(())
    }
}

// ============================================================================
// Trait Implementation Compilation
// ============================================================================

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn compile_trait_implementation(&mut self, trait_impl: &TraitImplementation) -> Result<(), CompileError> {
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

                let llvm_param = if param_name == "self" || matches!(actual_type, AstType::Struct { .. }) {
                    let st = self.to_llvm_type(&actual_type)?;
                    if matches!(st, super::Type::Struct(_)) {
                        super::Type::Basic(self.context.ptr_type(inkwell::AddressSpace::default()).into())
                    } else { st }
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
            for (i, (param_name, param_type)) in method.args.iter().enumerate() {
                if i < function.count_params() as usize {
                    let param_value = function.get_nth_param(i as u32).ok_or_else(|| {
                        CompileError::InternalError(
                            format!("Missing parameter {} in method", i),
                            self.get_current_span(),
                        )
                    })?;
                    let alloca = self.builder.build_alloca(param_value.get_type(), param_name)?;
                    self.builder.build_store(alloca, param_value)?;
                    self.symbols.insert(param_name.clone(), super::symbols::Symbol::Variable(alloca));

                    let actual_type = if param_name == "self" {
                        let resolved = self.resolve_self_type(param_type, type_name, &[]);
                        AstType::ptr(resolved)
                    } else {
                        param_type.clone()
                    };

                    self.variables.insert(param_name.clone(), super::VariableInfo {
                        pointer: alloca,
                        ast_type: actual_type,
                        is_mutable: false,
                        is_initialized: true,
                        definition_span: self.get_current_span(),
                    });
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
            }
        }

        // Generate vtable
        let methods: Vec<_> = trait_impl.methods.iter()
            .filter_map(|m| {
                let name = format!("{}_{}_{}", type_name, trait_name, m.name);
                self.module.get_function(&name).map(|f| (m.name.as_str(), f))
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
                    let qualified = format!("{}.{}", name, method_name);
                    return super::functions::calls::compile_function_call(self, &qualified, args);
                }
            }
        }

        // Handle Range constructors (Range.new, Range.with_step)
        if let Expression::Identifier(name) = object {
            if name == "Range" && (method_name == "new" || method_name == "with_step") {
                let qualified = format!("Range.{}", method_name);
                return super::functions::calls::compile_function_call(self, &qualified, args);
            }
        }

        // HashMap methods now use stdlib Zen implementation via normal resolution

        // Handle Range methods
        if let Some(result) = self.try_compile_range_method(object, method_name, args)? {
            return Ok(result);
        }

        // Try behavior codegen dispatch
        let type_name = self.infer_type_name(object)?;
        if let Some(result) = self.try_behavior_dispatch(object, &type_name, method_name, args)? {
            return Ok(result);
        }

        // Try qualified method name
        if let Some(result) = self.try_qualified_method_call(object, &type_name, method_name, args)? {
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

    fn compile_std_method_call(&mut self, method_name: &str, args: &[Expression]) -> Result<BasicValueEnum<'ctx>, CompileError> {
        if let Ok(result) = super::functions::calls::compile_function_call(self, method_name, args) {
            return Ok(result);
        }
        let qualified = format!("Std.{}", method_name);
        super::functions::calls::compile_function_call(self, &qualified, args)
    }

    // HashMap methods removed - now use stdlib Zen implementation

    fn try_compile_range_method(
        &mut self,
        object: &Expression,
        method_name: &str,
        _args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
        // Handle direct identifier (e.g., range.has_next())
        let range_ptr = if let Expression::Identifier(name) = object {
            self.variables.get(name).and_then(|v| {
                if let AstType::Struct { name: tn, .. } = &v.ast_type {
                    if tn == "Range" {
                        return Some(v.pointer);
                    }
                }
                None
            })
        // Handle CreateMutableReference (e.g., range.mut_ref().next())
        } else if let Expression::CreateMutableReference(inner_obj) = object {
            if let Expression::Identifier(name) = inner_obj.as_ref() {
                self.variables.get(name).and_then(|v| {
                    if let AstType::Struct { name: tn, .. } = &v.ast_type {
                        if tn == "Range" {
                            return Some(v.pointer);
                        }
                    }
                    None
                })
            } else {
                None
            }
        } else {
            None
        };

        let Some(range_ptr) = range_ptr else { return Ok(None) };

        // Ensure Range struct is registered from stdlib
        self.ensure_struct_type("Range")?;

        // Range struct: { current: i64, end: i64, step: i64 }
        let i64_type = self.context.i64_type();

        // Get the registered struct type (should be available after ensure_struct_type)
        let range_struct_type = self.struct_types.get("Range")
            .map(|info| info.llvm_type)
            .unwrap_or_else(|| self.context.struct_type(
                &[i64_type.into(), i64_type.into(), i64_type.into()],
                false,
            ));

        match method_name {
            "has_next" => {
                // return self.current < self.end
                let current_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 0, "current_ptr")?;
                let end_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 1, "end_ptr")?;
                let current = self.builder.build_load(i64_type, current_ptr, "current")?
                    .into_int_value();
                let end = self.builder.build_load(i64_type, end_ptr, "end")?
                    .into_int_value();
                let result = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLT, current, end, "has_next")?;
                Ok(Some(result.into()))
            }
            "count" => {
                // return (self.end - self.current) / self.step
                let current_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 0, "current_ptr")?;
                let end_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 1, "end_ptr")?;
                let step_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 2, "step_ptr")?;
                let current = self.builder.build_load(i64_type, current_ptr, "current")?
                    .into_int_value();
                let end = self.builder.build_load(i64_type, end_ptr, "end")?
                    .into_int_value();
                let step = self.builder.build_load(i64_type, step_ptr, "step")?
                    .into_int_value();

                // Check if end > current
                let is_positive = self.builder.build_int_compare(
                    inkwell::IntPredicate::SGT, end, current, "is_positive")?;

                let diff = self.builder.build_int_sub(end, current, "diff")?;
                let count = self.builder.build_int_signed_div(diff, step, "count")?;

                let zero = i64_type.const_int(0, false);
                let result = self.builder.build_select(is_positive, count, zero, "count_result")?;
                Ok(Some(result))
            }
            "next" => {
                // Range.next() returns Option<i64>
                // if current < end { Some(current), current += step } else { None }

                // Get current block and function for branching
                let current_fn = self.builder.get_insert_block()
                    .ok_or_else(|| CompileError::InternalError("No current block".to_string(), None))?
                    .get_parent()
                    .ok_or_else(|| CompileError::InternalError("No parent function".to_string(), None))?;

                // Load current and end values
                let current_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 0, "current_ptr")?;
                let end_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 1, "end_ptr")?;
                let step_ptr = self.builder.build_struct_gep(
                    range_struct_type, range_ptr, 2, "step_ptr")?;

                let current = self.builder.build_load(i64_type, current_ptr, "current")?
                    .into_int_value();
                let end = self.builder.build_load(i64_type, end_ptr, "end")?
                    .into_int_value();
                let step = self.builder.build_load(i64_type, step_ptr, "step")?
                    .into_int_value();

                // Create blocks for branching
                let has_next_block = self.context.append_basic_block(current_fn, "range_has_next");
                let exhausted_block = self.context.append_basic_block(current_fn, "range_exhausted");
                let merge_block = self.context.append_basic_block(current_fn, "range_merge");

                // Compare current < end
                let is_valid = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLT, current, end, "is_valid")?;
                self.builder.build_conditional_branch(is_valid, has_next_block, exhausted_block)?;

                // has_next block: return Some(current), advance current
                self.builder.position_at_end(has_next_block);
                let new_current = self.builder.build_int_add(current, step, "new_current")?;
                self.builder.build_store(current_ptr, new_current)?;

                // Create Option.Some(current) - enum struct: { discriminant: i64, payload_ptr: ptr }
                let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
                let option_struct_type = self.context.struct_type(
                    &[i64_type.into(), ptr_type.into()],
                    false,
                );

                // Heap allocate the payload (i64)
                let malloc_fn = self.module.get_function("malloc").unwrap_or_else(|| {
                    let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
                    self.module.add_function("malloc", malloc_type, None)
                });
                let payload_size = i64_type.const_int(8, false); // 8 bytes for i64
                let malloc_call = self.builder.build_call(malloc_fn, &[payload_size.into()], "payload_heap")?;
                let payload_heap_ptr = malloc_call
                    .try_as_basic_value()
                    .left()
                    .ok_or_else(|| {
                        CompileError::InternalError(
                            "malloc must return a value".to_string(),
                            self.get_current_span(),
                        )
                    })?
                    .into_pointer_value();
                self.builder.build_store(payload_heap_ptr, current)?;

                let some_alloca = self.builder.build_alloca(option_struct_type, "option_some")?;
                let some_disc_ptr = self.builder.build_struct_gep(
                    option_struct_type, some_alloca, 0, "some_disc")?;
                let some_payload_ptr = self.builder.build_struct_gep(
                    option_struct_type, some_alloca, 1, "some_payload_ptr")?;
                self.builder.build_store(some_disc_ptr, i64_type.const_int(0, false))?; // Some = 0 (first variant)
                self.builder.build_store(some_payload_ptr, payload_heap_ptr)?;
                let some_value = self.builder.build_load(option_struct_type, some_alloca, "some_val")?;
                self.builder.build_unconditional_branch(merge_block)?;

                // exhausted block: return None
                self.builder.position_at_end(exhausted_block);
                let none_alloca = self.builder.build_alloca(option_struct_type, "option_none")?;
                let none_disc_ptr = self.builder.build_struct_gep(
                    option_struct_type, none_alloca, 0, "none_disc")?;
                let none_payload_ptr = self.builder.build_struct_gep(
                    option_struct_type, none_alloca, 1, "none_payload_ptr")?;
                self.builder.build_store(none_disc_ptr, i64_type.const_int(1, false))?; // None = 1 (second variant)
                // Store null pointer for None's payload
                let null_ptr = ptr_type.const_null();
                self.builder.build_store(none_payload_ptr, null_ptr)?;
                let none_value = self.builder.build_load(option_struct_type, none_alloca, "none_val")?;
                self.builder.build_unconditional_branch(merge_block)?;

                // Merge block with PHI
                self.builder.position_at_end(merge_block);
                let phi = self.builder.build_phi(option_struct_type, "range_next_result")?;
                phi.add_incoming(&[
                    (&some_value, has_next_block),
                    (&none_value, exhausted_block),
                ]);

                Ok(Some(phi.as_basic_value()))
            }
            _ => Ok(None),
        }
    }

    fn try_behavior_dispatch(
        &mut self,
        object: &Expression,
        type_name: &str,
        method_name: &str,
        args: &[Expression],
    ) -> Result<Option<BasicValueEnum<'ctx>>, CompileError> {
        let Some(ref bc) = self.behavior_codegen else { return Ok(None) };
        let Some(function) = bc.resolve_method(type_name, method_name) else { return Ok(None) };

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

        let args_meta: Vec<_> = compiled_args.iter()
            .map(|a| inkwell::values::BasicMetadataValueEnum::from(*a))
            .collect();

        let call = self.builder.build_call(function, &args_meta, "method_call")?;
        call.try_as_basic_value().left().ok_or_else(|| {
            CompileError::TypeError("Method call returned void where value expected".to_string(), self.get_current_span())
        }).map(Some)
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

        let method_to_use = if self.function_types.contains_key(&qualified) || self.module.get_function(&qualified).is_some() {
            Some(qualified)
        } else if self.function_types.contains_key(&qualified_generic) || self.module.get_function(&qualified_generic).is_some() {
            Some(qualified_generic)
        } else {
            None
        };

        if let Some(name) = method_to_use {
            let mut ufc_args = vec![object.clone()];
            ufc_args.extend_from_slice(args);
            if let Ok(result) = super::functions::calls::compile_function_call(self, &name, &ufc_args) {
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
            if let Ok(result) = super::functions::calls::compile_function_call(self, method_name, &ufc_args) {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }

    fn infer_type_name(&self, expr: &Expression) -> Result<String, CompileError> {
        match expr {
            Expression::Identifier(name) => {
                if let Some(var_info) = self.variables.get(name) {
                    let effective = var_info.ast_type.ptr_inner().unwrap_or(&var_info.ast_type);
                    if let Some(n) = Self::type_name_from_ast(effective) {
                        return Ok(n);
                    }
                }
                // Check for type expression like "Ptr<i32>"
                if let Some(pos) = name.find('<') {
                    return Ok(name[..pos].to_string());
                }
                if crate::well_known::well_known().get_type(name).is_some() {
                    return Ok(name.clone());
                }
                if let Ok(ast_type) = self.infer_expression_type(expr) {
                    let effective = ast_type.ptr_inner().unwrap_or(&ast_type);
                    if let Some(n) = Self::type_name_from_ast(effective) {
                        return Ok(n);
                    }
                }
                Ok("UnknownType".to_string())
            }
            Expression::StructLiteral { name, .. } => Ok(name.clone()),
            _ => {
                if let Ok(ast_type) = self.infer_expression_type(expr) {
                    let effective = ast_type.ptr_inner().unwrap_or(&ast_type);
                    if let Some(n) = Self::type_name_from_ast(effective) {
                        return Ok(n);
                    }
                }
                Ok("UnknownType".to_string())
            }
        }
    }
}
