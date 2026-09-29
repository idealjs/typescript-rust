#![allow(dead_code, unused_imports, unused_variables)]

use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::ast::node_data_generated::NodeData;
use tsox_frontend::ast::node_flags::NodeFlags;
use tsox_frontend::ast::node_node_list::NodeList;
use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
use tsox_frontend::ast::ModifierList;
use tsox_frontend::astnav;
use tsox_core::core;
use tsox_core::core::compiler_options::{CompilerOptions, ModuleKind};
use tsox_core::core::tristate::Tristate;
use tsox_core::diagnostics;
use tsox_core::locale::Locale;
use tsox_frontend::scanner::{TokenFlags, TOKEN_FLAGS_SINGLE_QUOTE};

use crate::ls::autoimport::AddAsTypeOnly;
use crate::ls::autoimport::AutoImportFix;
use crate::ls::autoimport::AutoImportFixKind;
use crate::ls::autoimport::ImportKind;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_export::ExportSyntax;
use crate::ls::autoimport_fix::AddToExistingImportFix;
use crate::ls::autoimport_fix::Fix;
use crate::ls::autoimport_fix::NewImportBinding;
use crate::ls::autoimport_view::ExistingImport;
use crate::ls::autoimport_view::View;
use crate::ls::change_tracker_tracker::Tracker;
use crate::ls::lsutil_user_preferences::UserPreferences;
use tsox_compile::compiler::Program;
use tsox_checker::checker::Program as _;

pub fn get_import_kind_for_import_statement(
    importing_file: &SourceFile,
    export: &Export,
    program: &Program,
) -> ImportKind {
    get_import_kind(importing_file, export, program, true)
}

pub fn get_import_kind(
    importing_file: &SourceFile,
    export: &Export,
    program: &Program,
    force_import_keyword: bool,
) -> ImportKind {
    if program.options().verbatim_module_syntax.is_true()
        && program.get_emit_module_format_of_file(&importing_file.file_name) == ModuleKind::CommonJS
    {
        return ImportKind::CommonJS;
    }
    match export.syntax {
        ExportSyntax::DefaultModifier | ExportSyntax::DefaultDeclaration => {
            ImportKind::Default
        }
        ExportSyntax::Named => {
            if export.export_id.export_name == ast::INTERNAL_SYMBOL_NAME_DEFAULT {
                return ImportKind::Default;
            }
            named_or_star_like()
        }
        ExportSyntax::Modifier | ExportSyntax::Star | ExportSyntax::CommonJSExportsProperty => {
            named_or_star_like()
        }
        ExportSyntax::Equals | ExportSyntax::CommonJSModuleExports | ExportSyntax::UMD => {
            if export.export_id.export_name != ast::INTERNAL_SYMBOL_NAME_EXPORT_EQUALS {
                return ImportKind::Named;
            }
            for statement in source_file_statements(importing_file) {
                if ast::is_import_equals_declaration(statement)
                    && !ast::node_is_missing(Some(&statement.module_reference()))
                {
                    return ImportKind::CommonJS;
                }
            }
            if importing_file.external_module_indicator.is_some()
                || force_import_keyword
                || !ast::is_source_file_js(importing_file)
            {
                return ImportKind::Default;
            }
            ImportKind::CommonJS
        }
        _ => panic!("unhandled export syntax kind"),
    }
}

fn named_or_star_like() -> ImportKind {
    ImportKind::Named
}

pub fn source_file_statements(file: &SourceFile) -> &[Arc<Node>] {
    match &file.node.data {
        NodeData::SourceFile(data) => &data.statements.nodes,
        _ => panic!("expected source file node"),
    }
}

pub fn file_edits(tracker: &mut Tracker, file: &SourceFile) -> (Vec<crate::lsp::lsproto::TextEdit>, bool) {
    let (changes, unmappable) = tracker.get_changes_with_unmappable();
    let edits = changes
        .get(&file.file_name)
        .cloned()
        .unwrap_or_default();
    (edits, unmappable.is_empty())
}

pub fn add_import_type(
    f: &Fix,
    file: &SourceFile,
    preferences: &UserPreferences,
    tracker: &mut Tracker,
    locale: &Locale,
) -> String {
    let usage_position = f.auto_import_fix.usage_position.clone().expect("UsagePosition must be set for JSDoc type import fix");
    let quote_preference = crate::ls::lsutil_utilities::get_quote_preference(file, preferences);
    let quote_char = if quote_preference == crate::ls::lsutil_user_preferences::QuotePreference::Single {
        "'"
    } else {
        "\""
    };
    let import_type_prefix = format!("import({}{}{}).", quote_char, f.auto_import_fix.module_specifier, quote_char);
    tracker.insert_text(file, usage_position, import_type_prefix.clone());
    let replacement = format!("{}{}", import_type_prefix, f.auto_import_fix.name);
    diagnostics::CHANGE_0_TO_1.localize(locale, &[&f.auto_import_fix.name, &replacement])
}

pub fn add_namespace_qualifier(
    f: &Fix,
    tracker: &mut Tracker,
    file: &SourceFile,
    locale: &Locale,
) -> String {
    let prefix = f.auto_import_fix.namespace_prefix.clone();
    if f.auto_import_fix.usage_position.is_none() || prefix.is_empty() {
        panic!("namespace fix requires usage position and prefix");
    }
    let qualified = format!("{}.{}", prefix, f.auto_import_fix.name);
    tracker.insert_text(file, f.auto_import_fix.usage_position.clone().unwrap(), format!("{}.", prefix));
    diagnostics::CHANGE_0_TO_1.localize(locale, &[&f.auto_import_fix.name, &qualified])
}

pub fn get_add_to_existing_import_fix(file: &SourceFile, fix: &Fix) -> AddToExistingImportFix {
    if fix.auto_import_fix.kind != AutoImportFixKind::AddToExisting {
        panic!("expected add to existing import fix");
    }
    let module_specifier = &file.imports[fix.auto_import_fix.import_index as usize];
    let import_node = ast::mig::m3g_3::try_get_import_from_module_specifier(module_specifier)
        .expect("expected import declaration");
    let import_clause_or_binding_pattern: Arc<Node> = match import_node.kind {
        ast::SyntaxKind::ImportDeclaration => import_node
            .import_clause()
            .expect("expected import clause"),
        ast::SyntaxKind::CallExpression => {
            if !import_node
                .parent()
                .as_ref()
                .map_or(false, |p| ast::mig::m3g_3::is_variable_declaration_initialized_to_require(p))
            {
                panic!("expected require call expression to be in variable declaration");
            }
            let name = import_node
                .parent()
                .and_then(|p| p.name().cloned())
                .expect("expected object binding pattern in variable declaration");
            if !ast::is_object_binding_pattern(&name) {
                panic!("expected object binding pattern in variable declaration");
            }
            name
        }
        _ => panic!("expected import declaration or require call expression"),
    };

    let default_import = if fix.auto_import_fix.import_kind == ImportKind::Default {
        Some(NewImportBinding {
            kind: ImportKind::Default,
            property_name: String::new(),
            name: fix.auto_import_fix.name.clone(),
            add_as_type_only: fix.auto_import_fix.add_as_type_only,
        })
    } else {
        None
    };
    let named_import = if fix.auto_import_fix.import_kind == ImportKind::Named {
        Some(NewImportBinding {
            kind: ImportKind::Named,
            property_name: String::new(),
            name: fix.auto_import_fix.name.clone(),
            add_as_type_only: fix.auto_import_fix.add_as_type_only,
        })
    } else {
        None
    };
    AddToExistingImportFix {
        import_clause_or_binding_pattern: Some(import_clause_or_binding_pattern),
        default_import,
        named_import,
    }
}

pub fn ordering_of_i32(value: i32) -> std::cmp::Ordering {
    value.cmp(&0)
}

pub fn add_to_existing_import(
    ct: &mut Tracker,
    file: &SourceFile,
    import_clause_or_binding_pattern: &Arc<Node>,
    default_import: Option<&NewImportBinding>,
    named_imports: &[NewImportBinding],
    preferences: &UserPreferences,
) {
    match import_clause_or_binding_pattern.kind {
        ast::SyntaxKind::ObjectBindingPattern => {
            if let Some(default_import) = default_import {
                add_element_to_binding_pattern(ct, file, import_clause_or_binding_pattern, &default_import.name, "default");
            }
            for named_import in named_imports {
                add_element_to_binding_pattern(ct, file, import_clause_or_binding_pattern, &named_import.name, "");
            }
        }
        ast::SyntaxKind::ImportClause => {
            let promote_from_type_only = import_clause_or_binding_pattern.is_type_only()
                && named_imports
                    .iter()
                    .chain(default_import.iter().copied())
                    .any(|i| i.add_as_type_only == AddAsTypeOnly::NotAllowed);

            let mut existing_specifiers: Vec<Arc<Node>> = Vec::new();
            if let Some(named_bindings) = import_clause_or_binding_pattern.named_bindings() {
                if named_bindings.kind == ast::SyntaxKind::NamedImports {
                    existing_specifiers = named_bindings.elements();
                }
            }

            if let Some(default_import) = default_import {
                let default_import_identifier = ct.node_factory().new_identifier(&default_import.name);
                ct.insert_node_at(
                    file,
                    astnav::get_start_of_node(import_clause_or_binding_pattern, file, false) as i32,
                    &default_import_identifier,
                    crate::ls::change_tracker_edit::NodeOptions { suffix: ", ".to_string(), ..Default::default() },
                );
            }

            if !named_imports.is_empty() {
                let (specifier_comparer, is_sorted) = crate::ls::lsutil_organize_imports_comparers::get_named_import_specifier_comparer_with_detection(
                    &import_clause_or_binding_pattern.parent().expect("parent"),
                    Some(file),
                    preferences,
                );
                let mut new_specifiers: Vec<Arc<Node>> = named_imports
                    .iter()
                    .map(|named_import| {
                        let identifier = if !named_import.property_name.is_empty() {
                            Some(ct.node_factory().new_identifier(&named_import.property_name))
                        } else {
                            None
                        };
                        ct.node_factory().new_import_specifier(
                            (!import_clause_or_binding_pattern.is_type_only() || promote_from_type_only)
                                && should_use_type_only(named_import.add_as_type_only, preferences),
                            identifier,
                            ct.node_factory().new_identifier(&named_import.name),
                        )
                    })
                    .collect();
                new_specifiers.sort_by(|a, b| ordering_of_i32(specifier_comparer(a, b)));
                if !existing_specifiers.is_empty() && is_sorted != Tristate::False {
                    let mut specs_to_compare_against: Vec<Arc<Node>> = existing_specifiers.clone();
                    if promote_from_type_only {
                        specs_to_compare_against = existing_specifiers
                            .iter()
                            .map(|e| {
                                let property_name = e.property_name();
                                ct.node_factory().new_import_specifier(true, property_name, e.name().cloned().expect("name"))
                            })
                            .collect();
                    }

                    for spec in &new_specifiers {
                        let insertion_index = crate::ls::lsutil_organize_imports_imports::get_import_specifier_insertion_index(
                            &specs_to_compare_against,
                            spec,
                            &specifier_comparer,
                        );
                        ct.insert_import_specifier_at_index(
                            file,
                            spec,
                            &import_clause_or_binding_pattern.named_bindings().expect("named bindings"),
                            insertion_index,
                        );
                    }
                } else if !existing_specifiers.is_empty() {
                    for spec in &new_specifiers {
                        ct.insert_node_in_list_after(
                            file,
                            &existing_specifiers[existing_specifiers.len() - 1],
                            spec,
                            None,
                        );
                    }
                } else if !new_specifiers.is_empty() {
                    let named_imports_node = ct
                        .node_factory()
                        .new_named_imports(ct.node_factory().new_node_list(new_specifiers.clone()));
                    if let Some(named_bindings) = import_clause_or_binding_pattern.named_bindings() {
                        ct.replace_node(file, &named_bindings, &named_imports_node, None);
                    } else {
                        if import_clause_or_binding_pattern.name().is_none() {
                            panic!("Import clause must have either named imports or a default import");
                        }
                        ct.insert_node_after(file, import_clause_or_binding_pattern.name().unwrap(), &named_imports_node);
                    }
                }
            }

            if promote_from_type_only {
                let type_keyword = get_type_keyword_of_type_only_import(import_clause_or_binding_pattern, file);
                ct.delete(file, &type_keyword);

                for specifier in &existing_specifiers {
                    if !specifier.is_type_only() {
                        ct.insert_modifier_before(file, ast::SyntaxKind::TypeKeyword, specifier);
                    }
                }
            }
        }
        _ => panic!("Unsupported clause kind for addToExistingImport"),
    }
}

pub fn get_type_keyword_of_type_only_import(
    import_clause: &Arc<Node>,
    source_file: &SourceFile,
) -> Arc<Node> {
    astnav::find_child_of_kind(import_clause, ast::SyntaxKind::TypeKeyword)
        .expect("type-only import clause should have a type keyword")
}

pub fn add_element_to_binding_pattern(
    ct: &mut Tracker,
    file: &SourceFile,
    binding_pattern: &Arc<Node>,
    name: &str,
    property_name: &str,
) {
    let element = ct.node_factory().new_binding_element(
        None,
        if property_name.is_empty() { None } else { Some(ct.node_factory().new_identifier(property_name)) },
        ct.node_factory().new_identifier(name),
        None,
    );
    let elements = binding_pattern.elements();
    if !elements.is_empty() {
        let containing_list = binding_pattern.binding_elements_list().map(|l| l.as_ref());
        ct.insert_node_in_list_after(file, &elements[elements.len() - 1], &element, containing_list);
    } else {
        ct.replace_node(
            file,
            binding_pattern,
            &ct.node_factory()
                .new_binding_pattern(ast::SyntaxKind::ObjectBindingPattern, ct.node_factory().new_node_list(vec![element])),
            None,
        );
    }
}

pub fn get_new_imports(
    ct: &mut Tracker,
    module_specifier: &str,
    quote_preference: crate::ls::lsutil_user_preferences::QuotePreference,
    default_import: Option<&NewImportBinding>,
    named_imports: &[NewImportBinding],
    namespace_like_import: Option<&NewImportBinding>,
    compiler_options: &CompilerOptions,
    preferences: &UserPreferences,
) -> Vec<Arc<Node>> {
    let token_flags: TokenFlags = if quote_preference == crate::ls::lsutil_user_preferences::QuotePreference::Single {
        TOKEN_FLAGS_SINGLE_QUOTE
    } else {
        0
    };
    let module_specifier_string_literal = ct.node_factory().new_string_literal(module_specifier, token_flags);
    let mut statements: Vec<Arc<Node>> = Vec::new();
    if default_import.is_some() || !named_imports.is_empty() {
        let every_named_needs_type_only = named_imports.iter().all(|i| needs_type_only(i.add_as_type_only));
        let no_named_disallow = !named_imports.iter().any(|i| i.add_as_type_only == AddAsTypeOnly::NotAllowed);
        let default_ok = default_import.map_or(true, |d| d.add_as_type_only != AddAsTypeOnly::NotAllowed);
        let top_level_type_only = (default_import.is_none() || default_import.map_or(false, |d| needs_type_only(d.add_as_type_only)))
            && every_named_needs_type_only
            || (compiler_options.verbatim_module_syntax.is_true()
                || preferences.prefer_type_only_auto_imports.is_true())
                && default_ok
                && no_named_disallow;

        let default_import_node = default_import.map(|d| ct.node_factory().new_identifier(&d.name));

        statements.push(make_import(
            ct,
            default_import_node.as_ref(),
            named_imports
                .iter()
                .map(|named_import| {
                    let named_import_property_name = if !named_import.property_name.is_empty() {
                        Some(ct.node_factory().new_identifier(&named_import.property_name))
                    } else {
                        None
                    };
                    ct.node_factory().new_import_specifier(
                        !top_level_type_only && should_use_type_only(named_import.add_as_type_only, preferences),
                        named_import_property_name,
                        ct.node_factory().new_identifier(&named_import.name),
                    )
                })
                .collect(),
            &module_specifier_string_literal,
            top_level_type_only,
        ));
    }

    if let Some(namespace_like_import) = namespace_like_import {
        let declaration = if namespace_like_import.kind == ImportKind::CommonJS {
            ct.node_factory().new_import_equals_declaration(
                None,
                should_use_type_only(namespace_like_import.add_as_type_only, preferences),
                ct.node_factory().new_identifier(&namespace_like_import.name),
                ct.node_factory().new_external_module_reference(&module_specifier_string_literal),
            )
        } else {
            ct.node_factory().new_import_declaration(
                None,
                Some(ct.node_factory().new_import_clause(
                    if should_use_type_only(namespace_like_import.add_as_type_only, preferences) {
                        ast::SyntaxKind::TypeKeyword
                    } else {
                        ast::SyntaxKind::Unknown
                    },
                    None,
                    Some(ct.node_factory().new_namespace_import(ct.node_factory().new_identifier(&namespace_like_import.name))),
                )),
                &module_specifier_string_literal,
                None,
            )
        };
        statements.push(declaration);
    }
    if statements.is_empty() {
        panic!("No statements to insert for new imports");
    }
    statements
}

pub fn get_new_requires(
    change_tracker: &mut Tracker,
    module_specifier: &str,
    quote_preference: crate::ls::lsutil_user_preferences::QuotePreference,
    default_import: Option<&NewImportBinding>,
    named_imports: &[NewImportBinding],
    namespace_like_import: Option<&NewImportBinding>,
    compiler_options: &CompilerOptions,
) -> Vec<Arc<Node>> {
    let token_flags: TokenFlags = if quote_preference == crate::ls::lsutil_user_preferences::QuotePreference::Single {
        TOKEN_FLAGS_SINGLE_QUOTE
    } else {
        0
    };
    let quoted_module_specifier = change_tracker.node_factory().new_string_literal(module_specifier, token_flags);
    let mut statements: Vec<Arc<Node>> = Vec::new();

    if default_import.is_some() || !named_imports.is_empty() {
        let mut binding_elements: Vec<Arc<Node>> = Vec::new();
        for named_import in named_imports {
            let property_name = if !named_import.property_name.is_empty() {
                Some(change_tracker.node_factory().new_identifier(&named_import.property_name))
            } else {
                None
            };
            binding_elements.push(change_tracker.node_factory().new_binding_element(
                None,
                property_name,
                change_tracker.node_factory().new_identifier(&named_import.name),
                None,
            ));
        }
        if let Some(default_import) = default_import {
            binding_elements.insert(
                0,
                change_tracker.node_factory().new_binding_element(
                    None,
                    Some(change_tracker.node_factory().new_identifier("default")),
                    change_tracker.node_factory().new_identifier(&default_import.name),
                    None,
                ),
            );
        }
        let declaration = create_const_equals_require_declaration(
            change_tracker,
            &change_tracker.node_factory().new_binding_pattern(
                ast::SyntaxKind::ObjectBindingPattern,
                change_tracker.node_factory().new_node_list(binding_elements),
            ),
            &quoted_module_specifier,
        );
        statements.push(declaration);
    }

    if let Some(namespace_like_import) = namespace_like_import {
        let declaration = create_const_equals_require_declaration(
            change_tracker,
            &change_tracker.node_factory().new_identifier(&namespace_like_import.name),
            &quoted_module_specifier,
        );
        statements.push(declaration);
    }

    statements
}

pub fn create_const_equals_require_declaration(
    change_tracker: &mut Tracker,
    name: &Arc<Node>,
    quoted_module_specifier: &Arc<Node>,
) -> Arc<Node> {
    change_tracker.node_factory().new_variable_statement(
        None,
        change_tracker.node_factory().new_variable_declaration_list(
            change_tracker.node_factory().new_node_list(vec![change_tracker
                .node_factory()
                .new_variable_declaration(
                    name.clone(),
                    None,
                    None,
                    Some(change_tracker.node_factory().new_call_expression(
                        change_tracker.node_factory().new_identifier("require"),
                        None,
                        None,
                        change_tracker.node_factory().new_node_list(vec![quoted_module_specifier.clone()]),
                    )),
                )]),
            NodeFlags::Const,
        ),
    )
}

pub fn insert_imports(
    ct: &mut Tracker,
    source_file: &SourceFile,
    imports: &[Arc<Node>],
    blank_line_between: bool,
    preferences: &UserPreferences,
) {
    let existing_import_statements: Vec<Arc<Node>> = if imports[0].kind == ast::SyntaxKind::VariableStatement {
        source_file_statements(source_file)
            .iter()
            .filter(|s| ast::mig::m3g_2::is_require_variable_statement(s))
            .cloned()
            .collect()
    } else {
        source_file_statements(source_file)
            .iter()
            .filter(|s| ast::is_any_import_syntax(s))
            .cloned()
            .collect()
    };
    let (comparer, is_sorted) =
        crate::ls::lsutil_organize_imports_comparers::get_organize_imports_string_comparer_with_detection(&existing_import_statements, preferences);
    let mut sorted_new_imports = imports.to_vec();
    sorted_new_imports.sort_by(|a, b| {
        ordering_of_i32(crate::ls::lsutil_organize_imports_imports::compare_imports_or_require_statements(a, b, &comparer))
    });

    if !existing_import_statements.is_empty() && is_sorted {
        for new_import in &sorted_new_imports {
            let insertion_index = crate::ls::lsutil_organize_imports_imports::get_import_declaration_insert_index(
                &existing_import_statements,
                new_import,
                &|a: &Arc<Node>, b: &Arc<Node>| {
                    crate::ls::lsutil_organize_imports_imports::compare_imports_or_require_statements(a, b, &comparer)
                },
            );
            if insertion_index == 0 {
                let leading_trivia_option = if Arc::ptr_eq(&existing_import_statements[0], &source_file_statements(source_file)[0]) {
                    crate::ls::change_tracker_edit::LeadingTriviaOption::Exclude
                } else {
                    crate::ls::change_tracker_edit::LeadingTriviaOption::None
                };
                ct.insert_node_before(source_file, &existing_import_statements[0], new_import, false, leading_trivia_option);
            } else {
                let prev_import = &existing_import_statements[insertion_index - 1];
                ct.insert_node_after(source_file, prev_import, new_import);
            }
        }
    } else if !existing_import_statements.is_empty() {
        let last = existing_import_statements[existing_import_statements.len() - 1].clone();
        ct.insert_nodes_after(source_file, &last, &sorted_new_imports);
    } else {
        ct.insert_at_top_of_file(source_file, &sorted_new_imports, blank_line_between);
    }
}

pub fn make_import(
    ct: &mut Tracker,
    default_import: Option<&Arc<Node>>,
    named_imports: Vec<Arc<Node>>,
    module_specifier: &Arc<Node>,
    is_type_only: bool,
) -> Arc<Node> {
    let mut new_named_imports: Option<Arc<Node>> = None;
    if !named_imports.is_empty() {
        new_named_imports = Some(ct.node_factory().new_named_imports(ct.node_factory().new_node_list(named_imports)));
    }
    let import_clause = if default_import.is_some() || new_named_imports.is_some() {
        Some(ct.node_factory().new_import_clause(
            if is_type_only { ast::SyntaxKind::TypeKeyword } else { ast::SyntaxKind::Unknown },
            default_import.cloned(),
            new_named_imports,
        ))
    } else {
        None
    };
    ct.node_factory().new_import_declaration(None, import_clause, module_specifier, None)
}

pub fn needs_type_only(add_as_type_only: AddAsTypeOnly) -> bool {
    add_as_type_only == AddAsTypeOnly::Required
}

pub fn should_use_type_only(add_as_type_only: AddAsTypeOnly, preferences: &UserPreferences) -> bool {
    needs_type_only(add_as_type_only)
        || add_as_type_only != AddAsTypeOnly::NotAllowed && preferences.prefer_type_only_auto_imports.is_true()
}

pub mod m5o2_ext {
    use std::sync::Arc;

    use tsox_frontend::ast::node_data_generated::NodeData;
    use tsox_frontend::ast::node_flags::NodeFlags;
    use tsox_frontend::ast::node_node_list::NodeList;
    use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
    use tsox_frontend::ast::ModifierList;
    use tsox_frontend::ast::Node;

    use crate::ls::change_tracker_tracker::Tracker;
    use tsox_frontend::ast::mig::m3b_2::NodeFactory;

    pub trait M5o2NodeExt {
        fn is_type_only(&self) -> bool;
        fn import_clause(&self) -> Option<Arc<Node>>;
        fn module_specifier(&self) -> Arc<Node>;
        fn module_reference(&self) -> Arc<Node>;
        fn named_bindings(&self) -> Option<Arc<Node>>;
        fn elements(&self) -> Vec<Arc<Node>>;
        fn binding_elements_list(&self) -> Option<&Arc<NodeList>>;
        fn property_name(&self) -> Option<Arc<Node>>;
        fn phase_modifier(&self) -> Option<SyntaxKind>;
    }

    impl M5o2NodeExt for Arc<Node> {
        fn is_type_only(&self) -> bool {
            match &self.data {
                NodeData::ImportClause(d) => d.phase_modifier == Some(SyntaxKind::TypeKeyword),
                NodeData::ImportSpecifier(d) => d.is_type_only,
                NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
                _ => false,
            }
        }

        fn import_clause(&self) -> Option<Arc<Node>> {
            match &self.data {
                NodeData::ImportDeclaration(d) => d.import_clause.clone(),
                _ => None,
            }
        }

        fn module_specifier(&self) -> Arc<Node> {
            match &self.data {
                NodeData::ImportDeclaration(d) => d.module_specifier.clone(),
                _ => panic!("expected module specifier"),
            }
        }

        fn module_reference(&self) -> Arc<Node> {
            match &self.data {
                NodeData::ImportEqualsDeclaration(d) => d.module_reference.clone(),
                _ => panic!("expected module reference"),
            }
        }

        fn named_bindings(&self) -> Option<Arc<Node>> {
            match &self.data {
                NodeData::ImportClause(d) => d.named_bindings.clone(),
                _ => None,
            }
        }

        fn elements(&self) -> Vec<Arc<Node>> {
            match &self.data {
                NodeData::NamedImports(d) => d.elements.nodes.clone(),
                NodeData::NamedExports(d) => d.elements.nodes.clone(),
                NodeData::BindingPattern(d) => d.elements.nodes.clone(),
                _ => Vec::new(),
            }
        }

        fn binding_elements_list(&self) -> Option<&Arc<NodeList>> {
            match &self.data {
                NodeData::BindingPattern(d) => Some(&d.elements),
                _ => None,
            }
        }

        fn property_name(&self) -> Option<Arc<Node>> {
            match &self.data {
                NodeData::ImportSpecifier(d) => d.property_name.clone(),
                NodeData::BindingElement(d) => d.property_name.clone(),
                _ => None,
            }
        }

        fn phase_modifier(&self) -> Option<SyntaxKind> {
            match &self.data {
                NodeData::ImportClause(d) => d.phase_modifier,
                _ => None,
            }
        }
    }

    pub trait M5o2TrackerExt {
        fn node_factory(&self) -> NodeFactory;
    }

    impl M5o2TrackerExt for Tracker {
        fn node_factory(&self) -> NodeFactory {
            NodeFactory::new()
        }
    }

    pub trait M5o2NodeFactoryExt {
        fn new_import_declaration(
            &self,
            modifiers: Option<Arc<ModifierList>>,
            import_clause: Option<Arc<Node>>,
            module_specifier: &Arc<Node>,
            attributes: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn new_import_clause(
            &self,
            is_type_only: SyntaxKind,
            name: Option<Arc<Node>>,
            named_bindings: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn new_named_imports(&self, elements: NodeList) -> Arc<Node>;
        fn new_import_specifier(
            &self,
            is_type_only: bool,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
        ) -> Arc<Node>;
        fn new_namespace_import(&self, name: Arc<Node>) -> Arc<Node>;
        fn new_import_equals_declaration(
            &self,
            modifiers: Option<Arc<ModifierList>>,
            is_type_only: bool,
            name: Arc<Node>,
            module_reference: Arc<Node>,
        ) -> Arc<Node>;
        fn new_external_module_reference(&self, expression: &Arc<Node>) -> Arc<Node>;
        fn new_binding_element(
            &self,
            dot_dot_dot_token: Option<Arc<Node>>,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
            initializer: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn new_binding_pattern(&self, kind: SyntaxKind, elements: NodeList) -> Arc<Node>;
        fn new_variable_statement(
            &self,
            modifiers: Option<Arc<ModifierList>>,
            declaration_list: Arc<Node>,
        ) -> Arc<Node>;
        fn new_variable_declaration_list(&self, declarations: NodeList, flags: NodeFlags) -> Arc<Node>;
        fn new_variable_declaration(
            &self,
            name: Arc<Node>,
            exclamation_token: Option<Arc<Node>>,
            type_node: Option<Arc<Node>>,
            initializer: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn new_call_expression(
            &self,
            expression: Arc<Node>,
            question_dot_token: Option<Arc<Node>>,
            type_arguments: Option<Arc<NodeList>>,
            arguments: NodeList,
        ) -> Arc<Node>;
    }

    impl M5o2NodeFactoryExt for NodeFactory {
        fn new_import_declaration(
            &self,
            modifiers: Option<Arc<ModifierList>>,
            import_clause: Option<Arc<Node>>,
            module_specifier: &Arc<Node>,
            attributes: Option<Arc<Node>>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::ImportDeclaration,
                NodeData::ImportDeclaration(node_data_import_declaration(
                    modifiers,
                    import_clause,
                    module_specifier.clone(),
                    attributes,
                )),
            ))
        }

        fn new_import_clause(
            &self,
            is_type_only: SyntaxKind,
            name: Option<Arc<Node>>,
            named_bindings: Option<Arc<Node>>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::ImportClause,
                NodeData::ImportClause(import_clause_data(is_type_only, name, named_bindings)),
            ))
        }

        fn new_named_imports(&self, elements: NodeList) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::NamedImports,
                NodeData::NamedImports(tsox_frontend::ast::node_data_generated::NamedImportsData {
                    elements: Arc::new(elements),
                }),
            ))
        }

        fn new_import_specifier(
            &self,
            is_type_only: bool,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::ImportSpecifier,
                NodeData::ImportSpecifier(tsox_frontend::ast::node_data_generated::ImportSpecifierData {
                    is_type_only,
                    property_name,
                    name,
                }),
            ))
        }

        fn new_namespace_import(&self, name: Arc<Node>) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::NamespaceImport,
                NodeData::NamespaceImport(tsox_frontend::ast::node_data_generated::NamespaceImportData { name }),
            ))
        }

        fn new_import_equals_declaration(
            &self,
            modifiers: Option<Arc<ModifierList>>,
            is_type_only: bool,
            name: Arc<Node>,
            module_reference: Arc<Node>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::ImportEqualsDeclaration,
                NodeData::ImportEqualsDeclaration(tsox_frontend::ast::node_data_generated::ImportEqualsDeclarationData {
                    modifiers,
                    is_type_only,
                    name,
                    module_reference,
                }),
            ))
        }

        fn new_external_module_reference(&self, expression: &Arc<Node>) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::ExternalModuleReference,
                NodeData::ExternalModuleReference(tsox_frontend::ast::node_data_generated::ExternalModuleReferenceData {
                    expression: expression.clone(),
                }),
            ))
        }

        fn new_binding_element(
            &self,
            dot_dot_dot_token: Option<Arc<Node>>,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
            initializer: Option<Arc<Node>>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::BindingElement,
                NodeData::BindingElement(tsox_frontend::ast::node_data_generated::BindingElementData {
                    dot_dot_dot_token,
                    property_name,
                    name: Some(name),
                    initializer,
                }),
            ))
        }

        fn new_binding_pattern(&self, kind: SyntaxKind, elements: NodeList) -> Arc<Node> {
            Arc::new(Node::new(
                kind,
                NodeData::BindingPattern(tsox_frontend::ast::node_data_generated::BindingPatternData {
                    elements: Arc::new(elements),
                }),
            ))
        }

        fn new_variable_statement(
            &self,
            modifiers: Option<Arc<ModifierList>>,
            declaration_list: Arc<Node>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::VariableStatement,
                NodeData::VariableStatement(tsox_frontend::ast::node_data_generated::VariableStatementData {
                    modifiers,
                    declaration_list,
                }),
            ))
        }

        fn new_variable_declaration_list(&self, declarations: NodeList, flags: NodeFlags) -> Arc<Node> {
            Arc::new(Node::with_loc_flags(
                SyntaxKind::VariableDeclarationList,
                NodeData::VariableDeclarationList(tsox_frontend::ast::node_data_generated::VariableDeclarationListData {
                    declarations: Arc::new(declarations),
                }),
                tsox_core::core::text::TextRange::undefined(),
                flags,
            ))
        }

        fn new_variable_declaration(
            &self,
            name: Arc<Node>,
            exclamation_token: Option<Arc<Node>>,
            type_node: Option<Arc<Node>>,
            initializer: Option<Arc<Node>>,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::VariableDeclaration,
                NodeData::VariableDeclaration(tsox_frontend::ast::node_data_generated::VariableDeclarationData {
                    name,
                    exclamation_token,
                    type_node,
                    initializer,
                }),
            ))
        }

        fn new_call_expression(
            &self,
            expression: Arc<Node>,
            question_dot_token: Option<Arc<Node>>,
            type_arguments: Option<Arc<NodeList>>,
            arguments: NodeList,
        ) -> Arc<Node> {
            Arc::new(Node::new(
                SyntaxKind::CallExpression,
                NodeData::CallExpression(tsox_frontend::ast::node_data_generated::CallExpressionData {
                    expression,
                    question_dot_token,
                    type_arguments,
                    arguments: Arc::new(arguments),
                }),
            ))
        }
    }

    fn node_data_import_declaration(
        modifiers: Option<Arc<ModifierList>>,
        import_clause: Option<Arc<Node>>,
        module_specifier: Arc<Node>,
        attributes: Option<Arc<Node>>,
    ) -> tsox_frontend::ast::node_data_generated::ImportDeclarationData {
        tsox_frontend::ast::node_data_generated::ImportDeclarationData {
            modifiers,
            import_clause,
            module_specifier,
            attributes,
        }
    }

    fn import_clause_data(
        is_type_only: SyntaxKind,
        name: Option<Arc<Node>>,
        named_bindings: Option<Arc<Node>>,
    ) -> tsox_frontend::ast::node_data_generated::ImportClauseData {
        tsox_frontend::ast::node_data_generated::ImportClauseData {
            phase_modifier: if is_type_only == SyntaxKind::TypeKeyword {
                Some(SyntaxKind::TypeKeyword)
            } else {
                None
            },
            name,
            named_bindings,
        }
    }
}

use m5o2_ext::M5o2NodeExt;
use m5o2_ext::M5o2NodeFactoryExt;
use m5o2_ext::M5o2TrackerExt;
