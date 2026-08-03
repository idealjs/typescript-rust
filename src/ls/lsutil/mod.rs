//! Language-service shared utilities (1:1 port of Go's `internal/ls/lsutil/`).
//!
//! This module groups the helper types and functions used across the language
//! service: user preferences, format-code options, organize-imports logic, ASI
//! helpers, child-node iteration, completed-node detection, and symbol display.
//!
//! These are skeleton ports: type definitions and self-contained logic are
//! ported in full, while bodies that depend on not-yet-ported infrastructure
//! (scanner, printer, checker, AST node accessors) are stubbed with `todo!()` /
//! conservative defaults and marked `// TODO`.

#![allow(dead_code)]

pub mod asi;
pub mod children;
pub mod completed_node;
pub mod format_code_options;
pub mod organize_imports;
pub mod symbol_display;
pub mod user_preferences;
pub mod utilities;

pub use format_code_options::{
    EditorSettings, FormatCodeSettings, IndentStyle, SemicolonPreference, from_ls_format_options,
    get_default_format_code_settings, to_ls_format_options,
};
pub use user_preferences::{
    CodeLensUserPreferences, IncludeInlayParameterNameHints, InlayHintsPreferences,
    JsxAttributeCompletionStyle, OrganizeImportsCaseFirst, OrganizeImportsCollation,
    OrganizeImportsSort, OrganizeImportsTypeOrder, QuotePreference, UserPreferences,
    new_default_user_preferences, parse_user_preferences,
};
