mod behavior;
mod module_roots;
mod mutability;
mod pattern;
mod prefix;
mod this_methods;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserPrefixKeyword {
    True,
    False,
    Return,
    As,
    Break,
    Continue,
    Loop,
    Cast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserPatternKeyword {
    True,
    False,
    Wildcard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserThisMethod {
    Defer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserModuleRoot {
    AtBuiltin,
    AtStd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserMutabilityKeyword {
    Mut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ParserBehaviorKeyword {
    Behavior,
}
