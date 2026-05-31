pub(super) fn has_c_call_outside_signature(c_source: &str, name: &str) -> bool {
    let call = format!("{name}(");
    c_source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains(&call) && !is_c_function_signature_line(trimmed, name)
    })
}

pub(super) fn is_any_c_function_signature_line(trimmed: &str) -> bool {
    if !(trimmed.ends_with(';') || trimmed.ends_with('{')) {
        return false;
    }

    let Some((before, name)) = c_function_head(trimmed) else {
        return false;
    };
    !before[..before.len() - name.len()].trim().is_empty()
        && is_tracked_c_function_name(name)
        && !before.contains('=')
        && !before.contains("return")
}

pub(super) fn generated_c_calls_on_line(trimmed: &str) -> Vec<&str> {
    let mut calls = Vec::new();
    let bytes = trimmed.as_bytes();
    let mut index = 0;

    while let Some(relative) = trimmed[index..].find('(') {
        let paren = index + relative;
        let mut start = paren;
        while start > 0 {
            let ch = bytes[start - 1] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                start -= 1;
            } else {
                break;
            }
        }

        let name = &trimmed[start..paren];
        if is_tracked_c_function_name(name) && !calls.contains(&name) {
            calls.push(name);
        }

        index = paren + 1;
    }

    calls
}

pub(super) fn is_tracked_c_function_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && !is_untracked_c_call_name(name)
}

pub(super) fn c_function_head(trimmed: &str) -> Option<(&str, &str)> {
    let paren = trimmed.find('(')?;
    let before = trimmed[..paren].trim_end();
    let name_start = before
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    Some((before, before[name_start..].trim()))
}

fn is_untracked_c_call_name(name: &str) -> bool {
    // Compiler builtins (`__builtin_popcountll`, `__builtin_bswap64`, the
    // overflow-checked arithmetic, trap/unreachable, …) and the atomic builtins
    // (`__atomic_load_n`, `__atomic_compare_exchange_n`, `__atomic_thread_fence`,
    // …) are provided by the C compiler, never emitted as Zen function
    // definitions.
    if name.starts_with("__builtin_") || name.starts_with("__atomic_") {
        return true;
    }
    matches!(
        name,
        "abort"
            | "ceil"
            | "dlclose"
            | "dlerror"
            | "dlopen"
            | "dlsym"
            | "floor"
            | "for"
            | "fprintf"
            | "fputc"
            | "free"
            | "fwrite"
            | "getenv"
            | "if"
            | "isatty"
            | "malloc"
            | "memcmp"
            | "memcpy"
            | "memmove"
            | "memset"
            | "pow"
            | "printf"
            | "read"
            | "realloc"
            | "snprintf"
            | "sizeof"
            | "sqrt"
            | "strlen"
            | "switch"
            | "syscall"
            | "while"
            | "write"
    )
}

fn is_c_function_signature_line(trimmed: &str, name: &str) -> bool {
    let Some((before, signature_name)) = c_function_head(trimmed) else {
        return false;
    };
    signature_name == name
        && !before.contains('=')
        && !before.contains("return")
        && (trimmed.ends_with(';') || trimmed.ends_with('{'))
}
