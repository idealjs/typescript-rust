#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{Definition, DefinitionKind, EntryKind, ReferenceEntry, RefOptions, ReferenceUse, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::astnav;
use tsox_frontend::scanner;

use super::m5s_3::{ImpExpKind, RefSearch, RefState};
use super::m5t::{is_for_rename_with_prefix_and_suffix_text, is_valid_reference_position, new_node_entry, new_node_entry_with_kind, skip_past_export_or_import_specifier_or_union};
use tsox_frontend::ast::mig::m3e_4::SemanticMeaning;
use tsox_frontend::ast::mig::m3b as node_accessors;

fn to_import_tracker_export_info(info: super::m5u::M5uExportInfo) -> crate::ls::import_tracker::ExportInfo { ::tsox_core::fntrace::enter("to_import_tracker_export_info"); 
    crate::ls::import_tracker::ExportInfo {
        exporting_module_symbol: Some(info.exporting_module_symbol),
        export_kind: match info.export_kind {
            super::m5u::M5uExportKind::Named => crate::ls::import_tracker::ExportKind::Named,
            super::m5u::M5uExportKind::Default => crate::ls::import_tracker::ExportKind::Default,
            super::m5u::M5uExportKind::ExportEquals => crate::ls::import_tracker::ExportKind::ExportEquals,
            super::m5u::M5uExportKind::Umd => crate::ls::import_tracker::ExportKind::Umd,
            super::m5u::M5uExportKind::Module => crate::ls::import_tracker::ExportKind::Module,
        },
    }
}

fn node_symbol_of(checker: &tsox_checker::checker::Checker, node: &Node) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("node_symbol_of"); 
    checker.program.symbol_map().symbol_of(node).cloned()
}

pub fn get_special_search_kind(node: Option<&Arc<Node>>) -> &'static str { ::tsox_core::fntrace::enter("get_special_search_kind"); 
    let Some(node) = node else {
        return "none";
    };
    match node.kind {
        SyntaxKind::Constructor | SyntaxKind::ConstructorKeyword => "constructor",
        SyntaxKind::Identifier => {
            node.parent().map(|p| ast::is_class_like(&p)).unwrap_or(false)
                .then_some("class")
                .unwrap_or("none")
        }
        _ => "none",
    }
}

pub fn get_merged_aliased_symbol_of_namespace_export_declaration(node: &Arc<Node>, symbol: &Arc<Symbol>, checker: &tsox_checker::checker::Checker) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_merged_aliased_symbol_of_namespace_export_declaration"); 
    let parent = node.parent()?;
    if parent.kind != SyntaxKind::NamespaceExportDeclaration {
        return None;
    }
    let aliased_symbol = checker.get_merged_symbol(symbol);
    let target_symbol = checker.get_merged_symbol(&aliased_symbol);
    if !Arc::ptr_eq(&aliased_symbol, &target_symbol) {
        return Some(target_symbol);
    }
    None
}

pub fn get_possible_symbol_reference_nodes(source_file: &Arc<SourceFile>, symbol_name: &str, container: Option<&Arc<Node>>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_possible_symbol_reference_nodes"); 
    let container = container.cloned().unwrap_or_else(|| source_file.node.clone());
    get_possible_symbol_reference_positions(source_file, symbol_name, Some(&container))
        .into_iter()
        .filter_map(|pos| {
            let reference_location = astnav::get_touching_property_name(&source_file.node, pos)?;
            if !Arc::ptr_eq(&reference_location, &source_file.node) {
                Some(reference_location)
            } else {
                None
            }
        })
        .collect()
}

pub fn get_possible_symbol_reference_positions(source_file: &Arc<SourceFile>, symbol_name: &str, container: Option<&Arc<Node>>) -> Vec<usize> { ::tsox_core::fntrace::enter("get_possible_symbol_reference_positions"); 
    let mut positions: Vec<usize> = Vec::new();
    if symbol_name.is_empty() {
        return positions;
    }
    let text = &source_file.text;
    let source_length = text.len();
    let symbol_name_length = symbol_name.len();
    let container = container.cloned().unwrap_or_else(|| source_file.node.clone());

    let mut position = text[container.pos() as usize..].find(symbol_name).map(|i| container.pos() as usize + i);
    let end_pos = container.end() as usize;
    while let Some(pos) = position {
        if pos >= end_pos {
            break;
        }
        let end_position = pos + symbol_name_length;
        let before_ok = pos == 0 || !scanner::is_identifier_part(text.as_bytes()[pos - 1] as char);
        let after_ok = end_position == source_length || !scanner::is_identifier_part(text.as_bytes()[end_position] as char);
        if before_ok && after_ok {
            positions.push(pos);
        }
        let start_index = pos + symbol_name_length + 1;
        if start_index > text.len() {
            break;
        }
        match text[start_index..].find(symbol_name) {
            Some(found_index) => position = Some(start_index + found_index),
            None => break,
        }
    }
    positions
}

pub fn get_references_for_non_module(_referenced_file: &Arc<SourceFile>, _program: &Arc<tsox_compile::compiler::Program>) -> Vec<ReferenceEntry> { ::tsox_core::fntrace::enter("get_references_for_non_module"); 
    Vec::new()
}

pub fn is_method_or_accessor(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_method_or_accessor"); 
    matches!(node.kind, SyntaxKind::MethodDeclaration | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor)
}

pub fn try_get_class_by_extending_identifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_class_by_extending_identifier"); 
    let parent = tsox_frontend::ast::mig::m3h::climb_past_property_access(node).parent()?;
    tsox_frontend::ast::mig::m3g_3::try_get_class_extending_expression_with_type_arguments(&parent)
}

pub fn has_own_constructor(class_declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_own_constructor"); 
    let Some(class_symbol) =
        tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node(class_declaration)
    else {
        return false;
    };
    class_symbol.members.get(ast::INTERNAL_SYMBOL_NAME_CONSTRUCTOR).is_some()
}

pub fn get_reference_entries_for_shorthand_property_assignment(node: &Arc<Node>, checker: &mut tsox_checker::checker::Checker, add_reference: &mut impl FnMut(&Arc<Node>)) { ::tsox_core::fntrace::enter("get_reference_entries_for_shorthand_property_assignment"); 
    let Some(ref_symbol) = checker.get_symbol_at_location(node) else {
        return;
    };
    let Some(value_declaration) = ref_symbol.value_declaration.clone() else {
        return;
    };
    let Some(shorthand_symbol) = checker.get_shorthand_assignment_value_symbol(Some(&value_declaration)) else {
        return;
    };
    if !shorthand_symbol.declarations.is_empty() {
        for declaration in &shorthand_symbol.declarations {
            if get_meaning_from_declaration(declaration).0 & SemanticMeaning::VALUE.0 != 0 {
                add_reference(declaration);
            }
        }
    }
}

fn get_meaning_from_declaration(node: &Arc<Node>) -> SemanticMeaning { ::tsox_core::fntrace::enter("get_meaning_from_declaration"); 
    tsox_frontend::ast::mig::m3e_4::get_meaning_from_declaration(node)
}

impl<'a> RefState<'a> {
    pub fn get_references_at_location(&mut self, source_file: &Arc<SourceFile>, position: usize, search: &RefSearch, add_references_here: bool) { ::tsox_core::fntrace::enter("get_references_at_location"); 
        let Some(reference_location) = astnav::get_touching_property_name(&source_file.node, position) else {
            return;
        };

        if !is_valid_reference_position(&reference_location, &search.text) {
            return;
        }

        if SemanticMeaning(crate::ls::utilities::get_meaning_from_location(&reference_location)).0 & self.search_meaning.0 == 0 {
            return;
        }

        let Some(mut reference_symbol) = self.checker.get_symbol_at_location(&reference_location) else {
            return;
        };

        let Some(parent) = reference_location.parent() else {
            return;
        };
        if parent.kind == SyntaxKind::ImportSpecifier
            && node_accessors::property_name(&parent).map(|p| Arc::ptr_eq(p, &reference_location)).unwrap_or(false)
        {
            return;
        }

        if parent.kind == SyntaxKind::ExportSpecifier {
            self.get_references_at_export_specifier(&reference_location, &reference_symbol, &parent, search, add_references_here, false);
            return;
        }

        let Some((related_symbol, related_symbol_kind)) = self.get_related_symbol(search, &reference_symbol, &reference_location) else {
            self.get_reference_for_shorthand_property(&reference_symbol, search);
            return;
        };

        match super::m5t_3::state_special_search_kind(self) {
            "none" => {
                if add_references_here {
                    self.add_reference(reference_location.clone(), Some(related_symbol), related_symbol_kind);
                }
            }
            "constructor" | "class" => {
                if add_references_here {
                    self.add_reference(reference_location.clone(), Some(related_symbol), related_symbol_kind);
                }
            }
            _ => {}
        }

        if ast::is_in_js_file(&reference_location)
            && reference_location.parent().map(|p| p.kind) == Some(SyntaxKind::BindingElement)
            && reference_location
                .parent()
                .as_ref()
                .and_then(|p| p.parent())
                .as_ref()
                .and_then(|p| p.parent())
                .map(|p| tsox_frontend::ast::mig::m3g_3::is_variable_declaration_initialized_to_bare_or_accessed_require(&p))
                .unwrap_or(false)
        {
            let Some(js_parent) = reference_location.parent() else {
                return;
            };
            let Some(parent_symbol) = node_symbol_of(&self.checker, &js_parent) else {
                return;
            };
            reference_symbol = parent_symbol;
        }

        self.get_import_or_export_references(&reference_location, &reference_symbol, search);
    }

    pub fn get_references_at_export_specifier(
        &mut self,
        reference_location: &Arc<Node>,
        reference_symbol: &Arc<Symbol>,
        export_specifier: &Arc<Node>,
        search: &RefSearch,
        add_references_here: bool,
        always_get_references: bool,
    ) { ::tsox_core::fntrace::enter("get_references_at_export_specifier"); 
        let Some(export_declaration) = export_specifier.parent().and_then(|p| p.parent()) else {
            return;
        };
        let property_name = node_accessors::property_name(export_specifier);
        let name = export_specifier.name();
        let local_symbol = Some(super::m5x_5::get_local_symbol_for_export_specifier(reference_location, reference_symbol, export_specifier, &mut self.checker));

        if !always_get_references && !search.all_search_symbols.iter().any(|s| local_symbol.as_ref().map(|l| Arc::ptr_eq(s, l)).unwrap_or(false)) {
            return;
        }

        let mut add_ref = |state: &mut RefState<'_>| {
            if add_references_here {
                if let Some(local_symbol) = &local_symbol {
                    state.add_reference(reference_location.clone(), Some(local_symbol.clone()), EntryKind::Node);
                }
            }
        };

        if property_name.is_none() {
            if !(self.options.use_ == ReferenceUse::Rename && name.map(|n| tsox_frontend::ast::mig::m3g_3::module_export_name_is_default(n)).unwrap_or(false)) {
                add_ref(self);
            }
        } else if Arc::ptr_eq(reference_location, property_name.unwrap()) {
            if !has_module_specifier(&export_declaration) {
                add_ref(self);
            }
            if add_references_here && self.options.use_ != ReferenceUse::Rename && name.map(|n| self.mark_seen_re_export_rhs(n)).unwrap_or(false) {
                if let Some(export_symbol) = node_symbol_of(&self.checker, export_specifier) {
                    if let Some(name) = name {
                        self.add_reference(name.clone(), Some(export_symbol), EntryKind::Node);
                    }
                }
            }
        } else {
            if self.mark_seen_re_export_rhs(reference_location) {
                add_ref(self);
            }
        }

        if !is_for_rename_with_prefix_and_suffix_text(&self.options) || always_get_references {
            let name_is_default = name.map(|n| tsox_frontend::ast::mig::m3g_3::module_export_name_is_default(n)).unwrap_or(false);
            let is_default_export = tsox_frontend::ast::mig::m3g_3::module_export_name_is_default(reference_location) || name_is_default;
            let export_kind = if is_default_export {
                super::m5u::M5uExportKind::Default
            } else {
                super::m5u::M5uExportKind::Named
            };
            if let Some(export_symbol) = node_symbol_of(&self.checker, export_specifier) {
                if let Some(export_info) = super::m5u::get_export_info(&export_symbol, export_kind, &self.checker) {
                    let export_info = to_import_tracker_export_info(export_info);
                    self.search_for_imports_of_export(reference_location, &export_symbol, &export_info);
                }
            }
        }

        if search.coming_from != ImpExpKind::Export && has_module_specifier(&export_declaration) && property_name.is_none() && !is_for_rename_with_prefix_and_suffix_text(&self.options) {
            if let Some(imported) = self.checker.get_export_specifier_local_target_symbol(export_specifier) {
                self.search_for_imported_symbol(&imported);
            }
        }
    }

    pub fn get_reference_for_shorthand_property(&mut self, reference_symbol: &Arc<Symbol>, search: &RefSearch) { ::tsox_core::fntrace::enter("get_reference_for_shorthand_property"); 
        if reference_symbol.flags.intersects(ast::SymbolFlags::Transient) || reference_symbol.value_declaration.is_none() {
            return;
        }
        let value_declaration = reference_symbol.value_declaration.clone().unwrap();
        let shorthand_value_symbol = self.checker.get_shorthand_assignment_value_symbol(Some(&value_declaration));
        let name = ast::get_name_of_declaration(&value_declaration);

        if let Some(name) = name {
            if let Some(shorthand_value_symbol) = &shorthand_value_symbol {
                if search.all_search_symbols.iter().any(|s| Arc::ptr_eq(s, shorthand_value_symbol)) {
                    self.add_reference(name, Some(shorthand_value_symbol.clone()), EntryKind::Node);
                }
            }
        }
    }

    pub fn get_import_or_export_references(&mut self, reference_location: &Arc<Node>, reference_symbol: &Arc<Symbol>, search: &RefSearch) { ::tsox_core::fntrace::enter("get_import_or_export_references"); 
        let import_or_export = super::m5u::get_import_or_export_symbol(reference_location, reference_symbol, &mut self.checker, search.coming_from == ImpExpKind::Export);
        let Some(import_or_export) = import_or_export else {
            return;
        };
        if import_or_export.kind == super::m5u::M5uImpExpKind::Import {
            if !is_for_rename_with_prefix_and_suffix_text(&self.options) {
                if let Some(symbol) = &import_or_export.symbol {
                    self.search_for_imported_symbol(symbol);
                }
            }
        } else {
            let export_info = match import_or_export.export_info {
                Some(info) => to_import_tracker_export_info(info),
                None => return,
            };
            match import_or_export.symbol {
                Some(symbol) => self.search_for_imports_of_export(reference_location, &symbol, &export_info),
                None => return,
            }
        }
    }
}

fn has_module_specifier(export_declaration: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("has_module_specifier"); 
    node_accessors::module_specifier(export_declaration).is_some()
}

pub struct ReferencedSymbolDefinitionInfo {
    pub node: Arc<Node>,
    pub location: crate::lsp::lsproto_lsp::Location,
    pub display_text: String,
}

pub fn definition_to_referenced_symbol_definition_info(
    l: &LanguageService,
    def: &Definition,
    original_node: &Arc<Node>,
    feature: crate::ls::mig::m5s::SpanFeature,
) -> Option<ReferencedSymbolDefinitionInfo> { ::tsox_core::fntrace::enter("definition_to_referenced_symbol_definition_info"); 
    let node = def.node.clone()?;
    let file_node = ast::get_source_file_of_node(&node)?;
    let program = l.get_program();
    let file = program.get_source_files().into_iter().find(|f| f.node.id() == file_node.id())?;
    let name = ast::get_name_of_declaration(&node).unwrap_or_else(|| node.clone());
    let text_range = super::m5t::get_range_of_node(&name, Some(&file), None);
    let (location, fidelity) = l.source_file_range_to_lsp_location_for_feature(&file, text_range, feature);
    if fidelity == super::m5s::SpanFidelity::None_ {
        return None;
    }
    let display_text = name.text().to_string();
    Some(ReferencedSymbolDefinitionInfo {
        node,
        location,
        display_text,
    })
}
