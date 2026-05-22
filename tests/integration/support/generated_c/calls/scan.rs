use super::super::definitions::c_function_definitions;
use super::super::is_generated_c_function_name;
use super::function_pointers::{
    c_function_pointer_bindings, collect_function_pointer_bindings_on_line,
};
use super::signatures::{brace_delta, is_any_c_function_signature_line, starts_c_function_body};

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
