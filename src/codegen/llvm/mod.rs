#![allow(dead_code)]

use crate::ast::{self, AstType};
use crate::comptime;
use crate::error::{CompileError, Span};
use crate::stdlib_types::stdlib_types;
use crate::type_context::TypeContext;
use crate::well_known::WellKnownTypes;
use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, FunctionType, StructType},
    values::{BasicValueEnum, FunctionValue, PointerValue},
};
use std::collections::HashMap;

mod behaviors;
mod binary_ops;
mod builtins;
mod expressions;
mod functions;
mod generics;
mod literals;
mod patterns;
mod pointers;
mod statements;
mod stdlib_codegen;
mod structs;
mod symbols;
mod types;

#[derive(Debug, Clone)]
pub enum Type<'ctx> {
    Basic(BasicTypeEnum<'ctx>),
    Pointer(Box<Type<'ctx>>),
    Struct(StructType<'ctx>),
    Function(FunctionType<'ctx>),
    Void,
}

impl<'ctx> Type<'ctx> {
    pub fn into_basic_type(self) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        match self {
            Type::Basic(t) => Ok(t),
            // Note: This method doesn't have access to current_span, so we leave it as None
            // The caller should use add_span_to_error if needed
            _ => Err(CompileError::TypeMismatch {
                expected: "basic type".to_string(),
                found: format!("{:?}", self),
                span: None,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StructTypeInfo<'ctx> {
    pub llvm_type: StructType<'ctx>,
    pub fields: HashMap<String, (usize, AstType)>,
}

// Variable information with mutability tracking
#[derive(Debug, Clone)]
pub struct VariableInfo<'ctx> {
    pub pointer: PointerValue<'ctx>,
    pub ast_type: AstType,
    pub is_mutable: bool,
    pub is_initialized: bool,
    /// The source location where this variable was defined.
    /// Used for "previous declaration was here" diagnostics.
    pub definition_span: Option<Span>,
}

pub struct LLVMCompiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub variables: HashMap<String, VariableInfo<'ctx>>,
    pub functions: HashMap<String, FunctionValue<'ctx>>,
    pub function_types: HashMap<String, AstType>,
    pub current_function: Option<FunctionValue<'ctx>>,
    pub symbols: symbols::SymbolTable<'ctx>,
    pub struct_types: HashMap<String, StructTypeInfo<'ctx>>,
    pub loop_stack: Vec<(BasicBlock<'ctx>, BasicBlock<'ctx>)>,
    pub defer_stack: Vec<ast::Expression>,
    pub comptime_evaluator: comptime::ComptimeInterpreter,
    pub behavior_codegen: Option<behaviors::BehaviorCodegen<'ctx>>,
    pub current_impl_type: Option<String>,
    pub inline_counter: usize,
    pub load_counter: usize,
    pub generic_type_context: HashMap<String, AstType>,
    pub generic_tracker: generics::GenericTypeTracker,
    pub module_imports: HashMap<String, u64>,
    pub current_span: Option<Span>,
    pub well_known: WellKnownTypes,
    /// Type context from typechecker - use this for type lookups instead of re-inferring
    pub type_ctx: TypeContext,
}

impl<'ctx> LLVMCompiler<'ctx> {
    // ============================================================================
    // SPAN TRACKING HELPERS
    // These methods help propagate source location information to error messages
    // ============================================================================

    /// Set the current span for error reporting
    pub fn set_span(&mut self, span: Option<Span>) {
        self.current_span = span;
    }

    /// Get the current span for error reporting
    pub fn get_current_span(&self) -> Option<Span> {
        self.current_span.clone()
    }

    /// Create an error with the current span context
    pub fn error_with_span(&self, error: CompileError) -> CompileError {
        if self.current_span.is_some() {
            self.add_span_to_error(error)
        } else {
            error
        }
    }

    /// Add the current span to an error if it doesn't already have one
    fn add_span_to_error(&self, error: CompileError) -> CompileError {
        match error {
            CompileError::UndeclaredVariable(name, None) => {
                CompileError::UndeclaredVariable(name, self.current_span.clone())
            }
            CompileError::UndeclaredFunction(name, None) => {
                CompileError::UndeclaredFunction(name, self.current_span.clone())
            }
            CompileError::TypeMismatch {
                expected,
                found,
                span: None,
            } => CompileError::TypeMismatch {
                expected,
                found,
                span: self.current_span.clone(),
            },
            CompileError::InternalError(msg, None) => {
                CompileError::InternalError(msg, self.current_span.clone())
            }
            CompileError::UnsupportedFeature(msg, None) => {
                CompileError::UnsupportedFeature(msg, self.current_span.clone())
            }
            CompileError::SyntaxError(msg, None) => {
                CompileError::SyntaxError(msg, self.current_span.clone())
            }
            CompileError::TypeError(msg, None) => {
                CompileError::TypeError(msg, self.current_span.clone())
            }
            // If error already has a span, keep it
            other => other,
        }
    }

    // ============================================================================
    // BLOCK AND FUNCTION CONTEXT HELPERS
    // Safe accessors for current compilation context
    // ============================================================================

    /// Get current basic block with proper error handling
    /// Use this instead of `builder.get_insert_block().unwrap()`
    pub fn current_block(&self) -> Result<BasicBlock<'ctx>, CompileError> {
        self.builder.get_insert_block().ok_or_else(|| {
            CompileError::InternalError(
                "No current block - builder not positioned".to_string(),
                self.current_span.clone(),
            )
        })
    }

    /// Get current function with proper error handling
    /// Use this instead of `current_function.unwrap()`
    pub fn current_fn(&self) -> Result<FunctionValue<'ctx>, CompileError> {
        self.current_function.ok_or_else(|| {
            CompileError::InternalError(
                "No current function context".to_string(),
                self.current_span.clone(),
            )
        })
    }

    // ============================================================================
    // PATTERN MATCHING
    // Basic pattern matching implementation for common cases
    // ============================================================================

    /// Get module ID for import tracking
    fn get_module_id(&self, module_name: &str) -> u64 {
        if self.well_known.is_option(module_name) || self.well_known.is_option_variant(module_name)
        {
            100
        } else if self.well_known.is_result(module_name)
            || self.well_known.is_result_variant(module_name)
        {
            101
        } else {
            // Query stdlib_types for dynamic lookup from parsed .zen files
            // Fall back to primitives for bootstrap/static analysis
            let stdlib = stdlib_types();

            // Collection types (dynamically discovered from stdlib)
            if stdlib.is_collection_type(module_name) {
                return 102;
            }
            // Allocator-related (dynamically discovered from stdlib/memory/*.zen)
            if stdlib.is_allocator_type(module_name) {
                return 103;
            }
            // Math functions (dynamically discovered from stdlib/math.zen)
            if stdlib.is_math_function(module_name) {
                return 104;
            }
            // Stdlib module priorities
            match module_name {
                "io" => 1,
                "math" => 2,
                "core" => 3,
                "build" => 7,
                _ => 0,
            }
        }
    }

    // Pattern matching code moved to patterns.rs module

    /// Helper to track generic types in both old and new systems
    pub fn track_generic_type(&mut self, key: String, type_: AstType) {
        self.generic_type_context.insert(key.clone(), type_.clone());
        self.generic_tracker.insert(key, type_);
    }

    /// Helper to track complex generic types recursively
    pub fn track_complex_generic(&mut self, type_: &AstType, prefix: &str) {
        self.generic_tracker.track_generic_type(type_, prefix);

        // Also update the old system for backwards compatibility
        if let AstType::Generic { name, type_args } = type_ {
            if self.well_known.is_result(name) && type_args.len() == 2 {
                self.generic_type_context
                    .insert(format!("{}_Ok_Type", prefix), type_args[0].clone());
                self.generic_type_context
                    .insert(format!("{}_Err_Type", prefix), type_args[1].clone());
            } else if self.well_known.is_option(name) && type_args.len() == 1 {
                self.generic_type_context
                    .insert(format!("{}_Some_Type", prefix), type_args[0].clone());
            }
        }
    }

    pub fn new(context: &'ctx Context, type_ctx: TypeContext) -> Self {
        let module = context.create_module("main");
        let builder = context.create_builder();
        let mut symbols = symbols::SymbolTable::new();
        let comptime_evaluator = comptime::ComptimeInterpreter::new();

        let i64_type = context.i64_type();
        let i32_type = context.i32_type();
        let float_type = context.f64_type();
        let bool_type = context.bool_type();

        symbols.insert("i64", symbols::Symbol::Type(i64_type.as_basic_type_enum()));
        symbols.insert("i32", symbols::Symbol::Type(i32_type.as_basic_type_enum()));
        symbols.insert(
            "f64",
            symbols::Symbol::Type(float_type.as_basic_type_enum()),
        );
        symbols.insert(
            "bool",
            symbols::Symbol::Type(bool_type.as_basic_type_enum()),
        );

        let mut compiler = Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            functions: HashMap::new(),
            function_types: HashMap::new(),
            current_function: None,
            symbols,
            struct_types: HashMap::new(),
            loop_stack: Vec::new(),
            defer_stack: Vec::new(),
            comptime_evaluator,
            behavior_codegen: Some(behaviors::BehaviorCodegen::new()),
            current_impl_type: None,
            inline_counter: 0,
            load_counter: 0,
            generic_type_context: HashMap::new(),
            generic_tracker: generics::GenericTypeTracker::new(),
            module_imports: HashMap::new(),
            current_span: None,
            well_known: WellKnownTypes::new(),
            type_ctx,
        };

        // Auto-inject built-in modules (always available without explicit import)
        for (name, id) in crate::intrinsics::BUILTIN_MODULES {
            compiler.module_imports.insert(name.to_string(), *id);
        }

        // Declare standard library functions
        compiler.declare_stdlib_functions();

        // Register built-in Option and Result enums
        compiler.register_builtin_enums();

        compiler
    }

    pub fn get_type(&self, name: &str) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        self.symbols
            .lookup(name)
            .and_then(|sym| match sym {
                symbols::Symbol::Type(ty) => Some(*ty),
                _ => None,
            })
            .ok_or_else(|| {
                CompileError::UndeclaredVariable(name.to_string(), self.current_span.clone())
            })
    }

    /// Get the target-specific pointer-sized integer type (usize).
    /// Currently hardcoded to i64 for 64-bit platforms (Linux x86_64).
    /// TODO: Use TargetData for proper cross-platform support when needed.
    pub fn ptr_sized_int_type(&self) -> inkwell::types::IntType<'ctx> {
        // For 64-bit Linux (our primary target), pointers are 64 bits
        self.context.i64_type()
    }

    /// Get the discriminant type for enums based on variant count.
    /// Uses the smallest integer type that can represent all variants.
    /// This is centralized to ensure consistent enum layouts across the codebase.
    pub fn enum_discriminant_type(&self, variant_count: usize) -> inkwell::types::IntType<'ctx> {
        if variant_count <= 256 {
            self.context.i8_type()
        } else if variant_count <= 65536 {
            self.context.i16_type()
        } else if variant_count <= 4_294_967_296 {
            self.context.i32_type()
        } else {
            self.context.i64_type()
        }
    }

    /// Get the standard struct type for well-known enums (Option, Result).
    /// These always have 2 variants, so they use i8 discriminant + ptr payload.
    /// This is centralized to ensure all Option/Result types have matching layouts.
    pub fn well_known_enum_type(&self) -> inkwell::types::StructType<'ctx> {
        let discriminant = self.enum_discriminant_type(2); // Option/Result have 2 variants
        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        self.context
            .struct_type(&[discriminant.into(), ptr_type.into()], false)
    }

    // ============================================================================
    // ENUM HELPER FUNCTIONS
    // Centralized helpers to reduce code duplication in enum handling
    // ============================================================================

    /// Extract the discriminant (tag) type from an enum struct type.
    /// The discriminant is always the first field of the enum struct.
    #[inline]
    pub fn get_enum_discriminant_type(
        &self,
        enum_struct_type: inkwell::types::StructType<'ctx>,
    ) -> inkwell::types::IntType<'ctx> {
        match enum_struct_type.get_field_type_at_index(0) {
            Some(field_type) => field_type.into_int_type(),
            None => {
                // Fallback: use i32 if struct has no fields (shouldn't happen for valid enums)
                eprintln!(
                    "Warning: Enum struct type has no discriminant field, using i32 fallback"
                );
                self.context.i32_type()
            }
        }
    }

    /// Create a constant tag value for an enum discriminant.
    /// Uses the correct integer type for the given enum struct.
    #[inline]
    pub fn enum_tag_const(
        &self,
        enum_struct_type: inkwell::types::StructType<'ctx>,
        tag: u64,
    ) -> inkwell::values::IntValue<'ctx> {
        self.get_enum_discriminant_type(enum_struct_type)
            .const_int(tag, false)
    }

    /// Load the discriminant (tag) value from an enum pointer.
    /// Returns the loaded tag as an IntValue with the correct discriminant type.
    pub fn load_enum_tag(
        &self,
        enum_struct_type: inkwell::types::StructType<'ctx>,
        enum_ptr: inkwell::values::PointerValue<'ctx>,
        name: &str,
    ) -> Result<inkwell::values::IntValue<'ctx>, CompileError> {
        let tag_ptr = self.builder.build_struct_gep(
            enum_struct_type,
            enum_ptr,
            0,
            &format!("{}_tag_ptr", name),
        )?;
        let discriminant_type = self.get_enum_discriminant_type(enum_struct_type);
        let tag_val =
            self.builder
                .build_load(discriminant_type, tag_ptr, &format!("{}_tag", name))?;
        Ok(tag_val.into_int_value())
    }

    /// Store a tag value into an enum's discriminant field.
    pub fn store_enum_tag(
        &self,
        enum_struct_type: inkwell::types::StructType<'ctx>,
        enum_ptr: inkwell::values::PointerValue<'ctx>,
        tag: u64,
        name: &str,
    ) -> Result<(), CompileError> {
        let tag_ptr = self.builder.build_struct_gep(
            enum_struct_type,
            enum_ptr,
            0,
            &format!("{}_tag_ptr", name),
        )?;
        let tag_val = self.enum_tag_const(enum_struct_type, tag);
        self.builder.build_store(tag_ptr, tag_val)?;
        Ok(())
    }

    /// Get a pointer to the payload field of an enum.
    pub fn get_enum_payload_ptr(
        &self,
        enum_struct_type: inkwell::types::StructType<'ctx>,
        enum_ptr: inkwell::values::PointerValue<'ctx>,
        name: &str,
    ) -> Result<inkwell::values::PointerValue<'ctx>, CompileError> {
        Ok(self.builder.build_struct_gep(
            enum_struct_type,
            enum_ptr,
            1,
            &format!("{}_payload_ptr", name),
        )?)
    }

    // ============================================================================
    // TYPE-SAFE IR GENERATION HELPERS
    // These catch type mismatches at compile time instead of causing runtime segfaults
    // ============================================================================

    /// Type-safe store that verifies the value type matches the expected type.
    /// This prevents bugs like storing i64 into an i32 alloca.
    pub fn verified_store(
        &self,
        value: BasicValueEnum<'ctx>,
        ptr: PointerValue<'ctx>,
        expected_type: BasicTypeEnum<'ctx>,
        context: &str, // For error messages, e.g., "variable 'x'" or "struct field 'name'"
    ) -> Result<(), CompileError> {
        let value_type = value.get_type();

        // Check for type mismatch
        let mismatch = match (value_type, expected_type) {
            (BasicTypeEnum::IntType(vt), BasicTypeEnum::IntType(et)) => {
                vt.get_bit_width() != et.get_bit_width()
            }
            (BasicTypeEnum::FloatType(vt), BasicTypeEnum::FloatType(et)) => {
                // Compare by checking if they're the same type
                vt != et
            }
            (BasicTypeEnum::PointerType(_), BasicTypeEnum::PointerType(_)) => {
                // Opaque pointers are always compatible
                false
            }
            (BasicTypeEnum::StructType(vt), BasicTypeEnum::StructType(et)) => {
                // Struct types should match exactly
                vt != et
            }
            (BasicTypeEnum::ArrayType(vt), BasicTypeEnum::ArrayType(et)) => vt != et,
            (BasicTypeEnum::VectorType(vt), BasicTypeEnum::VectorType(et)) => vt != et,
            // Different type categories = mismatch
            _ => {
                // Special case: pointer and int can be compatible in some contexts
                // But generally different categories are a mismatch
                !matches!(
                    (&value_type, &expected_type),
                    (BasicTypeEnum::PointerType(_), BasicTypeEnum::IntType(_))
                        | (BasicTypeEnum::IntType(_), BasicTypeEnum::PointerType(_))
                )
            }
        };

        if mismatch {
            return Err(CompileError::InternalError(
                format!(
                    "LLVM IR type mismatch in store for {}: value has type {:?} but storage expects {:?}. \
                     This is a compiler bug - please report it.",
                    context,
                    value_type,
                    expected_type
                ),
                None,
            ));
        }

        self.builder
            .build_store(ptr, value)
            .map_err(CompileError::from)?;
        Ok(())
    }

    /// Type-safe store with automatic type coercion for integers.
    /// If value is an integer and sizes don't match, it will truncate or extend as needed.
    /// Returns the (possibly coerced) value that was stored.
    pub fn coercing_store(
        &self,
        value: BasicValueEnum<'ctx>,
        ptr: PointerValue<'ctx>,
        expected_type: BasicTypeEnum<'ctx>,
        _context: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        let final_value = if let BasicValueEnum::IntValue(int_val) = value {
            if let BasicTypeEnum::IntType(expected_int_type) = expected_type {
                let val_bits = int_val.get_type().get_bit_width();
                let expected_bits = expected_int_type.get_bit_width();

                if val_bits > expected_bits {
                    // Truncate
                    self.builder
                        .build_int_truncate(int_val, expected_int_type, "trunc")
                        .map_err(CompileError::from)?
                        .into()
                } else if val_bits < expected_bits {
                    // Zero-extend
                    self.builder
                        .build_int_z_extend(int_val, expected_int_type, "zext")
                        .map_err(CompileError::from)?
                        .into()
                } else {
                    value
                }
            } else {
                value
            }
        } else {
            value
        };

        self.builder
            .build_store(ptr, final_value)
            .map_err(CompileError::from)?;
        Ok(final_value)
    }

    /// Type-safe load that returns a value with the correct type.
    /// Uses a unique name counter to avoid LLVM naming conflicts.
    pub fn verified_load(
        &mut self,
        ptr: PointerValue<'ctx>,
        expected_type: BasicTypeEnum<'ctx>,
        name_hint: &str,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        self.load_counter += 1;
        let name = format!("{}_{}", name_hint, self.load_counter);

        self.builder
            .build_load(expected_type, ptr, &name)
            .map_err(CompileError::from)
    }

    /// Debug helper: Print type information for troubleshooting IR generation issues
    #[allow(dead_code)]
    pub fn debug_type_info(&self, label: &str, value: BasicValueEnum<'ctx>) {
        if std::env::var("DEBUG_TYPES").is_ok() {
            eprintln!("[DEBUG_TYPES] {}: {:?}", label, value.get_type());
        }
    }

    pub fn declare_variable(
        &mut self,
        name: &str,
        _ty: AstType,
        ptr: PointerValue<'ctx>,
    ) -> Result<(), CompileError> {
        let symbol = symbols::Symbol::Variable(ptr);
        if self.symbols.exists_in_current_scope(name) {
            return Err(CompileError::UndeclaredVariable(
                name.to_string(),
                self.current_span.clone(),
            ));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    pub fn get_variable(&self, name: &str) -> Result<(PointerValue<'ctx>, AstType), CompileError> {
        // First check the HashMap-based variables (main storage)
        if let Some(var_info) = self.variables.get(name) {
            return Ok((var_info.pointer, var_info.ast_type.clone()));
        }

        // Then check the SymbolTable (used in trait methods and other contexts)
        if let Some(symbols::Symbol::Variable(ptr)) = self.symbols.lookup(name) {
            // We don't have type info in symbols, so use a generic type
            // This is primarily for 'self' in trait methods
            let ty = if name == "self" {
                // For 'self', we should have the struct type
                // This is a workaround - ideally we'd store the type in symbols
                AstType::Struct {
                    name: String::new(), // Will be resolved in context
                    fields: vec![],
                }
            } else {
                AstType::Void // Generic fallback
            };
            return Ok((*ptr, ty));
        }

        // Check if it's a function
        if let Some(function) = self.module.get_function(name) {
            let ptr = function.as_global_value().as_pointer_value();
            let ty = AstType::ptr(AstType::Function {
                args: vec![],
                return_type: Box::new(AstType::Void),
            });
            return Ok((ptr, ty));
        }

        Err(CompileError::UndeclaredVariable(
            name.to_string(),
            self.current_span.clone(),
        ))
    }

    pub fn compile_program(&mut self, program: &ast::Program) -> Result<(), CompileError> {
        // First pass: register all struct types (may have forward references)
        // We do this in two sub-passes:
        // 1. Register all structs with their names (so they can be looked up)
        // 2. Then resolve field types (which may reference other structs)

        // Sub-pass 1: Register all struct names first
        let struct_defs: Vec<_> = program
            .declarations
            .iter()
            .filter_map(|d| {
                if let ast::Declaration::Struct(struct_def) = d {
                    Some(struct_def)
                } else {
                    None
                }
            })
            .collect();

        // Sub-pass 2: Now register structs with resolved field types
        for struct_def in &struct_defs {
            self.register_struct_type(struct_def)?;
        }

        // Register enum types
        for declaration in &program.declarations {
            if let ast::Declaration::Enum(enum_def) = declaration {
                self.register_enum_type(enum_def)?;
            }
        }

        for declaration in &program.declarations {
            match declaration {
                ast::Declaration::ExternalFunction(ext_func) => {
                    self.declare_external_function(ext_func)?;
                }
                ast::Declaration::Function(_) => {}
                ast::Declaration::Struct(_) => {} // Already handled above
                ast::Declaration::Enum(_) => {}   // Already handled above
                ast::Declaration::Export { .. } => {
                    // Exports are handled at module level, no codegen needed
                }
                ast::Declaration::ModuleImport {
                    alias, module_path, ..
                } => {
                    let module_name = module_path.split('.').next_back().unwrap_or(alias);
                    let module_id = self.get_module_id(module_name);
                    self.module_imports.insert(alias.clone(), module_id);
                }
                ast::Declaration::Behavior(_) => {} // Behaviors are interface definitions, no codegen needed
                ast::Declaration::Trait(_) => {} // Trait definitions are interface definitions, no direct codegen needed
                ast::Declaration::TraitImplementation(_) => {} // Compiled after function declarations
                ast::Declaration::ImplBlock(_) => {} // Compiled after function declarations
                ast::Declaration::TraitRequirement(_) => {
                    // Trait requirements are checked at compile time, no codegen needed
                }
                ast::Declaration::ComptimeBlock(statements) => {
                    // Evaluate comptime blocks and generate constants
                    if let Err(e) = self.comptime_evaluator.execute_comptime_block(statements) {
                        return Err(CompileError::InternalError(
                            format!("Comptime evaluation error: {}", e),
                            None,
                        ));
                    }
                }
                ast::Declaration::TypeAlias(_) => {
                    // Type aliases are resolved at compile time, no codegen needed
                }
                ast::Declaration::Constant { name, value, .. } => {
                    // Evaluate the constant value and store it in the comptime environment
                    // This allows it to be used in subsequent code
                    if let Ok(comptime_value) =
                        self.comptime_evaluator.evaluate_expression(value, None)
                    {
                        self.comptime_evaluator
                            .set_variable(name.clone(), comptime_value);
                    }
                    // Constants are compile-time values, no runtime codegen needed
                }
            }
        }

        // Process top-level statements BEFORE function compilation
        // This ensures imported modules are available inside functions
        if !program.statements.is_empty() {
            // Create a temporary main block to process top-level statements
            let main_fn = if let Some(main) = self.module.get_function("main") {
                main
            } else {
                // Create a temporary function to process top-level statements
                let fn_type = self.context.i32_type().fn_type(&[], false);
                self.module.add_function("__temp_toplevel", fn_type, None)
            };

            let entry = self.context.append_basic_block(main_fn, "toplevel");
            let saved_block = self.builder.get_insert_block();
            self.builder.position_at_end(entry);

            for statement in &program.statements {
                self.compile_statement(statement)?;
            }

            // Restore the builder position
            if let Some(saved) = saved_block {
                self.builder.position_at_end(saved);
            }

            // Mark the temporary function as internal linkage so LLVM can eliminate it
            // instead of using unsafe delete which risks UB if the function is still referenced
            if main_fn.get_name().to_str() == Ok("__temp_toplevel") {
                main_fn.set_linkage(inkwell::module::Linkage::Private);
            }
        }

        // First pass: Declare all functions (skip generic functions - they're instantiated when called)
        for declaration in &program.declarations {
            if let ast::Declaration::Function(func) = declaration {
                if func.type_params.is_empty() {
                    self.declare_function(func)?;
                }
            }
        }

        // Compile trait implementations and impl blocks AFTER function declarations
        // This ensures stdlib functions (io.println, etc.) are available in trait impl bodies
        for declaration in &program.declarations {
            match declaration {
                ast::Declaration::TraitImplementation(trait_impl) => {
                    self.compile_trait_implementation(trait_impl)?;
                }
                ast::Declaration::ImplBlock(impl_block) => {
                    self.compile_impl_block(impl_block)?;
                }
                _ => {}
            }
        }

        // Second pass: Define and compile all functions (skip generic functions)
        for declaration in &program.declarations {
            if let ast::Declaration::Function(func) = declaration {
                if func.type_params.is_empty() {
                    self.compile_function_body(func)?;
                }
            }
        }

        Ok(())
    }

    pub fn cast_value_to_type(
        &self,
        value: BasicValueEnum<'ctx>,
        target_type: BasicTypeEnum<'ctx>,
    ) -> Result<BasicValueEnum<'ctx>, CompileError> {
        // If the types already match, no cast is needed
        if value.get_type() == target_type {
            return Ok(value);
        }

        // Handle casting between integer types
        if let (BasicValueEnum::IntValue(int_val), BasicTypeEnum::IntType(target_int_type)) =
            (value, target_type)
        {
            let source_width = int_val.get_type().get_bit_width();
            let target_width = target_int_type.get_bit_width();

            if source_width < target_width {
                // Sign extend or zero extend
                Ok(self
                    .builder
                    .build_int_s_extend(int_val, target_int_type, "cast")?
                    .into())
            } else if source_width > target_width {
                // Truncate
                Ok(self
                    .builder
                    .build_int_truncate(int_val, target_int_type, "cast")?
                    .into())
            } else {
                // Same width, just return as is
                Ok(int_val.into())
            }
        } else if let (
            BasicValueEnum::FloatValue(float_val),
            BasicTypeEnum::FloatType(target_float_type),
        ) = (value, target_type)
        {
            // Handle float casting
            let source_width = if float_val.get_type() == self.context.f32_type() {
                32
            } else {
                64
            };
            let target_width = if target_float_type == self.context.f32_type() {
                32
            } else {
                64
            };

            if source_width < target_width {
                Ok(self
                    .builder
                    .build_float_ext(float_val, target_float_type, "cast")?
                    .into())
            } else if source_width > target_width {
                Ok(self
                    .builder
                    .build_float_trunc(float_val, target_float_type, "cast")?
                    .into())
            } else {
                Ok(float_val.into())
            }
        } else {
            // For other types, return as is for now
            Ok(value)
        }
    }
}
