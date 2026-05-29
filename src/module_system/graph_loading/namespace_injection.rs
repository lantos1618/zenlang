use std::collections::HashSet;

use crate::ast::{Declaration, Program};
use crate::error::CompileError;
use crate::root_spelling::{AT_STD_ROOT, STD_ROOT};
use crate::{lexer, parser};

use super::namespace_refs::rename_expr_refs;
use crate::module_system::stdlib_paths::find_stdlib_root;

/// Splice bare `{ io } = std` namespace imports into `program`.
///
/// A bare-`std` import names a stdlib namespace module (`stdlib/io/io.zen`); its
/// public functions are cloned in under the `<name>_` prefix (`println` →
/// `io_println`) so `io.println()` — mangled to `io_println` — resolves to real
/// Zen built on `@builtin` intrinsics rather than a hardcoded C stand-in.
/// Intra-module references (recursion, sibling calls) are rewritten to the
/// prefixed name. Only `stdlib/<name>/<name>.zen` is spliced; names that don't
/// resolve to such a file (builtin types, flat modules) are left alone.
pub(super) fn inject_stdlib_namespace_functions(
    program: &mut Program,
) -> Result<(), Vec<CompileError>> {
    let Some(root) = find_stdlib_root() else {
        return Ok(());
    };

    let names: Vec<String> = program
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            Declaration::Import {
                names, module_path, ..
            } if module_path.len() == 1
                && matches!(module_path[0].as_str(), STD_ROOT | AT_STD_ROOT) =>
            {
                Some(names.clone())
            }
            _ => None,
        })
        .flatten()
        .collect();

    let mut injected = Vec::new();
    let mut spliced: HashSet<String> = HashSet::new();
    for name in names {
        // The same namespace may be requested by more than one import
        // statement (`{ io } = std` twice); splice it only once.
        if !spliced.insert(name.clone()) {
            continue;
        }
        let file = root.join(&name).join(format!("{name}.zen"));
        if !file.exists() {
            continue;
        }
        let source = std::fs::read_to_string(&file).map_err(|e| {
            vec![CompileError::Internal(format!(
                "cannot read stdlib namespace module {}: {e}",
                file.display()
            ))]
        })?;
        // Parse with file id 0 so the namespace's spans attribute to the
        // importing program and never leak into IR/diagnostics JSON.
        let tokens = lexer::tokenize(&source, 0).map_err(|e| vec![e])?;
        let dep = parser::parse(tokens, 0)?;
        collect_namespace_module(&dep, &name, &mut injected);
    }

    // A function the program already declares (user-defined, or carried in by
    // another path) wins over a spliced one — drop the collision rather than
    // emit a duplicate `<prefix>_<fn>` definition.
    let existing: HashSet<&str> = program
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Function { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    injected.retain(|d| match d {
        Declaration::Function { name, .. } => !existing.contains(name.as_str()),
        _ => true,
    });

    program.declarations.append(&mut injected);
    Ok(())
}

fn collect_namespace_module(dep: &Program, prefix: &str, out: &mut Vec<Declaration>) {
    let fn_names: HashSet<String> = dep
        .declarations
        .iter()
        .filter_map(|d| match d {
            Declaration::Function { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    for decl in &dep.declarations {
        if let Declaration::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        {
            let mut body = body.clone();
            rename_expr_refs(&mut body, &fn_names, prefix);
            out.push(Declaration::Function {
                name: format!("{prefix}_{name}"),
                type_params: type_params.clone(),
                params: params.clone(),
                return_type: return_type.clone(),
                body,
                public: false,
                span: *span,
            });
        }
    }
}
