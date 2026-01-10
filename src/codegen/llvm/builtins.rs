use super::{symbols, LLVMCompiler};
use crate::ast::{self, AstType};
use crate::well_known::well_known;
use std::collections::HashMap;

impl<'ctx> LLVMCompiler<'ctx> {
    pub fn register_builtin_enums(&mut self) {
        // NOTE: Array and String are no longer registered here.
        // Their layouts come from TypeContext.structs (populated from stdlib parsing).

        let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
        let enum_struct_type = self
            .context
            .struct_type(&[self.context.i64_type().into(), ptr_type.into()], false);

        let wk = well_known();
        let mut variant_indices = HashMap::new();
        variant_indices.insert(wk.some_name().to_string(), 0);
        variant_indices.insert(wk.none_name().to_string(), 1);

        let option_info = symbols::EnumInfo {
            llvm_type: enum_struct_type,
            variant_indices: variant_indices.clone(),
            variants: vec![
                ast::EnumVariant {
                    name: wk.some_name().to_string(),
                    payload: Some(AstType::Generic {
                        name: "T".to_string(),
                        type_args: vec![],
                    }),
                },
                ast::EnumVariant {
                    name: wk.none_name().to_string(),
                    payload: None,
                },
            ],
        };
        self.symbols
            .insert(wk.option_name(), symbols::Symbol::EnumType(option_info));

        let result_struct_type = self
            .context
            .struct_type(&[self.context.i64_type().into(), ptr_type.into()], false);

        let mut result_variant_indices = HashMap::new();
        result_variant_indices.insert(wk.ok_name().to_string(), 0);
        result_variant_indices.insert(wk.err_name().to_string(), 1);

        let result_info = symbols::EnumInfo {
            llvm_type: result_struct_type,
            variant_indices: result_variant_indices,
            variants: vec![
                ast::EnumVariant {
                    name: wk.ok_name().to_string(),
                    payload: Some(AstType::Generic {
                        name: "T".to_string(),
                        type_args: vec![],
                    }),
                },
                ast::EnumVariant {
                    name: wk.err_name().to_string(),
                    payload: Some(AstType::Generic {
                        name: "E".to_string(),
                        type_args: vec![],
                    }),
                },
            ],
        };
        self.symbols
            .insert(wk.result_name(), symbols::Symbol::EnumType(result_info));
    }

    pub fn declare_stdlib_functions(&mut self) {
        if self.module.get_function("malloc").is_none() {
            let i64_type = self.context.i64_type();
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let malloc_type = ptr_type.fn_type(&[i64_type.into()], false);
            self.module.add_function("malloc", malloc_type, None);
        }

        if self.module.get_function("free").is_none() {
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let void_type = self.context.void_type();
            let free_type = void_type.fn_type(&[ptr_type.into()], false);
            self.module.add_function("free", free_type, None);
        }

        if self.module.get_function("memcpy").is_none() {
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let i64_type = self.context.i64_type();
            let void_type = self.context.void_type();
            let memcpy_type =
                void_type.fn_type(&[ptr_type.into(), ptr_type.into(), i64_type.into()], false);
            self.module.add_function("memcpy", memcpy_type, None);
        }

        if self.module.get_function("get_default_allocator").is_none() {
            let ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());
            let get_alloc_type = ptr_type.fn_type(&[], false);
            let func = self
                .module
                .add_function("get_default_allocator", get_alloc_type, None);

            let entry = self.context.append_basic_block(func, "entry");
            let current_block = self.builder.get_insert_block();
            self.builder.position_at_end(entry);

            let i64_type = self.context.i64_type();
            let marker_int = i64_type.const_int(1, false);
            let marker_ptr = self
                .builder
                .build_int_to_ptr(marker_int, ptr_type, "allocator_marker")
                .unwrap_or_else(|_| ptr_type.const_null());
            let _ = self.builder.build_return(Some(&marker_ptr));

            if let Some(block) = current_block {
                self.builder.position_at_end(block);
            }

            self.functions
                .insert("get_default_allocator".to_string(), func);
            self.function_types.insert(
                "get_default_allocator".to_string(),
                AstType::ptr(AstType::Void),
            );
        }
    }
}
