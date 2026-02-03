use super::{symbols, LLVMCompiler, StructTypeInfo, Type};
use crate::ast::{self, AstType};
use crate::error::CompileError;
use crate::stdlib_types::StdlibTypeRegistry;
use inkwell::{
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum},
    AddressSpace,
};
use std::collections::HashMap;

impl<'ctx> LLVMCompiler<'ctx> {
    /// Convert primitive AstType to LLVM BasicTypeEnum
    /// Returns None for non-primitive types
    fn primitive_to_llvm_basic(&self, type_: &AstType) -> Option<BasicTypeEnum<'ctx>> {
        Some(match type_ {
            AstType::I8 | AstType::U8 => self.context.i8_type().into(),
            AstType::I16 | AstType::U16 => self.context.i16_type().into(),
            AstType::I32 | AstType::U32 => self.context.i32_type().into(),
            AstType::I64 | AstType::U64 => self.context.i64_type().into(),
            AstType::Usize => self.ptr_sized_int_type().into(),
            AstType::F32 => self.context.f32_type().into(),
            AstType::F64 => self.context.f64_type().into(),
            AstType::Bool => self.context.bool_type().into(),
            AstType::StaticLiteral | AstType::StaticString => {
                self.context.ptr_type(AddressSpace::default()).into()
            }
            _ => return None,
        })
    }

    pub fn to_llvm_type(&mut self, type_: &AstType) -> Result<Type<'ctx>, CompileError> {
        // Handle primitive types first using centralized helper
        if let Some(basic) = self.primitive_to_llvm_basic(type_) {
            return Ok(Type::Basic(basic));
        }

        let result = match type_ {
            // Note: String structs are now handled by the normal AstType::Struct branch below
            // since String is registered in struct_types via register_builtin_enums()
            AstType::Void => Ok(Type::Void),
            // Handle all pointer types (Ptr, MutPtr, RawPtr) - they're all the same in LLVM
            t if t.is_ptr_type() => {
                if let Some(inner) = t.ptr_inner() {
                    let inner_type = self.to_llvm_type(inner)?;
                    match inner_type {
                        Type::Basic(_) | Type::Struct(_) | Type::Void => Ok(Type::Basic(
                            self.context.ptr_type(AddressSpace::default()).into(),
                        )),
                        _ => Err(CompileError::UnsupportedFeature(
                            "Unsupported pointer type".to_string(),
                            None,
                        )),
                    }
                } else {
                    Err(CompileError::UnsupportedFeature(
                        "Invalid pointer type".to_string(),
                        None,
                    ))
                }
            }
            AstType::Struct { name, fields: _ } => {
                // Try to ensure the struct type is registered (might come from stdlib)
                self.ensure_struct_type(name)?;
                let struct_info = self.struct_types.get(name).ok_or_else(|| {
                    CompileError::TypeError(
                        format!("Undefined struct type: {}", name),
                        self.get_current_span(),
                    )
                })?;
                Ok(Type::Struct(struct_info.llvm_type))
            }
            AstType::Slice(inner) => {
                // Slice [T] is a fat pointer: (ptr to data, length)
                // For now, represent as pointer to element type
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(_) => Ok(Type::Basic(
                        self.context.ptr_type(AddressSpace::default()).into(),
                    )),
                    _ => Ok(Type::Basic(
                        self.context.ptr_type(AddressSpace::default()).into(),
                    )),
                }
            }
            AstType::FixedArray { element_type, size } => {
                let elem_type = self.to_llvm_type(element_type)?;
                match elem_type {
                    Type::Basic(basic_type) => {
                        // Create an LLVM array type with the specified size
                        let array_type = match basic_type {
                            BasicTypeEnum::IntType(int_type) => {
                                int_type.array_type(*size as u32).into()
                            }
                            BasicTypeEnum::FloatType(float_type) => {
                                float_type.array_type(*size as u32).into()
                            }
                            BasicTypeEnum::PointerType(ptr_type) => {
                                ptr_type.array_type(*size as u32).into()
                            }
                            BasicTypeEnum::StructType(struct_type) => {
                                struct_type.array_type(*size as u32).into()
                            }
                            BasicTypeEnum::ArrayType(arr_type) => {
                                arr_type.array_type(*size as u32).into()
                            }
                            BasicTypeEnum::VectorType(vec_type) => {
                                vec_type.array_type(*size as u32).into()
                            }
                            BasicTypeEnum::ScalableVectorType(_) => {
                                // For now, use a default array of i8
                                self.context.i8_type().array_type(*size as u32).into()
                            }
                        };
                        Ok(Type::Basic(array_type))
                    }
                    _ => Ok(Type::Basic(
                        self.context.i8_type().array_type(*size as u32).into(),
                    )), // Default to array of bytes
                }
            }
            AstType::Function { args, return_type } => {
                let return_llvm_type = self.to_llvm_type(return_type)?;
                let arg_llvm_types: Result<Vec<BasicTypeEnum<'ctx>>, CompileError> = args
                    .iter()
                    .map(|arg| {
                        let arg_type = self.to_llvm_type(arg)?;
                        match arg_type {
                            Type::Basic(basic_type) => Ok(basic_type),
                            _ => Ok(self.context.i64_type().into()), // Default to i64 for complex types
                        }
                    })
                    .collect();
                let arg_llvm_types = arg_llvm_types?;

                // Convert BasicTypeEnum to BasicMetadataTypeEnum for function signatures
                let arg_metadata_types: Vec<BasicMetadataTypeEnum<'ctx>> =
                    arg_llvm_types.iter().map(|ty| (*ty).into()).collect();

                let function_type = match return_llvm_type {
                    Type::Basic(basic_type) => basic_type.fn_type(&arg_metadata_types, false),
                    _ => self.context.i64_type().fn_type(&arg_metadata_types, false),
                };
                Ok(Type::Function(function_type))
            }
            AstType::FunctionPointer {
                param_types,
                return_type,
            } => {
                // Function pointers are represented as pointers to functions
                let return_llvm_type = self.to_llvm_type(return_type)?;
                let arg_llvm_types: Result<Vec<BasicTypeEnum<'ctx>>, CompileError> = param_types
                    .iter()
                    .map(|arg| {
                        let arg_type = self.to_llvm_type(arg)?;
                        match arg_type {
                            Type::Basic(basic_type) => Ok(basic_type),
                            _ => Ok(self.context.i64_type().into()), // Default to i64 for complex types
                        }
                    })
                    .collect();
                let arg_llvm_types = arg_llvm_types?;

                // Convert BasicTypeEnum to BasicMetadataTypeEnum for function signatures
                let arg_metadata_types: Vec<BasicMetadataTypeEnum<'ctx>> =
                    arg_llvm_types.iter().map(|ty| (*ty).into()).collect();

                let _function_type = match return_llvm_type {
                    Type::Basic(basic_type) => basic_type.fn_type(&arg_metadata_types, false),
                    Type::Void => self.context.void_type().fn_type(&arg_metadata_types, false),
                    _ => self.context.i64_type().fn_type(&arg_metadata_types, false),
                };

                // Return a pointer to the function type
                Ok(Type::Basic(
                    self.context.ptr_type(AddressSpace::default()).into(),
                ))
            }
            AstType::Enum { name, variants: _ } => {
                // Look up the registered enum type
                if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name) {
                    Ok(Type::Struct(enum_info.llvm_type))
                } else {
                    // Fallback to a simple tag-only enum if not registered
                    // This should rarely happen as enums should be registered during declaration phase
                    // Use i8 discriminant as minimum (1 variant minimum)
                    let enum_struct_type = self
                        .context
                        .struct_type(&[self.enum_discriminant_type(1).into()], false);
                    Ok(Type::Struct(enum_struct_type))
                }
            }
            AstType::Ref(inner) => {
                // Ref<T> is represented as a pointer to T
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(basic_type) => Ok(Type::Basic(basic_type)),
                    _ => Ok(Type::Basic(
                        self.context.ptr_type(AddressSpace::default()).into(),
                    )),
                }
            }
            // Option and Result are now Generic types - they're handled in the Generic match above
            AstType::Range {
                start_type,
                end_type,
                inclusive: _,
            } => {
                // Range is represented as a struct with start, end, and inclusive values
                // Use actual types instead of hardcoded i64 for proper memory layout
                let start_llvm = self.to_llvm_type(start_type)?;
                let end_llvm = self.to_llvm_type(end_type)?;

                // Extract BasicTypeEnum from Type, defaulting to ptr_sized_int for unknown types
                let start_basic = match start_llvm {
                    Type::Basic(b) => b,
                    _ => self.ptr_sized_int_type().into(),
                };
                let end_basic = match end_llvm {
                    Type::Basic(b) => b,
                    _ => self.ptr_sized_int_type().into(),
                };

                let range_struct = self.context.struct_type(
                    &[
                        start_basic,
                        end_basic,
                        self.context.bool_type().into(), // inclusive field
                    ],
                    false,
                );
                Ok(Type::Struct(range_struct))
            }
            // Vec<T>, DynVec<T>, HashMap<K,V> etc. are stdlib Generic types
            // They're handled by the Generic branch below, resolved from stdlib struct definitions
            AstType::Generic { name, type_args } => {
                if name.is_empty() {
                    return Ok(Type::Basic(self.context.i32_type().into()));
                }

                if name.len() == 1
                    && name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    && type_args.is_empty()
                {
                    let placeholder = self.context.struct_type(&[], false);
                    return Ok(Type::Struct(placeholder));
                }

                // Well-known types: Option and Result use tagged union representation
                if self.well_known.is_result(name) || self.well_known.is_option(name) {
                    return Ok(Type::Struct(self.well_known_enum_type()));
                }

                // Try to get struct definition from StdlibTypeRegistry
                let registry = crate::stdlib_types::stdlib_types();
                if let Some(struct_type) = registry.get_struct_type(name) {
                    return self.to_llvm_type(&struct_type);
                }

                // Check registered enum types in symbol table
                if self.well_known.is_option(name) || self.well_known.is_result(name) {
                    // Look up the registered enum type
                    if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name) {
                        return Ok(Type::Struct(enum_info.llvm_type));
                    } else {
                        // Fallback to well-known enum struct if not registered (shouldn't happen)
                        return Ok(Type::Struct(self.well_known_enum_type()));
                    }
                }

                // Check if this is actually a user-defined struct type
                if let Some(struct_info) = self.struct_types.get(name) {
                    Ok(Type::Struct(struct_info.llvm_type))
                } else if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name)
                {
                    // Check if it's an enum type that was parsed as Generic
                    Ok(Type::Struct(enum_info.llvm_type))
                } else if (name == "Vec" || name == "DynVec") && !type_args.is_empty() {
                    // Vec<T> and DynVec<T> are stdlib structs: { ptr, len, capacity, allocator }
                    let ptr_type = self.context.ptr_type(AddressSpace::default());
                    let len_type = self.ptr_sized_int_type(); // Use platform-sized int for len/capacity
                    let vec_struct = self.context.struct_type(
                        &[
                            ptr_type.into(), // data: Ptr<T>
                            len_type.into(), // len: usize (platform-sized)
                            len_type.into(), // capacity: usize (platform-sized)
                            ptr_type.into(), // allocator: Allocator (pointer)
                        ],
                        false,
                    );
                    Ok(Type::Struct(vec_struct))
                } else if type_args
                    .iter()
                    .any(|t| matches!(t, AstType::Generic { .. }))
                {
                    let placeholder = self.context.struct_type(&[], false);
                    Ok(Type::Struct(placeholder))
                } else if let Some(aliased_type) = self.type_ctx.type_aliases.get(name).cloned() {
                    // Resolve type alias (e.g., CompletionFn -> FunctionPointer)
                    self.to_llvm_type(&aliased_type)
                } else {
                    Err(CompileError::InternalError(
                        format!("Unresolved generic type '{}' found after monomorphization. This is a compiler bug.", name),
                        self.get_current_span()
                    ))
                }
            }
            AstType::EnumType { name } => {
                // EnumType is used when an enum is referenced as a type constructor
                // Look up the registered enum type
                if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name) {
                    Ok(Type::Struct(enum_info.llvm_type))
                } else if self.well_known.is_option(name) || self.well_known.is_result(name) {
                    // Well-known enums use the centralized type
                    Ok(Type::Struct(self.well_known_enum_type()))
                } else {
                    // Fallback: 2-variant enum with pointer payload (common case)
                    let discriminant = self.enum_discriminant_type(2);
                    let ptr_type = self.context.ptr_type(AddressSpace::default());
                    let enum_struct_type = self
                        .context
                        .struct_type(&[discriminant.into(), ptr_type.into()], false);
                    Ok(Type::Struct(enum_struct_type))
                }
            }
            AstType::StdModule => {
                // StdModule is a marker type for imported stdlib modules (like math, io)
                // It's represented as an i64 in LLVM (storing module identifier)
                Ok(Type::Basic(self.context.i64_type().into()))
            }
            // Catch-all: primitives are handled by primitive_to_llvm_basic() above
            // This should never be reached for valid code
            _ => Err(CompileError::UnsupportedFeature(
                format!("Unhandled type in LLVM conversion: {:?}", type_),
                self.get_current_span(),
            )),
        };
        result
    }
    pub fn expect_basic_type<'a>(&self, t: Type<'a>) -> Result<BasicTypeEnum<'a>, CompileError> {
        match t {
            Type::Basic(ty) => Ok(ty),
            Type::Struct(struct_type) => Ok(struct_type.as_basic_type_enum()),
            _ => Err(CompileError::UnsupportedFeature(
                "Expected basic type, got non-basic type (e.g., function type)".to_string(),
                self.get_current_span(),
            )),
        }
    }

    /// Parse comma-separated types from a string, handling nested generics
    pub fn parse_comma_separated_types(&self, type_str: &str) -> Vec<AstType> {
        crate::parser::parse_type_args_from_string(type_str).unwrap_or_default()
    }

    /// Parse a single type string into an AstType
    pub fn parse_type_string(&self, type_str: &str) -> AstType {
        crate::parser::parse_type_from_string(type_str).unwrap_or(AstType::I32)
    }

    pub fn register_struct_type(
        &mut self,
        struct_def: &ast::StructDefinition,
    ) -> Result<(), CompileError> {
        let mut field_types = Vec::new();
        let mut fields = HashMap::new();

        for (index, field) in struct_def.fields.iter().enumerate() {
            // Use centralized primitive-to-LLVM conversion
            let llvm_type = if let Some(basic) = self.primitive_to_llvm_basic(&field.type_) {
                basic
            } else {
                match &field.type_ {
                    AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => {
                        self.context
                            .ptr_type(AddressSpace::default())
                            .as_basic_type_enum()
                    }
                    AstType::Void => {
                        return Err(CompileError::TypeError(
                            "Void type not allowed in struct fields".to_string(),
                            None,
                        ))
                    }
                    t if t.is_ptr_type() => {
                        // Ptr<T> and MutPtr<T> are enums with { i64 discriminant, ptr payload } (16 bytes)
                        // RawPtr<T> is just a plain pointer (8 bytes)
                        if t.is_raw_ptr() {
                            self.context
                                .ptr_type(AddressSpace::default())
                                .as_basic_type_enum()
                        } else {
                            // Ptr<T> and MutPtr<T> are enums: { i64 discriminant, ptr payload }
                            self.context
                                .struct_type(
                                    &[
                                        self.context.i64_type().into(),
                                        self.context.ptr_type(AddressSpace::default()).into(),
                                    ],
                                    false,
                                )
                                .as_basic_type_enum()
                        }
                    }
                    AstType::Generic { name, .. } => {
                        if let Some(struct_info) = self.struct_types.get(name) {
                            struct_info.llvm_type.as_basic_type_enum()
                        } else {
                            self.context
                                .ptr_type(AddressSpace::default())
                                .as_basic_type_enum()
                        }
                    }
                    AstType::Struct { name, .. } => {
                        if let Some(struct_info) = self.struct_types.get(name) {
                            struct_info.llvm_type.as_basic_type_enum()
                        } else {
                            return Err(CompileError::TypeError(
                            format!("Struct '{}' not yet registered. This may be a forward reference issue. Structs should be defined before use, or the typechecker should resolve Generic types to Struct types.", name),
                            None
                        ));
                        }
                    }
                    AstType::FunctionPointer { .. } => self
                        .context
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum(),
                    _ => {
                        return Err(CompileError::TypeError(
                            format!("Unsupported type in struct: {:?}", field.type_),
                            None,
                        ))
                    }
                }
            };

            field_types.push(llvm_type);
            fields.insert(field.name.clone(), (index, field.type_.clone()));
        }

        let struct_type = self.context.struct_type(&field_types, false);

        let struct_info = StructTypeInfo {
            llvm_type: struct_type,
            fields,
        };

        self.struct_types
            .insert(struct_def.name.clone(), struct_info);

        Ok(())
    }

    /// Try to get a struct type, registering from stdlib if not found locally
    pub fn ensure_struct_type(&mut self, name: &str) -> Result<bool, CompileError> {
        // Already registered locally
        if self.struct_types.contains_key(name) {
            return Ok(true);
        }

        // Try to get from stdlib
        let registry = crate::stdlib_types::stdlib_types();
        if let Some(struct_def) = registry.get_struct_definition(name) {
            // Clone the struct_def to avoid borrow issues
            let struct_def = struct_def.clone();
            self.register_struct_type(&struct_def)?;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn register_enum_type(
        &mut self,
        enum_def: &ast::EnumDefinition,
    ) -> Result<(), CompileError> {
        let mut variant_indices = HashMap::new();
        let mut max_payload_size = 0u32;
        let mut has_payloads = false;

        for (index, variant) in enum_def.variants.iter().enumerate() {
            variant_indices.insert(variant.name.clone(), index as u64);

            if let Some(payload_type) = &variant.payload {
                if !matches!(payload_type, AstType::Void) {
                    has_payloads = true;
                    // Use centralized bit_size for numeric types, otherwise default to 64 bits
                    let payload_size =
                        crate::ast::bit_size(payload_type).unwrap_or(match payload_type {
                            AstType::Bool => 8, // Bool stored as 1 byte in LLVM
                            AstType::Void => 0,
                            _ => 64, // Pointers, strings, structs, generics are 64-bit
                        });
                    max_payload_size = max_payload_size.max(payload_size);
                }
            }
        }

        // Size discriminant based on variant count to save memory
        let discriminant_type = self.enum_discriminant_type(enum_def.variants.len());

        let enum_struct_type = if has_payloads {
            let ptr_type = self.context.ptr_type(AddressSpace::default());
            self.context
                .struct_type(&[discriminant_type.into(), ptr_type.into()], false)
        } else {
            self.context.struct_type(&[discriminant_type.into()], false)
        };

        let enum_info = symbols::EnumInfo {
            llvm_type: enum_struct_type,
            variant_indices,
            variants: enum_def.variants.clone(),
        };

        self.symbols
            .insert(&enum_def.name, symbols::Symbol::EnumType(enum_info));

        Ok(())
    }
}
