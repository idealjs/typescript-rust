#![allow(unused_imports)]
#![allow(dead_code)]

use crate::format::FormatCodeSettings;
use crate::format::Tristate;

pub fn insert_space_after_comma_delimiter_option(options: &FormatCodeSettings) -> Tristate {
    options.insert_space_after_comma_delimiter
}

pub fn insert_space_after_constructor_option(options: &FormatCodeSettings) -> Tristate {
    options.insert_space_after_constructor
}

pub fn insert_space_after_keywords_in_control_flow_statements_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_keywords_in_control_flow_statements
}

pub fn insert_space_after_function_keyword_for_anonymous_functions_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_function_keyword_for_anonymous_functions
}

pub fn insert_space_after_opening_and_before_closing_nonempty_braces_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_opening_and_before_closing_nonempty_braces
}

pub fn insert_space_after_opening_and_before_closing_empty_braces_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_opening_and_before_closing_empty_braces
}

pub fn insert_space_after_opening_and_before_closing_jsx_expression_braces_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.insert_space_after_opening_and_before_closing_jsx_expression_braces
}

pub fn indent_switch_case_option(options: &FormatCodeSettings) -> Tristate {
    options.indent_switch_case
}

pub fn indent_multi_line_object_literal_beginning_on_blank_line_option(
    options: &FormatCodeSettings,
) -> Tristate {
    options.indent_multi_line_object_literal_beginning_on_blank_line
}
