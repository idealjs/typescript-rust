use super::compare_strings::{
    compare_organize_imports_natural_strings, compare_organize_imports_unicode_strings,
};
use crate::ast::Node;
use crate::core::tristate::Tristate;
use crate::stringutil;
use std::sync::Arc;

use super::super::user_preferences::{
    OrganizeImportsCollation, OrganizeImportsSort, OrganizeImportsTypeOrder, UserPreferences,
};

pub type StringComparer = Arc<dyn Fn(&str, &str) -> i32>;

pub type StatementComparer = Box<dyn Fn(&Arc<Node>, &Arc<Node>) -> i32>;

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

pub(super) fn get_organize_imports_string_comparer(
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
        super::imports::compare_import_or_export_specifiers(s1, s2, &cmp, type_order)
    })
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
    _source_file: Option<&crate::ast::SourceFile>,
    preferences: &UserPreferences,
) -> (StatementComparer, Tristate) {
    let (comparers_to_test, type_orders_to_test) = get_detection_lists(preferences);
    let specifier_comparer =
        get_named_import_specifier_comparer(preferences, comparers_to_test.into_iter().next());

    let _ = type_orders_to_test;
    (specifier_comparer, Tristate::Unknown)
}
