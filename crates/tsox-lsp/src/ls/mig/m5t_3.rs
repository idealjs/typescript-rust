#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{Definition, DefinitionKind, EntryKind, ReferenceEntry, RefOptions, ReferenceUse, SymbolAndEntries};
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};

use tsox_frontend::ast::mig::m3e_4::SemanticMeaning;

use super::m5s_3::{ImpExpKind, RefSearch, RefState};
use super::m5t::{get_symbol_scope, is_for_rename_with_prefix_and_suffix_text, is_valid_reference_position, new_node_entry, new_node_entry_with_kind};
use super::m5t_4::get_special_search_kind;

pub fn state_special_search_kind(state: &RefState) -> &'static str { ::tsox_core::fntrace::enter("state_special_search_kind"); 
    get_special_search_kind(Some(&state.node))
}

pub fn new_state<'a>(
    program: &Arc<tsox_compile::compiler::Program>,
    source_files: &[Arc<SourceFile>],
    source_files_set: &std::collections::HashSet<String>,
    node: Option<&Arc<Node>>,
    checker: &'a mut tsox_checker::checker::Checker,
    search_meaning: SemanticMeaning,
    options: RefOptions,
) -> RefState<'a> { ::tsox_core::fntrace::enter("new_state"); 
    RefState {
        program: program.clone(),
        source_files: source_files.to_vec(),
        source_files_set: source_files_set.clone(),
        node: node.cloned().unwrap_or_else(|| program.get_source_files()[0].node.clone()),
        checker,
        search_meaning,
        options,
        seen_imported_symbols: Default::default(),
        seen_export_symbols: Default::default(),
        seenReExportRHS: Default::default(),
        seen_contains_symbol: Default::default(),
        seen_containing_type_references: Default::default(),
        pending_entries: Vec::new(),
    }
}

impl<'a> RefState<'a> {
    pub fn includes_source_file(&self, source_file: &Arc<SourceFile>) -> bool { ::tsox_core::fntrace::enter("includes_source_file"); 
        self.source_files_set.contains(&source_file.file_name)
    }

    pub fn mark_searched_symbols(&mut self, source_file: &Arc<SourceFile>, symbols: &[Arc<Symbol>]) -> bool { ::tsox_core::fntrace::enter("mark_searched_symbols"); 
        let file_key = Arc::as_ptr(source_file) as usize;
        let mut any_new_symbols = false;
        for sym in symbols {
            if self.seen_contains_symbol.insert((file_key, Arc::as_ptr(sym) as usize)) {
                any_new_symbols = true;
            }
        }
        any_new_symbols
    }

    pub fn mark_seen_re_export_rhs(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("mark_seen_re_export_rhs"); 
        self.seenReExportRHS.insert(Arc::as_ptr(node) as usize)
    }

    pub fn seen_containing_type_references_add_if_absent(&mut self, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("seen_containing_type_references_add_if_absent"); 
        self.seen_containing_type_references.insert(Arc::as_ptr(node) as usize)
    }

    pub fn reference_adder(&mut self, search_symbol: &Arc<Symbol>) -> impl FnMut(&Arc<Node>, EntryKind) + '_ { ::tsox_core::fntrace::enter("reference_adder"); 
        self.pending_entries.push(ReferenceEntry {
            kind: EntryKind::None,
            node: None,
            context: None,
            file_name: String::new(),
            text_range: None,
            lsp_range: None,
        });
        let symbol = search_symbol.clone();
        move |node: &Arc<Node>, kind: EntryKind| {
            let _ = &symbol;
            let _ = node;
            let _ = kind;
        }
    }

    pub fn populate_search_symbol_set(
        &mut self,
        symbol: &Arc<Symbol>,
        location: Option<&Arc<Node>>,
        is_for_rename: bool,
        provide_prefix_and_suffix_text: bool,
        implementations: bool,
    ) -> Vec<Arc<Symbol>> { ::tsox_core::fntrace::enter("populate_search_symbol_set"); 
        let Some(_location) = location else {
            return vec![symbol.clone()];
        };
        let mut result: Vec<Arc<Symbol>> = Vec::new();
        self.for_each_related_symbol(
            symbol,
            location.unwrap(),
            is_for_rename,
            !(is_for_rename && provide_prefix_and_suffix_text),
            &mut |sym, root, base| {
                let mut base = base.cloned();
                if let Some(b) = &base {
                    if super::m5x_3::is_static_symbol(symbol) != super::m5x_3::is_static_symbol(b) {
                        base = None;
                    }
                }
                result.push(
                    base.or_else(|| root.cloned())
                        .unwrap_or_else(|| sym.clone()),
                );
                None
            },
            &|_root| !implementations,
        );
        result
    }

    pub fn get_related_symbol(&mut self, search: &RefSearch, reference_symbol: &Arc<Symbol>, reference_location: &Arc<Node>) -> Option<(Arc<Symbol>, EntryKind)> { ::tsox_core::fntrace::enter("get_related_symbol"); 
        self.for_each_related_symbol(
            reference_symbol,
            reference_location,
            false,
            self.options.use_ != ReferenceUse::Rename || self.options.use_aliases_for_rename,
            &mut |sym, root_symbol, base_symbol| {
                let mut base_symbol = base_symbol.cloned();
                if let Some(base) = &base_symbol {
                    if super::m5x_3::is_static_symbol(reference_symbol) != super::m5x_3::is_static_symbol(base) {
                        base_symbol = None;
                    }
                }
                let search_sym = base_symbol.or_else(|| root_symbol.cloned()).unwrap_or_else(|| sym.clone());
                if search.all_search_symbols.iter().any(|s| Arc::ptr_eq(s, &search_sym)) {
                    if let Some(root) = root_symbol {
                        if !sym.check_flags.intersects(ast::CheckFlags::SYNTHETIC) {
                            return Some(root.clone());
                        }
                    }
                    return Some(sym.clone());
                }
                None
            },
            &|_root| true,
        )
    }

    pub fn for_each_related_symbol(
        &mut self,
        symbol: &Arc<Symbol>,
        location: &Arc<Node>,
        is_for_rename_populate_search_symbol_set: bool,
        only_include_binding_element_at_reference_location: bool,
        cb_symbol: &mut dyn FnMut(&Arc<Symbol>, Option<&Arc<Symbol>>, Option<&Arc<Symbol>>) -> Option<Arc<Symbol>>,
        allow_base_types: &dyn Fn(&Arc<Symbol>) -> bool,
    ) -> Option<(Arc<Symbol>, EntryKind)> { ::tsox_core::fntrace::enter("for_each_related_symbol"); 
        if let Some(res) = self.from_root(symbol, cb_symbol, allow_base_types) {
            return Some((res, EntryKind::Node));
        }
        if let Some(aliased) = super::m5t_4::get_merged_aliased_symbol_of_namespace_export_declaration(location, symbol, &self.checker) {
            if let Some(res) = cb_symbol(&aliased, None, None) {
                return Some((res, EntryKind::Node));
            }
        }
        if let Some(value_declaration) = symbol.value_declaration.clone() {
            if let Some(vd_parent) = value_declaration.parent() {
                if tsox_frontend::ast::mig::m3g_2::is_parameter_property_declaration(&value_declaration, &vd_parent) {
                    return None;
                }
            }
        }
        if let Some(export_specifier) = tsox_frontend::ast::mig::m3e_4::get_declaration_of_kind(symbol, SyntaxKind::ExportSpecifier) {
            let has_property_name = export_specifier.property_name().is_some();
            if !is_for_rename_populate_search_symbol_set || !has_property_name {
                if let Some(local_symbol) = self.checker.get_export_specifier_local_target_symbol(&export_specifier) {
                    if let Some(res) = cb_symbol(&local_symbol, None, None) {
                        return Some((res, EntryKind::Node));
                    }
                }
            }
        }
        if !is_for_rename_populate_search_symbol_set {
            if only_include_binding_element_at_reference_location {
                let Some(location_parent) = location.parent() else {
                    return None;
                };
                if super::m5x_3::is_object_binding_element_without_property_name(&location_parent) {
                    return None;
                }
                let property_symbol = super::m5x_5::get_property_symbol_from_binding_element(&mut self.checker, &location_parent)?;
                return self.from_root(&property_symbol, cb_symbol, allow_base_types).map(|s| (s, EntryKind::SearchedPropertyFoundLocal));
            }
            let binding_element_property_symbol = super::m5x_5::get_property_symbol_of_object_binding_pattern_without_property_name(symbol, &mut self.checker);
            if let Some(property_symbol) = binding_element_property_symbol {
                return self.from_root(&property_symbol, cb_symbol, allow_base_types).map(|s| (s, EntryKind::SearchedPropertyFoundLocal));
            }
        }
        None
    }

    fn from_root(
        &mut self,
        sym: &Arc<Symbol>,
        cb_symbol: &mut dyn FnMut(&Arc<Symbol>, Option<&Arc<Symbol>>, Option<&Arc<Symbol>>) -> Option<Arc<Symbol>>,
        allow_base_types: &dyn Fn(&Arc<Symbol>) -> bool,
    ) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("from_root"); 
        for root_symbol in self.checker.get_root_symbols(sym) {
            if let Some(result) = cb_symbol(sym, Some(&root_symbol), None) {
                return Some(result);
            }
            if let Some(parent) = root_symbol.parent() {
                if parent.flags.intersects(ast::SymbolFlags::Class | ast::SymbolFlags::Interface) && allow_base_types(&root_symbol) {
                    let result = super::m5x_5::get_property_symbols_from_base_types(&parent, root_symbol.name.as_str(), &mut self.checker, &mut |base| cb_symbol(sym, Some(&root_symbol), Some(base)));
                    if let Some(result) = result {
                        return Some(result);
                    }
                }
            }
        }
        None
    }

    pub fn has_matching_meaning(&self, reference_location: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_matching_meaning"); 
        crate::ls::utilities::get_meaning_from_location(reference_location) & self.search_meaning.0 != 0
    }

    pub fn search_for_imported_symbol(&mut self, symbol: &Arc<Symbol>) { ::tsox_core::fntrace::enter("search_for_imported_symbol"); 
        let declarations = symbol.declarations.clone();
        for declaration in declarations {
            let Some(exporting_file) = source_file_of_node(&self.program, &declaration) else {
                continue;
            };
            let search = self.create_search(&declaration, symbol, ImpExpKind::Import, "", Vec::new());
            let add_refs_here = self.includes_source_file(&exporting_file);
            self.get_references_in_source_file(&exporting_file, &search, add_refs_here);
        }
    }

    pub fn search_for_imports_of_export(&mut self, export_location: &Arc<Node>, export_symbol: &Arc<Symbol>, export_info: &crate::ls::import_tracker::ExportInfo) { ::tsox_core::fntrace::enter("search_for_imports_of_export"); 
        let Some(r) = self.get_import_searches(export_symbol, export_info) else {
            return;
        };

        if !r.single_references.is_empty() {
            for single_ref in &r.single_references {
                if self.should_add_single_reference(single_ref) {
                    self.add_reference(single_ref.clone(), Some(export_symbol.clone()), EntryKind::Node);
                }
            }
        }

        for i in &r.import_searches {
            let Some(import_location) = i.import_location.as_ref() else {
                continue;
            };
            let Some(import_symbol) = i.import_symbol.as_ref() else {
                continue;
            };
            let Some(import_file) = source_file_of_node(&self.program, import_location) else {
                continue;
            };
            let search = self.create_search(import_location, import_symbol, ImpExpKind::Export, "", Vec::new());
            self.get_references_in_source_file(&import_file, &search, true);
        }

        if !r.indirect_users.is_empty() {
            let indirect_search = match export_info.export_kind {
                crate::ls::import_tracker::ExportKind::Named => {
                    Some(self.create_search(export_location, export_symbol, ImpExpKind::Export, "", Vec::new()))
                }
                crate::ls::import_tracker::ExportKind::Default => {
                    if self.options.use_ != ReferenceUse::Rename {
                        Some(self.create_search(export_location, export_symbol, ImpExpKind::Export, "default", Vec::new()))
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(indirect_search) = indirect_search {
                for indirect_user in &r.indirect_users {
                    self.search_for_name(indirect_user, &indirect_search);
                }
            }
        }
    }

    pub fn should_add_single_reference(&self, single_ref: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("should_add_single_reference"); 
        if !self.has_matching_meaning(single_ref) {
            return false;
        }
        if self.options.use_ != ReferenceUse::Rename {
            return true;
        }
        let parent = single_ref.parent();
        let is_specifier = parent.as_ref().is_some_and(|p| ast::is_import_or_export_specifier(p));
        if !ast::is_identifier(single_ref) && !is_specifier {
            return false;
        }
        !(is_specifier && tsox_frontend::ast::mig::m3g_3::module_export_name_is_default(single_ref))
    }

    pub fn search_for_name(&mut self, source_file: &Arc<SourceFile>, search: &RefSearch) { ::tsox_core::fntrace::enter("search_for_name"); 
        if tsox_frontend::ast::mig::m3b::get_name_table(source_file).contains_key(search.text.as_str()) {
            self.get_references_in_source_file(source_file, search, true);
        }
    }

    pub fn get_references_in_container_or_files(&mut self, symbol: &Arc<Symbol>, search: &RefSearch) { ::tsox_core::fntrace::enter("get_references_in_container_or_files"); 
        let scope = get_symbol_scope(symbol, &self.checker);
        if let Some(scope) = scope {
            let add_references_here = scope.kind != SyntaxKind::SourceFile
                || self.source_files.iter().any(|f| ast::get_source_file_of_node(&scope).map(|sf| Arc::ptr_eq(&f.node, &sf)).unwrap_or(false));
            if let Some(scope_file) = source_file_of_node(&self.program, &scope) {
                self.get_references_in_container(&scope, &scope_file, search, add_references_here);
            }
        } else {
            let source_files = self.source_files.clone();
            for source_file in &source_files {
                self.search_for_name(source_file, search);
            }
        }
    }

    pub fn get_references_in_source_file(&mut self, source_file: &Arc<SourceFile>, search: &RefSearch, add_references_here: bool) { ::tsox_core::fntrace::enter("get_references_in_source_file"); 
        let file_node = source_file.node.clone();
        self.get_references_in_container(&file_node, source_file, search, add_references_here);
    }

    pub fn get_references_in_container(&mut self, container: &Arc<Node>, source_file: &Arc<SourceFile>, search: &RefSearch, add_references_here: bool) { ::tsox_core::fntrace::enter("get_references_in_container"); 
        if !self.mark_searched_symbols(source_file, &search.all_search_symbols) {
            return;
        }
        let positions = super::m5t_4::get_possible_symbol_reference_positions(source_file, &search.text, Some(container));
        for position in positions {
            self.get_references_at_location(source_file, position, search, add_references_here);
        }
    }
}

pub fn source_file_of_node(program: &Arc<tsox_compile::compiler::Program>, node: &Arc<Node>) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_of_node"); 
    let file_node = ast::get_source_file_of_node(node)?;
    program
        .source_files()
        .iter()
        .find(|f| Arc::ptr_eq(&f.node, &file_node))
        .cloned()
}
