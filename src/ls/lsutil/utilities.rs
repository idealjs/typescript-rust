//! Shared language-service utilities.
//!
//! Ported from `internal/ls/lsutil/utilities.go`. These functions inspect the
//! AST and scanner state (quote usage, semicolon usage, node-core-module
//! style); they are stubbed until the relevant AST accessors and scanner
//! helpers are ported.

use crate::ast::{Node, SourceFile, SyntaxKind};
use crate::core::tristate::Tristate;

use super::user_preferences::{QuotePreference, UserPreferences};

/// Guesses whether `file` is written in a semicolon-using style.
///
/// Mirrors `ProbablyUsesSemicolons` in Go. Requires `ForEachChild` +
/// `GetLastToken` + scanner line helpers; stubbed to a conservative `true`.
pub fn probably_uses_semicolons(_file: &SourceFile) -> bool {
    // TODO: port the full statement-observation visitor.
    true
}

/// Determines whether `file` uses `node:`-prefixed (URI-style) core modules.
///
/// Mirrors `ShouldUseUriStyleNodeCoreModules` in Go.
pub fn should_use_uri_style_node_core_modules(
    _file: &SourceFile,
    _program: &crate::compiler::Program,
) -> Tristate {
    // TODO: requires core.NodeCoreModules + program.UsesUriStyleNodeCoreModules.
    Tristate::Unknown
}

/// Returns the quote preference implied by a string-literal node.
///
/// Mirrors `QuotePreferenceFromString` in Go.
pub fn quote_preference_from_string(str_node: &Node) -> QuotePreference {
    // TODO: requires TokenFlags access (TokenFlagsSingleQuote). Stubbed.
    let _ = str_node;
    QuotePreference::Double
}

/// Returns the quote preference for `source_file`, considering `preferences`.
///
/// Mirrors `GetQuotePreference` in Go.
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
    // TODO: detect from first non-synthetic module specifier.
    QuotePreference::Double
}

/// Converts a module symbol's name to a valid identifier.
///
/// Mirrors `ModuleSymbolToValidIdentifier` in Go.
pub fn module_symbol_to_valid_identifier(
    module_symbol: &crate::ast::Symbol,
    force_capitalize: bool,
) -> String {
    module_specifier_to_valid_identifier(&strip_quotes(&module_symbol.name), force_capitalize)
}

/// Converts a module specifier string to a valid identifier.
///
/// Mirrors `ModuleSpecifierToValidIdentifier` in Go.
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

/// Whether `token` is a keyword that is not a contextual keyword.
///
/// Mirrors `IsNonContextualKeyword` in Go.
pub fn is_non_contextual_keyword(token: Option<SyntaxKind>) -> bool {
    // TODO: requires ast.IsKeywordKind + ast.IsContextualKeyword, neither of
    // which is ported. Conservative stub: treat any recognized keyword as
    // non-contextual.
    token.is_some()
}

/// Whether a character may start an identifier (mirrors the scanner's private
/// `isIdentifierStart`).
fn is_identifier_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || c == '_'
        || c == '$'
        || (!c.is_ascii() && unicode_ident::is_xid_start(c))
}

/// Whether a character may continue an identifier.
fn is_identifier_part(c: char) -> bool {
    crate::scanner::is_identifier_part(c)
}

/// Strip surrounding quotes from a string (mirrors `stringutil.StripQuotes`).
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
