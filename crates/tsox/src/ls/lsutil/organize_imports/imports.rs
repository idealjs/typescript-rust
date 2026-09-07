use super::compare_strings::{cmp_compare_i32, compare_booleans};
use super::comparers::{StatementComparer, StringComparer};
use crate::ast::Node;
use crate::tspath;
use std::sync::Arc;

use super::super::user_preferences::OrganizeImportsTypeOrder;

pub fn filter_import_declarations(statements: &[Arc<Node>]) -> Vec<Arc<Node>> {
    statements
        .iter()
        .filter(|stmt| stmt.kind == crate::ast::SyntaxKind::ImportDeclaration)
        .cloned()
        .collect()
}

pub fn get_external_module_name(specifier: Option<&Arc<Node>>) -> String {
    let _ = specifier;
    String::new()
}

pub fn compare_module_specifiers(
    m1: Option<&Arc<Node>>,
    m2: Option<&Arc<Node>>,
    comparer: &StringComparer,
) -> i32 {
    let name1 = get_external_module_name(m1);
    let name2 = get_external_module_name(m2);
    let ord = compare_booleans(name1.is_empty(), name2.is_empty());
    if ord != 0 {
        return ord;
    }
    let ord = compare_booleans(
        tspath::is_external_module_name_relative(&name1),
        tspath::is_external_module_name_relative(&name2),
    );
    if ord != 0 {
        return ord;
    }
    comparer(&name1, &name2)
}

pub fn compare_imports_or_require_statements(
    s1: &Arc<Node>,
    s2: &Arc<Node>,
    comparer: &StringComparer,
) -> i32 {
    let ord = compare_module_specifiers(None, None, comparer);
    if ord != 0 {
        return ord;
    }
    compare_import_kind(s1, s2)
}

fn compare_import_kind(s1: &Arc<Node>, s2: &Arc<Node>) -> i32 {
    cmp_compare_i32(get_import_kind_order(s1), get_import_kind_order(s2))
}

const IMPORT_KIND_ORDER_SIDE_EFFECT: i32 = 0;
const IMPORT_KIND_ORDER_TYPE_ONLY: i32 = 1;
const IMPORT_KIND_ORDER_NAMESPACE: i32 = 2;
const IMPORT_KIND_ORDER_DEFAULT: i32 = 3;
const IMPORT_KIND_ORDER_NAMED: i32 = 4;
const IMPORT_KIND_ORDER_IMPORT_EQUALS: i32 = 5;
const IMPORT_KIND_ORDER_REQUIRE: i32 = 6;
const IMPORT_KIND_ORDER_UNKNOWN: i32 = 7;

fn get_import_kind_order(s1: &Arc<Node>) -> i32 {
    match s1.kind {
        crate::ast::SyntaxKind::ImportDeclaration => IMPORT_KIND_ORDER_NAMED,
        crate::ast::SyntaxKind::ImportEqualsDeclaration => IMPORT_KIND_ORDER_IMPORT_EQUALS,

        _ => IMPORT_KIND_ORDER_UNKNOWN,
    }
}

pub(super) fn compare_import_or_export_specifiers(
    _s1: &Arc<Node>,
    _s2: &Arc<Node>,
    comparer: &StringComparer,
    type_order: OrganizeImportsTypeOrder,
) -> i32 {
    let s1_name = String::new();
    let s2_name = String::new();
    let s1_type_only = false;
    let s2_type_only = false;
    match type_order {
        OrganizeImportsTypeOrder::First => {
            let ord = compare_booleans(s2_type_only, s1_type_only);
            if ord != 0 {
                return ord;
            }
            comparer(&s1_name, &s2_name)
        }
        OrganizeImportsTypeOrder::Inline => comparer(&s1_name, &s2_name),

        _ => {
            let ord = compare_booleans(s1_type_only, s2_type_only);
            if ord != 0 {
                return ord;
            }
            comparer(&s1_name, &s2_name)
        }
    }
}

pub fn get_import_specifier_insertion_index(
    _sorted_imports: &[Arc<Node>],
    _new_import: &Arc<Node>,
    _comparer: &StatementComparer,
) -> usize {
    0
}

pub fn get_import_declaration_insert_index(
    _sorted_imports: &[Arc<Node>],
    _new_import: &Arc<Node>,
    _comparer: &dyn Fn(&Arc<Node>, &Arc<Node>) -> i32,
) -> usize {
    0
}

struct CaseSensitivityDetectionResult {
    comparer: Option<StringComparer>,
    is_sorted: bool,
}

pub fn detect_module_specifier_case_by_sort(
    import_decls_by_group: &[Vec<Arc<Node>>],
    comparers_to_test: &[StringComparer],
) -> (Option<StringComparer>, bool) {
    let module_specifiers_by_group: Vec<Vec<String>> = import_decls_by_group
        .iter()
        .map(|import_group| import_group.iter().map(|_decl| String::new()).collect())
        .collect();
    let result = detect_case_sensitivity_by_sort(&module_specifiers_by_group, comparers_to_test);
    (result.comparer, result.is_sorted)
}

fn detect_case_sensitivity_by_sort(
    original_groups: &[Vec<String>],
    comparers_to_test: &[StringComparer],
) -> CaseSensitivityDetectionResult {
    let mut best_index: Option<usize> = None;
    let mut best_diff = i32::MAX;

    for (i, cur_comparer) in comparers_to_test.iter().enumerate() {
        let mut diff_of_current_comparer = 0;
        for list_to_sort in original_groups {
            if list_to_sort.len() <= 1 {
                continue;
            }
            diff_of_current_comparer += measure_sortedness(list_to_sort, cur_comparer);
        }
        if diff_of_current_comparer < best_diff {
            best_diff = diff_of_current_comparer;
            best_index = Some(i);
        }
    }

    let comparer = best_index
        .or(if comparers_to_test.is_empty() {
            None
        } else {
            Some(0)
        })
        .map(|i| Arc::clone(&comparers_to_test[i]));

    CaseSensitivityDetectionResult {
        comparer,
        is_sorted: best_diff == 0,
    }
}

fn measure_sortedness(arr: &[String], comparer: &StringComparer) -> i32 {
    let mut count = 0i32;
    for j in 0..arr.len().saturating_sub(1) {
        if comparer(&arr[j], &arr[j + 1]) > 0 {
            count += 1;
        }
    }
    count
}
