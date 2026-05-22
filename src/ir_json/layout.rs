use std::collections::BTreeMap;

use crate::ast::typed::{Type, TypeDefKind, TypedProgram, TypedTypeDef};

#[path = "layout/builtins.rs"]
mod builtins;
#[path = "layout/metrics.rs"]
mod metrics;
#[path = "layout/schema.rs"]
mod schema;

use builtins::{builtin_layout_name, seed_builtin_layouts};
use metrics::{align_to, scalar_layout, POINTER_ALIGN, POINTER_SIZE, USIZE_SIZE};
use schema::{
    LayoutJsonField, LayoutJsonProgram, LayoutJsonTarget, LayoutJsonType, LayoutJsonVariant,
};

pub(super) fn program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let context = LayoutContext::new(program);
    let graph = LayoutJsonProgram {
        format: "zen.layout.v0",
        schema_version: 0,
        semantic_status: "checked",
        target: LayoutJsonTarget {
            pointer_size: POINTER_SIZE,
            pointer_alignment: POINTER_ALIGN,
            usize_size: USIZE_SIZE,
        },
        layouts: context.layouts(),
    };

    serde_json::to_string_pretty(&graph)
}

struct LayoutContext<'a> {
    types: BTreeMap<&'a str, &'a TypedTypeDef>,
    layouts: BTreeMap<String, LayoutJsonType>,
}

impl<'a> LayoutContext<'a> {
    fn new(program: &'a TypedProgram) -> Self {
        let mut context = Self {
            types: program
                .types
                .iter()
                .map(|type_def| (type_def.name.as_str(), type_def))
                .collect(),
            layouts: BTreeMap::new(),
        };
        seed_builtin_layouts(&mut context.layouts);
        for type_def in &program.types {
            context.layout_type_def(type_def);
        }
        context
    }

    fn layouts(self) -> BTreeMap<String, LayoutJsonType> {
        self.layouts
    }

    fn layout_type_def(&mut self, type_def: &'a TypedTypeDef) -> LayoutJsonType {
        if let Some(layout) = self.layouts.get(&type_def.name) {
            return layout.clone();
        }

        let layout = match &type_def.kind {
            TypeDefKind::Struct { fields } => self.layout_struct(fields),
            TypeDefKind::Enum { variants } => {
                let mut max_payload_size = 0;
                let mut max_payload_align = 1;
                let mut json_variants = Vec::new();
                for variant in variants {
                    let payload_fields = variant
                        .payload
                        .as_deref()
                        .map(|fields| {
                            let (layout_fields, payload_size, payload_align) =
                                self.layout_fields(fields);
                            max_payload_size = max_payload_size.max(payload_size);
                            max_payload_align = max_payload_align.max(payload_align);
                            layout_fields
                        })
                        .unwrap_or_default();
                    json_variants.push(LayoutJsonVariant {
                        name: variant.name.clone(),
                        tag: variant.tag,
                        payload_fields,
                    });
                }
                let payload_offset = align_to(4, max_payload_align);
                let alignment = 4.max(max_payload_align);
                LayoutJsonType {
                    kind: "enum",
                    size: align_to(payload_offset + max_payload_size, alignment),
                    alignment,
                    fields: Vec::new(),
                    variants: json_variants,
                }
            }
        };
        self.layouts.insert(type_def.name.clone(), layout.clone());
        layout
    }

    fn layout_struct(&mut self, fields: &[(String, Type)]) -> LayoutJsonType {
        let (fields, size, alignment) = self.layout_fields(fields);
        LayoutJsonType {
            kind: "struct",
            size,
            alignment,
            fields,
            variants: Vec::new(),
        }
    }

    fn layout_fields(&mut self, fields: &[(String, Type)]) -> (Vec<LayoutJsonField>, u32, u32) {
        let mut offset = 0;
        let mut alignment = 1;
        let mut json_fields = Vec::new();
        for (name, ty) in fields {
            let field_layout = self.layout_type(ty);
            offset = align_to(offset, field_layout.alignment);
            json_fields.push(LayoutJsonField {
                name: name.clone(),
                r#type: ty.display_name(),
                offset,
            });
            offset += field_layout.size;
            alignment = alignment.max(field_layout.alignment);
        }
        (json_fields, align_to(offset, alignment), alignment)
    }

    fn layout_type(&mut self, ty: &Type) -> LayoutJsonType {
        if let Some(name) = builtin_layout_name(ty) {
            return self.layout_by_name(name);
        }

        match ty {
            Type::Named(name) => self.layout_named(name),
            Type::Struct { fields, .. } => self.layout_struct(fields),
            Type::Enum { name, .. } => self.layout_named(name),
            Type::Array { elem, size } => {
                let elem_layout = self.layout_type(elem);
                self.cache_compound_layout(
                    ty,
                    "array",
                    elem_layout.size * size.unwrap_or_default() as u32,
                    elem_layout.alignment,
                )
            }
            Type::Slice(_) => {
                self.cache_compound_layout(ty, "slice", POINTER_SIZE + USIZE_SIZE, POINTER_ALIGN)
            }
            Type::Ptr(_) | Type::MutPtr(_) | Type::RawPtr(_) | Type::Function { .. } => {
                self.cache_compound_layout(ty, "pointer", POINTER_SIZE, POINTER_ALIGN)
            }
            Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Usize
            | Type::F32
            | Type::F64
            | Type::Bool
            | Type::Void
            | Type::Never
            | Type::Unknown
            | Type::Str
            | Type::String => {
                unreachable!("builtin layout types are handled before compound types")
            }
        }
    }

    fn cache_compound_layout(
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

    fn layout_named(&mut self, name: &str) -> LayoutJsonType {
        if let Some(layout) = self.layouts.get(name) {
            return layout.clone();
        }
        if let Some(type_def) = self.types.get(name).copied() {
            return self.layout_type_def(type_def);
        }
        scalar_layout("opaque", 0, 1)
    }

    fn layout_by_name(&self, name: &str) -> LayoutJsonType {
        self.layouts
            .get(name)
            .cloned()
            .unwrap_or_else(|| scalar_layout("opaque", 0, 1))
    }
}
