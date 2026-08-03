//! Symbol display parts construction.
//!
//! Ported from `internal/ls/lsutil/symboldisplay.go`. The `ScriptElementKind`
//! and `ScriptElementKindModifier` enums (and their names table) are fully
//! ported. The `GetSymbolKind` / `GetSymbolModifiers` functions require the type
//! checker and AST accessors; their bodies are stubbed.

use std::sync::Arc;

use crate::ast::{Node, Symbol};
use crate::collections::set::Set;

// Placeholder for the checker type until `crate::checker::Checker` is wired up.
// Using `()` keeps signatures documenting the dependency without requiring it.
type Checker = ();

/// The kind of a script element, used for display in completion / nav lists.
///
/// Mirrors `ScriptElementKind` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum ScriptElementKind {
    #[default]
    Unknown = 0,
    Warning,
    /// predefined type (void) or keyword (class)
    Keyword,
    /// top level script node
    ScriptElement,
    /// module foo {}
    ModuleElement,
    /// class X {}
    ClassElement,
    /// var x = class X {}
    LocalClassElement,
    /// interface Y {}
    InterfaceElement,
    /// type T = ...
    TypeElement,
    /// enum E {}
    EnumElement,
    EnumMemberElement,
    /// Inside module and script only. const v = ...
    VariableElement,
    /// Inside function.
    LocalVariableElement,
    /// using foo = ...
    VariableUsingElement,
    /// await using foo = ...
    VariableAwaitUsingElement,
    /// Inside module and script only. function f() {}
    FunctionElement,
    /// Inside function.
    LocalFunctionElement,
    /// class X { [public|private]* foo() {} }
    MemberFunctionElement,
    /// class X { [public|private]* [get|set] foo:number; }
    MemberGetAccessorElement,
    MemberSetAccessorElement,
    /// class X { [public|private]* foo:number; }
    /// interface Y { foo:number; }
    MemberVariableElement,
    /// class X { [public|private]* accessor foo: number; }
    MemberAccessorVariableElement,
    /// class X { constructor() { } }
    /// class X { static { } }
    ConstructorImplementationElement,
    /// interface Y { ():number; }
    CallSignatureElement,
    /// interface Y { []:number; }
    IndexSignatureElement,
    /// interface Y { new():Y; }
    ConstructSignatureElement,
    /// function foo(*Y*: string)
    ParameterElement,
    TypeParameterElement,
    PrimitiveType,
    Label,
    Alias,
    ConstElement,
    LetElement,
    Directory,
    ExternalModuleName,
    /// String literal
    String,
    /// Jsdoc @link: in `{@link C link text}`, the before/after text
    Link,
    /// Jsdoc @link: in `{@link C link text}`, the entity name
    LinkName,
    /// Jsdoc @link: in `{@link C link text}`, the link text
    LinkText,
}

/// Bitflags describing a script element's modifiers.
///
/// Mirrors `ScriptElementKindModifier` in Go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScriptElementKindModifier(pub u32);

impl ScriptElementKindModifier {
    pub const NONE: Self = Self(0);
    pub const PUBLIC: Self = Self(1 << 0);
    pub const PRIVATE: Self = Self(1 << 1);
    pub const PROTECTED: Self = Self(1 << 2);
    pub const EXPORTED: Self = Self(1 << 3);
    pub const AMBIENT: Self = Self(1 << 4);
    pub const STATIC: Self = Self(1 << 5);
    pub const ABSTRACT: Self = Self(1 << 6);
    pub const OPTIONAL: Self = Self(1 << 7);
    pub const DEPRECATED: Self = Self(1 << 8);
    pub const DTS: Self = Self(1 << 9);
    pub const TS: Self = Self(1 << 10);
    pub const TSX: Self = Self(1 << 11);
    pub const JS: Self = Self(1 << 12);
    pub const JSX: Self = Self(1 << 13);
    pub const JSON: Self = Self(1 << 14);
    pub const DMTS: Self = Self(1 << 15);
    pub const MTS: Self = Self(1 << 16);
    pub const MJS: Self = Self(1 << 17);
    pub const DCTS: Self = Self(1 << 18);
    pub const CTS: Self = Self(1 << 19);
    pub const CJS: Self = Self(1 << 20);

    /// Returns the set of modifier-name strings for this value.
    ///
    /// Mirrors `ScriptElementKindModifier.Strings` in Go.
    pub fn strings(self) -> Set<String> {
        let mut result = Set::new();
        for (flag, name) in SCRIPT_ELEMENT_KIND_MODIFIER_NAMES {
            if (self.0 & flag) != 0 {
                result.add((*name).to_string());
            }
        }
        result
    }
}

impl std::ops::BitOr for ScriptElementKindModifier {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for ScriptElementKindModifier {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd<u32> for ScriptElementKindModifier {
    type Output = u32;
    fn bitand(self, rhs: u32) -> u32 {
        self.0 & rhs
    }
}

/// The `(flag, name)` table used by [`ScriptElementKindModifier::strings`].
const SCRIPT_ELEMENT_KIND_MODIFIER_NAMES: &[(u32, &str)] = &[
    (ScriptElementKindModifier::PUBLIC.0, "public"),
    (ScriptElementKindModifier::PRIVATE.0, "private"),
    (ScriptElementKindModifier::PROTECTED.0, "protected"),
    (ScriptElementKindModifier::EXPORTED.0, "export"),
    (ScriptElementKindModifier::AMBIENT.0, "declare"),
    (ScriptElementKindModifier::STATIC.0, "static"),
    (ScriptElementKindModifier::ABSTRACT.0, "abstract"),
    (ScriptElementKindModifier::OPTIONAL.0, "optional"),
    (ScriptElementKindModifier::DEPRECATED.0, "deprecated"),
    (ScriptElementKindModifier::DTS.0, ".d.ts"),
    (ScriptElementKindModifier::TS.0, ".ts"),
    (ScriptElementKindModifier::TSX.0, ".tsx"),
    (ScriptElementKindModifier::JS.0, ".js"),
    (ScriptElementKindModifier::JSX.0, ".jsx"),
    (ScriptElementKindModifier::JSON.0, ".json"),
    (ScriptElementKindModifier::DMTS.0, ".d.mts"),
    (ScriptElementKindModifier::MTS.0, ".mts"),
    (ScriptElementKindModifier::MJS.0, ".mjs"),
    (ScriptElementKindModifier::DCTS.0, ".d.cts"),
    (ScriptElementKindModifier::CTS.0, ".cts"),
    (ScriptElementKindModifier::CJS.0, ".cjs"),
];

/// The OR of all file-extension modifiers.
///
/// Mirrors `FileExtensionKindModifiers` in Go.
pub const FILE_EXTENSION_KIND_MODIFIERS: ScriptElementKindModifier = ScriptElementKindModifier(
    ScriptElementKindModifier::DTS.0
        | ScriptElementKindModifier::TS.0
        | ScriptElementKindModifier::TSX.0
        | ScriptElementKindModifier::JS.0
        | ScriptElementKindModifier::JSX.0
        | ScriptElementKindModifier::JSON.0
        | ScriptElementKindModifier::DMTS.0
        | ScriptElementKindModifier::MTS.0
        | ScriptElementKindModifier::MJS.0
        | ScriptElementKindModifier::DCTS.0
        | ScriptElementKindModifier::CTS.0
        | ScriptElementKindModifier::CJS.0,
);

/// Returns the [`ScriptElementKind`] for a symbol.
///
/// Mirrors `GetSymbolKind` in Go. Requires the type checker; stubbed.
pub fn get_symbol_kind(
    _type_checker: Option<&Checker>,
    _symbol: &Symbol,
    _location: &Arc<Node>,
) -> ScriptElementKind {
    // TODO: port getSymbolKindOfConstructorPropertyMethodAccessorFunctionOrVar +
    // the combined-flags switch.
    ScriptElementKind::Unknown
}

/// Returns the modifiers for a symbol.
///
/// Mirrors `GetSymbolModifiers` in Go. Requires the type checker; stubbed.
pub fn get_symbol_modifiers(
    _type_checker: Option<&Checker>,
    _symbol: Option<&Symbol>,
) -> ScriptElementKindModifier {
    // TODO: port getNormalizedSymbolModifiers + getNodeModifiers.
    ScriptElementKindModifier::NONE
}
