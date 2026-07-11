//! Node flags, ported from `internal/ast/nodeflags.go`.

use bitflags::bitflags;

bitflags! {
    /// Flags on an AST node, tracking parsing context and node properties.
    ///
    /// Mirrors `ast.NodeFlags` in Go.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct NodeFlags: u32 {
        /// Variable declaration with `let`.
        const Let                             = 1 << 0;
        /// Variable declaration with `const`.
        const Const                           = 1 << 1;
        /// Variable declaration with `using`.
        const Using                           = 1 << 2;
        /// Node was synthesized during parsing.
        const Reparsed                        = 1 << 3;
        /// Node was synthesized during transformation.
        const Synthesized                     = 1 << 4;
        /// Chained MemberExpression rooted to a pseudo-OptionalExpression.
        const OptionalChain                   = 1 << 5;
        /// Export context (initialized by binding).
        const ExportContext                   = 1 << 6;
        /// Interface contains references to "this".
        const ContainsThis                    = 1 << 7;
        /// If function implicitly returns on one of codepaths.
        const HasImplicitReturn               = 1 << 8;
        /// If function has explicit reachable return.
        const HasExplicitReturn               = 1 << 9;
        /// If node was parsed in a context where 'in-expressions' are not allowed.
        const DisallowInContext               = 1 << 10;
        /// If node was parsed in the 'yield' context.
        const YieldContext                    = 1 << 11;
        /// If node was parsed as part of a decorator.
        const DecoratorContext                = 1 << 12;
        /// If node was parsed in the 'await' context.
        const AwaitContext                    = 1 << 13;
        /// If node was parsed in a context where conditional types are not allowed.
        const DisallowConditionalTypesContext = 1 << 14;
        /// If the parser encountered an error when parsing the code that created this node.
        const ThisNodeHasError                = 1 << 15;
        /// If node was parsed in a JavaScript file.
        const JavaScriptFile                  = 1 << 16;
        /// If this node or any of its children had an error.
        const ThisNodeOrAnySubNodesHasError   = 1 << 17;
        /// If the file has async functions.
        const HasAsyncFunctions               = 1 << 18;
        /// Possibly contains a dynamic import expression.
        const PossiblyContainsDynamicImport   = 1 << 19;
        /// Possibly contains import.meta.
        const PossiblyContainsImportMeta      = 1 << 20;
        /// If node has preceding JSDoc comment(s).
        const HasJSDoc                        = 1 << 21;
        /// If node was parsed inside jsdoc.
        const JSDoc                           = 1 << 22;
        /// If node was inside an ambient context.
        const Ambient                        = 1 << 23;
        /// If any ancestor was the statement of a WithStatement.
        const InWithStatement                = 1 << 24;
        /// If node was parsed in a JSON file.
        const JsonFile                       = 1 << 25;
        /// Set during parse if comment text contains '@deprecated'.
        const PossiblyContainsDeprecatedTag  = 1 << 26;
        /// If node is unreachable according to the binder.
        const Unreachable                    = 1 << 27;
        /// If node was transformed during parsing.
        const ReparserTransformedLiteral     = 1 << 28;
    }
}

impl NodeFlags {
    /// Block-scoped variable flags: `let | const | using`.
    pub const BlockScoped: Self = Self::Let.union(Self::Const).union(Self::Using);
    /// Constant variable flags: `const | using`.
    pub const Constant: Self = Self::Const.union(Self::Using);
    /// Await-using: `const | using`.
    pub const AwaitUsing: Self = Self::Const.union(Self::Using);
    /// Reachability check flags.
    pub const ReachabilityCheckFlags: Self = Self::HasImplicitReturn.union(Self::HasExplicitReturn);
    /// Reachability and emit flags.
    pub const ReachabilityAndEmitFlags: Self =
        Self::ReachabilityCheckFlags.union(Self::HasAsyncFunctions);
    /// All context flags.
    pub const ContextFlags: Self = Self::DisallowInContext
        .union(Self::DisallowConditionalTypesContext)
        .union(Self::YieldContext)
        .union(Self::DecoratorContext)
        .union(Self::AwaitContext)
        .union(Self::JavaScriptFile)
        .union(Self::InWithStatement)
        .union(Self::Ambient);
    /// Flags excluded when parsing a Type.
    pub const TypeExcludesFlags: Self = Self::YieldContext.union(Self::AwaitContext);
    /// Permanently set incremental flags.
    pub const PermanentlySetIncrementalFlags: Self =
        Self::PossiblyContainsDynamicImport.union(Self::PossiblyContainsImportMeta);
}

bitflags! {
    /// Modifier flags, tracking syntactic and JSDoc modifiers on declarations.
    ///
    /// Mirrors `ast.ModifierFlags` in Go.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ModifierFlags: u32 {
        /// `public` modifier.
        const Public    = 1 << 0;
        /// `private` modifier.
        const Private   = 1 << 1;
        /// `protected` modifier.
        const Protected = 1 << 2;
        /// `readonly` modifier.
        const Readonly  = 1 << 3;
        /// `override` modifier.
        const Override  = 1 << 4;
        /// `export` modifier.
        const Export    = 1 << 5;
        /// `abstract` modifier.
        const Abstract  = 1 << 6;
        /// `declare` modifier.
        const Ambient   = 1 << 7;
        /// `static` modifier.
        const Static    = 1 << 8;
        /// `accessor` modifier.
        const Accessor  = 1 << 9;
        /// `async` modifier.
        const Async     = 1 << 10;
        /// `default` modifier (export default).
        const Default   = 1 << 11;
        /// `const` modifier (const enum).
        const Const     = 1 << 12;
        /// `in` modifier (contravariance).
        const In        = 1 << 13;
        /// `out` modifier (covariance).
        const Out       = 1 << 14;
        /// Has a decorator.
        const Decorator = 1 << 15;
        /// `@deprecated` JSDoc tag.
        const Deprecated = 1 << 16;
        // JSDoc cache-only modifiers
        const JSDocPublic               = 1 << 23;
        const JSDocPrivate              = 1 << 24;
        const JSDocProtected            = 1 << 25;
        const JSDocReadonly             = 1 << 26;
        const JSDocOverride             = 1 << 27;
        const HasComputedJSDocModifiers = 1 << 28;
        const HasComputedFlags          = 1 << 29;
    }
}

impl ModifierFlags {
    /// Syntactic or JSDoc modifiers.
    pub const SyntacticOrJSDocModifiers: Self = Self::Public
        .union(Self::Private)
        .union(Self::Protected)
        .union(Self::Readonly)
        .union(Self::Override);

    /// Syntactic-only modifiers.
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

    /// All syntactic modifiers.
    pub const SyntacticModifiers: Self =
        Self::SyntacticOrJSDocModifiers.union(Self::SyntacticOnlyModifiers);

    /// Accessibility modifiers: `public | private | protected`.
    pub const AccessibilityModifier: Self =
        Self::Public.union(Self::Private).union(Self::Protected);

    /// Parameter property modifiers.
    pub const ParameterPropertyModifier: Self = Self::AccessibilityModifier
        .union(Self::Readonly)
        .union(Self::Override);

    /// Non-public accessibility modifiers.
    pub const NonPublicAccessibilityModifier: Self = Self::Private.union(Self::Protected);

    /// All modifiers except `Decorator`.
    pub const Modifier: Self = Self::All.difference(Self::Decorator);

    /// JavaScript-supported modifiers.
    pub const JavaScript: Self = Self::Export
        .union(Self::Static)
        .union(Self::Accessor)
        .union(Self::Async)
        .union(Self::Default);

    /// All modifiers.
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

    /// Export default flags.
    pub const ExportDefault: Self = Self::Export.union(Self::Default);
}
