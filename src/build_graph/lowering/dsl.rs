#[path = "dsl/fields.rs"]
mod fields;
#[path = "dsl/host_effects.rs"]
mod host_effects;
#[path = "dsl/idents.rs"]
mod idents;
#[path = "dsl/kinds.rs"]
mod kinds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuildTargetDslKind {
    Executable,
    Test,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum BuildTargetField {
    Name,
    Main,
    Root,
    RootSourceFile,
    OutDir,
    Dependencies,
    Features,
    Exports,
    Packages,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BuildTargetDslIdent {
    Builder,
    Add,
    Build,
    Env,
    Os,
    ReadFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HostEffectResultVariant {
    Ok,
    Err,
}
