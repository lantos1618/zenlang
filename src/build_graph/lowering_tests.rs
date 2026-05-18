use super::{BuildTargetDslIdent, BuildTargetDslKind, BuildTargetField, HostEffectResultVariant};

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
    assert_eq!(BuildTargetField::Packages.as_str(), "packages");
    assert_eq!(BuildTargetField::Link.as_str(), "link");
    assert_eq!("name".parse(), Ok(BuildTargetField::Name));
    assert_eq!(
        "root_source_file".parse(),
        Ok(BuildTargetField::RootSourceFile)
    );
    assert_eq!("packages".parse(), Ok(BuildTargetField::Packages));
    assert_eq!("link".parse(), Ok(BuildTargetField::Link));
    assert!("output_dir".parse::<BuildTargetField>().is_err());
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
    assert_eq!("b".parse(), Ok(BuildTargetDslIdent::Builder));
    assert_eq!("add".parse(), Ok(BuildTargetDslIdent::Add));
    assert_eq!("build".parse(), Ok(BuildTargetDslIdent::Build));
    assert_eq!("env".parse(), Ok(BuildTargetDslIdent::Env));
    assert_eq!("os".parse(), Ok(BuildTargetDslIdent::Os));
    assert_eq!("read_file".parse(), Ok(BuildTargetDslIdent::ReadFile));
    assert!("read_env".parse::<BuildTargetDslIdent>().is_err());
    assert_eq!(BuildTargetDslIdent::Builder.to_string(), "b");
}

#[test]
fn host_effect_result_variant_owns_source_spelling() {
    assert_eq!(HostEffectResultVariant::Ok.as_str(), "Ok");
    assert_eq!(HostEffectResultVariant::Err.as_str(), "Err");
    assert_eq!("Ok".parse(), Ok(HostEffectResultVariant::Ok));
    assert_eq!("Err".parse(), Ok(HostEffectResultVariant::Err));
    assert!("Missing".parse::<HostEffectResultVariant>().is_err());
    assert_eq!(HostEffectResultVariant::Ok.to_string(), "Ok");
    assert_eq!(HostEffectResultVariant::Err.to_string(), "Err");
}
