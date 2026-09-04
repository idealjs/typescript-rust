use crate::ast::{Node, SourceFile, SyntaxKind};
use crate::core::tristate::Tristate;

use super::user_preferences::{QuotePreference, UserPreferences};

pub fn probably_uses_semicolons(_file: &SourceFile) -> bool {

    true
}

pub fn should_use_uri_style_node_core_modules(
    _file: &SourceFile,
    _program: &crate::compiler::Program,
) -> Tristate {

    Tristate::Unknown
}

pub fn quote_preference_from_string(str_node: &Node) -> QuotePreference {

    let _ = str_node;
    QuotePreference::Double
}

pub fn get_quote_preference(
    _source_file: &SourceFile,
    preferences: &UserPreferences,
) -> QuotePreference {
    if preferences.quote_preference != QuotePreference::Unknown
        && preferences.quote_preference != QuotePreference::Auto
    {
        return if preferences.quote_preference == QuotePreference::Single {
            QuotePreference::Single
        } else {
            QuotePreference::Double
        };
    }

    QuotePreference::Double
}

pub fn module_symbol_to_valid_identifier(
    module_symbol: &crate::ast::Symbol,
    force_capitalize: bool,
) -> String {
    module_specifier_to_valid_identifier(&strip_quotes(&module_symbol.name), force_capitalize)
}

pub fn module_specifier_to_valid_identifier(
    module_specifier: &str,
    force_capitalize: bool,
) -> String {
    let base_name = crate::tspath::get_base_file_name(
        &module_specifier
            .strip_suffix("/index")
            .unwrap_or(module_specifier)
            .rsplit_once('.')
            .map(|(stem, _)| stem)
            .unwrap_or(module_specifier),
    );
    let base_name_chars: Vec<char> = base_name.chars().collect();
    let mut res: Vec<char> = Vec::new();
    let mut last_char_was_valid = true;

    if let Some(&first) = base_name_chars.first() {
        if is_identifier_start(first) {
            if force_capitalize {
                res.push(first.to_ascii_uppercase());
            } else {
                res.push(first);
            }
        } else {
            last_char_was_valid = false;
        }
    }

    for &c in base_name_chars.iter().skip(1) {
        let is_valid = is_identifier_part(c);
        if is_valid {
            if !last_char_was_valid {
                res.push(c.to_ascii_uppercase());
            } else {
                res.push(c);
            }
        }
        last_char_was_valid = is_valid;
    }

    let res_string: String = res.into_iter().collect();
    if !res_string.is_empty()
        && !is_non_contextual_keyword(crate::scanner::string_to_keyword(&res_string))
    {
        res_string
    } else {
        format!("_{res_string}")
    }
}

pub fn is_non_contextual_keyword(token: Option<SyntaxKind>) -> bool {

    token.is_some()
}

fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && unicode_ident::is_xid_start(c))
}

fn is_identifier_part(c: char) -> bool {
    crate::scanner::is_identifier_part(c)
}

fn strip_quotes(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}
