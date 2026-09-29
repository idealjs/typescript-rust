#![allow(unused_imports, dead_code)]

use std::io::Write;

use tsox_core::locale::Locale;
use tsox_tsoptions::tsoptions::{OptValue, OptionDecl, OptionKind};

use super::m5a_4::{Colors, create_colors};
use crate::execute::System;

pub fn generate_option_output(
    sys: &dyn System,
    locale: &Locale,
    option: &OptionDecl,
    right_align_of_left: usize,
    left_align_of_right: usize,
) -> Vec<String> {
    let mut text: Vec<String> = Vec::new();
    let colors = create_colors(sys);

    let name = get_display_name_text_of_option(option);
    let value_candidates = get_value_candidate(sys, locale, option);

    let default_value_description = format_default_value(
        None,
        if option.kind == OptionKind::List {
            option.elements()
        } else {
            Some(option)
        },
    );

    let terminal_width = sys.width_of_terminal();

    if terminal_width >= 80 {
        let description: String = option.description.to_string();
        text.extend(get_pretty_output(
            &colors,
            &name,
            &description,
            right_align_of_left,
            left_align_of_right,
            terminal_width,
            true,
        ));
        text.push("\n".to_string());
        if show_additional_info_output(value_candidates.as_ref(), option) {
            if let Some(candidate) = &value_candidates {
                text.extend(get_pretty_output(
                    &colors,
                    &candidate.value_type,
                    &candidate.possible_values,
                    right_align_of_left,
                    left_align_of_right,
                    terminal_width,
                    false,
                ));
                text.push("\n".to_string());
            }
            if !default_value_description.is_empty() {
                text.extend(get_pretty_output(
                    &colors,
                    &tsox_core::diagnostics::X_DEFAULT_COLON.localize(locale, &[]),
                    &default_value_description,
                    right_align_of_left,
                    left_align_of_right,
                    terminal_width,
                    false,
                ));
                text.push("\n".to_string());
            }
        }
        text.push("\n".to_string());
    } else {
        text.push(colors.blue(&name));
        text.push("\n".to_string());
        text.push(option.description.to_string());
        text.push("\n".to_string());
        if show_additional_info_output(value_candidates.as_ref(), option) {
            if let Some(candidate) = &value_candidates {
                text.push(candidate.value_type.clone());
                text.push(" ".to_string());
                text.push(candidate.possible_values.clone());
            }
            if !default_value_description.is_empty() {
                if value_candidates.is_some() {
                    text.push("\n".to_string());
                }
                text.push(tsox_core::diagnostics::X_DEFAULT_COLON.localize(locale, &[]));
                text.push(" ".to_string());
                text.push(default_value_description.clone());
            }
            text.push("\n".to_string());
        }
        text.push("\n".to_string());
    }
    text
}

pub fn format_default_value(
    default_value: Option<&dyn std::fmt::Debug>,
    option: Option<&OptionDecl>,
) -> String {
    let Some(option) = option else {
        return "undefined".to_string();
    };
    let Some(default_value) = default_value else {
        return "undefined".to_string();
    };
    if option.kind == OptionKind::Enum {
        let mut names: Vec<&str> = Vec::new();
        if let Some(enum_values) = option.enum_values {
            let rendered = format!("{default_value:?}").trim_matches('"').to_string();
            for name in enum_values {
                if *name == rendered {
                    names.push(name);
                }
            }
        }
        return names.join("/");
    }
    format!("{default_value:?}")
}

pub struct ValueCandidate {
    pub value_type: String,
    pub possible_values: String,
}

pub fn show_additional_info_output(
    value_candidates: Option<&ValueCandidate>,
    option: &OptionDecl,
) -> bool {
    if option.is_command_line_only {
        return false;
    }
    if let Some(candidate) = value_candidates {
        if candidate.possible_values == "string" {
            return false;
        }
    }
    true
}

pub fn get_value_candidate(
    _sys: &dyn System,
    locale: &Locale,
    option: &OptionDecl,
) -> Option<ValueCandidate> {
    if option.kind == OptionKind::ListOrElement {
        panic!("no value candidate for list or element");
    }

    let value_type = match option.kind {
        OptionKind::String | OptionKind::Number | OptionKind::Boolean => {
            tsox_core::diagnostics::X_TYPE_COLON.localize(locale, &[])
        }
        OptionKind::List => tsox_core::diagnostics::X_ONE_OR_MORE_COLON.localize(locale, &[]),
        _ => tsox_core::diagnostics::X_ONE_OF_COLON.localize(locale, &[]),
    };

    Some(ValueCandidate {
        value_type,
        possible_values: get_possible_values(option),
    })
}

pub fn get_possible_values(option: &OptionDecl) -> String {
    match option.kind {
        OptionKind::String => "string".to_string(),
        OptionKind::Number => "number".to_string(),
        OptionKind::Boolean => "boolean".to_string(),
        OptionKind::List | OptionKind::ListOrElement => match option.elements() {
            Some(elem) => get_possible_values(elem),
            None => String::new(),
        },
        _ => {
            let enum_map = match option.enum_map() {
                Some(enum_map) => enum_map,
                None => return String::new(),
            };
            let deprecated_keys = option.deprecated_keys();
            let mut inverted: Vec<(&OptValue, Vec<String>)> = Vec::new();
            for (name, value) in enum_map.iter() {
                if deprecated_keys.map(|keys| !keys.has(name)).unwrap_or(true) {
                    if let Some(entry) = inverted.iter_mut().find(|(v, _)| opt_value_eq(v, value)) {
                        entry.1.push(name.to_string());
                    } else {
                        inverted.push((value, vec![name.to_string()]));
                    }
                }
            }
            let syns: Vec<String> = inverted
                .into_iter()
                .map(|(_, synonyms)| synonyms.join("/"))
                .collect();
            syns.join(", ")
        }
    }
}

fn opt_value_eq(a: &OptValue, b: &OptValue) -> bool {
    match (a, b) {
        (OptValue::Bool(x), OptValue::Bool(y)) => x == y,
        (OptValue::Str(x), OptValue::Str(y)) => x == y,
        (OptValue::Num(x), OptValue::Num(y)) => x == y,
        (OptValue::List(x), OptValue::List(y)) => x == y,
        (OptValue::Null, OptValue::Null) => true,
        _ => false,
    }
}

pub fn get_pretty_output(
    colors: &Colors,
    left: &str,
    right: &str,
    right_align_of_left: usize,
    left_align_of_right: usize,
    terminal_width: usize,
    color_left: bool,
) -> Vec<String> {
    let mut res: Vec<String> = Vec::with_capacity(4);
    let mut is_first_line = true;
    let mut remain_right = right;
    let right_character_number = terminal_width.saturating_sub(left_align_of_right);
    while !remain_right.is_empty() {
        let cur_left = if is_first_line {
            let aligned_left = format!("{left:>right_align_of_left$}");
            let padded = format!("{aligned_left:<left_align_of_right$}");
            if color_left {
                colors.blue(&padded)
            } else {
                padded
            }
        } else {
            " ".repeat(left_align_of_right)
        };

        let idx = right_character_number.min(remain_right.len());
        let (cur_right, rest) = remain_right.split_at(idx);
        remain_right = rest;
        res.push(cur_left);
        res.push(cur_right.to_string());
        res.push("\n".to_string());
        is_first_line = false;
    }
    res
}

pub fn get_display_name_text_of_option(option: &OptionDecl) -> String {
    match option.short_name {
        Some(short) if !short.is_empty() => format!("--{}, -{short}", option.name),
        _ => format!("--{}", option.name),
    }
}
