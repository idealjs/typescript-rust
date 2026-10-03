#![allow(dead_code, unused_imports, unused_variables)]

pub mod m5v3_ext {
    use std::sync::Arc;
    use tsox_frontend::ast::node_data_generated::{
        ExportDeclarationData, ImportAttributeData, ImportAttributesData, ImportClauseData,
        ImportDeclarationData, ImportSpecifierData, NamedExportsData, NamedImportsData, NodeData,
    };
    use tsox_frontend::ast::node::Node;
    use tsox_frontend::ast::node::{ModifierList, NodeList};
    use tsox_frontend::ast::syntax_kind_generated::SyntaxKind;
    use tsox_frontend::ast::mig::m3b_2::NodeFactory;
    use tsox_frontend::format::mig::m4o::{EmitContext, EmitFlags};
    use std::cell::RefCell;

    pub trait M5v3NodeExt {
        fn import_clause(&self) -> Option<&Arc<Node>>;
        fn module_specifier(&self) -> Option<Arc<Node>>;
        fn attributes(&self) -> Option<Arc<Node>>;
        fn named_bindings(&self) -> Option<&Arc<Node>>;
        fn phase_modifier(&self) -> Option<SyntaxKind>;
        fn is_type_only(&self) -> bool;
        fn as_import_clause(&self) -> &ImportClauseData;
        fn as_named_imports(&self) -> &NamedImportsData;
        fn as_named_exports(&self) -> &NamedExportsData;
        fn as_import_specifier(&self) -> &ImportSpecifierData;
        fn as_import_attributes(&self) -> &ImportAttributesData;
        fn as_import_attribute(&self) -> &ImportAttributeData;
        fn as_export_declaration(&self) -> &ExportDeclarationData;
    }

    macro_rules! m5v3_as_data {
        ($name:ident, $variant:ident, $ty:ty) => {
            fn $name(&self) -> &$ty {
                match &self.data {
                    NodeData::$variant(d) => d,
                    _ => panic!(concat!("As", stringify!($variant), " on wrong node kind")),
                }
            }
        };
    }

    impl M5v3NodeExt for Node {
        fn import_clause(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("import_clause"); 
            match &self.data {
                NodeData::ImportDeclaration(d) => d.import_clause.as_ref(),
                _ => None,
            }
        }

        fn module_specifier(&self) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("module_specifier"); 
            match &self.data {
                NodeData::ImportDeclaration(d) => Some(Arc::clone(&d.module_specifier)),
                NodeData::ExportDeclaration(d) => d.module_specifier.clone(),
                _ => None,
            }
        }

        fn attributes(&self) -> Option<Arc<Node>> { ::tsox_core::fntrace::enter("attributes"); 
            match &self.data {
                NodeData::ImportDeclaration(d) => d.attributes.clone(),
                NodeData::ExportDeclaration(d) => d.attributes.clone(),
                _ => None,
            }
        }

        fn named_bindings(&self) -> Option<&Arc<Node>> { ::tsox_core::fntrace::enter("named_bindings"); 
            match &self.data {
                NodeData::ImportClause(d) => d.named_bindings.as_ref(),
                _ => None,
            }
        }

        fn phase_modifier(&self) -> Option<SyntaxKind> { ::tsox_core::fntrace::enter("phase_modifier"); 
            match &self.data {
                NodeData::ImportClause(d) => d.phase_modifier,
                _ => None,
            }
        }

        fn is_type_only(&self) -> bool { ::tsox_core::fntrace::enter("is_type_only"); 
            match &self.data {
                NodeData::ImportClause(d) => d.phase_modifier == Some(SyntaxKind::TypeKeyword),
                NodeData::ImportEqualsDeclaration(d) => d.is_type_only,
                NodeData::ImportSpecifier(d) => d.is_type_only,
                NodeData::ExportSpecifier(d) => d.is_type_only,
                NodeData::ExportDeclaration(d) => d.is_type_only,
                _ => false,
            }
        }

        m5v3_as_data!(as_import_clause, ImportClause, ImportClauseData);
        m5v3_as_data!(as_named_imports, NamedImports, NamedImportsData);
        m5v3_as_data!(as_named_exports, NamedExports, NamedExportsData);
        m5v3_as_data!(as_import_specifier, ImportSpecifier, ImportSpecifierData);
        m5v3_as_data!(as_import_attributes, ImportAttributes, ImportAttributesData);
        m5v3_as_data!(as_import_attribute, ImportAttribute, ImportAttributeData);
        m5v3_as_data!(as_export_declaration, ExportDeclaration, ExportDeclarationData);
    }

    pub trait M5v3FactoryExt {
        fn new_identifier(&self, text: &str) -> Arc<Node>;
        fn new_import_specifier(
            &self,
            is_type_only: bool,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
        ) -> Arc<Node>;
        fn new_named_imports(&self, elements: NodeList) -> Arc<Node>;
        fn new_import_clause(
            &self,
            phase_modifier: Option<SyntaxKind>,
            name: Option<Arc<Node>>,
            named_bindings: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn update_import_specifier(
            &self,
            node: &Arc<Node>,
            is_type_only: bool,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
        ) -> Arc<Node>;
        fn update_named_imports(&self, node: &Arc<Node>, elements: NodeList) -> Arc<Node>;
        fn update_import_clause(
            &self,
            node: &Arc<Node>,
            phase_modifier: Option<SyntaxKind>,
            name: Option<Arc<Node>>,
            named_bindings: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn update_import_declaration(
            &self,
            node: &Arc<Node>,
            modifiers: Option<Arc<ModifierList>>,
            import_clause: Option<Arc<Node>>,
            module_specifier: Arc<Node>,
            attributes: Option<Arc<Node>>,
        ) -> Arc<Node>;
        fn update_named_exports(&self, node: &Arc<Node>, elements: &NodeList) -> Arc<Node>;
        fn update_export_declaration(
            &self,
            node: &Arc<Node>,
            modifiers: Option<Arc<ModifierList>>,
            is_type_only: bool,
            export_clause: Option<Arc<Node>>,
            module_specifier: Option<Arc<Node>>,
            attributes: Option<Arc<Node>>,
        ) -> Arc<Node>;
    }

    fn clone_list(elements: &NodeList) -> Arc<NodeList> { ::tsox_core::fntrace::enter("clone_list"); 
        Arc::new(NodeList {
            loc: elements.loc,
            nodes: elements.nodes.clone(),
        })
    }

    fn updated_from(node: &Node, kind: SyntaxKind, data: NodeData) -> Arc<Node> { ::tsox_core::fntrace::enter("updated_from"); 
        let mut updated = Node::new(kind, data);
        updated.loc = node.loc;
        updated.flags = node.flags;
        Arc::new(updated)
    }

    impl M5v3FactoryExt for NodeFactory {
        fn new_identifier(&self, text: &str) -> Arc<Node> { ::tsox_core::fntrace::enter("new_identifier"); 
            self.new_identifier(text)
        }

        fn new_import_specifier(
            &self,
            is_type_only: bool,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
        ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_import_specifier"); 
            Arc::new(Node::new(
                SyntaxKind::ImportSpecifier,
                NodeData::ImportSpecifier(ImportSpecifierData {
                    is_type_only,
                    property_name,
                    name,
                }),
            ))
        }

        fn new_named_imports(&self, elements: NodeList) -> Arc<Node> { ::tsox_core::fntrace::enter("new_named_imports"); 
            Arc::new(Node::new(
                SyntaxKind::NamedImports,
                NodeData::NamedImports(NamedImportsData {
                    elements: Arc::new(elements),
                }),
            ))
        }

        fn new_import_clause(
            &self,
            phase_modifier: Option<SyntaxKind>,
            name: Option<Arc<Node>>,
            named_bindings: Option<Arc<Node>>,
        ) -> Arc<Node> { ::tsox_core::fntrace::enter("new_import_clause"); 
            Arc::new(Node::new(
                SyntaxKind::ImportClause,
                NodeData::ImportClause(ImportClauseData {
                    phase_modifier,
                    name,
                    named_bindings,
                }),
            ))
        }

        fn update_import_specifier(
            &self,
            node: &Arc<Node>,
            is_type_only: bool,
            property_name: Option<Arc<Node>>,
            name: Arc<Node>,
        ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_import_specifier"); 
            updated_from(
                node,
                SyntaxKind::ImportSpecifier,
                NodeData::ImportSpecifier(ImportSpecifierData {
                    is_type_only,
                    property_name,
                    name,
                }),
            )
        }

        fn update_named_imports(&self, node: &Arc<Node>, elements: NodeList) -> Arc<Node> { ::tsox_core::fntrace::enter("update_named_imports"); 
            updated_from(
                node,
                SyntaxKind::NamedImports,
                NodeData::NamedImports(NamedImportsData {
                    elements: clone_list(&elements),
                }),
            )
        }

        fn update_import_clause(
            &self,
            node: &Arc<Node>,
            phase_modifier: Option<SyntaxKind>,
            name: Option<Arc<Node>>,
            named_bindings: Option<Arc<Node>>,
        ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_import_clause"); 
            updated_from(
                node,
                SyntaxKind::ImportClause,
                NodeData::ImportClause(ImportClauseData {
                    phase_modifier,
                    name,
                    named_bindings,
                }),
            )
        }

        fn update_import_declaration(
            &self,
            node: &Arc<Node>,
            modifiers: Option<Arc<ModifierList>>,
            import_clause: Option<Arc<Node>>,
            module_specifier: Arc<Node>,
            attributes: Option<Arc<Node>>,
        ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_import_declaration"); 
            updated_from(
                node,
                SyntaxKind::ImportDeclaration,
                NodeData::ImportDeclaration(ImportDeclarationData {
                    modifiers,
                    import_clause,
                    module_specifier,
                    attributes,
                }),
            )
        }

        fn update_named_exports(&self, node: &Arc<Node>, elements: &NodeList) -> Arc<Node> { ::tsox_core::fntrace::enter("update_named_exports"); 
            updated_from(
                node,
                SyntaxKind::NamedExports,
                NodeData::NamedExports(tsox_frontend::ast::node_data_generated::NamedExportsData {
                    elements: clone_list(elements),
                }),
            )
        }

        fn update_export_declaration(
            &self,
            node: &Arc<Node>,
            modifiers: Option<Arc<ModifierList>>,
            is_type_only: bool,
            export_clause: Option<Arc<Node>>,
            module_specifier: Option<Arc<Node>>,
            attributes: Option<Arc<Node>>,
        ) -> Arc<Node> { ::tsox_core::fntrace::enter("update_export_declaration"); 
            updated_from(
                node,
                SyntaxKind::ExportDeclaration,
                NodeData::ExportDeclaration(ExportDeclarationData {
                    modifiers,
                    is_type_only,
                    export_clause,
                    module_specifier,
                    attributes,
                }),
            )
        }
    }

    thread_local! {
        static M5V3_EMIT_CONTEXT: RefCell<EmitContext> = RefCell::new(EmitContext::default());
    }

    pub trait M5v3TrackerExt {
        fn set_emit_flags(&mut self, node: &Arc<Node>, flags: EmitFlags);
        fn add_emit_flags(&mut self, node: &Arc<Node>, flags: EmitFlags);
    }

    impl M5v3TrackerExt for crate::ls::change_tracker::Tracker {
        fn set_emit_flags(&mut self, node: &Arc<Node>, flags: EmitFlags) { ::tsox_core::fntrace::enter("set_emit_flags"); 
            M5V3_EMIT_CONTEXT.with(|ctx| ctx.borrow().set_emit_flags(node, flags));
        }

        fn add_emit_flags(&mut self, node: &Arc<Node>, flags: EmitFlags) { ::tsox_core::fntrace::enter("add_emit_flags"); 
            M5V3_EMIT_CONTEXT.with(|ctx| ctx.borrow().add_emit_flags(node, flags));
        }
    }
}

use m5v3_ext::{M5v3FactoryExt, M5v3NodeExt, M5v3TrackerExt};

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile, Symbol, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;
use tsox_frontend::scanner;

use crate::ls::change_tracker::Tracker as ChangeTracker;
use crate::ls::lsutil::OrganizeImportsTypeOrder;
use crate::ls::lsutil_organize_imports_comparers::{StatementComparer, StringComparer};

use crate::ls::lsutil::UserPreferences as M5vUserPreferences;

pub struct OrganizeImportsComparerSettings {
    pub module_specifier_comparer: StringComparer,
    pub named_import_comparer: StringComparer,
    pub type_order: OrganizeImportsTypeOrder,
}

pub fn organize_imports_worker(
    old_import_decls: &[Arc<Node>],
    comparer: &OrganizeImportsComparerSettings,
    should_sort: bool,
    should_combine: bool,
    should_remove: bool,
    source_file: &Arc<SourceFile>,
    program: &tsox_compile::compiler::Program,
    change_tracker: &mut ChangeTracker,
    ctx: &crate::mig::m5m::ResolvedClientCapabilitiesContext,
) { ::tsox_core::fntrace::enter("organize_imports_worker"); 
    if old_import_decls.is_empty() {
        return;
    }

    let mut processed_imports: Vec<Arc<Node>> = old_import_decls.to_vec();
    if should_remove {
        let mut type_checker = program.get_type_checker_for_file(source_file);
        processed_imports = remove_unused_imports(
            &processed_imports,
            source_file,
            &mut type_checker,
            program,
            change_tracker,
        );
    }

    let mut new_import_decls: Vec<Arc<Node>> = Vec::new();
    if should_combine {
        let mut grouped = group_by_module_specifier(&processed_imports);
        if should_sort {
            grouped.sort_by(|a, b| {
                if a.is_empty() || b.is_empty() {
                    return std::cmp::Ordering::Equal;
                }
                let ord = (comparer.module_specifier_comparer)(
                    &crate::ls::lsutil_organize_imports_imports::get_external_module_name(
                        tsox_frontend::ast::mig::m3b::module_specifier(&a[0]),
                    ),
                    &crate::ls::lsutil_organize_imports_imports::get_external_module_name(
                        tsox_frontend::ast::mig::m3b::module_specifier(&b[0]),
                    ),
                );
                ord.cmp(&0)
            });
        }

        let specifier_comparer = crate::ls::lsutil_organize_imports_comparers::get_named_import_specifier_comparer(
            &M5vUserPreferences {
                organize_imports_type_order: comparer.type_order,
                ..new_user_preferences_for_worker()
            },
            Some(comparer.named_import_comparer.clone()),
        );

        for import_group in grouped {
            let mut coalesced = coalesce_imports_worker(
                &import_group,
                &comparer.module_specifier_comparer,
                &specifier_comparer,
                Some(source_file),
                change_tracker,
            );
            if should_sort {
                coalesced.sort_by(|a, b| {
                    let ord = crate::ls::lsutil_organize_imports_imports::compare_imports_or_require_statements(
                        a,
                        b,
                        &comparer.module_specifier_comparer,
                    );
                    ord.cmp(&0)
                });
            }
            new_import_decls.extend(coalesced);
        }
    } else {
        new_import_decls = processed_imports;
    }

    if should_sort && !should_combine {
        new_import_decls.sort_by(|a, b| {
            let ord = crate::ls::lsutil_organize_imports_imports::compare_imports_or_require_statements(
                a,
                b,
                &comparer.module_specifier_comparer,
            );
            ord.cmp(&0)
        });
    }

    if new_import_decls.is_empty() {
        change_tracker.delete_node_range(
            source_file,
            &old_import_decls[0],
            &old_import_decls[old_import_decls.len() - 1],
            crate::ls::change_tracker::LeadingTriviaOption::Exclude,
            crate::ls::change_tracker::TrailingTriviaOption::Include,
        );
    } else {
        for imp in &new_import_decls {
            change_tracker.set_emit_flags(
                imp,
                tsox_frontend::format::mig::m4o::EmitFlags::NO_LEADING_COMMENTS,
            );
        }

        let options = crate::ls::change_tracker::NodeOptions {
            leading_trivia_option: crate::ls::change_tracker::LeadingTriviaOption::Exclude,
            trailing_trivia_option: crate::ls::change_tracker::TrailingTriviaOption::Include,
            suffix: "\n".to_string(),
            ..Default::default()
        };

        let new_nodes: Vec<Arc<Node>> = new_import_decls.clone();
        change_tracker.replace_node_with_nodes(
            source_file,
            &old_import_decls[0],
            &new_nodes,
            Some(&options),
        );

        if old_import_decls.len() > 1 {
            for old in &old_import_decls[1..] {
                change_tracker.delete(source_file, old);
            }
        }
    }
}

fn new_user_preferences_for_worker() -> M5vUserPreferences { ::tsox_core::fntrace::enter("new_user_preferences_for_worker"); 
    crate::ls::lsutil::new_default_user_preferences()
}

pub fn group_by_module_specifier(imports: &[Arc<Node>]) -> Vec<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("group_by_module_specifier"); 
    let mut groups: HashMap<String, Vec<Arc<Node>>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for imp in imports {
        let specifier = crate::ls::lsutil_organize_imports_imports::get_external_module_name(
            tsox_frontend::ast::mig::m3b::module_specifier(imp),
        );
        if !groups.contains_key(&specifier) {
            order.push(specifier.clone());
        }
        groups.entry(specifier).or_default().push(Arc::clone(imp));
    }

    order
        .iter()
        .map(|key| groups.remove(key).unwrap_or_default())
        .collect()
}

pub fn remove_unused_imports(
    old_imports: &[Arc<Node>],
    source_file: &Arc<SourceFile>,
    type_checker: &mut tsox_checker::checker::Checker,
    program: &tsox_compile::compiler::Program,
    change_tracker: &mut ChangeTracker,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("remove_unused_imports"); 
    let compiler_options = program.options();
    let jsx_elements_present = source_file
        .node
        .subtree_facts()
        .contains(tsox_frontend::ast::subtree_facts::SubtreeContainsJsx);
    let jsx_mode_needs_explicit_import = compiler_options.jsx
        == tsox_core::core::compiler_options_kinds::JsxEmit::React
        || compiler_options.jsx == tsox_core::core::compiler_options_kinds::JsxEmit::ReactNative;

    let factory = NodeFactory::new();
    let mut used_imports: Vec<Arc<Node>> = Vec::with_capacity(old_imports.len());

    for import_decl in old_imports {
        let import_clause = import_decl.import_clause();
        let Some(import_clause) = import_clause else {
            used_imports.push(Arc::clone(import_decl));
            continue;
        };

        let mut name = import_clause.name().cloned();
        let mut named_bindings = import_clause.named_bindings().cloned();

        if let Some(name_node) = name.clone() {
            let used = type_checker.is_declaration_used(
                source_file,
                &name_node,
                jsx_elements_present,
                jsx_mode_needs_explicit_import,
            );
            if !used {
                name = None;
            }
        }

        if let Some(bindings) = named_bindings.clone() {
            match bindings.kind {
                SyntaxKind::NamespaceImport => {
                    let ns_name = bindings.name().expect("namespace import has name");
                    let used = type_checker.is_declaration_used(
                        source_file,
                        &ns_name,
                        jsx_elements_present,
                        jsx_mode_needs_explicit_import,
                    );
                    if !used {
                        named_bindings = None;
                    }
                }
                SyntaxKind::NamedImports => {
                    let original_bindings = bindings.clone();
                    let elements: &[Arc<Node>] = &bindings.as_named_imports().elements.nodes;
                    let new_elements = filter_used_import_specifiers(
                        elements,
                        type_checker,
                        source_file,
                        jsx_elements_present,
                        jsx_mode_needs_explicit_import,
                    );
                    if new_elements.is_empty() {
                        named_bindings = None;
                    } else if new_elements.len() < elements.len() {
                        let new_list = factory.new_node_list(new_elements);
                        let updated = factory.update_named_imports(&bindings, new_list);
                        named_bindings = Some(updated);
                    }
                    if let Some(updated) = named_bindings.clone() {
                        if !ast::node_is_synthesized(&original_bindings)
                            && !tsox_frontend::format::mig::m4t_3::range_is_on_single_line(
                                original_bindings.loc,
                                source_file,
                            )
                        {
                            change_tracker.set_emit_flags(
                                &updated,
                                tsox_frontend::format::mig::m4o::EmitFlags::MULTI_LINE,
                            );
                        }
                    }
                }
                _ => {}
            }
        }

        if name.is_some() || named_bindings.is_some() {
            let new_clause =
                factory.update_import_clause(import_clause, import_clause.phase_modifier(), name, named_bindings);
            let new_import_decl = factory.update_import_declaration(
                import_decl,
                import_decl.modifiers().cloned(),
                Some(new_clause),
                import_decl
                    .module_specifier()
                    .expect("import declaration has module specifier"),
                import_decl.attributes(),
            );
            used_imports.push(new_import_decl);
        } else {
            let module_specifier = import_decl.module_specifier();
            if has_module_declaration_matching_specifier(source_file, module_specifier.as_ref()) {
                if source_file.is_declaration_file {
                    let new_import_decl = factory.update_import_declaration(
                        import_decl,
                        import_decl.modifiers().cloned(),
                        None,
                        import_decl
                            .module_specifier()
                            .expect("import declaration has module specifier"),
                        import_decl.attributes(),
                    );
                    used_imports.push(new_import_decl);
                } else {
                    used_imports.push(Arc::clone(import_decl));
                }
            }
        }
    }

    used_imports
}

pub fn filter_used_import_specifiers(
    elements: &[Arc<Node>],
    type_checker: &mut tsox_checker::checker::Checker,
    source_file: &Arc<SourceFile>,
    jsx_elements_present: bool,
    jsx_mode_needs_explicit_import: bool,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("filter_used_import_specifiers"); 
    let mut result = Vec::new();
    for elem in elements {
        let spec = elem.as_import_specifier();
        let used = type_checker.is_declaration_used(
            source_file,
            &spec.name,
            jsx_elements_present,
            jsx_mode_needs_explicit_import,
        );
        if used {
            result.push(Arc::clone(elem));
        }
    }
    result
}

pub fn has_module_declaration_matching_specifier(
    source_file: &Arc<SourceFile>,
    module_specifier: Option<&Arc<Node>>,
) -> bool { ::tsox_core::fntrace::enter("has_module_declaration_matching_specifier"); 
    let Some(module_specifier) = module_specifier else {
        return false;
    };
    if !ast::is_string_literal(module_specifier) {
        return false;
    }
    let module_specifier_text = module_specifier.text();

    for module_name in &source_file.module_augmentations {
        if ast::is_string_literal(module_name) && module_name.text() == module_specifier_text {
            return true;
        }
    }

    false
}

pub fn get_import_attributes_key(attributes: Option<&Arc<Node>>) -> String { ::tsox_core::fntrace::enter("get_import_attributes_key"); 
    let Some(attributes) = attributes else {
        return String::new();
    };

    let import_attrs = attributes.as_import_attributes();
    let mut key = String::new();
    key.push_str(&format!("{:?}", import_attrs.token));
    key.push(' ');

    let mut attr_nodes: Vec<Arc<Node>> = import_attrs.attributes.nodes.clone();
    attr_nodes.sort_by(|a, b| {
        let a_name = a.as_import_attribute().name.text();
        let b_name = b.as_import_attribute().name.text();
        tsox_core::stringutil::compare_strings_case_sensitive(&a_name, &b_name).cmp(&0)
    });

    for attr_node in &attr_nodes {
        let attr = attr_node.as_import_attribute();
        key.push_str(&attr.name.text());
        key.push(':');
        if ast::is_string_literal_like(&attr.value) {
            key.push('"');
            key.push_str(&attr.value.text());
            key.push('"');
        } else {
            key.push_str(&attr.value.text());
        }
        key.push(' ');
    }

    key
}

pub fn group_by_newline_contiguous(
    source_file: &Arc<SourceFile>,
    decls: &[Arc<Node>],
) -> Vec<Vec<Arc<Node>>> { ::tsox_core::fntrace::enter("group_by_newline_contiguous"); 
    let mut s = scanner::Scanner::new(String::new());
    s.set_skip_trivia(false);
    let mut groups: Vec<Vec<Arc<Node>>> = Vec::new();
    let mut current_group: Vec<Arc<Node>> = Vec::new();

    for decl in decls {
        if !current_group.is_empty() && is_new_group(source_file, decl, &mut s) {
            groups.push(std::mem::take(&mut current_group));
        }
        current_group.push(Arc::clone(decl));
    }

    if !current_group.is_empty() {
        groups.push(current_group);
    }

    groups
}

pub fn is_new_group(source_file: &Arc<SourceFile>, decl: &Arc<Node>, s: &mut scanner::Scanner) -> bool { ::tsox_core::fntrace::enter("is_new_group"); 
    let full_start = decl.pos();
    if full_start < 0 {
        return false;
    }

    let text = source_file.text.as_str();
    let text_len = text.len();
    if full_start as usize >= text_len {
        return false;
    }

    let start_pos = scanner::skip_trivia(text, full_start as usize);
    if start_pos <= full_start as usize {
        return false;
    }

    let trivia_len = start_pos - full_start as usize;
    s.set_text(&text[full_start as usize..start_pos]);

    let mut number_of_new_lines = 0;
    while s.token_pos() < trivia_len {
        let token_kind = s.scan();
        if token_kind == SyntaxKind::NewLineTrivia {
            number_of_new_lines += 1;
            if number_of_new_lines >= 2 {
                return true;
            }
        }
    }

    false
}

pub struct ImportGroup {
    pub default_imports: Vec<Arc<Node>>,
    pub namespace_imports: Vec<Arc<Node>>,
    pub named_imports: Vec<Arc<Node>>,
}

impl ImportGroup {
    pub fn is_empty(&self) -> bool { ::tsox_core::fntrace::enter("is_empty"); 
        self.default_imports.is_empty()
            && self.namespace_imports.is_empty()
            && self.named_imports.is_empty()
    }
}

impl Default for ImportGroup {
    fn default() -> Self { ::tsox_core::fntrace::enter("default"); 
        ImportGroup {
            default_imports: Vec::new(),
            namespace_imports: Vec::new(),
            named_imports: Vec::new(),
        }
    }
}

pub struct CategorizedImports {
    pub import_without_clause: Option<Arc<Node>>,
    pub type_only_imports: ImportGroup,
    pub regular_imports: ImportGroup,
}

pub fn coalesce_imports_worker(
    import_decls: &[Arc<Node>],
    comparer: &StringComparer,
    specifier_comparer: &StatementComparer,
    source_file: Option<&Arc<SourceFile>>,
    change_tracker: &mut ChangeTracker,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("coalesce_imports_worker"); 
    if import_decls.is_empty() {
        return import_decls.to_vec();
    }

    let mut import_groups_by_attributes: HashMap<String, Vec<Arc<Node>>> = HashMap::new();
    let mut attribute_keys: Vec<String> = Vec::new();

    for import_decl in import_decls {
        let key = get_import_attributes_key(import_decl.attributes().as_ref());
        if !import_groups_by_attributes.contains_key(&key) {
            attribute_keys.push(key.clone());
        }
        import_groups_by_attributes.entry(key).or_default().push(Arc::clone(import_decl));
    }

    let mut coalesced_imports: Vec<Arc<Node>> = Vec::new();
    let factory = NodeFactory::new();

    for attribute_key in &attribute_keys {
        let import_group_same_attrs = &import_groups_by_attributes[attribute_key];
        let categorized = get_categorized_imports(import_group_same_attrs);

        if let Some(without_clause) = categorized.import_without_clause {
            coalesced_imports.push(without_clause);
        }

        for (i, group) in [&categorized.regular_imports, &categorized.type_only_imports]
            .into_iter()
            .enumerate()
        {
            if group.is_empty() {
                continue;
            }

            let is_type_only = i == 1;

            if !is_type_only
                && group.default_imports.len() == 1
                && group.namespace_imports.len() == 1
                && group.named_imports.is_empty()
            {
                let default_import = &group.default_imports[0];
                let namespace_import = &group.namespace_imports[0];

                let default_clause = default_import.import_clause().unwrap();
                let namespace_bindings = namespace_import
                    .import_clause()
                    .and_then(|c| c.named_bindings().cloned());

                let new_clause = factory.update_import_clause(
                    default_clause,
                    default_clause.phase_modifier(),
                    default_clause.name().cloned(),
                    namespace_bindings,
                );
                let new_import_decl = factory.update_import_declaration(
                    default_import,
                    default_import.modifiers().cloned(),
                    Some(new_clause),
                    default_import
                        .module_specifier()
                        .expect("import declaration has module specifier"),
                    default_import.attributes(),
                );
                coalesced_imports.push(new_import_decl);
                continue;
            }

            let mut namespace_imports = group.namespace_imports.clone();
            namespace_imports.sort_by(|a, b| {
                let n1 = a.import_clause().and_then(|c| c.named_bindings().cloned()).and_then(|b| b.name().cloned());
                let n2 = b.import_clause().and_then(|c| c.named_bindings().cloned()).and_then(|b| b.name().cloned());
                let t1 = n1.map(|n| n.text().to_string()).unwrap_or_default();
                let t2 = n2.map(|n| n.text().to_string()).unwrap_or_default();
                let ord = comparer(&t1, &t2);
                ord.cmp(&0)
            });

            for ns_import in &namespace_imports {
                let clause = ns_import.import_clause().unwrap();
                let new_clause = factory.update_import_clause(
                    clause,
                    clause.phase_modifier(),
                    None,
                    clause.named_bindings().cloned(),
                );
                let new_import_decl = factory.update_import_declaration(
                    ns_import,
                    ns_import.modifiers().cloned(),
                    Some(new_clause),
                    ns_import
                        .module_specifier()
                        .expect("import declaration has module specifier"),
                    ns_import.attributes(),
                );
                coalesced_imports.push(new_import_decl);
            }

            let first_default_import = group.default_imports.first();
            let first_named_import = group.named_imports.first();

            let import_decl = first_default_import.or(first_named_import);
            let Some(import_decl) = import_decl else {
                continue;
            };

            let mut new_default_import: Option<Arc<Node>> = None;
            let mut new_import_specifiers: Vec<Arc<Node>> = Vec::new();

            if group.default_imports.len() == 1 {
                new_default_import = group.default_imports[0]
                    .import_clause()
                    .and_then(|c| c.name().cloned());
            } else {
                for default_import in &group.default_imports {
                    let default_clause = default_import.import_clause().unwrap();
                    let default_name = default_clause
                        .name()
                        .cloned()
                        .expect("default import clause has name");
                    let property_name = factory.new_identifier("default");
                    let import_spec =
                        factory.new_import_specifier(false, Some(property_name), default_name);
                    new_import_specifiers.push(import_spec);
                }
            }

            new_import_specifiers.extend(get_new_import_specifiers(
                &group.named_imports,
                &factory,
            ));
            new_import_specifiers.sort_by(|a, b| specifier_comparer(a, b).cmp(&0));

            let mut new_named_imports: Option<Arc<Node>> = None;
            if new_import_specifiers.is_empty() {
                if new_default_import.is_some() {
                    new_named_imports = None;
                } else {
                    new_named_imports = Some(factory.new_named_imports(factory.new_node_list(Vec::new())));
                }
            } else {
                let sorted_list = factory.new_node_list(new_import_specifiers);
                if let Some(first_named) = first_named_import {
                    let first_named_bindings_node = first_named
                        .import_clause()
                        .and_then(|c| c.named_bindings())
                        .cloned()
                        .unwrap();
                    let first_named_bindings = first_named_bindings_node.as_named_imports();
                    let original_elements = first_named_bindings.elements.clone();
                    let mut sorted_list = sorted_list;
                    if original_elements.has_trailing_comma() {
                        sorted_list.loc = original_elements.loc;
                    }
                    new_named_imports =
                        Some(factory.update_named_imports(&first_named_bindings_node, sorted_list));
                } else {
                    new_named_imports = Some(factory.new_named_imports(sorted_list));
                }
            }

            if let (Some(sf), Some(new_named)) = (source_file, new_named_imports.as_ref()) {
                if let Some(first_named) = first_named_import {
                    let first_named_bindings = first_named
                        .import_clause()
                        .and_then(|c| c.named_bindings())
                        .cloned()
                        .unwrap();
                    if !ast::node_is_synthesized(&first_named_bindings)
                        && !tsox_frontend::format::mig::m4t_3::range_is_on_single_line(
                            first_named_bindings.loc,
                            sf,
                        )
                    {
                        change_tracker.set_emit_flags(
                            new_named,
                            tsox_frontend::format::mig::m4o::EmitFlags::MULTI_LINE,
                        );
                    }
                }
            }

            if is_type_only && new_default_import.is_some() && new_named_imports.is_some() {
                let default_clause = factory.new_import_clause(
                    import_decl
                        .import_clause()
                        .and_then(|c| c.phase_modifier()),
                    new_default_import.clone(),
                    None,
                );
                let default_import_decl = factory.update_import_declaration(
                    import_decl,
                    import_decl.modifiers().cloned(),
                    Some(default_clause),
                    import_decl
                        .module_specifier()
                        .expect("import declaration has module specifier"),
                    import_decl.attributes(),
                );
                coalesced_imports.push(default_import_decl);

                let named_decl_node = first_named_import.unwrap_or(import_decl);
                let named_clause = factory.new_import_clause(
                    named_decl_node
                        .import_clause()
                        .and_then(|c| c.phase_modifier()),
                    None,
                    new_named_imports,
                );
                let named_import_decl = factory.update_import_declaration(
                    named_decl_node,
                    named_decl_node.modifiers().cloned(),
                    Some(named_clause),
                    named_decl_node
                        .module_specifier()
                        .expect("import declaration has module specifier"),
                    named_decl_node.attributes(),
                );
                coalesced_imports.push(named_import_decl);
            } else {
                let clause_node = import_decl.import_clause().unwrap();
                let new_clause = factory.update_import_clause(
                    clause_node,
                    clause_node.phase_modifier(),
                    new_default_import,
                    new_named_imports,
                );
                let new_import_decl = factory.update_import_declaration(
                    import_decl,
                    import_decl.modifiers().cloned(),
                    Some(new_clause),
                    import_decl
                        .module_specifier()
                        .expect("import declaration has module specifier"),
                    import_decl.attributes(),
                );
                coalesced_imports.push(new_import_decl);
            }
        }
    }
    coalesced_imports
}

pub fn get_categorized_imports(import_decls: &[Arc<Node>]) -> CategorizedImports { ::tsox_core::fntrace::enter("get_categorized_imports"); 
    let mut import_without_clause: Option<Arc<Node>> = None;
    let mut type_only_imports = ImportGroup::default();
    let mut regular_imports = ImportGroup::default();

    for import_decl in import_decls {
        let Some(clause) = import_decl.import_clause() else {
            if import_without_clause.is_none() {
                import_without_clause = Some(Arc::clone(import_decl));
            }
            continue;
        };

        let is_type_only = clause.is_type_only();
        let group: &mut ImportGroup = if is_type_only {
            &mut type_only_imports
        } else {
            &mut regular_imports
        };

        let name = clause.name();
        let named_bindings = clause.named_bindings();

        if name.is_some() {
            group.default_imports.push(Arc::clone(import_decl));
        }

        if let Some(bindings) = named_bindings {
            match bindings.kind {
                SyntaxKind::NamespaceImport => {
                    group.namespace_imports.push(Arc::clone(import_decl));
                }
                SyntaxKind::NamedImports => {
                    group.named_imports.push(Arc::clone(import_decl));
                }
                _ => {}
            }
        }
    }

    CategorizedImports {
        import_without_clause,
        type_only_imports,
        regular_imports,
    }
}

pub fn get_new_import_specifiers(
    named_imports: &[Arc<Node>],
    factory: &NodeFactory,
) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("get_new_import_specifiers"); 
    let mut result: Vec<Arc<Node>> = Vec::new();

    for named_import in named_imports {
        let elements = try_get_named_binding_elements(named_import);
        if elements.is_empty() {
            continue;
        }

        for elem in elements {
            let spec = elem.as_import_specifier();

            if let (Some(property_name), Some(name)) =
                (spec.property_name.as_ref(), Some(&spec.name))
            {
                let property_text = property_name.text();
                let name_text = name.text();

                if property_text == name_text {
                    let normalized = factory.update_import_specifier(
                        &elem,
                        spec.is_type_only,
                        None,
                        Arc::clone(&spec.name),
                    );
                    result.push(normalized);
                    continue;
                }
            }

            result.push(elem);
        }
    }

    result
}

pub fn try_get_named_binding_elements(named_import: &Arc<Node>) -> Vec<Arc<Node>> { ::tsox_core::fntrace::enter("try_get_named_binding_elements"); 
    if named_import.kind != SyntaxKind::ImportDeclaration {
        return Vec::new();
    }

    let Some(clause) = named_import.import_clause() else {
        return Vec::new();
    };

    if let Some(named_bindings) = clause.named_bindings() {
        if named_bindings.kind == SyntaxKind::NamedImports {
            return named_bindings.as_named_imports().elements.nodes.clone();
        }
    }

    Vec::new()
}
