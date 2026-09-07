#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn get_resolved_signature_for_signature_help(
        &mut self,
        node: &Arc<Node>,
        argument_count: i32,
    ) -> (Option<Arc<Signature>>, Vec<Arc<Signature>>) {
        self.get_resolved_signature_worker(node, CheckMode::IsForSignatureHelp, argument_count)
    }

    pub fn skip_alias(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if symbol.flags.contains(SymbolFlags::Alias) {
            return self.get_aliased_symbol(symbol);
        }
        Arc::clone(symbol)
    }

    pub fn get_aliased_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        if let Some(links) = self.alias_symbol_links.get(symbol) {
            if let Some(ref target) = links.alias_target {
                return Arc::clone(target);
            }
            if let Some(ref target) = links.immediate_target {
                return Arc::clone(target);
            }
        }
        Arc::clone(symbol)
    }

    pub fn get_root_symbols(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        let roots = self.get_immediate_root_symbols(symbol);
        if roots.is_empty() {
            return vec![Arc::clone(symbol)];
        }
        let mut result = Vec::new();
        for root in &roots {
            result.extend(self.get_root_symbols(root));
        }
        result
    }

    pub fn get_immediate_root_symbols(&self, symbol: &Arc<Symbol>) -> Vec<Arc<Symbol>> {
        if symbol.check_flags.intersects(CheckFlags::SYNTHETIC) {
            if let Some(links) = self.value_symbol_links.get(symbol) {
                if let Some(ref containing) = links.containing_type {
                    let types = containing.types().unwrap_or(&[]);
                    let mut result = Vec::new();
                    for t in types {
                        if let Some(prop) = self.get_property_of_type_cached(t, &symbol.name) {
                            result.push(prop);
                        }
                    }
                    return result;
                }
            }
        }
        if symbol.flags.contains(SymbolFlags::Transient) {
            if let Some(links) = self.spread_links.get(symbol) {
                if links.left_spread.is_some() {
                    let mut result = Vec::new();
                    if let Some(ref left) = links.left_spread {
                        result.push(Arc::clone(left));
                    }
                    if let Some(ref right) = links.right_spread {
                        result.push(Arc::clone(right));
                    }
                    return result;
                }
            }
            if let Some(links) = self.mapped_symbol_links.get(symbol) {
                if let Some(ref origin) = links.synthetic_origin {
                    return vec![Arc::clone(origin)];
                }
            }
            let target = self.try_get_target(symbol);
            if let Some(target) = target {
                return vec![target];
            }
        }
        Vec::new()
    }

    pub fn try_get_target(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        let mut target = None;
        let mut next = Arc::clone(symbol);
        loop {
            let resolved = if let Some(links) = self.value_symbol_links.get(&next) {
                links.target.clone()
            } else if let Some(links) = self.export_type_links.get(&next) {
                links.target.clone()
            } else {
                None
            };
            match resolved {
                Some(n) => {
                    target = Some(Arc::clone(&n));
                    next = n;
                }
                None => break,
            }
        }
        target
    }

    pub fn get_mapped_type_symbol_of_property(&self, symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
        if let Some(value_links) = self.value_symbol_links.get(symbol) {
            if let Some(ref containing) = value_links.containing_type {
                return containing.symbol.clone();
            }
        }
        None
    }

    pub fn get_export_symbol_of_symbol(&self, symbol: &Arc<Symbol>) -> Arc<Symbol> {
        let source = if let Some(ref export) = symbol.export_symbol {
            Arc::clone(export)
        } else {
            Arc::clone(symbol)
        };

        source
    }

    pub fn get_export_specifier_local_target_symbol(
        &mut self,
        node: &Arc<Node>,
    ) -> Option<Arc<Symbol>> {
        match node.kind {
            SyntaxKind::ExportSpecifier => None,
            SyntaxKind::Identifier => None,
            _ => {
                panic!(
                    "Unhandled case in getExportSpecifierLocalTargetSymbol, node should be ExportSpecifier | Identifier"
                )
            }
        }
    }

    pub fn get_shorthand_assignment_value_symbol(
        &mut self,
        location: Option<&Arc<Node>>,
    ) -> Option<Arc<Symbol>> {
        if let Some(loc) = location {
            if loc.kind == SyntaxKind::ShorthandPropertyAssignment {
                if let Some(name) = loc.name() {
                    return None;
                }
            }
        }
        None
    }

    pub fn get_symbols_of_parameter_property_declaration(
        &self,
        parameter: &Arc<Node>,
        parameter_name: &str,
    ) -> Option<(Arc<Symbol>, Arc<Symbol>)> {
        let constructor_declaration = parameter.parent.as_ref()?;
        let class_declaration = constructor_declaration.parent.as_ref()?;

        let _ = parameter_name;
        let _ = class_declaration;
        None
    }

    pub fn is_declaration_used(
        &mut self,
        source_file: &Arc<SourceFile>,
        identifier: &Arc<Node>,
        jsx_elements_present: bool,
        jsx_mode_needs_explicit_import: bool,
    ) -> bool {
        if jsx_elements_present && jsx_mode_needs_explicit_import {
            let identifier_text = identifier.text();

            if identifier_text == "React" || identifier_text == "h" {
                return true;
            }
        }

        let symbol = self.get_symbol_at_location(identifier);
        let symbol = match symbol {
            Some(s) => s,
            None => return true,
        };

        self.is_symbol_referenced_in_file(source_file, identifier, &symbol)
    }

    pub fn is_symbol_referenced_in_file(
        &mut self,
        source_file: &Arc<SourceFile>,
        definition: &Arc<Node>,
        symbol: &Arc<Symbol>,
    ) -> bool {
        let identifier_text = definition.text();
        for token in get_possible_symbol_reference_nodes(source_file, identifier_text, None) {
            if token.kind != SyntaxKind::Identifier {
                continue;
            }
            if token.text() != identifier_text {
                continue;
            }
            if Arc::ptr_eq(&token, definition) {
                continue;
            }
            let ref_symbol = self.get_symbol_at_location(&token);
            if let Some(ref ref_sym) = ref_symbol {
                if Arc::ptr_eq(ref_sym, symbol) {
                    return true;
                }
            }
            if let Some(ref parent) = token.parent {
                if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                    let shorthand_symbol = self.get_shorthand_assignment_value_symbol(Some(parent));
                    if let Some(ref shorthand) = shorthand_symbol {
                        if Arc::ptr_eq(shorthand, symbol) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn get_references_to_symbol_in_file(
        &mut self,
        source_file: &Arc<SourceFile>,
        symbol: &Arc<Symbol>,
    ) -> Vec<Arc<Node>> {
        let identifier_text = &symbol.name;
        let mut result = Vec::new();
        for token in get_possible_symbol_reference_nodes(source_file, identifier_text, None) {
            if token.kind != SyntaxKind::Identifier {
                continue;
            }
            if token.text() != identifier_text.as_str() {
                continue;
            }
            let ref_symbol = self.get_symbol_at_location(&token);
            if let Some(ref ref_sym) = ref_symbol {
                if Arc::ptr_eq(ref_sym, symbol) {
                    result.push(Arc::clone(&token));
                    continue;
                }
            }
            if let Some(ref parent) = token.parent {
                if parent.kind == SyntaxKind::ShorthandPropertyAssignment {
                    let shorthand_symbol = self.get_shorthand_assignment_value_symbol(Some(parent));
                    if let Some(ref shorthand) = shorthand_symbol {
                        if Arc::ptr_eq(shorthand, symbol) {
                            result.push(Arc::clone(&token));
                            continue;
                        }
                    }
                }
            }
        }
        result
    }

    pub fn get_type_argument_constraint(&mut self, node: &Arc<Node>) -> Option<Arc<Type>> {
        None
    }

}
