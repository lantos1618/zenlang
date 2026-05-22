use super::super::is_generated_c_function_name;

pub(super) fn starts_c_function_body(trimmed: &str) -> bool {
    trimmed.ends_with('{') && is_any_c_function_signature_line(trimmed)
}

pub(super) fn brace_delta(trimmed: &str, current: usize) -> usize {
    let opens = trimmed.bytes().filter(|byte| *byte == b'{').count();
    let closes = trimmed.bytes().filter(|byte| *byte == b'}').count();
    current.saturating_add(opens).saturating_sub(closes)
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
        && is_generated_c_function_name(name)
        && !before.contains('=')
        && !before.contains("return")
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
