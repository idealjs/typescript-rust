#![allow(dead_code, unused_imports, unused_variables)]

use std::collections::HashMap;
use std::sync::Arc;

use tsox_frontend::ast::{self, Node, SourceFile, SyntaxKind};
use tsox_frontend::ast::mig::m3b_2::NodeFactory;

use crate::ls::change_tracker::Tracker as ChangeTracker;
use crate::ls::mig::m5v_3::m5v3_ext::{M5v3FactoryExt, M5v3NodeExt, M5v3TrackerExt};
use crate::ls::mig::m5v_3::OrganizeImportsComparerSettings;
use crate::ls::lsutil::OrganizeImportsTypeOrder;
use crate::ls::lsutil::UserPreferences;
use crate::ls::lsutil_organize_imports_comparers::{StatementComparer, StringComparer};

fn new_default_user_preferences_with_type_order(
    type_order: OrganizeImportsTypeOrder,
) -> UserPreferences {
    UserPreferences {
        organize_imports_type_order: type_order,
        ..crate::ls::lsutil::new_default_user_preferences()
    }
}

pub fn get_top_level_export_groups(source_file: &Arc<SourceFile>) -> Vec<Vec<Arc<Node>>> {
    let mut top_level_export_groups: Vec<Vec<Arc<Node>>> = Vec::new();
    let statements = match &source_file.node.data {
        tsox_frontend::ast::NodeData::SourceFile(d) => d.statements.nodes.clone(),
        _ => Vec::new(),
    };
    let statements_len = statements.len();

    let mut i = 0;
    let mut group_index = 0;
    while i < statements_len {
        if statements[i].kind == SyntaxKind::ExportDeclaration {
            if group_index >= top_level_export_groups.len() {
                top_level_export_groups.push(Vec::new());
            }
            let export_decl = statements[i].as_export_declaration();
            if export_decl.module_specifier.is_some() {
                top_level_export_groups[group_index].push(Arc::clone(&statements[i]));
                i += 1;
            } else {
                while i < statements_len && statements[i].kind == SyntaxKind::ExportDeclaration {
                    top_level_export_groups[group_index].push(Arc::clone(&statements[i]));
                    i += 1;
                }
                group_index += 1;
            }
        } else {
            i += 1;
            if group_index < top_level_export_groups.len()
                && !top_level_export_groups[group_index].is_empty()
            {
                group_index += 1;
            }
        }
    }

    let mut result: Vec<Vec<Arc<Node>>> = Vec::new();
    for export_group in &top_level_export_groups {
        let sub_groups = crate::ls::mig::m5v_3::group_by_newline_contiguous(source_file, export_group);
        result.extend(sub_groups);
    }
    result
}

pub fn organize_exports_worker(
    old_export_decls: &[Arc<Node>],
    comparer: &OrganizeImportsComparerSettings,
    source_file: &Arc<SourceFile>,
    change_tracker: &mut ChangeTracker,
) {
    if old_export_decls.is_empty() {
        return;
    }

    let specifier_comparer_func =
        crate::ls::lsutil_organize_imports_comparers::get_named_import_specifier_comparer(
            &new_default_user_preferences_with_type_order(comparer.type_order),
            Some(comparer.named_import_comparer.clone()),
        );

    let new_export_decls = coalesce_exports_worker(
        old_export_decls,
        &specifier_comparer_func,
        &comparer.module_specifier_comparer,
        Some(source_file),
        change_tracker,
    );

    if !old_export_decls.is_empty() {
        if new_export_decls.is_empty() {
            change_tracker.delete_node_range(
                source_file,
                &old_export_decls[0],
                &old_export_decls[old_export_decls.len() - 1],
                crate::ls::change_tracker::LeadingTriviaOption::Exclude,
                crate::ls::change_tracker::TrailingTriviaOption::Include,
            );
        } else {
            for exp in &new_export_decls {
                change_tracker.add_emit_flags(
                    exp,
                    tsox_frontend::format::mig::m4o::EmitFlags::NO_LEADING_COMMENTS,
                );
            }

            let options = crate::ls::change_tracker::NodeOptions {
                leading_trivia_option: crate::ls::change_tracker::LeadingTriviaOption::Exclude,
                trailing_trivia_option: crate::ls::change_tracker::TrailingTriviaOption::Include,
                suffix: "\n".to_string(),
                ..Default::default()
            };

            let new_nodes: Vec<Arc<Node>> = new_export_decls.clone();
            change_tracker.replace_node_with_nodes(
                source_file,
                &old_export_decls[0],
                &new_nodes,
                Some(&options),
            );

            if old_export_decls.len() > 1 {
                for old in &old_export_decls[1..] {
                    change_tracker.delete(source_file, old);
                }
            }
        }
    }
}

pub fn coalesce_exports_worker(
    export_group: &[Arc<Node>],
    specifier_comparer: &StatementComparer,
    module_specifier_comparer: &StringComparer,
    source_file: Option<&Arc<SourceFile>>,
    change_tracker: &mut ChangeTracker,
) -> Vec<Arc<Node>> {
    if export_group.is_empty() {
        return export_group.to_vec();
    }

    let mut exports_by_module_specifier: HashMap<String, Vec<Arc<Node>>> = HashMap::new();
    let mut module_specifier_order: Vec<String> = Vec::new();

    for export_decl in export_group {
        let export = export_decl.as_export_declaration();
        let module_specifier = export
            .module_specifier
            .as_ref()
            .map(|m| m.text().to_string())
            .unwrap_or_default();
        if !exports_by_module_specifier.contains_key(&module_specifier) {
            module_specifier_order.push(module_specifier.clone());
        }
        exports_by_module_specifier
            .entry(module_specifier)
            .or_default()
            .push(Arc::clone(export_decl));
    }

    module_specifier_order.sort_by(|a, b| {
        if a.is_empty() && !b.is_empty() {
            return std::cmp::Ordering::Greater;
        }
        if !a.is_empty() && b.is_empty() {
            return std::cmp::Ordering::Less;
        }
        let ord = module_specifier_comparer(a, b);
        ord.cmp(&0)
    });

    let mut coalesced_exports: Vec<Arc<Node>> = Vec::new();
    let factory = NodeFactory::new();

    for module_specifier in &module_specifier_order {
        let group = &exports_by_module_specifier[module_specifier];

        let categorized = get_categorized_exports(group);

        if let Some(without_clause) = categorized.export_without_clause {
            coalesced_exports.push(without_clause);
        }

        for sub_group in [&categorized.named_exports, &categorized.type_only_exports] {
            if sub_group.is_empty() {
                continue;
            }

            let mut new_export_specifiers: Vec<Arc<Node>> = Vec::new();
            for export_decl in sub_group {
                let export_clause = export_decl.as_export_declaration().export_clause.clone();
                if let Some(export_clause) = export_clause {
                    if export_clause.kind == SyntaxKind::NamedExports {
                        let named_exports = export_clause.as_named_exports();
                        new_export_specifiers.extend(named_exports.elements.nodes.iter().cloned());
                    }
                }
            }

            new_export_specifiers.sort_by(|a, b| specifier_comparer(a, b).cmp(&0));

            let export_decl = sub_group[0].as_export_declaration();

            let mut updated_export_clause: Option<Arc<Node>> = None;
            if let Some(export_clause) = export_decl.export_clause.as_ref() {
                if export_clause.kind == SyntaxKind::NamedExports {
                    let named_exports = export_clause.as_named_exports();
                    let sorted_list = factory.new_node_list(new_export_specifiers.clone());
                    let updated = factory.update_named_exports(export_clause, &sorted_list);
                    updated_export_clause = Some(updated);

                    if let Some(sf) = source_file {
                        if !ast::node_is_synthesized(&export_clause)
                            && !tsox_frontend::format::mig::m4t_3::range_is_on_single_line(
                                export_clause.loc,
                                sf,
                            )
                        {
                            change_tracker.set_emit_flags(
                                updated_export_clause.as_ref().unwrap(),
                                tsox_frontend::format::mig::m4o::EmitFlags::MULTI_LINE,
                            );
                        }
                    }
                } else {
                    updated_export_clause = Some(export_clause.clone());
                }
            }

            let new_export_decl = factory.update_export_declaration(
                &sub_group[0],
                sub_group[0].modifiers().cloned(),
                export_decl.is_type_only,
                updated_export_clause,
                export_decl.module_specifier.clone(),
                export_decl.attributes.clone(),
            );
            coalesced_exports.push(new_export_decl);
        }
    }

    coalesced_exports
}

pub struct CategorizedExports {
    pub export_without_clause: Option<Arc<Node>>,
    pub named_exports: Vec<Arc<Node>>,
    pub type_only_exports: Vec<Arc<Node>>,
}

pub fn get_categorized_exports(export_group: &[Arc<Node>]) -> CategorizedExports {
    let mut export_without_clause: Option<Arc<Node>> = None;
    let mut named_exports: Vec<Arc<Node>> = Vec::new();
    let mut type_only_exports: Vec<Arc<Node>> = Vec::new();

    for export_decl in export_group {
        let export = export_decl.as_export_declaration();
        if export.export_clause.is_none() {
            if export_without_clause.is_none() {
                export_without_clause = Some(Arc::clone(export_decl));
            }
        } else if export.is_type_only {
            type_only_exports.push(Arc::clone(export_decl));
        } else {
            named_exports.push(Arc::clone(export_decl));
        }
    }

    CategorizedExports {
        export_without_clause,
        named_exports,
        type_only_exports,
    }
}
