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
    pub fn to_llvm_type(&mut self, type_: &AstType) -> Result<Type<'ctx>, CompileError> {
        // Debug empty Generic types
        if let AstType::Generic { name, .. } = type_ {
            if name.is_empty() {
                // eprintln!("DEBUG: Empty generic type encountered: {:?}", type_);
            }
        }

        let result = match type_ {
            AstType::I8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::I16 => Ok(Type::Basic(self.context.i16_type().into())),
            AstType::I32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::I64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::U8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::U16 => Ok(Type::Basic(self.context.i16_type().into())),
            AstType::U32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::U64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::Usize => Ok(Type::Basic(self.context.i64_type().into())), // usize as i64 on 64-bit systems
            AstType::F32 => Ok(Type::Basic(self.context.f32_type().into())),
            AstType::F64 => Ok(Type::Basic(self.context.f64_type().into())),
            AstType::Bool => Ok(Type::Basic(self.context.bool_type().into())),
            AstType::StaticLiteral | AstType::StaticString => Ok(Type::Basic(
                self.context.ptr_type(AddressSpace::default()).into(),
            )),
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
            AstType::Array(inner) => {
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(basic_type) => Ok(Type::Basic(basic_type)), // Dynamic array (pointer)
                    _ => Ok(Type::Basic(self.context.i8_type().array_type(0).into())), // Default to array of bytes
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
                    let enum_struct_type = self.context.struct_type(
                        &[
                            self.context.i64_type().into(), // discriminant/tag only
                        ],
                        false,
                    );
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
                let _start_type = self.to_llvm_type(start_type)?;
                let _end_type = self.to_llvm_type(end_type)?;
                // For now, just use i64 for both start and end, and bool for inclusive
                let range_struct = self.context.struct_type(
                    &[
                        self.context.i64_type().into(),
                        self.context.i64_type().into(),
                        self.context.bool_type().into(), // Add inclusive field
                    ],
                    false,
                );
                Ok(Type::Struct(range_struct))
            }
            AstType::Vec { element_type, size } => {
                // Vec<T, size> - Fixed-size vector as struct containing array and length
                let elem_llvm_type = self.to_llvm_type(element_type)?;
                match elem_llvm_type {
                    Type::Basic(basic_type) => {
                        // Create struct: { [T; size], usize }
                        let array_type = basic_type.array_type(*size as u32);
                        let len_type = self.context.i64_type(); // Use i64 for length
                        let vec_struct = self.context.struct_type(
                            &[
                                array_type.into(), // data: [T; size]
                                len_type.into(),   // len: usize (current length)
                            ],
                            false,
                        );
                        Ok(Type::Struct(vec_struct))
                    }
                    Type::Struct(struct_type) => {
                        // Handle struct element types properly
                        let array_type = struct_type.array_type(*size as u32);
                        let len_type = self.context.i64_type(); // Use i64 for length
                        let vec_struct = self.context.struct_type(
                            &[
                                array_type.into(), // data: [T; size] where T is a struct
                                len_type.into(),   // len: usize (current length)
                            ],
                            false,
                        );
                        Ok(Type::Struct(vec_struct))
                    }
                    _ => {
                        // Fallback for other types (should not normally reach here)
                        let array_type = self.context.i8_type().array_type(*size as u32);
                        let len_type = self.context.i64_type();
                        let vec_struct = self
                            .context
                            .struct_type(&[array_type.into(), len_type.into()], false);
                        Ok(Type::Struct(vec_struct))
                    }
                }
            }
            AstType::DynVec {
                element_types,
                allocator_type: _,
            } => {
                // DynVec<T> - Dynamic vector as struct containing pointer, length, and capacity
                // For mixed variant types, use a union or tagged union approach
                if element_types.len() == 1 {
                    // Single type DynVec: { ptr, len, capacity }
                    let ptr_type = self.context.ptr_type(AddressSpace::default());
                    let len_type = self.context.i64_type();
                    let cap_type = self.context.i64_type();
                    let dynvec_struct = self.context.struct_type(
                        &[
                            ptr_type.into(), // data: Ptr<T>
                            len_type.into(), // len: usize
                            cap_type.into(), // capacity: usize
                        ],
                        false,
                    );
                    Ok(Type::Struct(dynvec_struct))
                } else {
                    // Mixed variant DynVec: { ptr, len, capacity, discriminants }
                    let ptr_type = self.context.ptr_type(AddressSpace::default());
                    let len_type = self.context.i64_type();
                    let cap_type = self.context.i64_type();
                    let discriminant_ptr = self.context.ptr_type(AddressSpace::default()); // Pointer to discriminant array
                    let dynvec_struct = self.context.struct_type(
                        &[
                            ptr_type.into(),         // data: Ptr<union>
                            len_type.into(),         // len: usize
                            cap_type.into(),         // capacity: usize
                            discriminant_ptr.into(), // discriminants: Ptr<u8> for variant tracking
                        ],
                        false,
                    );
                    Ok(Type::Struct(dynvec_struct))
                }
            }
            AstType::Generic { name, type_args } => {
                if name.is_empty() {
                    return Ok(Type::Basic(self.context.i32_type().into()));
                }

                if name.len() == 1
                    && name.chars().next().unwrap().is_uppercase()
                    && type_args.is_empty()
                {
                    let placeholder = self.context.struct_type(&[], false);
                    return Ok(Type::Struct(placeholder));
                }

                // PRIORITY 1: Check local struct_types cache first (already generated LLVM types)
                if let Some(struct_info) = self.struct_types.get(name) {
                    return Ok(Type::Struct(struct_info.llvm_type));
                }

                // PRIORITY 2: Check TypeContext.structs for struct field info from parsed stdlib
                // This is the correct approach - read struct layouts from .zen files
                if let Some(struct_info) = self.type_ctx.structs.get(name).cloned() {
                    let llvm_type = self.generate_struct_from_type_context_fields(name, &struct_info.fields)?;
                    return Ok(Type::Struct(llvm_type));
                }

                // PRIORITY 3: Handle well-known types (Option, Result) - these have language semantics
                if self.well_known.is_option(name) || self.well_known.is_result(name) {
                    if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name) {
                        return Ok(Type::Struct(enum_info.llvm_type));
                    }
                }

                // PRIORITY 4: Check if it's an enum type
                if let Some(symbols::Symbol::EnumType(enum_info)) = self.symbols.lookup(name) {
                    return Ok(Type::Struct(enum_info.llvm_type));
                }

                // PRIORITY 5: Use TypeContext with stdlib_types fallback
                if let Some(fields) = self.type_ctx.get_struct_definition_with_fallback(name) {
                    let llvm_type = self.generate_struct_from_type_context_fields(name, &fields)?;
                    return Ok(Type::Struct(llvm_type));
                }

                // Handle unresolved generic type parameters
                if type_args
                    .iter()
                    .any(|t| matches!(t, AstType::Generic { .. }))
                {
                    let placeholder = self.context.struct_type(&[], false);
                    Ok(Type::Struct(placeholder))
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
                } else {
                    // Fallback to a default enum structure if not registered
                    let enum_struct_type = self.context.struct_type(
                        &[
                            self.context.i64_type().into(), // discriminant/tag
                            self.context.i64_type().into(), // payload (simplified)
                        ],
                        false,
                    );
                    Ok(Type::Struct(enum_struct_type))
                }
            }
            AstType::StdModule => {
                // StdModule is a marker type for imported stdlib modules (like math, io)
                // It's represented as an i64 in LLVM (storing module identifier)
                Ok(Type::Basic(self.context.i64_type().into()))
            }
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
        let mut result = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in type_str.chars() {
            match ch {
                '<' => {
                    depth += 1;
                    current.push(ch);
                }
                '>' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    // End of current type
                    let parsed = self.parse_type_string(current.trim());
                    result.push(parsed);
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        // Don't forget the last type
        if !current.is_empty() {
            let parsed = self.parse_type_string(current.trim());
            result.push(parsed);
        }

        result
    }

    /// Parse a single type string into an AstType
    pub fn parse_type_string(&self, type_str: &str) -> AstType {
        // Check for generic types
        if let Some(angle_pos) = type_str.find('<') {
            let base_type = &type_str[..angle_pos];
            let type_params_str = &type_str[angle_pos + 1..type_str.len() - 1];

            match base_type {
                "DynVec" => {
                    let element_types = self.parse_comma_separated_types(type_params_str);
                    AstType::DynVec {
                        element_types,
                        allocator_type: None,
                    }
                }
                "Vec" => {
                    // Vec<T, N> where N is the size
                    let parts = self.parse_comma_separated_types(type_params_str);
                    if !parts.is_empty() {
                        // For now, default size to 10 if not specified
                        AstType::Vec {
                            element_type: Box::new(parts[0].clone()),
                            size: 10, // Default size
                        }
                    } else {
                        AstType::Vec {
                            element_type: Box::new(AstType::I32),
                            size: 10,
                        }
                    }
                }
                // Handle Option, Result via well_known
                base if self.well_known.is_option(base) || self.well_known.is_result(base) => {
                    let type_args = self.parse_comma_separated_types(type_params_str);
                    AstType::Generic {
                        name: base.to_string(),
                        type_args,
                    }
                }
                // All other generic types (including collections with intrinsic layouts)
                _ => {
                    let type_args = self.parse_comma_separated_types(type_params_str);
                    AstType::Generic {
                        name: base_type.to_string(),
                        type_args,
                    }
                }
            }
        } else {
            // Simple types
            match type_str {
                "i8" => AstType::I8,
                "i16" => AstType::I16,
                "i32" => AstType::I32,
                "i64" => AstType::I64,
                "u8" => AstType::U8,
                "u16" => AstType::U16,
                "u32" => AstType::U32,
                "u64" => AstType::U64,
                "f32" => AstType::F32,
                "f64" => AstType::F64,
                "bool" => AstType::Bool,
                "string" => AstType::StaticLiteral,
                "StaticString" => AstType::StaticString,
                "void" => AstType::Void,
                // Types with intrinsic layouts become Generic types
                // (codegen will look up layout via TypeContext)
                name if self.type_ctx.get_intrinsic_layout(name).is_some() => {
                    AstType::Generic {
                        name: name.to_string(),
                        type_args: vec![],
                    }
                }
                _ => AstType::I32, // Default fallback
            }
        }
    }

    pub fn register_struct_type(
        &mut self,
        struct_def: &ast::StructDefinition,
    ) -> Result<(), CompileError> {
        let mut field_types = Vec::new();
        let mut fields = HashMap::new();

        for (index, field) in struct_def.fields.iter().enumerate() {
            let llvm_type = match &field.type_ {
                AstType::I8 => self.context.i8_type().as_basic_type_enum(),
                AstType::I16 => self.context.i16_type().as_basic_type_enum(),
                AstType::I32 => self.context.i32_type().as_basic_type_enum(),
                AstType::I64 => self.context.i64_type().as_basic_type_enum(),
                AstType::U8 => self.context.i8_type().as_basic_type_enum(),
                AstType::U16 => self.context.i16_type().as_basic_type_enum(),
                AstType::U32 => self.context.i32_type().as_basic_type_enum(),
                AstType::U64 => self.context.i64_type().as_basic_type_enum(),
                AstType::Usize => self.context.i64_type().as_basic_type_enum(),
                AstType::F32 => self.context.f32_type().as_basic_type_enum(),
                AstType::F64 => self.context.f64_type().as_basic_type_enum(),
                AstType::Bool => self.context.bool_type().as_basic_type_enum(),
                AstType::StaticLiteral | AstType::StaticString => self
                    .context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum(),
                AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => self
                    .context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum(),
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
                        self.context.struct_type(
                            &[
                                self.context.i64_type().into(),
                                self.context.ptr_type(AddressSpace::default()).into(),
                            ],
                            false,
                        ).as_basic_type_enum()
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

    /// Try to get a struct type, registering from TypeContext (with stdlib fallback) if not found locally
    pub fn ensure_struct_type(&mut self, name: &str) -> Result<bool, CompileError> {
        // Already registered locally
        if self.struct_types.contains_key(name) {
            return Ok(true);
        }

        // Try to get from TypeContext (which has stdlib_types fallback built in)
        if let Some(fields) = self.type_ctx.get_struct_definition_with_fallback(name) {
            self.register_struct_from_fields(name, &fields)?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Register a struct type from field definitions (name, type pairs)
    /// This is used when getting struct info from TypeContext
    fn register_struct_from_fields(
        &mut self,
        name: &str,
        fields: &[(String, AstType)],
    ) -> Result<(), CompileError> {
        let mut field_types = Vec::new();
        let mut field_map = HashMap::new();

        for (index, (field_name, field_type)) in fields.iter().enumerate() {
            let llvm_type = self.ast_type_to_llvm_basic_type(field_type)?;
            field_types.push(llvm_type);
            field_map.insert(field_name.clone(), (index, field_type.clone()));
        }

        let struct_type = self.context.struct_type(&field_types, false);

        let struct_info = StructTypeInfo {
            llvm_type: struct_type,
            fields: field_map,
        };

        self.struct_types.insert(name.to_string(), struct_info);

        Ok(())
    }

    /// Convert an AstType to LLVM BasicTypeEnum for struct fields
    fn ast_type_to_llvm_basic_type(&mut self, ast_type: &AstType) -> Result<BasicTypeEnum<'ctx>, CompileError> {
        use inkwell::AddressSpace;

        match ast_type {
            AstType::I8 => Ok(self.context.i8_type().as_basic_type_enum()),
            AstType::I16 => Ok(self.context.i16_type().as_basic_type_enum()),
            AstType::I32 => Ok(self.context.i32_type().as_basic_type_enum()),
            AstType::I64 => Ok(self.context.i64_type().as_basic_type_enum()),
            AstType::U8 => Ok(self.context.i8_type().as_basic_type_enum()),
            AstType::U16 => Ok(self.context.i16_type().as_basic_type_enum()),
            AstType::U32 => Ok(self.context.i32_type().as_basic_type_enum()),
            AstType::U64 => Ok(self.context.i64_type().as_basic_type_enum()),
            AstType::Usize => Ok(self.context.i64_type().as_basic_type_enum()),
            AstType::F32 => Ok(self.context.f32_type().as_basic_type_enum()),
            AstType::F64 => Ok(self.context.f64_type().as_basic_type_enum()),
            AstType::Bool => Ok(self.context.bool_type().as_basic_type_enum()),
            AstType::StaticLiteral | AstType::StaticString => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),
            AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),
            AstType::Void => Err(CompileError::TypeError(
                "Void type not allowed in struct fields".to_string(),
                None,
            )),
            t if t.is_ptr_type() => {
                if t.is_raw_ptr() {
                    Ok(self.context
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum())
                } else {
                    // Ptr<T> and MutPtr<T> are enums: { i64 discriminant, ptr payload }
                    Ok(self.context.struct_type(
                        &[
                            self.context.i64_type().into(),
                            self.context.ptr_type(AddressSpace::default()).into(),
                        ],
                        false,
                    ).as_basic_type_enum())
                }
            }
            AstType::Generic { name, .. } => {
                if let Some(struct_info) = self.struct_types.get(name) {
                    Ok(struct_info.llvm_type.as_basic_type_enum())
                } else {
                    Ok(self.context
                        .ptr_type(AddressSpace::default())
                        .as_basic_type_enum())
                }
            }
            AstType::Struct { name, .. } => {
                if let Some(struct_info) = self.struct_types.get(name) {
                    Ok(struct_info.llvm_type.as_basic_type_enum())
                } else {
                    Err(CompileError::TypeError(
                        format!("Struct '{}' not yet registered", name),
                        None
                    ))
                }
            }
            AstType::FunctionPointer { .. } => Ok(self
                .context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()),
            _ => Err(CompileError::TypeError(
                format!("Unsupported type in struct: {:?}", ast_type),
                None,
            ))
        }
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
                    let payload_size = match payload_type {
                        AstType::I8 | AstType::U8 | AstType::Bool => 8,
                        AstType::I16 | AstType::U16 => 16,
                        AstType::I32 | AstType::U32 | AstType::F32 => 32,
                        AstType::I64 | AstType::U64 | AstType::F64 | AstType::Usize => 64,
                        AstType::StaticLiteral | AstType::StaticString => 64,
                        AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => 64,
                        t if t.is_ptr_type() => 64,
                        AstType::Struct { .. } | AstType::Generic { .. } => 64,
                        AstType::Void => 0,
                        _ => 64,
                    };
                    max_payload_size = max_payload_size.max(payload_size);
                }
            }
        }

        let enum_struct_type = if has_payloads {
            let ptr_type = self.context.ptr_type(AddressSpace::default());

            self.context
                .struct_type(&[self.context.i64_type().into(), ptr_type.into()], false)
        } else {
            self.context
                .struct_type(&[self.context.i64_type().into()], false)
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

    /// Generate LLVM struct type for an intrinsic layout.
    /// This centralizes all collection/stdlib struct layouts in one place.
    pub fn llvm_type_for_layout(
        &self,
        layout: crate::type_context::IntrinsicLayout,
    ) -> inkwell::types::StructType<'ctx> {
        use crate::type_context::IntrinsicLayout;
        let ptr = self.context.ptr_type(AddressSpace::default());

        match layout {
            // Closure: { fn_ptr, captures_ptr } - the only truly intrinsic layout
            IntrinsicLayout::Closure => {
                self.context.struct_type(&[ptr.into(), ptr.into()], false)
            }
        }
    }

    /// Generate LLVM struct type from TypeContext struct fields.
    /// This is the correct approach - read struct layouts from parsed .zen files,
    /// not from hardcoded IntrinsicLayout patterns.
    pub fn generate_struct_from_type_context_fields(
        &mut self,
        name: &str,
        fields: &[(String, AstType)],
    ) -> Result<inkwell::types::StructType<'ctx>, CompileError> {
        let ptr = self.context.ptr_type(AddressSpace::default());
        let i64_type = self.context.i64_type();

        let mut field_types: Vec<inkwell::types::BasicTypeEnum> = Vec::new();
        let mut field_map = HashMap::new();

        for (idx, (field_name, field_type)) in fields.iter().enumerate() {
            let llvm_type = match field_type {
                AstType::I8 | AstType::U8 => self.context.i8_type().into(),
                AstType::I16 | AstType::U16 => self.context.i16_type().into(),
                AstType::I32 | AstType::U32 => self.context.i32_type().into(),
                AstType::I64 | AstType::U64 | AstType::Usize => i64_type.into(),
                AstType::F32 => self.context.f32_type().into(),
                AstType::F64 => self.context.f64_type().into(),
                AstType::Bool => self.context.bool_type().into(),
                // Pointers, strings, and unknown types default to ptr
                _ => ptr.into(),
            };
            field_types.push(llvm_type);
            field_map.insert(field_name.clone(), (idx, field_type.clone()));
        }

        let struct_type = self.context.struct_type(&field_types, false);

        // Cache in struct_types for future lookups
        let struct_info = StructTypeInfo {
            llvm_type: struct_type,
            fields: field_map,
        };
        self.struct_types.insert(name.to_string(), struct_info);

        Ok(struct_type)
    }
}
