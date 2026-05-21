use super::*;

impl<'a> LayoutContext<'a> {
    pub(super) fn seed_builtin_layouts(&mut self) {
        for (name, size, alignment) in [
            ("bool", 1, 1),
            ("i8", 1, 1),
            ("u8", 1, 1),
            ("i16", 2, 2),
            ("u16", 2, 2),
            ("i32", 4, 4),
            ("u32", 4, 4),
            ("f32", 4, 4),
            ("i64", 8, 8),
            ("u64", 8, 8),
            ("usize", USIZE_SIZE, POINTER_ALIGN),
            ("f64", 8, 8),
            ("void", 0, 1),
        ] {
            self.layouts
                .insert(name.into(), scalar_layout("primitive", size, alignment));
        }
        self.layouts.insert(
            "StaticString".into(),
            scalar_layout("static_string", POINTER_SIZE + USIZE_SIZE, POINTER_ALIGN),
        );
        self.layouts.insert(
            "String".into(),
            scalar_layout(
                "dynamic_string",
                POINTER_SIZE + USIZE_SIZE * 2 + POINTER_SIZE,
                POINTER_ALIGN,
            ),
        );
    }

    pub(super) fn layout_builtin_type(&self, ty: &Type) -> Option<LayoutJsonType> {
        let name = match ty {
            Type::I8 => "i8",
            Type::I16 => "i16",
            Type::I32 => "i32",
            Type::I64 => "i64",
            Type::U8 => "u8",
            Type::U16 => "u16",
            Type::U32 => "u32",
            Type::U64 => "u64",
            Type::Usize => "usize",
            Type::F32 => "f32",
            Type::F64 => "f64",
            Type::Bool => "bool",
            Type::Void | Type::Never | Type::Unknown => "void",
            Type::Str => "StaticString",
            Type::String => "String",
            _ => return None,
        };

        Some(self.layout_by_name(name))
    }

    pub(super) fn cache_compound_layout(
        &mut self,
        ty: &Type,
        kind: &'static str,
        size: u32,
        alignment: u32,
    ) -> LayoutJsonType {
        let layout = scalar_layout(kind, size, alignment);
        self.layouts
            .entry(ty.display_name())
            .or_insert_with(|| layout.clone());
        layout
    }

    pub(super) fn layout_by_name(&self, name: &str) -> LayoutJsonType {
        self.layouts
            .get(name)
            .cloned()
            .unwrap_or_else(|| scalar_layout("opaque", 0, 1))
    }
}
