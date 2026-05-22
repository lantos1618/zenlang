use super::*;

impl TypeChecker {
    pub(super) fn typed_struct_def(
        &mut self,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) -> TypedTypeDef {
        let resolved_fields = fields
            .iter()
            .map(|field| (field.name.clone(), self.resolve_type(&field.ty)))
            .collect();

        TypedTypeDef {
            name: name.to_string(),
            kind: TypeDefKind::Struct {
                fields: resolved_fields,
            },
            methods: Vec::new(),
            span,
        }
    }

    pub(super) fn typed_enum_def(
        &mut self,
        name: &str,
        variants: &[EnumVariant],
        span: Span,
    ) -> TypedTypeDef {
        let typed_variants = variants
            .iter()
            .enumerate()
            .map(|(index, variant)| TypedVariant {
                name: variant.name.clone(),
                tag: index as u32,
                payload: variant
                    .payload
                    .as_ref()
                    .map(|ty| vec![("payload".to_string(), self.resolve_type(ty))]),
            })
            .collect();

        TypedTypeDef {
            name: name.to_string(),
            kind: TypeDefKind::Enum {
                variants: typed_variants,
            },
            methods: Vec::new(),
            span,
        }
    }
}
