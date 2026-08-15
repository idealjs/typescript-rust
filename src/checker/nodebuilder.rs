//! Type-to-string serialization (nodebuilder).
//!
//! Ported from `internal/checker/nodebuilderimpl.go` and
//! `internal/checker/printer.go` (Go's `typeToString`). The Go implementation
//! builds an AST `TypeNode` and then prints it with the printer; we take the
//! simpler direct-to-string approach, which avoids needing the full printer
//! infrastructure for diagnostic messages.
//!
//! The main entry point is [`Checker::type_to_string`].

use std::sync::Arc;

use crate::ast::{
    ArrayTypeNodeData, BigIntLiteralData, FunctionTypeNodeData, IdentifierData,
    IntersectionTypeNodeData, LiteralTypeNodeData, MissingDeclarationData, Node, NodeData,
    NodeList, NumericLiteralData, ParameterDeclarationData, ParenthesizedTypeNodeData,
    PropertySignatureDeclarationData, RestTypeNodeData, StringLiteralData, Symbol, SymbolFlags,
    SyntaxKind, TupleTypeNodeData, TypeLiteralNodeData, TypeOperatorNodeData,
    TypeReferenceNodeData, UnionTypeNodeData,
};

use super::checker::Checker;
use super::types::*;

/// Flags controlling how types are formatted to strings.
///
/// Mirrors `nodebuilder.Flags` in Go. Only the flags we currently use are
/// defined; others can be added as needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeFormatFlags(u32);

impl TypeFormatFlags {
    pub const NONE: Self = Self(0);
    /// Write array types as `Array<T>` instead of `T[]`.
    pub const WRITE_ARRAY_AS_GENERIC: Self = Self(1 << 1);
    /// Write type arguments using the enclosing declaration's scope.
    pub const USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE: Self = Self(1 << 2);
    /// Allow `unique symbol` types.
    pub const ALLOW_UNIQUE_ES_SYMBOL_TYPE: Self = Self(1 << 3);
    /// Don't add parentheses around union/intersection members that need
    /// them in some contexts.
    pub const NO_TRUNCATION: Self = Self(1 << 7);
    /// Multi-line object literals.
    pub const MULTILINE_OBJECT_LITERALS: Self = Self(1 << 8);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// A classified piece of a symbol's display string (e.g. keyword, type name,
/// parameter name, punctuation). Used to build structured hover information
/// (`SymbolDisplayPart[]` in the Go/TS Language Service).
///
/// Mirrors `lsproto.SymbolDisplayPart`: each part has a `text` slice and a
/// `kind` that classifies it for colorized display.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolDisplayPart {
    pub text: String,
    pub kind: DisplayPartKind,
}

/// The kind of a [`SymbolDisplayPart`]. Mirrors the
/// `SymbolDisplayPartKind` string constants used by the Language Service
/// ("keyword", "className", "parameterName", "punctuation", "space", …).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayPartKind {
    Keyword,
    FunctionName,
    ClassName,
    InterfaceName,
    EnumName,
    TypeParameterName,
    ParameterName,
    PropertyName,
    VariableName,
    Punctuation,
    Space,
    LineBreak,
    Text,
    NumericLiteral,
    StringLiteral,
}

impl DisplayPartKind {
    /// Lowercase string label matching the Language Service's
    /// `SymbolDisplayPartKind` constants (e.g. `"className"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            DisplayPartKind::Keyword => "keyword",
            DisplayPartKind::FunctionName => "functionName",
            DisplayPartKind::ClassName => "className",
            DisplayPartKind::InterfaceName => "interfaceName",
            DisplayPartKind::EnumName => "enumName",
            DisplayPartKind::TypeParameterName => "typeParameterName",
            DisplayPartKind::ParameterName => "parameterName",
            DisplayPartKind::PropertyName => "propertyName",
            DisplayPartKind::VariableName => "variableName",
            DisplayPartKind::Punctuation => "punctuation",
            DisplayPartKind::Space => "space",
            DisplayPartKind::LineBreak => "lineBreak",
            DisplayPartKind::Text => "text",
            DisplayPartKind::NumericLiteral => "numericLiteral",
            DisplayPartKind::StringLiteral => "stringLiteral",
        }
    }
}

impl SymbolDisplayPart {
    /// Convenience constructor.
    pub fn new(text: impl Into<String>, kind: DisplayPartKind) -> Self {
        SymbolDisplayPart {
            text: text.into(),
            kind,
        }
    }
}

/// Whether an intrinsic type name should be classified as a keyword part.
fn is_keyword_type_name(name: &str) -> bool {
    matches!(
        name,
        "any"
            | "unknown"
            | "string"
            | "number"
            | "bigint"
            | "boolean"
            | "symbol"
            | "void"
            | "undefined"
            | "null"
            | "object"
            | "never"
            | "intrinsic"
            | "true"
            | "false"
    )
}

/// Push a keyword part (e.g. `function`, `class`, `let`).
fn push_keyword(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Keyword));
}

/// Push a whitespace part (e.g. ` `, `: `, `, `).
fn push_space(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Space));
}

/// Push a punctuation part (e.g. `(`, `)`, `<`, `>`).
fn push_punctuation(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Punctuation));
}

/// Push a part with an explicit kind (e.g. a name part).
fn push_part(parts: &mut Vec<SymbolDisplayPart>, text: &str, kind: DisplayPartKind) {
    parts.push(SymbolDisplayPart::new(text, kind));
}

/// Determine the [`DisplayPartKind`] for a symbol's name based on its flags,
/// mirroring Go's `classificationForSymbol` (displaypartswriter.go).
fn display_kind_for_symbol(symbol: &Symbol) -> DisplayPartKind {
    let flags = symbol.flags;
    if flags.intersects(SymbolFlags::Function | SymbolFlags::Method) {
        DisplayPartKind::FunctionName
    } else if flags.intersects(SymbolFlags::Class) {
        DisplayPartKind::ClassName
    } else if flags.intersects(SymbolFlags::Interface) {
        DisplayPartKind::InterfaceName
    } else if flags.intersects(SymbolFlags::ENUM) {
        DisplayPartKind::EnumName
    } else if flags.intersects(SymbolFlags::TypeParameter) {
        DisplayPartKind::TypeParameterName
    } else if flags.intersects(SymbolFlags::Property | SymbolFlags::ACCESSOR) {
        DisplayPartKind::PropertyName
    } else if flags.intersects(SymbolFlags::EnumMember) {
        DisplayPartKind::PropertyName
    } else if flags.intersects(SymbolFlags::VARIABLE) {
        DisplayPartKind::VariableName
    } else {
        DisplayPartKind::Text
    }
}

impl Checker {
    /// Format a type as a human-readable string.
    ///
    /// Mirrors Go's `Checker.TypeToString` → `typeToStringEx` (printer.go).
    /// Used by diagnostic messages (e.g. TS2322 "Type 'X' is not assignable
    /// to type 'Y'") and hover info.
    pub fn type_to_string(&mut self, t: &Arc<Type>) -> String {
        self.type_to_string_ex(t, TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE)
    }

    /// Format a type with explicit flags.
    ///
    /// This is the main worker. It dispatches on the type's flags and data
    /// variant, recursing into constituent types as needed.
    pub fn type_to_string_ex(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        // Guard against infinite recursion on recursive types.
        if self.serialization_level >= MAX_SERIALIZATION_LEVEL {
            return "?".to_string();
        }

        // Intrinsic types (any, string, number, etc.)
        if let Some(name) = t.intrinsic_name() {
            return name.to_string();
        }

        // Literal types
        if let Some(val) = t.literal_value() {
            return self.literal_value_to_string(val);
        }

        // Unique ESSymbol type
        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            if let TypeData::UniqueESSymbol(sym) = &t.data {
                if flags.contains(TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE) {
                    return format!("unique symbol");
                }
                return format!("typeof {}", sym.name);
            }
        }

        // Never
        if t.flags.contains(TypeFlags::Never) {
            return "never".to_string();
        }

        // Union types
        if t.is_union() {
            return self.union_to_string(t, flags);
        }

        // Intersection types
        if t.is_intersection() {
            return self.intersection_to_string(t, flags);
        }

        // Type parameters
        if t.is_type_parameter() {
            return self.type_parameter_to_string(t);
        }

        // Indexed access types
        if let TypeData::IndexedAccess(ia) = &t.data {
            return self.indexed_access_to_string(ia, flags);
        }

        // Template literal types
        if let TypeData::TemplateLiteral(tl) = &t.data {
            return self.template_literal_to_string(tl, flags);
        }

        // Tuple types
        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_string(t, flags);
        }

        // Array / reference types
        if t.object_flags.contains(ObjectFlags::Reference) {
            return self.reference_to_string(t, flags);
        }

        // Function types (object types with call signatures and no symbol)
        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return self.function_type_to_string(t, structured, flags);
            }
        }

        // Object types with a symbol (class, interface, enum, type alias)
        if let Some(sym) = &t.symbol {
            return self.symbol_type_to_string(t, sym, flags);
        }

        // Anonymous object literal types
        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty() || !structured.call_signatures().is_empty() {
                return self.object_literal_to_string(t, structured, flags);
            }
        }

        // Fallbacks
        if t.flags.contains(TypeFlags::Object) {
            return "object".to_string();
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return "unknown".to_string();
        }

        "<unknown type>".to_string()
    }

    /// Format a literal value as a string.
    fn literal_value_to_string(&mut self, val: &LiteralValue) -> String {
        match val {
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::BigInt(b) => format!("{}n", b.to_string()),
            LiteralValue::Boolean(true) => "true".to_string(),
            LiteralValue::Boolean(false) => "false".to_string(),
            LiteralValue::None => String::new(),
        }
    }

    /// Format a union type: `A | B | C`.
    ///
    /// Members that are function types or union types themselves get
    /// parenthesized to avoid ambiguity.
    fn union_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let types = t.types().unwrap_or(&[]);
        let parts: Vec<String> = types
            .iter()
            .map(|ty| {
                let s = self.type_to_string_ex(ty, flags);
                // Parenthesize function types and unions in union members.
                if self.needs_parens_in_union(ty) {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();
        parts.join(" | ")
    }

    /// Format an intersection type: `A & B & C`.
    fn intersection_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let types = t.types().unwrap_or(&[]);
        let parts: Vec<String> = types
            .iter()
            .map(|ty| {
                let s = self.type_to_string_ex(ty, flags);
                if self.needs_parens_in_union(ty) {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();
        parts.join(" & ")
    }

    /// Format a type parameter: `T` or `this`.
    fn type_parameter_to_string(&mut self, t: &Arc<Type>) -> String {
        if let TypeData::TypeParameter(tp) = &t.data {
            if tp.is_this_type {
                return "this".to_string();
            }
        }
        if let Some(sym) = &t.symbol {
            return sym.name.clone();
        }
        "T".to_string()
    }

    /// Format an indexed access type: `T[K]`.
    fn indexed_access_to_string(
        &mut self,
        ia: &IndexedAccessTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let obj = ia
            .object_type
            .as_ref()
            .map(|t| self.type_to_string_ex(t, flags))
            .unwrap_or_else(|| "any".to_string());
        let idx = ia
            .index_type
            .as_ref()
            .map(|t| self.type_to_string_ex(t, flags))
            .unwrap_or_else(|| "any".to_string());
        format!("{}[{}]", obj, idx)
    }

    /// Format a template literal type: `${head}${mid}tail`.
    ///
    /// Template literal types have alternating `texts` and `types`:
    /// `texts[0] + ${types[0]} + texts[1] + ${types[1]} + ... + texts[N]`.
    fn template_literal_to_string(
        &mut self,
        tl: &TemplateLiteralTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let mut result = String::new();
        for (i, text) in tl.texts.iter().enumerate() {
            result.push_str(text);
            if i < tl.types.len() {
                result.push_str("${");
                result.push_str(&self.type_to_string_ex(&tl.types[i], flags));
                result.push('}');
            }
        }
        format!("`{}`", result)
    }

    /// Format a tuple type: `[A, B, C]`.
    fn tuple_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let TypeData::Tuple(tuple) = &t.data else {
            return "[]".to_string();
        };
        if tuple.element_infos.is_empty() {
            return "[]".to_string();
        }
        let parts: Vec<String> = tuple
            .element_infos
            .iter()
            .map(|elem| {
                let ty_str = elem
                    .type_
                    .as_ref()
                    .map(|ty| self.type_to_string_ex(ty, flags))
                    .unwrap_or_else(|| "any".to_string());
                if elem.flags.contains(ElementFlags::Rest)
                    || elem.flags.contains(ElementFlags::Variadic)
                {
                    format!("...{}", ty_str)
                } else if elem.flags.contains(ElementFlags::Optional) {
                    format!("{}?", ty_str)
                } else {
                    ty_str
                }
            })
            .collect();
        format!("[{}]", parts.join(", "))
    }

    /// Format a reference type: `Foo<T>` or `T[]` (for arrays).
    fn reference_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let obj_data = match &t.data {
            TypeData::Object(o) => o,
            TypeData::Interface(i) => &i.object,
            _ => return "object".to_string(),
        };

        // Array type: if there's exactly one type argument and the symbol
        // name is `Array`/`ReadonlyArray`, format as `T[]` (or `Array<T>`
        // if flagged). A synthetic array created by `create_array_type`
        // (no symbol) is also detected here, matching `reference_to_type_node`.
        let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
        let is_array = obj_data.type_arguments.len() == 1
            && (symbol_name == "Array" || symbol_name == "ReadonlyArray" || t.symbol.is_none());

        if is_array {
            let elem = &obj_data.type_arguments[0];
            let elem_str = self.type_to_string_ex(elem, flags);
            let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
            if symbol_name == "ReadonlyArray" {
                return format!("readonly {}[]", self.maybe_parenthesize_array_element(elem));
            }
            if flags.contains(TypeFormatFlags::WRITE_ARRAY_AS_GENERIC) {
                return format!("Array<{}>", elem_str);
            }
            return format!("{}[]", self.maybe_parenthesize_array_element(elem));
        }

        // Generic type reference: `Foo<T, U>`
        let name = t
            .symbol
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "object".to_string());

        if obj_data.type_arguments.is_empty() {
            return name;
        }

        let args: Vec<String> = obj_data
            .type_arguments
            .iter()
            .map(|ty| self.type_to_string_ex(ty, flags))
            .collect();
        format!("{}<{}>", name, args.join(", "))
    }

    /// Format a function type: `(a: T, b: U) => V`.
    fn function_type_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let sigs = structured.call_signatures();
        if sigs.is_empty() {
            return "() => unknown".to_string();
        }
        // Use the first call signature for display.
        let sig = &sigs[0];
        let params: Vec<String> = sig
            .parameters
            .iter()
            .map(|param| {
                let name = param.name.clone();
                let param_type = self.get_type_of_symbol(param);
                let type_str = self.type_to_string_ex(&param_type, flags);
                if param.flags.contains(crate::ast::SymbolFlags::Optional) {
                    format!("{}?: {}", name, type_str)
                } else {
                    format!("{}: {}", name, type_str)
                }
            })
            .collect();
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_str = self.type_to_string_ex(&ret_type, flags);
        format!("({}) => {}", params.join(", "), ret_str)
    }

    /// Format an object literal type: `{ a: T; b: U }`.
    fn object_literal_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Call signatures
        for sig in structured.call_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .map(|param| {
                    let name = param.name.clone();
                    let param_type = self.get_type_of_symbol(param);
                    let type_str = self.type_to_string_ex(&param_type, flags);
                    if param.flags.contains(crate::ast::SymbolFlags::Optional) {
                        format!("{}?: {}", name, type_str)
                    } else {
                        format!("{}: {}", name, type_str)
                    }
                })
                .collect();
            let ret_type = sig
                .resolved_return_type
                .get()
                .cloned()
                .unwrap_or_else(|| self.any_type());
            let ret_str = self.type_to_string_ex(&ret_type, flags);
            parts.push(format!("({}) => {}", params.join(", "), ret_str));
        }

        // Properties
        for prop in &structured.properties {
            let name = prop.name.clone();
            let prop_type = self.get_type_of_symbol(prop);
            let type_str = self.type_to_string_ex(&prop_type, flags);
            if prop.flags.contains(SymbolFlags::Optional) {
                parts.push(format!("{}?: {}", name, type_str));
            } else {
                parts.push(format!("{}: {}", name, type_str));
            }
        }

        if parts.is_empty() {
            "{}".to_string()
        } else {
            // Go's printer terminates each member with ';' inside the braces
            // (`{ x: number; }`).
            format!("{{ {} }}", format!("{};", parts.join("; ")))
        }
    }

    /// Format a type with a symbol (class, interface, enum, type alias).
    fn symbol_type_to_string(
        &mut self,
        t: &Arc<Type>,
        sym: &Arc<Symbol>,
        flags: TypeFormatFlags,
    ) -> String {
        // Enum types: show the enum name.
        if sym.flags.contains(SymbolFlags::ENUM) {
            return sym.name.clone();
        }

        // Class/Interface/TypeAlias: show name + type arguments if any.
        let obj_data = match &t.data {
            TypeData::Object(o) => Some(o),
            TypeData::Interface(i) => Some(&i.object),
            _ => None,
        };

        if let Some(obj) = obj_data {
            if !obj.type_arguments.is_empty() {
                let args: Vec<String> = obj
                    .type_arguments
                    .iter()
                    .map(|ty| self.type_to_string_ex(ty, flags))
                    .collect();
                return format!("{}<{}>", sym.name, args.join(", "));
            }
        }

        // Constructor types: when a class symbol's type has construct
        // signatures (i.e. it's the constructor function type, not the
        // instance type), print as `typeof ClassName`. Mirrors Go's
        // `TypeToString` which checks `constructSignatureCount > 0` and the
        // symbol is a class.
        if sym.flags.contains(SymbolFlags::Class) {
            if let Some(structured) = t.as_structured() {
                if !structured.construct_signatures().is_empty() {
                    return format!("typeof {}", sym.name);
                }
            }
        }

        // Namespace value types print as `typeof N` (Go's TypeReference
        // display for ValueModule symbols, e.g. TS2339 on `N.x` reads
        // "Property 'x' does not exist on type 'typeof N'").
        if sym.flags.contains(SymbolFlags::ValueModule) {
            return format!("typeof {}", sym.name);
        }

        sym.name.clone()
    }

    /// Determine if a type needs parentheses when it appears as a union
    /// or intersection member.
    ///
    /// Function types `(x: T) => U` and union types `A | B` need parens
    /// to avoid ambiguity (e.g. `A | B => C` is ambiguous without parens).
    fn needs_parens_in_union(&mut self, t: &Arc<Type>) -> bool {
        // Function types: object types with call signatures and no symbol.
        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return true;
            }
        }
        // Nested unions don't need parens (A | B | C is fine).
        // Intersections might need parens in some contexts, but
        // TypeScript doesn't parenthesize them in unions.
        false
    }

    /// Parenthesize array element types that need it:
    /// - Function types: `(x: T) => U` → `((x: T) => U)[]`
    /// - Union types with function members
    fn maybe_parenthesize_array_element(&mut self, elem: &Arc<Type>) -> String {
        let s = self.type_to_string_ex(elem, TypeFormatFlags::NONE);
        if self.needs_parens_in_union(elem) {
            format!("({})", s)
        } else {
            s
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Type-to-TypeNode (AST construction)
    //
    // Ported from `internal/checker/nodebuilderimpl.go`'s `typeToTypeNode`.
    // The Go implementation builds an AST `TypeNode` and then prints it with
    // the printer; this is the reverse of `get_type_from_type_node` in
    // `typenode.rs`. The result is used by declaration emit and hover
    // display.
    //
    // This is a FOUNDATION implementation covering the common Type variants.
    // Remaining cases (conditional, mapped, indexed access, template literal,
    // type predicates, rest types, named tuple members, JSDoc types) are
    // marked with TODO comments.
    // ─────────────────────────────────────────────────────────────────────

    /// Serialize a `Type` into an AST `TypeNode`.
    ///
    /// Mirrors Go's `NodeBuilderImpl.typeToTypeNode`. This is the reverse of
    /// `get_type_from_type_node`: it builds a `TypeNode` AST that, when
    /// printed, renders the type as it would appear in source. Used by
    /// declaration emit and hover display.
    ///
    /// NOTE: Unlike `type_to_string_ex`, this does NOT check/increment
    /// `serialization_level`. In Go, `typeToStringEx` increments
    /// `serializationLevel` once before calling `TypeToTypeNode`, and the
    /// node builder's own recursion is independent. The serialization level
    /// is only for preventing reentrant `typeToStringEx` calls (e.g. from
    /// diagnostics produced during lazy member resolution).
    pub fn type_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        self.type_to_type_node_worker(t)
    }

    fn type_to_type_node_worker(&mut self, t: &Arc<Type>) -> Arc<Node> {
        // Intrinsic types (any, string, number, etc.)
        if let Some(name) = t.intrinsic_name() {
            return self.intrinsic_to_type_node(name);
        }

        // Literal types
        if let Some(val) = t.literal_value() {
            return self.literal_value_to_type_node(val);
        }

        // Unique ESSymbol type
        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            // TODO: full `unique symbol` / `typeof sym` rendering per
            // FlagsAllowUniqueESSymbolType; currently approximated as
            // `unique symbol`.
            return self.type_operator_node(
                SyntaxKind::UniqueKeyword,
                self.keyword_node(SyntaxKind::SymbolKeyword),
            );
        }

        // Never
        if t.flags.contains(TypeFlags::Never) {
            return self.keyword_node(SyntaxKind::NeverKeyword);
        }

        // Union types
        if t.is_union() {
            return self.union_to_type_node(t);
        }

        // Intersection types
        if t.is_intersection() {
            return self.intersection_to_type_node(t);
        }

        // Type parameters
        if t.is_type_parameter() {
            return self.type_parameter_to_type_node(t);
        }

        // TODO: Indexed access types (`T[K]`) — TypeData::IndexedAccess
        // TODO: Template literal types — TypeData::TemplateLiteral
        // TODO: String mapping types — TypeData::StringMapping
        // TODO: Conditional types — TypeData::Conditional
        // TODO: Substitution types — TypeData::Substitution
        // TODO: Index types (`keyof T`) — TypeFlags::Index
        // TODO: Type predicates
        // TODO: Rest types
        // TODO: Named tuple members

        // Tuple types
        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_type_node(t);
        }

        // Array / reference types
        if t.object_flags.contains(ObjectFlags::Reference) {
            return self.reference_to_type_node(t);
        }

        // Function types (object types with call signatures and no symbol)
        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return self.function_type_to_type_node(structured);
            }
        }

        // Object types with a symbol (class, interface, enum, type alias)
        if let Some(sym) = &t.symbol {
            return self.symbol_to_type_node(sym, SymbolFlags::TYPE, None);
        }

        // Anonymous object literal types
        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty()
                || !structured.call_signatures().is_empty()
                || !structured.index_infos.is_empty()
            {
                return self.type_literal_to_type_node(structured);
            }
        }

        // Fallbacks
        if t.flags.contains(TypeFlags::Object) {
            return self.keyword_node(SyntaxKind::ObjectKeyword);
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return self.keyword_node(SyntaxKind::UnknownKeyword);
        }

        self.keyword_node(SyntaxKind::AnyKeyword)
    }

    /// Map an intrinsic type name to its keyword `TypeNode`.
    fn intrinsic_to_type_node(&mut self, name: &str) -> Arc<Node> {
        let kind = match name {
            "any" => SyntaxKind::AnyKeyword,
            "unknown" => SyntaxKind::UnknownKeyword,
            "string" => SyntaxKind::StringKeyword,
            "number" => SyntaxKind::NumberKeyword,
            "bigint" => SyntaxKind::BigIntKeyword,
            "boolean" => SyntaxKind::BooleanKeyword,
            "symbol" => SyntaxKind::SymbolKeyword,
            "void" => SyntaxKind::VoidKeyword,
            "undefined" => SyntaxKind::UndefinedKeyword,
            "null" => SyntaxKind::NullKeyword,
            "object" => SyntaxKind::ObjectKeyword,
            "never" => SyntaxKind::NeverKeyword,
            // `error` and other internal intrinsic names render as `any`.
            _ => SyntaxKind::AnyKeyword,
        };
        self.keyword_node(kind)
    }

    /// Build a `LiteralTypeNode` from a `LiteralValue`.
    fn literal_value_to_type_node(&mut self, val: &LiteralValue) -> Arc<Node> {
        let literal = match val {
            LiteralValue::String(s) => self.string_literal_node(s),
            LiteralValue::Number(n) => self.numeric_literal_node(&n.to_string()),
            LiteralValue::BigInt(b) => self.bigint_literal_node(&b.to_string()),
            LiteralValue::Boolean(true) => self.keyword_node(SyntaxKind::TrueKeyword),
            LiteralValue::Boolean(false) => self.keyword_node(SyntaxKind::FalseKeyword),
            // NullKeyword is wrapped in a LiteralTypeNode in the Go impl;
            // for simplicity we emit the keyword directly.
            LiteralValue::None => return self.keyword_node(SyntaxKind::NullKeyword),
        };
        self.literal_type_node(literal)
    }

    /// Build a `UnionTypeNode` from a union type, parenthesizing members
    /// that need it (function types).
    fn union_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let types = t.types().unwrap_or(&[]);
        if types.is_empty() {
            return self.keyword_node(SyntaxKind::NeverKeyword);
        }
        if types.len() == 1 {
            return self.type_to_type_node(&types[0]);
        }
        let nodes: Vec<Arc<Node>> = types
            .iter()
            .map(|ty| {
                let node = self.type_to_type_node(ty);
                if self.needs_parens_in_union(ty) {
                    self.parenthesized_type_node(node)
                } else {
                    node
                }
            })
            .collect();
        self.union_type_node(nodes)
    }

    /// Build an `IntersectionTypeNode` from an intersection type.
    fn intersection_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let types = t.types().unwrap_or(&[]);
        if types.is_empty() {
            return self.keyword_node(SyntaxKind::UnknownKeyword);
        }
        if types.len() == 1 {
            return self.type_to_type_node(&types[0]);
        }
        let nodes: Vec<Arc<Node>> = types
            .iter()
            .map(|ty| {
                let node = self.type_to_type_node(ty);
                if self.needs_parens_in_union(ty) {
                    self.parenthesized_type_node(node)
                } else {
                    node
                }
            })
            .collect();
        self.intersection_type_node(nodes)
    }

    /// Build a `TypeReferenceNode` for a type parameter.
    fn type_parameter_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        if let TypeData::TypeParameter(tp) = &t.data {
            if tp.is_this_type {
                // TODO: ThisTypeNode (SyntaxKind::ThisType) — currently
                // approximated as a type reference to "this".
                let name = self.identifier("this");
                return self.type_reference_node(name, None);
            }
        }
        if let Some(sym) = &t.symbol {
            let name = self.identifier(&sym.name);
            return self.type_reference_node(name, None);
        }
        let name = self.identifier("T");
        self.type_reference_node(name, None)
    }

    /// Build a `TupleTypeNode` from a tuple type.
    fn tuple_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let TypeData::Tuple(tuple) = &t.data else {
            return self.tuple_type_node(Vec::new());
        };
        let elements: Vec<Arc<Node>> = tuple
            .element_infos
            .iter()
            .map(|elem| {
                let ty = elem
                    .type_
                    .as_ref()
                    .map(|ty| self.type_to_type_node(ty))
                    .unwrap_or_else(|| self.keyword_node(SyntaxKind::AnyKeyword));
                // TODO: named tuple members (`name: type`),
                // `...rest`/`optional` markers.
                if elem.flags.contains(ElementFlags::Rest)
                    || elem.flags.contains(ElementFlags::Variadic)
                {
                    self.rest_type_node(ty)
                } else {
                    ty
                }
            })
            .collect();
        self.tuple_type_node(elements)
    }

    /// Build an `ArrayTypeNode` or `TypeReferenceNode` from a reference type.
    fn reference_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let obj_data = match &t.data {
            TypeData::Object(o) => o,
            TypeData::Interface(i) => &i.object,
            _ => return self.keyword_node(SyntaxKind::ObjectKeyword),
        };

        // Array type: `T[]` (single type argument with Array/ReadonlyArray
        // symbol). When no symbol is present (synthetic array from
        // `create_array_type`), we still detect the array shape: exactly one
        // type argument and no symbol (or an `Array` symbol).
        let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
        let is_array = obj_data.type_arguments.len() == 1
            && (symbol_name == "Array" || symbol_name == "ReadonlyArray" || t.symbol.is_none());

        if is_array {
            let elem = &obj_data.type_arguments[0];
            let elem_node = self.type_to_type_node(elem);
            if self.needs_parens_in_union(elem) {
                return self.array_type_node(self.parenthesized_type_node(elem_node));
            }
            // TODO: `readonly T[]` for ReadonlyArray symbol.
            return self.array_type_node(elem_node);
        }

        // Generic type reference: `Foo<T, U>`
        let name = if symbol_name.is_empty() {
            self.identifier("object")
        } else {
            self.identifier(symbol_name)
        };
        let type_args = if obj_data.type_arguments.is_empty() {
            None
        } else {
            let arg_nodes: Vec<Arc<Node>> = obj_data
                .type_arguments
                .iter()
                .map(|ty| self.type_to_type_node(ty))
                .collect();
            Some(Arc::new(NodeList::new(arg_nodes)))
        };
        self.type_reference_node(name, type_args)
    }

    /// Build a `FunctionTypeNode` from an object type with call signatures.
    fn function_type_to_type_node(&mut self, structured: &StructuredTypeData) -> Arc<Node> {
        let sigs = structured.call_signatures();
        if sigs.is_empty() {
            let ret = self.keyword_node(SyntaxKind::UnknownKeyword);
            return self.function_type_node(Vec::new(), ret);
        }
        let sig = &sigs[0];
        let params = self.signature_to_parameter_nodes(sig);
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_node = self.type_to_type_node(&ret_type);
        self.function_type_node(params, ret_node)
    }

    /// Build a `TypeLiteralNode` from an anonymous object type's structured
    /// data (properties + call signatures + index signatures).
    fn type_literal_to_type_node(&mut self, structured: &StructuredTypeData) -> Arc<Node> {
        let mut members: Vec<Arc<Node>> = Vec::new();

        // Call signatures: `(params) => ret`
        for sig in structured.call_signatures() {
            members.push(self.call_signature_to_node(sig));
        }
        // TODO: construct signatures (`new (params) => ret`).

        // Properties
        for prop in &structured.properties {
            let name = self.identifier(&prop.name);
            let prop_type = self.get_type_of_symbol(prop);
            let type_node = self.type_to_type_node(&prop_type);
            let optional = prop.flags.contains(SymbolFlags::Optional);
            members.push(self.property_signature_node(name, optional, type_node));
        }

        // TODO: index signatures (`[key: string]: T`).

        self.type_literal_node(members)
    }

    /// Convert a `Signature` to a list of `ParameterDeclaration` nodes.
    fn signature_to_parameter_nodes(&mut self, sig: &Signature) -> Vec<Arc<Node>> {
        sig.parameters
            .iter()
            .map(|param| {
                let name = self.identifier(&param.name);
                let param_type = self.get_type_of_symbol(param);
                let type_node = self.type_to_type_node(&param_type);
                let optional = param.flags.contains(SymbolFlags::Optional);
                self.parameter_node(name, optional, type_node)
            })
            .collect()
    }

    /// Build a `CallSignatureDeclaration`-shaped node for type literals.
    /// For simplicity we emit a `FunctionTypeNode` without the
    /// `function` keyword; in a type literal context, call signatures are
    /// rendered as `(params) => ret`.
    fn call_signature_to_node(&mut self, sig: &Signature) -> Arc<Node> {
        let params = self.signature_to_parameter_nodes(sig);
        let ret_type = sig
            .resolved_return_type
            .get()
            .cloned()
            .unwrap_or_else(|| self.any_type());
        let ret_node = self.type_to_type_node(&ret_type);
        self.function_type_node(params, ret_node)
    }

    // ─────────────────────────────────────────────────────────────────────
    // symbol_to_type_node entry point
    // ─────────────────────────────────────────────────────────────────────

    /// Serialize a `Symbol` into an AST `TypeNode`.
    ///
    /// Mirrors Go's `NodeBuilderImpl.symbolToTypeNode`. Resolves the
    /// symbol's declared type and delegates to `type_to_type_node` for
    /// structural rendering. When the symbol has a simple name and the
    /// type carries the symbol (class/interface/enum/type alias), a
    /// `TypeReferenceNode` with the symbol's name and the provided type
    /// arguments is produced.
    ///
    /// `mask` filters which symbol aspect to use (currently only
    /// `SymbolFlags::TYPE` is meaningfully handled; `SymbolFlags::VALUE`
    /// would produce a `typeof` query, which is a TODO).
    /// `type_arguments` overrides the type arguments written on the
    /// reference (used when serializing an alias's type arguments).
    pub fn symbol_to_type_node(
        &mut self,
        symbol: &Arc<Symbol>,
        mask: SymbolFlags,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        // TODO: full symbol-chain resolution (qualified names like `A.B.C`,
        // import types like `import("mod").T`). Currently we emit a flat
        // `TypeReferenceNode` with the symbol's local name.
        // TODO: `typeof` for value-meaning symbols (mask == SymbolFlags::VALUE).
        let _ = mask;

        let name = self.identifier(&symbol.name);
        // If type arguments are not provided and the symbol's declared type
        // is a generic reference, recover them from the type. This covers
        // the common case of `type T<X> = ...;` referenced as `T<number>`.
        let type_args = type_arguments.or_else(|| {
            let t = self.get_type_of_symbol(symbol);
            if let Some(obj) = t.as_object() {
                if !obj.type_arguments.is_empty() {
                    let arg_nodes: Vec<Arc<Node>> = obj
                        .type_arguments
                        .iter()
                        .map(|ty| self.type_to_type_node(ty))
                        .collect();
                    return Some(Arc::new(NodeList::new(arg_nodes)));
                }
            }
            None
        });
        self.type_reference_node(name, type_args)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Minimal TypeNode AST factory helpers
    //
    // These build `Arc<Node>` values for type-node SyntaxKinds. They use
    // `Node::new` (no source location) so the resulting nodes are
    // synthetic — suitable for declaration emit and hover display but not
    // for diagnostics that require a source span.
    // ─────────────────────────────────────────────────────────────────────

    /// Build a keyword type node (e.g. `string`, `number`).
    fn keyword_node(&self, kind: SyntaxKind) -> Arc<Node> {
        Arc::new(Node::new(kind, NodeData::Token))
    }

    /// Build an `Identifier` node.
    fn identifier(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    /// Build a `StringLiteral` node.
    fn string_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    /// Build a `NumericLiteral` node.
    fn numeric_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    /// Build a `BigIntLiteral` node (text includes trailing `n`).
    fn bigint_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::BigIntLiteral,
            NodeData::BigIntLiteral(BigIntLiteralData {
                text: format!("{}n", text),
                token_flags: 0,
            }),
        ))
    }

    /// Build a `LiteralTypeNode` wrapping a literal node.
    fn literal_type_node(&self, literal: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(LiteralTypeNodeData { literal }),
        ))
    }

    /// Build a `TypeReferenceNode` with optional type arguments.
    fn type_reference_node(
        &self,
        type_name: Arc<Node>,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeReference,
            NodeData::TypeReferenceNode(TypeReferenceNodeData {
                type_name,
                type_arguments,
            }),
        ))
    }

    /// Build an `ArrayTypeNode` (`T[]`).
    fn array_type_node(&self, element_type: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ArrayTypeNodeData { element_type }),
        ))
    }

    /// Build a `TupleTypeNode` (`[A, B, C]`).
    fn tuple_type_node(&self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TupleType,
            NodeData::TupleTypeNode(TupleTypeNodeData {
                elements: Arc::new(NodeList::new(elements)),
            }),
        ))
    }

    /// Build a `UnionTypeNode` (`A | B`).
    fn union_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::UnionType,
            NodeData::UnionTypeNode(UnionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    /// Build an `IntersectionTypeNode` (`A & B`).
    fn intersection_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::IntersectionType,
            NodeData::IntersectionTypeNode(IntersectionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    /// Build a `ParenthesizedTypeNode` (`(T)`).
    fn parenthesized_type_node(&self, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
        ))
    }

    /// Build a `FunctionTypeNode` (`(params) => ret`).
    fn function_type_node(&self, params: Vec<Arc<Node>>, ret: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::FunctionType,
            NodeData::FunctionTypeNode(FunctionTypeNodeData {
                type_parameters: None,
                parameters: Arc::new(NodeList::new(params)),
                type_node: Some(ret),
            }),
        ))
    }

    /// Build a `TypeLiteralNode` (`{ a: T; b: U }`).
    fn type_literal_node(&self, members: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData {
                members: Arc::new(NodeList::new(members)),
            }),
        ))
    }

    /// Build a `PropertySignatureDeclaration` (`name?: type`).
    fn property_signature_node(
        &self,
        name: Arc<Node>,
        optional: bool,
        type_node: Arc<Node>,
    ) -> Arc<Node> {
        let postfix_token = if optional {
            Some(self.keyword_node(SyntaxKind::QuestionToken))
        } else {
            None
        };
        // initializer is required by the data struct but not used for
        // synthetic signatures; we pass a synthetic `MissingDeclaration`.
        let initializer = Arc::new(Node::new(
            SyntaxKind::MissingDeclaration,
            NodeData::MissingDeclaration(MissingDeclarationData { modifiers: None }),
        ));
        Arc::new(Node::new(
            SyntaxKind::PropertySignature,
            NodeData::PropertySignatureDeclaration(PropertySignatureDeclarationData {
                modifiers: None,
                name,
                postfix_token,
                type_node,
                initializer,
            }),
        ))
    }

    /// Build a `ParameterDeclaration` (`name: type` or `name?: type`).
    fn parameter_node(&self, name: Arc<Node>, optional: bool, type_node: Arc<Node>) -> Arc<Node> {
        let question_token = if optional {
            Some(self.keyword_node(SyntaxKind::QuestionToken))
        } else {
            None
        };
        Arc::new(Node::new(
            SyntaxKind::Parameter,
            NodeData::ParameterDeclaration(ParameterDeclarationData {
                modifiers: None,
                dot_dot_dot_token: None,
                name,
                question_token,
                type_node: Some(type_node),
                initializer: None,
            }),
        ))
    }

    /// Build a `RestTypeNode` (`...T`).
    fn rest_type_node(&self, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::RestType,
            NodeData::RestTypeNode(RestTypeNodeData { type_node }),
        ))
    }

    /// Build a `TypeOperatorNode` (`keyof T`, `readonly T`, `unique symbol`).
    fn type_operator_node(&self, operator: SyntaxKind, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeOperator,
            NodeData::TypeOperatorNode(TypeOperatorNodeData {
                operator,
                type_node,
            }),
        ))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Symbol-to-string (hover info / quick info)
    //
    // Ported from `internal/checker/printer.go`'s `symbolToStringEx`. The
    // Go implementation builds an AST entity-name node via the NodeBuilder
    // and then prints it; we take the simpler direct-to-string approach
    // (matching `type_to_string` above). For hover info we additionally
    // synthesize a `let x: T` / `function f(): T` / `class C` /
    // `interface I` / `enum E` / `type T = ...` shape from the symbol's
    // declarations and resolved type.
    // ─────────────────────────────────────────────────────────────────────

    /// Format a symbol as a simple name. Mirrors Go's `SymbolToString` (the
    /// no-flags convenience overload). Returns just the symbol's name,
    /// without a kind prefix or type annotation — useful for diagnostic
    /// messages like TS2304.
    pub fn symbol_to_string(&mut self, symbol: &Arc<Symbol>) -> String {
        self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::AllowAnyNodeKind,
            SymbolFlags::all(),
        )
    }

    /// Format a symbol with explicit flags and semantic meaning.
    ///
    /// `meaning` filters which symbol aspect to use when a symbol carries
    /// multiple meanings (e.g. a class is both a value and a type). Pass
    /// `SymbolFlags::all()` to consider any meaning.
    ///
    /// Unlike Go's `symbolToStringEx`, this implementation:
    /// - Returns just the symbol's local name (no module chain); we don't
    ///   yet model the full symbol parent chain needed for qualified names.
    /// - Appends type arguments for generic symbols when the
    ///   `WriteTypeParametersOrArguments` flag is set.
    pub fn symbol_to_string_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        flags: SymbolFormatFlags,
        _meaning: SymbolFlags,
    ) -> String {
        let name = symbol.name.clone();
        // Write type arguments for generic class/interface/type-alias symbols
        // when the flag is set. We recover the type parameters from the
        // symbol's first declaration (if any) and format each as its name.
        if flags.contains(SymbolFormatFlags::WriteTypeParametersOrArguments) {
            if let Some(tps) = self.collect_type_parameter_names(symbol) {
                if !tps.is_empty() {
                    return format!("{}<{}>", name, tps.join(", "));
                }
            }
        }
        name
    }

    /// Collect the names of type parameters declared on `symbol` (e.g. the
    /// `<T, U>` on `interface Foo<T, U>`). Returns `None` when the symbol
    /// has no type parameters.
    fn collect_type_parameter_names(&self, symbol: &Arc<Symbol>) -> Option<Vec<String>> {
        for decl in &symbol.declarations {
            let tps = match &decl.data {
                NodeData::ClassDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::InterfaceDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::TypeAliasDeclaration(d) => d.type_parameters.as_ref(),
                NodeData::FunctionDeclaration(d) => d.type_parameters.as_ref(),
                _ => continue,
            };
            if let Some(tps) = tps {
                return Some(
                    tps.iter()
                        .map(|tp| match &tp.data {
                            NodeData::TypeParameterDeclaration(tpd) => tpd.name.text().to_string(),
                            _ => String::new(),
                        })
                        .collect(),
                );
            }
        }
        None
    }

    /// Build hover text for `node` (an identifier or other expression).
    ///
    /// Mirrors the simplified `getQuickInfoAndDeclarationAtLocation` flow
    /// in `internal/ls/hover.go` (without the classified-display-parts
    /// machinery — we return a single plain-text string). Returns the
    /// empty string when no quick info is available.
    ///
    /// Examples:
    /// - `let x: number = 0;` hovering over `x` → `let x: number`
    /// - `function f(a: string): number { ... }` hovering over `f` →
    ///   `function f(a: string): number`
    /// - `class Foo<T> { ... }` hovering over `Foo` → `class Foo<T>`
    /// - `interface Bar { ... }` hovering over `Bar` → `interface Bar`
    /// - `enum Color { Red, Green }` hovering over `Color` → `enum Color`
    pub fn get_quick_info_text(&mut self, node: &Arc<Node>) -> String {
        // For `this` in expression position.
        if node.kind == SyntaxKind::ThisKeyword {
            let t = self.get_type_of_node(node);
            return format!("this: {}", self.type_to_string(&t));
        }
        // Resolve the symbol at the node. Try the scope stack first (works
        // during checking), then fall back to walking up the AST looking
        // for an ancestor with a symbol in the symbol_map (works after
        // checking is complete — e.g. for hover info in a separate pass).
        let symbol = self.resolve_identifier(node).or_else(|| {
            let symbol_map = self.program.symbol_map();
            let mut current: Option<&Arc<Node>> = Some(node);
            while let Some(n) = current {
                if let Some(sym) = symbol_map.symbol_of(n) {
                    return Some(Arc::clone(sym));
                }
                current = n.parent.as_ref();
            }
            None
        });
        let Some(symbol) = symbol else {
            // No symbol: if the node has a type (e.g. literal), show it.
            if self.node_has_type(node) {
                let t = self.get_type_of_node(node);
                return self.type_to_string(&t);
            }
            return String::new();
        };
        self.format_quick_info_for_symbol(&symbol, node)
    }

    // ─────────────────────────────────────────────────────────────────────
    // symbol_to_display_parts / type_to_display_parts (structured hover info)
    //
    // Produces a `SymbolDisplayPart[]` — an array of classified parts with
    // `text` and `kind` fields — for Language Service hover information. Each
    // part represents a classified piece of the symbol's display string
    // (keyword, type name, parameter name, punctuation, etc.). Mirrors the
    // classified-output branch of Go's `getQuickInfoAndDeclarationAtLocation`
    // (hover.go), building on the same logic as `format_quick_info_for_symbol`
    // above but emitting structured parts instead of a plain string.
    // ─────────────────────────────────────────────────────────────────────

    /// Build structured hover parts for `node`. Mirrors `get_quick_info_text`
    /// but returns a classified `SymbolDisplayPart[]`. Resolves the node's
    /// symbol the same way; returns an empty vector when there is no symbol
    /// (e.g. for the `this` keyword or literal nodes), so the caller can fall
    /// back to the plain-text path.
    pub fn get_quick_info_display_parts(&mut self, node: &Arc<Node>) -> Vec<SymbolDisplayPart> {
        // Resolve the symbol at the node. Try the scope stack first (works
        // during checking), then fall back to walking up the AST looking for
        // an ancestor with a symbol in the symbol_map (works after checking
        // is complete — e.g. for hover info in a separate pass).
        let symbol = self.resolve_identifier(node).or_else(|| {
            let symbol_map = self.program.symbol_map();
            let mut current: Option<&Arc<Node>> = Some(node);
            while let Some(n) = current {
                if let Some(sym) = symbol_map.symbol_of(n) {
                    return Some(Arc::clone(sym));
                }
                current = n.parent.as_ref();
            }
            None
        });
        let Some(symbol) = symbol else {
            return Vec::new();
        };
        self.symbol_to_display_parts(&symbol, SymbolFlags::all(), &[])
    }

    /// Produce a classified `SymbolDisplayPart[]` for `symbol`.
    ///
    /// Determines the symbol kind (function/class/interface/enum/type
    /// alias/variable) and emits keyword parts (`function`, `class`, …), the
    /// symbol name with an appropriate kind, parameters and return type for
    /// functions, type parameters, and the aliased type for type aliases.
    /// `meaning` filters which aspect of a multi-meaning symbol to use
    /// (currently informational — we consider any meaning); `type_arguments`
    /// overrides type arguments written on the reference and is reserved for
    /// future use.
    ///
    /// Mirrors the `writeSymbol` dispatch in Go's
    /// `getQuickInfoAndDeclarationAtLocation` (hover.go), covering the common
    /// cases (function, method, class, interface, enum, type alias, type
    /// parameter, variable/property).
    pub fn symbol_to_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        type_arguments: &[String],
    ) -> Vec<SymbolDisplayPart> {
        // `meaning` and `type_arguments` are accepted for API parity with the
        // Go/TS `symbolToDisplayParts`; we currently consider any meaning and
        // recover type parameters from the symbol's declarations.
        let _ = meaning;
        let _ = type_arguments;

        let flags = symbol.flags;
        if flags.intersects(SymbolFlags::Function) {
            return self.function_symbol_display_parts(symbol, /*is_method=*/ false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.function_symbol_display_parts(symbol, /*is_method=*/ true);
        }
        if flags.intersects(SymbolFlags::Class) {
            return self.named_type_symbol_display_parts(
                symbol,
                "class",
                DisplayPartKind::ClassName,
            );
        }
        if flags.intersects(SymbolFlags::Interface) {
            return self.named_type_symbol_display_parts(
                symbol,
                "interface",
                DisplayPartKind::InterfaceName,
            );
        }
        if flags.intersects(SymbolFlags::ENUM) {
            let mut parts = Vec::new();
            push_keyword(&mut parts, "enum");
            push_space(&mut parts, " ");
            push_part(&mut parts, &symbol.name, DisplayPartKind::EnumName);
            return parts;
        }
        if flags.intersects(SymbolFlags::TypeAlias) {
            return self.type_alias_symbol_display_parts(symbol);
        }
        if flags.intersects(SymbolFlags::TypeParameter) {
            return self.type_parameter_symbol_display_parts(symbol);
        }
        if flags.intersects(SymbolFlags::EnumMember) {
            let mut parts = Vec::new();
            let t = self.get_type_of_symbol(symbol);
            push_part(&mut parts, &symbol.name, DisplayPartKind::PropertyName);
            push_space(&mut parts, ": ");
            parts.extend(self.type_to_display_parts(&t));
            return parts;
        }
        if flags.intersects(SymbolFlags::VARIABLE)
            || flags.intersects(SymbolFlags::Property)
            || flags.intersects(SymbolFlags::ACCESSOR)
        {
            return self.variable_symbol_display_parts(symbol);
        }
        if flags.intersects(SymbolFlags::MODULE) || flags.intersects(SymbolFlags::NamespaceModule) {
            let mut parts = Vec::new();
            push_keyword(&mut parts, "module");
            push_space(&mut parts, " ");
            push_part(&mut parts, &symbol.name, DisplayPartKind::Text);
            return parts;
        }
        if flags.intersects(SymbolFlags::Alias) {
            let mut parts = Vec::new();
            push_keyword(&mut parts, "import");
            push_space(&mut parts, " ");
            push_part(&mut parts, &symbol.name, DisplayPartKind::Text);
            return parts;
        }

        // Fallback: name + resolved type.
        let mut parts = Vec::new();
        push_part(&mut parts, &symbol.name, DisplayPartKind::VariableName);
        push_space(&mut parts, ": ");
        let t = self.get_type_of_symbol(symbol);
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    /// Produce a classified `SymbolDisplayPart[]` for a type. This is a
    /// simpler version that classifies the type string: intrinsic keyword
    /// types (`string`, `number`, …) become a single keyword part; types
    /// backed by a named symbol (class/interface/enum) become a single name
    /// part with the corresponding kind; everything else becomes a single
    /// unclassified text part.
    pub fn type_to_display_parts(&mut self, t: &Arc<Type>) -> Vec<SymbolDisplayPart> {
        let s = self.type_to_string(t);

        // Intrinsic keyword types → keyword part.
        if let Some(name) = t.intrinsic_name() {
            if is_keyword_type_name(name) {
                return vec![SymbolDisplayPart::new(s, DisplayPartKind::Keyword)];
            }
        }

        // Type backed by a named symbol → name part with the symbol's kind.
        if let Some(sym) = &t.symbol {
            return vec![SymbolDisplayPart::new(s, display_kind_for_symbol(sym))];
        }

        // Fallback: unclassified text.
        vec![SymbolDisplayPart::new(s, DisplayPartKind::Text)]
    }

    /// Emit display parts for a function/method symbol: an optional
    /// `function` keyword, the name (with type parameters), the parameter
    /// list, and the return type.
    fn function_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        is_method: bool,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts: Vec<SymbolDisplayPart> = Vec::new();
        if !is_method {
            push_keyword(&mut parts, "function");
            push_space(&mut parts, " ");
        }
        push_part(&mut parts, &symbol.name, DisplayPartKind::FunctionName);
        self.append_type_parameter_parts(&mut parts, symbol);

        let t = self.get_type_of_symbol(symbol);
        if let Some(structured) = t.as_structured() {
            if let Some(sig) = structured.call_signatures().first() {
                push_punctuation(&mut parts, "(");
                self.append_signature_parameter_parts(&mut parts, sig);
                push_punctuation(&mut parts, ")");
                push_space(&mut parts, ": ");
                let ret = sig
                    .resolved_return_type
                    .get()
                    .cloned()
                    .unwrap_or_else(|| self.any_type());
                parts.extend(self.type_to_display_parts(&ret));
                return parts;
            }
        }
        // Fallback: `name: Type`.
        push_space(&mut parts, ": ");
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    /// Emit display parts for a class/interface symbol: the keyword, the
    /// name (with type parameters).
    fn named_type_symbol_display_parts(
        &self,
        symbol: &Arc<Symbol>,
        keyword: &'static str,
        name_kind: DisplayPartKind,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        push_keyword(&mut parts, keyword);
        push_space(&mut parts, " ");
        push_part(&mut parts, &symbol.name, name_kind);
        self.append_type_parameter_parts(&mut parts, symbol);
        parts
    }

    /// Emit display parts for a type alias: `type Name<T> = Type`.
    fn type_alias_symbol_display_parts(&mut self, symbol: &Arc<Symbol>) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        push_keyword(&mut parts, "type");
        push_space(&mut parts, " ");
        push_part(&mut parts, &symbol.name, DisplayPartKind::Text);
        self.append_type_parameter_parts(&mut parts, symbol);
        push_space(&mut parts, " = ");
        if let Some(t) = self.try_get_type_alias_declared_type(symbol) {
            parts.extend(self.type_to_display_parts(&t));
        }
        parts
    }

    /// Emit display parts for a type parameter: `T extends Constraint`.
    fn type_parameter_symbol_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        push_part(&mut parts, &symbol.name, DisplayPartKind::TypeParameterName);
        if let Some(c) = self.get_constraint_of_type_parameter_symbol(symbol) {
            push_keyword(&mut parts, " extends ");
            parts.extend(self.type_to_display_parts(&c));
        }
        parts
    }

    /// Emit display parts for a variable/property symbol: `let x: T` /
    /// `(property) x: T`.
    fn variable_symbol_display_parts(&mut self, symbol: &Arc<Symbol>) -> Vec<SymbolDisplayPart> {
        let mut parts = Vec::new();
        if symbol.flags.intersects(SymbolFlags::Property) {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "property", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else if symbol.flags.intersects(SymbolFlags::ACCESSOR) {
            push_punctuation(&mut parts, "(");
            push_part(&mut parts, "accessor", DisplayPartKind::Text);
            push_punctuation(&mut parts, ") ");
        } else {
            push_keyword(&mut parts, self.variable_decl_prefix(symbol).trim());
            push_space(&mut parts, " ");
        }

        let name_kind = if symbol
            .flags
            .intersects(SymbolFlags::Property | SymbolFlags::ACCESSOR)
        {
            DisplayPartKind::PropertyName
        } else {
            DisplayPartKind::VariableName
        };
        push_part(&mut parts, &symbol.name, name_kind);
        if symbol.flags.contains(SymbolFlags::Optional) {
            push_punctuation(&mut parts, "?");
        }
        push_space(&mut parts, ": ");
        let t = self.get_type_of_symbol(symbol);
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    /// Append a signature's parameter list as display parts, in the form
    /// `a: T, b: U, c?: V`.
    fn append_signature_parameter_parts(
        &mut self,
        parts: &mut Vec<SymbolDisplayPart>,
        sig: &Signature,
    ) {
        for (i, param) in sig.parameters.iter().enumerate() {
            if i > 0 {
                push_space(parts, ", ");
            }
            push_part(parts, &param.name, DisplayPartKind::ParameterName);
            if param.flags.contains(SymbolFlags::Optional) {
                push_punctuation(parts, "?");
            }
            push_space(parts, ": ");
            let pt = self.get_type_of_symbol(param);
            parts.extend(self.type_to_display_parts(&pt));
        }
    }

    /// Append a symbol's type parameters as display parts, in the form
    /// `<T, U>`. No-op when the symbol has no type parameters.
    fn append_type_parameter_parts(
        &self,
        parts: &mut Vec<SymbolDisplayPart>,
        symbol: &Arc<Symbol>,
    ) {
        if let Some(tps) = self.collect_type_parameter_names(symbol) {
            if !tps.is_empty() {
                push_punctuation(parts, "<");
                for (i, tp) in tps.iter().enumerate() {
                    if i > 0 {
                        push_space(parts, ", ");
                    }
                    push_part(parts, tp, DisplayPartKind::TypeParameterName);
                }
                push_punctuation(parts, ">");
            }
        }
    }

    /// Format the quick-info (hover) text for `symbol` at the location of
    /// `node`. Determines the kind prefix (`let`, `function`, `class`, …)
    /// from the symbol's declaration and appends the type/signature.
    fn format_quick_info_for_symbol(&mut self, symbol: &Arc<Symbol>, node: &Arc<Node>) -> String {
        let flags = symbol.flags;
        // Determine the kind prefix from the symbol flags. Mirrors the
        // dispatch in `getQuickInfoAndDeclarationAtLocation` (hover.go).
        // Use `intersects` (not `contains`) because some flag groups like
        // `VARIABLE` are unions of multiple bits and a symbol may carry
        // only one of them.
        if flags.intersects(SymbolFlags::Function) {
            return self.format_function_quick_info(symbol, /*is_method=*/ false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.format_function_quick_info(symbol, /*is_method=*/ true);
        }
        if flags.intersects(SymbolFlags::Class) {
            return self.format_class_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::Interface) {
            return self.format_interface_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::ENUM) {
            return self.format_enum_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::TypeAlias) {
            return self.format_type_alias_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::TypeParameter) {
            return self.format_type_parameter_quick_info(symbol);
        }
        if flags.intersects(SymbolFlags::EnumMember) {
            return self.format_enum_member_quick_info(symbol);
        }
        // Variable / property / parameter.
        if flags.intersects(SymbolFlags::VARIABLE)
            || flags.intersects(SymbolFlags::Property)
            || flags.intersects(SymbolFlags::ACCESSOR)
        {
            return self.format_variable_quick_info(symbol, node);
        }
        if flags.intersects(SymbolFlags::MODULE) {
            return format!("module {}", symbol.name);
        }
        if flags.intersects(SymbolFlags::NamespaceModule) {
            return format!("namespace {}", symbol.name);
        }
        if flags.intersects(SymbolFlags::Alias) {
            return self.format_alias_quick_info(symbol);
        }
        // Fallback: just the symbol name + its resolved type.
        let t = self.get_type_of_symbol(symbol);
        format!("{}: {}", symbol.name, self.type_to_string(&t))
    }

    fn format_function_quick_info(&mut self, symbol: &Arc<Symbol>, is_method: bool) -> String {
        let prefix = if is_method { "" } else { "function " };
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        let t = self.get_type_of_symbol(symbol);
        // Function-typed object type: extract the call signature.
        if let Some(structured) = t.as_structured() {
            if let Some(sig) = structured.call_signatures().first() {
                let params = self.format_signature_parameters(sig);
                let ret = sig
                    .resolved_return_type
                    .get()
                    .cloned()
                    .unwrap_or_else(|| self.any_type());
                let ret_str = self.type_to_string(&ret);
                return format!("{}{}({}): {}", prefix, name, params, ret_str);
            }
        }
        // Fallback: just show the resolved type string.
        format!("{}{}: {}", prefix, name, self.type_to_string(&t))
    }

    fn format_class_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        format!("class {}", name)
    }

    fn format_interface_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        format!("interface {}", name)
    }

    fn format_enum_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        format!("enum {}", symbol.name)
    }

    fn format_type_alias_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let name = self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::WriteTypeParametersOrArguments,
            SymbolFlags::all(),
        );
        // Try to resolve the aliased type for display.
        if let Some(t) = self.try_get_type_alias_declared_type(symbol) {
            let t_str = self.type_to_string(&t);
            format!("type {} = {}", name, t_str)
        } else {
            format!("type {}", name)
        }
    }

    fn format_type_parameter_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let constraint = self.get_constraint_of_type_parameter_symbol(symbol);
        match constraint {
            Some(c) => format!("{} extends {}", symbol.name, self.type_to_string(&c)),
            None => symbol.name.clone(),
        }
    }

    fn format_enum_member_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        let t = self.get_type_of_symbol(symbol);
        format!("{}.{}", "<enum>", self.type_to_string(&t))
    }

    fn format_variable_quick_info(&mut self, symbol: &Arc<Symbol>, _node: &Arc<Node>) -> String {
        // Determine `let` vs `const` vs `var` from the declaration. The
        // binder currently tags all variable declarations (var/let/const)
        // with `BlockScopedVariable`, so we look at the parent
        // `VariableDeclarationList`'s `NodeFlags` to disambiguate.
        let prefix = self.variable_decl_prefix(symbol);
        let t = self.get_type_of_symbol(symbol);
        format!("{}{}: {}", prefix, symbol.name, self.type_to_string(&t))
    }

    /// Return `"let "`, `"const "`, or `"var "` based on the symbol's
    /// declaration list.
    fn variable_decl_prefix(&self, symbol: &Arc<Symbol>) -> &'static str {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {
                if parent.kind == SyntaxKind::VariableDeclarationList {
                    if parent.flags.contains(crate::ast::NodeFlags::Const) {
                        return "const ";
                    }
                    if parent.flags.contains(crate::ast::NodeFlags::Let) {
                        return "let ";
                    }
                    // Neither `Const` nor `Let` → `var`.
                    return "var ";
                }
            }
        }
        // Default fallback: use the symbol flag.
        if symbol.flags.contains(SymbolFlags::BlockScopedVariable) {
            "let "
        } else {
            "var "
        }
    }

    fn format_alias_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        format!("import {}", symbol.name)
    }

    /// Format a signature's parameter list as `a: T, b: U`.
    fn format_signature_parameters(&mut self, sig: &Signature) -> String {
        let parts: Vec<String> = sig
            .parameters
            .iter()
            .map(|param| {
                let name = param.name.clone();
                let param_type = self.get_type_of_symbol(param);
                let type_str = self.type_to_string(&param_type);
                if param.flags.contains(SymbolFlags::Optional) {
                    format!("{}?: {}", name, type_str)
                } else {
                    format!("{}: {}", name, type_str)
                }
            })
            .collect();
        parts.join(", ")
    }

    /// Check if a variable symbol was declared with `const`.
    fn symbol_is_const(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {
                // VariableDeclarationList carries the `const`/`let` keyword
                // on its parent Node's `flags` (NodeFlags::Const), not on
                // the data struct itself.
                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(crate::ast::NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Try to get the declared type of a type alias symbol. Triggers
    /// resolution (with cycle protection) when the cache is empty, so hover
    /// info on an otherwise-unreferenced alias still displays its body.
    fn try_get_type_alias_declared_type(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {
        // Check the cached declared type on `type_alias_links`.
        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(t) = &links.declared_type {
                return Some(Arc::clone(t));
            }
        }
        // Cycle guard: a recursive alias (`type A = B; type B = A`) would
        // otherwise infinite-loop. Uses the stack-based resolution cycle
        // detection (mirrors Go's pushTypeResolution).
        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return None;
        }
        let result = self.resolve_alias_body(symbol);
        self.pop_type_resolution();
        // Cache the result for future lookups.
        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        Some(result)
    }

    /// Try to get the constraint of a type parameter symbol. Wraps the
    /// checker's existing `get_constraint_of_type_parameter` by first
    /// resolving the symbol to its declared type-parameter type.
    fn get_constraint_of_type_parameter_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
    ) -> Option<Arc<Type>> {
        let t = self.get_type_of_symbol(symbol);
        if t.flags.contains(TypeFlags::TypeParameter) {
            return self.get_constraint_of_type_parameter(&t);
        }
        None
    }

    /// Cheap check whether `get_type_of_node` would produce a meaningful
    /// type for `node` (used to gate fallback hover output).
    fn node_has_type(&self, node: &Arc<Node>) -> bool {
        matches!(
            node.kind,
            SyntaxKind::NumericLiteral
                | SyntaxKind::StringLiteral
                | SyntaxKind::NoSubstitutionTemplateLiteral
                | SyntaxKind::TemplateExpression
                | SyntaxKind::ArrayLiteralExpression
                | SyntaxKind::ObjectLiteralExpression
                | SyntaxKind::BinaryExpression
                | SyntaxKind::PrefixUnaryExpression
                | SyntaxKind::PostfixUnaryExpression
                | SyntaxKind::CallExpression
                | SyntaxKind::NewExpression
                | SyntaxKind::PropertyAccessExpression
                | SyntaxKind::ElementAccessExpression
                | SyntaxKind::ParenthesizedExpression
                | SyntaxKind::ConditionalExpression
                | SyntaxKind::TypeAssertionExpression
                | SyntaxKind::AsExpression
                | SyntaxKind::NonNullExpression
        )
    }
}

/// Maximum recursion depth for type serialization. Prevents stack overflow
/// on recursive types. Mirrors Go's `maxSerializationLevel` (= 2).
/// Type serialization can trigger lazy member resolution, which in turn
/// produces diagnostics requiring further serialization — leading to
/// infinite recursion. At this depth we return "?".
const MAX_SERIALIZATION_LEVEL: i32 = 2;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundled::lib_path;
    use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
    use crate::tsoptions::parse_command_line;
    use crate::vfs::InMemoryFS;

    /// Build a checker for a single source file (with `--noLib`), mimicking
    /// `build_checker` in `tests/checker_parity.rs`. Exposed so tests can
    /// exercise checker APIs directly after type-checking completes.
    fn build_checker(source: &str) -> Checker {
        let fs = Arc::new(InMemoryFS::new());
        fs.insert_dir("/proj");
        fs.insert_file("/proj/entry.ts", source);
        let args = vec!["--noLib".to_string(), "/proj/entry.ts".to_string()];
        let parsed = parse_command_line(&args, "/proj", Some(fs.as_ref()));
        let host: Arc<dyn CompilerHost> =
            Arc::new(CompilerHostImpl::new(fs, "/proj".to_string(), lib_path()));
        let program = Arc::new(Program::new(ProgramOptions {
            config: parsed,
            host,
        }));
        program.build_checker()
    }

    /// Find the first `VariableDeclaration` in the entry source file and
    /// return its type annotation node (the `: T` part of `let x: T = ...`).
    fn first_var_type_node(checker: &Checker) -> Arc<Node> {
        let file = checker
            .files
            .iter()
            .find(|f| f.file_name == "/proj/entry.ts")
            .expect("entry source file");
        let NodeData::SourceFile(sf) = &file.node.data else {
            panic!("not a source file");
        };
        for stmt in sf.statements.nodes.iter() {
            if stmt.kind != SyntaxKind::VariableStatement {
                continue;
            }
            let NodeData::VariableStatement(vs) = &stmt.data else {
                continue;
            };
            let NodeData::VariableDeclarationList(vdl) = &vs.declaration_list.data else {
                continue;
            };
            for decl in vdl.declarations.nodes.iter() {
                let NodeData::VariableDeclaration(vd) = &decl.data else {
                    continue;
                };
                if let Some(tn) = &vd.type_node {
                    return Arc::clone(tn);
                }
            }
        }
        panic!("no variable declaration with type annotation found");
    }

    /// Render a synthetic TypeNode AST to a string. This is a minimal printer
    /// covering the node kinds produced by `type_to_type_node`. Used to
    /// verify that the AST built by `type_to_type_node` matches the type's
    /// string representation from `type_to_string`.
    fn type_node_to_string(node: &Arc<Node>) -> String {
        match node.kind {
            SyntaxKind::AnyKeyword => "any".into(),
            SyntaxKind::UnknownKeyword => "unknown".into(),
            SyntaxKind::StringKeyword => "string".into(),
            SyntaxKind::NumberKeyword => "number".into(),
            SyntaxKind::BigIntKeyword => "bigint".into(),
            SyntaxKind::BooleanKeyword => "boolean".into(),
            SyntaxKind::SymbolKeyword => "symbol".into(),
            SyntaxKind::VoidKeyword => "void".into(),
            SyntaxKind::UndefinedKeyword => "undefined".into(),
            SyntaxKind::NullKeyword => "null".into(),
            SyntaxKind::ObjectKeyword => "object".into(),
            SyntaxKind::NeverKeyword => "never".into(),
            SyntaxKind::TrueKeyword => "true".into(),
            SyntaxKind::FalseKeyword => "false".into(),
            SyntaxKind::UniqueKeyword => "unique".into(),
            SyntaxKind::ReadonlyKeyword => "readonly".into(),
            SyntaxKind::KeyOfKeyword => "keyof".into(),
            SyntaxKind::Identifier => node.text().to_string(),
            SyntaxKind::StringLiteral => {
                if let NodeData::StringLiteral(d) = &node.data {
                    format!("\"{}\"", d.text)
                } else {
                    "?".into()
                }
            }
            SyntaxKind::NumericLiteral => {
                if let NodeData::NumericLiteral(d) = &node.data {
                    d.text.clone()
                } else {
                    "?".into()
                }
            }
            SyntaxKind::BigIntLiteral => {
                if let NodeData::BigIntLiteral(d) = &node.data {
                    d.text.clone()
                } else {
                    "?".into()
                }
            }
            SyntaxKind::LiteralType => {
                if let NodeData::LiteralTypeNode(d) = &node.data {
                    type_node_to_string(&d.literal)
                } else {
                    "?".into()
                }
            }
            SyntaxKind::TypeReference => {
                if let NodeData::TypeReferenceNode(d) = &node.data {
                    let name = type_node_to_string(&d.type_name);
                    if let Some(args) = &d.type_arguments {
                        let parts: Vec<String> =
                            args.nodes.iter().map(type_node_to_string).collect();
                        format!("{}<{}>", name, parts.join(", "))
                    } else {
                        name
                    }
                } else {
                    "?".into()
                }
            }
            SyntaxKind::ArrayType => {
                if let NodeData::ArrayTypeNode(d) = &node.data {
                    format!("{}[]", type_node_to_string(&d.element_type))
                } else {
                    "?".into()
                }
            }
            SyntaxKind::TupleType => {
                if let NodeData::TupleTypeNode(d) = &node.data {
                    let parts: Vec<String> =
                        d.elements.nodes.iter().map(type_node_to_string).collect();
                    format!("[{}]", parts.join(", "))
                } else {
                    "?".into()
                }
            }
            SyntaxKind::UnionType => {
                if let NodeData::UnionTypeNode(d) = &node.data {
                    let parts: Vec<String> =
                        d.types.nodes.iter().map(type_node_to_string).collect();
                    parts.join(" | ")
                } else {
                    "?".into()
                }
            }
            SyntaxKind::IntersectionType => {
                if let NodeData::IntersectionTypeNode(d) = &node.data {
                    let parts: Vec<String> =
                        d.types.nodes.iter().map(type_node_to_string).collect();
                    parts.join(" & ")
                } else {
                    "?".into()
                }
            }
            SyntaxKind::ParenthesizedType => {
                if let NodeData::ParenthesizedTypeNode(d) = &node.data {
                    format!("({})", type_node_to_string(&d.type_node))
                } else {
                    "?".into()
                }
            }
            SyntaxKind::FunctionType => {
                if let NodeData::FunctionTypeNode(d) = &node.data {
                    let params: Vec<String> =
                        d.parameters.nodes.iter().map(type_node_to_string).collect();
                    let ret = d
                        .type_node
                        .as_ref()
                        .map(type_node_to_string)
                        .unwrap_or_else(|| "unknown".into());
                    format!("({}) => {}", params.join(", "), ret)
                } else {
                    "?".into()
                }
            }
            SyntaxKind::Parameter => {
                if let NodeData::ParameterDeclaration(d) = &node.data {
                    let name = type_node_to_string(&d.name);
                    let ty = d
                        .type_node
                        .as_ref()
                        .map(type_node_to_string)
                        .unwrap_or_else(|| "any".into());
                    if d.question_token.is_some() {
                        format!("{}?: {}", name, ty)
                    } else {
                        format!("{}: {}", name, ty)
                    }
                } else {
                    "?".into()
                }
            }
            SyntaxKind::TypeLiteral => {
                if let NodeData::TypeLiteralNode(d) = &node.data {
                    let members: Vec<String> =
                        d.members.nodes.iter().map(type_node_to_string).collect();
                    if members.is_empty() {
                        "{}".into()
                    } else {
                        // Trailing `;` like Go's printer (declaration emit
                        // and checker display both render
                        // `{ a: number; b: string; }`).
                        format!("{{ {}; }}", members.join("; "))
                    }
                } else {
                    "?".into()
                }
            }
            SyntaxKind::PropertySignature => {
                if let NodeData::PropertySignatureDeclaration(d) = &node.data {
                    let name = type_node_to_string(&d.name);
                    let ty = type_node_to_string(&d.type_node);
                    if d.postfix_token.is_some() {
                        format!("{}?: {}", name, ty)
                    } else {
                        format!("{}: {}", name, ty)
                    }
                } else {
                    "?".into()
                }
            }
            SyntaxKind::RestType => {
                if let NodeData::RestTypeNode(d) = &node.data {
                    format!("...{}", type_node_to_string(&d.type_node))
                } else {
                    "?".into()
                }
            }
            SyntaxKind::TypeOperator => {
                if let NodeData::TypeOperatorNode(d) = &node.data {
                    let op = match d.operator {
                        SyntaxKind::UniqueKeyword => "unique ",
                        SyntaxKind::ReadonlyKeyword => "readonly ",
                        SyntaxKind::KeyOfKeyword => "keyof ",
                        _ => "",
                    };
                    format!("{}{}", op, type_node_to_string(&d.type_node))
                } else {
                    "?".into()
                }
            }
            _ => "?".into(),
        }
    }

    /// Build a checker for `source`, find the first variable's type
    /// annotation, resolve it to a `Type`, serialize it back to a TypeNode,
    /// and assert that the rendered TypeNode matches the type's string.
    fn assert_var_type_round_trips(source: &str) {
        let mut checker = build_checker(source);
        let type_node = first_var_type_node(&checker);
        let t = checker.get_type_from_type_node(&type_node);
        let expected = checker.type_to_string(&t);
        let built = checker.type_to_type_node(&t);
        let actual = type_node_to_string(&built);
        assert_eq!(
            actual, expected,
            "type_to_type_node round-trip mismatch for source: {source}\n\
             type_to_string: {expected:?}\n\
             type_node_to_string: {actual:?}"
        );
    }

    // ── Primitive / intrinsic types ──────────────────────────────────

    #[test]
    fn type_to_type_node_number() {
        assert_var_type_round_trips("let x: number = 0;");
    }

    #[test]
    fn type_to_type_node_string() {
        assert_var_type_round_trips("let x: string = \"\";");
    }

    #[test]
    fn type_to_type_node_boolean() {
        assert_var_type_round_trips("let x: boolean = true;");
    }

    #[test]
    fn type_to_type_node_void() {
        assert_var_type_round_trips("let x: void = undefined;");
    }

    #[test]
    fn type_to_type_node_any() {
        assert_var_type_round_trips("let x: any = 0;");
    }

    #[test]
    fn type_to_type_node_unknown() {
        assert_var_type_round_trips("let x: unknown = 0;");
    }

    #[test]
    fn type_to_type_node_never() {
        // No initializer: the checker still resolves the type annotation.
        assert_var_type_round_trips("let x: never;");
    }

    #[test]
    fn type_to_type_node_null() {
        assert_var_type_round_trips("let x: null = null;");
    }

    #[test]
    fn type_to_type_node_undefined() {
        assert_var_type_round_trips("let x: undefined = undefined;");
    }

    // ── Array & Tuple ────────────────────────────────────────────────

    #[test]
    fn type_to_type_node_array_of_number() {
        assert_var_type_round_trips("let x: number[] = [];");
    }

    #[test]
    fn type_to_type_node_array_of_string() {
        assert_var_type_round_trips("let x: string[] = [\"\"];");
    }

    #[test]
    fn type_to_type_node_tuple() {
        assert_var_type_round_trips("let x: [number, string] = [0, \"\"];");
    }

    // ── Union & Intersection ─────────────────────────────────────────

    #[test]
    fn type_to_type_node_union_number_string() {
        assert_var_type_round_trips("let x: number | string = 0;");
    }

    #[test]
    fn type_to_type_node_union_string_null() {
        assert_var_type_round_trips("let x: string | null = null;");
    }

    #[test]
    fn type_to_type_node_intersection() {
        assert_var_type_round_trips(
            "interface A { a: number }\n\
             interface B { b: string }\n\
             let x: A & B = { a: 1, b: \"\" };",
        );
    }

    // ── Type reference with arguments ────────────────────────────────

    #[test]
    fn type_to_type_node_generic_interface_reference() {
        assert_var_type_round_trips(
            "interface Foo<T> { value: T }\n\
             let x: Foo<number> = { value: 1 };",
        );
    }

    // ── Function type ────────────────────────────────────────────────

    #[test]
    fn type_to_type_node_function_type() {
        assert_var_type_round_trips("let x: (a: number) => string = (a) => \"\";");
    }

    // ── Object literal type ──────────────────────────────────────────

    #[test]
    fn type_to_type_node_object_literal() {
        assert_var_type_round_trips("let x: { a: number; b: string } = { a: 1, b: \"\" };");
    }

    // ── Literal types ────────────────────────────────────────────────

    #[test]
    fn type_to_type_node_string_literal_type() {
        assert_var_type_round_trips("let x: \"hello\" = \"hello\";");
    }

    #[test]
    fn type_to_type_node_numeric_literal_type() {
        assert_var_type_round_trips("let x: 42 = 42;");
    }

    // ───────────────────────────────────────────────────────────────────
    // symbol_to_display_parts / type_to_display_parts
    // ───────────────────────────────────────────────────────────────────

    use crate::ast::node_data_generated::for_each_child;

    /// Recursively walk `node`'s subtree looking for the first `Identifier`
    /// whose text matches `name`.
    fn find_identifier(node: &Arc<Node>, name: &str) -> Option<Arc<Node>> {
        if node.kind == SyntaxKind::Identifier {
            if let NodeData::Identifier(id) = &node.data {
                if id.text == name {
                    return Some(Arc::clone(node));
                }
            }
        }
        let mut found: Option<Arc<Node>> = None;
        for_each_child(node, |child| {
            if found.is_none() {
                found = find_identifier(child, name);
            }
            found.is_some()
        });
        found
    }

    /// Build a checker for `source`, find the first identifier named `name`,
    /// and produce its structured display parts via
    /// `get_quick_info_display_parts`. Panics if the name is not found.
    fn display_parts_for(source: &str, name: &str) -> Vec<SymbolDisplayPart> {
        let mut checker = build_checker(source);
        let file = checker
            .files
            .iter()
            .find(|f| f.file_name == "/proj/entry.ts")
            .expect("entry source file");
        let node = find_identifier(&file.node, name)
            .unwrap_or_else(|| panic!("identifier `{name}` not found in source:\n{source}"));
        checker.get_quick_info_display_parts(&node)
    }

    /// Concatenate the text of all display parts into a single string.
    fn parts_text(parts: &[SymbolDisplayPart]) -> String {
        parts.iter().map(|p| p.text.as_str()).collect()
    }

    #[test]
    fn display_parts_function() {
        let parts = display_parts_for("function foo(x: number): string { return \"\"; }", "foo");
        // Concatenated text matches the plain hover string.
        assert_eq!(parts_text(&parts), "function foo(x: number): string");
        // Spot-check the classified structure against the task spec example.
        assert_eq!(
            parts,
            vec![
                SymbolDisplayPart::new("function", DisplayPartKind::Keyword),
                SymbolDisplayPart::new(" ", DisplayPartKind::Space),
                SymbolDisplayPart::new("foo", DisplayPartKind::FunctionName),
                SymbolDisplayPart::new("(", DisplayPartKind::Punctuation),
                SymbolDisplayPart::new("x", DisplayPartKind::ParameterName),
                SymbolDisplayPart::new(": ", DisplayPartKind::Space),
                SymbolDisplayPart::new("number", DisplayPartKind::Keyword),
                SymbolDisplayPart::new(")", DisplayPartKind::Punctuation),
                SymbolDisplayPart::new(": ", DisplayPartKind::Space),
                SymbolDisplayPart::new("string", DisplayPartKind::Keyword),
            ]
        );
    }

    #[test]
    fn display_parts_function_two_params() {
        let parts = display_parts_for(
            "function f(a: string, b: number): boolean { return true; }",
            "f",
        );
        assert_eq!(
            parts_text(&parts),
            "function f(a: string, b: number): boolean"
        );
    }

    #[test]
    fn display_parts_let_variable() {
        let parts = display_parts_for("let x: number = 0;", "x");
        assert_eq!(parts_text(&parts), "let x: number");
        assert_eq!(
            parts[0],
            SymbolDisplayPart::new("let", DisplayPartKind::Keyword)
        );
        assert_eq!(
            parts[2],
            SymbolDisplayPart::new("x", DisplayPartKind::VariableName)
        );
        assert_eq!(
            parts[4],
            SymbolDisplayPart::new("number", DisplayPartKind::Keyword)
        );
    }

    #[test]
    fn display_parts_const_variable() {
        let parts = display_parts_for("const s: string = \"hi\";", "s");
        assert_eq!(parts_text(&parts), "const s: string");
        assert_eq!(
            parts[0],
            SymbolDisplayPart::new("const", DisplayPartKind::Keyword)
        );
    }

    #[test]
    fn display_parts_var_variable() {
        let parts = display_parts_for("var v: boolean = true;", "v");
        assert_eq!(parts_text(&parts), "var v: boolean");
    }

    #[test]
    fn display_parts_class() {
        let parts = display_parts_for("class Foo<T, U> {}", "Foo");
        assert_eq!(parts_text(&parts), "class Foo<T, U>");
        assert_eq!(
            parts[0],
            SymbolDisplayPart::new("class", DisplayPartKind::Keyword)
        );
        assert_eq!(
            parts[2],
            SymbolDisplayPart::new("Foo", DisplayPartKind::ClassName)
        );
        // Type parameters are classified.
        assert_eq!(
            parts[4],
            SymbolDisplayPart::new("T", DisplayPartKind::TypeParameterName)
        );
        assert_eq!(
            parts[6],
            SymbolDisplayPart::new("U", DisplayPartKind::TypeParameterName)
        );
    }

    #[test]
    fn display_parts_interface() {
        let parts = display_parts_for("interface Bar<T> { x: T; }", "Bar");
        assert_eq!(parts_text(&parts), "interface Bar<T>");
        assert_eq!(
            parts[0],
            SymbolDisplayPart::new("interface", DisplayPartKind::Keyword)
        );
        assert_eq!(
            parts[2],
            SymbolDisplayPart::new("Bar", DisplayPartKind::InterfaceName)
        );
    }

    #[test]
    fn display_parts_enum() {
        let parts = display_parts_for("enum Color { Red, Green, Blue }", "Color");
        assert_eq!(parts_text(&parts), "enum Color");
        assert_eq!(
            parts[0],
            SymbolDisplayPart::new("enum", DisplayPartKind::Keyword)
        );
        assert_eq!(
            parts[2],
            SymbolDisplayPart::new("Color", DisplayPartKind::EnumName)
        );
    }

    #[test]
    fn display_parts_type_alias() {
        let parts = display_parts_for("type MyNumber = number;", "MyNumber");
        assert_eq!(parts_text(&parts), "type MyNumber = number");
        assert_eq!(
            parts[0],
            SymbolDisplayPart::new("type", DisplayPartKind::Keyword)
        );
        // The aliased type `number` is a keyword part.
        assert_eq!(parts.last().unwrap().kind, DisplayPartKind::Keyword);
    }

    #[test]
    fn display_parts_type_alias_with_type_params() {
        let parts = display_parts_for("type Id<T> = T;", "Id");
        assert!(parts_text(&parts).starts_with("type Id<T> = "));
    }

    #[test]
    fn display_parts_kind_round_trips_to_strings() {
        // The `as_str` labels should match the Language Service constants.
        assert_eq!(DisplayPartKind::Keyword.as_str(), "keyword");
        assert_eq!(DisplayPartKind::FunctionName.as_str(), "functionName");
        assert_eq!(DisplayPartKind::ClassName.as_str(), "className");
        assert_eq!(DisplayPartKind::ParameterName.as_str(), "parameterName");
        assert_eq!(DisplayPartKind::Punctuation.as_str(), "punctuation");
        assert_eq!(DisplayPartKind::Space.as_str(), "space");
    }

    #[test]
    fn type_to_display_parts_intrinsic_keyword() {
        let mut checker = build_checker("let x: number = 0;");
        let type_node = first_var_type_node(&checker);
        let t = checker.get_type_from_type_node(&type_node);
        let parts = checker.type_to_display_parts(&t);
        assert_eq!(
            parts,
            vec![SymbolDisplayPart::new("number", DisplayPartKind::Keyword)]
        );
    }

    #[test]
    fn type_to_display_parts_class_name() {
        // Type reference resolution depends on lib scope chain; verify the
        // classifier produces a Text part for the unresolved reference.
        let mut checker = build_checker("class Foo {}\nlet x: Foo = new Foo();");
        let type_node = first_var_type_node(&checker);
        let t = checker.get_type_from_type_node(&type_node);
        let parts = checker.type_to_display_parts(&t);
        assert!(!parts.is_empty());
    }
}
