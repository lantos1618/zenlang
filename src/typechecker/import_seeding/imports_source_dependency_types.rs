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
                let mut info = struct_info_from_ast_fields(type_params, fields);
                if !*public {
                    info.specialization_scope = specialization_scope.map(str::to_string);
                }
                dependencies.structs.insert(local_name.to_string(), info);
            }
            Declaration::Enum {
                type_params,
                variants,
                public,
                ..
            } => {
                let mut info = enum_info_from_ast_variants(type_params, variants);
                if !*public {
                    info.specialization_scope = specialization_scope.map(str::to_string);
                }
                dependencies.enums.insert(local_name.to_string(), info);
            }
            _ => {}
        }
    }
}
