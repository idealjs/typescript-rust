#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::m1a::r19k2_defs::symbol_of_node;
use tsox_compile::compiler::Program;
use tsox_frontend::ast::{self, Node, NodeData, NodeList, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::node_source_file::FileReference;
use tsox_frontend::ast::mig::m3e_4::{get_assignment_declaration_kind, get_first_identifier, JsDeclarationKind};
use tsox_frontend::ast::mig::m3f_2::import_from_module_specifier;
use tsox_frontend::ast::mig::m3f_4::is_default_import;
use tsox_frontend::ast::mig::m3g::is_module_exports_access_expression;
use tsox_frontend::ast::mig::m3g_3::is_variable_declaration_initialized_to_bare_or_accessed_require;
use tsox_frontend::ast::mig::m3h::walk_up_binding_elements_and_patterns;
use tsox_frontend::ast::mig::w7a::is_implicitly_exported_jsdoc_declaration;
use crate::ls::mig::m5x_5::get_property_symbol_of_object_binding_pattern_without_property_name;
use crate::ls::mig::m5x_6::is_source_file_with_global_exports;
use tsox_frontend::ast::is_ambient_module;
use tsox_frontend::ast::mig::m3g::is_module_augmentation_external;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M5uImpExpKind {
    Unknown,
    Import,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M5uExportKind {
    Named,
    Default,
    ExportEquals,
    Umd,
    Module,
}

pub struct M5uImportExportSymbol {
    pub kind: M5uImpExpKind,
    pub symbol: Option<Arc<Symbol>>,
    pub export_info: Option<M5uExportInfo>,
}

pub struct M5uExportInfo {
    pub exporting_module_symbol: Arc<Symbol>,
    pub export_kind: M5uExportKind,
}

pub struct M5uLocationAndSymbol {
    pub import_location: Arc<Node>,
    pub import_symbol: Option<Arc<Symbol>>,
}

pub struct M5uImportsResult {
    pub import_searches: Vec<M5uLocationAndSymbol>,
    pub single_references: Vec<Arc<Node>>,
    pub indirect_users: Vec<Arc<SourceFile>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum M5uModuleReferenceKind {
    Import,
    Reference,
    Implicit,
}

pub struct M5uModuleReference {
    pub kind: M5uModuleReferenceKind,
    pub literal: Option<Arc<Node>>,
    pub referencing_file: Option<Arc<SourceFile>>,
    pub ref_directive: Option<FileReference>,
}


fn is_external_module_augmentation(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_module_augmentation"); 
    is_ambient_module(node) && is_module_augmentation_external(node)
}

pub fn node_module_specifier(declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_module_specifier"); 
    match &declaration.data {
        NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
        NodeData::ExportDeclaration(d) => d.module_specifier.as_ref().map(Arc::clone),
        NodeData::JSDocImportTag(d) => Some(Arc::clone(&d.module_specifier)),
        _ => None,
    }
}

pub fn node_import_clause(declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_import_clause"); 
    match &declaration.data {
        NodeData::ImportDeclaration(d) => d.import_clause.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn node_named_bindings(import_clause: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_named_bindings"); 
    match &import_clause.data {
        NodeData::ImportClause(d) => d.named_bindings.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn node_named_bindings_elements(named_bindings: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("node_named_bindings_elements"); 
    match &named_bindings.data {
        NodeData::NamedImports(d) => d.elements.nodes.clone(),
        NodeData::NamedExports(d) => d.elements.nodes.clone(),
        _ => Vec::new(),
    }
}

pub fn node_import_equals_module_reference(declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_import_equals_module_reference"); 
    match &declaration.data {
        NodeData::ImportEqualsDeclaration(d) => Some(Arc::clone(&d.module_reference)),
        _ => None,
    }
}

pub fn node_export_clause(declaration: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("node_export_clause"); 
    match &declaration.data {
        NodeData::ExportDeclaration(d) => d.export_clause.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn is_ambient_module_declaration(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_ambient_module_declaration"); 
    is_ambient_module_declaration_node(node)
}

fn is_ambient_module_declaration_node(node: &Node) -> bool { ::tsox_core::fntrace::enter("is_ambient_module_declaration_node"); 
    ast::is_module_declaration(node)
        && node.name().map_or(false, |name| ast::is_string_literal(&name))
}

pub fn get_statements_of_source_file_like(node: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_statements_of_source_file_like"); 
    if ast::is_source_file(node) {
        if let NodeData::SourceFile(d) = &node.data {
            return d.statements.nodes.clone();
        }
        return Vec::new();
    }
    if let Some(body) = node.body() {
        if let NodeData::ModuleBlock(d) = &body.data {
            return d.statements.nodes.clone();
        }
    }
    Vec::new()
}

pub fn for_each_possible_import_or_export_statement(
    source_file_like: &Arc<Node>,
    action: &mut dyn FnMut(&Arc<Node>) -> bool,
) -> bool { ::tsox_core::fntrace::enter("for_each_possible_import_or_export_statement"); 
    for statement in get_statements_of_source_file_like(source_file_like) {
        if action(&statement)
            || is_ambient_module_declaration(&statement)
                && for_each_possible_import_or_export_statement(&statement, action)
        {
            return true;
        }
    }
    false
}

pub fn get_source_file_like_for_import_declaration(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_source_file_like_for_import_declaration"); 
    if ast::is_call_expression(node) || ast::is_jsdoc_import_tag(node) {
        return ast::get_source_file_of_node(node);
    }
    let parent = node.parent()?;
    if ast::is_source_file(&parent) {
        return Some(parent);
    }
    let grand = parent.parent()?;
    if ast::is_module_block(&parent) && is_ambient_module_declaration(&grand) {
        return Some(grand);
    }
    None
}

pub fn for_each_import(
    program: &Program,
    source_file: &Arc<SourceFile>,
    action: &mut dyn FnMut(&Arc<Node>, &Arc<Node>),
) { ::tsox_core::fntrace::enter("for_each_import"); 
    let mut implicit_imports: Vec<Arc<Node>> = Vec::new();
    if let (_, Some(jsx_specifier)) =
        program.get_jsx_runtime_import_specifier(&source_file.file_name)
    {
        implicit_imports.push(jsx_specifier);
    }
    if let Some(import_helpers_specifier) =
        program.get_import_helpers_import_specifier(&source_file.file_name)
    {
        implicit_imports.push(import_helpers_specifier);
    }
    if source_file.external_module_indicator.is_some()
        || !source_file.imports.is_empty()
        || !implicit_imports.is_empty()
    {
        for i in &source_file.imports {
            if let Some(import_statement) = import_from_module_specifier(i) {
                action(&import_statement, i);
            }
        }
        for i in &implicit_imports {
            if let Some(import_statement) = import_from_module_specifier(i) {
                action(&import_statement, i);
            }
        }
    } else {
        for_each_possible_import_or_export_statement(&source_file.node, &mut |node| {
            match node.kind {
                SyntaxKind::ExportDeclaration
                | SyntaxKind::ImportDeclaration
                | SyntaxKind::JSImportDeclaration => {
                    if let Some(specifier) = node_module_specifier(node) {
                        if ast::is_string_literal(&specifier) {
                            action(node, &specifier);
                        }
                    }
                }
                SyntaxKind::ImportEqualsDeclaration => {
                    if is_external_module_import_equals(node) {
                        if let Some(module_reference) = node_import_equals_module_reference(node)
                        {
                            if let Some(expr) = module_reference.expression() {
                                action(node, expr);
                            }
                        }
                    }
                }
                _ => {}
            }
            false
        });
    }
}

pub fn get_direct_imports_map(
    program: &Program,
    source_files: &[Arc<SourceFile>],
    checker: &Checker,
) -> HashMap<u64, Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("get_direct_imports_map"); 
    let mut result: HashMap<u64, Vec<Arc<Node>>> = HashMap::new();
    for source_file in source_files {
        for_each_import(program, source_file, &mut |import_decl, module_specifier| {
            if let Some(module_symbol) = checker.get_symbol_at_location(module_specifier) {
                result
                    .entry(module_symbol.id())
                    .or_default()
                    .push(Arc::clone(import_decl));
            }
        });
    }
    result
}

pub fn get_containing_module_symbol(
    importer: &Arc<Node>,
    checker: &Checker,
) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_containing_module_symbol"); 
    let source_file_like = get_source_file_like_for_import_declaration(importer)?;
    let symbol = symbol_of_node(&source_file_like)?;
    Some(checker.get_merged_symbol(&symbol))
}

pub fn find_namespace_re_exports(
    source_file_like: &Arc<Node>,
    name: &Arc<Node>,
    checker: &mut Checker,
) -> bool { ::tsox_core::fntrace::enter("find_namespace_re_exports"); 
    let namespace_import_symbol = checker.get_symbol_at_location(name);
    let namespace_import_ptr = namespace_import_symbol.as_ref().map(Arc::as_ptr);
    for_each_possible_import_or_export_statement(source_file_like, &mut |statement| {
        if !ast::is_export_declaration(statement) {
            return false;
        }
        let export_clause = node_export_clause(statement);
        let module_specifier = node_module_specifier(statement);
        module_specifier.is_none()
            && export_clause
                .as_ref()
                .map_or(false, |export_clause| {
                    ast::is_named_exports(export_clause)
                        && node_named_bindings_elements(export_clause).iter().any(|element| {
                            checker.get_export_specifier_local_target_symbol(element).as_ref()
                                .map(Arc::as_ptr)
                                == namespace_import_ptr
                        })
                })
    })
}

pub fn get_importers_for_export(
    source_files: &[Arc<SourceFile>],
    source_files_set: &HashSet<String>,
    all_direct_imports: &HashMap<u64, Vec<Arc<Node>>>,
    export_info: &M5uExportInfo,
    checker: &mut Checker,
) -> (Vec<Arc<Node>>, Vec<Arc<SourceFile>>) { ::tsox_core::fntrace::enter("get_importers_for_export"); 
    let mut direct_imports: Vec<Arc<Node>> = Vec::new();
    let mut indirect_user_declarations: Vec<Arc<Node>> = Vec::new();
    let mut seen_direct_import: HashSet<u64> = HashSet::new();
    let mut seen_indirect_user: HashSet<u64> = HashSet::new();
    let is_available_through_global = export_info
        .exporting_module_symbol
        .value_declaration
        .as_ref()
        .map_or(false, is_source_file_with_global_exports);

    let get_direct_imports = |module_symbol: &Arc<Symbol>| -> Vec<Arc<Node>> {
        all_direct_imports
            .get(&module_symbol.id())
            .cloned()
            .unwrap_or_default()
    };

    fn mark_seen(seen: &mut HashSet<u64>, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("mark_seen"); 
        seen.insert(node.id())
    }

    fn add_indirect_user(
        indirect_user_declarations: &mut Vec<Arc<Node>>,
        seen_indirect_user: &mut HashSet<u64>,
        source_file_like: &Arc<Node>,
        add_transitive_dependencies: bool,
        is_available_through_global: bool,
        checker: &Checker,
        all_direct_imports: &HashMap<u64, Vec<Arc<Node>>>,
    ) { ::tsox_core::fntrace::enter("add_indirect_user"); 
        if is_available_through_global {
            return;
        }
        if !mark_seen(seen_indirect_user, source_file_like) {
            return;
        }
        indirect_user_declarations.push(Arc::clone(source_file_like));
        if !add_transitive_dependencies {
            return;
        }
        let Some(symbol) = symbol_of_node(source_file_like) else {
            return;
        };
        let module_symbol = checker.get_merged_symbol(&symbol);
        if module_symbol.flags & ast::SymbolFlags::MODULE == ast::SymbolFlags::None {
            return;
        }
        let direct_imports = all_direct_imports
            .get(&module_symbol.id())
            .cloned()
            .unwrap_or_default();
        for direct_import in direct_imports {
            if !ast::is_import_type_node(&direct_import) {
                if let Some(source_file_like) =
                    get_source_file_like_for_import_declaration(&direct_import)
                {
                    add_indirect_user(
                        indirect_user_declarations,
                        seen_indirect_user,
                        &source_file_like,
                        true,
                        is_available_through_global,
                        checker,
                        all_direct_imports,
                    );
                }
            }
        }
    }

    fn is_exported(node: &Arc<Node>, stop_at_ambient_module: bool) -> bool { ::tsox_core::fntrace::enter("is_exported"); 
        let mut current = Some(Arc::clone(node));
        while let Some(node) = current {
            if stop_at_ambient_module && is_ambient_module_declaration(&node) {
                break;
            }
            if node.has_syntactic_modifier(ast::ModifierFlags::Export) {
                return true;
            }
            current = node.parent();
        }
        false
    }

    fn handle_import_call(
        import_call: &Arc<Node>,
        indirect_user_declarations: &mut Vec<Arc<Node>>,
        seen_indirect_user: &mut HashSet<u64>,
        is_available_through_global: bool,
        checker: &Checker,
        all_direct_imports: &HashMap<u64, Vec<Arc<Node>>>,
    ) { ::tsox_core::fntrace::enter("handle_import_call"); 
        let top = ast::find_ancestor(import_call, is_ambient_module_declaration_node);
        let top = top.or_else(|| ast::get_source_file_of_node(import_call));
        if let Some(top) = top {
            add_indirect_user(
                indirect_user_declarations,
                seen_indirect_user,
                &top,
                is_exported(import_call, true),
                is_available_through_global,
                checker,
                all_direct_imports,
            );
        }
    }

    fn handle_namespace_import(
        import_declaration: &Arc<Node>,
        name: &Arc<Node>,
        is_re_export: bool,
        already_added_direct: bool,
        direct_imports: &mut Vec<Arc<Node>>,
        indirect_user_declarations: &mut Vec<Arc<Node>>,
        seen_indirect_user: &mut HashSet<u64>,
        is_available_through_global: bool,
        export_info: &M5uExportInfo,
        checker: &mut Checker,
        all_direct_imports: &HashMap<u64, Vec<Arc<Node>>>,
    ) {
        if export_info.export_kind == M5uExportKind::ExportEquals {
            if !already_added_direct {
                direct_imports.push(Arc::clone(import_declaration));
            }
        } else if !is_available_through_global {
            if let Some(source_file_like) =
                get_source_file_like_for_import_declaration(import_declaration)
            {
                let add_transitive =
                    is_re_export || find_namespace_re_exports(&source_file_like, name, checker);
                add_indirect_user(
                    indirect_user_declarations,
                    seen_indirect_user,
                    &source_file_like,
                    add_transitive,
                    is_available_through_global,
                    checker,
                    all_direct_imports,
                );
            }
        }
    }

    let export_info_ref = export_info;
    fn handle_direct_imports_inner(
        exporting_module_symbol: &Arc<Symbol>,
        seen_direct_import: &mut HashSet<u64>,
        seen_indirect_user: &mut HashSet<u64>,
        indirect_user_declarations: &mut Vec<Arc<Node>>,
        direct_imports: &mut Vec<Arc<Node>>,
        is_available_through_global: bool,
        export_info_ref: &M5uExportInfo,
        checker: &mut Checker,
        all_direct_imports: &HashMap<u64, Vec<Arc<Node>>>,
        get_direct_imports: &dyn Fn(&Arc<Symbol>) -> Vec<Arc<Node>>,
    ) { ::tsox_core::fntrace::enter("handle_direct_imports_inner"); 
        let these_direct_imports = get_direct_imports(exporting_module_symbol);
        for direct in these_direct_imports {
            if !mark_seen(seen_direct_import, &direct) {
                continue;
            }
            match direct.kind {
                SyntaxKind::CallExpression => {
                    if ast::is_import_call(&direct) {
                        handle_import_call(
                            &direct,
                            indirect_user_declarations,
                            seen_indirect_user,
                            is_available_through_global,
                            checker,
                            all_direct_imports,
                        );
                    } else if !is_available_through_global {
                        if let Some(parent) = direct.parent() {
                            if export_info_ref.export_kind == M5uExportKind::ExportEquals
                                && ast::is_variable_declaration(&parent)
                            {
                                if let Some(name) = parent.name() {
                                    if ast::is_identifier(name) {
                                        direct_imports.push(Arc::clone(name));
                                    }
                                }
                            }
                        }
                    }
                }
                SyntaxKind::Identifier => {}
                SyntaxKind::ImportEqualsDeclaration => {
                    if let Some(name) = direct.name() {
                        handle_namespace_import(
                            &direct,
                            name,
                            direct.has_syntactic_modifier(ast::ModifierFlags::Export),
                            false,
                            direct_imports,
                            indirect_user_declarations,
                            seen_indirect_user,
                            is_available_through_global,
                            export_info_ref,
                            checker,
                            all_direct_imports,
                        );
                    }
                }
                SyntaxKind::ImportDeclaration
                | SyntaxKind::JSImportDeclaration
                | SyntaxKind::JSDocImportTag => {
                    direct_imports.push(Arc::clone(&direct));
                    let mut handled_namespace = false;
                    if let Some(import_clause) = node_import_clause(&direct) {
                        if let Some(named_bindings) = node_named_bindings(&import_clause) {
                            if ast::is_namespace_import(&named_bindings) {
                                if let Some(name) = named_bindings.name() {
                                    handled_namespace = true;
                                    handle_namespace_import(
                                        &direct,
                                        name,
                                        false,
                                        true,
                                        direct_imports,
                                        indirect_user_declarations,
                                        seen_indirect_user,
                                        is_available_through_global,
                                        export_info_ref,
                                        checker,
                                        all_direct_imports,
                                    );
                                }
                            }
                        }
                    }
                    if handled_namespace {
                        continue;
                    }
                    if !is_available_through_global && is_default_import(&direct) {
                        if let Some(source_file_like) =
                            get_source_file_like_for_import_declaration(&direct)
                        {
                            add_indirect_user(
                                indirect_user_declarations,
                                seen_indirect_user,
                                &source_file_like,
                                false,
                                is_available_through_global,
                                checker,
                                all_direct_imports,
                            );
                        }
                    }
                }
                SyntaxKind::ExportDeclaration => {
                    let export_clause = node_export_clause(&direct);
                    match export_clause {
                        None => {
                            if let Some(module_symbol) =
                                get_containing_module_symbol(&direct, checker)
                            {
                                handle_direct_imports_inner(
                                    &module_symbol,
                                    seen_direct_import,
                                    seen_indirect_user,
                                    indirect_user_declarations,
                                    direct_imports,
                                    is_available_through_global,
                                    export_info_ref,
                                    checker,
                                    all_direct_imports,
                                    get_direct_imports,
                                );
                            }
                        }
                        Some(export_clause) if ast::is_namespace_export(&export_clause) => {
                            if let Some(source_file_like) =
                                get_source_file_like_for_import_declaration(&direct)
                            {
                                add_indirect_user(
                                    indirect_user_declarations,
                                    seen_indirect_user,
                                    &source_file_like,
                                    true,
                                    is_available_through_global,
                                    checker,
                                    all_direct_imports,
                                );
                            }
                        }
                        Some(_) => {
                            direct_imports.push(Arc::clone(&direct));
                        }
                    }
                }
                SyntaxKind::ImportType => {
                    if !is_available_through_global {
                        let is_type_of = match &direct.data {
                            NodeData::ImportTypeNode(d) => d.is_type_of,
                            _ => false,
                        };
                        let has_qualifier = match &direct.data {
                            NodeData::ImportTypeNode(d) => d.qualifier.is_some(),
                            _ => true,
                        };
                        if is_type_of && !has_qualifier && is_exported(&direct, false) {
                            if let Some(source_file) = ast::get_source_file_of_node(&direct) {
                                add_indirect_user(
                                    indirect_user_declarations,
                                    seen_indirect_user,
                                    &source_file,
                                    true,
                                    is_available_through_global,
                                    checker,
                                    all_direct_imports,
                                );
                            }
                        }
                    }
                    direct_imports.push(Arc::clone(&direct));
                }
                _ => {}
            }
        }
    }
    let mut handle_direct_imports = |exporting_module_symbol: &Arc<Symbol>| {
        handle_direct_imports_inner(
            exporting_module_symbol,
            &mut seen_direct_import,
            &mut seen_indirect_user,
            &mut indirect_user_declarations,
            &mut direct_imports,
            is_available_through_global,
            export_info_ref,
            checker,
            all_direct_imports,
            &get_direct_imports,
        )
    };

    handle_direct_imports(&export_info.exporting_module_symbol);

    let mut indirect_users: Vec<Arc<SourceFile>> = Vec::new();
    if is_available_through_global {
        indirect_users.extend(source_files.iter().cloned());
    } else {
        for decl in &export_info.exporting_module_symbol.declarations {
            if is_external_module_augmentation(decl) {
                if let Some(source_file) = ast::get_source_file_of_node(decl) {
                    if let Some(file) = node_as_source_file(&source_file) {
                        if source_files_set.contains(&file.file_name) {
                            add_indirect_user(
                                &mut indirect_user_declarations,
                                &mut seen_indirect_user,
                                decl,
                                false,
                                is_available_through_global,
                                checker,
                                all_direct_imports,
                            );
                        }
                    }
                }
            }
        }
        for decl in &indirect_user_declarations {
            if let Some(source_file) = ast::get_source_file_of_node(decl) {
                if let Some(file) = node_as_source_file(&source_file) {
                    indirect_users.push(file);
                }
            }
        }
    }
    (direct_imports, indirect_users)
}

pub fn node_as_source_file(node: &Arc<Node>) -> Option<Arc<SourceFile>> { ::tsox_core::fntrace::enter("node_as_source_file"); 
    tsox_checker::checker::mig::m3a_2::r31k1_defs::source_file_of_node(node)
}

pub fn get_searches_from_direct_imports(
    direct_imports: &[Arc<Node>],
    export_symbol: &Arc<Symbol>,
    export_kind: M5uExportKind,
    checker: &mut Checker,
    is_for_rename: bool,
) -> (Vec<M5uLocationAndSymbol>, Vec<Arc<Node>>) { ::tsox_core::fntrace::enter("get_searches_from_direct_imports"); 
    let mut import_searches: Vec<M5uLocationAndSymbol> = Vec::new();
    let mut single_references: Vec<Arc<Node>> = Vec::new();

    let is_name_match = |name: &str| -> bool {
        name == export_symbol.name
            || export_kind != M5uExportKind::Named && name == "default"
    };

    for decl in direct_imports {
        if ast::is_import_equals_declaration(decl) {
            if is_external_module_import_equals(decl) {
                if let Some(name) = decl.name() {
                    if export_kind == M5uExportKind::ExportEquals
                        && (!is_for_rename || is_name_match(name.text()))
                    {
                        import_searches.push(M5uLocationAndSymbol {
                            import_location: Arc::clone(name),
                            import_symbol: checker.get_symbol_at_location(name),
                        });
                    }
                }
            }
            continue;
        }
        if ast::is_identifier(decl) {
            if export_kind == M5uExportKind::ExportEquals
                && (!is_for_rename || is_name_match(decl.text()))
            {
                import_searches.push(M5uLocationAndSymbol {
                    import_location: Arc::clone(decl),
                    import_symbol: checker.get_symbol_at_location(decl),
                });
            }
            continue;
        }
        if ast::is_import_type_node(decl) {
            match import_type_qualifier(decl) {
                Some(qualifier) => {
                    let first_identifier = get_first_identifier(&qualifier);
                    if first_identifier.text() == export_symbol.name {
                        single_references.push(first_identifier);
                    }
                }
                None => {
                    if export_kind == M5uExportKind::ExportEquals {
                        if let Some(literal) = import_type_argument_literal(decl) {
                            single_references.push(literal);
                        }
                    }
                }
            }
            continue;
        }
        let module_specifier = node_module_specifier(decl);
        let Some(module_specifier) = module_specifier else {
            continue;
        };
        if !ast::is_string_literal(&module_specifier) {
            continue;
        }
        if ast::is_export_declaration(decl) {
            if let Some(export_clause) = node_export_clause(decl) {
                if ast::is_named_exports(&export_clause) {
                    for element in node_named_bindings_elements(&export_clause) {
                        let name = element.name();
                        let property_name = import_specifier_property_name(&element);
                        let match_name = property_name
                            .as_ref()
                            .map_or(name.as_ref().map(|n| n.text()), |p| Some(p.text()));
                        let Some(match_name) = match_name else {
                            continue;
                        };
                        if !is_name_match(match_name) {
                            continue;
                        }
                        if let Some(property_name) = property_name {
                            single_references.push(Arc::clone(&property_name));
                            let should_add = !is_for_rename
                                || name.map_or(false, |n| n.text() == export_symbol.name);
                            if should_add {
                                if let Some(name) = name {
                                    import_searches.push(M5uLocationAndSymbol {
                                        import_location: Arc::clone(name),
                                        import_symbol: checker.get_symbol_at_location(name),
                                    });
                                }
                            }
                        } else {
                            let local_symbol = if ast::is_export_specifier(&element)
                                && import_specifier_property_name(&element).is_some()
                            {
                                checker.get_export_specifier_local_target_symbol(&element)
                            } else {
                                name.and_then(|n| checker.get_symbol_at_location(n))
                            };
                            if let Some(name) = name {
                                import_searches.push(M5uLocationAndSymbol {
                                    import_location: Arc::clone(name),
                                    import_symbol: local_symbol,
                                });
                            }
                        }
                    }
                }
            }
            continue;
        }
        if let Some(import_clause) = node_import_clause(decl) {
            if let Some(named_bindings) = node_named_bindings(&import_clause) {
                match named_bindings.kind {
                    SyntaxKind::NamespaceImport => {
                        if let Some(name) = named_bindings.name() {
                            if export_kind == M5uExportKind::ExportEquals
                                && (!is_for_rename || is_name_match(name.text()))
                            {
                                import_searches.push(M5uLocationAndSymbol {
                                    import_location: Arc::clone(name),
                                    import_symbol: checker.get_symbol_at_location(name),
                                });
                            }
                        }
                    }
                    SyntaxKind::NamedImports => {
                        if export_kind == M5uExportKind::Named
                            || export_kind == M5uExportKind::Default
                        {
                            for element in node_named_bindings_elements(&named_bindings) {
                                let name = element.name();
                                let property_name = import_specifier_property_name(&element);
                                let Some(name) = name else {
                                    continue;
                                };
                                let match_name = property_name
                                    .as_ref()
                                    .map_or(name.text(), |p| p.text());
                                if !is_name_match(match_name) {
                                    continue;
                                }
                                if let Some(property_name) = property_name {
                                    single_references.push(Arc::clone(&property_name));
                                    if !is_for_rename || name.text() == export_symbol.name {
                                        import_searches.push(M5uLocationAndSymbol {
                                            import_location: Arc::clone(&name),
                                            import_symbol: checker
                                                .get_symbol_at_location(&name),
                                        });
                                    }
                                } else {
                                    let local_symbol =
                                        checker.get_symbol_at_location(&name);
                                    import_searches.push(M5uLocationAndSymbol {
                                        import_location: Arc::clone(&name),
                                        import_symbol: local_symbol,
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            let clause_name = match &import_clause.data {
                NodeData::ImportClause(d) => d.name.as_ref().map(Arc::clone),
                _ => None,
            };
            if let Some(name) = clause_name {
                if (export_kind == M5uExportKind::Default
                    || export_kind == M5uExportKind::ExportEquals)
                    && (!is_for_rename
                        || name.text() == symbol_name_no_default(export_symbol))
                {
                    import_searches.push(M5uLocationAndSymbol {
                        import_location: Arc::clone(&name),
                        import_symbol: checker.get_symbol_at_location(&name),
                    });
                }
            }
        }
    }
    (import_searches, single_references)
}

pub fn import_type_qualifier(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_type_qualifier"); 
    match &node.data {
        NodeData::ImportTypeNode(d) => d.qualifier.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn import_type_argument_literal(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_type_argument_literal"); 
    match &node.data {
        NodeData::ImportTypeNode(d) => match &d.argument.data {
            NodeData::LiteralTypeNode(literal) => Some(Arc::clone(&literal.literal)),
            _ => None,
        },
        _ => None,
    }
}

pub fn import_specifier_property_name(specifier: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("import_specifier_property_name"); 
    match &specifier.data {
        NodeData::ImportSpecifier(d) => d.property_name.as_ref().map(Arc::clone),
        NodeData::ExportSpecifier(d) => d.property_name.as_ref().map(Arc::clone),
        _ => None,
    }
}

pub fn get_import_or_export_symbol(
    node: &Arc<Node>,
    symbol: &Arc<Symbol>,
    checker: &mut Checker,
    coming_from_export: bool,
) -> Option<M5uImportExportSymbol> { ::tsox_core::fntrace::enter("get_import_or_export_symbol"); 
    let export_info_of = |symbol: &Arc<Symbol>, kind: M5uExportKind| -> Option<M5uImportExportSymbol> {
        get_export_info(symbol, kind, checker).map(|export_info| M5uImportExportSymbol {
            kind: M5uImpExpKind::Export,
            symbol: Some(Arc::clone(symbol)),
            export_info: Some(export_info),
        })
    };

    fn get_export_kind_for_declaration(node: &Arc<Node>) -> M5uExportKind { ::tsox_core::fntrace::enter("get_export_kind_for_declaration"); 
        if node.has_syntactic_modifier(ast::ModifierFlags::Default) {
            M5uExportKind::Default
        } else {
            M5uExportKind::Named
        }
    }

    let get_export = || -> Option<M5uImportExportSymbol> {
        let get_export_assignment_export = |ex: &Arc<Node>| -> Option<M5uImportExportSymbol> {
            let ex_symbol = symbol_of_node(ex)?;
            let parent = ex_symbol.parent()?;
            let export_kind = match &ex.data {
                NodeData::ExportAssignment(d) if d.is_export_equals => {
                    M5uExportKind::ExportEquals
                }
                _ => M5uExportKind::Default,
            };
            return Some(M5uImportExportSymbol {
                kind: M5uImpExpKind::Export,
                symbol: Some(Arc::clone(symbol)),
                export_info: Some(M5uExportInfo {
                    exporting_module_symbol: parent,
                    export_kind,
                }),
            });
        };

        let get_special_property_export = |node: &Arc<Node>,
                                            use_lhs_symbol: bool|
         -> Option<M5uImportExportSymbol> {
            let kind = match get_assignment_declaration_kind(node) {
                JsDeclarationKind::ExportsProperty => M5uExportKind::Named,
                JsDeclarationKind::ModuleExports => M5uExportKind::ExportEquals,
                _ => return None,
            };
            let sym = if use_lhs_symbol { symbol_of_node(node) } else { Some(Arc::clone(symbol)) };
            let sym = sym?;
            export_info_of(&sym, kind)
        };

        let parent = node.parent()?;
        let grandparent = parent.parent();
        if let Some(export_symbol) = &symbol.export_symbol {
            if ast::is_property_access_expression(&parent) {
                if ast::is_binary_expression(grandparent.as_ref().unwrap_or(&parent))
                    && symbol.declarations.iter().any(|d| d.id() == parent.id())
                {
                    return get_special_property_export(
                        grandparent.as_ref().unwrap_or(&parent),
                        false,
                    );
                }
                return None;
            }
            return export_info_of(
                export_symbol,
                get_export_kind_for_declaration(&parent),
            );
        } else {
            let export_node = get_export_node(&parent, node);
            if let Some(export_node) = &export_node {
                if node.has_syntactic_modifier(ast::ModifierFlags::Export)
                    || is_implicitly_exported_jsdoc_declaration(export_node)
                {
                    if ast::is_import_equals_declaration(export_node)
                        && import_equals_module_reference_is(export_node, node)
                    {
                        if coming_from_export {
                            return None;
                        }
                        let lhs_symbol = export_node
                            .name()
                            .and_then(|name| checker.get_symbol_at_location(name));
                        return Some(M5uImportExportSymbol {
                            kind: M5uImpExpKind::Import,
                            symbol: lhs_symbol,
                            export_info: None,
                        });
                    }
                    return export_info_of(
                        symbol,
                        get_export_kind_for_declaration(export_node),
                    );
                }
            }
            if ast::is_namespace_export(&parent) {
                return export_info_of(symbol, M5uExportKind::Named);
            }
            if ast::is_export_assignment(&parent) {
                return get_export_assignment_export(&parent);
            }
            if grandparent
                .as_ref()
                .map_or(false, |g| ast::is_export_assignment(g))
            {
                return get_export_assignment_export(grandparent.as_ref().unwrap());
            }
            if grandparent
                .as_ref()
                .map_or(false, |g| ast::is_namespace_export(g))
            {
                return export_info_of(symbol, M5uExportKind::Named);
            }
            if ast::is_binary_expression(&parent) {
                return get_special_property_export(&parent, true);
            }
            if grandparent
                .as_ref()
                .map_or(false, |g| ast::is_binary_expression(g))
            {
                return get_special_property_export(grandparent.as_ref().unwrap(), true);
            }
            if ast::is_jsdoc_typedef_tag(&parent) || ast::is_jsdoc_callback_tag(&parent) {
                return export_info_of(symbol, M5uExportKind::Named);
            }
        }
        None
    };

    fn get_import(
        node: &Arc<Node>,
        symbol: &Arc<Symbol>,
        checker: &mut Checker,
    ) -> Option<M5uImportExportSymbol> { ::tsox_core::fntrace::enter("get_import"); 
        if !is_node_import(node) {
            return None;
        }
        let mut imported_symbol = if symbol.flags & ast::SymbolFlags::Alias != ast::SymbolFlags::None
        {
            checker.get_immediate_aliased_symbol(symbol)?
        } else {
            get_property_symbol_of_object_binding_pattern_without_property_name(symbol, checker)?
        };
        imported_symbol = skip_export_specifier_symbol(&imported_symbol, checker);
        if imported_symbol.name == "export=" {
            imported_symbol = get_export_equals_local_symbol(&imported_symbol, checker)?;
        }
        let imported_name = symbol_name_no_default(&imported_symbol);
        if imported_name.is_empty()
            || imported_name == "default"
            || imported_name == symbol.name
        {
            return Some(M5uImportExportSymbol {
                kind: M5uImpExpKind::Import,
                symbol: Some(imported_symbol),
                export_info: None,
            });
        }
        None
    }

    let result = get_export();
    if result.is_none() && !coming_from_export {
        return get_import(node, symbol, checker);
    }
    result
}

pub fn import_equals_module_reference_is(declaration: &Arc<Node>, node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("import_equals_module_reference_is"); 
    node_import_equals_module_reference(declaration)
        .map_or(false, |module_reference| module_reference.id() == node.id())
}

pub fn get_export_info(
    export_symbol: &Arc<Symbol>,
    export_kind: M5uExportKind,
    checker: &Checker,
) -> Option<M5uExportInfo> { ::tsox_core::fntrace::enter("get_export_info"); 
    if let Some(parent) = export_symbol.parent() {
        let exporting_module_symbol = checker.get_merged_symbol(&parent);
        if tsox_checker::checker::is_external_module_symbol(&exporting_module_symbol) {
            return Some(M5uExportInfo {
                exporting_module_symbol,
                export_kind,
            });
        }
    }
    None
}

pub fn get_export_node(parent: &Arc<Node>, node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("get_export_node"); 
    let declaration: Option<Arc<Node>> = if ast::is_variable_declaration(parent) {
        Some(Arc::clone(parent))
    } else if ast::is_binding_element(parent) {
        walk_up_binding_elements_and_patterns(parent)
    } else {
        None
    };
    if let Some(declaration) = declaration {
        let matches_name = declaration
            .name()
            .map_or(false, |n| n.id() == node.id());
        if matches_name {
            if let Some(declaration_parent) = declaration.parent() {
                if !ast::is_catch_clause(&declaration_parent) {
                    if let Some(declaration_grand) = declaration_parent.parent() {
                        if ast::is_variable_statement(&declaration_grand) {
                            return Some(declaration_grand);
                        }
                    }
                }
            }
        }
        return None;
    }
    Some(Arc::clone(parent))
}

pub fn is_node_import(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_node_import"); 
    let Some(parent) = node.parent() else {
        return false;
    };
    match parent.kind {
        SyntaxKind::ImportEqualsDeclaration => {
            parent
                .name()
                .map_or(false, |name| name.id() == node.id())
                && is_external_module_import_equals(&parent)
        }
        SyntaxKind::ImportSpecifier => import_specifier_property_name(&parent).is_none(),
        SyntaxKind::ImportClause | SyntaxKind::NamespaceImport => {
            parent
                .name()
                .map_or(false, |name| name.id() == node.id())
        }
        SyntaxKind::BindingElement => {
            ast::is_in_js_file(node)
                && parent
                    .parent()
                    .and_then(|p| p.parent())
                    .map_or(false, |pp| {
                        is_variable_declaration_initialized_to_bare_or_accessed_require(&pp)
                    })
        }
        _ => false,
    }
}

pub fn is_external_module_import_equals(node: &Arc<Node>) -> bool { ::tsox_core::fntrace::enter("is_external_module_import_equals"); 
    node_import_equals_module_reference(node).map_or(false, |module_reference| {
        ast::is_external_module_reference(&module_reference)
            && module_reference
                .expression()
                .map_or(false, |expr| expr.kind == SyntaxKind::StringLiteral)
    })
}

pub fn skip_export_specifier_symbol(
    symbol: &Arc<Symbol>,
    checker: &mut Checker,
) -> Arc<Symbol> { ::tsox_core::fntrace::enter("skip_export_specifier_symbol"); 
    for declaration in &symbol.declarations {
        if ast::is_export_specifier(declaration)
            && import_specifier_property_name(declaration).is_none()
            && declaration
                .parent()
                .and_then(|p| p.parent())
                .and_then(|pp| node_module_specifier(&pp))
                .is_none()
        {
            return checker
                .get_export_specifier_local_target_symbol(declaration)
                .unwrap_or_else(|| Arc::clone(symbol));
        }
        if ast::is_property_access_expression(declaration)
            && declaration
                .expression()
                .map_or(false, |expr| is_module_exports_access_expression(expr))
            && declaration
                .name()
                .map_or(true, |name| !ast::is_private_identifier(name))
        {
            if let Some(sym) = checker.get_symbol_at_location(declaration) {
                return sym;
            }
        }
        if ast::is_shorthand_property_assignment(declaration)
            && declaration
                .parent()
                .and_then(|p| p.parent())
                .map_or(false, |pp| {
                    ast::is_binary_expression(&pp)
                        && get_assignment_declaration_kind(&pp)
                            == JsDeclarationKind::ModuleExports
                })
        {
            if let Some(name) = declaration.name() {
                if let Some(sym) = checker.get_export_specifier_local_target_symbol(&name) {
                    return sym;
                }
            }
        }
    }
    Arc::clone(symbol)
}

pub fn get_export_equals_local_symbol(
    imported_symbol: &Arc<Symbol>,
    checker: &mut Checker,
) -> Option<Arc<Symbol>> { ::tsox_core::fntrace::enter("get_export_equals_local_symbol"); 
    if imported_symbol.flags & ast::SymbolFlags::Alias != ast::SymbolFlags::None {
        return checker.get_immediate_aliased_symbol(imported_symbol);
    }
    let decl = imported_symbol.value_declaration.as_ref()?;
    if ast::is_export_assignment(decl) {
        return decl.expression().and_then(|expr| symbol_of_node(expr));
    }
    if ast::is_binary_expression(decl) {
        return binary_expression_right(decl).and_then(|right| symbol_of_node(&right));
    }
    if ast::is_source_file(decl) {
        return symbol_of_node(decl);
    }
    None
}

pub fn binary_expression_right(node: &Arc<Node>) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("binary_expression_right"); 
    match &node.data {
        NodeData::BinaryExpression(d) => Some(Arc::clone(&d.right)),
        _ => None,
    }
}

pub fn symbol_name_no_default(symbol: &Arc<Symbol>) -> String { ::tsox_core::fntrace::enter("symbol_name_no_default"); 
    if symbol.name != "default" {
        return symbol.name.clone();
    }
    for decl in &symbol.declarations {
        if let Some(name) = ast::get_name_of_declaration(decl) {
            if ast::is_identifier(&name) {
                return name.text().to_string();
            }
        }
    }
    String::new()
}

pub fn find_module_references(
    program: &Program,
    source_files: &[Arc<SourceFile>],
    search_module_symbol: &Arc<Symbol>,
    checker: &Checker,
) -> Vec<M5uModuleReference> { ::tsox_core::fntrace::enter("find_module_references"); 
    let mut refs: Vec<M5uModuleReference> = Vec::new();

    for referencing_file in source_files {
        let search_source_file = search_module_symbol.value_declaration.clone();
        if let Some(search_source_file) = &search_source_file {
            if search_source_file.kind == SyntaxKind::SourceFile {
                for ref_directive in &referencing_file.referenced_files {
                    if let Some(resolved) = program.get_source_file_from_reference(
                        referencing_file,
                        ref_directive,
                    ) {
                        if resolved.id() == node_as_source_file(search_source_file).map_or(0, |f| f.id())
                        {
                            refs.push(M5uModuleReference {
                                kind: M5uModuleReferenceKind::Reference,
                                literal: None,
                                referencing_file: Some(Arc::clone(referencing_file)),
                                ref_directive: Some(ref_directive.clone()),
                            });
                        }
                    }
                }
                for ref_directive in &referencing_file.type_reference_directives {
                    let referenced = program
                        .get_resolved_type_reference_directive_from_type_reference_directive(
                            ref_directive,
                            referencing_file,
                        );
                    if let Some(referenced) = referenced {
                        if referenced.resolved_file_name == search_source_file_text(referencing_file)
                        {
                            refs.push(M5uModuleReference {
                                kind: M5uModuleReferenceKind::Reference,
                                literal: None,
                                referencing_file: Some(Arc::clone(referencing_file)),
                                ref_directive: Some(ref_directive.clone()),
                            });
                        }
                    }
                }
            }
        }

        for_each_import(program, referencing_file, &mut |import_decl, module_specifier| {
            let module_symbol = checker.get_symbol_at_location(module_specifier);
            if module_symbol
                .as_ref()
                .map_or(false, |s| Arc::ptr_eq(s, search_module_symbol))
            {
                if ast::node_is_synthesized(import_decl) {
                    refs.push(M5uModuleReference {
                        kind: M5uModuleReferenceKind::Implicit,
                        literal: Some(Arc::clone(module_specifier)),
                        referencing_file: Some(Arc::clone(referencing_file)),
                        ref_directive: None,
                    });
                } else {
                    refs.push(M5uModuleReference {
                        kind: M5uModuleReferenceKind::Import,
                        literal: Some(Arc::clone(module_specifier)),
                        referencing_file: None,
                        ref_directive: None,
                    });
                }
            }
        });
    }

    refs
}

pub fn search_source_file_text(referencing_file: &Arc<SourceFile>) -> String { ::tsox_core::fntrace::enter("search_source_file_text"); 
    referencing_file.file_name.clone()
}

// 缺失依赖,按命名约定直接调用,不在此定义:
// - get_property_symbol_of_object_binding_pattern_without_property_name(symbol, checker)
// - is_source_file_with_global_exports(node)
