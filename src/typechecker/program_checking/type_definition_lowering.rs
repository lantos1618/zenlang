use super::*;

impl TypeChecker {
    pub(super) fn push_non_generic_struct_type(
        &mut self,
        types: &mut Vec<TypedTypeDef>,
        name: &str,
        fields: &[StructField],
        span: &Span,
    ) {
        let resolved_fields: Vec<(String, Type)> = fields
            .iter()
            .map(|f| (f.name.clone(), self.resolve_type(&f.ty)))
            .collect();
        types.push(TypedTypeDef {
            name: name.into(),
            kind: TypeDefKind::Struct {
                fields: resolved_fields,
            },
            methods: Vec::new(),
            span: *span,
        });
    }

    pub(super) fn push_non_generic_enum_type(
        &mut self,
        types: &mut Vec<TypedTypeDef>,
        name: &str,
        variants: &[EnumVariant],
        span: &Span,
    ) {
        let typed_variants: Vec<TypedVariant> = variants
            .iter()
            .enumerate()
            .map(|(i, v)| TypedVariant {
                name: v.name.clone(),
                tag: i as u32,
                payload: v
                    .payload
                    .as_ref()
                    .map(|ty| vec![("payload".to_string(), self.resolve_type(ty))]),
            })
            .collect();
        types.push(TypedTypeDef {
            name: name.into(),
            kind: TypeDefKind::Enum {
                variants: typed_variants,
            },
            methods: Vec::new(),
            span: *span,
        });
    }
}
