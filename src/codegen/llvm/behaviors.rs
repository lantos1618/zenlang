use crate::ast::{ImplBlock, Expression};
use crate::error::CompileError;
use super::LLVMCompiler;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue};
use inkwell::types::BasicMetadataTypeEnum;
use std::collections::HashMap;

/// Manages behavior/trait implementations and method dispatch in LLVM
pub struct BehaviorCodegen<'ctx> {
    /// Maps (type_name, behavior_name) -> vtable global
    vtables: HashMap<(String, String), PointerValue<'ctx>>,
    /// Maps (type_name, method_name) -> function
    method_impls: HashMap<(String, String), FunctionValue<'ctx>>,
}

impl<'ctx> BehaviorCodegen<'ctx> {
    pub fn new() -> Self {
        Self {
            vtables: HashMap::new(),
            method_impls: HashMap::new(),
        }
    }

    /// Generate a vtable for a behavior implementation
    pub fn generate_vtable(
        &mut self,
        context: &'ctx inkwell::context::Context,
        module: &inkwell::module::Module<'ctx>,
        type_name: &str,
        behavior_name: &str,
        methods: &[(&str, FunctionValue<'ctx>)],
    ) -> Result<PointerValue<'ctx>, CompileError> {
        // Create vtable type: array of function pointers
        let fn_ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        let field_types: Vec<_> = (0..methods.len())
            .map(|_| fn_ptr_type.into())
            .collect();
        let vtable_type = context.struct_type(&field_types, false);

        // Create global vtable
        let vtable_name = format!("vtable_{}_{}", type_name, behavior_name);
        let vtable_global = module.add_global(vtable_type, None, &vtable_name);
        
        // Initialize vtable with method pointers
        let mut method_ptrs = Vec::new();
        for (_, func) in methods {
            let ptr = func.as_global_value().as_pointer_value();
            method_ptrs.push(ptr.const_cast(fn_ptr_type));
        }
        
        let method_values: Vec<BasicValueEnum> = method_ptrs
            .into_iter()
            .map(|ptr| ptr.into())
            .collect();
        let vtable_value = vtable_type.const_named_struct(&method_values);
        vtable_global.set_initializer(&vtable_value);
        
        let vtable_ptr = vtable_global.as_pointer_value();
        self.vtables.insert((type_name.to_string(), behavior_name.to_string()), vtable_ptr);
        
        Ok(vtable_ptr)
    }

    /// Register a method implementation
    pub fn register_method(
        &mut self,
        type_name: &str,
        method_name: &str,
        function: FunctionValue<'ctx>,
    ) {
        self.method_impls.insert(
            (type_name.to_string(), method_name.to_string()),
            function,
        );
    }

    /// Resolve a method call on a type
    pub fn resolve_method(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<FunctionValue<'ctx>> {
        self.method_impls.get(&(type_name.to_string(), method_name.to_string())).copied()
    }
}

impl<'ctx> LLVMCompiler<'ctx> {
    /// Compile an impl block
    pub fn compile_impl_block(&mut self, impl_block: &ImplBlock) -> Result<(), CompileError> {
        let type_name = &impl_block.type_name;
        
        // Process each method in the impl block
        for method in &impl_block.methods {
            // Generate a mangled name for the method
            let mangled_name = if let Some(behavior) = &impl_block.behavior_name {
                format!("{}_{}_{}", type_name, behavior, method.name)
            } else {
                format!("{}_{}", type_name, method.name)
            };

            // Create LLVM function for the method
            let llvm_return_type = self.to_llvm_type(&method.return_type)?;
            
            let mut param_types = Vec::new();
            for (_, param_type) in &method.args {
                let llvm_param_type = self.to_llvm_type(param_type)?;
                if let Ok(basic_type) = llvm_param_type.into_basic_type() {
                    param_types.push(BasicMetadataTypeEnum::from(basic_type));
                }
            }

            let fn_type = if let super::Type::Void = llvm_return_type {
                self.context.void_type().fn_type(&param_types, false)
            } else if let super::Type::Basic(basic_type) = llvm_return_type {
                match basic_type {
                    inkwell::types::BasicTypeEnum::IntType(int_type) => int_type.fn_type(&param_types, false),
                    inkwell::types::BasicTypeEnum::FloatType(float_type) => float_type.fn_type(&param_types, false),
                    inkwell::types::BasicTypeEnum::PointerType(ptr_type) => ptr_type.fn_type(&param_types, false),
                    inkwell::types::BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(&param_types, false),
                    _ => {
                        return Err(CompileError::UnsupportedFeature(
                            format!("Unsupported method return type: {:?}", basic_type),
                            None,
                        ))
                    }
                }
            } else {
                return Err(CompileError::UnsupportedFeature(
                    format!("Method return type not yet supported: {:?}", llvm_return_type),
                    None,
                ))
            };

            let function = self.module.add_function(&mangled_name, fn_type, None);
            
            // Set up the function body
            let entry = self.context.append_basic_block(function, "entry");
            self.builder.position_at_end(entry);
            
            // Store the current function
            let prev_function = self.current_function;
            self.current_function = Some(function);
            
            // Add parameters to symbol table
            self.symbols.enter_scope();
            for (i, (param_name, _)) in method.args.iter().enumerate() {
                if i < function.count_params() as usize {
                    let param_value = function.get_nth_param(i as u32).unwrap();
                    let alloca = self.builder.build_alloca(param_value.get_type(), param_name)?;
                    self.builder.build_store(alloca, param_value)?;
                    self.symbols.insert(param_name.clone(), super::symbols::Symbol::Variable(alloca));
                }
            }
            
            // Compile method body
            for stmt in &method.body {
                self.compile_statement(stmt)?;
            }
            
            // Add implicit return if needed
            if matches!(llvm_return_type, super::Type::Void) {
                if self.builder.get_insert_block().unwrap().get_terminator().is_none() {
                    self.builder.build_return(None)?;
                }
            }
            
            // Clean up
            self.symbols.exit_scope();
            self.current_function = prev_function;
            
            // Register the method in our behavior codegen
            if !self.behavior_codegen.is_some() {
                self.behavior_codegen = Some(BehaviorCodegen::new());
            }
            
            if let Some(ref mut behavior_codegen) = self.behavior_codegen {
                behavior_codegen.register_method(type_name, &method.name, function);
            }
        }
        
        // If this implements a behavior, generate vtable
        if let Some(behavior_name) = &impl_block.behavior_name {
            let mut methods = Vec::new();
            
            for method in &impl_block.methods {
                let mangled_name = format!("{}_{}_{}", type_name, behavior_name, method.name);
                if let Some(func) = self.module.get_function(&mangled_name) {
                    methods.push((method.name.as_str(), func));
                }
            }
            
            if let Some(ref mut behavior_codegen) = self.behavior_codegen {
                behavior_codegen.generate_vtable(self.context, &self.module, type_name, behavior_name, &methods)?;
            }
        }
        
        Ok(())
    }

    /// Compile a method call (e.g., obj.method(args))
    pub fn compile_method_call(
        &mut self,
        object: &Expression,
        method_name: &str,
        args: &[Expression],
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // For now, we'll implement static dispatch only
        // Dynamic dispatch would require trait objects
        
        // Get the type of the object
        // This is simplified - in a real implementation we'd need proper type tracking
        let type_name = self.infer_type_name(object)?;
        
        // Look up the method
        if let Some(ref behavior_codegen) = self.behavior_codegen {
            if let Some(function) = behavior_codegen.resolve_method(&type_name, method_name) {
                // Compile arguments
                let mut compiled_args = Vec::new();
                
                // First argument is 'self' - the object
                let self_value = self.compile_expression(object)?;
                compiled_args.push(self_value);
                
                // Compile remaining arguments
                for arg in args {
                    compiled_args.push(self.compile_expression(arg)?);
                }
                
                // Make the call
                let args_metadata: Vec<inkwell::values::BasicMetadataValueEnum> = compiled_args
                    .iter()
                    .map(|arg| inkwell::values::BasicMetadataValueEnum::try_from(*arg).unwrap())
                    .collect();
                
                let call_site = self.builder.build_call(function, &args_metadata, "method_call")?;
                
                return call_site.try_as_basic_value()
                    .left()
                    .ok_or_else(|| CompileError::TypeError(
                        "Method call returned void where value expected".to_string(),
                        None,
                    ));
            }
        }
        
        Err(CompileError::UndeclaredFunction(
            format!("{}.{}", type_name, method_name),
            None,
        ))
    }

    /// Helper to infer type name from an expression (simplified)
    fn infer_type_name(&self, expr: &Expression) -> Result<String, CompileError> {
        match expr {
            Expression::Identifier(_name) => {
                // Look up the variable's type in our type tracking
                // This is simplified - real implementation would track types properly
                Ok("UnknownType".to_string())
            }
            Expression::StructLiteral { name, .. } => Ok(name.clone()),
            _ => Ok("UnknownType".to_string()),
        }
    }
}