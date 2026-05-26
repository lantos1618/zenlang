impl TypeChecker {
    fn insert_source_type_dependency(
        local_name: &str,
        decl: &Declaration,
        dependencies: &mut SourceModuleDependencies,
        specialization_scope: Option<&str>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                public,
                ..
            } => {
                let info = struct_info_from_ast_fields(local_name.to_string(), type_params, fields);
                dependencies.structs.insert(
                    local_name.to_string(),
                    Self::scoped_source_type_info(info, *public, specialization_scope),
                );
            }
            Declaration::Enum {
                type_params,
                variants,
                public,
                ..
            } => {
                let info =
                    enum_info_from_ast_variants(local_name.to_string(), type_params, variants);
                dependencies.enums.insert(
                    local_name.to_string(),
                    Self::scoped_source_type_info(info, *public, specialization_scope),
                );
            }
            _ => {}
        }
    }

    fn scoped_source_type_info<T: SourceTypeSpecializationScope>(
        info: T,
        public: bool,
        specialization_scope: Option<&str>,
    ) -> T {
        if public {
            return info;
        }

        match specialization_scope {
            Some(scope) => info.with_source_scope(scope.to_string()),
            None => info,
        }
    }
}

trait SourceTypeSpecializationScope {
    fn with_source_scope(self, scope: String) -> Self;
}

impl SourceTypeSpecializationScope for StructInfo {
    fn with_source_scope(self, scope: String) -> Self {
        self.with_specialization_scope(scope)
    }
}

impl SourceTypeSpecializationScope for EnumInfo {
    fn with_source_scope(self, scope: String) -> Self {
        self.with_specialization_scope(scope)
    }
}
