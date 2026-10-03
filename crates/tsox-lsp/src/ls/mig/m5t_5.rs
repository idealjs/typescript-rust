#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{Definition, DefinitionKind, EntryKind, ReferenceEntry, RefOptions, ReferenceUse, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};

use super::m5s_3::{ImpExpKind, RefSearch, RefState};
use super::m5t::{is_for_rename_with_prefix_and_suffix_text, new_node_entry, new_node_entry_with_kind, skip_past_export_or_import_specifier_or_union};
use super::m5t_3::new_state;
use super::m5s_3::get_label_references_in_node;
use super::m5t_4::{get_merged_aliased_symbol_of_namespace_export_declaration, get_possible_symbol_reference_nodes};
use tsox_frontend::ast::mig::m3e_4::SemanticMeaning;
use tsox_frontend::ast::mig::m3c as node_accessors;

fn is_external_module_source_file(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_module_source_file"); 
    super::m5u::node_as_source_file(node)
        .map(|file| file.external_module_indicator.is_some())
        .unwrap_or(false)
}

fn get_super_container(node: &Arc<Node>, stop_on_functions: bool) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_super_container"); 
    let mut node = node.parent()?;
    loop {
        match node.kind {
            SyntaxKind::ComputedPropertyName => {
                node = node.parent()?;
                node = node.parent()?;
            }
            SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression | SyntaxKind::ArrowFunction => {
                if stop_on_functions {
                    return Some(node);
                }
                node = node.parent()?;
            }
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature | SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor | SyntaxKind::ClassStaticBlockDeclaration => {
                return Some(node);
            }
            SyntaxKind::Decorator => {
                if let Some(parent) = node.parent() {
                    if parent.kind == SyntaxKind::Parameter {
                        if let Some(grand) = parent.parent() {
                            if ast::is_class_element(&grand) {
                                node = grand;
                            }
                        }
                    } else if ast::is_class_element(&parent) {
                        node = parent;
                    }
                }
                node = node.parent()?;
            }
            _ => {
                node = node.parent()?;
            }
        }
    }
}

impl LanguageService {
    pub fn get_referenced_symbols_for_module(
        &self,
        program: &Arc<tsox_compile::compiler::Program>,
        symbol: &Arc<Symbol>,
        exclude_import_type_of_export_equals: bool,
        source_files: &[Arc<SourceFile>],
        source_files_set: &std::collections::HashSet<String>,
    ) -> Vec<SymbolAndEntries> { ::tsox_core::fntrace::enter("get_referenced_symbols_for_module"); 
        let checker = program.get_type_checker();
        let module_refs = super::m5u::find_module_references(program, source_files, symbol, &checker);
        let program_files = program.get_source_files();
        let mut references: Vec<ReferenceEntry> = Vec::new();
        for reference in &module_refs {
            match reference.kind {
                super::m5u::M5uModuleReferenceKind::Import => {
                    let Some(literal) = &reference.literal else {
                        continue;
                    };
                    let Some(parent) = literal.parent() else {
                        continue;
                    };
                    if ast::is_literal_type_node(&parent) {
                        if let Some(import_type) = parent.parent() {
                            if ast::is_import_type_node(&import_type) {
                                if exclude_import_type_of_export_equals && crate::ls::mig::m5u::import_type_qualifier(&import_type).is_none() {
                                    continue;
                                }
                            }
                        }
                    }
                    references.push(new_node_entry(literal));
                }
                super::m5u::M5uModuleReferenceKind::Implicit => {
                    let Some(referencing_file) = &reference.referencing_file else {
                        continue;
                    };
                    let mut range_node: Option<Arc<Node>> = None;
                    let is_tslib = reference.literal.as_ref().map(|l| l.text() == "tslib").unwrap_or(false);
                    if !is_tslib {
                        range_node = super::m5s2_2::find_first_jsx_node(&referencing_file.node);
                    }
                    let range_node = range_node.unwrap_or_else(|| {
                        node_accessors::statements(&referencing_file.node)
                            .first()
                            .cloned()
                            .unwrap_or_else(|| referencing_file.node.clone())
                    });
                    references.push(new_node_entry(&range_node));
                }
                super::m5u::M5uModuleReferenceKind::Reference => {
                    let file_name = reference.referencing_file.as_ref().map(|f| f.file_name.clone()).unwrap_or_default();
                    let text_range = reference.ref_directive.as_ref().map(|d| d.range);
                    references.push(ReferenceEntry {
                        kind: EntryKind::Range,
                        node: None,
                        context: None,
                        file_name,
                        text_range,
                        lsp_range: None,
                    });
                }
            }
        }

        for decl in &symbol.declarations {
            match decl.kind {
                SyntaxKind::SourceFile => continue,
                SyntaxKind::ModuleDeclaration => {
                    let Some(decl_file) = source_file_of_node(program_files.iter(), decl) else {
                        continue;
                    };
                    if source_files_set.contains(&decl_file.file_name) {
                        if let Some(name) = decl.name() {
                            references.push(new_node_entry(name));
                        }
                    }
                }
                _ => continue,
            }
        }

        let exported = symbol.exports.get(ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS).cloned();
        if let Some(exported) = exported {
            for decl in &exported.declarations {
                let Some(decl_file) = source_file_of_node(program_files.iter(), decl) else {
                    continue;
                };
                if !source_files_set.contains(&decl_file.file_name) {
                    continue;
                }
                let node = if ast::is_binary_expression(decl) && ast::is_property_access_expression(&decl.left()) {
                    decl.left().expression().cloned()
                } else if ast::is_export_assignment(decl) {
                    tsox_frontend::astnav::find_child_of_kind(decl, SyntaxKind::ExportKeyword)
                } else {
                    ast::get_name_of_declaration(decl).or(Some(decl.clone()))
                };
                if let Some(node) = node {
                    references.push(new_node_entry(&node));
                }
            }
        }

        if !references.is_empty() {
            vec![SymbolAndEntries {
                definition: Definition {
                    kind: DefinitionKind::Symbol,
                    symbol: Some(symbol.clone()),
                    node: None,
                },
                references,
            }]
        } else {
            Vec::new()
        }
    }

    pub fn get_referenced_symbols_for_module_if_declared_by_source_file(
        &self,
        symbol: &Arc<Symbol>,
        program: &Arc<tsox_compile::compiler::Program>,
        source_files: &[Arc<SourceFile>],
        checker: &mut tsox_checker::checker::Checker,
        options: &RefOptions,
        source_files_set: &std::collections::HashSet<String>,
    ) -> Option<Vec<SymbolAndEntries>> { ::tsox_core::fntrace::enter("get_referenced_symbols_for_module_if_declared_by_source_file"); 
        if !symbol.flags.intersects(ast::SymbolFlags::MODULE) || symbol.declarations.is_empty() {
            return None;
        }
        let program_files = program.get_source_files();
        let module_source_file = symbol.declarations.iter().find(|d| ast::is_source_file(d))?;
        let module_source_file = source_file_of_node(program_files.iter(), module_source_file)?;
        let module_source_file_name = module_source_file.file_name.clone();
        let export_equals = symbol.exports.get(ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS).cloned();
        let module_references = self.get_referenced_symbols_for_module(program, symbol, export_equals.is_some(), source_files, source_files_set);
        match &export_equals {
            Some(export_equals)
                if export_equals.flags.intersects(ast::SymbolFlags::Alias)
                    && source_files_set.contains(&module_source_file_name) => {}
            _ => return Some(module_references),
        }
        let aliased = checker.get_merged_symbol(export_equals.as_ref().unwrap());
        let mut merged = module_references;
        merged.extend(get_referenced_symbols_for_symbol(self, program, &aliased, None, source_files, source_files_set, checker, options));
        Some(merged)
    }

    pub fn get_references_for_string_literal(
        &self,
        node: &Arc<Node>,
        source_files: &[Arc<SourceFile>],
        checker: &mut tsox_checker::checker::Checker,
    ) -> Vec<SymbolAndEntries> { ::tsox_core::fntrace::enter("get_references_for_string_literal"); 
        let t = super::m5x_6::get_contextual_type_from_parent_or_ancestor_type_node(node, checker);
        let mut references: Vec<ReferenceEntry> = Vec::new();
        for source_file in source_files {
            let possible_references = get_possible_symbol_reference_nodes(source_file, &node.text(), None);
            for r in possible_references {
                if ast::is_string_literal_like(&r) && r.text() == node.text() {
                    if let Some(t) = &t {
                        let ref_type = super::m5x_6::get_contextual_type_from_parent_or_ancestor_type_node(&r, checker);
                        if t != &checker.get_string_type()
                            && (ref_type.as_ref().map(|rt| types_equal(checker, t, rt)).unwrap_or(false)
                                || is_string_literal_property_reference(&r, checker))
                        {
                            references.push(new_node_entry_with_kind(&r, EntryKind::StringLiteral));
                        }
                    } else {
                        if ast::is_no_substitution_template_literal(&r)
                            && !tsox_frontend::format::mig::m4t_3::range_is_on_single_line(r.loc, source_file)
                        {
                            continue;
                        }
                        references.push(new_node_entry_with_kind(&r, EntryKind::StringLiteral));
                    }
                }
            }
        }
        vec![SymbolAndEntries {
            definition: Definition {
                kind: DefinitionKind::String,
                symbol: None,
                node: Some(node.clone()),
            },
            references,
        }]
    }

    pub fn get_referenced_symbols_special(&self, node: &Arc<Node>, source_files: &[Arc<SourceFile>]) -> Option<Vec<SymbolAndEntries>> { ::tsox_core::fntrace::enter("get_referenced_symbols_special"); 
        let parent = node.parent();
        if super::m5x_3::is_type_keyword(node.kind) {
            if node.kind == SyntaxKind::VoidKeyword && parent.as_ref().map(|p| p.kind) == Some(SyntaxKind::VoidExpression) {
                return None;
            }
            if node.kind == SyntaxKind::ReadonlyKeyword && !super::m5x_3::is_readonly_type_operator(node) {
                return None;
            }
            return Some(super::m5s_3::get_all_references_for_keyword(
                source_files,
                node.kind,
                node.kind == SyntaxKind::ReadonlyKeyword,
            ));
        }
        if parent.as_ref().map(|p| tsox_frontend::ast::mig::m3e_2::is_import_meta(p)).unwrap_or(false)
            && parent.as_ref().and_then(|p| p.name()).map(|n| Arc::ptr_eq(n, node)).unwrap_or(false)
        {
            return Some(super::m5s_3::get_all_references_for_import_meta(source_files));
        }
        if node.kind == SyntaxKind::StaticKeyword && parent.as_ref().map(|p| p.kind) == Some(SyntaxKind::ClassStaticBlockDeclaration) {
            return Some(vec![SymbolAndEntries {
                definition: Definition {
                    kind: DefinitionKind::Keyword,
                    symbol: None,
                    node: Some(node.clone()),
                },
                references: vec![new_node_entry(node)],
            }]);
        }
        if super::m5x_3::is_jump_statement_target(node) {
            if let Some(parent) = &parent {
                if let Some(label_definition) = super::m5x_4::get_target_label(parent, &node.text()) {
                    if let Some(label_parent) = label_definition.parent() {
                        return Some(get_label_references_in_node(&label_parent, &label_definition));
                    }
                }
            }
            return None;
        }
        if super::m5x_3::is_label_of_labeled_statement(node) {
            if let Some(parent) = &parent {
                return Some(get_label_references_in_node(parent, node));
            }
            return None;
        }
        if super::m5x_4::is_this(node) {
            return Some(self.get_references_for_this_keyword(node, source_files));
        }
        if node.kind == SyntaxKind::SuperKeyword {
            return Some(self.get_references_for_super_keyword(node));
        }
        None
    }

    pub fn get_references_for_this_keyword(&self, this_or_super_keyword: &Arc<Node>, source_files: &[Arc<SourceFile>]) -> Vec<SymbolAndEntries> { ::tsox_core::fntrace::enter("get_references_for_this_keyword"); 
        let mut search_space_node = tsox_frontend::ast::mig::m3e_4::get_this_container(this_or_super_keyword, false, false);
        let mut static_flag = ast::ModifierFlags::Static;
        let is_parameter_name = |node: &Arc<Node>| -> bool {
            node.kind == SyntaxKind::Identifier
                && node.parent().map(|p| p.kind) == Some(SyntaxKind::Parameter)
                && node.parent().and_then(|p| p.name().cloned()).map(|n| Arc::ptr_eq(&n, node)).unwrap_or(false)
        };

        match search_space_node.kind {
            SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature | SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature
            | SyntaxKind::Constructor | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                if (search_space_node.kind == SyntaxKind::MethodDeclaration || search_space_node.kind == SyntaxKind::MethodSignature)
                    && ast::is_object_literal_method(&search_space_node)
                {
                    static_flag &= search_space_node.syntactic_modifier_flags();
                    search_space_node = search_space_node.parent().expect("search space node should have parent");
                } else {
                    static_flag &= search_space_node.syntactic_modifier_flags();
                    search_space_node = search_space_node.parent().expect("search space node should have parent");
                }
            }
            SyntaxKind::SourceFile => {
                if is_external_module_source_file(&search_space_node) || is_parameter_name(this_or_super_keyword) {
                    return Vec::new();
                }
            }
            SyntaxKind::FunctionDeclaration | SyntaxKind::FunctionExpression => {}
            _ => return Vec::new(),
        }

        let files_to_search: Vec<Arc<SourceFile>> = if search_space_node.kind != SyntaxKind::SourceFile {
            vec![source_file_of_node_by_node(self, &search_space_node)]
        } else {
            source_files.to_vec()
        };
        let program = self.get_program();
        let symbol_map = program.symbol_map();
        let mut references: Vec<ReferenceEntry> = Vec::new();
        for source_file in &files_to_search {
            let container: Option<Arc<Node>> = if search_space_node.kind == SyntaxKind::SourceFile {
                Some(source_file.node.clone())
            } else {
                Some(search_space_node.clone())
            };
            let possible = get_possible_symbol_reference_nodes(source_file, "this", container.as_ref());
            for n in possible {
                if !super::m5x_4::is_this(&n) {
                    continue;
                }
                let node_container = tsox_frontend::ast::mig::m3e_4::get_this_container(&n, false, false);
                if !ast::can_have_symbol(&node_container) {
                    continue;
                }
                let keep = match search_space_node.kind {
                    SyntaxKind::FunctionExpression | SyntaxKind::FunctionDeclaration => {
                        symbol_map.symbol_of(&search_space_node).cloned().zip(symbol_map.symbol_of(&node_container).cloned()).map(|(a, b)| Arc::ptr_eq(&a, &b)).unwrap_or(false)
                    }
                    SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature => {
                        ast::is_object_literal_method(&search_space_node)
                            && symbol_map.symbol_of(&search_space_node).cloned().zip(symbol_map.symbol_of(&node_container).cloned()).map(|(a, b)| Arc::ptr_eq(&a, &b)).unwrap_or(false)
                    }
                    SyntaxKind::ClassExpression | SyntaxKind::ClassDeclaration | SyntaxKind::ObjectLiteralExpression => {
                        let container_parent = node_container.parent();
                        container_parent
                            .as_ref()
                            .and_then(|p| symbol_map.symbol_of(p).cloned())
                            .zip(symbol_map.symbol_of(&search_space_node).cloned())
                            .map(|(ps, ss)| Arc::ptr_eq(&ps, &ss))
                            .unwrap_or(false)
                            && container_parent.as_ref().map(|p| ast::can_have_symbol(p)).unwrap_or(false)
                            && ast::is_static(&node_container) == (static_flag != ast::ModifierFlags::empty())
                    }
                    SyntaxKind::SourceFile => {
                        node_container.kind == SyntaxKind::SourceFile
                            && !is_external_module_source_file(&node_container)
                            && !is_parameter_name(&n)
                    }
                    _ => false,
                };
                if keep {
                    references.push(new_node_entry(&n));
                }
            }
        }
        vec![SymbolAndEntries {
            definition: Definition {
                kind: DefinitionKind::This,
                symbol: None,
                node: Some(this_or_super_keyword.clone()),
            },
            references,
        }]
    }

    pub fn get_references_for_super_keyword(&self, super_keyword: &Arc<Node>) -> Vec<SymbolAndEntries> { ::tsox_core::fntrace::enter("get_references_for_super_keyword"); 
        let Some(mut search_space_node) = get_super_container(super_keyword, false) else {
            return Vec::new();
        };
        let mut static_flag = ast::ModifierFlags::Static;
        match search_space_node.kind {
            SyntaxKind::PropertyDeclaration | SyntaxKind::PropertySignature | SyntaxKind::MethodDeclaration | SyntaxKind::MethodSignature
            | SyntaxKind::Constructor | SyntaxKind::GetAccessor | SyntaxKind::SetAccessor => {
                static_flag &= search_space_node.syntactic_modifier_flags();
                search_space_node = search_space_node.parent().expect("search space node should have parent");
            }
            _ => return Vec::new(),
        }

        let program = self.get_program();
        let symbol_map = program.symbol_map();
        let Some(source_file) = source_file_of_node(program.get_source_files().iter(), &search_space_node) else {
            return Vec::new();
        };
        let references: Vec<ReferenceEntry> = get_possible_symbol_reference_nodes(&source_file, "super", Some(&search_space_node))
            .into_iter()
            .filter_map(|node| {
                if node.kind != SyntaxKind::SuperKeyword {
                    return None;
                }
                let container = get_super_container(&node, false);
                if let Some(container) = container {
                    if ast::is_static(&container) == (static_flag != ast::ModifierFlags::empty())
                        && container
                            .parent()
                            .as_ref()
                            .and_then(|p| symbol_map.symbol_of(p).cloned())
                            .zip(symbol_map.symbol_of(&search_space_node).cloned())
                            .map(|(a, b)| Arc::ptr_eq(&a, &b))
                            .unwrap_or(false)
                    {
                        return Some(new_node_entry(&node));
                    }
                }
                None
            })
            .collect();

        vec![SymbolAndEntries {
            definition: Definition {
                kind: DefinitionKind::Symbol,
                symbol: symbol_map.symbol_of(&search_space_node).cloned(),
                node: None,
            },
            references,
        }]
    }
}

fn source_file_of_node<'a>(mut program_files: impl Iterator<Item = &'a Arc<SourceFile>>, node: &Arc<Node>) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("source_file_of_node"); 
    let file_node = ast::get_source_file_of_node(node)?;
    program_files.find(|f| f.node.id() == file_node.id()).cloned()
}

fn source_file_of_node_by_node(l: &LanguageService, node: &Arc<Node>) -> Arc<SourceFile> { ::tsox_core::fntrace::enter("source_file_of_node_by_node"); 
    let program = l.get_program();
    source_file_of_node(program.get_source_files().iter(), node).expect("node should belong to a program source file")
}

pub fn is_string_literal_property_reference(node: &Arc<Node>, checker: &mut tsox_checker::checker::Checker) -> bool { ::tsox_core::fntrace::enter("is_string_literal_property_reference"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    if ast::is_property_signature_declaration(&parent) {
        let Some(parent_parent) = parent.parent() else {
            return false;
        };
        let ty = checker.get_type_at_location(&parent_parent);
        return checker.get_property_of_type(&ty, &node.text()).is_some();
    }
    false
}

pub fn get_referenced_symbols_for_symbol(
    l: &LanguageService,
    program: &Arc<tsox_compile::compiler::Program>,
    original_symbol: &Arc<Symbol>,
    node: Option<&Arc<Node>>,
    source_files: &[Arc<SourceFile>],
    source_files_set: &std::collections::HashSet<String>,
    checker: &mut tsox_checker::checker::Checker,
    options: &RefOptions,
) -> Vec<SymbolAndEntries> { ::tsox_core::fntrace::enter("get_referenced_symbols_for_symbol"); 
    let symbol = skip_past_export_or_import_specifier_or_union(original_symbol, node, checker, !is_for_rename_with_prefix_and_suffix_text(options))
        .unwrap_or_else(|| original_symbol.clone());

    let search_meaning = if options.use_ != ReferenceUse::Rename {
        super::m5x_5::get_intersecting_meaning_from_declarations(node, &symbol, SemanticMeaning::ALL)
    } else {
        SemanticMeaning::ALL
    };
    let mut state = new_state(program, source_files, source_files_set, node, checker, search_meaning, options.clone());

    let mut export_specifier: Option<Arc<Node>> = None;
    if is_for_rename_with_prefix_and_suffix_text(options) && !symbol.declarations.is_empty() {
        export_specifier = symbol.declarations.iter().find(|d| ast::is_export_specifier(d)).cloned();
    }
    if let Some(export_specifier) = export_specifier {
        let name = export_specifier.name().expect("ExportSpecifier should have a name");
        let search_location = node.cloned().unwrap_or_else(|| state.node.clone());
        let search = state.create_search(&search_location, original_symbol, ImpExpKind::None, "", Vec::new());
        state.get_references_at_export_specifier(name, &symbol, &export_specifier, &search, true, true);
    } else if let Some(node) = node {
        if node.kind == SyntaxKind::DefaultKeyword
            && symbol.name == ast::INTERNAL_SYMBOL_NAME_DEFAULT
            && symbol.parent().is_some()
        {
            state.add_reference(node.clone(), Some(symbol.clone()), EntryKind::Node);
            let export_info = crate::ls::import_tracker::ExportInfo {
                exporting_module_symbol: symbol.parent(),
                export_kind: crate::ls::import_tracker::ExportKind::Default,
            };
            state.search_for_imports_of_export(node, &symbol, &export_info);
        } else {
            let all_search = state.populate_search_symbol_set(&symbol, Some(node), options.use_ == ReferenceUse::Rename, options.use_aliases_for_rename, options.implementations);
            let search = state.create_search(node, &symbol, ImpExpKind::None, "", all_search);
            state.get_references_in_container_or_files(&symbol, &search);
        }
    } else {
        let all_search = state.populate_search_symbol_set(&symbol, None, options.use_ == ReferenceUse::Rename, options.use_aliases_for_rename, options.implementations);
        let search = state.create_search(&state.node, &symbol, ImpExpKind::None, "", all_search);
        state.get_references_in_container_or_files(&symbol, &search);
    }

    state.take_result()
}

fn types_equal(checker: &mut tsox_checker::checker::Checker, a: &Arc<tsox_checker::checker::types::Type>, b: &Arc<tsox_checker::checker::types::Type>) -> bool { ::tsox_core::fntrace::enter("types_equal"); 
    checker.is_type_identical_to(a, b)
}

impl<'a> RefState<'a> {
    pub fn take_result(&mut self) -> Vec<SymbolAndEntries> { ::tsox_core::fntrace::enter("take_result"); 
        if self.pending_entries.is_empty() {
            return Vec::new();
        }
        let references = std::mem::take(&mut self.pending_entries);
        vec![SymbolAndEntries {
            definition: crate::ls::find_all_references::Definition {
                kind: crate::ls::find_all_references::DefinitionKind::Symbol,
                symbol: None,
                node: None,
            },
            references,
        }]
    }
}
