use super::super::*;

pub(super) fn assert_needles_moved_to_focused_file(
    root_path: &str,
    focused_path: &str,
    needles: &[&str],
    root_label: &str,
    focused_label: &str,
) {
    let root = read(root_path);
    let focused = read(focused_path);

    for needle in needles {
        assert!(
            !root.contains(needle),
            "{root_label} should not own moved item `{needle}`"
        );
        assert!(
            focused.contains(needle),
            "{focused_label} should contain moved item `{needle}`"
        );
    }
}

pub(super) fn assert_file_contains(path: &str, needle: &str, message: &str) {
    let source = read(path);

    assert!(source.contains(needle), "{message}");
}

pub(super) fn assert_file_lacks(path: &str, needle: &str, message: &str) {
    let source = read(path);

    assert!(!source.contains(needle), "{message}");
}

pub(super) fn assert_file_line_count_below(path: &str, max_lines: usize, message: &str) {
    let source = read(path);

    assert!(source.lines().count() < max_lines, "{message}");
}
