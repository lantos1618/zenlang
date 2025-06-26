use super::*;

impl<'ctx> Compiler<'ctx> {
    pub fn to_llvm_type(&mut self, type_: &AstType) -> Result<Type<'ctx>, CompileError> {
        match type_ {
            AstType::I8 => Ok(Type::Basic(self.context.i8_type().into())),
            AstType::I16 => Ok(Type::Basic(self.context.i16_type().into())),
            AstType::I32 => Ok(Type::Basic(self.context.i32_type().into())),
            AstType::I64 => Ok(Type::Basic(self.context.i64_type().into())),
            AstType::Float => Ok(Type::Basic(self.context.f64_type().into())),
            AstType::String => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
            AstType::Pointer(inner) => {
                if let AstType::Void = **inner {
                    return Err(CompileError::UnsupportedFeature(
                        "pointer to void is not supported".to_string(),
                        None,
                    ));
                }
                let inner_type = self.to_llvm_type(inner)?;
                match inner_type {
                    Type::Basic(_b) => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
                    Type::Function(_) | Type::Void => Ok(Type::Basic(self.context.ptr_type(AddressSpace::default()).into())),
                    Type::Pointer(_) => Err(CompileError::UnsupportedFeature(
                        "Pointer to pointer is not supported".to_string(),
                        None,
                    )),
                    Type::Struct(_) => Err(CompileError::UnsupportedFeature(
                        "Pointer to struct is not supported".to_string(),
                        None,
                    )),
                }
            }
            AstType::Struct { name, fields } => {
                // Check if we already have this struct type
                if let Some(info) = self.struct_types.get(name) {
                    return Ok(Type::Basic(info.llvm_type.into()));
                }
                // Check if the struct type exists in the module but not in our map
                if let Some(struct_type) = self.module.get_struct_type(name) {
                    // Rebuild the field mapping
                    let field_mapping: HashMap<String, (usize, AstType)> = fields
                        .iter()
                        .enumerate()
                        .map(|(i, (name, ty))| (name.clone(), (i, ty.clone())))
                        .collect();
                    let info = StructTypeInfo {
                        llvm_type: struct_type,
                        fields: field_mapping,
                    };
                    let name_clone = name.clone();
                    self.struct_types.insert(name_clone, info);
                    return Ok(Type::Basic(struct_type.into()));
                }
                // Create a new struct type
                let field_types: Result<Vec<BasicTypeEnum>, CompileError> = fields
                    .iter()
                    .map(|(_, ty)| {
                        self.to_llvm_type(ty)
                            .and_then(|lyn_type| self.expect_basic_type(lyn_type))
                    })
                    .collect();
                let field_types = field_types?;
                let field_mapping: HashMap<String, (usize, AstType)> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, (name, ty))| (name.clone(), (i, ty.clone())))
                    .collect();
                let struct_type = self.context.opaque_struct_type(name);
                struct_type.set_body(&field_types, false);
                let info = StructTypeInfo {
                    llvm_type: struct_type,
                    fields: field_mapping,
                };
                let name_clone = name.clone();
                self.struct_types.insert(name_clone, info);
                Ok(Type::Basic(struct_type.into()))
            }
            AstType::Function { args, return_type } => {
                let param_types: Result<Vec<BasicTypeEnum>, CompileError> = args
                    .iter()
                    .map(|ty| {
                        let llvm_ty = self.to_llvm_type(ty)?;
                        match llvm_ty {
                            Type::Basic(b) => Ok(b),
                            _ => Err(CompileError::InternalError("Function argument type must be a basic type".to_string(), None)),
                        }
                    })
                    .collect();
                let param_types = param_types?;
                let param_metadata: Vec<BasicMetadataTypeEnum> = param_types.iter().map(|ty| (*ty).into()).collect();
                let ret_type = self.to_llvm_type(return_type)?;
                match ret_type {
                    Type::Basic(b) => Ok(Type::Function(b.fn_type(&param_metadata, false))),
                    Type::Void => Ok(Type::Function(self.context.void_type().fn_type(&param_metadata, false))),
                    _ => Err(CompileError::InternalError("Function return type must be a basic type or void".to_string(), None)),
                }
            }
            AstType::Void => Ok(Type::Void),
        }
    }
    pub fn expect_basic_type<'a>(&self, t: Type<'a>) -> Result<BasicTypeEnum<'a>, CompileError> {
        match t {
            Type::Basic(b) => Ok(b),
            _ => Err(CompileError::UnsupportedFeature(
                "Expected basic type, got non-basic type (e.g., function type)".to_string(),
                None,
            )),
        }
    }
} 