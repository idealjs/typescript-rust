#![allow(dead_code, unused_imports, unused_variables)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use tsox_frontend::ast::{self, Node, NodeList, SourceFile};
use tsox_frontend::ast::mig::m3d::{NodeVisitor, NodeVisitorHooks};

use crate::ls::autoimport::AddAsTypeOnly;
use crate::ls::autoimport::ImportKind;
use crate::ls::autoimport_export::Export;
use crate::ls::autoimport_fix::Fix;
use crate::ls::autoimport_import_adder::ImportAdder;
use crate::ls::autoimport_import_adder::ImportAdderTrait;
use crate::ls::autoimport_import_adder::ImportsCollection;
use crate::ls::autoimport_import_adder::new_imports_key;
use crate::ls::autoimport_import_adder::reduce_add_as_type_only_values;
use crate::ls::autoimport_view::View;
use crate::ls::lsutil_format_code_options::FormatCodeSettings;
use crate::ls::lsutil_user_preferences::UserPreferences;
use tsox_checker::checker::Checker;
use tsox_checker::checker::mig::m2b::r22k6_defs::R22K6NodeFactoryExt;
use tsox_checker::checker::mig::m2g::r21k9_defs::NodeFactoryExt21;
use tsox_checker::checker::nodecopy::NodeFactoryStub;
use tsox_compile::compiler::Program;
use tsox_frontend::ast::Symbol;
use tsox_frontend::ast::mig::m3e_4::get_first_identifier;

use crate::ls::mig::m5y_pc_2::PseudoNodeExt;

pub fn new_import_adder(
    program: &Program,
    checker: Arc<Checker>,
    file: &SourceFile,
    view: Arc<View>,
    format_options: FormatCodeSettings,
    converters: (),
    preferences: UserPreferences,
) -> ImportAdder {
    ImportAdder::new(program, checker, file, view, format_options, converters, preferences)
}

impl ImportAdder {
    pub fn get_all_exports_for_symbol(&self, symbol: &Arc<Symbol>) -> Vec<Export> {
        let ch = self.checker.as_ref().expect("import adder requires checker");
        let merged = ch.get_merged_symbol(&ch.skip_alias(symbol));
        if let Some(export) = crate::ls::autoimport_export::symbol_to_export(merged.as_ref(), ch) {
            return self
                .view
                .as_ref()
                .expect("import adder requires view")
                .search_by_export_id(&export.export_id);
        }
        Vec::new()
    }

    pub fn get_import_fix_for_symbol(
        &self,
        view: &View,
        file: &SourceFile,
        exports: &[Export],
        is_valid_type_only_use_site: bool,
    ) -> Option<Fix> {
        let mut fixes: Vec<Fix> = exports
            .iter()
            .flat_map(|export| view.get_fixes(export, false, is_valid_type_only_use_site, None))
            .collect();
        fixes.sort_by(|a, b| view.compare_fixes_for_ranking(a, b));
        fixes.into_iter().next()
    }

    pub fn get_new_import_entry(
        &mut self,
        module_specifier: &str,
        import_kind: ImportKind,
        use_require: bool,
        add_as_type_only: AddAsTypeOnly,
    ) -> &mut ImportsCollection {
        let type_only_key = new_imports_key(module_specifier, true);
        let non_type_only_key = new_imports_key(module_specifier, false);
        let new_entry = ImportsCollection {
            use_require,
            ..Default::default()
        };

        if import_kind == ImportKind::Default && add_as_type_only == AddAsTypeOnly::Required {
            if !self.new_imports.contains_key(&type_only_key) {
                self.new_imports.insert(type_only_key.clone(), new_entry);
            }
            return self.new_imports.get_mut(&type_only_key).unwrap();
        }

        if add_as_type_only == AddAsTypeOnly::Allowed
            && (self.new_imports.contains_key(&type_only_key) || self.new_imports.contains_key(&non_type_only_key))
        {
            let key = if self.new_imports.contains_key(&type_only_key) {
                type_only_key
            } else {
                non_type_only_key
            };
            return self.new_imports.get_mut(&key).unwrap();
        }

        if !self.new_imports.contains_key(&non_type_only_key) {
            self.new_imports.insert(non_type_only_key.clone(), new_entry);
        }
        self.new_imports.get_mut(&non_type_only_key).unwrap()
    }
}

pub fn import_symbols(import_adder: &mut dyn ImportAdderTrait, symbols: &[Arc<Symbol>]) {
    for symbol in symbols {
        import_adder.add_import_from_exported_symbol(symbol, true);
    }
}

pub fn type_node_to_auto_importable_type_node(
    type_node: &Arc<Node>,
    import_adder: &mut dyn ImportAdderTrait,
    id_to_symbol: &HashMap<u64, Arc<Symbol>>,
) -> Arc<Node> {
    let (reference_type_node, importable_symbols) =
        try_get_auto_importable_reference_from_type_node(type_node, id_to_symbol);
    let mut result = type_node.clone();
    if let Some(reference_type_node) = reference_type_node {
        import_symbols(import_adder, &importable_symbols);
        result = reference_type_node;
    }
    result
}

pub fn try_get_auto_importable_reference_from_type_node(
    import_type_node: &Arc<Node>,
    id_to_symbol: &HashMap<u64, Arc<Symbol>>,
) -> (Option<Arc<Node>>, Vec<Arc<Symbol>>) {
    let factory = NodeFactoryStub;
    let mut symbols: Vec<Arc<Symbol>> = Vec::new();
    let type_node = visit_auto_import_node(import_type_node, &factory, id_to_symbol, &mut symbols);
    (Some(type_node), symbols)
}

fn is_literal_import_type_node(node: &Arc<Node>) -> bool {
    if node.kind != ast::SyntaxKind::ImportType {
        return false;
    }
    let tsox_frontend::ast::NodeData::ImportTypeNode(d) = &node.data else {
        return false;
    };
    d.argument.kind == ast::SyntaxKind::LiteralType
        && matches!(
            &d.argument.data,
            tsox_frontend::ast::NodeData::LiteralTypeNode(lt) if lt.literal.kind == ast::SyntaxKind::StringLiteral
        )
}

fn visit_auto_import_node(
    node: &Arc<Node>,
    _factory: &NodeFactoryStub,
    id_to_symbol: &HashMap<u64, Arc<Symbol>>,
    symbols: &mut Vec<Arc<Symbol>>,
) -> Arc<Node> {
    let state = Rc::new(VisitAutoImportState {
        id_to_symbol: Arc::new(id_to_symbol.clone()),
        symbols: RefCell::new(Vec::new()),
    });
    let result = visit_auto_import_with_state(node, Rc::clone(&state));
    symbols.extend(state.symbols.borrow().iter().cloned());
    result
}

struct VisitAutoImportState {
    id_to_symbol: Arc<HashMap<u64, Arc<Symbol>>>,
    symbols: RefCell<Vec<Arc<Symbol>>>,
}

fn visit_auto_import_with_state(node: &Arc<Node>, state: Rc<VisitAutoImportState>) -> Arc<Node> {
    if is_literal_import_type_node(node) {
        let tsox_frontend::ast::NodeData::ImportTypeNode(d) = &node.data else {
            unreachable!();
        };
        if let Some(qualifier_node) = d.qualifier.clone() {
            let first_identifier = get_first_identifier(&qualifier_node);
            if let Some(symbol) = state.id_to_symbol.get(&first_identifier.id()) {
                let name = crate::ls::autoimport_import_adder::get_name_for_exported_symbol(symbol, false);
                let qualifier = if name != first_identifier.text() {
                    replace_first_identifier_of_entity_name(
                        &NodeFactoryStub,
                        &qualifier_node,
                        &NodeFactoryStub.new_identifier(&name),
                    )
                } else {
                    qualifier_node.clone()
                };
                state.symbols.borrow_mut().push(Arc::clone(symbol));
                let type_arguments = d.type_arguments.as_ref().map(|ta| {
                    let mapped: Vec<Arc<Node>> = ta
                        .nodes
                        .iter()
                        .map(|n| visit_auto_import_with_state(n, Rc::clone(&state)))
                        .collect();
                    Arc::new(NodeList::new(mapped))
                });
                return NodeFactoryStub.new_type_reference_node(&qualifier, type_arguments);
            }
        }
    }
    let visitor_factory = tsox_frontend::ast::mig::m3d::NodeFactory {
        node_count: 0,
        hooks: tsox_frontend::ast::mig::m3d::NodeFactoryHooks::default(),
    };
    let visitor = NodeVisitor::new(
        move |child: &Arc<Node>| visit_auto_import_with_state(child, Rc::clone(&state)),
        &visitor_factory,
        NodeVisitorHooks::default(),
    );
    visitor.visit_each_child_node(node)
}

pub fn replace_first_identifier_of_entity_name(
    factory: &NodeFactoryStub,
    name: &Arc<Node>,
    new_identifier: &Arc<Node>,
) -> Arc<Node> {
    if name.kind == ast::SyntaxKind::Identifier {
        return new_identifier.clone();
    }
    let tsox_frontend::ast::NodeData::QualifiedName(q) = &name.data else {
        unreachable!();
    };
    factory.new_qualified_name(
        &replace_first_identifier_of_entity_name(factory, &q.left, new_identifier),
        &q.right,
    )
}

pub fn sorted_named_imports(
    named_imports: &HashMap<String, crate::ls::autoimport_fix::NewImportBinding>,
) -> Vec<crate::ls::autoimport_fix::NewImportBinding> {
    let mut keys: Vec<&String> = named_imports.keys().collect();
    keys.sort();
    keys.into_iter()
        .map(|k| named_imports[k].clone())
        .collect()
}
