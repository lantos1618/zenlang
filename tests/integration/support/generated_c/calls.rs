use super::definitions::{assert_c_function_definition, c_function_definitions};
use super::is_generated_c_function_name;

pub fn assert_c_call_resolves_to_definition(c_source: &str, name: &str) {
    assert_c_function_definition(c_source, name);
    assert!(
        has_c_call_outside_signature(c_source, name),
        "expected generated C call to `{name}` outside declarations/definitions:\n{c_source}"
    );
}

pub fn assert_generated_c_calls_resolve_to_definitions(c_source: &str) {
    let undefined = undefined_generated_c_calls(c_source);
    assert!(
        undefined.is_empty(),
        "generated C calls missing emitted definitions: {undefined:?}\n{c_source}"
    );
}

pub fn undefined_generated_c_calls(c_source: &str) -> Vec<String> {
    let definitions = c_function_definitions(c_source);
    let mut function_pointer_bindings = Vec::new();
    let mut function_body_depth = 0usize;
    let mut undefined = Vec::new();

    for line in c_source.lines() {
        let trimmed = line.trim();
        if starts_c_function_body(trimmed) {
            function_pointer_bindings = c_function_pointer_bindings(trimmed);
            function_body_depth = brace_delta(trimmed, 0);
            continue;
        }

        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("typedef ")
            || is_any_c_function_signature_line(trimmed)
        {
            if function_body_depth > 0 {
                function_body_depth = brace_delta(trimmed, function_body_depth);
                if function_body_depth == 0 {
                    function_pointer_bindings.clear();
                }
            }
            continue;
        }

        if function_body_depth > 0 {
            collect_function_pointer_bindings_on_line(trimmed, &mut function_pointer_bindings);
        }

        for call in generated_c_calls_on_line(trimmed) {
            if !definitions.contains(&call)
                && !function_pointer_bindings.contains(&call)
                && !undefined.contains(&call)
            {
                undefined.push(call);
            }
        }

        if function_body_depth > 0 {
            function_body_depth = brace_delta(trimmed, function_body_depth);
            if function_body_depth == 0 {
                function_pointer_bindings.clear();
            }
        }
    }

    undefined
}

fn starts_c_function_body(trimmed: &str) -> bool {
    trimmed.ends_with('{') && is_any_c_function_signature_line(trimmed)
}

fn brace_delta(trimmed: &str, current: usize) -> usize {
    let opens = trimmed.bytes().filter(|byte| *byte == b'{').count();
    let closes = trimmed.bytes().filter(|byte| *byte == b'}').count();
    current.saturating_add(opens).saturating_sub(closes)
}

fn is_any_c_function_signature_line(trimmed: &str) -> bool {
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
        && is_generated_c_function_name(name)
        && !before.contains('=')
        && !before.contains("return")
}

fn generated_c_calls_on_line(trimmed: &str) -> Vec<String> {
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
        if is_generated_c_function_name(name) && !calls.iter().any(|call| call == name) {
            calls.push(name.to_string());
        }

        index = paren + 1;
    }

    calls
}

fn c_function_pointer_bindings(line: &str) -> Vec<String> {
    let mut bindings = Vec::new();
    collect_function_pointer_bindings_on_line(line, &mut bindings);
    bindings
}

fn collect_function_pointer_bindings_on_line(line: &str, bindings: &mut Vec<String>) {
    let bytes = line.as_bytes();
    let mut index = 0;

    while let Some(relative) = line[index..].find("(*") {
        let mut name_start = index + relative + 2;
        while name_start < bytes.len() && (bytes[name_start] as char).is_ascii_whitespace() {
            name_start += 1;
        }

        let mut name_end = name_start;
        while name_end < bytes.len() {
            let ch = bytes[name_end] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' {
                name_end += 1;
            } else {
                break;
            }
        }

        if name_end > name_start
            && function_pointer_binding_has_call_suffix(bytes, name_end)
            && !bindings
                .iter()
                .any(|binding| binding == &line[name_start..name_end])
        {
            bindings.push(line[name_start..name_end].to_string());
        }

        index = name_end.max(name_start + 1);
    }
}

fn function_pointer_binding_has_call_suffix(bytes: &[u8], mut index: usize) -> bool {
    while index < bytes.len() && (bytes[index] as char).is_ascii_whitespace() {
        index += 1;
    }
    if index >= bytes.len() || bytes[index] != b')' {
        return false;
    }

    index += 1;
    while index < bytes.len() && (bytes[index] as char).is_ascii_whitespace() {
        index += 1;
    }

    index < bytes.len() && bytes[index] == b'('
}

pub fn has_c_call_outside_signature(c_source: &str, name: &str) -> bool {
    let call = format!("{name}(");
    c_source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains(&call) && !is_c_function_signature_line(trimmed, name)
    })
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
