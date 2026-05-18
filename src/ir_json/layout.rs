use std::collections::BTreeMap;

use serde::Serialize;

use crate::ast::typed::{Type, TypeDefKind, TypedProgram, TypedTypeDef};

#[derive(Serialize)]
struct LayoutJsonProgram {
    format: &'static str,
    semantic_status: &'static str,
    target: LayoutJsonTarget,
    layouts: BTreeMap<String, LayoutJsonType>,
}

#[derive(Serialize)]
struct LayoutJsonTarget {
    pointer_size: u32,
    pointer_alignment: u32,
    usize_size: u32,
}

#[derive(Clone, Serialize)]
struct LayoutJsonType {
    kind: &'static str,
    size: u32,
    alignment: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<LayoutJsonField>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    variants: Vec<LayoutJsonVariant>,
}

#[derive(Clone, Serialize)]
struct LayoutJsonField {
    name: String,
    r#type: String,
    offset: u32,
}

#[derive(Clone, Serialize)]
struct LayoutJsonVariant {
    name: String,
    tag: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    payload_fields: Vec<LayoutJsonField>,
}

pub(super) fn program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let context = LayoutContext::new(program);
    let graph = LayoutJsonProgram {
        format: "zen.layout.v0",
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

const POINTER_SIZE: u32 = 8;
const POINTER_ALIGN: u32 = 8;
const USIZE_SIZE: u32 = 8;

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
        context.seed_builtin_layouts();
        for type_def in &program.types {
            context.layout_type_def(type_def);
        }
        context
    }

    fn layouts(self) -> BTreeMap<String, LayoutJsonType> {
        self.layouts
    }

    fn seed_builtin_layouts(&mut self) {
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
        match ty {
            Type::I8 => self.layout_by_name("i8"),
            Type::I16 => self.layout_by_name("i16"),
            Type::I32 => self.layout_by_name("i32"),
            Type::I64 => self.layout_by_name("i64"),
            Type::U8 => self.layout_by_name("u8"),
            Type::U16 => self.layout_by_name("u16"),
            Type::U32 => self.layout_by_name("u32"),
            Type::U64 => self.layout_by_name("u64"),
            Type::Usize => self.layout_by_name("usize"),
            Type::F32 => self.layout_by_name("f32"),
            Type::F64 => self.layout_by_name("f64"),
            Type::Bool => self.layout_by_name("bool"),
            Type::Void | Type::Never | Type::Unknown => self.layout_by_name("void"),
            Type::Str => self.layout_by_name("StaticString"),
            Type::String => self.layout_by_name("String"),
            Type::Named(name) => self.layout_named(name),
            Type::Struct { fields, .. } => self.layout_struct(fields),
            Type::Enum { name, .. } => self.layout_named(name),
            Type::Array { elem, size } => {
                let elem_layout = self.layout_type(elem);
                scalar_layout(
                    "array",
                    elem_layout.size * size.unwrap_or_default() as u32,
                    elem_layout.alignment,
                )
            }
            Type::Slice(_) => scalar_layout("slice", POINTER_SIZE + USIZE_SIZE, POINTER_ALIGN),
            Type::Ptr(_) | Type::MutPtr(_) | Type::RawPtr(_) | Type::Function { .. } => {
                scalar_layout("pointer", POINTER_SIZE, POINTER_ALIGN)
            }
        }
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

fn scalar_layout(kind: &'static str, size: u32, alignment: u32) -> LayoutJsonType {
    LayoutJsonType {
        kind,
        size,
        alignment,
        fields: Vec::new(),
        variants: Vec::new(),
    }
}

fn align_to(value: u32, alignment: u32) -> u32 {
    if alignment <= 1 {
        value
    } else {
        value.div_ceil(alignment) * alignment
    }
}
