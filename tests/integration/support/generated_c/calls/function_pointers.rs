pub(super) fn c_function_pointer_bindings(line: &str) -> Vec<String> {
    let mut bindings = Vec::new();
    collect_function_pointer_bindings_on_line(line, &mut bindings);
    bindings
}

pub(super) fn collect_function_pointer_bindings_on_line(line: &str, bindings: &mut Vec<String>) {
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
