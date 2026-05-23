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

    let Some(paren) = trimmed.find('(') else {
        return false;
    };
    let before = &trimmed[..paren];
    let name_start = before
        .trim_end()
        .rfind(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .map_or(0, |idx| idx + 1);
    let return_type = before[..name_start].trim();
    let name = before[name_start..].trim();

    !return_type.is_empty()
        && is_tracked_c_function_name(name)
        && !before.contains('=')
        && !before.contains("return")
}

pub(super) fn generated_c_calls_on_line(trimmed: &str) -> Vec<String> {
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
        if is_tracked_c_function_name(name) && !calls.iter().any(|call| call == name) {
            calls.push(name.to_string());
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

fn is_untracked_c_call_name(name: &str) -> bool {
    matches!(
        name,
        "abort"
            | "ceil"
            | "floor"
            | "for"
            | "fprintf"
            | "fputc"
            | "free"
            | "fwrite"
            | "if"
            | "malloc"
            | "memcpy"
            | "pow"
            | "printf"
            | "snprintf"
            | "sizeof"
            | "sqrt"
            | "strlen"
            | "switch"
            | "while"
    )
}

fn is_c_function_signature_line(trimmed: &str, name: &str) -> bool {
    let needle = format!(" {name}(");
    let Some(call_start) = trimmed.find(&needle) else {
        return false;
    };
    let prefix = &trimmed[..call_start];
    !prefix.contains('=')
        && !prefix.contains("return")
        && (trimmed.ends_with(';') || trimmed.ends_with('{'))
}
