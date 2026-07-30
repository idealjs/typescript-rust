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

use crate::ast::{Symbol, SymbolFlags};

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
        // name is `Array`, format as `T[]` (or `Array<T>` if flagged).
        let is_array = obj_data.type_arguments.len() == 1
            && t.symbol
                .as_ref()
                .map(|s| s.name == "Array" || s.name == "ReadonlyArray")
                .unwrap_or(false);

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
            format!("{{ {} }}", parts.join("; "))
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
}

/// Maximum recursion depth for type serialization. Prevents stack overflow
/// on recursive types. Mirrors Go's `maxSerializationLevel`.
const MAX_SERIALIZATION_LEVEL: i32 = 300;
