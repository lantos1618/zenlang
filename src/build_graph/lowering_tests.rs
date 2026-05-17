use super::{BuildTargetDslIdent, BuildTargetDslKind, BuildTargetField};

#[test]
fn build_target_dsl_kind_owns_source_spelling() {
    assert_eq!(BuildTargetDslKind::Executable.as_str(), "Executable");
    assert_eq!(BuildTargetDslKind::Test.as_str(), "Test");
    assert_eq!(BuildTargetDslKind::Library.as_str(), "Library");
    assert_eq!("Executable".parse(), Ok(BuildTargetDslKind::Executable));
    assert_eq!("Test".parse(), Ok(BuildTargetDslKind::Test));
    assert_eq!("Library".parse(), Ok(BuildTargetDslKind::Library));
    assert!("Benchmark".parse::<BuildTargetDslKind>().is_err());
    assert_eq!(BuildTargetDslKind::Executable.to_string(), "Executable");
    assert_eq!(BuildTargetDslKind::Test.to_string(), "Test");
    assert_eq!(BuildTargetDslKind::Library.to_string(), "Library");
    assert_eq!(
        BuildTargetDslKind::supported_display_list(),
        "`Executable`, `Test`, and `Library`"
    );
}

#[test]
fn build_target_field_owns_source_spelling() {
    assert_eq!(BuildTargetField::Name.as_str(), "name");
    assert_eq!(BuildTargetField::Main.as_str(), "main");
    assert_eq!(BuildTargetField::Root.as_str(), "root");
    assert_eq!(
        BuildTargetField::RootSourceFile.as_str(),
        "root_source_file"
    );
    assert_eq!(BuildTargetField::OutDir.as_str(), "out_dir");
    assert_eq!(BuildTargetField::Dependencies.as_str(), "dependencies");
    assert_eq!(BuildTargetField::Features.as_str(), "features");
    assert_eq!(BuildTargetField::Exports.as_str(), "exports");
    assert_eq!(
        BuildTargetField::RootSourceFile.to_string(),
        "root_source_file"
    );
}

#[test]
fn build_target_dsl_ident_owns_source_spelling() {
    assert_eq!(BuildTargetDslIdent::Builder.as_str(), "b");
    assert_eq!(BuildTargetDslIdent::Add.as_str(), "add");
    assert_eq!(BuildTargetDslIdent::Build.as_str(), "build");
    assert_eq!(BuildTargetDslIdent::Env.as_str(), "env");
    assert_eq!(BuildTargetDslIdent::Os.as_str(), "os");
    assert_eq!(BuildTargetDslIdent::ReadFile.as_str(), "read_file");
    assert_eq!(BuildTargetDslIdent::Builder.to_string(), "b");
}
