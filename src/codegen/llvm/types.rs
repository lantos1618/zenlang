use super::{LLVMCompiler, Type};
use crate::ast::AstType;
use crate::error::CompileError;
use inkwell::{
    types::{BasicType, BasicTypeEnum, BasicMetadataTypeEnum},
    AddressSpace,
};

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn to_llvm_type(&mut self, type_: &AstType) -> Result<Type<'ctx>, CompileError> {
        println!("DEBUG: to_llvm_type called with AstType: {:?}", type_);
        let result = match type_ {
            AstType::I8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::I16 => Ok(Type::Basic(self.context.i16_type().into())),
            AstType::I32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::I64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::U8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::U16 => Ok(Type::Basic(self.context.i16_type().into())),
            AstType::U32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::U64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::F32 => Ok(Type::Basic(self.context.f32_type().into())),
            AstType::F64 => Ok(Type::Basic(self.context.f64_type().into())),
            AstType::Bool => Ok(Type::Basic(self.context.bool_type().into())),
            AstType::String => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
            AstType::Void => Ok(Type::Void),
            AstType::Pointer(inner) => {
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(_) => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
                    Type::Struct(_) => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
                    Type::Void => {
                        // For void pointers, use i8* as the LLVM representation
                        Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into()))
                    },
                    _ => Err(CompileError::UnsupportedFeature("Unsupported pointer type".to_string(), None)),
                }
            },
            AstType::Struct { name, fields } => {
                let struct_info = self.struct_types.get(name)
                    .ok_or_else(|| CompileError::TypeError(format!("Undefined struct type: {}", name), None))?;
                Ok(Type::Struct(struct_info.llvm_type))
            },
            AstType::Array(inner) => {
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(basic_type) => Ok(Type::Basic(basic_type)), // 0 means unknown size
                    _ => Ok(Type::Basic(self.context.i8_type().array_type(0).into())), // Default to array of bytes
                }
            },
            AstType::Function { args, return_type } => {
                let return_llvm_type = self.to_llvm_type(return_type)?;
                let arg_llvm_types: Result<Vec<BasicTypeEnum<'ctx>>, CompileError> = args.iter().map(|arg| {
                    let arg_type = self.to_llvm_type(arg)?;
                    match arg_type {
                        Type::Basic(basic_type) => Ok(basic_type),
                        _ => Ok(self.context.i64_type().into()), // Default to i64 for complex types
                    }
                }).collect();
                let arg_llvm_types = arg_llvm_types?;
                
                // Convert BasicTypeEnum to BasicMetadataTypeEnum for function signatures
                let arg_metadata_types: Vec<BasicMetadataTypeEnum<'ctx>> = arg_llvm_types.iter().map(|ty| (*ty).into()).collect();
                
                let function_type = match return_llvm_type {
                    Type::Basic(basic_type) => basic_type.fn_type(&arg_metadata_types, false),
                    _ => self.context.i64_type().fn_type(&arg_metadata_types, false),
                };
                Ok(Type::Function(function_type))
            },
            AstType::Enum { name, variants: _ } => {
                // Enums are represented as integers for now
                // TODO: Implement proper enum representation
                Ok(Type::Basic(self.context.i64_type().into()))
            },
            AstType::Ref(inner) => {
                // Ref<T> is represented as a pointer to T
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(basic_type) => Ok(Type::Basic(basic_type)),
                    _ => Ok(Type::Basic(self.context.i8_type().ptr_type(AddressSpace::default()).into())),
                }
            },
            AstType::Option(inner) => {
                // Option<T> is represented as a pointer to T (null = None, non-null = Some)
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(basic_type) => Ok(Type::Basic(basic_type)),
                    _ => Ok(Type::Basic(self.context.i8_type().ptr_type(AddressSpace::default()).into())),
                }
            },
            AstType::Result { ok_type, err_type } => {
                // Result<T, E> is represented as a struct with a tag and union
                // For now, just use a pointer to represent it
                let _ok_type = self.to_llvm_type(ok_type)?;
                let _err_type = self.to_llvm_type(err_type)?;
                Ok(Type::Basic(self.context.i8_type().ptr_type(AddressSpace::default()).into()))
            },
            AstType::Range { start_type, end_type, inclusive: _ } => {
                // Range is represented as a struct with start and end values
                let _start_type = self.to_llvm_type(start_type)?;
                let _end_type = self.to_llvm_type(end_type)?;
                // For now, just use i64 for both start and end
                let range_struct = self.context.struct_type(&[
                    self.context.i64_type().into(),
                    self.context.i64_type().into(),
                ], false);
                Ok(Type::Struct(range_struct))
            },
            AstType::Generic { name, type_args } => {
                // For now, just use the first type argument or default to i64
                if let Some(first_arg) = type_args.first() {
                    self.to_llvm_type(first_arg)
                } else {
                    Ok(Type::Basic(self.context.i64_type().into()))
                }
            },
        };
        println!("DEBUG: to_llvm_type returning: {:?}", result);
        result
    }
    pub fn expect_basic_type<'a>(&self, t: Type<'a>) -> Result<BasicTypeEnum<'a>, CompileError> {
        println!("DEBUG: expect_basic_type called with type: {:?}", t);
        match t {
            Type::Basic(ty) => Ok(ty),
            Type::Struct(struct_type) => Ok(struct_type.as_basic_type_enum()),
            _ => Err(CompileError::UnsupportedFeature(
                "Expected basic type, got non-basic type (e.g., function type)".to_string(),
                None,
            )),
        }
    }
} 