#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile};
use tsox_core::core;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleDetectionKind};
use tsox_core::core::tristate::Tristate;
use tsox_core::tspath;

use crate::ls::autoimport::AddAsTypeOnly;
use crate::ls::autoimport::AutoImportFix;
use crate::ls::autoimport::AutoImportFixKind;
use crate::ls::autoimport::ImportKind;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_export::ModuleID;
use crate::ls::autoimport_fix::Fix;
use crate::ls::autoimport_specifiers::ResultKind;
use crate::ls::autoimport_util::try_get_module_id_and_file_name_of_module_symbol;
use crate::ls::autoimport_view::ExistingImport;
use crate::ls::autoimport_view::View;
use crate::ls::change_tracker_tracker::Tracker;
use crate::ls::lsutil_user_preferences::UserPreferences;
use crate::mig::m5o2::get_import_kind;
use crate::mig::m5o2::get_add_to_existing_import_fix;
use crate::mig::m5o2::m5o2_ext::M5o2NodeExt;
use crate::mig::m5o2::m5o2_ext::M5o2NodeFactoryExt;
use crate::mig::m5o2::m5o2_ext::M5o2TrackerExt;
use crate::mig::m5o2::ordering_of_i32;
use crate::mig::m5o2::should_use_type_only;
use tsox_compile::compiler::Program;
use tsox_core::collections::multimap::MultiMap;

impl View {
    pub fn try_use_existing_namespace_import(
        &self,
        export: &Export,
        usage_position: Option<&crate::lsp::lsproto::Position>,
    ) -> Option<Fix> { ::tsox_core::fntrace::enter("try_use_existing_namespace_import"); 
        let usage_position = usage_position?;

        if get_import_kind(&self.importing_file, export, &self.program, false) != ImportKind::Named {
            return None;
        }

        let existing_imports = self.get_existing_imports();
        let matching_declarations = existing_imports.get(&export.export_id.module_id);
        for existing_import in matching_declarations {
            let namespace_prefix = crate::ls::autoimport_fix::get_namespace_like_import_text(&existing_import.node);
            if namespace_prefix.is_empty() || existing_import.module_specifier.is_empty() {
                continue;
            }
            return Some(Fix {
                auto_import_fix: AutoImportFix {
                    kind: AutoImportFixKind::UseNamespace,
                    name: export.name().to_string(),
                    module_specifier: existing_import.module_specifier.clone(),
                    import_kind: ImportKind::Namespace,
                    add_as_type_only: AddAsTypeOnly::Allowed,
                    import_index: existing_import.index as i32,
                    usage_position: Some(usage_position.clone()),
                    namespace_prefix,
                    ..Default::default()
                },
                ..Default::default()
            });
        }

        None
    }

    pub fn try_add_to_existing_import(
        &self,
        export: &Export,
        is_valid_type_only_use_site: bool,
    ) -> Option<Fix> { ::tsox_core::fntrace::enter("try_add_to_existing_import"); 
        let existing_imports = self.get_existing_imports();
        let matching_declarations = existing_imports.get(&export.export_id.module_id);
        if matching_declarations.is_empty() {
            return None;
        }

        if ast::is_source_file_js(&self.importing_file)
            && !export.flags.intersects(ast::SymbolFlags::VALUE)
            && !matching_declarations.iter().all(|i| ast::is_jsdoc_import_tag(&i.node))
        {
            return None;
        }

        let import_kind = get_import_kind(&self.importing_file, export, &self.program, false);
        if import_kind == ImportKind::CommonJS || import_kind == ImportKind::Namespace {
            return None;
        }

        let add_as_type_only = crate::ls::autoimport_fix::get_add_as_type_only(
            is_valid_type_only_use_site,
            export,
            self.program.options(),
        );

        let mut best: Option<Fix> = None;
        for existing_import in matching_declarations {
            if existing_import.node.kind == ast::SyntaxKind::ImportEqualsDeclaration {
                continue;
            }

            if existing_import.node.kind == ast::SyntaxKind::VariableDeclaration {
                if (import_kind == ImportKind::Named || import_kind == ImportKind::Default)
                    && existing_import.node.name().map(|n| n.kind) == Some(ast::SyntaxKind::ObjectBindingPattern)
                {
                    let fix = Fix {
                        auto_import_fix: AutoImportFix {
                            kind: AutoImportFixKind::AddToExisting,
                            name: export.name().to_string(),
                            import_kind,
                            import_index: existing_import.index as i32,
                            module_specifier: existing_import.module_specifier.clone(),
                            add_as_type_only,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    if add_as_type_only == AddAsTypeOnly::NotAllowed {
                        return Some(fix);
                    }
                    if best.is_none() {
                        best = Some(fix);
                    }
                }
                continue;
            }

            let import_clause_node = existing_import.node.import_clause();
            if import_clause_node.is_none()
                || !ast::is_string_literal_like(&existing_import.node.module_specifier())
            {
                continue;
            }
            let import_clause = import_clause_node.unwrap();

            let named_bindings = import_clause.named_bindings();
            if import_clause.is_type_only() && !(import_kind == ImportKind::Named && named_bindings.is_some()) {
                continue;
            }

            if import_kind == ImportKind::Default
                && (import_clause.name().is_some()
                    || add_as_type_only == AddAsTypeOnly::Required && named_bindings.is_some())
            {
                continue;
            }

            if import_kind == ImportKind::Named
                && named_bindings.as_ref().map(|n| n.kind) == Some(ast::SyntaxKind::NamespaceImport)
            {
                continue;
            }

            let fix = Fix {
                auto_import_fix: AutoImportFix {
                    kind: AutoImportFixKind::AddToExisting,
                    name: export.name().to_string(),
                    import_kind,
                    import_index: existing_import.index as i32,
                    module_specifier: existing_import.module_specifier.clone(),
                    add_as_type_only,
                    ..Default::default()
                },
                ..Default::default()
            };

            let is_type_only = import_clause.is_type_only();
            if (add_as_type_only != AddAsTypeOnly::NotAllowed && is_type_only)
                || (add_as_type_only == AddAsTypeOnly::NotAllowed && !is_type_only)
            {
                return Some(fix);
            }
            if best.is_none() {
                best = Some(fix);
            }
        }

        best
    }

    pub fn get_existing_imports(&self) -> MultiMap<ModuleID, ExistingImport> { ::tsox_core::fntrace::enter("get_existing_imports"); 
        let mut result: MultiMap<ModuleID, ExistingImport> = MultiMap::new();
        let ch = self.program.get_type_checker();

        for (i, module_specifier) in self.importing_file.imports.iter().enumerate() {
            let node = match ast::mig::m3g_3::try_get_import_from_module_specifier(module_specifier) {
                Some(n) => n,
                None => panic!("error: did not expect node kind {:?}", module_specifier.kind),
            };
            if node
                .parent()
                .as_ref()
                .map_or(false, |p| ast::mig::m3g_3::is_variable_declaration_initialized_to_require(p))
            {
                if let Some(module_symbol) = ch.resolve_external_module_name(module_specifier) {
                    if let Some((module_id, _)) = try_get_module_id_and_file_name_of_module_symbol(&module_symbol) {
                        result.add(
                            module_id,
                            ExistingImport {
                                node: node.parent().expect("require declaration parent"),
                                module_specifier: module_specifier.text().to_string(),
                                index: i,
                            },
                        );
                    }
                }
            } else if node.kind == ast::SyntaxKind::ImportDeclaration
                || node.kind == ast::SyntaxKind::ImportEqualsDeclaration
                || node.kind == ast::SyntaxKind::JSDocImportTag
            {
                if let Some(module_symbol) = ch.get_symbol_at_location(module_specifier) {
                    if let Some((module_id, _)) = try_get_module_id_and_file_name_of_module_symbol(&module_symbol) {
                        result.add(
                            module_id,
                            ExistingImport {
                                node: node.clone(),
                                module_specifier: module_specifier.text().to_string(),
                                index: i,
                            },
                        );
                    }
                }
            }
        }
        result
    }

    pub fn compare_module_specifiers_for_ranking(&self, a: &Fix, b: &Fix) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_module_specifiers_for_ranking"); 
        let comparison = crate::ls::autoimport_fix::compare_module_specifier_relativity(a, b, &self.preferences);
        if comparison != std::cmp::Ordering::Equal {
            return comparison;
        }
        if a.module_specifier_kind == ResultKind::Ambient && b.module_specifier_kind == ResultKind::Ambient {
            let comparison = self.compare_node_core_module_specifiers(
                &a.auto_import_fix.module_specifier,
                &b.auto_import_fix.module_specifier,
                &self.importing_file,
                &self.program,
            );
            if comparison != std::cmp::Ordering::Equal {
                return comparison;
            }
        }
        if a.module_specifier_kind == ResultKind::Relative && b.module_specifier_kind == ResultKind::Relative {
            let comparison = core::mig::m3j_2::compare_booleans(
                crate::ls::autoimport_fix::is_fix_possibly_re_exporting_importing_file(a, &self.importing_file.file_name),
                crate::ls::autoimport_fix::is_fix_possibly_re_exporting_importing_file(b, &self.importing_file.file_name),
            );
            if comparison != 0 {
                return ordering_of_i32(comparison);
            }
        }
        ordering_of_i32(tspath::mig::m3i::compare_number_of_directory_separators(&a.auto_import_fix.module_specifier, &b.auto_import_fix.module_specifier))
    }

    pub fn compare_module_specifiers_for_sorting(&self, a: &Fix, b: &Fix) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_module_specifiers_for_sorting"); 
        let res = self.compare_module_specifiers_for_ranking(a, b);
        if res != std::cmp::Ordering::Equal {
            return res;
        }
        let a_spec = &a.auto_import_fix.module_specifier;
        let b_spec = &b.auto_import_fix.module_specifier;
        if a_spec.starts_with("./") && !b_spec.starts_with("./") {
            return std::cmp::Ordering::Less;
        }
        if b_spec.starts_with("./") && !a_spec.starts_with("./") {
            return std::cmp::Ordering::Greater;
        }
        let comparison = a_spec.cmp(b_spec);
        if comparison != std::cmp::Ordering::Equal {
            return comparison;
        }
        a.auto_import_fix.import_kind.cmp(&b.auto_import_fix.import_kind)
    }

    pub fn compare_node_core_module_specifiers(
        &self,
        a: &str,
        b: &str,
        importing_file: &SourceFile,
        program: &Program,
    ) -> std::cmp::Ordering { ::tsox_core::fntrace::enter("compare_node_core_module_specifiers"); 
        if a.starts_with("node:") && !b.starts_with("node:") {
            if self.should_use_uri_style_node_core_modules.is_true() {
                return std::cmp::Ordering::Less;
            } else if self.should_use_uri_style_node_core_modules.is_false() {
                return std::cmp::Ordering::Greater;
            }
            return std::cmp::Ordering::Equal;
        }
        if b.starts_with("node:") && !a.starts_with("node:") {
            if self.should_use_uri_style_node_core_modules.is_true() {
                return std::cmp::Ordering::Greater;
            } else if self.should_use_uri_style_node_core_modules.is_false() {
                return std::cmp::Ordering::Less;
            }
        }
        std::cmp::Ordering::Equal
    }
}

pub fn detect_syntax_indicators(file: &SourceFile, options: &CompilerOptions) -> (bool, bool) { ::tsox_core::fntrace::enter("detect_syntax_indicators"); 
    let has_cjs = file.common_js_module_indicator.is_some();
    if options.get_emit_module_detection_kind() != ModuleDetectionKind::Force {
        let has_esm = file.external_module_indicator.is_some();
        return (has_esm, has_cjs);
    }
    if let Some(indicator) = &file.external_module_indicator {
        if !Arc::ptr_eq(indicator, &file.node) {
            return (true, has_cjs);
        }
    }
    for imp in &file.imports {
        if imp.flags.intersects(ast::NodeFlags::Synthesized) {
            continue;
        }
        let parent = imp.parent().expect("import parent");
        match parent.kind {
            ast::SyntaxKind::ImportDeclaration
            | ast::SyntaxKind::JSImportDeclaration
            | ast::SyntaxKind::ExportDeclaration => return (true, has_cjs),
            ast::SyntaxKind::ExternalModuleReference => return (true, has_cjs),
            _ => {}
        }
    }
    (false, has_cjs)
}

pub fn promote_from_type_only(
    changes: &mut Tracker,
    alias_declaration: &Arc<Node>,
    compiler_options: &CompilerOptions,
    source_file: &SourceFile,
    preferences: &UserPreferences,
) -> Arc<Node> { ::tsox_core::fntrace::enter("promote_from_type_only"); 
    let convert_existing_to_type_only = compiler_options.verbatim_module_syntax;

    match alias_declaration.kind {
        ast::SyntaxKind::ImportSpecifier => {
            if alias_declaration.is_type_only() {
                let parent = alias_declaration.parent().expect("parent");
                if parent.kind == ast::SyntaxKind::NamedImports {
                    let elements = parent.elements();
                    if elements.len() > 1 {
                        let property_name = alias_declaration
                            .property_name()
                            .map(|p| changes.node_factory().new_identifier(p.text()));
                        let new_specifier = changes.node_factory().new_import_specifier(
                            false,
                            property_name,
                            changes.node_factory().new_identifier(alias_declaration.name().expect("name").text()),
                        );
                        let (specifier_comparer, _) = crate::ls::lsutil_organize_imports_comparers::get_named_import_specifier_comparer_with_detection(
                            &parent.parent().expect("parent").parent().expect("parent"),
                            Some(source_file),
                            preferences,
                        );
                        let insertion_index = crate::ls::lsutil_organize_imports_imports::get_import_specifier_insertion_index(
                            &elements,
                            &new_specifier,
                            &specifier_comparer,
                        );
                        let current_index = elements.iter().position(|e| Arc::ptr_eq(e, alias_declaration)).unwrap_or(0);
                        if insertion_index != current_index {
                            changes.delete(source_file, alias_declaration);
                            changes.insert_import_specifier_at_index(source_file, &new_specifier, &parent, insertion_index);
                            return alias_declaration.clone();
                        }
                    }
                    let first_token = crate::ls::lsutil_children::get_first_token(alias_declaration, source_file)
                        .expect("expected first token");
                    let type_keyword_pos = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&first_token, source_file, false);
                    let target_node = alias_declaration
                        .property_name()
                        .unwrap_or_else(|| alias_declaration.name().cloned().expect("name"));
                    let target_pos = tsox_frontend::scanner::mig::x5a::get_token_pos_of_node(&target_node, source_file, false);
                    changes.delete_range(source_file, tsox_core::core::text::TextRange::new(type_keyword_pos, target_pos));
                }
                alias_declaration.clone()
            } else {
                let parent = alias_declaration.parent().expect("parent");
                if parent.kind != ast::SyntaxKind::NamedImports {
                    panic!("ImportSpecifier parent must be NamedImports");
                }
                let import_clause = parent.parent().expect("parent");
                if import_clause.kind != ast::SyntaxKind::ImportClause {
                    panic!("NamedImports parent must be ImportClause");
                }
                promote_import_clause(
                    changes,
                    &import_clause,
                    compiler_options,
                    source_file,
                    preferences,
                    convert_existing_to_type_only,
                    Some(alias_declaration),
                );
                import_clause
            }
        }

        ast::SyntaxKind::ImportClause => {
            promote_import_clause(
                changes,
                alias_declaration,
                compiler_options,
                source_file,
                preferences,
                convert_existing_to_type_only,
                Some(alias_declaration),
            );
            alias_declaration.clone()
        }

        ast::SyntaxKind::NamespaceImport => {
            let parent = alias_declaration.parent().expect("parent");
            if parent.kind != ast::SyntaxKind::ImportClause {
                panic!("NamespaceImport parent must be ImportClause");
            }
            promote_import_clause(
                changes,
                &parent,
                compiler_options,
                source_file,
                preferences,
                convert_existing_to_type_only,
                Some(alias_declaration),
            );
            parent
        }

        ast::SyntaxKind::ImportEqualsDeclaration => {
            let mut scan = tsox_frontend::scanner::mig::m4d_2::get_scanner_for_source_file(source_file, alias_declaration.pos());
            scan.scan();
            delete_type_keyword(changes, source_file, scan.token_pos());
            alias_declaration.clone()
        }
        _ => panic!("Unexpected alias declaration kind"),
    }
}

pub fn promote_import_clause(
    changes: &mut Tracker,
    import_clause: &Arc<Node>,
    compiler_options: &CompilerOptions,
    source_file: &SourceFile,
    preferences: &UserPreferences,
    convert_existing_to_type_only: Tristate,
    alias_declaration: Option<&Arc<Node>>,
) { ::tsox_core::fntrace::enter("promote_import_clause"); 
    if import_clause.phase_modifier() == Some(ast::SyntaxKind::TypeKeyword) {
        delete_type_keyword(changes, source_file, import_clause.pos());
    }

    if compiler_options.allow_importing_ts_extensions.is_false() {
        if let Some(parent) = import_clause.parent() {
            let module_specifier = tsox_checker::checker::mig::m2g::try_get_module_specifier_from_declaration_worker(&parent);
            if module_specifier.is_some() {
                // ResolvedUsingTsExtension 校验依赖 program，此处上下文不可得
            }
        }
    }

    if convert_existing_to_type_only.is_true() {
        let named_imports = import_clause.named_bindings();
        if let Some(named_imports) = named_imports {
            if named_imports.kind == ast::SyntaxKind::NamedImports {
                let elements = named_imports.elements();
                if elements.len() > 1 {
                    let (_, is_sorted) = crate::ls::lsutil_organize_imports_comparers::get_named_import_specifier_comparer_with_detection(
                        &import_clause.parent().expect("parent"),
                        Some(source_file),
                        preferences,
                    );

                    let alias_is_specifier = alias_declaration
                        .map(|a| a.kind == ast::SyntaxKind::ImportSpecifier)
                        .unwrap_or(false);
                    if is_sorted != Tristate::False && alias_is_specifier {
                        let alias = alias_declaration.unwrap();
                        let alias_index = elements.iter().position(|e| Arc::ptr_eq(e, alias)).map(|i| i as i32).unwrap_or(-1);
                        if alias_index > 0 {
                            changes.delete(source_file, alias);
                            changes.insert_import_specifier_at_index(source_file, alias, &named_imports, 0);
                        }
                    }

                    for element in &elements {
                        if alias_is_specifier {
                            if let Some(alias) = alias_declaration {
                                if Arc::ptr_eq(element, alias) {
                                    continue;
                                }
                            }
                        }
                        if !element.is_type_only() {
                            changes.insert_modifier_before(source_file, ast::SyntaxKind::TypeKeyword, element);
                        }
                    }
                }
            }
        }
    }
}

pub fn get_module_specifier_text(promoted_declaration: &Arc<Node>) -> String { ::tsox_core::fntrace::enter("get_module_specifier_text"); 
    if promoted_declaration.kind == ast::SyntaxKind::ImportEqualsDeclaration {
        let module_reference = promoted_declaration.module_reference();
        if ast::is_external_module_reference(&module_reference) {
            if let Some(expr) = module_reference.expression() {
                if ast::is_string_literal_like(expr) {
                    return expr.text().to_string();
                }
                return tsox_frontend::scanner::mig::m3i::get_text_of_node(expr);
            }
        }
        return tsox_frontend::scanner::mig::m3i::get_text_of_node(&module_reference);
    }
    let module_specifier = promoted_declaration.parent().expect("parent").module_specifier();
    if ast::is_string_literal_like(&module_specifier) {
        return module_specifier.text().to_string();
    }
    tsox_frontend::scanner::mig::m3i::get_text_of_node(&module_specifier)
}

pub fn delete_type_keyword(changes: &mut Tracker, source_file: &SourceFile, start_pos: usize) { ::tsox_core::fntrace::enter("delete_type_keyword"); 
    let mut scan = tsox_frontend::scanner::mig::m4d_2::get_scanner_for_source_file(source_file, start_pos);
    if scan.token() != ast::SyntaxKind::TypeKeyword {
        return;
    }
    let type_start = scan.token_pos();
    let mut type_end = scan.token_end();
    let bytes = source_file.text.as_bytes();
    while type_end < bytes.len() && (bytes[type_end] == b' ' || bytes[type_end] == b'\t') {
        type_end += 1;
    }
    changes.delete_range(source_file, tsox_core::core::text::TextRange::new(type_start, type_end));
}
