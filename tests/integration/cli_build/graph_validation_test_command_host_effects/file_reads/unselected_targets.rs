#[path = "unselected_targets/declared.rs"]
mod declared;
#[path = "unselected_targets/rejections.rs"]
mod rejections;

fn write_zero_main(path: impl AsRef<std::path::Path>) {
    std::fs::write(
        path,
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write zero main");
}
