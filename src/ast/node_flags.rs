use bitflags::bitflags;

bitflags! {

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct NodeFlags: u32 {

        const Let                             = 1 << 0;

        const Const                           = 1 << 1;

        const Using                           = 1 << 2;

        const Reparsed                        = 1 << 3;

        const Synthesized                     = 1 << 4;

        const OptionalChain                   = 1 << 5;

        const ExportContext                   = 1 << 6;

        const ContainsThis                    = 1 << 7;

        const HasImplicitReturn               = 1 << 8;

        const HasExplicitReturn               = 1 << 9;

        const DisallowInContext               = 1 << 10;

        const YieldContext                    = 1 << 11;

        const DecoratorContext                = 1 << 12;

        const AwaitContext                    = 1 << 13;

        const DisallowConditionalTypesContext = 1 << 14;

        const ThisNodeHasError                = 1 << 15;

        const JavaScriptFile                  = 1 << 16;

        const ThisNodeOrAnySubNodesHasError   = 1 << 17;

        const HasAsyncFunctions               = 1 << 18;

        const PossiblyContainsDynamicImport   = 1 << 19;

        const PossiblyContainsImportMeta      = 1 << 20;

        const HasJSDoc                        = 1 << 21;

        const JSDoc                           = 1 << 22;

        const Ambient                        = 1 << 23;

        const InWithStatement                = 1 << 24;

        const JsonFile                       = 1 << 25;

        const PossiblyContainsDeprecatedTag  = 1 << 26;

        const Unreachable                    = 1 << 27;

        const ReparserTransformedLiteral     = 1 << 28;
    }
}

#[allow(non_upper_case_globals)]
impl NodeFlags {

    pub const BlockScoped: Self = Self::Let.union(Self::Const).union(Self::Using);

    pub const Constant: Self = Self::Const.union(Self::Using);

    pub const AwaitUsing: Self = Self::Const.union(Self::Using);

    pub const ReachabilityCheckFlags: Self = Self::HasImplicitReturn.union(Self::HasExplicitReturn);

    pub const ReachabilityAndEmitFlags: Self =
        Self::ReachabilityCheckFlags.union(Self::HasAsyncFunctions);

    pub const ContextFlags: Self = Self::DisallowInContext
        .union(Self::DisallowConditionalTypesContext)
        .union(Self::YieldContext)
        .union(Self::DecoratorContext)
        .union(Self::AwaitContext)
        .union(Self::JavaScriptFile)
        .union(Self::InWithStatement)
        .union(Self::Ambient);

    pub const TypeExcludesFlags: Self = Self::YieldContext.union(Self::AwaitContext);

    pub const PermanentlySetIncrementalFlags: Self =
        Self::PossiblyContainsDynamicImport.union(Self::PossiblyContainsImportMeta);
}

bitflags! {

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ModifierFlags: u32 {

        const Public    = 1 << 0;

        const Private   = 1 << 1;

        const Protected = 1 << 2;

        const Readonly  = 1 << 3;

        const Override  = 1 << 4;

        const Export    = 1 << 5;

        const Abstract  = 1 << 6;

        const Ambient   = 1 << 7;

        const Static    = 1 << 8;

        const Accessor  = 1 << 9;

        const Async     = 1 << 10;

        const Default   = 1 << 11;

        const Const     = 1 << 12;

        const In        = 1 << 13;

        const Out       = 1 << 14;

        const Decorator = 1 << 15;

        const Deprecated = 1 << 16;

        const JSDocPublic               = 1 << 23;
        const JSDocPrivate              = 1 << 24;
        const JSDocProtected            = 1 << 25;
        const JSDocReadonly             = 1 << 26;
        const JSDocOverride             = 1 << 27;
        const HasComputedJSDocModifiers = 1 << 28;
        const HasComputedFlags          = 1 << 29;
    }
}

#[allow(non_upper_case_globals)]
impl ModifierFlags {

    pub const SyntacticOrJSDocModifiers: Self = Self::Public
        .union(Self::Private)
        .union(Self::Protected)
        .union(Self::Readonly)
        .union(Self::Override);

    pub const SyntacticOnlyModifiers: Self = Self::Export
        .union(Self::Ambient)
        .union(Self::Abstract)
        .union(Self::Static)
        .union(Self::Accessor)
        .union(Self::Async)
        .union(Self::Default)
        .union(Self::Const)
        .union(Self::In)
        .union(Self::Out)
        .union(Self::Decorator);

    pub const SyntacticModifiers: Self =
        Self::SyntacticOrJSDocModifiers.union(Self::SyntacticOnlyModifiers);

    pub const AccessibilityModifier: Self =
        Self::Public.union(Self::Private).union(Self::Protected);

    pub const ParameterPropertyModifier: Self = Self::AccessibilityModifier
        .union(Self::Readonly)
        .union(Self::Override);

    pub const NonPublicAccessibilityModifier: Self = Self::Private.union(Self::Protected);

    pub const Modifier: Self = Self::All.difference(Self::Decorator);

    pub const JavaScript: Self = Self::Export
        .union(Self::Static)
        .union(Self::Accessor)
        .union(Self::Async)
        .union(Self::Default);

    pub const All: Self = Self::Export
        .union(Self::Ambient)
        .union(Self::Public)
        .union(Self::Private)
        .union(Self::Protected)
        .union(Self::Static)
        .union(Self::Readonly)
        .union(Self::Abstract)
        .union(Self::Accessor)
        .union(Self::Async)
        .union(Self::Default)
        .union(Self::Const)
        .union(Self::Deprecated)
        .union(Self::Override)
        .union(Self::In)
        .union(Self::Out)
        .union(Self::Decorator);

    pub const ExportDefault: Self = Self::Export.union(Self::Default);
}
