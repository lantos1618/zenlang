use serde::Serialize;

use crate::ast::typed::{Type, TypeDefKind, TypedFunction, TypedProgram, TypedTypeDef};

#[derive(Serialize)]
struct HirJsonProgram {
    format: &'static str,
    schema_version: u32,
    semantic_status: &'static str,
    declarations: HirDeclarations,
}

#[derive(Serialize)]
struct HirDeclarations {
    types: Vec<HirTypeDecl>,
    functions: Vec<HirFunctionDecl>,
    globals: Vec<HirGlobalDecl>,
}

#[derive(Serialize)]
struct HirTypeDecl {
    name: String,
    kind: &'static str,
    fields: Vec<HirField>,
    variants: Vec<HirVariant>,
}

#[derive(Serialize)]
struct HirField {
    name: String,
    r#type: String,
}

#[derive(Serialize)]
struct HirVariant {
    name: String,
    tag: u32,
    payload: Vec<HirField>,
}

#[derive(Serialize)]
struct HirFunctionDecl {
    name: String,
    params: Vec<HirParam>,
    return_type: String,
}

#[derive(Serialize)]
struct HirParam {
    name: String,
    r#type: String,
}

#[derive(Serialize)]
struct HirGlobalDecl {
    name: String,
    r#type: String,
    mutable: bool,
}

pub(super) fn program_to_json(program: &TypedProgram) -> serde_json::Result<String> {
    let graph = HirJsonProgram {
        format: "zen.hir.v0",
        schema_version: 0,
        semantic_status: "checked",
        declarations: HirDeclarations {
            types: program.types.iter().map(hir_type_decl).collect(),
            functions: program.functions.iter().map(hir_function_decl).collect(),
            globals: program
                .globals
                .iter()
                .map(|global| HirGlobalDecl {
                    name: global.name.clone(),
                    r#type: global.ty.display_name(),
                    mutable: global.mutable,
                })
                .collect(),
        },
    };

    serde_json::to_string_pretty(&graph)
}

fn hir_type_decl(type_def: &TypedTypeDef) -> HirTypeDecl {
    match &type_def.kind {
        TypeDefKind::Struct { fields } => HirTypeDecl {
            name: type_def.name.clone(),
            kind: "struct",
            fields: hir_fields(fields),
            variants: Vec::new(),
        },
        TypeDefKind::Enum { variants } => HirTypeDecl {
            name: type_def.name.clone(),
            kind: "enum",
            fields: Vec::new(),
            variants: variants
                .iter()
                .map(|variant| HirVariant {
                    name: variant.name.clone(),
                    tag: variant.tag,
                    payload: variant
                        .payload
                        .as_deref()
                        .map(hir_fields)
                        .unwrap_or_default(),
                })
                .collect(),
        },
    }
}

fn hir_function_decl(function: &TypedFunction) -> HirFunctionDecl {
    HirFunctionDecl {
        name: function.name.clone(),
        params: function
            .params
            .iter()
            .map(|param| HirParam {
                name: param.name.clone(),
                r#type: param.ty.display_name(),
            })
            .collect(),
        return_type: function.return_type.display_name(),
    }
}

fn hir_fields(fields: &[(String, Type)]) -> Vec<HirField> {
    fields
        .iter()
        .map(|(name, ty)| HirField {
            name: name.clone(),
            r#type: ty.display_name(),
        })
        .collect()
}
