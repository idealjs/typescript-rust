#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use crate::ls::find_all_references::{DefinitionKind, EntryKind, ReferenceEntry, SymbolAndEntries};
use crate::ls::language_service::LanguageService;
use crate::ls::mig::m5s::{SpanFeature, SpanFidelity};
use crate::lsp::lsproto_lsp::DocumentUri;
use crate::lsp::lsproto_lsp_basic::{Location, Range};
use crate::ls::types_highlight::LocationLink;
use tsox_core::core::text::TextRange;
use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};

pub type PathUpdater<'a> = dyn Fn(&str) -> (String, bool) + 'a;

pub struct ToImport {
    pub new_file_name: String,
    pub updated: bool,
}

pub struct MovedFile {
    pub source_file: Arc<SourceFile>,
    pub new_file_name: String,
}

pub fn create_string_text_range(source_file: &Arc<SourceFile>, node: &Arc<Node>) -> TextRange {
    TextRange::new(
        tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(node, source_file, false) + 1,
        node.end() - 1,
    )
}

pub fn get_ts_config_object_literal_expression(
    ts_config_source_file: Option<&Arc<SourceFile>>,
) -> Option<Arc<Node>> {
    let ts_config_source_file = ts_config_source_file?;
    let statements = match &ts_config_source_file.node.data {
        ast::node_data_generated::NodeData::SourceFile(d) => &d.statements,
        _ => return None,
    };
    let first = statements.nodes.first()?;
    let expression = first.expression()?;
    if ast::is_object_literal_expression(expression) {
        Some(expression.clone())
    } else {
        None
    }
}

pub fn for_each_object_property(
    object_literal: Option<&Arc<Node>>,
    mut cb: impl FnMut(&Arc<Node>, &str),
) {
    let Some(object_literal) = object_literal else {
        return;
    };
    let properties = match &object_literal.data {
        ast::node_data_generated::NodeData::ObjectLiteralExpression(d) => &d.properties.nodes,
        _ => return,
    };
    for property in properties {
        if !ast::is_property_assignment(property) {
            continue;
        }
        if let Some(name) = property
            .name()
            .and_then(|name| ast::mig::m3h::try_get_text_of_property_name(name))
        {
            cb(property, &name);
        }
    }
}

pub fn relative_path_from_directory(
    from_directory: &str,
    to: &str,
    use_case_sensitive_file_names: bool,
) -> String {
    tsox_core::tspath::mig::m3i::get_relative_path_from_directory(
        from_directory,
        to,
        &tsox_core::tspath::ComparePathsOptions {
            use_case_sensitive_file_names,
            current_directory: String::new(),
        },
    )
}

pub fn relative_import_path_from_directory(
    from_directory: &str,
    to: &str,
    use_case_sensitive_file_names: bool,
) -> String {
    tsox_core::tspath::ensure_path_is_non_module_name(&relative_path_from_directory(
        from_directory,
        to,
        use_case_sensitive_file_names,
    ))
}

pub fn is_ambient_module_symbol(symbol: Option<&Arc<Symbol>>) -> bool {
    let Some(symbol) = symbol else {
        return false;
    };
    symbol
        .declarations
        .iter()
        .any(|d| ast::is_module_with_string_literal_name(d))
}

pub fn update_paths_property(
    config_file: &Arc<SourceFile>,
    config_dir: &str,
    property: &Arc<Node>,
    change_tracker: &mut crate::ls::change_tracker_tracker::Tracker,
    old_to_new: &PathUpdater,
    use_case_sensitive_file_names: bool,
) -> bool {
    let initializer = match &property.data {
        ast::node_data_generated::NodeData::PropertyAssignment(d) => Some(Arc::clone(&d.initializer)),
        _ => None,
    };
    let mut elements: Vec<Arc<Node>> = initializer.iter().cloned().collect();
    if initializer.as_ref().is_some_and(|i| ast::is_array_literal_expression(i)) {
        elements = match &initializer.as_ref().unwrap().data {
            ast::node_data_generated::NodeData::ArrayLiteralExpression(d) => d.elements.nodes.clone(),
            _ => Vec::new(),
        };
    }

    let mut found_exact_match = false;
    for element in &elements {
        found_exact_match = try_update_config_string(
            config_file,
            config_dir,
            &element,
            change_tracker,
            old_to_new,
            use_case_sensitive_file_names,
        ) || found_exact_match;
    }
    found_exact_match
}

pub fn try_update_config_string(
    config_file: &Arc<SourceFile>,
    config_dir: &str,
    element: &Arc<Node>,
    change_tracker: &mut crate::ls::change_tracker_tracker::Tracker,
    old_to_new: &PathUpdater,
    use_case_sensitive_file_names: bool,
) -> bool {
    if !ast::is_string_literal(element) {
        return false;
    }

    let element_file_name = tsox_core::tspath::normalize_path(&tsox_core::tspath::combine_paths(
        config_dir,
        &[element.text()],
    ));
    let (updated, ok) = old_to_new(&element_file_name);
    if !ok {
        return false;
    }

    let text_range = TextRange::new(
        tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(element, config_file, false) + 1,
        element.end() - 1,
    );
    let script = crate::mig::m5u_conv::SourceFileScriptView {
        file: Arc::clone(config_file),
    };
    let converters = crate::mig::m5u_conv::new_converters(
        crate::ls::lsconv_converters::PositionEncodingKind::Utf16,
        Box::new(crate::ls::lsconv_linemap::compute_lsp_line_starts),
    );
    let (lsp_range, _fidelity) = converters.to_lsp_range(&script, text_range);
    change_tracker.replace_range_with_text(
        config_file,
        lsp_range,
        relative_path_from_directory(config_dir, &updated, use_case_sensitive_file_names),
    );
    true
}

impl LanguageService {
    pub fn update_relative_path(
        &self,
        old_to_new: &PathUpdater,
        old_import_from_path: &str,
        new_import_from_path: &str,
        relative_specifier: &str,
    ) -> String {
        let old_absolute = tsox_core::tspath::normalize_path(&tsox_core::tspath::combine_paths(
            &tsox_core::tspath::get_directory_path(old_import_from_path),
            &[relative_specifier],
        ));
        let (new_absolute, ok) = old_to_new(&old_absolute);
        let new_absolute = if ok { new_absolute } else { old_absolute };
        relative_import_path_from_directory(
            &tsox_core::tspath::get_directory_path(new_import_from_path),
            &new_absolute,
            self.use_case_sensitive_file_names(),
        )
    }

    pub fn get_updated_import_specifier(
        &self,
        program: &tsox_compile::compiler::Program,
        checker: &tsox_checker::checker::Checker,
        source_file: &Arc<SourceFile>,
        import_literal: &Arc<Node>,
        old_to_new: &PathUpdater,
        moved_files: &[MovedFile],
        new_import_from_path: &str,
        importing_source_file_moved: bool,
        user_preferences: &tsox_tsoptions::modulespecifiers::UserPreferences,
    ) -> String {
        let imported_module_symbol = checker.get_symbol_at_location(import_literal);
        if is_ambient_module_symbol(imported_module_symbol.as_ref()) {
            return String::new();
        }

        let target = get_source_file_to_import(program, source_file, import_literal, old_to_new);

        let Some(target) = target else {
            let updated = get_updated_import_specifier_from_moved_source_files(
                program,
                source_file,
                import_literal,
                moved_files,
                new_import_from_path,
                user_preferences,
            );
            if !updated.is_empty() && updated != import_literal.text() {
                return updated;
            }
            if tsox_core::tspath::is_external_module_name_relative(&import_literal.text()) {
                return self.update_relative_path(
                    old_to_new,
                    &source_file.file_name,
                    new_import_from_path,
                    &import_literal.text(),
                );
            }
            return String::new();
        };

        if !target.updated
            && !(importing_source_file_moved
                && tsox_core::tspath::is_external_module_name_relative(&import_literal.text()))
        {
            return String::new();
        }

        let host = ProgramSpecifierHost(program);
        tsox_tsoptions::modulespecifiers::update_module_specifier(
            new_import_from_path,
            &target.new_file_name,
            &host,
            program.options(),
            user_preferences,
            &import_literal.text(),
            &tsox_tsoptions::modulespecifiers::ModuleSpecifierOptions {
                override_import_mode: program.get_mode_for_usage_location(source_file, import_literal),
            },
        )
        .unwrap_or_default()
    }
}

struct ProgramSpecifierHost<'a>(&'a tsox_compile::compiler::Program);

impl tsox_tsoptions::modulespecifiers::ModuleSpecifierGenerationHost for ProgramSpecifierHost<'_> {
    fn get_current_directory(&self) -> String {
        self.0.get_current_directory().to_string()
    }

    fn use_case_sensitive_file_names(&self) -> bool {
        self.0.use_case_sensitive_file_names()
    }

    fn common_source_directory(&self) -> String {
        tsox_checker::checker::Program::common_source_directory(self.0)
    }

    fn file_exists(&self, path: &str) -> bool {
        self.0.file_exists(path)
    }
}

pub fn get_source_file_to_import(
    program: &tsox_compile::compiler::Program,
    source_file: &Arc<SourceFile>,
    import_literal: &Arc<Node>,
    old_to_new: &PathUpdater,
) -> Option<ToImport> {
    let resolved = program.get_resolved_module_from_module_specifier(source_file, import_literal);
    if let Some(resolved) = resolved {
        if !resolved.resolved_file_name.is_empty() {
            let old_file_name = resolved.resolved_file_name.clone();
            let (new_file_name, ok) = old_to_new(&old_file_name);
            if ok {
                return Some(ToImport {
                    new_file_name,
                    updated: true,
                });
            }
            return Some(ToImport {
                new_file_name: old_file_name,
                updated: false,
            });
        }
    }
    None
}

pub fn get_updated_import_specifier_from_moved_source_files(
    program: &tsox_compile::compiler::Program,
    source_file: &Arc<SourceFile>,
    import_literal: &Arc<Node>,
    moved_files: &[MovedFile],
    importing_source_file_name: &str,
    user_preferences: &tsox_tsoptions::modulespecifiers::UserPreferences,
) -> String {
    let resolution_mode = program.get_mode_for_usage_location(source_file, import_literal);
    let host = ProgramSpecifierHost(program);
    for candidate in moved_files {
        let old_specifier = tsox_tsoptions::modulespecifiers::update_module_specifier(
            importing_source_file_name,
            &candidate.source_file.file_name,
            &host,
            program.options(),
            user_preferences,
            &import_literal.text(),
            &tsox_tsoptions::modulespecifiers::ModuleSpecifierOptions {
                override_import_mode: resolution_mode,
            },
        )
        .unwrap_or_default();
        if old_specifier != import_literal.text() {
            continue;
        }

        return tsox_tsoptions::modulespecifiers::update_module_specifier(
            importing_source_file_name,
            &candidate.new_file_name,
            &host,
            program.options(),
            user_preferences,
            &import_literal.text(),
            &tsox_tsoptions::modulespecifiers::ModuleSpecifierOptions {
                override_import_mode: resolution_mode,
            },
        )
        .unwrap_or_default();
    }
    String::new()
}

impl SymbolAndEntries {
    pub fn definition_node(&self) -> Option<&Arc<Node>> {
        if self.definition.kind == DefinitionKind::Symbol {
            return None;
        }
        self.definition.node.as_ref()
    }

    pub fn definition_symbol(&self) -> Option<&Arc<Symbol>> {
        self.definition.symbol.as_ref()
    }

    pub fn can_use_definition_symbol(&self) -> bool {
        match self.definition.kind {
            DefinitionKind::Symbol | DefinitionKind::This => self.definition.symbol.is_some(),
            _ => false,
        }
    }
}

impl ReferenceEntry {
    pub fn node(&self) -> Option<&Arc<Node>> {
        self.node.as_ref()
    }

    pub fn is_node_entry(&self) -> bool {
        self.node.is_some()
    }
}

pub struct RefState<'a> {
    pub program: Arc<tsox_compile::compiler::Program>,
    pub source_files: Vec<Arc<SourceFile>>,
    pub source_files_set: std::collections::HashSet<String>,
    pub node: Arc<Node>,
    pub checker: &'a mut tsox_checker::checker::Checker,
    pub search_meaning: ast::mig::m3e_4::SemanticMeaning,
    pub options: crate::ls::find_all_references::RefOptions,
    pub seen_imported_symbols: std::collections::HashSet<usize>,
    pub seen_export_symbols: std::collections::HashSet<usize>,
    pub seenReExportRHS: std::collections::HashSet<usize>,
    pub seen_contains_symbol: std::collections::HashSet<(usize, usize)>,
    pub seen_containing_type_references: std::collections::HashSet<usize>,
    pub pending_entries: Vec<ReferenceEntry>,
}

pub struct RefSearch {
    pub symbol: Arc<Symbol>,
    pub coming_from: ImpExpKind,
    pub text: String,
    pub all_search_symbols: Vec<Arc<Symbol>>,
    pub check_children: bool,
    pub is_crossing_had_file: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpExpKind {
    None,
    Import,
    Export,
}

impl<'a> RefState<'a> {
    pub fn create_search(
        &self,
        location: &Arc<Node>,
        symbol: &Arc<Symbol>,
        coming_from: ImpExpKind,
        text: &str,
        all_search_symbols: Vec<Arc<Symbol>>,
    ) -> RefSearch {
        let mut search = RefSearch {
            symbol: symbol.clone(),
            coming_from,
            text: text.to_string(),
            all_search_symbols,
            check_children: false,
            is_crossing_had_file: false,
        };
        if ast::is_export_assignment(location) {
            search.check_children = true;
        }
        search
    }

    pub fn get_import_searches(
        &self,
        export_symbol: &Arc<Symbol>,
        export_info: &crate::ls::import_tracker::ExportInfo,
    ) -> Option<crate::ls::import_tracker::ImportsResult> {
        let tracker = crate::ls::import_tracker::create_import_tracker(
            &self.program,
            &self.source_files,
            &self.source_files_set,
            &self.checker,
        );
        let is_for_rename = self.options.use_ == crate::ls::find_all_references::ReferenceUse::Rename;
        Some(tracker(export_symbol, export_info, is_for_rename))
    }

    pub fn add_reference(&mut self, reference_location: Arc<Node>, symbol: Option<Arc<Symbol>>, kind: EntryKind) {
        let entry = match kind {
            EntryKind::Range => ReferenceEntry {
                kind,
                node: None,
                context: None,
                file_name: String::new(),
                text_range: Some(TextRange::new(reference_location.pos(), reference_location.end())),
                lsp_range: None,
            },
            _ => ReferenceEntry {
                kind,
                node: Some(reference_location),
                context: None,
                file_name: String::new(),
                text_range: None,
                lsp_range: None,
            },
        };
        self.pending_entries.push(entry);
    }

    pub fn explicitly_inherits_from(&mut self, symbol: &Arc<Symbol>, parent: &Arc<Symbol>) -> bool {
        let mut seen = std::collections::HashSet::new();
        explicitly_inherits_from_worker(&mut self.checker, symbol, parent, &mut seen)
    }
}

fn explicitly_inherits_from_worker(
    checker: &mut tsox_checker::checker::Checker,
    symbol: &Arc<Symbol>,
    parent: &Arc<Symbol>,
    seen: &mut std::collections::HashSet<u64>,
) -> bool {
    if Arc::ptr_eq(symbol, parent) {
        return true;
    }
    if !seen.insert(symbol.id()) {
        return false;
    }
    if symbol.declarations.is_empty() {
        return false;
    }
    symbol.declarations.iter().any(|declaration| {
        let super_type_nodes = crate::ls::mig::m5x_5::get_all_super_type_nodes(declaration);
        super_type_nodes.iter().any(|type_reference| {
            let typ = checker.get_type_at_location(type_reference);
            typ.symbol()
                .is_some_and(|type_symbol| {
                    explicitly_inherits_from_worker(checker, type_symbol, parent, seen)
                })
        })
    })
}

pub fn get_class_constructor_symbol(class_symbol: &Arc<Symbol>) -> Option<Arc<Symbol>> {
    class_symbol
        .members
        .get(ast::INTERNAL_SYMBOL_NAME_CONSTRUCTOR)
        .cloned()
}

pub fn find_own_constructor_references(
    class_symbol: Option<&Arc<Symbol>>,
    source_file: &Arc<SourceFile>,
    add_node: &mut impl FnMut(Arc<Node>),
) {
    let Some(ctor_symbol) = class_symbol.and_then(get_class_constructor_symbol) else {
        return;
    };
    for decl in &ctor_symbol.declarations {
        if let Some(class_declaration) = ast::find_ancestor(decl, |n| ast::is_class_like(n)) {
            forEach_descendant_of_kind(&class_declaration, SyntaxKind::ConstructorKeyword, &mut |node| {
                add_node(node.clone());
            });
            forEach_descendant_of_kind(&class_declaration, SyntaxKind::NewKeyword, &mut |node| {
                add_node(node.clone());
            });
        }
    }
}

pub fn find_super_constructor_accesses(
    class_declaration: &Arc<Node>,
    add_node: &mut impl FnMut(Arc<Node>),
) {
    forEach_descendant_of_kind(class_declaration, SyntaxKind::SuperKeyword, &mut |node| {
        if node
            .parent()
            .map(|p| p.kind == SyntaxKind::PropertyAccessExpression)
            .unwrap_or(false)
        {
            add_node(node.clone());
        }
    });
}

pub fn forEach_descendant_of_kind(
    node: &Arc<Node>,
    kind: SyntaxKind,
    action: &mut dyn FnMut(&Arc<Node>),
) {
    if node.kind == kind {
        action(node);
    }
    ast::for_each_child(node, |child| {
        forEach_descendant_of_kind(child, kind, action);
        false
    });
}

pub fn get_all_references_for_import_meta(source_files: &[Arc<SourceFile>]) -> Vec<SymbolAndEntries> {
    let mut references: Vec<ReferenceEntry> = Vec::new();
    for source_file in source_files {
        for node in crate::ls::mig::m5t_4::get_possible_symbol_reference_nodes(
            source_file,
            "meta",
            Some(&source_file.node),
        ) {
            if let Some(parent) = node.parent() {
                if ast::mig::m3e_2::is_import_meta(&parent) {
                    references.push(ReferenceEntry {
                        kind: EntryKind::Node,
                        node: Some(parent),
                        context: None,
                        file_name: String::new(),
                        text_range: None,
                        lsp_range: None,
                    });
                }
            }
        }
    }
    if references.is_empty() {
        return Vec::new();
    }
    let definition_node = references[0].node.clone();
    vec![SymbolAndEntries {
        definition: crate::ls::find_all_references::Definition {
            kind: DefinitionKind::Keyword,
            symbol: None,
            node: definition_node,
        },
        references,
    }]
}

pub fn get_all_references_for_keyword(
    source_files: &[Arc<SourceFile>],
    keyword_kind: SyntaxKind,
    filter_read_only_type_operator: bool,
) -> Vec<SymbolAndEntries> {
    let mut result = Vec::new();
    for source_file in source_files {
        let keywords = collect_keywords(source_file.as_ref(), keyword_kind);
        for keyword in keywords {
            if filter_read_only_type_operator && !is_readonly_type_operator(&keyword) {
                continue;
            }
            result.push(SymbolAndEntries {
                definition: crate::ls::find_all_references::Definition {
                    kind: DefinitionKind::Keyword,
                    symbol: None,
                    node: Some(keyword),
                },
                references: Vec::new(),
            });
        }
    }
    result
}

fn collect_keywords(source_file: &SourceFile, kind: SyntaxKind) -> Vec<Arc<Node>> {
    let mut result = Vec::new();
    let root = &source_file.node;
    forEach_descendant_of_kind(root, kind, &mut |node| {
        result.push(node.clone());
    });
    result
}

fn is_readonly_type_operator(node: &Arc<Node>) -> bool {
    if node.kind != SyntaxKind::ReadonlyKeyword {
        return false;
    }
    let Some(parent) = node.parent() else {
        return false;
    };
    if parent.kind != SyntaxKind::TypeOperator {
        return false;
    }
    match &parent.data {
        ast::node_data_generated::NodeData::TypeOperatorNode(d) => {
            d.operator == SyntaxKind::ReadonlyKeyword
        }
        _ => false,
    }
}

pub fn get_label_references_in_node(
    container: &Arc<Node>,
    target_label: &Arc<Node>,
) -> Vec<SymbolAndEntries> {
    let label_name = target_label.text();
    let references = get_label_references_in_node_worker(container, &label_name);
    if !references.is_empty() {
        return vec![SymbolAndEntries {
            definition: crate::ls::find_all_references::Definition {
                kind: DefinitionKind::Label,
                symbol: None,
                node: Some(target_label.clone()),
            },
            references,
        }];
    }
    Vec::new()
}

fn get_label_references_in_node_worker(
    container: &Arc<Node>,
    label_name: &str,
) -> Vec<ReferenceEntry> {
    let mut result = Vec::new();
    ast::for_each_child(container, |child| {
        if ast::is_labeled_statement(child) {
            if let Some(label) = ast::mig::m3b::label(child) {
                if label.text() == label_name {
                    result.push(ReferenceEntry {
                        kind: EntryKind::Node,
                        node: Some(Arc::clone(label)),
                        context: None,
                        file_name: String::new(),
                        text_range: None,
                        lsp_range: None,
                    });
                }
            }
        }
        if let Some(break_or_continue) = get_break_or_continue_target(child, label_name) {
            result.push(ReferenceEntry {
                kind: EntryKind::Node,
                node: Some(break_or_continue),
                context: None,
                file_name: String::new(),
                text_range: None,
                lsp_range: None,
            });
        }
        result.extend(get_label_references_in_node_worker(child, label_name));
        false
    });
    result
}

fn get_break_or_continue_target(node: &Arc<Node>, label_name: &str) -> Option<Arc<Node>> {
    if node.kind != SyntaxKind::BreakStatement && node.kind != SyntaxKind::ContinueStatement {
        return None;
    }
    let label = ast::mig::m3b::label(node)?;
    if label.text() == label_name {
        Some(Arc::clone(label))
    } else {
        None
    }
}

pub fn get_context_node_for_node_entry(node: &Arc<Node>) -> Arc<Node> {
    let mut last_node = node.clone();
    let mut current = node.parent();
    while let Some(parent) = current {
        if !is_node_entry_context(parent.kind) {
            break;
        }
        last_node = parent.clone();
        current = parent.parent();
    }
    last_node
}

fn is_node_entry_context(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::PropertyAccessExpression
            | SyntaxKind::BinaryExpression
            | SyntaxKind::CallExpression
            | SyntaxKind::ElementAccessExpression
            | SyntaxKind::VariableDeclaration
            | SyntaxKind::ExportSpecifier
            | SyntaxKind::ImportSpecifier
    )
}

pub fn get_context_node(node: &Arc<Node>) -> Option<Arc<Node>> {
    if ast::is_export_assignment(node) {
        return Some(node.clone());
    }
    let parent = node.parent()?;
    if parent.kind == SyntaxKind::ExportSpecifier || parent.kind == SyntaxKind::ImportSpecifier {
        return get_context_node_for_export_or_import_specifier(&parent);
    }
    if parent.kind == SyntaxKind::MetaProperty {
        return Some(parent.clone());
    }
    if node.kind == SyntaxKind::DefaultKeyword && ast::is_export_declaration(&parent) {
        return Some(parent.clone());
    }
    None
}

fn get_context_node_for_export_or_import_specifier(specifier: &Arc<Node>) -> Option<Arc<Node>> {
    let parent = specifier.parent()?;
    let export_declaration = parent.parent()?;
    if ast::is_export_declaration(&export_declaration) {
        if let Some(module_specifier) = ast::mig::m3b::module_specifier(&export_declaration) {
            return Some(Arc::clone(module_specifier));
        }
    }
    None
}
