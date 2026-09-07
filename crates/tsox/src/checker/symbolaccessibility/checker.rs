#![allow(unused_imports)]

use super::*;

impl Checker {
    pub fn is_type_symbol_accessible(
        &mut self,
        type_symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool {
        let access = self.is_symbol_accessible_worker(
            type_symbol,
            enclosing_declaration,
            SymbolFlags::TYPE,
            false,
            true,
        );
        access.accessibility == SymbolAccessibility::Accessible
    }

    pub fn is_value_symbol_accessible(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
    ) -> bool {
        let access = self.is_symbol_accessible_worker(
            symbol,
            enclosing_declaration,
            SymbolFlags::VALUE,
            false,
            true,
        );
        access.accessibility == SymbolAccessibility::Accessible
    }

    pub fn is_symbol_accessible_by_flags(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        flags: SymbolFlags,
    ) -> bool {
        let access =
            self.is_symbol_accessible_worker(symbol, enclosing_declaration, flags, false, false);
        access.accessibility == SymbolAccessibility::Accessible
    }

    pub fn is_any_symbol_accessible(
        &mut self,
        symbols: &[Arc<Symbol>],
        enclosing_declaration: Option<&Arc<Node>>,
        initial_symbol: &Arc<Symbol>,
        meaning: SymbolFlags,
        should_compute_aliases_to_make_visible: bool,
        allow_modules: bool,
    ) -> Option<SymbolAccessibilityResult> {
        if symbols.is_empty() {
            return None;
        }

        let mut had_accessible_chain: Option<Arc<Symbol>> = None;
        let mut early_module_bail = false;
        for symbol in symbols {
            let accessible_symbol_chain =
                self.get_accessible_symbol_chain(symbol, enclosing_declaration, meaning, false);
            if !accessible_symbol_chain.is_empty() {
                had_accessible_chain = Some(Arc::clone(symbol));

                let has_accessible_declarations = self.has_visible_declarations_with_aliases(
                    &accessible_symbol_chain[0],
                    should_compute_aliases_to_make_visible,
                );
                if let Some(result) = has_accessible_declarations {
                    return Some(result);
                }
            }
            if allow_modules {
                if symbol
                    .declarations
                    .iter()
                    .any(has_non_global_augmentation_external_module_symbol)
                {
                    if should_compute_aliases_to_make_visible {
                        early_module_bail = true;

                        continue;
                    }

                    return Some(SymbolAccessibilityResult {
                        accessibility: SymbolAccessibility::Accessible,
                        ..Default::default()
                    });
                }
            }

            let containers = self.get_containers_of_symbol(symbol, enclosing_declaration, meaning);
            let mut next_meaning = meaning;
            if initial_symbol.id() == symbol.id() {
                next_meaning = get_qualified_left_meaning(meaning);
            }
            let parent_result = self.is_any_symbol_accessible(
                &containers,
                enclosing_declaration,
                initial_symbol,
                next_meaning,
                should_compute_aliases_to_make_visible,
                allow_modules,
            );
            if let Some(result) = parent_result {
                return Some(result);
            }
        }

        if early_module_bail {
            return Some(SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::Accessible,
                ..Default::default()
            });
        }

        if let Some(ref had_chain) = had_accessible_chain {
            let mut module_name = String::new();
            if had_chain.id() != initial_symbol.id() {
                module_name = self.symbol_to_string_ex_enclosing(
                    had_chain,
                    enclosing_declaration,
                    SymbolFlags::NAMESPACE,
                    crate::checker::types::SymbolFormatFlags::AllowAnyNodeKind,
                );
            }
            return Some(SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: self.symbol_to_string_ex_enclosing(
                    initial_symbol,
                    enclosing_declaration,
                    meaning,
                    crate::checker::types::SymbolFormatFlags::AllowAnyNodeKind,
                ),
                error_module_name: module_name,
                ..Default::default()
            });
        }
        None
    }

    pub fn is_symbol_accessible(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        should_compute_aliases_to_make_visible: bool,
    ) -> SymbolAccessibilityResult {
        self.is_symbol_accessible_worker(
            symbol,
            enclosing_declaration,
            meaning,
            should_compute_aliases_to_make_visible,
            true,
        )
    }

    pub(crate) fn is_symbol_accessible_worker(
        &mut self,
        symbol: &Arc<Symbol>,
        enclosing_declaration: Option<&Arc<Node>>,
        meaning: SymbolFlags,
        should_compute_aliases_to_make_visible: bool,
        allow_modules: bool,
    ) -> SymbolAccessibilityResult {
        if let (Some(_), Some(_)) = (Some(symbol), enclosing_declaration) {
            let symbols = vec![Arc::clone(symbol)];
            if let Some(result) = self.is_any_symbol_accessible(
                &symbols,
                enclosing_declaration,
                symbol,
                meaning,
                should_compute_aliases_to_make_visible,
                allow_modules,
            ) {
                return result;
            }

            let symbol_external_module = self.get_external_module_container_of_symbol(symbol);
            if let Some(ref symbol_external_module) = symbol_external_module {
                let enclosing_external_module =
                    enclosing_declaration.and_then(|enc| self.get_external_module_container(enc));
                if symbol_external_module.id()
                    != enclosing_external_module
                        .as_ref()
                        .map(|s| s.id())
                        .unwrap_or(0)
                {
                    let error_node = if enclosing_declaration
                        .map(|n| n.flags.contains(NodeFlags::JavaScriptFile))
                        .unwrap_or(false)
                    {
                        enclosing_declaration.cloned()
                    } else {
                        None
                    };
                    return SymbolAccessibilityResult {
                        accessibility: SymbolAccessibility::CannotBeNamed,
                        error_symbol_name: self.symbol_to_string_ex_enclosing(
                            symbol,
                            enclosing_declaration,
                            meaning,
                            crate::checker::types::SymbolFormatFlags::AllowAnyNodeKind,
                        ),
                        error_module_name: self.symbol_to_string(symbol_external_module),
                        error_node,
                        ..Default::default()
                    };
                }
            }

            return SymbolAccessibilityResult {
                accessibility: SymbolAccessibility::NotAccessible,
                error_symbol_name: self.symbol_to_string_ex_enclosing(
                    symbol,
                    enclosing_declaration,
                    meaning,
                    crate::checker::types::SymbolFormatFlags::AllowAnyNodeKind,
                ),
                ..Default::default()
            };
        }

        SymbolAccessibilityResult {
            accessibility: SymbolAccessibility::Accessible,
            ..Default::default()
        }
    }
}
