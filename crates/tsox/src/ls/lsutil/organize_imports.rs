use std::cmp::Ordering;
use std::sync::Arc;

use crate::ast::{Node, SourceFile};
use crate::core::tristate::Tristate;
use crate::stringutil;
use crate::tspath;

use super::user_preferences::{
    OrganizeImportsCaseFirst, OrganizeImportsCollation, OrganizeImportsSort,
    OrganizeImportsTypeOrder, UserPreferences,
};

pub type StringComparer = Arc<dyn Fn(&str, &str) -> i32>;

pub type StatementComparer = Box<dyn Fn(&Arc<Node>, &Arc<Node>) -> i32>;

pub fn filter_import_declarations(statements: &[Arc<Node>]) -> Vec<Arc<Node>> {
    statements
        .iter()
        .filter(|stmt| stmt.kind == crate::ast::SyntaxKind::ImportDeclaration)
        .cloned()
        .collect()
}

pub fn get_detection_lists(
    preferences: &UserPreferences,
) -> (Vec<StringComparer>, Vec<OrganizeImportsTypeOrder>) {
    let comparers_to_test: Vec<StringComparer> =
        if preferences.organize_imports_sort != OrganizeImportsSort::Auto {
            vec![get_organize_imports_preset_string_comparer(
                preferences.organize_imports_sort,
            )]
        } else if !preferences.organize_imports_ignore_case.is_unknown() {
            vec![get_organize_imports_string_comparer(
                preferences,
                preferences.organize_imports_ignore_case.is_true(),
            )]
        } else {
            vec![
                get_organize_imports_string_comparer(preferences, true),
                get_organize_imports_string_comparer(preferences, false),
            ]
        };

    let type_orders_to_test =
        if preferences.organize_imports_type_order != OrganizeImportsTypeOrder::Auto {
            vec![preferences.organize_imports_type_order]
        } else {
            vec![
                OrganizeImportsTypeOrder::Last,
                OrganizeImportsTypeOrder::Inline,
                OrganizeImportsTypeOrder::First,
            ]
        };

    (comparers_to_test, type_orders_to_test)
}

pub fn resolve_organize_imports_sort(preferences: &UserPreferences) -> OrganizeImportsSort {
    if preferences.organize_imports_sort != OrganizeImportsSort::Auto {
        return preferences.organize_imports_sort;
    }

    if preferences.organize_imports_collation == OrganizeImportsCollation::Unicode {
        return match preferences.organize_imports_ignore_case {
            Tristate::True => OrganizeImportsSort::NaturalIgnoreCase,
            Tristate::False => OrganizeImportsSort::Natural,
            Tristate::Unknown => OrganizeImportsSort::Auto,
        };
    }

    match preferences.organize_imports_ignore_case {
        Tristate::True => OrganizeImportsSort::OrdinalIgnoreCase,
        Tristate::False => OrganizeImportsSort::Ordinal,
        Tristate::Unknown => OrganizeImportsSort::Auto,
    }
}

fn get_organize_imports_ordinal_string_comparer(ignore_case: bool) -> StringComparer {
    if ignore_case {
        Arc::new(|a: &str, b: &str| stringutil::compare_strings_case_insensitive(a, b))
    } else {
        Arc::new(|a: &str, b: &str| stringutil::compare_strings_case_sensitive(a, b))
    }
}

fn get_organize_imports_natural_string_comparer(case_sensitive: bool) -> StringComparer {
    Arc::new(move |a: &str, b: &str| compare_organize_imports_natural_strings(a, b, case_sensitive))
}

fn get_organize_imports_unicode_string_comparer(
    ignore_case: bool,
    preferences: &UserPreferences,
) -> StringComparer {
    let case_first = preferences.organize_imports_case_first;
    let numeric = preferences.organize_imports_numeric_collation.is_true();
    let accents = !preferences.organize_imports_accent_collation.is_false();
    Arc::new(move |a: &str, b: &str| {
        compare_organize_imports_unicode_strings(a, b, ignore_case, case_first, numeric, accents)
    })
}

fn compare_organize_imports_natural_strings(a: &str, b: &str, case_sensitive: bool) -> i32 {
    let ord = compare_strings_numeric(&natural_collation_key(a), &natural_collation_key(b));
    if ord != 0 {
        return ord;
    }

    if case_sensitive {
        let ord = compare_organize_imports_case_upper_first(a, b);
        if ord != 0 {
            return ord;
        }
    }

    ord_to_i32(a.cmp(b))
}

#[allow(clippy::too_many_arguments)]
fn compare_organize_imports_unicode_strings(
    a: &str,
    b: &str,
    ignore_case: bool,
    case_first: OrganizeImportsCaseFirst,
    numeric: bool,
    accents: bool,
) -> i32 {
    let ord = compare_organize_imports_unicode_keys(
        &natural_collation_key(a),
        &natural_collation_key(b),
        numeric,
    );
    if ord != 0 {
        return ord;
    }

    if accents {
        let ord = compare_organize_imports_unicode_keys(
            &a.to_ascii_lowercase(),
            &b.to_ascii_lowercase(),
            numeric,
        );
        if ord != 0 {
            return ord;
        }
    }

    if !ignore_case {
        let ord = compare_organize_imports_case(a, b, case_first);
        if ord != 0 {
            return ord;
        }
    }

    ord_to_i32(a.cmp(b))
}

fn natural_collation_key(s: &str) -> String {
    s.to_ascii_lowercase()
        .chars()
        .filter(|&c| !is_combining_mark(c))
        .collect()
}

fn is_combining_mark(c: char) -> bool {
    let cp = c as u32;
    (0x0300..=0x036F).contains(&cp)
        || (0x0483..=0x0489).contains(&cp)
        || (0x0591..=0x05BD).contains(&cp)
        || cp == 0x05BF
        || (0x05C1..=0x05C2).contains(&cp)
        || (0x05C4..=0x05C5).contains(&cp)
        || cp == 0x05C7
        || (0x0610..=0x061A).contains(&cp)
        || (0x064B..=0x065F).contains(&cp)
        || cp == 0x0670
        || (0x06D6..=0x06DC).contains(&cp)
        || (0x06DF..=0x06E4).contains(&cp)
        || (0x06E7..=0x06E8).contains(&cp)
        || (0x06EA..=0x06ED).contains(&cp)
        || cp == 0x0711
        || (0x0730..=0x074A).contains(&cp)
}

fn compare_organize_imports_unicode_keys(a: &str, b: &str, numeric: bool) -> i32 {
    if numeric {
        compare_strings_numeric(a, b)
    } else {
        ord_to_i32(a.cmp(b))
    }
}

fn compare_strings_numeric(a: &str, b: &str) -> i32 {
    let mut a = a;
    let mut b = b;
    while !a.is_empty() && !b.is_empty() {
        let a0 = a.as_bytes()[0];
        let b0 = b.as_bytes()[0];
        if is_ascii_digit(a0) && is_ascii_digit(b0) {
            let a_run_end = ascii_digit_run_end(a);
            let b_run_end = ascii_digit_run_end(b);
            let ord = compare_numeric_text(&a[..a_run_end], &b[..b_run_end]);
            if ord != 0 {
                return ord;
            }
            a = &a[a_run_end..];
            b = &b[b_run_end..];
            continue;
        }

        let (a_rune, a_size) = next_rune(a);
        let (b_rune, b_size) = next_rune(b);
        if a_rune != b_rune {
            return cmp_compare_i32(a_rune as i32, b_rune as i32);
        }
        a = &a[a_size..];
        b = &b[b_size..];
    }

    cmp_compare_i32(a.len() as i32, b.len() as i32)
}

fn is_ascii_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

fn ascii_digit_run_end(s: &str) -> usize {
    s.bytes().take_while(|c| is_ascii_digit(*c)).count()
}

fn compare_numeric_text(a: &str, b: &str) -> i32 {
    let mut a_digits = a.trim_start_matches('0');
    let mut b_digits = b.trim_start_matches('0');
    if a_digits.is_empty() {
        a_digits = "0";
    }
    if b_digits.is_empty() {
        b_digits = "0";
    }

    if a_digits.len() != b_digits.len() {
        return cmp_compare_i32(a_digits.len() as i32, b_digits.len() as i32);
    }
    let ord = a_digits.cmp(b_digits);
    if ord != Ordering::Equal {
        return ord_to_i32(ord);
    }
    ord_to_i32(a.cmp(b))
}

fn compare_organize_imports_case_upper_first(a: &str, b: &str) -> i32 {
    compare_organize_imports_case(a, b, OrganizeImportsCaseFirst::Upper)
}

fn compare_organize_imports_case(a: &str, b: &str, case_first: OrganizeImportsCaseFirst) -> i32 {
    let a_runes: Vec<char> = a.chars().collect();
    let b_runes: Vec<char> = b.chars().collect();
    let min_len = a_runes.len().min(b_runes.len());

    for i in 0..min_len {
        let a_upper = a_runes[i].is_uppercase();
        let b_upper = b_runes[i].is_uppercase();
        if a_upper != b_upper {
            return match case_first {
                OrganizeImportsCaseFirst::Upper => {
                    if a_upper {
                        -1
                    } else {
                        1
                    }
                }
                OrganizeImportsCaseFirst::Lower => {
                    if !a_upper {
                        -1
                    } else {
                        1
                    }
                }
                OrganizeImportsCaseFirst::False => {
                    if a_upper {
                        1
                    } else {
                        -1
                    }
                }
            };
        }
    }

    cmp_compare_i32(a_runes.len() as i32, b_runes.len() as i32)
}

fn get_organize_imports_preset_string_comparer(sort: OrganizeImportsSort) -> StringComparer {
    match sort {
        OrganizeImportsSort::OrdinalIgnoreCase => {
            get_organize_imports_ordinal_string_comparer(true)
        }
        OrganizeImportsSort::Natural => get_organize_imports_natural_string_comparer(true),
        OrganizeImportsSort::NaturalIgnoreCase => {
            get_organize_imports_natural_string_comparer(false)
        }
        _ => get_organize_imports_ordinal_string_comparer(false),
    }
}

fn get_organize_imports_string_comparer(
    preferences: &UserPreferences,
    ignore_case: bool,
) -> StringComparer {
    if preferences.organize_imports_sort != OrganizeImportsSort::Auto {
        return get_organize_imports_preset_string_comparer(preferences.organize_imports_sort);
    }
    if preferences.organize_imports_collation == OrganizeImportsCollation::Unicode {
        return get_organize_imports_unicode_string_comparer(ignore_case, preferences);
    }
    get_organize_imports_ordinal_string_comparer(ignore_case)
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
        crate::ast::SyntaxKind::ImportDeclaration => {

            IMPORT_KIND_ORDER_NAMED
        }
        crate::ast::SyntaxKind::ImportEqualsDeclaration => IMPORT_KIND_ORDER_IMPORT_EQUALS,

        _ => IMPORT_KIND_ORDER_UNKNOWN,
    }
}

pub fn get_named_import_specifier_comparer(
    preferences: &UserPreferences,
    comparer: Option<StringComparer>,
) -> StatementComparer {
    let cmp = match comparer {
        Some(c) => c,
        None => {
            let ignore_case = if !preferences.organize_imports_ignore_case.is_unknown() {
                preferences.organize_imports_ignore_case.is_true()
            } else {
                false
            };
            get_organize_imports_string_comparer(preferences, ignore_case)
        }
    };
    let type_order = preferences.organize_imports_type_order;
    Box::new(move |s1: &Arc<Node>, s2: &Arc<Node>| {
        compare_import_or_export_specifiers(s1, s2, &cmp, type_order)
    })
}

fn compare_import_or_export_specifiers(
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
        .map(|import_group| {
            import_group
                .iter()
                .map(|_decl| String::new())
                .collect()
        })
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

pub fn get_organize_imports_string_comparer_with_detection(
    _original_import_decls: &[Arc<Node>],
    preferences: &UserPreferences,
) -> (StringComparer, bool) {

    let comparer = get_comparers(preferences)
        .into_iter()
        .next()
        .unwrap_or_else(|| get_organize_imports_string_comparer(preferences, false));
    (comparer, false)
}

fn get_comparers(preferences: &UserPreferences) -> Vec<StringComparer> {
    if preferences.organize_imports_sort != OrganizeImportsSort::Auto
        || !preferences.organize_imports_ignore_case.is_unknown()
    {
        let ignore_case = if !preferences.organize_imports_ignore_case.is_unknown() {
            preferences.organize_imports_ignore_case.is_true()
        } else {
            false
        };
        vec![get_organize_imports_string_comparer(
            preferences,
            ignore_case,
        )]
    } else {
        vec![
            get_organize_imports_string_comparer(preferences, true),
            get_organize_imports_string_comparer(preferences, false),
        ]
    }
}

pub fn get_named_import_specifier_comparer_with_detection(
    _import_decl: &Arc<Node>,
    _source_file: Option<&SourceFile>,
    preferences: &UserPreferences,
) -> (StatementComparer, Tristate) {
    let (comparers_to_test, type_orders_to_test) = get_detection_lists(preferences);
    let specifier_comparer =
        get_named_import_specifier_comparer(preferences, comparers_to_test.into_iter().next());

    let _ = type_orders_to_test;
    (specifier_comparer, Tristate::Unknown)
}

fn compare_booleans(a: bool, b: bool) -> i32 {
    cmp_compare_i32(a as i32, b as i32)
}

fn cmp_compare_i32(a: i32, b: i32) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

fn ord_to_i32(ord: Ordering) -> i32 {
    match ord {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn next_rune(s: &str) -> (char, usize) {
    match s.chars().next() {
        Some(c) => (c, c.len_utf8()),
        None => ('\0', 0),
    }
}
