#![allow(unused_imports)]

use super::*;
use super::type_format_flags_2::TypeFormatFlags;

impl Checker {
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
            return self.function_symbol_display_parts(symbol, false);
        }
        if flags.intersects(SymbolFlags::Method) {
            return self.function_symbol_display_parts(symbol, true);
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

    pub(crate) fn function_symbol_display_parts(
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

    pub(crate) fn named_type_symbol_display_parts(
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

    pub(crate) fn type_alias_symbol_display_parts(&mut self, symbol: &Arc<Symbol>) -> Vec<SymbolDisplayPart> {
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

    pub(crate) fn type_parameter_symbol_display_parts(
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

    pub(crate) fn variable_symbol_display_parts(&mut self, symbol: &Arc<Symbol>) -> Vec<SymbolDisplayPart> {
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

    pub(crate) fn append_signature_parameter_parts(
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

    pub(crate) fn append_type_parameter_parts(
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

}
