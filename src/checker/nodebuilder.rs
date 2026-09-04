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

#[derive(Debug, Clone, Copy, Default)]
pub struct TypeFormatFlags(u32);

impl TypeFormatFlags {
    pub const NONE: Self = Self(0);

    pub const WRITE_ARRAY_AS_GENERIC: Self = Self(1 << 1);

    pub const USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE: Self = Self(1 << 2);

    pub const ALLOW_UNIQUE_ES_SYMBOL_TYPE: Self = Self(1 << 3);

    pub const NO_TRUNCATION: Self = Self(1 << 7);

    pub const MULTILINE_OBJECT_LITERALS: Self = Self(1 << 8);

    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolDisplayPart {
    pub text: String,
    pub kind: DisplayPartKind,
}

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

    pub fn new(text: impl Into<String>, kind: DisplayPartKind) -> Self {
        SymbolDisplayPart {
            text: text.into(),
            kind,
        }
    }
}

fn module_specifier_of_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    for ext in [
        ".d.ts", ".d.mts", ".d.cts", ".ts", ".tsx", ".mts", ".cts", ".js", ".jsx", ".mjs", ".cjs",
    ] {
        if let Some(stem) = base.strip_suffix(ext) {
            return stem.to_string();
        }
    }
    base.to_string()
}

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

fn push_keyword(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Keyword));
}

fn push_space(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Space));
}

fn push_punctuation(parts: &mut Vec<SymbolDisplayPart>, text: &str) {
    parts.push(SymbolDisplayPart::new(text, DisplayPartKind::Punctuation));
}

fn push_part(parts: &mut Vec<SymbolDisplayPart>, text: &str, kind: DisplayPartKind) {
    parts.push(SymbolDisplayPart::new(text, kind));
}

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

    pub fn type_to_string(&mut self, t: &Arc<Type>) -> String {
        self.type_to_string_ex(t, TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE)
    }

    pub fn type_to_string_ex(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {

        let key = Arc::as_ptr(t) as usize;
        if self.type_print_stack.len() >= 300 || self.type_print_stack.contains(&key) {
            return "...".to_string();
        }
        if self.serialization_level >= MAX_SERIALIZATION_LEVEL {
            return "?".to_string();
        }
        self.type_print_stack.push(key);
        let result = self.type_to_string_ex_worker(t, flags);
        self.type_print_stack.pop();
        result
    }

    fn type_to_string_ex_worker(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {

        if let Some(name) = t.intrinsic_name() {
            return name.to_string();
        }

        if let Some(val) = t.literal_value() {
            return self.literal_value_to_string(val);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {
            if let TypeData::UniqueESSymbol(sym) = &t.data {
                if flags.contains(TypeFormatFlags::ALLOW_UNIQUE_ES_SYMBOL_TYPE) {
                    return format!("unique symbol");
                }
                return format!("typeof {}", sym.name);
            }
        }

        if t.flags.contains(TypeFlags::Never) {
            return "never".to_string();
        }

        if t.is_union() {
            return self.union_to_string(t, flags);
        }

        if t.is_intersection() {
            return self.intersection_to_string(t, flags);
        }

        if t.is_type_parameter() {
            return self.type_parameter_to_string(t);
        }

        if let TypeData::IndexedAccess(ia) = &t.data {
            return self.indexed_access_to_string(ia, flags);
        }

        if let TypeData::TemplateLiteral(tl) = &t.data {
            return self.template_literal_to_string(tl, flags);
        }

        if let TypeData::Index(i) = &t.data {
            let target = i
                .target
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            return format!("keyof {target}");
        }
        if let TypeData::StringMapping(s) = &t.data {
            let target = s
                .target
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            let name = t
                .symbol
                .as_ref()
                .map(|sym| sym.name.clone())
                .unwrap_or_default();
            if name.is_empty() {
                return target;
            }
            return format!("{name}<{target}>");
        }
        if let TypeData::Mapped(m) = &t.data {

            if let Some(alias) = &t.alias
                && let Some(sym) = &alias.symbol
            {
                let args: Vec<String> = alias
                    .type_arguments
                    .iter()
                    .map(|a| self.type_to_string_ex(a, flags))
                    .collect();
                if args.is_empty() {
                    return sym.name.clone();
                }
                return format!("{}<{}>", sym.name, args.join(", "));
            }

            let mut decl_tp_name: Option<String> = None;
            let mut decl_constraint: Option<String> = None;
            if let Some(decl) = m.declaration.as_ref()
                && let crate::ast::NodeData::MappedTypeNode(md) = &decl.data
                && let crate::ast::NodeData::TypeParameterDeclaration(tpd) =
                    &md.type_parameter.data
            {
                decl_tp_name = Some(tpd.name.text().to_string());
                if let Some(c) = &tpd.constraint {
                    decl_constraint = self.node_source_text(c);
                }
            }
            let tp = decl_tp_name
                .filter(|n| !n.is_empty())
                .or_else(|| {
                    m.type_parameter
                        .as_ref()
                        .and_then(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
                })
                .unwrap_or_else(|| "K".to_string());
            let constraint = decl_constraint
                .filter(|c| !c.is_empty())
                .unwrap_or_else(|| {
                    m.constraint_type
                        .as_ref()
                        .map(|c| self.type_to_string_ex(c, flags))
                        .unwrap_or_else(|| "keyof any".to_string())
                });
            let as_clause = m
                .name_type
                .as_ref()
                .map(|n| format!(" as {}", self.type_to_string_ex(n, flags)))
                .unwrap_or_default();
            let template = m
                .template_type
                .as_ref()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .unwrap_or_else(|| "any".to_string());
            return format!("{{ [{tp} in {constraint}{as_clause}]: {template}; }}");
        }
        if let TypeData::Substitution(sub) = &t.data {
            if let Some(base) = &sub.base_type {
                return self.type_to_string_ex(base, flags);
            }
            if let Some(c) = &sub.constraint {
                return self.type_to_string_ex(c, flags);
            }
        }
        if let TypeData::Conditional(c) = &t.data {

            if let Some(alias) = &t.alias
                && let Some(sym) = &alias.symbol
            {
                let args: Vec<String> = alias
                    .type_arguments
                    .iter()
                    .map(|a| self.type_to_string_ex(a, flags))
                    .collect();
                if args.is_empty() {
                    return sym.name.clone();
                }
                return format!("{}<{}>", sym.name, args.join(", "));
            }
            let root = c.root.as_ref();
            let check = root
                .and_then(|r| r.check_type.clone())
                .or_else(|| c.check_type.clone())
                .map(|ct| self.type_to_string_ex(&ct, flags))
                .unwrap_or_else(|| "unknown".to_string());
            let extends = root
                .and_then(|r| r.extends_type.clone())
                .or_else(|| c.extends_type.clone())
                .map(|et| self.type_to_string_ex(&et, flags))
                .unwrap_or_else(|| "unknown".to_string());

            let (cond_node, true_node, false_node) = root
                .and_then(|r| r.node.as_ref())
                .map(|n| {
                    match &n.data {
                        crate::ast::NodeData::ConditionalTypeNode(d) => (
                            Some(Arc::clone(n)),
                            Some(Arc::clone(&d.true_type)),
                            Some(Arc::clone(&d.false_type)),
                        ),
                        _ => (None, None, None),
                    }
                })
                .unwrap_or((None, None, None));
            if let Some(cn) = &cond_node {
                self.push_scope(cn);
            }
            let true_t = c
                .resolved_true_type
                .get()
                .map(|tt| self.type_to_string_ex(tt, flags))
                .or_else(|| {
                    true_node.map(|n| {
                        let t = self.get_type_from_type_node(&n);
                        self.type_to_string_ex(&t, flags)
                    })
                })
                .unwrap_or_else(|| "...".to_string());
            let false_t = c
                .resolved_false_type
                .get()
                .map(|ft| self.type_to_string_ex(ft, flags))
                .or_else(|| {
                    false_node.map(|n| {
                        let t = self.get_type_from_type_node(&n);
                        self.type_to_string_ex(&t, flags)
                    })
                })
                .unwrap_or_else(|| "...".to_string());
            if cond_node.is_some() {
                self.pop_scope();
            }
            return format!("{check} extends {extends} ? {true_t} : {false_t}");
        }

        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_string(t, flags);
        }

        if t.object_flags.contains(ObjectFlags::Reference) {
            return self.reference_to_string(t, flags);
        }

        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return self.function_type_to_string(t, structured, flags);
            }
        }

        if let Some(sym) = &t.symbol {
            return self.symbol_type_to_string(t, sym, flags);
        }

        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty()
                || !structured.call_signatures().is_empty()
                || !structured.construct_signatures().is_empty()
                || !structured.index_infos.is_empty()
            {
                return self.object_literal_to_string(t, structured, flags);
            }
            if t.object_flags.contains(ObjectFlags::ObjectLiteral) && t.symbol.is_none() {
                return "{}".to_string();
            }
        }

        if t.flags.contains(TypeFlags::Object) {
            return "object".to_string();
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return "unknown".to_string();
        }

        "<unknown type>".to_string()
    }

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

    fn union_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let types = t.types().unwrap_or(&[]);

        let mut ordered: Vec<&Arc<Type>> = Vec::with_capacity(types.len());
        let mut nulls: Vec<&Arc<Type>> = Vec::new();
        let mut undefs: Vec<&Arc<Type>> = Vec::new();
        for ty in types.iter() {
            if ty.flags.contains(TypeFlags::Undefined) {
                undefs.push(ty);
            } else if ty.flags.contains(TypeFlags::Null) {
                nulls.push(ty);
            } else {
                ordered.push(ty);
            }
        }
        ordered.extend(nulls);
        ordered.extend(undefs);
        let parts: Vec<String> = ordered
            .into_iter()
            .map(|ty| {
                let s = self.type_to_string_ex(ty, flags);

                if self.needs_parens_in_union(ty) {
                    format!("({})", s)
                } else {
                    s
                }
            })
            .collect();
        parts.join(" | ")
    }

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

    fn indexed_access_to_string(
        &mut self,
        ia: &IndexedAccessTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let obj = ia
            .object_type
            .as_ref()
            .map(|t| {
                let s = self.type_to_string_ex(t, flags);

                if matches!(
                    t.data,
                    TypeData::Conditional(_) | TypeData::Mapped(_)
                ) {
                    format!("({s})")
                } else {
                    s
                }
            })
            .unwrap_or_else(|| "any".to_string());
        let idx = ia
            .index_type
            .as_ref()
            .map(|t| self.type_to_string_ex(t, flags))
            .unwrap_or_else(|| "any".to_string());
        format!("{}[{}]", obj, idx)
    }

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

    fn tuple_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let TypeData::Tuple(tuple) = &t.data else {
            return "[]".to_string();
        };
        let readonly_prefix = if tuple.readonly { "readonly " } else { "" };
        if tuple.element_infos.is_empty() {
            return format!("{readonly_prefix}[]");
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
        format!("{readonly_prefix}[{}]", parts.join(", "))
    }

    fn reference_to_string(&mut self, t: &Arc<Type>, flags: TypeFormatFlags) -> String {
        let obj_data = match &t.data {
            TypeData::Object(o) => o,
            TypeData::Interface(i) => &i.object,
            _ => return "object".to_string(),
        };

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

    pub(crate) fn signature_instantiated_param_type(
        &self,
        sig: &Signature,
        i: usize,
    ) -> Option<Arc<Type>> {
        let overrides = sig.instantiated_parameter_types.as_ref()?;
        let rest_offset = usize::from(sig.has_rest_parameter());
        let fixed = overrides.len().saturating_sub(rest_offset);
        if i < fixed {
            return Some(Arc::clone(&overrides[i]));
        }

        if rest_offset == 1 && i == fixed {
            return Some(Arc::clone(&overrides[fixed]));
        }
        None
    }

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

        let sig = &sigs[0];
        let params: Vec<String> = sig
            .parameters
            .iter()
            .enumerate()
            .map(|(i, param)| {
                let name = param.name.clone();

                let param_type = self
                    .signature_instantiated_param_type(sig, i)
                    .unwrap_or_else(|| self.get_type_of_symbol(param));
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

        let tp_prefix = self.signature_type_param_prefix(sig);
        format!("{tp_prefix}({}) => {}", params.join(", "), ret_str)
    }

    fn signature_type_param_prefix(&self, sig: &Arc<Signature>) -> String {
        if sig.type_parameters.is_empty() {
            return String::new();
        }
        let names: Vec<String> = sig
            .type_parameters
            .iter()
            .filter_map(|tp| tp.symbol.as_ref().map(|s| s.name.clone()))
            .collect();
        if names.is_empty() {
            String::new()
        } else {
            format!("<{}>", names.join(", "))
        }
    }

    fn object_literal_to_string(
        &mut self,
        _t: &Arc<Type>,
        structured: &StructuredTypeData,
        flags: TypeFormatFlags,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();

        for sig in structured.call_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let name = param.name.clone();
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
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
            let tp = self.signature_type_param_prefix(sig);
            parts.push(format!("{tp}({}) => {}", params.join(", "), ret_str));
        }

        for sig in structured.construct_signatures() {
            let params: Vec<String> = sig
                .parameters
                .iter()
                .enumerate()
                .map(|(i, param)| {
                    let param_type = self
                        .signature_instantiated_param_type(sig, i)
                        .unwrap_or_else(|| self.get_type_of_symbol(param));
                    format!("{}: {}", param.name, self.type_to_string_ex(&param_type, flags))
                })
                .collect();
            let ret_type = sig
                .resolved_return_type
                .get()
                .cloned()
                .unwrap_or_else(|| self.any_type());
            let ret_str = self.type_to_string_ex(&ret_type, flags);
            let tp = self.signature_type_param_prefix(sig);
            parts.push(format!("new {tp}({}) => {}", params.join(", "), ret_str));
        }

        for prop in &structured.properties {
            let name = prop.name.clone();

            let name = if prop.declarations.iter().any(|d| {
                d.name().is_some_and(|n| n.kind == SyntaxKind::StringLiteral)
            }) {
                format!("\"{name}\"")
            } else {
                name
            };
            let prop_type = self.get_type_of_symbol(prop);
            let type_str = self.type_to_string_ex(&prop_type, flags);
            let readonly = prop
                .check_flags
                .contains(crate::ast::CheckFlags::Readonly);
            if prop.flags.contains(SymbolFlags::Optional) {

                let ro = if readonly { "readonly " } else { "" };
                parts.push(format!("{ro}{}?: {}", name, type_str));
            } else if readonly {
                parts.push(format!("readonly {}: {}", name, type_str));
            } else {
                parts.push(format!("{}: {}", name, type_str));
            }
        }

        for info in &structured.index_infos {
            let key_str = info
                .key_type
                .as_ref()
                .map(|k| self.type_to_string_ex(k, flags))
                .unwrap_or_else(|| "string".to_string());
            let val_str = info
                .value_type
                .as_ref()
                .map(|v| self.type_to_string_ex(v, flags))
                .unwrap_or_else(|| "any".to_string());

            let key_name = info
                .declaration
                .as_ref()
                .and_then(|d| {
                    let NodeData::IndexSignatureDeclaration(sd) = &d.data else {
                        return None;
                    };
                    sd.parameters.iter().next().and_then(|p| {
                        match &p.data {
                            NodeData::ParameterDeclaration(pd) => {
                                Some(pd.name.text().to_string())
                            }
                            _ => None,
                        }
                    })
                })
                .unwrap_or_else(|| "x".to_string());
            let readonly = if info.is_readonly { "readonly " } else { "" };
            parts.push(format!("{readonly}[{key_name}: {key_str}]: {val_str}"));
        }

        if parts.is_empty() {
            "{}".to_string()
        } else if structured.properties.is_empty()
            && structured.call_signatures().is_empty()
            && structured.construct_signatures().len() == 1
        {

            parts.join("")
        } else {

            format!("{{ {} }}", format!("{};", parts.join("; ")))
        }
    }

    fn symbol_type_to_string(
        &mut self,
        t: &Arc<Type>,
        sym: &Arc<Symbol>,
        flags: TypeFormatFlags,
    ) -> String {

        if sym.flags.contains(SymbolFlags::ENUM) {
            return sym.name.clone();
        }

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

        if sym.flags.contains(SymbolFlags::Class) {
            if let Some(structured) = t.as_structured() {
                if !structured.construct_signatures().is_empty() {
                    return format!("typeof {}", sym.name);
                }
            }
        }

        if sym.flags.contains(SymbolFlags::ValueModule) {
            if sym
                .declarations
                .iter()
                .any(|d| d.kind == SyntaxKind::SourceFile)
            {
                return format!("typeof import(\"{}\")", module_specifier_of_name(&sym.name));
            }
            for d in &sym.declarations {
                if let NodeData::ModuleDeclaration(md) = &d.data
                    && md.name.kind == SyntaxKind::StringLiteral
                {
                    return format!(
                        "typeof import(\"{}\")",
                        md.name.text().trim_matches(['"', '\''])
                    );
                }
            }
            return format!("typeof {}", sym.name);
        }

        sym.name.clone()
    }

    fn needs_parens_in_union(&mut self, t: &Arc<Type>) -> bool {

        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return true;
            }
        }

        false
    }

    fn needs_parens_as_array_element(&mut self, t: &Arc<Type>) -> bool {
        if t.is_union() || t.is_intersection() {
            return true;
        }
        if matches!(&t.data, TypeData::Conditional(_) | TypeData::Index(_)) {
            return true;
        }
        self.needs_parens_in_union(t)
    }

    fn maybe_parenthesize_array_element(&mut self, elem: &Arc<Type>) -> String {
        let s = self.type_to_string_ex(elem, TypeFormatFlags::NONE);
        if self.needs_parens_as_array_element(elem) {
            format!("({})", s)
        } else {
            s
        }
    }

    pub fn type_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        self.type_to_type_node_worker(t)
    }

    fn type_to_type_node_worker(&mut self, t: &Arc<Type>) -> Arc<Node> {

        if let Some(name) = t.intrinsic_name() {
            return self.intrinsic_to_type_node(name);
        }

        if let Some(val) = t.literal_value() {
            return self.literal_value_to_type_node(val);
        }

        if t.flags.contains(TypeFlags::UniqueESSymbol) {

            return self.type_operator_node(
                SyntaxKind::UniqueKeyword,
                self.keyword_node(SyntaxKind::SymbolKeyword),
            );
        }

        if t.flags.contains(TypeFlags::Never) {
            return self.keyword_node(SyntaxKind::NeverKeyword);
        }

        if t.is_union() {
            return self.union_to_type_node(t);
        }

        if t.is_intersection() {
            return self.intersection_to_type_node(t);
        }

        if t.is_type_parameter() {
            return self.type_parameter_to_type_node(t);
        }

        if t.object_flags.contains(ObjectFlags::Tuple) {
            return self.tuple_to_type_node(t);
        }

        if t.object_flags.contains(ObjectFlags::Reference) {
            return self.reference_to_type_node(t);
        }

        if let Some(structured) = t.as_structured() {
            if structured.call_signature_count > 0 && t.symbol.is_none() {
                return self.function_type_to_type_node(structured);
            }
        }

        if let Some(sym) = &t.symbol {
            let instance_args = t.as_object().and_then(|obj| {
                (!obj.type_arguments.is_empty()).then(|| {
                    let arg_nodes: Vec<Arc<Node>> = obj
                        .type_arguments
                        .iter()
                        .map(|ty| self.type_to_type_node(ty))
                        .collect();
                    Arc::new(NodeList::new(arg_nodes))
                })
            });
            return self.symbol_to_type_node(sym, SymbolFlags::TYPE, instance_args);
        }

        if let Some(structured) = t.as_structured() {
            if !structured.properties.is_empty()
                || !structured.call_signatures().is_empty()
                || !structured.index_infos.is_empty()
            {
                return self.type_literal_to_type_node(structured);
            }
        }

        if t.flags.contains(TypeFlags::Object) {
            return self.keyword_node(SyntaxKind::ObjectKeyword);
        }
        if t.flags.contains(TypeFlags::Unknown) {
            return self.keyword_node(SyntaxKind::UnknownKeyword);
        }

        self.keyword_node(SyntaxKind::AnyKeyword)
    }

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

            _ => SyntaxKind::AnyKeyword,
        };
        self.keyword_node(kind)
    }

    fn literal_value_to_type_node(&mut self, val: &LiteralValue) -> Arc<Node> {
        let literal = match val {
            LiteralValue::String(s) => self.string_literal_node(s),
            LiteralValue::Number(n) => self.numeric_literal_node(&n.to_string()),
            LiteralValue::BigInt(b) => self.bigint_literal_node(&b.to_string()),
            LiteralValue::Boolean(true) => self.keyword_node(SyntaxKind::TrueKeyword),
            LiteralValue::Boolean(false) => self.keyword_node(SyntaxKind::FalseKeyword),

            LiteralValue::None => return self.keyword_node(SyntaxKind::NullKeyword),
        };
        self.literal_type_node(literal)
    }

    fn union_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let types = t.types().unwrap_or(&[]);
        if types.is_empty() {
            return self.keyword_node(SyntaxKind::NeverKeyword);
        }
        if types.len() == 1 {
            return self.type_to_type_node(&types[0]);
        }

        let mut ordered: Vec<&Arc<Type>> = Vec::with_capacity(types.len());
        let mut nulls: Vec<&Arc<Type>> = Vec::new();
        let mut undefs: Vec<&Arc<Type>> = Vec::new();
        for ty in types.iter() {
            if ty.flags.contains(TypeFlags::Undefined) {
                undefs.push(ty);
            } else if ty.flags.contains(TypeFlags::Null) {
                nulls.push(ty);
            } else {
                ordered.push(ty);
            }
        }
        ordered.extend(nulls);
        ordered.extend(undefs);
        let nodes: Vec<Arc<Node>> = ordered
            .into_iter()
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

    fn type_parameter_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        if let TypeData::TypeParameter(tp) = &t.data {
            if tp.is_this_type {

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

    fn reference_to_type_node(&mut self, t: &Arc<Type>) -> Arc<Node> {
        let obj_data = match &t.data {
            TypeData::Object(o) => o,
            TypeData::Interface(i) => &i.object,
            _ => return self.keyword_node(SyntaxKind::ObjectKeyword),
        };

        let symbol_name = t.symbol.as_ref().map(|s| s.name.as_str()).unwrap_or("");
        let is_array = obj_data.type_arguments.len() == 1
            && (symbol_name == "Array" || symbol_name == "ReadonlyArray" || t.symbol.is_none());

        if is_array {
            let elem = &obj_data.type_arguments[0];
            let elem_node = self.type_to_type_node(elem);
            if self.needs_parens_in_union(elem) {
                return self.array_type_node(self.parenthesized_type_node(elem_node));
            }

            return self.array_type_node(elem_node);
        }

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

    fn type_literal_to_type_node(&mut self, structured: &StructuredTypeData) -> Arc<Node> {
        let mut members: Vec<Arc<Node>> = Vec::new();

        for sig in structured.call_signatures() {
            members.push(self.call_signature_to_node(sig));
        }

        for prop in &structured.properties {
            let name = self.identifier(&prop.name);
            let prop_type = self.get_type_of_symbol(prop);
            let type_node = self.type_to_type_node(&prop_type);
            let optional = prop.flags.contains(SymbolFlags::Optional);
            members.push(self.property_signature_node(name, optional, type_node));
        }

        self.type_literal_node(members)
    }

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

    pub fn symbol_to_type_node(
        &mut self,
        symbol: &Arc<Symbol>,
        mask: SymbolFlags,
        type_arguments: Option<Arc<NodeList>>,
    ) -> Arc<Node> {

        let _ = mask;

        let name = self.identifier(&symbol.name);

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

    fn keyword_node(&self, kind: SyntaxKind) -> Arc<Node> {
        Arc::new(Node::new(kind, NodeData::Token))
    }

    fn identifier(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::Identifier,
            NodeData::Identifier(IdentifierData {
                text: text.to_string(),
            }),
        ))
    }

    fn string_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::StringLiteral,
            NodeData::StringLiteral(StringLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    fn numeric_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::NumericLiteral,
            NodeData::NumericLiteral(NumericLiteralData {
                text: text.to_string(),
                token_flags: 0,
            }),
        ))
    }

    fn bigint_literal_node(&self, text: &str) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::BigIntLiteral,
            NodeData::BigIntLiteral(BigIntLiteralData {
                text: format!("{}n", text),
                token_flags: 0,
            }),
        ))
    }

    fn literal_type_node(&self, literal: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::LiteralType,
            NodeData::LiteralTypeNode(LiteralTypeNodeData { literal }),
        ))
    }

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

    fn array_type_node(&self, element_type: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ArrayType,
            NodeData::ArrayTypeNode(ArrayTypeNodeData { element_type }),
        ))
    }

    fn tuple_type_node(&self, elements: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TupleType,
            NodeData::TupleTypeNode(TupleTypeNodeData {
                elements: Arc::new(NodeList::new(elements)),
            }),
        ))
    }

    fn union_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::UnionType,
            NodeData::UnionTypeNode(UnionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    fn intersection_type_node(&self, types: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::IntersectionType,
            NodeData::IntersectionTypeNode(IntersectionTypeNodeData {
                types: Arc::new(NodeList::new(types)),
            }),
        ))
    }

    fn parenthesized_type_node(&self, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::ParenthesizedType,
            NodeData::ParenthesizedTypeNode(ParenthesizedTypeNodeData { type_node }),
        ))
    }

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

    fn type_literal_node(&self, members: Vec<Arc<Node>>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeLiteral,
            NodeData::TypeLiteralNode(TypeLiteralNodeData {
                members: Arc::new(NodeList::new(members)),
            }),
        ))
    }

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

    fn rest_type_node(&self, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::RestType,
            NodeData::RestTypeNode(RestTypeNodeData { type_node }),
        ))
    }

    fn type_operator_node(&self, operator: SyntaxKind, type_node: Arc<Node>) -> Arc<Node> {
        Arc::new(Node::new(
            SyntaxKind::TypeOperator,
            NodeData::TypeOperatorNode(TypeOperatorNodeData {
                operator,
                type_node,
            }),
        ))
    }

    pub fn symbol_to_string(&mut self, symbol: &Arc<Symbol>) -> String {
        self.symbol_to_string_ex(
            symbol,
            SymbolFormatFlags::AllowAnyNodeKind,
            SymbolFlags::all(),
        )
    }

    pub fn symbol_to_string_ex(
        &mut self,
        symbol: &Arc<Symbol>,
        flags: SymbolFormatFlags,
        _meaning: SymbolFlags,
    ) -> String {
        let name = symbol.name.clone();

        if flags.contains(SymbolFormatFlags::WriteTypeParametersOrArguments) {
            if let Some(tps) = self.collect_type_parameter_names(symbol) {
                if !tps.is_empty() {
                    return format!("{}<{}>", name, tps.join(", "));
                }
            }
        }
        name
    }

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

    pub fn get_quick_info_text(&mut self, node: &Arc<Node>) -> String {

        if node.kind == SyntaxKind::ThisKeyword {
            let t = self.get_type_of_node(node);
            return format!("this: {}", self.type_to_string(&t));
        }

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

            if self.node_has_type(node) {
                let t = self.get_type_of_node(node);
                return self.type_to_string(&t);
            }
            return String::new();
        };
        self.format_quick_info_for_symbol(&symbol, node)
    }

    pub fn get_quick_info_display_parts(&mut self, node: &Arc<Node>) -> Vec<SymbolDisplayPart> {

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

    pub fn symbol_to_display_parts(
        &mut self,
        symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        type_arguments: &[String],
    ) -> Vec<SymbolDisplayPart> {

        let _ = meaning;
        let _ = type_arguments;

        let flags = symbol.flags;
        if flags.intersects(SymbolFlags::Function) {
            return self.function_symbol_display_parts(symbol,  false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.function_symbol_display_parts(symbol,  true);
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

        let mut parts = Vec::new();
        push_part(&mut parts, &symbol.name, DisplayPartKind::VariableName);
        push_space(&mut parts, ": ");
        let t = self.get_type_of_symbol(symbol);
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

    pub fn type_to_display_parts(&mut self, t: &Arc<Type>) -> Vec<SymbolDisplayPart> {
        let s = self.type_to_string(t);

        if let Some(name) = t.intrinsic_name() {
            if is_keyword_type_name(name) {
                return vec![SymbolDisplayPart::new(s, DisplayPartKind::Keyword)];
            }
        }

        if let Some(sym) = &t.symbol {
            return vec![SymbolDisplayPart::new(s, display_kind_for_symbol(sym))];
        }

        vec![SymbolDisplayPart::new(s, DisplayPartKind::Text)]
    }

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

        push_space(&mut parts, ": ");
        parts.extend(self.type_to_display_parts(&t));
        parts
    }

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

    fn format_quick_info_for_symbol(&mut self, symbol: &Arc<Symbol>, node: &Arc<Node>) -> String {
        let flags = symbol.flags;

        if flags.intersects(SymbolFlags::Function) {
            return self.format_function_quick_info(symbol,  false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.format_function_quick_info(symbol,  true);
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

        let prefix = self.variable_decl_prefix(symbol);
        let t = self.get_type_of_symbol(symbol);
        format!("{}{}: {}", prefix, symbol.name, self.type_to_string(&t))
    }

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

                    return "var ";
                }
            }
        }

        if symbol.flags.contains(SymbolFlags::BlockScopedVariable) {
            "let "
        } else {
            "var "
        }
    }

    fn format_alias_quick_info(&mut self, symbol: &Arc<Symbol>) -> String {
        format!("import {}", symbol.name)
    }

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

    #[allow(dead_code)]
    fn symbol_is_const(&self, symbol: &Arc<Symbol>) -> bool {
        for decl in &symbol.declarations {
            if let Some(parent) = &decl.parent {

                if parent.kind == SyntaxKind::VariableDeclarationList
                    && parent.flags.contains(crate::ast::NodeFlags::Const)
                {
                    return true;
                }
            }
        }
        false
    }

    fn try_get_type_alias_declared_type(&mut self, symbol: &Arc<Symbol>) -> Option<Arc<Type>> {

        if let Some(links) = self.type_alias_links.get(symbol) {
            if let Some(t) = &links.declared_type {
                return Some(Arc::clone(t));
            }
        }

        let key = Arc::as_ptr(symbol) as *const crate::ast::Symbol;
        if !self.push_type_resolution(
            key,
            crate::checker::checker::TypeResolutionProperty::DeclaredType,
        ) {
            return None;
        }
        let result = self.resolve_alias_body(symbol);
        self.pop_type_resolution();

        self.type_alias_links.get_or_default(symbol).declared_type = Some(Arc::clone(&result));
        Some(result)
    }

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

const MAX_SERIALIZATION_LEVEL: i32 = 2;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundled::lib_path;
    use crate::compiler::{CompilerHost, CompilerHostImpl, Program, ProgramOptions};
    use crate::tsoptions::parse_command_line;
    use crate::vfs::InMemoryFS;

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

    #[test]
    fn type_to_type_node_generic_interface_reference() {
        assert_var_type_round_trips(
            "interface Foo<T> { value: T }\n\
             let x: Foo<number> = { value: 1 };",
        );
    }

    #[test]
    fn type_to_type_node_function_type() {
        assert_var_type_round_trips("let x: (a: number) => string = (a) => \"\";");
    }

    #[test]
    fn type_to_type_node_object_literal() {
        assert_var_type_round_trips("let x: { a: number; b: string } = { a: 1, b: \"\" };");
    }

    #[test]
    fn type_to_type_node_string_literal_type() {
        assert_var_type_round_trips("let x: \"hello\" = \"hello\";");
    }

    #[test]
    fn type_to_type_node_numeric_literal_type() {
        assert_var_type_round_trips("let x: 42 = 42;");
    }

    use crate::ast::node_data_generated::for_each_child;

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

    fn parts_text(parts: &[SymbolDisplayPart]) -> String {
        parts.iter().map(|p| p.text.as_str()).collect()
    }

    #[test]
    fn display_parts_function() {
        let parts = display_parts_for("function foo(x: number): string { return \"\"; }", "foo");

        assert_eq!(parts_text(&parts), "function foo(x: number): string");

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

        assert_eq!(parts.last().unwrap().kind, DisplayPartKind::Keyword);
    }

    #[test]
    fn display_parts_type_alias_with_type_params() {
        let parts = display_parts_for("type Id<T> = T;", "Id");
        assert!(parts_text(&parts).starts_with("type Id<T> = "));
    }

    #[test]
    fn display_parts_kind_round_trips_to_strings() {

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

        let mut checker = build_checker("class Foo {}\nlet x: Foo = new Foo();");
        let type_node = first_var_type_node(&checker);
        let t = checker.get_type_from_type_node(&type_node);
        let parts = checker.type_to_display_parts(&t);
        assert!(!parts.is_empty());
    }
}
