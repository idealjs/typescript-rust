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

use crate::ast::{Node, NodeData, Symbol, SymbolFlags, SyntaxKind};

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
        // otherwise infinite-loop. Reuses the same `resolving_type_aliases`
        // set as `resolve_type_reference` so cycles are detected across both
        // entry points.
        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.resolving_type_aliases.insert(key) {
            return None;
        }
        let result = self.resolve_alias_body(symbol);
        self.resolving_type_aliases.remove(&key);
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
/// on recursive types. Mirrors Go's `maxSerializationLevel`.
const MAX_SERIALIZATION_LEVEL: i32 = 300;
